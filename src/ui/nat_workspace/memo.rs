//! NAT-only, optimization-only cache. No saved grading credential.
use super::*;
use serde::{
    Serialize,
    ser::{self, SerializeMap, SerializeSeq, SerializeStruct},
};
use std::fmt;

pub(in crate::ui) const ENTRY_BYTES: usize = 4 * 1024 * 1024;
pub(in crate::ui) const TOTAL_BYTES: usize = 16 * 1024 * 1024;
const ENTRIES: usize = 16;
// Within the aggregate: <=1000 bounded activity IDs/booleans + one fallback report.
const FRAME_RESERVE: usize = 512 * 1024;
const FRAME_IDS: usize = 1000;
const REPORT_RESERVE: usize = 32 * 1024 + 128 * 128;
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) struct Counters {
    pub evaluations: usize,
    pub meters: usize,
    pub snapshots: usize,
    pub hits: usize,
    pub fallbacks: usize,
}
struct Entry {
    id: String,
    initial: core::State,
    definition: work::Definition,
    lesson: Option<dlms_learning_lab::nat_teaching::Lesson>,
    practice_rows: Vec<(bool, String)>,
    attempt: Attempt,
    rows: Vec<(bool, String)>,
    charge: usize,
    touched: u64,
}
#[derive(Default)]
pub(in crate::ui) struct Memo {
    entries: Vec<Entry>,
    bytes: usize,
    tick: u64,
    pub counters: Counters,
    // Frame-local booleans only. Uncacheable inputs never become owned snapshots.
    frame: std::collections::BTreeMap<String, bool>,
    fallback_rows: Option<(String, Vec<(bool, String)>)>,
    fallback_practice: Option<(String, Vec<(bool, String)>)>,
}
impl Memo {
    pub fn begin_frame(&mut self) {
        self.frame.clear();
        self.fallback_rows = None;
        self.fallback_practice = None;
    }
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.entries.retain(|e| keep(&e.id));
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
    }
    pub fn prepared(&self, id: &str) -> Option<bool> {
        self.frame.get(id).copied()
    }
    pub fn rows(&self, id: &str) -> Option<&[(bool, String)]> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.rows.as_slice())
            .or_else(|| {
                self.fallback_rows
                    .as_ref()
                    .filter(|(key, _)| key == id)
                    .map(|(_, r)| r.as_slice())
            })
    }
    pub fn practice_rows(&self, id: &str) -> Option<&[(bool, String)]> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.practice_rows.as_slice())
            .or_else(|| {
                self.fallback_practice
                    .as_ref()
                    .filter(|(k, _)| k == id)
                    .map(|(_, r)| r.as_slice())
            })
    }
    pub fn evaluate(&mut self, a: &Attempt, s: &Scenario, force: bool) {
        self.evaluate_limits(
            a,
            s,
            force,
            ENTRY_BYTES,
            TOTAL_BYTES - FRAME_RESERVE,
            ENTRIES,
        );
    }
    fn remember_frame(&mut self, id: &str, passed: bool) {
        if id.len() <= 80 && (self.frame.contains_key(id) || self.frame.len() < FRAME_IDS) {
            self.frame.insert(id.into(), passed);
        }
    }
    fn fallback(&mut self, id: &str, rows: Vec<(bool, String)>) {
        self.counters.fallbacks += 1;
        if id.len() <= 80 {
            self.fallback_rows = Some((id.into(), rows));
        }
    }
    fn evaluate_limits(
        &mut self,
        a: &Attempt,
        s: &Scenario,
        force: bool,
        entry_cap: usize,
        total_cap: usize,
        entries: usize,
    ) {
        let (
            engine::State::BasicNat {
                simulation: initial,
            },
            engine::Goal::BasicNat { definition },
            engine::State::BasicNat { .. },
        ) = (&s.initial, &s.goal, &a.state)
        else {
            self.entries.retain(|e| e.id != s.id);
            self.bytes = self.entries.iter().map(|e| e.charge).sum();
            self.remember_frame(&s.id, false);
            self.fallback(&s.id, vec![(false, "NAT assignment kind differs".into())]);
            return;
        };
        let definition = definition.as_ref();
        self.tick = self.tick.saturating_add(1);
        let shape = storage_shape(a, definition)
            && s.id.len() <= 80
            && s.nat_teaching
                .as_ref()
                .is_none_or(|l| l.shape() && storage_shape(a, &l.definition));
        if !force
            && shape
            && let Some(e) = self.entries.iter_mut().find(|e| {
                e.id == s.id
                    && &e.initial == initial
                    && &e.definition == definition
                    && e.lesson.as_ref() == s.nat_teaching.as_ref()
                    && same_attempt(&e.attempt, a)
            })
        {
            e.touched = self.tick;
            self.counters.hits += 1;
            let passed = e.rows.iter().all(|r| r.0);
            self.remember_frame(&s.id, passed);
            return;
        }
        self.counters.evaluations += 1;
        let rows = progress::current_nat_checks(a, s);
        let practice_rows = match (
            a.workspace.as_ref().and_then(|w| w.nat_learning.as_ref()),
            s.nat_teaching.as_ref(),
        ) {
            (Some(l), Some(lesson)) if lesson.shape() => l.checks(lesson),
            _ => Vec::new(),
        };
        if s.id.len() <= 80
            && practice_rows.len() <= 37
            && practice_rows
                .iter()
                .try_fold(0usize, |n, r| n.checked_add(r.1.len()))
                .is_some_and(|n| n <= 32 * 1024)
        {
            self.fallback_practice = Some((s.id.clone(), practice_rows.clone()));
        } else {
            self.fallback_practice = None;
        }
        // The authoritative helper retains bounded feedback; guard its storage too.
        let report_ok = rows.len().saturating_add(practice_rows.len()) <= 128
            && rows
                .iter()
                .chain(practice_rows.iter())
                .try_fold(0usize, |n, r| n.checked_add(r.1.len()))
                .is_some_and(|n| n <= 32 * 1024);
        let passed = rows.iter().all(|r| r.0);
        self.remember_frame(&s.id, passed);
        let rows = if report_ok {
            rows
        } else {
            vec![(
                passed,
                "Current check completed; detailed feedback exceeds the native display bound"
                    .into(),
            )]
        };
        // Replaced identity must never leave an old success available.
        self.entries.retain(|e| e.id != s.id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        if !shape || !report_ok || entries == 0 {
            self.fallback(&s.id, rows);
            return;
        }
        self.counters.meters += 1;
        let Some(charge) = charged(
            &(&s.id, initial, definition, &s.nat_teaching, a),
            entry_cap.min(total_cap),
        ) else {
            self.fallback(&s.id, rows);
            return;
        };
        // Evict BEFORE cloning any source inputs; replacement/eviction is NAT-local.
        while !self.entries.is_empty()
            && (self.entries.len() >= entries || self.bytes.saturating_add(charge) > total_cap)
        {
            let at = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.touched)
                .unwrap()
                .0;
            self.bytes -= self.entries.remove(at).charge;
        }
        if charge > total_cap {
            self.fallback(&s.id, rows);
            return;
        }
        self.counters.snapshots += 1;
        self.entries.push(Entry {
            id: s.id.clone(),
            initial: initial.clone(),
            definition: definition.clone(),
            lesson: s.nat_teaching.clone(),
            practice_rows,
            attempt: a.clone(),
            rows,
            charge,
            touched: self.tick,
        });
        self.bytes += charge;
    }
}
// Exhaustive projection: adding any new workspace field requires a deliberate decision.
// All typed foreign-family payloads are rejected by NAT admission. Never copy them.
fn foreign(w: &model::WorkspaceData) -> bool {
    let model::WorkspaceData {
        nat_draft: _,
        nat_learning: _,
        version: _,
        command: _,
        service_port: _,
        dns_address: _,
        dns_serial: _,
        mount_device: _,
        checked: _,
        ipv4_check: _,
        practice_check: _,
        migrated_host: _,
        legacy_host_reviewed: _,
        legacy_practice_reviewed: _,
        migrated_practice_host: _,
        script_history: _,
        script_history_more: _,
        debian_packages_terminal,
        apache_static_draft,
        apache_static_learning,
        debian_packages_draft,
        debian_packages_learning,
        spanning_tree_draft,
        spanning_tree_learning,
        dhcpv4_draft,
        dhcpv4_learning,
        unit_manager_draft,
        unit_manager_learning,
        unit_manager_terminal,
        ospf_draft,
        ospf_learning,
        acl_draft,
        acl_learning,
        script_learning,
        terminal_learning,
        process_learning,
        accounts_learning,
        discovery_draft,
        discovery_learning,
        storage_draft,
        storage_learning,
        ipv6_draft,
        ipv6_learning,
        routing_v4_draft,
        routing_v4_learning,
        address_learning,
        service_learning,
        client_learning,
        seed_dns_learning,
        seed_mount_learning,
        legacy_access_learning,
        legacy_logs_learning,
        client_draft,
        dns_learning,
        dns_draft,
        switching_draft,
        switching_learning,
        mac_draft,
        mac_learning,
    } = w;
    macro_rules! any { ($($v:ident),* $(,)?) => { false $(|| $v.is_some())* }; }
    any!(
        debian_packages_terminal,
        apache_static_draft,
        apache_static_learning,
        debian_packages_draft,
        debian_packages_learning,
        spanning_tree_draft,
        spanning_tree_learning,
        dhcpv4_draft,
        dhcpv4_learning,
        unit_manager_draft,
        unit_manager_learning,
        unit_manager_terminal,
        ospf_draft,
        ospf_learning,
        acl_draft,
        acl_learning,
        script_learning,
        terminal_learning,
        process_learning,
        accounts_learning,
        discovery_draft,
        discovery_learning,
        storage_draft,
        storage_learning,
        ipv6_draft,
        ipv6_learning,
        routing_v4_draft,
        routing_v4_learning,
        address_learning,
        service_learning,
        client_learning,
        seed_dns_learning,
        seed_mount_learning,
        legacy_access_learning,
        legacy_logs_learning,
        client_draft,
        dns_learning,
        dns_draft,
        switching_draft,
        switching_learning,
        mac_draft,
        mac_learning
    )
}
fn same_attempt(a: &Attempt, b: &Attempt) -> bool {
    a.scenario_version == b.scenario_version
        && a.state == b.state
        && a.step == b.step
        && a.challenge == b.challenge
        && a.hints == b.hints
        && a.transcript == b.transcript
        && a.visited == b.visited
        && a.ipv4 == b.ipv4
        && match (&a.workspace, &b.workspace) {
            (None, None) => true,
            (Some(a), Some(b)) => same_workspace(a, b),
            _ => false,
        }
}
fn same_workspace(a: &model::WorkspaceData, b: &model::WorkspaceData) -> bool {
    !foreign(a)
        && !foreign(b)
        && a.version == b.version
        && a.command == b.command
        && a.service_port == b.service_port
        && a.dns_address == b.dns_address
        && a.dns_serial == b.dns_serial
        && a.mount_device == b.mount_device
        && a.checked == b.checked
        && a.ipv4_check == b.ipv4_check
        && a.practice_check == b.practice_check
        && a.migrated_host == b.migrated_host
        && a.legacy_host_reviewed == b.legacy_host_reviewed
        && a.legacy_practice_reviewed == b.legacy_practice_reviewed
        && a.migrated_practice_host == b.migrated_practice_host
        && a.nat_learning == b.nat_learning
        && a.script_history == b.script_history
        && a.script_history_more == b.script_history_more
        && match (&a.nat_draft, &b.nat_draft) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.context == b.context
                    && a.statics == b.statics
                    && a.dynamic_enabled == b.dynamic_enabled
                    && (!a.dynamic_enabled
                        || (a.network == b.network
                            && a.prefix == b.prefix
                            && a.first == b.first
                            && a.last == b.last))
            }
            _ => false,
        }
}
fn record_shape(r: &shell::Record) -> bool {
    r.source.len() <= 16384
        && r.results.len() <= 12
        && r.results.iter().all(|o| {
            o.stdout.len().saturating_add(o.stderr.len()) <= 32768
                && o.files.len() <= 64
                && o.error.as_ref().is_none_or(|e| e.len() <= 1000)
                && o.files
                    .iter()
                    .all(|(k, v)| k.len() <= 256 && v.len() <= 262144)
                && o.files
                    .values()
                    .try_fold(0usize, |n, v| n.checked_add(v.len()))
                    .is_some_and(|n| n <= 262144)
        })
}
pub(super) fn draft_shape(d: &work::Draft) -> bool {
    d.context.fixture.len() <= 32
        && d.context.inside.len() <= 64
        && d.context.outside.len() <= 64
        && d.context.globals.len() <= 128
        && d.context
            .inside
            .keys()
            .chain(d.context.outside.keys())
            .all(|s| s.len() <= 32)
        && d.statics.len() <= 32
        && d.statics
            .iter()
            .all(|r| r.inside.len() <= 32 && r.global.len() <= 64)
        && d.network.len() <= 64
        && d.prefix.len() <= 16
        && d.first.len() <= 64
        && d.last.len() <= 64
        && d.inbound_global.len() <= 64
        && d.notice.as_ref().is_none_or(|s| s.len() <= 512)
        && d.inside
            .as_ref()
            .is_none_or(|s| s.len() <= 32 && d.context.inside.contains_key(s))
        && d.peer
            .as_ref()
            .is_none_or(|s| s.len() <= 32 && d.context.outside.contains_key(s))
        && d.selected_row
            .is_none_or(|n| usize::from(n) < d.statics.len())
}
fn storage_shape(a: &Attempt, d: &work::Definition) -> bool {
    if d.requirements.len() > 32
        || d.trials.len() > 4
        || a.transcript.len() > 1024
        || a.transcript.iter().any(|s| s.len() > 4096)
    {
        return false;
    }
    if d.requirements.iter().any(|r| match r {
        work::Requirement::CurrentMapping { inside, .. }
        | work::Requirement::NoCurrentMapping { inside }
        | work::Requirement::StaticPreserved { inside }
        | work::Requirement::DynamicPrefixMatches { inside, .. } => inside.len() > 32,
        _ => false,
    }) || d.trials.iter().any(|t| {
        t.name.len() > 64
            || t.steps.len() > 16
            || t.steps.iter().any(|s| match s {
                work::Step::Outbound { inside, peer, .. }
                | work::Step::Reverse { inside, peer, .. } => inside.len() > 32 || peer.len() > 32,
                work::Step::NoMapping { peer, .. } => peer.len() > 32,
            })
    }) {
        return false;
    }
    if a.ipv4
        .as_ref()
        .and_then(|l| l.variant.as_ref())
        .is_some_and(|e| {
            e.answers.iter().any(|s| s.len() > 4096)
                || e.second.as_ref().is_some_and(|s| s.len() > 4096)
        })
    {
        return false;
    }
    a.workspace.as_ref().is_none_or(|w| {
        !foreign(w)
            && w.command.len() <= 4096
            && w.nat_draft.as_ref().is_none_or(draft_shape)
            && w.nat_learning.as_ref().is_none_or(|l| {
                l.version == 1
                    && l.selected_action < 16
                    && l.notice.as_ref().is_none_or(|s| s.len() <= 512)
                    && draft_shape(&l.draft)
                    && l.fixture.inventory() == l.simulation.inventory()
                    && &l.draft.context == l.simulation.inventory()
            })
            && [
                &w.service_port,
                &w.dns_address,
                &w.dns_serial,
                &w.mount_device,
            ]
            .iter()
            .all(|s| s.as_ref().is_none_or(|s| s.len() <= 256))
            && [&w.ipv4_check, &w.practice_check]
                .iter()
                .all(|s| s.as_ref().is_none_or(|s| s.iter().all(|v| v.len() <= 64)))
            && w.script_history_more.len() + usize::from(w.script_history.is_some()) <= 32
            && w.script_history.as_ref().is_none_or(record_shape)
            && w.script_history_more.iter().all(record_shape)
    })
}
// Streaming structural counter; never builds JSON, Value or a formatted address String.
// Every value/key/collection is charged 128 bytes in addition to every string byte.
#[derive(Debug)]
struct Limit;
impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NAT memo storage ceiling")
    }
}
impl std::error::Error for Limit {}
impl ser::Error for Limit {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self
    }
}
struct Meter {
    s: usize,
    n: usize,
    cap: usize,
}
impl Meter {
    fn add(&mut self, s: usize) -> Result<(), Limit> {
        self.s = self.s.checked_add(s).ok_or(Limit)?;
        self.n = self.n.checked_add(1).ok_or(Limit)?;
        (self.charge().is_some_and(|n| n <= self.cap))
            .then_some(())
            .ok_or(Limit)
    }
    fn charge(&self) -> Option<usize> {
        self.n
            .checked_mul(128)?
            .checked_add(self.s)?
            .checked_add(4096 + REPORT_RESERVE)
    }
}
pub(in crate::ui) fn charged<T: Serialize>(value: &T, cap: usize) -> Option<usize> {
    let mut m = Meter { s: 0, n: 0, cap };
    value.serialize(&mut m).ok()?;
    m.charge()
}
struct Compound<'a>(&'a mut Meter);
macro_rules! scalar {($($name:ident($ty:ty)),*) => {$ (fn $name(self,_:$ty)->Result<(),Limit>{self.add(0)})*};}
impl<'a> ser::Serializer for &'a mut Meter {
    type Ok = ();
    type Error = Limit;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;
    scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64)
    );
    fn serialize_char(self, c: char) -> Result<(), Limit> {
        self.add(c.len_utf8())
    }
    fn serialize_str(self, s: &str) -> Result<(), Limit> {
        self.add(s.len())
    }
    fn serialize_bytes(self, s: &[u8]) -> Result<(), Limit> {
        self.add(0)?;
        for _ in s {
            self.add(0)?;
        }
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Limit> {
        self.add(0)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Limit> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Limit> {
        self.add(0)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Limit> {
        self.add(0)
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, v: &'static str) -> Result<(), Limit> {
        self.add(v.len())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Limit> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        k: &'static str,
        v: &T,
    ) -> Result<(), Limit> {
        self.add(0)?;
        self.add(k.len())?;
        v.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a>, Limit> {
        self.add(0)?;
        Ok(Compound(self))
    }
    fn serialize_tuple(self, n: usize) -> Result<Compound<'a>, Limit> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_struct(self, _: &'static str, n: usize) -> Result<Compound<'a>, Limit> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        k: &'static str,
        n: usize,
    ) -> Result<Compound<'a>, Limit> {
        self.add(0)?;
        self.add(k.len())?;
        self.serialize_seq(Some(n))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'a>, Limit> {
        self.add(0)?;
        Ok(Compound(self))
    }
    fn serialize_struct(self, _: &'static str, n: usize) -> Result<Compound<'a>, Limit> {
        self.serialize_map(Some(n))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        k: &'static str,
        n: usize,
    ) -> Result<Compound<'a>, Limit> {
        self.add(0)?;
        self.add(k.len())?;
        self.serialize_map(Some(n))
    }
    fn collect_str<T: fmt::Display + ?Sized>(self, v: &T) -> Result<(), Limit> {
        struct Text<'a>(&'a mut usize);
        impl fmt::Write for Text<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                *self.0 = self.0.checked_add(s.len()).ok_or(fmt::Error)?;
                Ok(())
            }
        }
        let mut n = 0;
        fmt::write(&mut Text(&mut n), format_args!("{v}")).map_err(|_| Limit)?;
        self.add(n)
    }
}
impl SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Limit> {
        v.serialize(&mut *self.0)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}
