//! Isolated LLDP teaching state, independent of engine, progress and assignment credit.
//! Default and try-change plans are typed and checked on disposable copies. Filling
//! the example projects configuration actions into draft only: it never sends an
//! advertisement/withdrawal, advances practice time or marks any outcome complete.
use crate::{
    discovery::{Check, Requirement, State},
    discovery_workspace::{self, Action, ActionResult, Draft},
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Deserializer, Serialize};

pub const MAX_LESSON_BYTES: usize = 2_097_152;
pub const MAX_LEARNING_BYTES: usize = 2_097_152;
pub const MAX_ACTIONS: usize = 16;
pub const MAX_OUTCOMES: usize = 20;
fn concepts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    discovery_workspace::bounded_list::<D, Concept, 6>(d)
}
fn actions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    discovery_workspace::bounded_list::<D, Action, MAX_ACTIONS>(d)
}
fn outcomes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    discovery_workspace::bounded_list::<D, Requirement, MAX_OUTCOMES>(d)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TryChange {
    pub description: String,
    #[serde(deserialize_with = "actions")]
    pub actions: Vec<Action>,
    #[serde(deserialize_with = "outcomes")]
    pub outcomes: Vec<Requirement>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    #[serde(deserialize_with = "concepts")]
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: State,
    #[serde(deserialize_with = "actions")]
    pub example_solution: Vec<Action>,
    #[serde(deserialize_with = "outcomes")]
    pub outcomes: Vec<Requirement>,
    pub try_change: TryChange,
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max
}
fn valid_outcomes(outcomes: &[Requirement], initial: &State) -> bool {
    !outcomes.is_empty()
        && outcomes.len() <= MAX_OUTCOMES
        && outcomes.iter().any(|r| {
            matches!(
                r,
                Requirement::CurrentNeighbor { .. } | Requirement::AbsentNeighbor { .. }
            )
        })
        && outcomes
            .iter()
            .filter(|r| matches!(r, Requirement::Configuration { .. }))
            .count()
            <= 1
        && outcomes
            .iter()
            .enumerate()
            .all(|(i, r)| r.validate(initial.topology()).is_ok() && !outcomes[..i].contains(r))
}
fn replay(
    initial: &State,
    start: &State,
    actions: &[Action],
    outcomes: &[Requirement],
) -> Result<State, String> {
    if actions.is_empty() || actions.len() > MAX_ACTIONS || !valid_outcomes(outcomes, initial) {
        return Err("Invalid LLDP example action/outcome bounds or references".into());
    }
    let mut candidate = start.clone();
    for action in actions {
        action.apply(&mut candidate)?;
    }
    if outcomes.iter().any(|r| {
        !r.check(initial.topology(), &candidate)
            .is_ok_and(Check::passed)
    }) {
        return Err("LLDP example actions cannot reach their declared current outcomes".into());
    }
    Ok(candidate)
}
impl Lesson {
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_LESSON_BYTES {
            return Err("LLDP lesson exceeds 2 MiB".into());
        }
        let lesson: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid LLDP lesson JSON: {e}"))?;
        lesson.validate()?;
        Ok(lesson)
    }
    /// Reachability is simulation-only and does not mutate this fixture or any learner state.
    pub fn validate(&self) -> Result<(), String> {
        self.fixture.validate().map_err(|e| e.message)?;
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self
                .concepts
                .iter()
                .any(|c| !prose(&c.title, 100) || !prose(&c.text, 1200))
            || [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change.description,
            ]
            .iter()
            .any(|s| !prose(s, 1600))
        {
            return Err("Invalid LLDP teaching description/concept bounds".into());
        }
        let worked = replay(
            &self.fixture,
            &self.fixture,
            &self.example_solution,
            &self.outcomes,
        )?;
        let changed = replay(
            &self.fixture,
            &worked,
            &self.try_change.actions,
            &self.try_change.outcomes,
        )?;
        if changed == worked {
            return Err("LLDP try-change plan must change the worked example's state".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub simulation: State,
    pub draft: Draft,
    pub checked: bool,
    pub notice: Option<String>,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            simulation: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture),
            checked: false,
            notice: None,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.simulation.validate().is_ok()
            && self.draft.same_shape(&self.simulation)
            && self.notice.as_ref().is_none_or(|s| s.len() <= 16_000)
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && lesson.validate().is_ok()
            && self
                .simulation
                .topology()
                .same_identity(lesson.fixture.topology())
    }
    pub fn from_json(text: &str, lesson: &Lesson) -> Result<Self, String> {
        if text.len() > MAX_LEARNING_BYTES {
            return Err("LLDP learning state exceeds 2 MiB".into());
        }
        let learning: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid LLDP learning JSON: {e}"))?;
        if !learning.valid_for(lesson) {
            return Err(
                "Saved LLDP learning state must preserve this example's identities and bounds"
                    .into(),
            );
        }
        Ok(learning)
    }
    /// Only configuration is projected into draft. Events remain explicit learner actions.
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("LLDP practice does not match this example".into());
        }
        let mut configured = lesson.fixture.clone();
        for action in &lesson.example_solution {
            if action.is_configuration() {
                action.apply(&mut configured)?;
            }
        }
        let selected = self.draft.selected_device.clone();
        self.draft = Draft::new(&configured);
        if self.draft.devices.contains_key(&selected) {
            self.draft.selected_device = selected;
        }
        self.checked = false;
        self.notice = None;
        Ok(())
    }
    /// Applies drafts, including protocol cache invalidation, but sends no information advertisement.
    pub fn apply(&mut self) -> Result<(), String> {
        let result = if self.valid() {
            self.draft.apply(&mut self.simulation)
        } else {
            Err("Invalid LLDP practice storage/identity shape".into())
        };
        self.checked = false;
        self.notice = result.as_ref().err().cloned();
        result
    }
    /// Explicit reference/control action. Pending edits must be applied first and are never discarded.
    pub fn execute(&mut self, action: &Action) -> Result<ActionResult, String> {
        let result = if !self.valid() {
            Err("Invalid LLDP practice state".into())
        } else if !self.draft.matches(&self.simulation) {
            Err("Apply or correct pending LLDP drafts before running an explicit action".into())
        } else {
            action.apply(&mut self.simulation)
        };
        if result.is_ok() {
            let selected = self.draft.selected_device.clone();
            self.draft = Draft::new(&self.simulation);
            if self.draft.devices.contains_key(&selected) {
                self.draft.selected_device = selected;
            }
        }
        self.checked = false;
        self.notice = result.as_ref().err().cloned();
        result
    }
    fn evaluate(&self, lesson: &Lesson) -> Result<Vec<Check>, String> {
        if !self.valid_for(lesson) {
            return Err("LLDP practice does not match the validated example".into());
        }
        if !self.draft.matches(&self.simulation) {
            return Err("LLDP editor has unapplied or invalid changes".into());
        }
        lesson
            .outcomes
            .iter()
            .map(|r| {
                r.check(lesson.fixture.topology(), &self.simulation)
                    .map_err(|e| e.message)
            })
            .collect()
    }
    /// Checking practice reveals current outcomes only; this type has no assignment completion field.
    pub fn check(&mut self, lesson: &Lesson) -> Result<Vec<Check>, String> {
        let result = self.evaluate(lesson);
        self.checked = result.is_ok();
        self.notice = result.as_ref().err().cloned();
        result
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.checked {
            return Vec::new();
        }
        if !self.valid_for(lesson) {
            return vec![(
                false,
                "LLDP practice does not match the validated example.".into(),
            )];
        }
        if !self.draft.matches(&self.simulation) {
            return vec![(false, "The LLDP editor has unapplied or invalid changes; apply corrected drafts before checking current results.".into())];
        }
        discovery_workspace::checks(
            lesson.fixture.topology(),
            &self.simulation,
            &lesson.outcomes,
        )
    }
}
