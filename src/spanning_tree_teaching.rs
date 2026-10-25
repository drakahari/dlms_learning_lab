//! Independent bounded worked configurations. No protocol events or completion credit.
use crate::{
    script_teaching::{Concept, View},
    spanning_tree::{self as core, State},
    spanning_tree_workspace as work,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_ACTIONS: usize = 32;
pub const MAX_LESSON_BYTES: usize = 524_288;
pub const MAX_LEARNING_BYTES: usize = 1_048_576;
fn prose<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    Ok(work::Text::<N>::deserialize(d)?.0)
}
fn concepts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct C {
        #[serde(deserialize_with = "prose::<_,160>")]
        title: String,
        #[serde(deserialize_with = "prose::<_,2400>")]
        text: String,
    }
    Ok(work::list::<D, C, 8>(d)?
        .into_iter()
        .map(|c| Concept {
            title: c.title,
            text: c.text,
        })
        .collect())
}
fn actions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<core::Action>, D::Error> {
    work::list::<D, core::Action, MAX_ACTIONS>(d)
}
fn steps<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<ActionStep>, D::Error> {
    work::list::<D, ActionStep, MAX_ACTIONS>(d)
}
fn expected<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<bool>, D::Error> {
    work::list::<D, bool, 32>(d)
}
fn prerequisites<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(work::list::<D, work::Text<64>, 16>(d)?
        .into_iter()
        .map(|s| s.0)
        .collect())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionStep {
    #[serde(deserialize_with = "prose::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "prose::<_,1600>")]
    pub explanation: String,
    pub action: core::Action,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TryChange {
    #[serde(deserialize_with = "prose::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "actions")]
    pub actions: Vec<core::Action>,
    #[serde(deserialize_with = "expected")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "prerequisites")]
    pub prerequisites: Vec<String>,
    #[serde(deserialize_with = "prose::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "concepts")]
    pub concepts: Vec<Concept>,
    pub fixture: State,
    #[serde(deserialize_with = "work::requirements")]
    pub requirements: Vec<work::Requirement>,
    #[serde(deserialize_with = "steps")]
    pub example_actions: Vec<ActionStep>,
    pub try_change: TryChange,
    #[serde(deserialize_with = "actions")]
    pub repair: Vec<core::Action>,
}
fn readable(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && work::bounded(s, n)
}
fn admissible_actions(a: &[core::Action], t: &core::Topology) -> bool {
    !a.is_empty()
        && a.len() <= MAX_ACTIONS
        && a.iter()
            .all(|a| a.validate_for(t).is_ok() && work::within(a, core::MAX_ACTION_BYTES))
}
fn replay(s: &mut State, actions: &[core::Action]) -> Result<(), String> {
    for a in actions {
        s.apply(a).map_err(|e| work::notice(&e.message))?;
    }
    Ok(())
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        // Validate fixed bounds first; action context checks may borrow this graph.
        self.fixture
            .validate()
            .map_err(|e| work::notice(&e.message))?;
        if self.prerequisites.len() > 16
            || self.prerequisites.iter().any(|s| {
                !readable(s, 64)
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            })
            || self
                .prerequisites
                .iter()
                .enumerate()
                .any(|(i, p)| self.prerequisites[..i].contains(p))
            || !readable(&self.prerequisite_note, 1600)
            || self.concepts.is_empty()
            || self.concepts.len() > 8
            || self
                .concepts
                .iter()
                .any(|c| !readable(&c.title, 160) || !readable(&c.text, 2400))
            || self.example_actions.is_empty()
            || self.example_actions.len() > MAX_ACTIONS
            || self.example_actions.iter().any(|a| {
                !readable(&a.title, 160)
                    || !readable(&a.explanation, 1600)
                    || a.action.validate_for(self.fixture.topology()).is_err()
                    || !work::within(&a.action, core::MAX_ACTION_BYTES)
            })
            || !readable(&self.try_change.description, 1600)
            || !admissible_actions(&self.try_change.actions, self.fixture.topology())
            || !admissible_actions(&self.repair, self.fixture.topology())
            || self.try_change.expected.len() != self.requirements.len()
            || !self.try_change.expected.iter().any(|v| !*v)
            || !work::valid_requirements(&self.fixture, &self.requirements)
            || !work::within(self, MAX_LESSON_BYTES)
        {
            return Err("Invalid bounded spanning-tree introduction, example or outcomes".into());
        }
        if work::checks(&self.fixture, &self.fixture, &self.requirements)
            .iter()
            .all(|r| r.0)
        {
            return Err(
                "Spanning-tree example must require a substantive configuration change".into(),
            );
        }
        let mut worked = self.fixture.clone();
        for a in &self.example_actions {
            worked
                .apply(&a.action)
                .map_err(|e| work::notice(&e.message))?;
        }
        if !work::checks(&self.fixture, &worked, &self.requirements)
            .iter()
            .all(|r| r.0)
        {
            return Err("Worked example does not reach every current spanning-tree outcome".into());
        }
        let mut changed = worked.clone();
        replay(&mut changed, &self.try_change.actions)?;
        if changed == worked
            || work::checks(&self.fixture, &changed, &self.requirements)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>()
                != self.try_change.expected
        {
            return Err("Example experiment differs from declared current outcomes".into());
        }
        replay(&mut changed, &self.repair)?;
        if !work::checks(&self.fixture, &changed, &self.requirements)
            .iter()
            .all(|r| r.0)
        {
            return Err("Example repair does not restore all current outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, s: &State) -> bool {
        self.fixture.validate().is_ok()
            && s.validate().is_ok()
            && self
                .fixture
                .topology()
                .bridges
                .keys()
                .all(|k| !s.topology().bridges.contains_key(k))
            && self
                .fixture
                .topology()
                .links
                .keys()
                .all(|k| !s.topology().links.contains_key(k))
            && self
                .fixture
                .topology()
                .bridges
                .values()
                .all(|b| s.topology().bridges.values().all(|o| b.mac != o.mac))
    }
    pub fn example_configuration(&self) -> Result<core::Configuration, String> {
        self.validate()?;
        let mut worked = self.fixture.clone();
        for a in &self.example_actions {
            worked
                .apply(&a.action)
                .map_err(|e| work::notice(&e.message))?;
        }
        Ok(worked.configuration().clone())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Learning {
    pub version: u8,
    pub fixture: State,
    pub simulation: State,
    pub draft: work::Draft,
    pub view: View,
    pub selected_action: usize,
    pub checked: bool,
    pub notice: Option<String>,
}
impl<'de> Deserialize<'de> for Learning {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u8,
            fixture: State,
            simulation: State,
            draft: work::Draft,
            view: View,
            selected_action: usize,
            checked: bool,
            #[serde(default, deserialize_with = "work::optional_notice")]
            notice: Option<String>,
        }
        let r = Raw::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded spanning-tree practice"))?;
        let v = Self {
            version: r.version,
            fixture: r.fixture,
            simulation: r.simulation,
            draft: r.draft,
            view: r.view,
            selected_action: r.selected_action,
            checked: r.checked,
            notice: r.notice,
        };
        if !v.valid() {
            return Err(D::Error::custom(
                "Foreign or oversized spanning-tree practice",
            ));
        }
        Ok(v)
    }
}
fn admitted_parts(
    fixture: &State,
    simulation: &State,
    draft: &work::Draft,
    view: View,
    selected_action: usize,
    checked: bool,
    notice: &Option<String>,
) -> bool {
    #[derive(Serialize)]
    struct Parts<'a> {
        version: u8,
        fixture: &'a State,
        simulation: &'a State,
        draft: &'a work::Draft,
        view: View,
        selected_action: usize,
        checked: bool,
        notice: &'a Option<String>,
    }
    work::within(
        &Parts {
            version: 1,
            fixture,
            simulation,
            draft,
            view,
            selected_action,
            checked,
            notice,
        },
        MAX_LEARNING_BYTES,
    )
}
impl Learning {
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate()?;
        let draft = work::Draft::new(&l.fixture)?;
        if !admitted_parts(
            &l.fixture,
            &l.fixture,
            &draft,
            View::Introduction,
            0,
            false,
            &None,
        ) {
            return Err("Spanning-tree practice exceeds aggregate bound".into());
        }
        let v = Self {
            version: 1,
            fixture: l.fixture.clone(),
            simulation: l.fixture.clone(),
            draft,
            view: View::Introduction,
            selected_action: 0,
            checked: false,
            notice: None,
        };
        if !v.valid_for(l) {
            return Err("Spanning-tree practice exceeds aggregate bound".into());
        }
        Ok(v)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.fixture.validate().is_ok()
            && self
                .simulation
                .validate_context(self.fixture.topology())
                .is_ok()
            && self.draft.valid_for(&self.simulation)
            && self.selected_action < MAX_ACTIONS
            && self.notice.as_ref().is_none_or(|s| work::bounded(s, 512))
            && work::within(self, MAX_LEARNING_BYTES)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid() && self.fixture == l.fixture && self.selected_action < l.example_actions.len()
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(
                false,
                "Spanning-tree practice differs from its authored fixture".into(),
            )];
        }
        let mut rows = vec![(
            self.draft.matches(&self.simulation),
            if self.draft.matches(&self.simulation) {
                "Current practice configuration is applied".into()
            } else {
                "Apply the pending practice configuration before checking the example".into()
            },
        )];
        rows.extend(work::checks(&l.fixture, &self.simulation, &l.requirements));
        rows
    }
    pub fn fill_example(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign spanning-tree practice fixture".into());
        }
        let c = l.example_configuration()?;
        self.draft.fill(&self.simulation, &c)?;
        self.checked = false;
        self.notice = Some(
            "Worked settings inserted into this practice draft only. Apply and inspect the result."
                .into(),
        );
        Ok(())
    }
    pub fn apply(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign spanning-tree practice fixture".into());
        }
        let mut candidate = self.simulation.clone();
        let mut draft = self.draft.clone();
        if let Err(e) = draft.apply(&self.fixture, &mut candidate) {
            self.checked = false;
            self.draft.notice = Some(work::notice(&e));
            self.notice = Some(work::notice(&e));
            return Err(e);
        }
        let message = Some(
            "Practice configuration applied; inspect the new ideal steady-state result".into(),
        );
        if !admitted_parts(
            &self.fixture,
            &candidate,
            &draft,
            self.view,
            self.selected_action,
            false,
            &message,
        ) {
            return Err(
                "Spanning-tree practice exceeds aggregate bound; previous work retained".into(),
            );
        }
        self.simulation = candidate;
        self.draft = draft;
        self.checked = false;
        self.notice = message;
        Ok(())
    }
    pub fn execute(&mut self, l: &Lesson, input: &str) -> Result<String, String> {
        if input.len() > work::PREFIX.len() + core::MAX_ACTION_BYTES {
            return Err("Spanning-tree typed input exceeds its bounded prefix/action limit".into());
        }
        if !self.valid_for(l) {
            return Err("Foreign spanning-tree practice fixture".into());
        }
        let mut candidate = self.simulation.clone();
        let output = work::execute(&self.fixture, &mut candidate, input)?;
        let draft = work::Draft::new(&candidate)?;
        let message = Some(work::notice(&output));
        if !admitted_parts(
            &self.fixture,
            &candidate,
            &draft,
            self.view,
            self.selected_action,
            false,
            &message,
        ) {
            return Err(
                "Spanning-tree practice exceeds aggregate bound; previous work retained".into(),
            );
        }
        self.simulation = candidate;
        self.draft = draft;
        self.checked = false;
        self.notice = message;
        Ok(output)
    }
}