impl ser::SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Limit> {
        SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}
impl ser::SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Limit> {
        SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}
impl ser::SerializeTupleVariant for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Limit> {
        SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}
impl SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Limit> {
        k.serialize(&mut *self.0)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Limit> {
        v.serialize(&mut *self.0)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}
impl SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        k: &'static str,
        v: &T,
    ) -> Result<(), Limit> {
        self.0.add(k.len())?;
        v.serialize(&mut *self.0)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}
impl ser::SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = Limit;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        k: &'static str,
        v: &T,
    ) -> Result<(), Limit> {
        SerializeStruct::serialize_field(self, k, v)
    }
    fn end(self) -> Result<(), Limit> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Scenario, Attempt) {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/nat-workspace-preview.json"
        ))
        .unwrap()
        .remove(0);
        let mut a = Attempt::new(&s, 1);
        model::prepare(&mut a);
        (s, a)
    }
    fn d(a: &mut Attempt) -> &mut work::Draft {
        a.workspace.as_mut().unwrap().nat_draft.as_mut().unwrap()
    }
    fn solve(s: &Scenario, a: &mut Attempt) {
        d(a).network = "10.0.0.8".into();
        d(a).prefix = "30".into();
        let engine::State::BasicNat {
            simulation: initial,
        } = &s.initial
        else {
            panic!()
        };
        let w = a.workspace.as_ref().unwrap().nat_draft.as_ref().unwrap();
        let engine::State::BasicNat { simulation } = &mut a.state else {
            panic!()
        };
        w.apply(initial, simulation).unwrap();
        for inside in ["alpha", "beta"] {
            work::perform(
                initial,
                simulation,
                &core::Action::Outbound {
                    inside: inside.into(),
                    peer: "peer".into(),
                },
            )
            .unwrap();
        }
        assert!(progress::current_solved(a, s));
    }
    fn teaching_fixture() -> (Scenario, Attempt) {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/nat-teaching-preview.json"
        ))
        .unwrap()
        .remove(0);
        let mut a = Attempt::new(&s, 1);
        model::prepare(&mut a);
        dlms_learning_lab::nat_teaching::prepare(&mut a, s.nat_teaching.as_ref().unwrap(), true)
            .unwrap();
        (s, a)
    }
    #[test]
    fn exact_learning_and_lesson_key_hits_do_not_run_admission_checks_or_copy_inputs() {
        let (mut s, mut a) = teaching_fixture();
        solve(&s, &mut a);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        let before = m.counters;
        for _ in 0..10 {
            m.begin_frame();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
        }
        assert_eq!(m.counters.evaluations, before.evaluations);
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
        a.workspace
            .as_mut()
            .unwrap()
            .nat_learning
            .as_mut()
            .unwrap()
            .draft
            .network = "unfinished example only".into();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert_eq!(m.counters.evaluations, before.evaluations + 1);
        let good = a.clone();
        a.workspace
            .as_mut()
            .unwrap()
            .nat_learning
            .as_mut()
            .unwrap()
            .version = 2;
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        a = good;
        s.nat_teaching.as_mut().unwrap().prerequisite_note.clear();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
    }
    #[test]
    fn practice_result_is_separate_final_input_and_truthful_when_storage_is_unavailable() {
        let (s, mut a) = teaching_fixture();
        solve(&s, &mut a);
        let mut m = Memo::default();
        m.evaluate_limits(&a, &s, false, 1, 1, 0);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert!(m.practice_rows(&s.id).unwrap().iter().any(|r| !r.0));
        let lesson = s.nat_teaching.as_ref().unwrap();
        let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
        for step in &lesson.example_actions {
            work::perform(&l.fixture, &mut l.simulation, &step.action).unwrap();
        }
        l.draft = work::Draft::new(&l.simulation).unwrap();
        m.evaluate_limits(&a, &s, false, 1, 1, 0);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert!(m.practice_rows(&s.id).unwrap().iter().all(|r| r.0));
        assert_eq!(m.counters.snapshots, 0);
        let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
        work::perform(
            &l.fixture,
            &mut l.simulation,
            &core::Action::ClearDynamic {},
        )
        .unwrap();
        m.evaluate_limits(&a, &s, false, 1, 1, 0);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert!(m.practice_rows(&s.id).unwrap().iter().any(|r| !r.0));
    }
    #[test]
    fn teaching_maximum_lists_prose_raw_drafts_and_three_owned_keys_are_fully_charged() {
        let (mut s, mut a) = teaching_fixture();
        let lesson = s.nat_teaching.as_mut().unwrap();
        lesson.prerequisites = (0..16).map(|n| format!("p{n}")).collect();
        lesson.prerequisite_note = "x".repeat(1600);
        lesson.example_description = "x".repeat(1600);
        lesson.concepts = vec![
            dlms_learning_lab::nat_teaching::Concept {
                title: "t".repeat(160),
                text: "c".repeat(2400)
            };
            6
        ];
        while lesson.example_actions.len() < 16 {
            lesson
                .example_actions
                .push(lesson.example_actions.last().unwrap().clone());
        }
        for step in &mut lesson.example_actions {
            step.title = "t".repeat(160);
            step.explanation = "e".repeat(800);
        }
        lesson.try_change.description = "e".repeat(1600);
        lesson.try_change.actions = vec![core::Action::ClearDynamic {}; 16];
        while lesson.repair.len() < 16 {
            lesson.repair.push(lesson.repair.last().unwrap().clone());
        }
        lesson.validate().unwrap();
        a.workspace.as_mut().unwrap().nat_learning =
            Some(dlms_learning_lab::nat_teaching::Learning::new(lesson).unwrap());
        solve(&s, &mut a);
        let learning = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
        learning.draft.network = "x".repeat(64);
        learning.draft.notice = Some("n".repeat(512));
        learning.notice = Some("n".repeat(512));
        assert!(learning.valid());
        let engine::State::BasicNat {
            simulation: initial,
        } = &s.initial
        else {
            panic!()
        };
        let engine::Goal::BasicNat { definition } = &s.goal else {
            panic!()
        };
        let key = (&s.id, initial, definition.as_ref(), &s.nat_teaching, &a);
        let (strings, tokens) = independent(&serde_json::to_value(key).unwrap());
        let charge = charged(&key, ENTRY_BYTES).unwrap();
        assert_eq!(charge, strings + 128 * tokens + 4096 + REPORT_RESERVE);
        assert!(charge * 3 + FRAME_RESERVE < TOTAL_BYTES);
        let mut m = Memo::default();
        for n in 0..3 {
            let mut next = s.clone();
            next.id = format!("nat-teaching-budget-{n}");
            m.evaluate(&a, &next, false);
            assert_eq!(m.prepared(&next.id), Some(true));
        }
        assert_eq!(m.entries.len(), 3);
        assert!(m.bytes + FRAME_RESERVE <= TOTAL_BYTES);
        let before = m.counters;
        for _ in 0..5 {
            m.begin_frame();
            for n in 0..3 {
                let mut next = s.clone();
                next.id = format!("nat-teaching-budget-{n}");
                m.evaluate(&a, &next, false);
            }
        }
        assert_eq!(m.counters.evaluations, before.evaluations);
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
    }
    #[test]
    fn hits_use_real_helpers_without_recount_copy_or_trials_and_readonly_meaning_is_exact() {
        let (s, mut a) = fixture();
        solve(&s, &mut a);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        let c = m.counters;
        assert_eq!(c.evaluations, 1);
        assert_eq!(c.snapshots, 1);
        for n in 0..10 {
            m.begin_frame();
            d(&mut a).inbound_global = format!("bad-but-readonly-{n}");
            d(&mut a).notice = Some(format!("request {n}"));
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
        }
        assert_eq!(m.counters.evaluations, c.evaluations);
        assert_eq!(m.counters.meters, c.meters);
        assert_eq!(m.counters.snapshots, c.snapshots);
        assert_eq!(m.counters.hits, 10);
        d(&mut a).inbound_global = "x".repeat(65);
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert_eq!(m.counters.snapshots, 1);
    }
    #[test]
    fn disabled_malformed_policy_is_inactive_but_enabled_invalid_policy_is_pending() {
        let (mut s, mut a) = fixture();
        let engine::State::BasicNat {
            simulation: initial,
        } = &mut s.initial
        else {
            panic!()
        };
        let mut c = initial.configuration().clone();
        c.dynamic = None;
        initial
            .apply(&core::Action::Configure { configuration: c })
            .unwrap();
        let engine::Goal::BasicNat { definition } = &mut s.goal else {
            panic!()
        };
        definition.initial = initial.clone();
        definition.requirements = vec![work::Requirement::StaticPreserved {
            inside: "observer".into(),
        }];
        definition.trials = vec![work::Trial {
            name: "Static reverse".into(),
            steps: vec![
                work::Step::Reverse {
                    inside: "observer".into(),
                    peer: "peer".into(),
                    mapping_kind: work::Kind::Static,
                    global: work::Global::Any {},
                },
                work::Step::Outbound {
                    inside: "observer".into(),
                    peer: "peer".into(),
                    expected: work::Expected::Translated {
                        mapping_kind: work::Kind::Static,
                        global: work::Global::Any {},
                    },
                },
            ],
        }];
        definition.validate().unwrap();
        a.state = s.initial.clone();
        a.workspace = None;
        model::prepare(&mut a);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        let before = m.counters;
        d(&mut a).network = "nonsense".into();
        d(&mut a).prefix = "invalid".into();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert_eq!(m.counters.evaluations, before.evaluations);
        let old = a.state.clone();
        d(&mut a).dynamic_enabled = true;
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert_eq!(a.state, old);
    }
    #[test]
    fn same_id_full_origin_goal_state_and_allowed_metadata_replacements_cannot_reuse_success() {
        let (s, mut a) = fixture();
        solve(&s, &mut a);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        let good = a.clone();
        d(&mut a).network = "10.0.1.0".into();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        a = good.clone();
        let mut changed = s.clone();
        let engine::Goal::BasicNat { definition } = &mut changed.goal else {
            panic!()
        };
        definition
            .requirements
            .push(work::Requirement::NoCurrentMapping {
                inside: "alpha".into(),
            });
        m.evaluate(&a, &changed, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        changed = s.clone();
        let engine::State::BasicNat { simulation } = &mut changed.initial else {
            panic!()
        };
        simulation.apply(&core::Action::ClearDynamic {}).unwrap();
        // A genuinely different authored configuration, not just an equal Clear.
        let mut config = simulation.configuration().clone();
        config.dynamic = None;
        simulation
            .apply(&core::Action::Configure {
                configuration: config,
            })
            .unwrap();
        m.evaluate(&a, &changed, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        a = good.clone();
        a.workspace.as_mut().unwrap().version = 2;
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        a = good.clone();
        a.workspace.as_mut().unwrap().command = "x".repeat(4097);
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        a = good.clone();
        let engine::State::BasicNat { simulation } = &mut a.state else {
            panic!()
        };
        simulation.apply(&core::Action::ClearDynamic {}).unwrap();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        a = good;
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        m.evaluate(&a, &s, true);
        assert_eq!(m.prepared(&s.id), Some(true));
    }
    #[test]
    fn foreign_valid_metadata_after_a_pass_is_rejected_without_snapshotting_its_payload() {
        let (s, mut a) = fixture();
        solve(&s, &mut a);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        let before = m.counters.snapshots;
        let scripts = pack::bundled().unwrap();
        let scripted = scripts.iter().find(|s| s.teaching.is_some()).unwrap();
        let mut other = Attempt::new(scripted, 1);
        model::prepare(&mut other);
        let lesson = scripted.teaching.as_ref().unwrap();
        dlms_learning_lab::script_teaching::prepare(&mut other, lesson, true);
        a.workspace.as_mut().unwrap().script_learning = other.workspace.unwrap().script_learning;
        assert!(a.workspace.as_ref().unwrap().script_learning.is_some());
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert_eq!(m.counters.snapshots, before);
    }
    #[test]
    fn tiny_cache_limits_fall_back_to_true_grading_lru_never_exceeds_budget_and_removal_evicts() {
        let (s, mut a) = fixture();
        solve(&s, &mut a);
        let mut m = Memo::default();
        m.evaluate_limits(&a, &s, false, 1, 1, 16);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert_eq!(m.counters.snapshots, 0);
        assert_eq!(m.counters.fallbacks, 1);
        let cap = charged(
            &(
                &s.id,
                match &s.initial {
                    engine::State::BasicNat { simulation } => simulation,
                    _ => panic!(),
                },
                match &s.goal {
                    engine::Goal::BasicNat { definition } => definition.as_ref(),
                    _ => panic!(),
                },
                &s.nat_teaching,
                &a,
            ),
            ENTRY_BYTES,
        )
        .unwrap();
        for n in 0..20 {
            let mut s = s.clone();
            s.id = format!("nat-case-{n:02}");
            m.evaluate_limits(&a, &s, false, ENTRY_BYTES, cap * 2, 2);
            assert!(m.entries.len() <= 2);
            assert!(m.bytes <= cap * 2);
        }
        assert_eq!(m.entries.len(), 2);
        assert!(
            m.entries
                .iter()
                .all(|e| e.id == "nat-case-18" || e.id == "nat-case-19")
        );
        let mut touched = s.clone();
        touched.id = "nat-case-18".into();
        m.evaluate_limits(&a, &touched, false, ENTRY_BYTES, cap * 2, 2);
        let mut newest = s.clone();
        newest.id = "nat-case-20".into();
        m.evaluate_limits(&a, &newest, false, ENTRY_BYTES, cap * 2, 2);
        assert!(
            m.entries
                .iter()
                .all(|e| e.id == "nat-case-18" || e.id == "nat-case-20")
        );
        m.retain(|_| false);
        assert!(m.entries.is_empty());
        assert_eq!(m.bytes, 0);
    }
    fn independent(v: &serde_json::Value) -> (usize, usize) {
        match v {
            serde_json::Value::String(s) => (s.len(), 1),
            serde_json::Value::Array(v) => v
                .iter()
                .map(independent)
                .fold((0, 1), |(s, n), (a, b)| (s + a, n + b)),
            serde_json::Value::Object(v) => v.iter().fold((0, 1), |(s, n), (k, v)| {
                let (a, b) = independent(v);
                (s + k.len() + a, n + 1 + b)
            }),
            _ => (0, 1),
        }
    }
    #[test]
    fn maximum_schema_walker_matches_independent_json_tokens_and_normal_three_key_closure_fits() {
        let (mut s, mut a) = fixture();
        let inventory = core::Inventory {
            fixture: "n".repeat(32),
            inside: (0..64)
                .map(|n| {
                    (
                        format!("i{n:030}"),
                        std::net::Ipv4Addr::new(10, 0, 0, n + 1),
                    )
                })
                .collect(),
            outside: (0..64)
                .map(|n| {
                    (
                        format!("o{n:030}"),
                        std::net::Ipv4Addr::new(203, 0, 113, n + 1),
                    )
                })
                .collect(),
            globals: (0..128)
                .map(|n| std::net::Ipv4Addr::new(198, 51, 100, n + 1))
                .collect(),
        };
        let config = core::Configuration {
            statics: inventory
                .inside
                .keys()
                .take(32)
                .cloned()
                .zip(inventory.globals.iter().copied())
                .collect(),
            dynamic: Some(core::DynamicRule {
                source: core::Prefix {
                    network: "10.0.0.0".parse().unwrap(),
                    length: 24,
                },
                pool: core::Pool {
                    first: "198.51.100.33".parse().unwrap(),
                    last: "198.51.100.96".parse().unwrap(),
                },
            }),
        };
        let state = core::State::new(&inventory, &config).unwrap();
        s.initial = engine::State::BasicNat {
            simulation: state.clone(),
        };
        let inside = inventory.inside.keys().nth(32).unwrap().clone();
        let peer = inventory.outside.keys().next().unwrap().clone();
        let engine::Goal::BasicNat { definition } = &mut s.goal else {
            panic!()
        };
        definition.initial = state.clone();
        definition.requirements = (1..=32)
            .map(|minimum| work::Requirement::DynamicCapacity { minimum })
            .collect();
        let pair = [
            work::Step::Outbound {
                inside: inside.clone(),
                peer: peer.clone(),
                expected: work::Expected::Translated {
                    mapping_kind: work::Kind::Dynamic,
                    global: work::Global::Any {},
                },
            },
            work::Step::Reverse {
                inside,
                peer,
                mapping_kind: work::Kind::Dynamic,
                global: work::Global::Any {},
            },
        ];
        definition.trials = (0..4)
            .map(|n| work::Trial {
                name: format!("{}{n}", "n".repeat(63)),
                steps: pair.iter().cloned().cycle().take(16).collect(),
            })
            .collect();
        definition.validate().unwrap();
        a.state = engine::State::BasicNat {
            simulation: state.clone(),
        };
        a.workspace = None;
        model::prepare(&mut a);
        let w = a.workspace.as_mut().unwrap();
        w.command = "x".repeat(4096);
        w.service_port = Some("s".repeat(256));
        w.dns_address = w.service_port.clone();
        w.dns_serial = w.service_port.clone();
        w.mount_device = w.service_port.clone();
        w.ipv4_check = Some(std::array::from_fn(|_| "v".repeat(64)));
        w.practice_check = w.ipv4_check.clone();
        let draft = w.nat_draft.as_mut().unwrap();
        draft.notice = Some("n".repeat(512));
        draft.inbound_global = "x".repeat(64);
        assert!(storage_shape(&a, definition));
        let key = (&s.id, &state, &*definition, &a);
        let (strings, tokens) = independent(&serde_json::to_value(key).unwrap());
        let actual = charged(&key, ENTRY_BYTES).unwrap();
        assert_eq!(actual, strings + 128 * tokens + 4096 + REPORT_RESERVE);
        assert!(actual <= 2060 * 1024, "normal maximum closure {actual}");
        assert!(actual * 3 <= 6180 * 1024);
        assert!(actual * 3 < TOTAL_BYTES);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        let before = m.counters;
        for _ in 0..5 {
            m.begin_frame();
            m.evaluate(&a, &s, false);
        }
        assert_eq!(m.counters.evaluations, before.evaluations);
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(m.counters.meters, before.meters);
    }
}

#[cfg(test)]
mod history_storage_tests {
    use super::*;
    #[test]
    fn genuine_record_copies_are_charged_and_uncacheable_success_is_shared_for_all_frame_lookups() {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/nat-workspace-preview.json"
        ))
        .unwrap()
        .remove(0);
        let mut a = Attempt::new(&s, 1);
        model::prepare(&mut a);
        let d = a.workspace.as_mut().unwrap().nat_draft.as_mut().unwrap();
        d.network = "10.0.0.8".into();
        d.prefix = "30".into();
        let engine::State::BasicNat {
            simulation: initial,
        } = &s.initial
        else {
            panic!()
        };
        let engine::State::BasicNat { simulation } = &mut a.state else {
            panic!()
        };
        d.apply(initial, simulation).unwrap();
        for inside in ["alpha", "beta"] {
            work::perform(
                initial,
                simulation,
                &core::Action::Outbound {
                    inside: inside.into(),
                    peer: "peer".into(),
                },
            )
            .unwrap();
        }
        let actual: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-v8-synthetic-writer-progress.json"
        ))
        .unwrap();
        let raw = actual["attempts"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|a| {
                a["state"]["run"]
                    .as_object()
                    .map(|_| a["state"]["run"].clone())
            })
            .unwrap();
        let record: shell::Record = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(record.engine_revision, 8);
        assert!(record.valid());
        // Repeated genuine records are storage stress only, not new observed runs.
        let w = a.workspace.as_mut().unwrap();
        w.script_history = Some(record.clone());
        w.script_history_more = vec![record; 31];
        assert!(w.valid());
        assert!(progress::current_solved(&a, &s));
        let engine::Goal::BasicNat { definition } = &s.goal else {
            panic!()
        };
        let full = charged(&(&s.id, initial, definition, &a), usize::MAX).unwrap();
        let mut bare = a.clone();
        bare.workspace.as_mut().unwrap().script_history = None;
        bare.workspace.as_mut().unwrap().script_history_more.clear();
        let small = charged(&(&s.id, initial, definition, &bare), usize::MAX).unwrap();
        assert!(full > small);
        let mut m = Memo::default();
        m.evaluate_limits(&a, &s, false, small, small, 16);
        assert_eq!(m.counters.snapshots, 0);
        assert_eq!(m.counters.evaluations, 1);
        for _ in 0..20 {
            assert_eq!(m.prepared(&s.id), Some(true));
        }
        assert_eq!(m.counters.evaluations, 1);
        m.begin_frame();
        m.evaluate_limits(&a, &s, false, small, small, 16);
        assert_eq!(m.counters.evaluations, 2);
        assert!(
            serde_json::to_value(
                a.workspace
                    .as_ref()
                    .unwrap()
                    .script_history
                    .as_ref()
                    .unwrap()
            )
            .unwrap()
                == raw,
            "Complete genuine Record must remain equal"
        );
    }
}
