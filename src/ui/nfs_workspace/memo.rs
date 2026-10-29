//! Exact complete-input NFS memo; never saved or a completion credential.
use super::*;
use serde::{
    Serialize,
    ser::{self, SerializeMap, SerializeSeq, SerializeStruct},
};
use std::fmt;
pub(super) const ENTRY_BYTES: usize = 8 * 1024 * 1024;
pub(super) const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const ENTRIES: usize = 4;
pub(super) const EDITOR_RESERVE: usize = 64 * 1024;
const FRAME_RESERVE: usize = 128 * 1024;
const REPORT_RESERVE: usize = 2 * 37 * (512 + 128) + 4096;
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) struct Counters {
    pub evaluations: usize,
    pub meters: usize,
    pub snapshots: usize,
    pub hits: usize,
    pub fallbacks: usize,
}
struct Entry {
    source: Scenario,
    attempt: Attempt,
    assignment: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
    charge: usize,
    touched: u64,
}
struct Report {
    id: String,
    assignment: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
}
// Fixed80-byte IDs keep <=1000 frame booleans plus the one bounded fallback within128KiB.
struct FrameGrade {
    id: [u8; 80],
    length: u8,
    passed: bool,
}
struct FrameGrades {
    values: Vec<FrameGrade>,
}
impl Default for FrameGrades {
    fn default() -> Self {
        Self {
            values: Vec::with_capacity(1000),
        }
    }
}
impl FrameGrades {
    fn clear(&mut self) {
        self.values.clear();
    }
    fn get(&self, id: &str) -> Option<&bool> {
        self.values
            .iter()
            .find(|x| &x.id[..usize::from(x.length)] == id.as_bytes())
            .map(|x| &x.passed)
    }
    fn put(&mut self, id: &str, passed: bool) {
        if id.len() > 80 {
            return;
        }
        if let Some(x) = self
            .values
            .iter_mut()
            .find(|x| &x.id[..usize::from(x.length)] == id.as_bytes())
        {
            x.passed = passed;
            return;
        }
        if self.values.len() < 1000 {
            let mut bytes = [0; 80];
            bytes[..id.len()].copy_from_slice(id.as_bytes());
            self.values.push(FrameGrade {
                id: bytes,
                length: id.len() as u8,
                passed,
            });
        }
    }
}
#[derive(Default)]
pub(in crate::ui) struct Memo {
    entries: Vec<Entry>,
    bytes: usize,
    tick: u64,
    frame: FrameGrades,
    fallback: Option<Report>,
    pub counters: Counters,
}
impl Memo {
    pub fn begin_frame(&mut self) {
        self.frame.clear();
        self.fallback = None;
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
        self.frame.clear();
        self.fallback = None;
    }
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.entries.retain(|e| keep(&e.source.id));
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
    }
    pub fn prepared(&self, id: &str) -> Option<bool> {
        self.frame.get(id).copied()
    }
    pub fn rows(&self, id: &str, practice: bool) -> Option<&[(bool, String)]> {
        self.entries
            .iter()
            .find(|e| e.source.id == id)
            .map(|e| {
                if practice {
                    e.practice.as_slice()
                } else {
                    e.assignment.as_slice()
                }
            })
            .or_else(|| {
                self.fallback.as_ref().filter(|e| e.id == id).map(|e| {
                    if practice {
                        e.practice.as_slice()
                    } else {
                        e.assignment.as_slice()
                    }
                })
            })
    }
    pub fn admitted(&self, id: &str) -> bool {
        self.rows(id, false)
            .and_then(|r| r.first())
            .is_some_and(|r| r.0)
    }
    fn report(&mut self, id: &str, assignment: Vec<(bool, String)>, practice: Vec<(bool, String)>) {
        self.counters.fallbacks += 1;
        if id.len() <= 80 {
            self.frame
                .put(id, !assignment.is_empty() && assignment.iter().all(|r| r.0));
            self.fallback = Some(Report {
                id: id.into(),
                assignment,
                practice,
            });
        }
    }
    pub fn evaluate(&mut self, a: &Attempt, s: &Scenario, force: bool) {
        self.evaluate_limits(
            a,
            s,
            force,
            ENTRY_BYTES,
            TOTAL_BYTES - EDITOR_RESERVE - FRAME_RESERVE,
            ENTRIES,
        )
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
        let cheap = source_shape(s)
            && s.id.len() <= 80
            && s.version == a.scenario_version
            && matches!(a.state, engine::State::Nfs { .. })
            && a.ipv4.is_none()
            && a.workspace
                .as_ref()
                .is_none_or(|w| w.version == 1 && !foreign(w));
        if !cheap {
            self.entries.retain(|e| e.source.id != s.id);
            self.bytes = self.entries.iter().map(|e| e.charge).sum();
            self.report(
                &s.id,
                vec![(
                    false,
                    "NFS source or foreign workspace metadata differs".into(),
                )],
                vec![],
            );
            return;
        }
        self.tick = self.tick.saturating_add(1);
        if !force
            && let Some(e) = self
                .entries
                .iter_mut()
                .find(|e| same_source(&e.source, s) && same_attempt(&e.attempt, a))
        {
            e.touched = self.tick;
            self.counters.hits += 1;
            self.frame.put(
                &s.id,
                !e.assignment.is_empty() && e.assignment.iter().all(|r| r.0),
            );
            return;
        }
        self.counters.evaluations += 1;
        let assignment = progress::current_nfs_checks(a, s);
        let practice = match (
            a.workspace.as_ref().and_then(|w| w.nfs_learning.as_ref()),
            s.nfs_teaching.as_ref(),
        ) {
            (Some(l), Some(source)) => l.checks(source),
            _ => vec![],
        };
        let valid = assignment.first().is_some_and(|r| r.0)
            && assignment.len() <= 37
            && practice.len() <= 37
            && assignment.iter().chain(&practice).all(|r| r.1.len() <= 512);
        self.entries.retain(|e| e.source.id != s.id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        if !valid {
            self.report(&s.id, assignment, practice);
            return;
        }
        self.counters.meters += 1;
        let charge = charge(&(s, a), entry_cap);
        if let Some(charge) = charge.filter(|n| *n <= entry_cap && *n <= total_cap && entries > 0) {
            while self.entries.len() >= entries
                || self.bytes.checked_add(charge).is_none_or(|n| n > total_cap)
            {
                let Some((at, _)) = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.touched)
                else {
                    break;
                };
                self.bytes -= self.entries.remove(at).charge;
            }
            if self
                .bytes
                .checked_add(charge)
                .is_some_and(|n| n <= total_cap)
            {
                self.counters.snapshots += 1;
                self.frame.put(&s.id, assignment.iter().all(|r| r.0));
                self.bytes += charge;
                self.entries.push(Entry {
                    source: s.clone(),
                    attempt: a.clone(),
                    assignment,
                    practice,
                    charge,
                    touched: self.tick,
                });
                return;
            }
        }
        self.report(&s.id, assignment, practice);
    }
}
fn charge<T: Serialize + ?Sized>(v: &T, cap: usize) -> Option<usize> {
    let logical = charged(v, cap)?;
    let wire = compact_size(v, cap)?;
    logical
        .max(wire)
        .checked_add(REPORT_RESERVE)
        .filter(|n| *n <= cap)
}
fn compact_size<T: Serialize + ?Sized>(v: &T, cap: usize) -> Option<usize> {
    struct Count {
        used: usize,
        cap: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.used = self
                .used
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("NFS memo wire bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Count { used: 0, cap };
    serde_json::to_writer(&mut c, v).ok()?;
    Some(c.used)
}
fn source_shape(s: &Scenario) -> bool {
    if s.one_off_teaching.is_some() {
        return false;
    }
    let Scenario {
        physical_link_teaching,
        one_off_teaching: _,
        file_backup_teaching,
        etherchannel_teaching,
        cron_teaching,
        nfs_teaching,
        nat_teaching,
        rpm_packages_teaching,
        apache_static_teaching,
        spanning_tree_teaching,
        dhcpv4_teaching,
        teaching,
        terminal_teaching,
        process_teaching,
        accounts_teaching,
        unit_manager_teaching,
        ospf_teaching,
        discovery_teaching,
        storage_teaching,
        acl_teaching,
        ipv6_teaching,
        ipv4_routing_teaching,
        address_teaching,
        service_teaching,
        client_teaching,
        seed_dns_teaching,
        seed_mount_teaching,
        legacy_access_teaching,
        legacy_logs_teaching,
        dns_teaching,
        switching_teaching,
        mac_teaching,
        debian_packages_teaching,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
    } = s;
    let _ = nfs_teaching;
    physical_link_teaching.is_none()
        && file_backup_teaching.is_none()
        && etherchannel_teaching.is_none()
        && cron_teaching.is_none()
        && nat_teaching.is_none()
        && rpm_packages_teaching.is_none()
        && apache_static_teaching.is_none()
        && spanning_tree_teaching.is_none()
        && dhcpv4_teaching.is_none()
        && teaching.is_none()
        && terminal_teaching.is_none()
        && process_teaching.is_none()
        && accounts_teaching.is_none()
        && unit_manager_teaching.is_none()
        && ospf_teaching.is_none()
        && discovery_teaching.is_none()
        && storage_teaching.is_none()
        && acl_teaching.is_none()
        && ipv6_teaching.is_none()
        && ipv4_routing_teaching.is_none()
        && address_teaching.is_none()
        && service_teaching.is_none()
        && client_teaching.is_none()
        && seed_dns_teaching.is_none()
        && seed_mount_teaching.is_none()
        && legacy_access_teaching.is_none()
        && legacy_logs_teaching.is_none()
        && dns_teaching.is_none()
        && switching_teaching.is_none()
        && mac_teaching.is_none()
        && debian_packages_teaching.is_none()
        && matches!(
            (&s.initial, &s.goal),
            (engine::State::Nfs { .. }, engine::Goal::Nfs { .. })
        )
}
fn same_source(a: &Scenario, b: &Scenario) -> bool {
    source_shape(a)
        && source_shape(b)
        && a.id == b.id
        && a.version == b.version
        && a.track == b.track
        && a.topic == b.topic
        && a.unit == b.unit
        && a.title == b.title
        && a.summary == b.summary
        && a.objective == b.objective
        && a.retry_guidance == b.retry_guidance
        && a.difficulty == b.difficulty
        && a.minutes == b.minutes
        && a.steps == b.steps
        && a.explanation == b.explanation
        && a.hints == b.hints
        && a.sources == b.sources
        && a.simplifications == b.simplifications
        && a.initial == b.initial
        && a.solution == b.solution
        && a.prerequisites == b.prerequisites
        && a.nfs_teaching == b.nfs_teaching
        && match (&a.goal, &b.goal) {
            (engine::Goal::Nfs { definition: a }, engine::Goal::Nfs { definition: b }) => a == b,
            _ => false,
        }
        && a.guide.len() == b.guide.len()
        && a.guide.iter().zip(&b.guide).all(|(a, b)| {
            a.title == b.title
                && a.why == b.why
                && a.action == b.action
                && a.commands.len() == b.commands.len()
                && a.commands.iter().zip(&b.commands).all(|(a, b)| {
                    a.command == b.command
                        && a.explanation == b.explanation
                        && a.simulator_only == b.simulator_only
                })
        })
}
fn foreign(w: &model::WorkspaceData) -> bool {
    if w.one_off_session.is_some() || w.one_off_learning.is_some() {
        return true;
    }
    let model::WorkspaceData {
        physical_link_draft,
        physical_link_learning,
        one_off_session: _,
        one_off_learning: _,
        file_backup_learning,
        etherchannel_draft,
        etherchannel_learning,
        cron_session,
        cron_learning,
        nfs_session: _,
        nfs_learning: _,
        rpm_session,
        rpm_learning,
        nat_draft,
        nat_learning,
        debian_packages_terminal,
        apache_static_draft,
        apache_static_learning,
        debian_packages_draft,
        debian_packages_learning,
        spanning_tree_draft,
        spanning_tree_learning,
        dhcpv4_draft,
        dhcpv4_learning,
        version: _,
        unit_manager_draft,
        unit_manager_learning,
        unit_manager_terminal,
        ospf_draft,
        ospf_learning,
        acl_draft,
        acl_learning,
        script_learning,
        script_history: _,
        script_history_more: _,
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
    } = w;
    false
        || physical_link_draft.is_some()
        || physical_link_learning.is_some()
        || file_backup_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || cron_session.is_some()
        || cron_learning.is_some()
        || rpm_session.is_some()
        || rpm_learning.is_some()
        || nat_draft.is_some()
        || nat_learning.is_some()
        || debian_packages_terminal.is_some()
        || apache_static_draft.is_some()
        || apache_static_learning.is_some()
        || debian_packages_draft.is_some()
        || debian_packages_learning.is_some()
        || spanning_tree_draft.is_some()
        || spanning_tree_learning.is_some()
        || dhcpv4_draft.is_some()
        || dhcpv4_learning.is_some()
        || unit_manager_draft.is_some()
        || unit_manager_learning.is_some()
        || unit_manager_terminal.is_some()
        || ospf_draft.is_some()
        || ospf_learning.is_some()
        || acl_draft.is_some()
        || acl_learning.is_some()
        || script_learning.is_some()
        || terminal_learning.is_some()
        || process_learning.is_some()
        || accounts_learning.is_some()
        || discovery_draft.is_some()
        || discovery_learning.is_some()
        || storage_draft.is_some()
        || storage_learning.is_some()
        || ipv6_draft.is_some()
        || ipv6_learning.is_some()
        || routing_v4_draft.is_some()
        || routing_v4_learning.is_some()
        || address_learning.is_some()
        || service_learning.is_some()
        || client_learning.is_some()
        || seed_dns_learning.is_some()
        || seed_mount_learning.is_some()
        || legacy_access_learning.is_some()
        || legacy_logs_learning.is_some()
        || client_draft.is_some()
        || dns_learning.is_some()
        || dns_draft.is_some()
        || switching_draft.is_some()
        || switching_learning.is_some()
        || mac_draft.is_some()
        || mac_learning.is_some()
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
        && a.nfs_session == b.nfs_session
        && a.nfs_learning == b.nfs_learning
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
        && a.script_history == b.script_history
        && a.script_history_more == b.script_history_more
}

// Streaming structural counter; never builds JSON, Value or a formatted address String.
// Every value/key/collection is charged 128 bytes in addition to every string byte.
#[derive(Debug)]
struct Limit;
impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NFS memo storage ceiling")
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
            .checked_add(4096)
    }
}
pub(in crate::ui) fn charged<T: Serialize + ?Sized>(value: &T, cap: usize) -> Option<usize> {
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
    #[test]
    fn admitted_backup_metadata_cannot_hit_warm_success_or_copy_foreign_payloads() {
        let s = sources().remove(0);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let good = a.clone();
        let source = s.clone();
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../../scenarios/file-backup-v1.json")).unwrap();
        let lesson: dlms_learning_lab::file_backup_teaching::Lesson =
            serde_json::from_value(raw["scenarios"][0]["file_backup_teaching"].clone()).unwrap();
        let learning = dlms_learning_lab::file_backup_teaching::Learning::new(&lesson).unwrap();
        assert!(learning.valid());
        for corrupt_source in [false, true] {
            let mut s = source.clone();
            a = good.clone();
            let mut m = Memo::default();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            let before = m.counters;
            if corrupt_source {
                s.file_backup_teaching = Some(lesson.clone());
            } else {
                a.workspace.as_mut().unwrap().file_backup_learning = Some(learning.clone());
            }
            let caller = serde_json::to_value(&a).unwrap();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(false));
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert_eq!(m.counters.meters, before.meters);
            assert!(m.entries.is_empty());
            assert_eq!(serde_json::to_value(&a).unwrap(), caller);
            if !corrupt_source {
                assert!(!progress::current_solved(&a, &s));
            }
        }
    }

    fn sources() -> Vec<Scenario> {
        crate::ui::tests::nfs_workspace::sources()
    }
    fn prepared(s: &Scenario) -> Attempt {
        let mut a = Attempt::new(s, 1);
        learn::prepare(&mut a, s, true).unwrap();
        a
    }
    fn solve(a: &mut Attempt, s: &Scenario) {
        for command in &s.solution {
            let action =
                core::Action::from_json(command.strip_prefix(work::PREFIX).unwrap()).unwrap();
            work::perform_attempt(a, s, &action).unwrap();
        }
        assert!(progress::current_solved(a, s));
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
    fn maximum_session(escape: bool) -> work::Session {
        let mut t = work::Session {
            relative: "x".repeat(160),
            write_text: "x".repeat(4096),
            entries: (0..16)
                .map(|_| work::Entry {
                    operation: "o".repeat(160),
                    detail: String::new(),
                })
                .collect(),
            ..Default::default()
        };
        assert!(t.valid());
        for i in 0..16 {
            let left = work::MAX_SESSION - serde_json::to_vec(&t).unwrap().len();
            if left == 0 {
                break;
            }
            let (unit, cost) = if escape { ('\u{1}', 6) } else { ('x', 1) };
            let n = (left / cost).min(4096);
            let mut text = unit.to_string().repeat(n);
            text.push_str(&"x".repeat((left - n * cost).min(4096 - text.len())));
            t.entries[i].detail = text;
        }
        assert_eq!(t.entries.len(), 16);
        assert!(t.valid());
        assert_eq!(serde_json::to_vec(&t).unwrap().len(), work::MAX_SESSION);
        t
    }
    #[test]
    fn four_complete_maximum_session_keys_both_reports_fit_and_idle_does_not_serialize() {
        let sources = sources();
        let mut owned = Vec::new();
        for escape in [false, true] {
            let mut m = Memo::default();
            let mut total = 0;
            for s in &sources {
                let mut a = prepared(s);
                a.workspace.as_mut().unwrap().nfs_session = Some(maximum_session(escape));
                a.workspace
                    .as_mut()
                    .unwrap()
                    .nfs_learning
                    .as_mut()
                    .unwrap()
                    .session = maximum_session(escape);
                assert!(progress::current_nfs_checks(&a, s).first().unwrap().0);
                let value = serde_json::to_value((s, &a)).unwrap();
                let (raw, n) = independent(&value);
                assert_eq!(charged(&(s, &a), usize::MAX), Some(raw + 128 * n + 4096));
                assert_eq!(
                    compact_size(&(s, &a), usize::MAX),
                    Some(serde_json::to_vec(&(s, &a)).unwrap().len())
                );
                let c = charge(&(s, &a), ENTRY_BYTES).unwrap();
                total += c;
                m.evaluate(&a, s, false);
                assert!(m.rows(&s.id, false).is_some_and(|r| !r.is_empty()));
                assert!(m.rows(&s.id, true).is_some_and(|r| !r.is_empty()));
                owned.push((s.clone(), a));
            }
            assert_eq!(m.entries.len(), 4);
            assert_eq!(m.bytes, total);
            assert!(m.bytes + FRAME_RESERVE + EDITOR_RESERVE <= TOTAL_BYTES);
            let before = m.counters;
            for _ in 0..12 {
                m.begin_frame();
                for (s, a) in &owned[owned.len() - 4..] {
                    m.evaluate(a, s, false);
                }
            }
            assert_eq!(m.counters.evaluations, before.evaluations);
            assert_eq!(m.counters.meters, before.meters);
            assert_eq!(m.counters.snapshots, before.snapshots);
        }
    }
    #[test]
    fn genuine_history_and_actual_quota_fallback_remain_truthful_with_no_snapshot() {
        let s = sources().remove(0);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-v8-synthetic-writer-progress.json"
        ))
        .unwrap();
        let value = raw["attempts"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|a| (!a["state"]["run"].is_null()).then(|| a["state"]["run"].clone()))
            .unwrap();
        let record: shell::Record = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(record.engine_revision, 8);
        let w = a.workspace.as_mut().unwrap();
        w.script_history = Some(record.clone());
        w.script_history_more = vec![record; 31];
        assert!(w.valid());
        let before = serde_json::to_value(&a).unwrap();
        let mut m = Memo::default();
        m.evaluate_limits(&a, &s, false, 1, 1, 0);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert!(m.rows(&s.id, false).unwrap().iter().all(|r| r.0));
        assert_eq!(m.counters.snapshots, 0);
        let c = m.counters;
        for _ in 0..20 {
            assert_eq!(m.prepared(&s.id), Some(true));
        }
        assert_eq!(m.counters, c);
        assert!(serde_json::to_value(&a).unwrap() == before);
        m.begin_frame();
        m.evaluate_limits(
            &a,
            &s,
            false,
            ENTRY_BYTES,
            TOTAL_BYTES - EDITOR_RESERVE - FRAME_RESERVE,
            4,
        );
        assert_eq!(m.counters.snapshots, 1);
        assert!(serde_json::to_value(&a).unwrap() == before);
    }
    #[test]
    fn complete_source_context_and_foreign_fields_cannot_hit_or_copy_warm_success() {
        let s = sources().remove(0);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let good = a.clone();
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        let before = m.counters;
        a.workspace.as_mut().unwrap().rpm_session =
            Some(dlms_learning_lab::rpm_packages_terminal::Session::default());
        let caller = serde_json::to_value(&a).unwrap();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert!(serde_json::to_value(&a).unwrap() == caller);
        a = good;
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        let before = m.counters;
        let mut changed = s.clone();
        changed.nfs_teaching.as_mut().unwrap().concepts[0]
            .text
            .push_str(" Reviewed prose change.");
        m.evaluate(&a, &changed, false);
        assert_eq!(m.counters.evaluations, before.evaluations + 1);
        assert_eq!(m.counters.snapshots, before.snapshots + 1);
        assert_eq!(m.prepared(&s.id), Some(true));
        changed.version += 1;
        let before = m.counters;
        m.evaluate(&a, &changed, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
    }
    #[test]
    fn frame_storage_reserve_and_lru_are_real_bounded_owned_storage() {
        let frame = FrameGrades::default();
        let frames = frame.values.capacity() * std::mem::size_of::<FrameGrade>();
        let report = 2 * (64 * std::mem::size_of::<(bool, String)>() + 37 * 512)
            + std::mem::size_of::<Report>()
            + 80;
        assert!(frames + report + std::mem::size_of::<FrameGrades>() <= FRAME_RESERVE);
        let sources = sources();
        let mut m = Memo::default();
        for s in &sources {
            let a = prepared(s);
            m.evaluate_limits(
                &a,
                s,
                false,
                ENTRY_BYTES,
                TOTAL_BYTES - EDITOR_RESERVE - FRAME_RESERVE,
                2,
            );
        }
        assert_eq!(m.entries.len(), 2);
        assert!(
            m.entries
                .iter()
                .all(|e| e.source.id == sources[2].id || e.source.id == sources[3].id)
        );
        assert!(m.bytes <= TOTAL_BYTES - EDITOR_RESERVE - FRAME_RESERVE);
        m.retain(|_| false);
        assert_eq!(m.bytes, 0);
    }
    #[test]
    fn admitted_etherchannel_payloads_cannot_reuse_nfs_warm_success_or_copy() {
        let s = sources().remove(0);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let good = a.clone();
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/etherchannel-workbench-v1.json"
        ))
        .unwrap();
        let lesson: dlms_learning_lab::etherchannel_teaching::Lesson =
            serde_json::from_value(raw["cases"][0]["lesson"].clone()).unwrap();
        let learning =
            dlms_learning_lab::etherchannel_teaching::Learning::new(&lesson, true).unwrap();
        assert!(learning.valid());
        for (draft, practice) in [(true, false), (false, true), (true, true)] {
            a = good.clone();
            let mut m = Memo::default();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            let before = m.counters;
            let w = a.workspace.as_mut().unwrap();
            if draft {
                w.etherchannel_draft = Some(
                    dlms_learning_lab::etherchannel_workspace::Draft::from_state(
                        &lesson.definition.initial,
                    )
                    .unwrap(),
                );
            }
            if practice {
                w.etherchannel_learning = Some(learning.clone());
            }
            assert!(w.valid());
            let caller = serde_json::to_value(&a).unwrap();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(false));
            assert_eq!(m.counters.meters, before.meters);
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert_eq!(m.counters.evaluations, before.evaluations);
            assert!(m.entries.iter().all(|e| e.source.id != s.id));
            assert_eq!(serde_json::to_value(&a).unwrap(), caller);
            assert!(!progress::current_solved(&a, &s));
            m.evaluate(&good, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
        }
    }
    #[test]
    fn cron_metadata_and_teaching_cannot_reuse_a_warm_success_or_copy_foreign_input() {
        let cron_source = pack::load(include_str!(
            "../../../tests/fixtures/cron-workbench-v1.json"
        ))
        .unwrap()
        .remove(0);
        let engine::Goal::Cron { definition } = &cron_source.goal else {
            unreachable!()
        };
        let session = dlms_learning_lab::cron_workspace::Session::new(definition).unwrap();
        let lesson = cron_source.cron_teaching.as_ref().unwrap();
        let learning = dlms_learning_lab::cron_teaching::Learning::new(lesson).unwrap();
        let source = sources().remove(0);
        let mut good = prepared(&source);
        solve(&mut good, &source);
        assert!(progress::current_solved(&good, &source));
        for payload in 0..4 {
            let mut source = source.clone();
            let mut a = good.clone();
            let mut m = Memo::default();
            m.evaluate(&a, &source, false);
            assert_eq!(m.prepared(&source.id), Some(true));
            let cold = m.counters;
            m.evaluate(&a, &source, false);
            assert_eq!(m.counters.hits, cold.hits + 1);
            assert_eq!(m.counters.meters, cold.meters);
            assert_eq!(m.counters.snapshots, cold.snapshots);
            let before = m.counters;
            match payload {
                0 => a.workspace.as_mut().unwrap().cron_session = Some(session.clone()),
                1 => a.workspace.as_mut().unwrap().cron_learning = Some(learning.clone()),
                2 => {
                    let w = a.workspace.as_mut().unwrap();
                    w.cron_session = Some(session.clone());
                    w.cron_learning = Some(learning.clone());
                }
                3 => source.cron_teaching = Some(lesson.clone()),
                _ => unreachable!(),
            }
            assert!(a.workspace.as_ref().unwrap().valid());
            let caller = serde_json::to_vec(&a).unwrap();
            let authored = serde_json::to_vec(&source).unwrap();
            m.evaluate(&a, &source, false);
            assert_eq!(m.prepared(&source.id), Some(false), "payload {payload}");
            assert_eq!(m.counters.hits, before.hits);
            assert_eq!(m.counters.meters, before.meters);
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert!(!progress::current_solved(&a, &source));
            assert_eq!(serde_json::to_vec(&a).unwrap(), caller);
            assert_eq!(serde_json::to_vec(&source).unwrap(), authored);
            source.cron_teaching = None;
            m.evaluate(&good, &source, false);
            assert_eq!(m.prepared(&source.id), Some(true));
        }
    }
    #[test]
    fn valid_physical_source_and_each_optional_field_refuse_a_warm_success_before_copy() {
        let source = sources().remove(0);
        let mut a = prepared(&source);
        solve(&mut a, &source);
        let good = a.clone();
        let (physical, p) = crate::ui::tests::physical_workspace::prepared();
        let pw = p.workspace.as_ref().unwrap();
        assert!(pw.valid());
        for mask in 0..4 {
            let mut source = source.clone();
            a = good.clone();
            let mut m = Memo::default();
            m.begin_frame();
            m.evaluate(&a, &source, false);
            assert_eq!(m.prepared(&source.id), Some(true));
            let before = m.counters;
            if mask == 0 {
                source.physical_link_teaching = physical.physical_link_teaching.clone();
            } else {
                let w = a.workspace.as_mut().unwrap();
                if mask & 1 != 0 {
                    w.physical_link_draft = pw.physical_link_draft.clone();
                }
                if mask & 2 != 0 {
                    w.physical_link_learning = pw.physical_link_learning.clone();
                }
                assert!(w.valid());
            }
            let caller = serde_json::to_value(&a).unwrap();
            m.evaluate(&a, &source, false);
            assert_eq!(m.prepared(&source.id), Some(false));
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert_eq!(m.counters.meters, before.meters);
            assert!(m.entries.is_empty());
            assert_eq!(serde_json::to_value(&a).unwrap(), caller);
            source.physical_link_teaching = None;
            a = good.clone();
            m.begin_frame();
            m.evaluate(&a, &source, false);
            assert_eq!(
                m.prepared(&source.id),
                Some(true),
                "Removing foreign physical fields must restore legitimate own completion"
            );
        }
    }
}
