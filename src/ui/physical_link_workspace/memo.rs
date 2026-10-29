//! Bounded full-source native memo. No persisted report grants completion.
use super::*;
use crate::ui::completion_index::{Family, Handle, Key};
use crate::ui::etherchannel_workspace::memo::charged;
use std::sync::Arc;
const ENTRY_BYTES: usize = 8 * 1024 * 1024;
const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const FRAME_RESERVE: usize = 8 * 1024 * 1024;
const REPORT_RESERVE: usize = 256 * 1024;
fn source_shape(s: &Scenario) -> bool {
    work::source_shape(s)
}
fn foreign(w: &model::WorkspaceData) -> bool {
    work::foreign(w)
}
pub(in crate::ui) fn same_source(a: &Scenario, b: &Scenario) -> bool {
    if !source_shape(a) || !source_shape(b) {
        return false;
    }
    let (Goal::PhysicalLink { definition: ad }, Goal::PhysicalLink { definition: bd }) =
        (&a.goal, &b.goal)
    else {
        return false;
    };
    a.id == b.id
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
        && a.solution == b.solution
        && a.prerequisites == b.prerequisites
        && a.initial == b.initial
        && ad == bd
        && a.physical_link_teaching == b.physical_link_teaching
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
    // No ..: an added persistent field cannot silently disappear from the key.
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
        && a.ipv4.is_none()
        && b.ipv4.is_none()
        && match (&a.workspace, &b.workspace) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                !foreign(a)
                    && !foreign(b)
                    && a.version == b.version
                    && a.command == b.command
                    && a.checked == b.checked
                    && a.physical_link_learning == b.physical_link_learning
                    && a.physical_link_draft == b.physical_link_draft
            }
            _ => false,
        }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) struct Counters {
    pub admissions: usize,
    pub copies: usize,
    pub grades: usize,
    pub hits: usize,
}
struct Entry {
    source: Arc<Scenario>,
    attempt: Arc<Attempt>,
    key: Arc<Key>,
    assignment: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
    views: [std::collections::BTreeMap<String, Result<core::LinkReport, String>>; 2],
    charge: usize,
    touched: u64,
    source_generation: u64,
}
#[derive(Default)]
pub(in crate::ui) struct Memo {
    entries: Vec<Entry>,
    pub certificates: Handle,
    tick: u64,
    source_tick: u64,
    pub counters: Counters,
    pub notice: Option<String>,
}
impl Memo {
    #[cfg(test)]
    pub(in crate::ui) fn entries_count(&self) -> usize {
        self.entries.len()
    }
    pub fn source_generation(&self, s: &Scenario) -> Option<u64> {
        self.entries
            .iter()
            .find(|e| same_source(&e.source, s))
            .map(|e| e.source_generation)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.notice = None;
    }
    fn cheap(a: &Attempt, s: &Scenario) -> bool {
        work::source_shape(s)
            && matches!(a.state, engine::State::PhysicalLink { .. })
            && a.scenario_version == s.version
            && a.step < s.steps.len()
            && a.hints <= s.hints.len()
            && a.ipv4.is_none()
            && a.workspace.as_ref().is_some_and(|w| {
                w.version == 1
                    && !work::foreign(w)
                    && w.physical_link_draft.is_some()
                    && w.physical_link_learning.is_some()
            })
    }
    fn ready(&self, a: &Attempt, s: &Scenario) -> Option<&Entry> {
        if !Self::cheap(a, s) {
            return None;
        }
        self.entries
            .iter()
            .find(|e| same_source(&e.source, s) && same_attempt(&e.attempt, a))
    }
    pub fn rows(&self, a: &Attempt, s: &Scenario, practice: bool) -> Option<&[(bool, String)]> {
        self.ready(a, s).map(|e| {
            if practice {
                e.practice.as_slice()
            } else {
                e.assignment.as_slice()
            }
        })
    }
    pub fn link(
        &self,
        a: &Attempt,
        s: &Scenario,
        practice: bool,
        id: &str,
    ) -> Option<&Result<core::LinkReport, String>> {
        self.ready(a, s)?.views[usize::from(practice)].get(id)
    }
    pub fn passed(&self, a: &Attempt, s: &Scenario) -> Option<bool> {
        self.certificates.status(a, s).and_then(|s| s.result())
    }
    pub fn prepare(&mut self, a: &Attempt, s: &Scenario, force: bool) -> Result<(), String> {
        if !Self::cheap(a, s) {
            self.entries.retain(|e| e.source.id != s.id);
            self.certificates.forget(&s.id);
            return Err("Physical source or workspace differs; your work is retained".into());
        }
        if !force && self.ready(a, s).is_some() {
            if self.certificates.status(a, s) == Some(completion_index::Status::Pending) {
                let e = self.ready(a, s).unwrap();
                self.certificates.publish(
                    &e.key,
                    a,
                    s,
                    !e.assignment.is_empty() && e.assignment.iter().all(|r| r.0),
                )?;
            }
            self.counters.hits += 1;
            return Ok(());
        }
        work::context(a, s)?;
        let charge=charged(s,ENTRY_BYTES).and_then(|x|charged(a,ENTRY_BYTES).and_then(|y|x.checked_add(y))).and_then(|n|n.checked_add(REPORT_RESERVE)).filter(|n|*n<=ENTRY_BYTES).ok_or("This physical workspace exceeds the native snapshot allowance; your work is retained")?;
        // Reserve all existing entries plus one bounded grading/edit frame BEFORE
        // copying a Scenario, Attempt, Definition or simulation State.
        let generation = self.source_generation(s).unwrap_or_else(|| {
            self.source_tick = self.source_tick.saturating_add(1);
            self.source_tick
        });
        self.entries.retain(|e| e.source.id != s.id);
        while self.entries.len() >= 4
            || self
                .entries
                .iter()
                .map(|e| e.charge)
                .sum::<usize>()
                .saturating_add(charge)
                .saturating_add(FRAME_RESERVE)
                > TOTAL_BYTES
        {
            let Some(at) = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.touched)
                .map(|(i, _)| i)
            else {
                return Err("Cannot reserve the next physical workspace".into());
            };
            self.entries.remove(at);
        }
        // Both computations are finite:≤8 links×8 cases, each with two arithmetic
        // directions, no learner loops. Timing acceptance still requires native tests.
        let assignment = work::checks(a, s);
        let practice = work::practice_checks(a, s);
        if assignment.is_empty() || practice.is_empty() {
            return Err("Physical check did not produce objective results".into());
        }
        let State::PhysicalLink { simulation } = &a.state else {
            return Err("Expected physical assignment".into());
        };
        let example = &a
            .workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .current;
        let view = |state: &core::State| {
            state
                .topology()
                .links
                .keys()
                .map(|id| (id.clone(), state.report(id).map_err(|e| e.to_string())))
                .collect()
        };
        let views = [view(simulation), view(example)];
        let (key, copied) = self.certificates.admit(a, s, Family::Physical)?;
        self.certificates.publish(
            &key,
            a,
            s,
            !assignment.is_empty() && assignment.iter().all(|r| r.0),
        )?;
        self.tick = self.tick.saturating_add(1);
        self.entries.push(Entry {
            source: Arc::clone(&key.source),
            attempt: Arc::clone(&key.attempt),
            key,
            assignment,
            practice,
            views,
            charge,
            touched: self.tick,
            source_generation: generation,
        });
        self.counters.admissions += 1;
        self.counters.copies += usize::from(copied);
        self.counters.grades += 2;
        self.notice = None;
        Ok(())
    }
}
