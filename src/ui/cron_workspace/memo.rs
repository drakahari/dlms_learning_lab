//! Complete borrowed-input continuation cache. No saved completion authority,
//! JSON on warm/idle paths, or synchronous fresh-case evaluation on UI frames.
use super::*;
const ENTRIES: usize = 4;
pub(super) const ENTRY_BYTES: usize = 8 * 1024 * 1024;
pub(super) const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const FRAME_RESERVE: usize = 256 * 1024;
const EDITOR_RESERVE: usize = 4 * 1024 * 1024;
pub(super) const RUN_RESERVE: usize = 8 * 1024 * 1024;
// A Check retains its Definition/State while a fresh-case Run additionally owns
// a state copy, parsed entries/environments, one bounded VM, completed output
// and transient validation/case copies. Charge this separately for every entry.
const CHECK_RUN_RESERVE: usize = 3 * 1024 * 1024;
const REPORT_RESERVE: usize = 2 * 34 * (4096 + 128) + 8192;
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) struct Counters {
    pub snapshots: usize,
    pub meters: usize,
    pub hits: usize,
    pub steps: usize,
    pub refusals: usize,
}
#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Assignment,
    Practice,
    Ready,
    Paused,
}
struct Entry {
    source: Scenario,
    attempt: Attempt,
    assignment: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
    phase: Phase,
    check: Option<work::Check>,
    charge: usize,
    touched: u64,
}
impl Entry {
    fn step(&mut self) {
        if matches!(self.phase, Phase::Ready | Phase::Paused) {
            return;
        }
        if self.check.is_none() {
            let context = if self.phase == Phase::Assignment {
                match (&self.source.goal, &self.attempt.state) {
                    (engine::Goal::Cron { definition }, engine::State::Cron { simulation }) => {
                        Some((
                            definition.as_ref(),
                            simulation,
                            self.attempt
                                .workspace
                                .as_ref()
                                .and_then(|w| w.cron_session.as_ref()),
                        ))
                    }
                    _ => None,
                }
            } else {
                self.attempt
                    .workspace
                    .as_ref()
                    .and_then(|w| w.cron_learning.as_ref())
                    .map(|l| (&l.definition, &l.simulation, Some(&l.session)))
            };
            let Some((d, state, session)) = context else {
                self.phase = Phase::Ready;
                return;
            };
            let pending = session.is_none_or(|t| t.applied(d, state));
            let rows = if self.phase == Phase::Assignment {
                &mut self.assignment
            } else {
                &mut self.practice
            };
            rows.push((
                pending,
                "Edited documents match the installed schedule and access policy".into(),
            ));
            match work::Check::new(d, state) {
                Ok(check) => self.check = Some(check),
                Err(e) => {
                    rows.push((false, e));
                    self.phase = Phase::Ready;
                    return;
                }
            }
        }
        let check = self.check.as_mut().unwrap();
        check.tick(1);
        if check.done() {
            let mut rows = std::mem::take(&mut check.rows);
            // Release the old working Definition/State before cloning the next
            // example context. The charge reserves only one active Check.
            self.check = None;
            for row in &mut rows {
                if row.1.len() > 4096 {
                    let mut cut = 4000;
                    while !row.1.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    row.1.truncate(cut);
                    row.1.push_str(" … (details truncated)");
                }
            }
            if self.phase == Phase::Assignment {
                self.assignment.extend(rows);
                self.phase = Phase::Practice;
            } else {
                self.practice.extend(rows);
                self.phase = Phase::Ready;
            }
        }
    }
}
struct FrameResult {
    id: [u8; 80],
    len: u8,
    passed: bool,
}
struct Frame {
    rows: Vec<FrameResult>,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            rows: Vec::with_capacity(1000),
        }
    }
}
impl Frame {
    fn put(&mut self, id: &str, passed: bool) {
        if id.len() > 80 {
            return;
        }
        if let Some(v) = self
            .rows
            .iter_mut()
            .find(|r| &r.id[..usize::from(r.len)] == id.as_bytes())
        {
            v.passed = passed;
            return;
        }
        if self.rows.len() < 1000 {
            let mut bytes = [0; 80];
            bytes[..id.len()].copy_from_slice(id.as_bytes());
            self.rows.push(FrameResult {
                id: bytes,
                len: id.len() as u8,
                passed,
            });
        }
    }
    fn get(&self, id: &str) -> Option<bool> {
        self.rows
            .iter()
            .find(|r| &r.id[..usize::from(r.len)] == id.as_bytes())
            .map(|r| r.passed)
    }
}
#[derive(Default)]
pub(in crate::ui) struct Memo {
    entries: Vec<Entry>,
    bytes: usize,
    tick: u64,
    cursor: usize,
    frame: Frame,
    pub counters: Counters,
}
impl Memo {
    pub fn begin_frame(&mut self) {
        self.frame.rows.clear()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
        self.cursor = 0;
        self.frame.rows.clear()
    }
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.entries.retain(|e| keep(&e.source.id));
        self.bytes = self.entries.iter().map(|e| e.charge).sum()
    }
    pub fn invalidate(&mut self, id: &str) {
        self.entries.retain(|e| e.source.id != id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        self.frame.put(id, false)
    }
    pub fn prepared(&self, id: &str) -> Option<bool> {
        self.frame.get(id)
    }
    pub fn pending(&self, a: &Attempt, s: &Scenario) -> bool {
        self.entries.iter().any(|e| {
            same_source(&e.source, s)
                && same_attempt(&e.attempt, a)
                && matches!(e.phase, Phase::Assignment | Phase::Practice)
        })
    }
    pub fn pause(&mut self, a: &Attempt, s: &Scenario) {
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| same_source(&e.source, s) && same_attempt(&e.attempt, a))
        {
            e.check = None;
            e.phase = Phase::Paused;
            self.frame.put(&s.id, false);
        }
    }
    pub fn paused(&self, a: &Attempt, s: &Scenario) -> bool {
        self.entries.iter().any(|e| {
            same_source(&e.source, s) && same_attempt(&e.attempt, a) && e.phase == Phase::Paused
        })
    }
    pub fn admitted(&self, a: &Attempt, s: &Scenario) -> bool {
        cheap(a, s)
            && self
                .entries
                .iter()
                .any(|e| same_source(&e.source, s) && same_attempt(&e.attempt, a))
    }
    pub fn rows<'a>(
        &'a self,
        a: &Attempt,
        s: &Scenario,
        practice: bool,
    ) -> Option<&'a [(bool, String)]> {
        if !cheap(a, s) {
            return None;
        }
        self.entries
            .iter()
            .find(|e| {
                same_source(&e.source, s) && same_attempt(&e.attempt, a) && e.phase == Phase::Ready
            })
            .map(|e| {
                if practice {
                    e.practice.as_slice()
                } else {
                    e.assignment.as_slice()
                }
            })
    }
    pub fn evaluate(&mut self, a: &Attempt, s: &Scenario, force: bool) {
        if !cheap(a, s) {
            self.invalidate(&s.id);
            self.counters.refusals += 1;
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
                e.phase == Phase::Ready
                    && !e.assignment.is_empty()
                    && e.assignment.iter().all(|r| r.0),
            );
            return;
        }
        self.invalidate(&s.id);
        if work::context(a, s).is_err() {
            self.counters.refusals += 1;
            return;
        }
        self.counters.meters += 1;
        let Some(charge) = charge(a, s) else {
            self.counters.refusals += 1;
            return;
        };
        let total = TOTAL_BYTES - FRAME_RESERVE - EDITOR_RESERVE - RUN_RESERVE;
        while !self.entries.is_empty()
            && (self.entries.len() >= ENTRIES
                || self.bytes.checked_add(charge).is_none_or(|n| n > total))
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
        if self.bytes.checked_add(charge).is_none_or(|n| n > total) {
            self.counters.refusals += 1;
            return;
        }
        self.counters.snapshots += 1;
        self.entries.push(Entry {
            source: s.clone(),
            attempt: a.clone(),
            assignment: Vec::new(),
            practice: Vec::new(),
            phase: Phase::Assignment,
            check: None,
            charge,
            touched: self.tick,
        });
        self.bytes += charge;
    }
    /// One global frame budget across all entries, calendar work and shell VM.
    pub fn advance(&mut self, budget: usize) -> bool {
        for _ in 0..budget.min(core::MAX_TICK) {
            if self.entries.is_empty() {
                break;
            }
            let n = self.entries.len();
            let mut next = None;
            for offset in 0..n {
                let i = (self.cursor + offset) % n;
                if matches!(self.entries[i].phase, Phase::Assignment | Phase::Practice) {
                    next = Some(i);
                    break;
                }
            }
            let Some(i) = next else { break };
            self.cursor = (i + 1) % n;
            self.entries[i].step();
            self.counters.steps += 1;
        }
        // Publish only current-frame results; evaluate() checks exact input again
        // before a library badge or workspace can consult the next frame.
        for e in &self.entries {
            self.frame.put(
                &e.source.id,
                e.phase == Phase::Ready
                    && !e.assignment.is_empty()
                    && e.assignment.iter().all(|r| r.0),
            );
        }
        self.entries
            .iter()
            .any(|e| matches!(e.phase, Phase::Assignment | Phase::Practice))
    }
}
fn cheap(a: &Attempt, s: &Scenario) -> bool {
    work::source_shape(s)
        && s.id.len() <= 80
        && a.scenario_version == s.version
        && matches!(a.state, engine::State::Cron { .. })
        && a.ipv4.is_none()
        && a.workspace
            .as_ref()
            .is_none_or(|w| w.version == 1 && !work::foreign(w))
}
fn charge(a: &Attempt, s: &Scenario) -> Option<usize> {
    // Reuse the existing pure structural counter, not any NFS model or grader.
    let heap = crate::ui::nfs_workspace::memo::charged(&(a, s), ENTRY_BYTES)?;
    let wire = work::size(&(a, s), ENTRY_BYTES)?;
    let engine::Goal::Cron { definition } = &s.goal else {
        return None;
    };
    let engine::State::Cron { simulation } = &a.state else {
        return None;
    };
    let mut working =
        crate::ui::nfs_workspace::memo::charged(&(definition, simulation), ENTRY_BYTES)?
            .max(work::size(&(definition, simulation), ENTRY_BYTES)?);
    if let Some(l) = a.workspace.as_ref().and_then(|w| w.cron_learning.as_ref()) {
        working = working.max(
            crate::ui::nfs_workspace::memo::charged(&(&l.definition, &l.simulation), ENTRY_BYTES)?
                .max(work::size(&(&l.definition, &l.simulation), ENTRY_BYTES)?),
        );
    }
    heap.max(wire)
        .checked_add(working)?
        .checked_add(CHECK_RUN_RESERVE)?
        .checked_add(REPORT_RESERVE)
        .filter(|n| *n <= ENTRY_BYTES)
}
fn same_source(a: &Scenario, b: &Scenario) -> bool {
    work::source_shape(a)
        && work::source_shape(b)
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
        && a.cron_teaching == b.cron_teaching
        && match (&a.goal, &b.goal) {
            (engine::Goal::Cron { definition: a }, engine::Goal::Cron { definition: b }) => a == b,
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
    !work::foreign(a)
        && !work::foreign(b)
        && a.version == b.version
        && a.cron_session == b.cron_session
        && a.cron_learning == b.cron_learning
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

pub(super) fn same_input(a: &Attempt, s: &Scenario, saved: &Attempt, source: &Scenario) -> bool {
    cheap(a, s) && same_source(s, source) && same_attempt(a, saved)
}
pub(super) fn run_admitted(a: &Attempt, s: &Scenario) -> bool {
    if !cheap(a, s) || work::context(a, s).is_err() {
        return false;
    }
    let Some(heap) = crate::ui::nfs_workspace::memo::charged(&(a, s), RUN_RESERVE) else {
        return false;
    };
    let Some(wire) = work::size(&(a, s), RUN_RESERVE) else {
        return false;
    };
    // Core clock/table/virtual-file clone, one bounded VM, pending output/history
    // and parser/transient buffers are reserved in addition to complete inputs.
    heap.max(wire)
        .checked_add(2 * 1024 * 1024)
        .is_some_and(|n| n <= RUN_RESERVE)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sources() -> Vec<Scenario> {
        crate::ui::tests::cron_workspace::sources()
    }
    fn solved(s: &Scenario) -> Attempt {
        let mut a = Attempt::new(s, 17);
        for input in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, input).unwrap();
        }
        work::budget::prepare(&mut a, s).unwrap();
        assert!(progress::current_solved(&a, s));
        a
    }
    fn finish(m: &mut Memo, a: &Attempt, s: &Scenario) {
        m.evaluate(a, s, false);
        for _ in 0..10000 {
            if !m.advance(64) {
                break;
            }
        }
        assert!(!m.pending(a, s));
        assert!(m.rows(a, s, false).is_some());
    }
    #[test]
    fn complete_input_warm_hit_does_no_meter_snapshot_or_execution() {
        for s in sources() {
            let a = solved(&s);
            let mut m = Memo::default();
            finish(&mut m, &a, &s);
            assert_eq!(m.prepared(&s.id), Some(true));
            let before = m.counters;
            for _ in 0..20 {
                m.begin_frame();
                m.evaluate(&a, &s, false);
                assert!(!m.advance(64));
                assert_eq!(m.prepared(&s.id), Some(true));
            }
            assert_eq!(m.counters.snapshots, before.snapshots);
            assert_eq!(m.counters.meters, before.meters);
            assert_eq!(m.counters.steps, before.steps);
            assert_eq!(m.counters.hits, before.hits + 20);
        }
    }
    #[test]
    fn run_and_fresh_checks_yield_and_pause_without_touching_saved_state() {
        let s = sources().remove(0);
        let a = solved(&s);
        let before = serde_json::to_vec(&a).unwrap();
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert!(m.advance(1));
        assert!(m.pending(&a, &s));
        assert_eq!(m.counters.steps, 1);
        m.pause(&a, &s);
        assert!(m.paused(&a, &s));
        assert!(!m.advance(usize::MAX));
        assert_eq!(serde_json::to_vec(&a).unwrap(), before);
        m.evaluate(&a, &s, true);
        finish(&mut m, &a, &s);
        assert_eq!(m.prepared(&s.id), Some(true));
    }
    #[test]
    fn each_source_raw_attempt_and_foreign_metadata_change_refuses_old_warm_authority() {
        let s = sources().remove(0);
        let a = solved(&s);
        for field in [
            "title",
            "solution",
            "source",
            "probe",
            "draft",
            "clock",
            "step",
            "hints",
            "transcript",
            "legacy-raw",
            "foreign",
        ] {
            let mut m = Memo::default();
            finish(&mut m, &a, &s);
            let before = m.counters;
            let mut source = s.clone();
            let mut attempt = a.clone();
            match field {
                "title" => source.title.push_str(" revised"),
                "solution" => source.solution[0].push(' '),
                "source" => source.sources[0].push_str("#review"),
                "probe" => {
                    let engine::Goal::Cron { definition } = &mut source.goal else {
                        unreachable!()
                    };
                    definition.probes[0].name.push_str(" changed");
                }
                "draft" => attempt
                    .workspace
                    .as_mut()
                    .unwrap()
                    .cron_session
                    .as_mut()
                    .unwrap()
                    .drafts
                    .get_mut("user:alex")
                    .unwrap()
                    .push_str("# pending\n"),
                "clock" => {
                    let engine::State::Cron { simulation } = &mut attempt.state else {
                        unreachable!()
                    };
                    simulation.clock = simulation.clock.next().unwrap();
                }
                "step" => attempt.step += 1,
                "hints" => attempt.hints += 1,
                "transcript" => attempt.transcript.push("retained output".into()),
                "legacy-raw" => {
                    attempt.workspace.as_mut().unwrap().command = "read-only raw command".into()
                }
                "foreign" => {
                    attempt.workspace.as_mut().unwrap().nfs_session = Some(Default::default())
                }
                _ => unreachable!(),
            }
            let state = serde_json::to_vec(&attempt).unwrap();
            m.begin_frame();
            m.evaluate(&attempt, &source, false);
            assert_eq!(m.prepared(&source.id), Some(false));
            assert_eq!(m.counters.hits, before.hits);
            assert_eq!(serde_json::to_vec(&attempt).unwrap(), state);
            if field == "foreign" {
                assert_eq!(m.counters.snapshots, before.snapshots);
                assert_eq!(m.counters.meters, before.meters);
            }
        }
    }
    #[test]
    fn structural_wire_working_and_frame_reserves_stay_within_owned_storage_budget() {
        let mut m = Memo::default();
        for s in sources() {
            let a = solved(&s);
            m.evaluate(&a, &s, false);
            assert!(m.entries.len() <= ENTRIES);
            assert!(m.bytes <= TOTAL_BYTES - EDITOR_RESERVE - FRAME_RESERVE - RUN_RESERVE);
            assert!(m.entries.iter().all(|e| e.charge <= ENTRY_BYTES));
        }
        assert!(
            1000 * std::mem::size_of::<FrameResult>() + std::mem::size_of::<Frame>()
                <= FRAME_RESERVE
        );
        //18 active IDs,16 text undo points plus flux/current/redo movement,
        // and one pre-edit undo snapshot are bounded by the explicit reserve.
        const {
            assert!(
                (18 * (super::super::EDITOR_UNDOS + 3) + super::super::EDITOR_UNDOS + 3)
                    * core::MAX_TABLE_BYTES
                    + 64 * 1024
                    <= EDITOR_RESERVE
            );
        }
        assert_eq!(m.entries.len(), ENTRIES);
        let first_id = m.entries[0].source.id.clone();
        let mut fifth = sources().remove(0);
        fifth.id = "cron-cache-only-eviction-fixture".into();
        let a = solved(&fifth);
        m.evaluate(&a, &fifth, false);
        assert_eq!(m.entries.len(), ENTRIES);
        assert!(!m.entries.iter().any(|e| e.source.id == first_id));
        assert!(m.entries.iter().any(|e| e.source.id == fifth.id));
        let before = m.counters;
        m.retain(|_| false);
        assert_eq!(m.bytes, 0);
        assert!(m.entries.is_empty());
        assert_eq!(m.counters, before);
    }

    #[test]
    fn admitted_ether_and_backup_fields_or_teaching_refuse_cron_before_warm_work() {
        let ether_raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/etherchannel-workbench-v1.json"
        ))
        .unwrap();
        let ether_lesson: dlms_learning_lab::etherchannel_teaching::Lesson =
            serde_json::from_value(ether_raw["cases"][0]["lesson"].clone()).unwrap();
        ether_lesson.validate().unwrap();
        let ether_draft = dlms_learning_lab::etherchannel_workspace::Draft::from_state(
            &ether_lesson.definition.initial,
        )
        .unwrap();
        let ether_learning =
            dlms_learning_lab::etherchannel_teaching::Learning::new(&ether_lesson, true).unwrap();
        let mut backup_pack: pack::Pack =
            serde_json::from_str(include_str!("../../../scenarios/file-backup-v1.json")).unwrap();
        let backup_lesson = backup_pack
            .scenarios
            .remove(0)
            .file_backup_teaching
            .unwrap();
        backup_lesson.validate().unwrap();
        let backup_learning =
            dlms_learning_lab::file_backup_teaching::Learning::new(&backup_lesson).unwrap();
        assert!(ether_learning.valid() && backup_learning.valid());
        for original in sources() {
            let good = solved(&original);
            for payload in 0..7 {
                let mut s = original.clone();
                let mut a = good.clone();
                let mut m = Memo::default();
                finish(&mut m, &a, &s);
                assert_eq!(m.prepared(&s.id), Some(true));
                m.begin_frame();
                m.evaluate(&a, &s, false);
                let before = m.counters;
                match payload {
                    0 => {
                        a.workspace.as_mut().unwrap().etherchannel_draft = Some(ether_draft.clone())
                    }
                    1 => {
                        a.workspace.as_mut().unwrap().etherchannel_learning =
                            Some(ether_learning.clone())
                    }
                    2 => {
                        let w = a.workspace.as_mut().unwrap();
                        w.etherchannel_draft = Some(ether_draft.clone());
                        w.etherchannel_learning = Some(ether_learning.clone());
                    }
                    3 => {
                        a.workspace.as_mut().unwrap().file_backup_learning =
                            Some(backup_learning.clone())
                    }
                    4 => s.etherchannel_teaching = Some(ether_lesson.clone()),
                    5 => s.file_backup_teaching = Some(backup_lesson.clone()),
                    6 => {
                        s.etherchannel_teaching = Some(ether_lesson.clone());
                        s.file_backup_teaching = Some(backup_lesson.clone());
                    }
                    _ => unreachable!(),
                }
                assert!(
                    a.workspace.as_ref().unwrap().valid(),
                    "Foreign metadata must be independently valid"
                );
                let raw = serde_json::to_vec(&a).unwrap();
                let source = serde_json::to_vec(&s).unwrap();
                m.begin_frame();
                m.evaluate(&a, &s, false);
                assert_eq!(m.prepared(&s.id), Some(false), "payload{payload}");
                assert_eq!(m.counters.hits, before.hits);
                assert_eq!(m.counters.meters, before.meters);
                assert_eq!(m.counters.snapshots, before.snapshots);
                assert_eq!(m.counters.steps, before.steps);
                assert!(!progress::current_solved(&a, &s));
                assert_eq!(serde_json::to_vec(&a).unwrap(), raw);
                assert_eq!(serde_json::to_vec(&s).unwrap(), source);
                assert!(!m.advance(64));
                finish(&mut m, &good, &original);
                assert_eq!(
                    m.prepared(&original.id),
                    Some(true),
                    "Restore real positive authority after rejecting foreign input"
                );
            }
        }
    }
}
