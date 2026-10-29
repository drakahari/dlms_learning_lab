//! Exact complete-input LACP memo; never saved or a completion credential.
use super::*;
use serde::{
    Serialize,
    ser::{self, SerializeMap, SerializeSeq, SerializeStruct},
};
use std::fmt;
pub(super) const ENTRY_BYTES: usize = 8 * 1024 * 1024;
pub(super) const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const ENTRIES: usize = 4;
pub(super) const EDITOR_RESERVE: usize = 4 * 1024 * 1024;
const FRAME_RESERVE: usize = 256 * 1024;
// The final4096 also covers128 fixed bytes of typed pair observations.
// Their bounded query-free transient snapshots/plan fit the256KiB frame reserve.
const REPORT_RESERVE: usize = 2 * 33 * (512 + 128) + 4096;
type PairCounts = [[Option<u8>; 32]; 2];
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) struct Counters {
    pub evaluations: usize,
    pub observations: usize,
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
    pairs: PairCounts,
    charge: usize,
    touched: u64,
}
struct Report {
    id: String,
    assignment: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
    pairs: PairCounts,
}
// Fixed80-byte IDs keep <=1000 frame booleans plus the one bounded fallback within256KiB.
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
    pub fn pairs(&self, id: &str, practice: bool, requirement: usize) -> Option<u8> {
        self.prepared(id)?;
        if !self.admitted(id) {
            return None;
        }
        self.entries
            .iter()
            .find(|e| e.source.id == id)
            .map(|e| &e.pairs)
            .or_else(|| {
                self.fallback
                    .as_ref()
                    .filter(|e| e.id == id)
                    .map(|e| &e.pairs)
            })?[usize::from(practice)]
        .get(requirement)
        .copied()
        .flatten()
    }
    pub fn admitted(&self, id: &str) -> bool {
        self.rows(id, false).is_some_and(|r| r.len() >= 4)
    }
    /// Row0 is pending only after exact complete-input context admission. Refuse
    /// a stale/unprepared/invalid report rather than reparsing or trusting a bit.
    pub fn pending(&self, id: &str, practice: bool) -> Option<bool> {
        self.prepared(id)?;
        if !self.admitted(id) {
            return None;
        }
        let rows = self.rows(id, practice)?;
        (rows.len() >= 4).then(|| !rows[0].0)
    }
    fn report(
        &mut self,
        id: &str,
        assignment: Vec<(bool, String)>,
        practice: Vec<(bool, String)>,
        pairs: PairCounts,
    ) {
        self.counters.fallbacks += 1;
        if id.len() <= 80 {
            self.frame
                .put(id, !assignment.is_empty() && assignment.iter().all(|r| r.0));
            self.fallback = Some(Report {
                id: id.into(),
                assignment,
                practice,
                pairs,
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
            && matches!(a.state, engine::State::EtherChannel { .. })
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
                    "LACP source or foreign workspace metadata differs".into(),
                )],
                vec![],
                [[None; 32]; 2],
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
        if let Err(error) = work::validate_context(a, s) {
            self.entries.retain(|e| e.source.id != s.id);
            self.bytes = self.entries.iter().map(|e| e.charge).sum();
            self.report(
                &s.id,
                vec![(false, work::notice(error))],
                vec![],
                [[None; 32]; 2],
            );
            return;
        }
        self.counters.evaluations += 1;
        let assignment = progress::current_etherchannel_checks(a, s);
        let practice = match (
            a.workspace
                .as_ref()
                .and_then(|w| w.etherchannel_learning.as_ref()),
            s.etherchannel_teaching.as_ref(),
        ) {
            (Some(l), Some(source)) => l.checks(source),
            _ => vec![],
        };
        // Pending raw edits invalidate success, not this admitted editing context.
        let valid = assignment.len() >= 4
            && assignment.len() <= 33
            && practice.len() <= 33
            && assignment.iter().chain(&practice).all(|r| r.1.len() <= 512);
        self.entries.retain(|e| e.source.id != s.id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        if !valid {
            self.report(&s.id, assignment, practice, [[None; 32]; 2]);
            return;
        }
        // Observations describe the applied network only. They never grade or
        // authorize saved work and are calculated only on an admitted miss.
        let mut pairs = [[None; 32]; 2];
        if let (
            engine::State::EtherChannel { simulation },
            engine::Goal::EtherChannel { definition },
        ) = (&a.state, &s.goal)
        {
            self.counters.observations += 1;
            pairs[0] = pair_counts(simulation, definition);
        }
        if let Some(learning) = a
            .workspace
            .as_ref()
            .and_then(|w| w.etherchannel_learning.as_ref())
        {
            self.counters.observations += 1;
            pairs[1] = pair_counts(&learning.simulation, &learning.context);
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
                    pairs,
                    charge,
                    touched: self.tick,
                });
                return;
            }
        }
        self.report(&s.id, assignment, practice, pairs);
    }
}
/// Count actual intended reciprocal pairs in each declared current/fault
/// snapshot. No grading condition is inferred from text or a successful bit.
fn pair_counts(simulation: &core::State, definition: &work::Definition) -> [Option<u8>; 32] {
    let mut counts = [None; 32];
    let plan = core::Evaluation {
        queries: vec![],
        faults: definition
            .faults
            .iter()
            .map(|f| f.condition.clone())
            .collect(),
    };
    let Ok(report) = simulation.evaluate(&plan) else {
        return counts;
    };
    for (n, requirement) in definition.requirements.iter().enumerate() {
        let work::Requirement::BundlePair { source, a, b, .. } = requirement else {
            continue;
        };
        let at = match source {
            work::Source::Current {} => Some(0),
            work::Source::Fault { id } => definition
                .faults
                .iter()
                .position(|f| f.id == *id)
                .map(|n| n + 1),
        };
        let Some(snapshot) = at.and_then(|n| report.snapshots.get(n)) else {
            continue;
        };
        let count = snapshot
            .links()
            .iter()
            .filter(|link| {
                link.participating
                    && ((link
                        .a
                        .is_some_and(|g| g.device == a.device && g.group == a.group)
                        && link
                            .b
                            .is_some_and(|g| g.device == b.device && g.group == b.group))
                        || (link
                            .b
                            .is_some_and(|g| g.device == a.device && g.group == a.group)
                            && link
                                .a
                                .is_some_and(|g| g.device == b.device && g.group == b.group)))
            })
            .count();
        if let Some(slot) = counts.get_mut(n) {
            *slot = u8::try_from(count).ok();
        }
    }
    counts
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
                .ok_or_else(|| std::io::Error::other("LACP memo wire bound"))?;
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
        cron_teaching,
        etherchannel_teaching,
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
    let _ = etherchannel_teaching;
    cron_teaching.is_none()
        && physical_link_teaching.is_none()
        && file_backup_teaching.is_none()
        && nfs_teaching.is_none()
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
            (
                engine::State::EtherChannel { .. },
                engine::Goal::EtherChannel { .. }
            )
        )
}
pub(super) fn same_source(a: &Scenario, b: &Scenario) -> bool {
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
        && a.etherchannel_teaching == b.etherchannel_teaching
        && match (&a.goal, &b.goal) {
            (
                engine::Goal::EtherChannel { definition: a },
                engine::Goal::EtherChannel { definition: b },
            ) => a == b,
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
        cron_session,
        cron_learning,
        etherchannel_draft: _,
        etherchannel_learning: _,
        nfs_session,
        nfs_learning,
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
    cron_session.is_some()
        || cron_learning.is_some()
        || physical_link_draft.is_some()
        || physical_link_learning.is_some()
        || file_backup_learning.is_some()
        || nfs_session.is_some()
        || nfs_learning.is_some()
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
        && a.etherchannel_draft == b.etherchannel_draft
        && a.etherchannel_learning == b.etherchannel_learning
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
        f.write_str("LACP memo storage ceiling")
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
    fn source(n: usize) -> Scenario {
        crate::ui::tests::etherchannel_workspace::sources().remove(n)
    }
    fn prepared(s: &Scenario) -> Attempt {
        let mut a = Attempt::new(s, 1);
        learn::prepare(&mut a, s, true).unwrap();
        a
    }
    fn solve(a: &mut Attempt, s: &Scenario) {
        for text in &s.solution {
            let action =
                core::Action::from_json(text.strip_prefix(work::PREFIX).unwrap().as_bytes())
                    .unwrap();
            let core::Action::Configure { configuration } = action;
            let engine::State::EtherChannel { simulation } = &a.state else {
                panic!()
            };
            let d = work::Draft::from_configuration(&configuration, simulation.topology()).unwrap();
            work::set_assignment_draft(a, s, &d).unwrap();
            work::apply_assignment(a, s).unwrap();
        }
    }
    #[test]
    fn exact_complete_keys_hit_without_idle_meters_snapshots_or_evaluations() {
        for n in 0..3 {
            let s = source(n);
            let mut a = prepared(&s);
            solve(&mut a, &s);
            let mut m = Memo::default();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            let before = m.counters;
            for _ in 0..80 {
                m.begin_frame();
                m.evaluate(&a, &s, false);
                assert_eq!(m.prepared(&s.id), Some(true));
            }
            assert_eq!(m.counters.evaluations, before.evaluations);
            assert_eq!(m.counters.observations, before.observations);
            assert_eq!(m.counters.meters, before.meters);
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert_eq!(m.counters.hits, before.hits + 80);
            let old = m.counters;
            a.visited += 1;
            m.begin_frame();
            m.evaluate(&a, &s, false);
            assert_eq!(m.counters.evaluations, old.evaluations + 1);
            assert_eq!(m.counters.snapshots, old.snapshots + 1);
            let mut changed = s.clone();
            changed.hints[0].push_str(" An additional source-owned note.");
            let old = m.counters;
            m.begin_frame();
            m.evaluate(&a, &changed, false);
            assert!(m.counters.evaluations > old.evaluations);
            assert!(m.counters.snapshots > old.snapshots);
            assert!(m.bytes + EDITOR_RESERVE + FRAME_RESERVE <= TOTAL_BYTES);
            assert!(m.entries.iter().all(|e| e.charge <= ENTRY_BYTES));
        }
    }
    #[test]
    fn admitted_pending_raw_fields_invalidate_success_without_locking_out_repair() {
        let s = source(0);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        let mut d = a
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_draft
            .as_ref()
            .unwrap()
            .clone();
        d.devices
            .values_mut()
            .next()
            .unwrap()
            .ports
            .values_mut()
            .next()
            .unwrap()
            .profile
            .allowed = "invalid".into();
        work::set_assignment_draft(&mut a, &s, &d).unwrap();
        m.begin_frame();
        m.evaluate(&a, &s, false);
        assert!(m.admitted(&s.id));
        assert_eq!(m.prepared(&s.id), Some(false));
        assert!(!m.rows(&s.id, false).unwrap()[0].0);
        let before = serde_json::to_value(&a).unwrap();
        assert!(work::apply_assignment(&mut a, &s).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        solve(&mut a, &s);
        m.begin_frame();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
    }
    #[test]
    fn full_source_context_and_foreign_metadata_reject_before_snapshot() {
        let s = source(0);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        let before = m.counters;
        let saved = serde_json::to_value(&a).unwrap();
        let mut foreign = a.clone();
        foreign.workspace.as_mut().unwrap().nfs_session = Some(Default::default());
        m.begin_frame();
        m.evaluate(&foreign, &s, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert!(!m.admitted(&s.id));
        assert_eq!(m.counters.evaluations, before.evaluations);
        assert_eq!(m.counters.observations, before.observations);
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(serde_json::to_value(&a).unwrap(), saved);
        m.begin_frame();
        m.evaluate(&a, &s, false);
        assert_eq!(m.prepared(&s.id), Some(true));
        let before = m.counters;
        let mut changed = s.clone();
        if let engine::Goal::EtherChannel { definition } = &mut changed.goal {
            let mut raw = serde_json::to_value(&definition.initial).unwrap();
            *raw["environment"]["cables"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap() = serde_json::Value::Bool(false);
            definition.initial = serde_json::from_value(raw).unwrap();
        }
        m.begin_frame();
        m.evaluate(&a, &changed, false);
        assert_eq!(m.prepared(&s.id), Some(false));
        assert!(!m.admitted(&s.id));
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
    }
    #[test]
    fn uncacheable_valid_inputs_grade_correctly_without_owned_snapshot_or_budget_growth() {
        let s = source(2);
        let mut a = prepared(&s);
        solve(&mut a, &s);
        let mut m = Memo::default();
        m.evaluate_limits(&a, &s, false, 1, 1, 0);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert!(m.admitted(&s.id));
        assert!(m.entries.is_empty());
        assert_eq!(m.bytes, 0);
        assert_eq!(m.counters.snapshots, 0);
        assert_eq!(m.counters.fallbacks, 1);
        let before = serde_json::to_value(&a).unwrap();
        m.begin_frame();
        m.evaluate_limits(&a, &s, false, 1, 1, 0);
        assert_eq!(m.prepared(&s.id), Some(true));
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
    #[test]
    fn four_entries_are_evictable_with_full_key_history_changes() {
        let base = source(0);
        let mut m = Memo::default();
        for n in 0..7 {
            let mut s = base.clone();
            s.id = format!("lacp-storage-{n}");
            let mut a = prepared(&s);
            solve(&mut a, &s);
            m.begin_frame();
            m.evaluate(&a, &s, false);
            assert_eq!(m.prepared(&s.id), Some(true));
            assert!(m.entries.len() <= 4);
            assert!(m.bytes + EDITOR_RESERVE + FRAME_RESERVE <= TOTAL_BYTES);
            let old = m.counters;
            a.transcript.push("A retained diagnostic entry".into());
            m.begin_frame();
            m.evaluate(&a, &s, false);
            assert_eq!(m.counters.evaluations, old.evaluations + 1);
        }
        assert_eq!(m.entries.len(), 4);
        assert!(m.entries.iter().all(|e| e.source.id != "lacp-storage-0"));
    }
    #[test]
    fn typed_observations_are_fresh_bounded_and_never_become_grading_authority() {
        assert!(std::mem::size_of::<PairCounts>() <= 128);
        assert!(REPORT_RESERVE >= 2 * 33 * (512 + 128) + std::mem::size_of::<PairCounts>());
        for n in 0..3 {
            let source = source(n);
            let mut a = prepared(&source);
            let original = serde_json::to_value(&a).unwrap();
            let mut m = Memo::default();
            m.evaluate(&a, &source, false);
            assert_eq!(m.pairs(&source.id, false, 0), Some([0, 1, 0][n]));
            assert_eq!(
                m.pairs(&source.id, false, 1),
                None,
                "VLAN rows are not fabricated pair counts"
            );
            assert_eq!(serde_json::to_value(&a).unwrap(), original);
            solve(&mut a, &source);
            let expected = progress::current_etherchannel_checks(&a, &source);
            m.evaluate(&a, &source, false);
            assert_eq!(m.rows(&source.id, false).unwrap(), expected);
            assert_eq!(m.pairs(&source.id, false, 0), Some([2, 2, 3][n]));
            if n == 2 {
                for (requirement, count) in [(3, 2), (6, 2), (9, 2), (12, 3)] {
                    assert_eq!(m.pairs(&source.id, false, requirement), Some(count));
                }
            }
            let initial_practice = serde_json::to_value(
                &a.workspace
                    .as_ref()
                    .unwrap()
                    .etherchannel_learning
                    .as_ref()
                    .unwrap()
                    .simulation,
            )
            .unwrap();
            work::fill_practice(&mut a, &source).unwrap();
            m.evaluate(&a, &source, false);
            assert_eq!(
                m.pairs(&source.id, true, 0),
                Some([0, 1, 0][n]),
                "Filled fields alone must retain this context's actual initial pairs"
            );
            assert_eq!(
                serde_json::to_value(
                    &a.workspace
                        .as_ref()
                        .unwrap()
                        .etherchannel_learning
                        .as_ref()
                        .unwrap()
                        .simulation
                )
                .unwrap(),
                initial_practice
            );
            work::apply_practice(&mut a, &source).unwrap();
            m.evaluate(&a, &source, false);
            assert_eq!(m.pairs(&source.id, true, 0), Some([2, 2, 3][n]));
            let before = m.counters;
            for _ in 0..60 {
                m.begin_frame();
                m.evaluate(&a, &source, false);
            }
            assert_eq!(m.counters.observations, before.observations);
            // A valid uncached fallback retains its own fresh counts.
            m.clear();
            m.evaluate_limits(&a, &source, false, 1, 1, 1);
            assert!(m.entries.is_empty());
            assert_eq!(m.pairs(&source.id, false, 0), Some([2, 2, 3][n]));
            assert_eq!(m.pairs(&source.id, true, 0), Some([2, 2, 3][n]));
            let before = m.counters;
            a.workspace.as_mut().unwrap().nfs_session = Some(Default::default());
            m.begin_frame();
            m.evaluate(&a, &source, false);
            assert_eq!(m.pairs(&source.id, false, 0), None);
            assert_eq!(m.pairs(&source.id, true, 0), None);
            assert_eq!(m.counters.observations, before.observations);
        }
    }
    #[test]
    fn valid_file_backup_metadata_and_source_refuse_before_warm_lacp_copy_or_observation() {
        let source = source(0);
        let mut a = prepared(&source);
        solve(&mut a, &source);
        let mut pack: pack::Pack =
            serde_json::from_str(include_str!("../../../scenarios/file-backup-v1.json")).unwrap();
        let backup = pack.scenarios.remove(0);
        let mut b = Attempt::new(&backup, 2);
        dlms_learning_lab::file_backup_teaching::prepare(
            &mut b,
            backup.file_backup_teaching.as_ref().unwrap(),
            true,
        )
        .unwrap();
        let mut m = Memo::default();
        m.evaluate(&a, &source, false);
        assert_eq!(m.prepared(&source.id), Some(true));
        let good = a.clone();
        a.workspace.as_mut().unwrap().file_backup_learning =
            b.workspace.as_ref().unwrap().file_backup_learning.clone();
        assert!(a.workspace.as_ref().unwrap().valid());
        let before = m.counters;
        let saved = serde_json::to_value(&a).unwrap();
        m.begin_frame();
        m.evaluate(&a, &source, false);
        assert_eq!(m.prepared(&source.id), Some(false));
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(m.counters.observations, before.observations);
        assert_eq!(serde_json::to_value(&a).unwrap(), saved);
        a = good;
        m.begin_frame();
        m.evaluate(&a, &source, false);
        assert_eq!(m.prepared(&source.id), Some(true));
        let mut wrong = source.clone();
        wrong.file_backup_teaching = backup.file_backup_teaching.clone();
        let before = m.counters;
        m.begin_frame();
        m.evaluate(&a, &wrong, false);
        assert_eq!(m.prepared(&source.id), Some(false));
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(m.counters.observations, before.observations);
    }

    #[test]
    fn valid_cron_session_example_and_source_refuse_ether_before_warm_copy_or_work() {
        let cron_source = crate::ui::tests::cron_workspace::sources().remove(0);
        let engine::Goal::Cron { definition } = &cron_source.goal else {
            panic!()
        };
        let session = dlms_learning_lab::cron_workspace::Session::new(definition).unwrap();
        let lesson = cron_source.cron_teaching.as_ref().unwrap();
        let learning = dlms_learning_lab::cron_teaching::Learning::new(lesson).unwrap();
        assert!(session.valid() && learning.valid());
        for n in 0..3 {
            let original = source(n);
            let mut good = prepared(&original);
            solve(&mut good, &original);
            for payload in 0..4 {
                let mut s = original.clone();
                let mut a = good.clone();
                let mut m = Memo::default();
                m.evaluate(&a, &s, false);
                assert_eq!(m.prepared(&s.id), Some(true));
                let before = m.counters;
                match payload {
                    0 => a.workspace.as_mut().unwrap().cron_session = Some(session.clone()),
                    1 => a.workspace.as_mut().unwrap().cron_learning = Some(learning.clone()),
                    2 => {
                        let w = a.workspace.as_mut().unwrap();
                        w.cron_session = Some(session.clone());
                        w.cron_learning = Some(learning.clone());
                    }
                    3 => s.cron_teaching = Some(lesson.clone()),
                    _ => unreachable!(),
                }
                assert!(a.workspace.as_ref().unwrap().valid());
                let raw = serde_json::to_vec(&a).unwrap();
                let authored = serde_json::to_vec(&s).unwrap();
                m.begin_frame();
                m.evaluate(&a, &s, false);
                assert_eq!(m.prepared(&s.id), Some(false), "payload{payload}");
                assert_eq!(m.counters.hits, before.hits);
                assert_eq!(m.counters.meters, before.meters);
                assert_eq!(m.counters.snapshots, before.snapshots);
                assert_eq!(m.counters.evaluations, before.evaluations);
                assert!(!progress::current_solved(&a, &s));
                assert_eq!(serde_json::to_vec(&a).unwrap(), raw);
                assert_eq!(serde_json::to_vec(&s).unwrap(), authored);
                m.evaluate(&good, &original, false);
                assert_eq!(m.prepared(&original.id), Some(true));
            }
        }
    }
    #[test]
    fn valid_physical_source_and_each_optional_field_refuse_a_warm_success_before_copy() {
        let source = source(0);
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
            assert_eq!(m.counters.observations, before.observations);
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
