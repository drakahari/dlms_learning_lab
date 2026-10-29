//! Ephemeral exact-input completion certificates, shared by bounded native LRUs.
//! No saved boolean, hash, pointer identity or mutation epoch is a credential.
use super::*;
use crate::ui::etherchannel_workspace::memo::charged;
use std::{cell::RefCell, rc::Rc, sync::Arc};
const KEY_BYTES: usize = 8 * 1024 * 1024 - 256 * 1024;
const INDEX_BYTES: usize = 64 * 1024 * 1024;
const MAX_KEYS: usize = 1000;
const META_RESERVE: usize = 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Family {
    Physical,
    Backup,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Status {
    Pending,
    Complete,
    Incomplete,
}
impl Status {
    pub fn result(self) -> Option<bool> {
        match self {
            Self::Pending => None,
            Self::Complete => Some(true),
            Self::Incomplete => Some(false),
        }
    }
}
fn cheap(a: &Attempt, s: &Scenario, family: Family) -> bool {
    if a.ipv4.is_some()
        || a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
    {
        return false;
    }
    match family {
        Family::Physical => {
            dlms_learning_lab::physical_link_workspace::source_shape(s)
                && matches!(a.state, engine::State::PhysicalLink { .. })
                && a.workspace.as_ref().is_some_and(|w| {
                    w.version == 1
                        && !dlms_learning_lab::physical_link_workspace::foreign(w)
                        && w.physical_link_draft.is_some()
                        && w.physical_link_learning.is_some()
                })
        }
        Family::Backup => {
            file_backup_workspace::source_shape(s)
                && matches!(a.state, engine::State::FileBackup { .. })
                && a.workspace.as_ref().is_some_and(|w| {
                    w.version == 1
                        && !file_backup_workspace::foreign(w)
                        && w.file_backup_learning.is_some()
                })
        }
    }
}
pub(super) fn family(s: &Scenario) -> Option<Family> {
    match s.initial {
        engine::State::PhysicalLink { .. } => Some(Family::Physical),
        engine::State::FileBackup { .. } => Some(Family::Backup),
        _ => None,
    }
}
fn source_eq(a: &Scenario, b: &Scenario, family: Family) -> bool {
    match family {
        Family::Physical => physical_link_workspace::memo::same_source(a, b),
        Family::Backup => file_backup_workspace::same_source(a, b),
    }
}
fn attempt_eq(a: &Attempt, b: &Attempt, family: Family, metadata: bool) -> bool {
    match family {
        Family::Physical => physical_link_workspace::memo::same_attempt_fields(a, b, metadata),
        Family::Backup => file_backup_workspace::same_attempt_fields(a, b, metadata),
    }
}
fn context(a: &Attempt, s: &Scenario, family: Family) -> Result<(), String> {
    if !cheap(a, s, family) {
        return Err("Source or saved work contains a different activity's fields".into());
    }
    match family {
        Family::Physical => {
            dlms_learning_lab::physical_link_workspace::context(a, s)?;
        }
        Family::Backup => {
            dlms_learning_lab::file_backup_workspace::context(a, s)?;
        }
    }
    Ok(())
}
/// Source/Attempt clones exist once per admitted key, aliased by the selected LRU
/// and certificate. No key is copied merely because a native report is evicted.
pub(super) struct Key {
    pub source: Arc<Scenario>,
    pub attempt: Arc<Attempt>,
    family: Family,
    charge: usize,
}
impl Key {
    fn matches(&self, a: &Attempt, s: &Scenario) -> bool {
        cheap(a, s, self.family)
            && source_eq(&self.source, s, self.family)
            && attempt_eq(&self.attempt, a, self.family, false)
    }
}
struct Certificate {
    key: Arc<Key>,
    passed: bool,
    touched: u64,
}
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Counters {
    pub contexts: usize,
    pub key_copies: usize,
    pub certificates: usize,
    pub lookups: usize,
}
#[derive(Default)]
struct Index {
    values: std::collections::BTreeMap<String, Certificate>,
    bytes: usize,
    tick: u64,
    counts: Counters,
    generation: u64,
    frame_work_used: bool,
    root_frame: Option<u64>,
    visible: Vec<String>,
    refused: std::collections::BTreeSet<String>,
}
#[derive(Default, Clone)]
pub(super) struct Handle(Rc<RefCell<Index>>);
impl Handle {
    pub fn begin_frame(&self, root_frame: u64) {
        let mut index = self.0.borrow_mut();
        if index.root_frame != Some(root_frame) {
            index.root_frame = Some(root_frame);
            index.frame_work_used = false;
        }
    }
    pub fn take_work(&self) -> bool {
        let mut i = self.0.borrow_mut();
        if i.frame_work_used {
            false
        } else {
            i.frame_work_used = true;
            true
        }
    }
    pub fn status(&self, a: &Attempt, s: &Scenario) -> Option<Status> {
        let f = family(s)?;
        if !cheap(a, s, f) {
            return Some(Status::Pending);
        }
        let mut index = self.0.borrow_mut();
        index.counts.lookups += 1;
        Some(
            index
                .values
                .get(&s.id)
                .filter(|c| c.key.matches(a, s))
                .map_or(Status::Pending, |c| {
                    if c.passed {
                        Status::Complete
                    } else {
                        Status::Incomplete
                    }
                }),
        )
    }
    pub fn admit(
        &self,
        a: &Attempt,
        s: &Scenario,
        family: Family,
    ) -> Result<(Arc<Key>, bool), String> {
        if !cheap(a, s, family) {
            return Err("Source/workspace cannot be admitted for verification".into());
        }
        let mut index = self.0.borrow_mut();
        if let Some(c) = index.values.get(&s.id).filter(|c| c.key.matches(a, s)) {
            return Ok((Arc::clone(&c.key), false));
        }
        index.counts.contexts += 1;
        context(a, s, family)?;
        let charge = charged(s, KEY_BYTES)
            .and_then(|x| charged(a, KEY_BYTES).and_then(|y| x.checked_add(y)))
            .filter(|n| *n <= KEY_BYTES)
            .ok_or("Exact saved-work proof exceeds its bounded allowance; work is retained")?;
        // A source can be shared with the preceding key only after exact borrowed Eq.
        let source = index
            .values
            .get(&s.id)
            .filter(|c| source_eq(&c.key.source, s, family))
            .map(|c| Arc::clone(&c.key.source));
        index.reserve(&s.id, charge)?;
        let key = Arc::new(Key {
            source: source.unwrap_or_else(|| Arc::new(s.clone())),
            attempt: Arc::new(a.clone()),
            family,
            charge,
        });
        index.counts.key_copies += 1;
        Ok((key, true))
    }
    /// Publish only a completed actual checker result for the still-current key.
    /// A stopped/partial job never calls this method.
    pub fn publish(
        &self,
        key: &Arc<Key>,
        a: &Attempt,
        s: &Scenario,
        passed: bool,
    ) -> Result<(), String> {
        if !key.matches(a, s) {
            return Err("Verification input changed; no completion was published".into());
        }
        let mut index = self.0.borrow_mut();
        index.reserve(&s.id, key.charge)?;
        index.tick = index.tick.saturating_add(1);
        let touched = index.tick;
        index.bytes += key.charge;
        index.values.insert(
            s.id.clone(),
            Certificate {
                key: Arc::clone(key),
                passed,
                touched,
            },
        );
        index.counts.certificates += 1;
        index.refused.remove(&s.id);
        Ok(())
    }
    /// ONLY validated visit and Learn/Challenge changes may carry an actual result.
    /// State, drafts, practice, transcripts, step/hints and every Source field stay
    /// exact. The exhaustive family comparator has no .. pattern. No answer is made.
    pub fn rekey_metadata(&self, a: &Attempt, s: &Scenario) -> Result<bool, String> {
        let Some(f) = family(s) else {
            return Ok(false);
        };
        if !cheap(a, s, f) {
            self.forget(&s.id);
            return Err("Changed source/workspace cannot retain a verification result".into());
        }
        let passed = {
            let index = self.0.borrow();
            index
                .values
                .get(&s.id)
                .filter(|c| {
                    source_eq(&c.key.source, s, f)
                        && a.visited >= c.key.attempt.visited
                        && attempt_eq(&c.key.attempt, a, f, true)
                })
                .map(|c| c.passed)
        };
        let Some(passed) = passed else {
            return Ok(false);
        };
        if self.status(a, s) != Some(Status::Pending) {
            return Ok(true);
        }
        let (key, _) = self.admit(a, s, f)?;
        self.publish(&key, a, s, passed)?;
        Ok(true)
    }
    pub fn forget(&self, id: &str) {
        let mut index = self.0.borrow_mut();
        index.values.remove(id);
        index.bytes = index.values.values().map(|c| c.key.charge).sum();
        index.refused.remove(id);
    }
    pub fn clear(&self) {
        let mut index = self.0.borrow_mut();
        index.generation = index.generation.saturating_add(1);
        index.values.clear();
        index.bytes = 0;
        index.visible.clear();
        index.refused.clear();
    }
    pub fn retain(&self, progress: &progress::Progress, sources: &[Scenario]) {
        let mut index = self.0.borrow_mut();
        index.values.retain(|_, c| {
            progress.attempts.contains_key(&c.key.source.id)
                && sources.iter().any(|s| s.id == c.key.source.id)
        });
        index.bytes = index.values.values().map(|c| c.key.charge).sum();
        index
            .refused
            .retain(|id| progress.attempts.contains_key(id) && sources.iter().any(|s| &s.id == id));
    }
    pub fn can_queue(&self, id: &str) -> bool {
        !self.0.borrow().refused.contains(id)
    }
    pub fn page(&self, ids: impl Iterator<Item = String>) {
        let visible = ids.take(12).filter(|id| id.len() <= 80).collect();
        self.0.borrow_mut().visible = visible;
    }
    pub fn pending(&self, progress: &progress::Progress, sources: &[Scenario]) -> Option<usize> {
        let ids = self.0.borrow().visible.clone();
        for id in ids {
            let Some(s) = sources.iter().find(|s| s.id == id) else {
                continue;
            };
            let Some(a) = progress.attempts.get(&id) else {
                continue;
            };
            if self.0.borrow().refused.contains(&id) {
                continue;
            }
            if self.status(a, s) == Some(Status::Pending) {
                return sources.iter().position(|s| s.id == id);
            }
        }
        None
    }
    pub fn refuse(&self, id: &str) {
        let mut index = self.0.borrow_mut();
        if id.len() <= 80 && index.refused.len() < MAX_KEYS {
            index.refused.insert(id.into());
        }
    }
    #[cfg(test)]
    pub fn generation(&self) -> u64 {
        self.0.borrow().generation
    }
    pub fn observed(&self) -> (usize, usize, Counters) {
        let i = self.0.borrow();
        (i.values.len(), i.bytes, i.counts)
    }
}
impl Index {
    fn reserve(&mut self, id: &str, charge: usize) -> Result<(), String> {
        if charge.saturating_add(META_RESERVE) > INDEX_BYTES {
            return Err("Cannot reserve an exact completion certificate".into());
        }
        if let Some(old) = self.values.remove(id) {
            self.bytes = self.bytes.saturating_sub(old.key.charge);
        }
        while self.values.len() >= MAX_KEYS
            || self
                .bytes
                .saturating_add(charge)
                .saturating_add(META_RESERVE)
                > INDEX_BYTES
        {
            let Some(id) = self
                .values
                .iter()
                .min_by_key(|(_, c)| c.touched)
                .map(|(id, _)| id.clone())
            else {
                return Err("Cannot reserve current verification proof".into());
            };
            let removed = self.values.remove(&id).unwrap();
            if self.refused.len() < MAX_KEYS {
                self.refused.insert(id);
            }
            self.bytes = self.bytes.saturating_sub(removed.key.charge);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
