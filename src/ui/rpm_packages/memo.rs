//! RPM-only optimization. Complete borrowed keys, no persisted completion authority.
use super::*;
use serde::{
    Serialize,
    ser::{self, SerializeMap, SerializeSeq, SerializeStruct},
};
use std::fmt;
pub(super) const ENTRY_BYTES: usize = 4 * 1024 * 1024;
pub(super) const TOTAL_BYTES: usize = 16 * 1024 * 1024;
const ENTRIES: usize = 16;
const FRAME_RESERVE: usize = 512 * 1024;
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
    source_version: u32,
    initial: core::State,
    definition: work::Definition,
    lesson: Option<learn::Lesson>,
    attempt: Attempt,
    rows: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
    charge: usize,
    touched: u64,
}
struct FrameReport {
    id: String,
    rows: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
}
#[derive(Default)]
pub(in crate::ui) struct Memo {
    entries: Vec<Entry>,
    bytes: usize,
    tick: u64,
    pub counters: Counters,
    frame: std::collections::BTreeMap<String, bool>,
    fallback: Option<FrameReport>,
}
impl Memo {
    pub fn begin_frame(&mut self) {
        self.frame.clear();
        self.fallback = None;
    }
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.entries.retain(|e| keep(&e.id));
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
    }
    pub fn prepared(&self, id: &str) -> Option<bool> {
        self.frame.get(id).copied()
    }
    pub fn rows(&self, id: &str, practice: bool) -> Option<&[(bool, String)]> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| {
                if practice {
                    e.practice.as_slice()
                } else {
                    e.rows.as_slice()
                }
            })
            .or_else(|| {
                self.fallback
                    .as_ref()
                    .filter(|report| report.id == id)
                    .map(|report| {
                        if practice {
                            report.practice.as_slice()
                        } else {
                            report.rows.as_slice()
                        }
                    })
            })
    }
    fn frame(&mut self, id: &str, passed: bool) {
        if id.len() <= 80 && (self.frame.contains_key(id) || self.frame.len() < 1000) {
            self.frame.insert(id.into(), passed);
        }
    }
    fn fallback(&mut self, id: &str, rows: Vec<(bool, String)>, practice: Vec<(bool, String)>) {
        self.counters.fallbacks += 1;
        if id.len() <= 80 {
            self.fallback = Some(FrameReport {
                id: id.into(),
                rows,
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
            TOTAL_BYTES - FRAME_RESERVE,
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
        if s.physical_link_teaching.is_some() {
            self.entries.retain(|e| e.id != s.id);
            self.bytes = self.entries.iter().map(|e| e.charge).sum();
            self.frame(&s.id, false);
            self.fallback(
                &s.id,
                vec![(
                    false,
                    "Physical-link teaching belongs to another activity".into(),
                )],
                Vec::new(),
            );
            return;
        }
        if s.file_backup_teaching.is_some() {
            self.entries.retain(|e| e.id != s.id);
            self.bytes = self.entries.iter().map(|e| e.charge).sum();
            self.frame(&s.id, false);
            self.fallback(
                &s.id,
                vec![(false, "Backup teaching belongs to another activity".into())],
                Vec::new(),
            );
            return;
        }
        let (
            engine::State::RpmPackages {
                simulation: initial,
            },
            engine::Goal::RpmPackages { definition },
            engine::State::RpmPackages { .. },
        ) = (&s.initial, &s.goal, &a.state)
        else {
            self.entries.retain(|e| e.id != s.id);
            self.bytes = self.entries.iter().map(|e| e.charge).sum();
            self.frame(&s.id, false);
            self.fallback(
                &s.id,
                vec![(false, "RPM assignment kind differs".into())],
                Vec::new(),
            );
            return;
        };
        let definition = definition.as_ref();
        self.tick = self.tick.saturating_add(1);
        let shape = s.etherchannel_teaching.is_none()
            && storage_shape(a)
            && s.id.len() <= 80
            && s.cron_teaching.is_none()
            && s.one_off_teaching.is_none();
        if !force
            && shape
            && let Some(e) = self.entries.iter_mut().find(|e| {
                e.id == s.id
                    && e.source_version == s.version
                    && &e.initial == initial
                    && &e.definition == definition
                    && e.lesson == s.rpm_packages_teaching
                    && same_attempt(&e.attempt, a)
            })
        {
            e.touched = self.tick;
            let passed = e.rows.iter().all(|r| r.0);
            self.counters.hits += 1;
            self.frame(&s.id, passed);
            return;
        }
        self.counters.evaluations += 1;
        let rows = progress::current_rpm_checks(a, s);
        let practice = match (
            a.workspace.as_ref().and_then(|w| w.rpm_learning.as_ref()),
            s.rpm_packages_teaching.as_ref(),
        ) {
            (Some(l), Some(source)) => l.checks(source),
            _ => Vec::new(),
        };
        let passed = rows.iter().all(|r| r.0);
        self.frame(&s.id, passed);
        self.entries.retain(|e| e.id != s.id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        let report_count = rows.len().saturating_add(practice.len());
        let report_strings = rows
            .iter()
            .chain(practice.iter())
            .try_fold(0usize, |n, r| n.checked_add(r.1.len()));
        let report_ok = report_count <= 128
            && report_strings.is_some_and(|strings| {
                strings <= 32 * 1024
                    && report_count
                        .checked_mul(3)
                        .and_then(|n| n.checked_add(2))
                        .and_then(|n| n.checked_mul(128))
                        .and_then(|n| n.checked_add(strings))
                        .is_some_and(|bytes| bytes <= REPORT_RESERVE)
            });
        let practice_passed = practice.iter().all(|r| r.0);
        if !report_ok {
            self.fallback(
                &s.id,
                vec![(
                    passed,
                    "Current checks completed; details exceed the display bound".into(),
                )],
                vec![(
                    practice_passed,
                    "Example checks completed; details exceed the display bound".into(),
                )],
            );
            return;
        }
        if !shape || entries == 0 {
            self.fallback(&s.id, rows, practice);
            return;
        }
        self.counters.meters += 1;
        let Some(charge) = charged(
            &(
                &s.id,
                s.version,
                initial,
                definition,
                &s.rpm_packages_teaching,
                a,
            ),
            entry_cap.min(total_cap),
        ) else {
            self.fallback(&s.id, rows, practice);
            return;
        };
        while !self.entries.is_empty()
            && (self.entries.len() >= entries || self.bytes.saturating_add(charge) > total_cap)
        {
            let i = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.touched)
                .unwrap()
                .0;
            self.bytes -= self.entries.remove(i).charge;
        }
        if charge > total_cap {
            self.fallback(&s.id, rows, practice);
            return;
        }
        self.counters.snapshots += 1;
        self.entries.push(Entry {
            id: s.id.clone(),
            source_version: s.version,
            initial: initial.clone(),
            definition: definition.clone(),
            lesson: s.rpm_packages_teaching.clone(),
            attempt: a.clone(),
            rows,
            practice,
            charge,
            touched: self.tick,
        });
        self.bytes += charge;
    }
}
pub(super) fn foreign(w: &model::WorkspaceData) -> bool {
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
        nfs_session,
        nfs_learning,
        rpm_session: _,
        rpm_learning: _,
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
        || nfs_session.is_some()
        || nfs_learning.is_some()
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
        && a.rpm_session == b.rpm_session
        && a.rpm_learning == b.rpm_learning
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
fn storage_shape(a: &Attempt) -> bool {
    if a.transcript.len() > 1024 || a.transcript.iter().any(|s| s.len() > 4096) {
        return false;
    }
    a.workspace.as_ref().is_none_or(|w| {
        !foreign(w)
            && w.command.len() <= 4096
            && w.script_history_more.len() + usize::from(w.script_history.is_some()) <= 32
    })
}
// Streaming structural counter; never builds JSON, Value or a formatted address String.
// Every value/key/collection is charged 128 bytes in addition to every string byte.
#[derive(Debug)]
struct Limit;
impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RPM memo storage ceiling")
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
    #[test]
    fn admitted_backup_metadata_cannot_hit_warm_success_or_copy_foreign_payloads() {
        let s = lessons().remove(0);
        let mut a = attempt(&s);
        solve(&s, &mut a);
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

    fn lessons() -> Vec<Scenario> {
        pack::load_catalog(&[
            include_str!("../../../scenarios/linux-files-v1.json"),
            include_str!("../../../scenarios/rpm-packages-v1.json"),
        ])
        .unwrap()
        .into_iter()
        .filter(|s| matches!(s.initial, engine::State::RpmPackages { .. }))
        .collect()
    }
    fn attempt(s: &Scenario) -> Attempt {
        let mut a = Attempt::new(s, 1);
        model::prepare(&mut a);
        let engine::Goal::RpmPackages { definition } = &s.goal else {
            panic!()
        };
        learn::prepare(
            &mut a,
            definition,
            s.rpm_packages_teaching.as_ref().unwrap(),
            true,
        )
        .unwrap();
        a
    }
    fn solve(s: &Scenario, a: &mut Attempt) {
        for c in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
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
    /// Synthetic maximum imported display metadata, explicitly not a utility observation.
    fn maximum_session(escaped: bool, state: &core::State) -> Session {
        let mut t = Session {
            command: "x".repeat(terminal::MAX_COMMAND),
            selected_path: state
                .catalog()
                .allowed_paths
                .iter()
                .max_by_key(|p| p.len())
                .cloned(),
            entries: (0..16)
                .map(|_| terminal::Entry {
                    command: "x".repeat(terminal::MAX_COMMAND),
                    output: Outcome::Utility {
                        stdout: String::new(),
                        stderr: String::new(),
                        status: 0,
                        summary: Some("s".repeat(512)),
                    },
                })
                .collect(),
            ..Default::default()
        };
        assert!(t.valid_for(state));
        // Reserve all16 entries and their maximum commands first. Fill only remaining bytes.
        for index in 0..16 {
            let bytes = serde_json::to_vec(&t).unwrap().len();
            let left = terminal::MAX_SESSION - bytes;
            if left == 0 {
                break;
            }
            let (unit, cost) = if escaped { ('\u{1}', 6) } else { ('x', 1) };
            let n = (left / cost).min(terminal::MAX_OUTPUT);
            let mut value = unit.to_string().repeat(n);
            // Fill the sub-escape remainder with ASCII, retaining the per-output byte cap.
            let plain = (left - n * cost).min(terminal::MAX_OUTPUT - value.len());
            value.push_str(&"x".repeat(plain));
            if let Outcome::Utility { stdout, .. } = &mut t.entries[index].output {
                *stdout = value;
            }
        }
        assert_eq!(t.entries.len(), 16);
        assert!(
            t.entries
                .iter()
                .all(|e| e.command.len() == terminal::MAX_COMMAND)
        );
        assert_eq!(serde_json::to_vec(&t).unwrap().len(), terminal::MAX_SESSION);
        assert!(t.valid_for(state));
        t
    }
    fn maximum_runtime(state: &core::State) -> core::State {
        let mut runtime = state.runtime().clone();
        runtime.installed.clear();
        for (id, a) in &state.catalog().archives {
            if !runtime
                .installed
                .iter()
                .any(|i| i.identity.name == a.identity.name)
            {
                runtime.installed.push(core::Installed {
                    identity: a.identity.clone(),
                    archive: id.clone(),
                });
            }
        }
        runtime.cached = Some(state.catalog().published.clone());
        for path in &state.catalog().allowed_paths {
            runtime.files.insert(
                path.clone(),
                core::File {
                    contents: "x".repeat(4096),
                    mode: 0o777,
                    owner: "u".repeat(32),
                    group: "g".repeat(32),
                    mtime: u64::MAX,
                },
            );
        }
        let result = core::State::new(state.catalog().clone(), runtime).unwrap();
        assert!(work::admitted(&result));
        result
    }
    #[test]
    fn compact_preflight_counts_real_escape_bytes_and_checked_overflow_without_a_buffer() {
        use std::io::Write;
        let value = ("é\n\t\u{0001}\"\\", vec!["second", "line"]);
        let size = serde_json::to_vec(&value).unwrap().len();
        assert!(within(&value, size));
        assert!(!within(&value, size - 1));
        let mut sink = CompactSize {
            bytes: usize::MAX,
            limit: usize::MAX,
        };
        assert!(sink.write_all(b"x").is_err());
        assert_eq!(sink.bytes, usize::MAX);
        let mut sink = CompactSize { bytes: 0, limit: 3 };
        assert!(sink.write_all(b"four").is_err());
        assert_eq!(sink.bytes, 0);
    }
    #[test]
    fn all_four_complete_keys_both_maximum_sessions_models_and_reports_fit_one_aggregate() {
        let sources = lessons();
        assert_eq!(sources.len(), 4);
        let mut aggregate_charges = [0; 2];
        for (case, escaped) in [false, true].into_iter().enumerate() {
            let mut m = Memo::default();
            let mut combined = 0;
            for s in &sources {
                let mut a = attempt(s);
                let engine::State::RpmPackages { simulation } = &mut a.state else {
                    panic!()
                };
                *simulation = maximum_runtime(simulation);
                let w = a.workspace.as_mut().unwrap();
                w.rpm_session = Some(maximum_session(escaped, simulation));
                let l = w.rpm_learning.as_mut().unwrap();
                l.simulation = maximum_runtime(&l.simulation);
                l.terminal = maximum_session(escaped, &l.simulation);
                assert!(l.valid_for(s.rpm_packages_teaching.as_ref().unwrap()));
                assert!(within(&a, work::MAX_ATTEMPT));
                let engine::State::RpmPackages {
                    simulation: initial,
                } = &s.initial
                else {
                    panic!()
                };
                let engine::Goal::RpmPackages { definition } = &s.goal else {
                    panic!()
                };
                let key = (
                    &s.id,
                    s.version,
                    initial,
                    definition.as_ref(),
                    &s.rpm_packages_teaching,
                    &a,
                );
                let (strings, tokens) = independent(&serde_json::to_value(key).unwrap());
                let expected = strings + tokens * 128 + 4096 + REPORT_RESERVE;
                assert_eq!(charged(&key, ENTRY_BYTES), Some(expected));
                combined += expected;
                m.evaluate(&a, s, false);
                assert_eq!(m.prepared(&s.id), Some(progress::current_solved(&a, s)));
                assert!(m.entries.iter().any(|e| e.id == s.id));
                let before = m.counters;
                for _ in 0..10 {
                    m.begin_frame();
                    m.evaluate(&a, s, false);
                }
                assert_eq!(m.counters.evaluations, before.evaluations);
                assert_eq!(m.counters.meters, before.meters);
                assert_eq!(m.counters.snapshots, before.snapshots);
            }
            assert_eq!(
                m.entries.len(),
                4,
                "One entry per activity contains both views"
            );
            assert_eq!(m.bytes, combined);
            assert!(combined + FRAME_RESERVE <= TOTAL_BYTES);
            aggregate_charges[case] = combined;
        }
        assert!(
            aggregate_charges[0] > aggregate_charges[1],
            "Plain bytes charge more raw strings than equally full escaped JSON"
        );
        assert!(aggregate_charges.into_iter().max().unwrap() + FRAME_RESERVE <= TOTAL_BYTES);
        let mut frame = Memo::default();
        for n in 0..1000 {
            frame.frame(&format!("{n:080}"), false);
        }
        frame.frame("cannot-add-a-1001st-key", false);
        assert_eq!(frame.frame.len(), 1000);
        let (strings, tokens) = independent(&serde_json::to_value(&frame.frame).unwrap());
        assert!(strings + 128 * tokens + 4096 + REPORT_RESERVE <= FRAME_RESERVE);
    }
    #[test]
    fn same_id_all_goal_lesson_metadata_and_current_changes_miss_and_never_grant_from_output() {
        let mut s = lessons().remove(0);
        let mut a = attempt(&s);
        solve(&s, &mut a);
        let good = a.clone();
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        let before = m.counters;
        s.rpm_packages_teaching
            .as_mut()
            .unwrap()
            .example_description
            .push_str(" A different bounded source.");
        m.evaluate(&a, &s, false);
        assert_eq!(m.counters.evaluations, before.evaluations + 1);
        assert_eq!(m.prepared(&s.id), Some(true));
        let nat = pack::load(include_str!(
            "../../../tests/fixtures/nat-teaching-preview.json"
        ))
        .unwrap()
        .remove(0);
        a.workspace.as_mut().unwrap().nat_learning = Some(
            dlms_learning_lab::nat_teaching::Learning::new(nat.nat_teaching.as_ref().unwrap())
                .unwrap(),
        );
        let before = m.counters;
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert!(m.entries.is_empty());
        a = good;
        m.evaluate(&a, &s, false);
        let engine::Goal::RpmPackages { definition } = &mut s.goal else {
            panic!()
        };
        definition.requirements.push(work::Requirement::Absent {
            package: core::Identity {
                name: "ledger-client".into(),
                arch: core::Architecture::Noarch,
            },
        });
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
    }
    #[test]
    fn true_quota_fallback_retains_genuine_history_and_lru_uses_complete_key_charge() {
        let sources = lessons();
        let s = &sources[0];
        let mut a = attempt(s);
        solve(s, &mut a);
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-v8-synthetic-writer-progress.json"
        ))
        .unwrap();
        let record = raw["attempts"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|v| {
                let r = &v["state"]["run"];
                (!r.is_null()).then(|| serde_json::from_value::<shell::Record>(r.clone()).unwrap())
            })
            .unwrap();
        assert_eq!(record.engine_revision, 8);
        assert!(record.valid());
        a.workspace.as_mut().unwrap().script_history = Some(record);
        assert!(progress::current_solved(&a, s));
        let caller = serde_json::to_value(&a).unwrap();
        let mut m = Memo::default();
        m.evaluate_limits(&a, s, false, 1, 1, 0);
        for _ in 0..20 {
            assert_eq!(m.prepared(&s.id), Some(true));
        }
        assert_eq!(m.counters.evaluations, 1);
        assert_eq!(m.counters.snapshots, 0);
        assert!(m.rows(&s.id, false).unwrap().iter().all(|r| r.0));
        assert!(serde_json::to_value(&a).unwrap() == caller);
        let mut n = Memo::default();
        for s in &sources {
            let a = attempt(s);
            n.evaluate_limits(&a, s, false, ENTRY_BYTES, TOTAL_BYTES - FRAME_RESERVE, 2);
            assert!(n.entries.len() <= 2);
            assert!(n.bytes <= TOTAL_BYTES - FRAME_RESERVE);
        }
        assert_eq!(n.entries.len(), 2);
        assert!(
            n.entries
                .iter()
                .all(|e| e.id == sources[2].id || e.id == sources[3].id)
        );
        n.retain(|_| false);
        assert_eq!(n.bytes, 0);
    }
    #[test]
    fn admitted_nfs_metadata_cannot_hit_warm_success_or_copy_a_foreign_snapshot() {
        let s = lessons().remove(0);
        let mut a = attempt(&s);
        solve(&s, &mut a);
        let good = a.clone();
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/nfs-workbench-v1.json"
        ))
        .unwrap();
        let lesson: dlms_learning_lab::nfs_teaching::Lesson =
            serde_json::from_value(raw["cases"][0]["lesson"].clone()).unwrap();
        let learning = dlms_learning_lab::nfs_teaching::Learning::new(&lesson).unwrap();
        assert!(learning.valid());
        for (session, practice) in [(true, false), (false, true), (true, true)] {
            a = good.clone();
            let mut m = Memo::default();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            let before = m.counters;
            let w = a.workspace.as_mut().unwrap();
            if session {
                w.nfs_session = Some(dlms_learning_lab::nfs_workspace::Session::default());
            }
            if practice {
                w.nfs_learning = Some(learning.clone());
            }
            assert!(w.valid());
            let caller = serde_json::to_value(&a).unwrap();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(false));
            assert_eq!(m.counters.meters, before.meters);
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert_eq!(m.counters.evaluations, before.evaluations + 1);
            assert!(m.entries.is_empty());
            assert!(
                serde_json::to_value(&a).unwrap() == caller,
                "complete caller unchanged"
            );
            assert!(!progress::current_solved(&a, &s));
            a = good.clone();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            assert_eq!(m.counters.snapshots, before.snapshots + 1);
        }
    }
    #[test]
    fn admitted_etherchannel_metadata_cannot_hit_warm_success_or_copy_a_foreign_snapshot() {
        let s = lessons().remove(0);
        let mut a = attempt(&s);
        solve(&s, &mut a);
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
        for (session, practice) in [(true, false), (false, true), (true, true)] {
            a = good.clone();
            let mut m = Memo::default();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            let before = m.counters;
            let w = a.workspace.as_mut().unwrap();
            if session {
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
            assert_eq!(m.counters.evaluations, before.evaluations + 1);
            assert!(m.entries.is_empty());
            assert!(
                serde_json::to_value(&a).unwrap() == caller,
                "complete caller unchanged"
            );
            assert!(!progress::current_solved(&a, &s));
            a = good.clone();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            assert_eq!(m.counters.snapshots, before.snapshots + 1);
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
        let source = lessons().remove(0);
        let mut good = attempt(&source);
        solve(&source, &mut good);
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
        let source = lessons().remove(0);
        let mut a = attempt(&source);
        solve(&source, &mut a);
        let good = a.clone();
        let (physical, p) = crate::ui::tests::physical_workspace::prepared();
        let pw = p.workspace.as_ref().unwrap();
        assert!(pw.valid());
        for mask in 0..4 {
            let mut source = source.clone();
            a = good.clone();
            let mut m = Memo::default();
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
