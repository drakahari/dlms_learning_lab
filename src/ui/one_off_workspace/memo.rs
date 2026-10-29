//! Complete borrowed-input continuation cache. No saved completion authority,
//! JSON on warm/idle paths, or synchronous fresh-case evaluation on UI frames.
use super::*;
use crate::ui::completion_index::{Family, Handle, Key};
use dlms_learning_lab::cron_workspace as bounded;
use std::sync::Arc;
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
const REPORT_RESERVE: usize = 2 * 35 * (4096 + 128) + 8192;
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
    source: Arc<Scenario>,
    attempt: Arc<Attempt>,
    key: Arc<Key>,
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
                    (engine::Goal::OneOff { definition }, engine::State::OneOff { simulation }) => {
                        Some((
                            definition.as_ref(),
                            simulation.as_ref(),
                            self.attempt
                                .workspace
                                .as_ref()
                                .and_then(|w| w.one_off_session.as_ref()),
                        ))
                    }
                    _ => None,
                }
            } else {
                self.attempt
                    .workspace
                    .as_ref()
                    .and_then(|w| w.one_off_learning.as_ref())
                    .map(|l| (&l.definition, &l.assignment, Some(&l.session)))
            };
            let Some((d, state, session)) = context else {
                self.phase = Phase::Ready;
                return;
            };
            let pending = session.is_some_and(|t| t.applied(d, state));
            let rows = if self.phase == Phase::Assignment {
                &mut self.assignment
            } else {
                &mut self.practice
            };
            rows.push((
                pending,
                "Saved virtual files match the current editor drafts".into(),
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
#[derive(Default)]
pub(in crate::ui) struct Memo {
    entries: Vec<Entry>,
    bytes: usize,
    tick: u64,
    cursor: usize,
    // Bounded runtime Stop intent, never completion authority or saved mastery.
    // The overflow fallback only stops additional automatic checks; explicit
    // Check can still run. 64 IDs (<=80 bytes each) fit the frame reserve.
    stopped: std::collections::BTreeSet<String>,
    stop_background: bool,
    pub certificates: Handle,
    pub counters: Counters,
}
impl Memo {
    pub fn clear(&mut self) {
        self.pause_away_from(None);
        self.entries.clear();
        self.bytes = 0;
        self.cursor = 0;
    }
    pub fn reconcile(&mut self, p: &progress::Progress, sources: &[Scenario]) {
        self.stopped
            .retain(|id| p.attempts.contains_key(id) && sources.iter().any(|s| &s.id == id));
        self.entries.retain(|e| {
            p.attempts.get(&e.source.id).is_some_and(|a| {
                sources
                    .iter()
                    .find(|s| s.id == e.source.id)
                    .is_some_and(|s| same_input(a, s, &e.attempt, &e.source))
            })
        });
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
    }
    #[cfg(test)]
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.entries.retain(|e| keep(&e.source.id));
        self.bytes = self.entries.iter().map(|e| e.charge).sum()
    }
    pub fn invalidate(&mut self, id: &str) {
        // Metadata and optional-help changes invalidate an exact result without
        // revoking a learner's explicit Stop intent.
        self.entries.retain(|e| e.source.id != id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        self.certificates.forget(id);
    }
    pub fn changed(&mut self, id: &str) {
        self.stopped.remove(id);
        self.invalidate(id);
    }
    pub fn passed(&self, a: &Attempt, s: &Scenario) -> Option<bool> {
        self.certificates
            .status(a, s)
            .and_then(|status| status.result())
    }
    pub fn publish_current(&self, a: &Attempt, s: &Scenario) -> Result<(), String> {
        if self.certificates.status(a, s) != Some(completion_index::Status::Pending) {
            return Ok(());
        }

        let Some(e) = self.entries.iter().find(|e| {
            same_source(&e.source, s) && same_attempt(&e.attempt, a) && e.phase == Phase::Ready
        }) else {
            return Ok(());
        };
        self.certificates.publish(
            &e.key,
            a,
            s,
            !e.assignment.is_empty() && e.assignment.iter().all(|r| r.0),
        )
    }
    pub fn pending(&self, a: &Attempt, s: &Scenario) -> bool {
        self.entries.iter().any(|e| {
            same_source(&e.source, s)
                && same_attempt(&e.attempt, a)
                && matches!(e.phase, Phase::Assignment | Phase::Practice)
        })
    }
    fn remember_stop(&mut self, id: &str) {
        if id.len() <= 80 && self.stopped.len() < 64 {
            self.stopped.insert(id.to_owned());
        } else if !self.stopped.contains(id) {
            self.stop_background = true;
        }
    }
    pub fn pause(&mut self, a: &Attempt, s: &Scenario) {
        if cheap(a, s) {
            self.remember_stop(&s.id);
        }
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| same_input(a, s, &e.attempt, &e.source))
        {
            e.check = None;
            e.phase = Phase::Paused;
        }
        self.certificates.forget(&s.id);
        self.certificates.refuse(&s.id);
    }
    /// Cancel unfinished Check work outside its actual selected workspace.
    /// No queue/file state, receipt, draft or completion record is published.
    pub fn pause_away_from(&mut self, selected: Option<&str>) {
        for n in 0..self.entries.len() {
            if selected != Some(self.entries[n].source.id.as_str())
                && matches!(self.entries[n].phase, Phase::Assignment | Phase::Practice)
            {
                // Clone only the already-admitted <=80-byte ID, not source or Attempt.
                let id = self.entries[n].source.id.clone();
                self.remember_stop(&id);
                self.entries[n].check = None;
                self.entries[n].phase = Phase::Paused;
                self.certificates.forget(&id);
                self.certificates.refuse(&id);
            }
        }
    }
    pub fn paused(&self, a: &Attempt, s: &Scenario) -> bool {
        cheap(a, s) && (self.stop_background || self.stopped.contains(&s.id)) && !self.pending(a, s)
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
        if force {
            self.stopped.remove(&s.id);
        }
        let paused = !force && (self.stop_background || self.stopped.contains(&s.id));
        if !force
            && let Some(e) = self
                .entries
                .iter_mut()
                .find(|e| same_source(&e.source, s) && same_attempt(&e.attempt, a))
        {
            e.touched = self.tick;
            self.counters.hits += 1;
            if e.phase == Phase::Ready
                && self.certificates.status(a, s) == Some(completion_index::Status::Pending)
            {
                let _ = self.certificates.publish(
                    &e.key,
                    a,
                    s,
                    !e.assignment.is_empty() && e.assignment.iter().all(|r| r.0),
                );
            }
            return;
        }
        self.entries.retain(|e| e.source.id != s.id);
        self.bytes = self.entries.iter().map(|e| e.charge).sum();
        if work::context(a, s).is_err() || !input::shape(a, s) {
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
        let Ok((key, copied)) = self.certificates.admit(a, s, Family::OneOff) else {
            self.counters.refusals += 1;
            return;
        };
        self.counters.snapshots += usize::from(copied);
        self.entries.push(Entry {
            source: Arc::clone(&key.source),
            attempt: Arc::clone(&key.attempt),
            key,
            assignment: Vec::new(),
            practice: Vec::new(),
            phase: if paused {
                Phase::Paused
            } else {
                Phase::Assignment
            },
            check: None,
            charge,
            touched: self.tick,
        });
        self.bytes += charge;
        if paused {
            self.certificates.refuse(&s.id);
        }
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
        self.entries
            .iter()
            .any(|e| matches!(e.phase, Phase::Assignment | Phase::Practice))
    }
}
pub(in crate::ui) fn cheap(a: &Attempt, s: &Scenario) -> bool {
    work::source_shape(s)
        && s.id.len() <= 80
        && a.scenario_version == s.version
        && a.step < s.steps.len()
        && a.hints <= s.hints.len()
        && a.transcript.len() <= 120
        && matches!(a.state, engine::State::OneOff { .. })
        && a.ipv4.is_none()
        && a.workspace
            .as_ref()
            .is_none_or(|w| w.version == 1 && !work::foreign(w))
}
fn charge(a: &Attempt, s: &Scenario) -> Option<usize> {
    // Reuse the existing pure structural counter, not any NFS model or grader.
    let heap = crate::ui::nfs_workspace::memo::charged(&(a, s), ENTRY_BYTES)?;
    let wire = bounded::size(&(a, s), ENTRY_BYTES)?;
    let engine::Goal::OneOff { definition } = &s.goal else {
        return None;
    };
    let engine::State::OneOff { simulation } = &a.state else {
        return None;
    };
    let mut working =
        crate::ui::nfs_workspace::memo::charged(&(definition, simulation), ENTRY_BYTES)?
            .max(bounded::size(&(definition, simulation), ENTRY_BYTES)?);
    if let Some(l) = a
        .workspace
        .as_ref()
        .and_then(|w| w.one_off_learning.as_ref())
    {
        working = working.max(
            crate::ui::nfs_workspace::memo::charged(&(&l.definition, &l.assignment), ENTRY_BYTES)?
                .max(bounded::size(&(&l.definition, &l.assignment), ENTRY_BYTES)?),
        );
    }
    heap.max(wire)
        .checked_add(working)?
        .checked_add(CHECK_RUN_RESERVE)?
        .checked_add(REPORT_RESERVE)
        .filter(|n| *n <= ENTRY_BYTES)
}
pub(in crate::ui) fn same_source(a: &Scenario, b: &Scenario) -> bool {
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
        && a.one_off_teaching == b.one_off_teaching
        && match (&a.goal, &b.goal) {
            (engine::Goal::OneOff { definition: a }, engine::Goal::OneOff { definition: b }) => {
                a == b
            }
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
    same_attempt_fields(a, b, false)
}
pub(in crate::ui) fn same_attempt_fields(a: &Attempt, b: &Attempt, metadata: bool) -> bool {
    let Attempt {
        scenario_version: _,
        state: _,
        step: _,
        challenge: _,
        hints: _,
        transcript: _,
        visited: _,
        ipv4: _,
        workspace: _,
    } = a;
    a.scenario_version == b.scenario_version
        && a.state == b.state
        && a.step == b.step
        && (metadata || a.challenge == b.challenge)
        && a.hints == b.hints
        && a.transcript == b.transcript
        && (metadata || a.visited == b.visited)
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
        && a.one_off_session == b.one_off_session
        && a.one_off_learning == b.one_off_learning
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

pub(in crate::ui) fn index_context(a: &Attempt, s: &Scenario) -> bool {
    input::shape(a, s)
}
pub(super) fn same_input(a: &Attempt, s: &Scenario, saved: &Attempt, source: &Scenario) -> bool {
    cheap(a, s) && same_source(s, source) && same_attempt(a, saved)
}
pub(super) fn run_admitted(a: &Attempt, s: &Scenario) -> bool {
    if !cheap(a, s) || work::context(a, s).is_err() || !input::shape(a, s) {
        return false;
    }
    let Some(heap) = crate::ui::nfs_workspace::memo::charged(&(a, s), RUN_RESERVE) else {
        return false;
    };
    let Some(wire) = bounded::size(&(a, s), RUN_RESERVE) else {
        return false;
    };
    // Core clock/table/virtual-file clone, one bounded VM, pending output/history
    // and parser/transient buffers are reserved in addition to complete inputs.
    heap.max(wire)
        .checked_mul(2)
        .and_then(|n| n.checked_add(2 * 1024 * 1024))
        .is_some_and(|n| n <= RUN_RESERVE)
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn source() -> Scenario {
        crate::ui::tests::one_off_workspace::fixture()
    }
    pub(super) fn solved(s: &Scenario) -> Attempt {
        let mut a = Attempt::new(s, 7);
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        work::budget::prepare(&mut a, s).unwrap();
        assert!(progress::current_solved(&a, s));
        a
    }
    pub(super) fn finish(m: &mut Memo, a: &Attempt, s: &Scenario) {
        m.evaluate(a, s, false);
        for _ in 0..10000 {
            if !m.advance(core::MAX_TICK) {
                break;
            }
        }
        assert!(!m.pending(a, s));
        assert!(m.rows(a, s, false).is_some());
        m.publish_current(a, s).unwrap();
    }
    #[test]
    fn exact_complete_input_warm_idle_does_not_replay_meter_or_copy() {
        let s = source();
        let a = solved(&s);
        let mut m = Memo::default();
        finish(&mut m, &a, &s);
        assert_eq!(m.passed(&a, &s), Some(true));
        let before = m.counters;
        for _ in 0..20 {
            m.evaluate(&a, &s, false);
            assert!(!m.advance(core::MAX_TICK));
            assert_eq!(m.passed(&a, &s), Some(true));
        }
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.steps, before.steps);
        assert_eq!(m.counters.hits, before.hits + 20);
    }
    #[test]
    fn complete_source_and_attempt_mutations_cannot_reuse_cached_success() {
        let s = source();
        let good = solved(&s);
        for kind in [
            "title",
            "source",
            "solution",
            "goal",
            "draft",
            "command",
            "clock",
            "transcript",
            "step",
            "foreign",
        ] {
            let mut a = good.clone();
            let mut source = s.clone();
            let mut m = Memo::default();
            finish(&mut m, &a, &source);
            let before = m.counters;
            match kind {
                "title" => source.title.push_str(" revised"),
                "source" => source.sources[0].push_str("#review"),
                "solution" => source.solution[0].push(' '),
                "goal" => {
                    let engine::Goal::OneOff { definition } = &mut source.goal else {
                        unreachable!()
                    };
                    definition.probes[0].name.push_str(" revised");
                }
                "draft" => a
                    .workspace
                    .as_mut()
                    .unwrap()
                    .one_off_session
                    .as_mut()
                    .unwrap()
                    .drafts
                    .get_mut("/work/job.sh")
                    .unwrap()
                    .push_str("# changed\n"),
                "command" => a
                    .workspace
                    .as_mut()
                    .unwrap()
                    .one_off_session
                    .as_mut()
                    .unwrap()
                    .command
                    .push_str(" inspected"),
                "clock" => {
                    let engine::State::OneOff { simulation } = &mut a.state else {
                        unreachable!()
                    };
                    simulation.simulation.clock = simulation.simulation.clock.next().unwrap();
                }
                "transcript" => a.transcript.push("retained raw output".into()),
                "step" => a.step = source.steps.len(),
                "foreign" => a.workspace.as_mut().unwrap().nfs_session = Some(Default::default()),
                _ => unreachable!(),
            }
            let bytes = serde_json::to_vec(&a).unwrap();
            let src = serde_json::to_vec(&source).unwrap();
            m.evaluate(&a, &source, false);
            assert_eq!(m.counters.hits, before.hits, "{kind}");
            assert_ne!(m.passed(&a, &source), Some(true), "{kind}");
            assert_eq!(serde_json::to_vec(&a).unwrap(), bytes);
            assert_eq!(serde_json::to_vec(&source).unwrap(), src);
            if kind == "foreign" || kind == "step" {
                assert_eq!(m.counters.snapshots, before.snapshots);
                assert_eq!(m.counters.meters, before.meters);
            }
        }
    }
    #[test]
    fn fresh_replay_checks_yield_stop_and_never_publish_probe_outputs() {
        let s = source();
        let a = solved(&s);
        let before = serde_json::to_vec(&a).unwrap();
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert!(m.advance(1));
        assert!(m.pending(&a, &s));
        m.pause(&a, &s);
        assert!(!m.advance(usize::MAX));
        assert!(m.paused(&a, &s));
        assert_eq!(serde_json::to_vec(&a).unwrap(), before);
        assert_ne!(m.passed(&a, &s), Some(true));
        m.evaluate(&a, &s, true);
        finish(&mut m, &a, &s);
        assert_eq!(m.passed(&a, &s), Some(true));
    }
    #[test]
    fn oversized_complete_inputs_refuse_without_grade_snapshot_or_unbounded_fallback() {
        let s = source();
        let mut a = solved(&s);
        a.transcript.push("x".repeat(work::MAX_ATTEMPT));
        let before = serde_json::to_vec(&a).unwrap();
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        assert_eq!(m.counters.snapshots, 0);
        assert_eq!(m.counters.meters, 0);
        assert!(!m.advance(core::MAX_TICK));
        assert_ne!(m.passed(&a, &s), Some(true));
        assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    }
    #[test]
    fn bounded_cache_and_native_editor_run_frame_reserves_cover_retained_storage() {
        const {
            assert!(
                (core::MAX_FILES_BYTES + 2048 + 32 + 16 * 1024) * (super::super::EDITOR_UNDOS + 3)
                    + core::MAX_FILES_BYTES * (super::super::EDITOR_UNDOS + 3)
                    + 64 * 1024
                    <= EDITOR_RESERVE
            );
        }
        const {
            assert!(FRAME_RESERVE >= 64 * 1024);
        }
        let base = source();
        let mut m = Memo::default();
        for n in 0..7 {
            let mut s = base.clone();
            s.id = format!("synthetic-one-off-cache-{n}");
            let a = solved(&s);
            m.evaluate(&a, &s, false);
            assert!(m.entries.len() <= ENTRIES);
            assert!(m.bytes <= TOTAL_BYTES - EDITOR_RESERVE - FRAME_RESERVE - RUN_RESERVE);
            assert!(m.entries.iter().all(|e| e.charge <= ENTRY_BYTES));
        }
        let before = m.counters;
        m.retain(|_| false);
        assert!(m.entries.is_empty());
        assert_eq!(m.bytes, 0);
        assert_eq!(m.counters, before);
    }
}
#[cfg(test)]
mod index_regressions {
    use super::tests::{finish, solved, source};
    use super::*;
    #[test]
    fn seven_actual_completed_inputs_keep_exact_certificates_after_four_report_slot_eviction() {
        let base = crate::ui::tests::one_off_workspace::fixture();
        let mut memo = Memo::default();
        let mut cases = Vec::new();
        for n in 0..7 {
            let mut source = base.clone();
            source.id = format!("synthetic-one-off-index-{n}");
            let mut a = Attempt::new(&source, 4);
            for text in &source.solution {
                engine::execute_for_goal(&mut a.state, &source.goal, text).unwrap();
            }
            work::budget::prepare(&mut a, &source).unwrap();
            memo.evaluate(&a, &source, false);
            for _ in 0..10000 {
                if !memo.advance(core::MAX_TICK) {
                    break;
                }
            }
            memo.publish_current(&a, &source).unwrap();
            assert_eq!(memo.passed(&a, &source), Some(true));
            cases.push((source, a));
        }
        assert!(memo.entries.len() <= 4);
        assert_eq!(memo.certificates.observed().0, 7);
        let sources = cases.iter().map(|(s, _)| s.clone()).collect::<Vec<_>>();
        let mut p = Progress {
            schema_version: 1,
            clock: 4,
            favorites: Default::default(),
            attempts: cases
                .iter()
                .map(|(s, a)| (s.id.clone(), a.clone()))
                .collect(),
            migration_receipts: None,
        };
        memo.certificates.page(sources.iter().map(|s| s.id.clone()));
        let before = memo.counters;
        for frame in 0..20 {
            memo.certificates.begin_frame(frame);
            assert!(memo.certificates.pending(&p, &sources).is_none());
            for s in &sources {
                assert_eq!(memo.passed(&p.attempts[&s.id], s), Some(true));
            }
        }
        assert_eq!(
            memo.counters, before,
            "Idle library must not replay evicted valid reports"
        );
        let first = &sources[0];
        p.attempts
            .get_mut(&first.id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .one_off_session
            .as_mut()
            .unwrap()
            .drafts
            .get_mut("/work/job.sh")
            .unwrap()
            .push_str("# unsaved\n");
        assert_eq!(memo.passed(&p.attempts[&first.id], first), None);
        assert_eq!(memo.certificates.pending(&p, &sources), Some(0));
        for s in &sources[1..] {
            assert_eq!(memo.passed(&p.attempts[&s.id], s), Some(true));
        }
    }

    #[test]
    fn stopped_navigation_rekeys_are_admitted_without_replay_and_force_is_explicit() {
        let s = source();
        let mut a = solved(&s);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        m.advance(1);
        assert!(m.pending(&a, &s));
        m.pause(&a, &s);
        m.clear(); // Same presentation path as Back and warm Resume.
        a.visited += 1;
        a.challenge = !a.challenge;
        // Opening help may alter hints; it must not silently undo Stop.
        a.hints = 1.min(s.hints.len());
        m.invalidate(&s.id);
        let old = serde_json::to_vec(&a).unwrap();
        let steps = m.counters.steps;
        m.evaluate(&a, &s, false);
        assert!(m.admitted(&a, &s));
        assert!(m.paused(&a, &s));
        assert!(!m.advance(core::MAX_TICK));
        assert!(m.rows(&a, &s, false).is_none());
        assert_ne!(m.passed(&a, &s), Some(true));
        let counts = m.counters;
        for _ in 0..20 {
            m.evaluate(&a, &s, false);
            assert!(!m.advance(core::MAX_TICK));
        }
        assert_eq!(m.counters.steps, steps);
        assert_eq!(m.counters.meters, counts.meters);
        assert_eq!(m.counters.snapshots, counts.snapshots);
        assert_eq!(serde_json::to_vec(&a).unwrap(), old);
        m.evaluate(&a, &s, true);
        assert!(m.pending(&a, &s));
        finish(&mut m, &a, &s);
        assert_eq!(m.passed(&a, &s), Some(true));
        assert_eq!(serde_json::to_vec(&a).unwrap(), old);
    }
    #[test]
    fn paused_miss_keeps_early_context_bounds_and_does_not_tick_another_entry() {
        let s = source();
        let mut a = solved(&s);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        m.pause(&a, &s);
        m.clear();
        let before = m.counters;
        a.step = s.steps.len();
        m.evaluate(&a, &s, false);
        assert!(!m.admitted(&a, &s));
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(m.counters.meters, before.meters);
        assert_eq!(m.counters.steps, before.steps);
        a = solved(&s);
        m.pause(&a, &s);
        a.transcript.push("x".repeat(work::MAX_ATTEMPT));
        let raw = serde_json::to_vec(&a).unwrap();
        m.evaluate(&a, &s, false);
        assert_eq!(m.counters.snapshots, before.snapshots);
        assert_eq!(m.counters.steps, before.steps);
        assert_eq!(serde_json::to_vec(&a).unwrap(), raw);
        assert!(!m.advance(core::MAX_TICK));
    }
    #[test]
    fn navigation_cancels_only_unselected_checks_and_bounded_stop_ids_do_not_starve_library() {
        let s = source();
        let a = solved(&s);
        let mut other = s.clone();
        other.id = "synthetic-one-off-other".into();
        let b = solved(&other);
        let mut m = Memo::default();
        m.evaluate(&a, &s, false);
        m.evaluate(&b, &other, false);
        m.pause_away_from(Some(&other.id));
        assert!(m.paused(&a, &s));
        assert!(!m.pending(&a, &s));
        assert!(m.pending(&b, &other));
        assert!(!m.certificates.can_queue(&s.id));
        assert!(m.certificates.can_queue(&other.id));
        for n in 0..65 {
            m.remember_stop(&format!("synthetic-stop-{n}"));
        }
        assert_eq!(m.stopped.len(), 64);
        assert!(m.stop_background);
        m.evaluate(&a, &s, true);
        finish(&mut m, &a, &s);
        assert_eq!(m.passed(&a, &s), Some(true));
        m.changed(&s.id); // A genuine change/reset clears only its own key.
        assert_ne!(m.passed(&a, &s), Some(true));
        assert!(m.stopped.len() <= 64);
    }
}