/// Initialize only this activity's optional metadata after all context checks.
/// Wrong kind/foreign existing practice never publishes partial new fields.
pub fn prepare(a: &mut crate::progress::Attempt, lesson: &Lesson) -> Result<(), String> {
    let crate::engine::State::SpanningTree { simulation } = &a.state else {
        return Err("Spanning-tree practice requires its own assignment kind".into());
    };
    lesson.validate()?;
    if !lesson.separate_from(simulation) {
        return Err("Spanning-tree practice needs separate fixed identities".into());
    }
    let old = a.workspace.as_ref();
    if old.is_some_and(|w| {
        !w.valid()
            || w.spanning_tree_draft
                .as_ref()
                .is_some_and(|d| !d.valid_for(simulation))
            || w.spanning_tree_learning
                .as_ref()
                .is_some_and(|l| !l.valid_for(lesson))
    }) {
        return Err(
            "Existing spanning-tree metadata has another context; prior work retained".into(),
        );
    }
    let draft = if old.is_none_or(|w| w.spanning_tree_draft.is_none()) {
        Some(work::Draft::new(simulation)?)
    } else {
        None
    };
    let learning = if old.is_none_or(|w| w.spanning_tree_learning.is_none()) {
        Some(Learning::new(lesson)?)
    } else {
        None
    };
    let w = a
        .workspace
        .get_or_insert_with(|| crate::workspace::WorkspaceData {
            version: 1,
            ..Default::default()
        });
    if let Some(d) = draft {
        w.spanning_tree_draft = Some(d);
    }
    if let Some(l) = learning {
        w.spanning_tree_learning = Some(l);
    }
    Ok(())
}
