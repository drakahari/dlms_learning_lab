//! Separate ungraded native OSPF settings practice; no assignment access or events.
use crate::{
    ospf::{Action, State},
    ospf_workspace::{self as workspace, Draft, Requirement},
    script_teaching::{Concept, View},
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, Visitor},
};
use std::fmt;
fn prose_text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} bytes of OSPF teaching text")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() <= N
                && !s
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
            {
                Ok(s.into())
            } else {
                Err(E::custom("OSPF teaching text bound"))
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn paragraph<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    prose_text::<D, 2400>(d)
}
fn explanation<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    prose_text::<D, 800>(d)
}
pub const MAX_ACTIONS: usize = 16;
pub const MAX_LEARNING_BYTES: usize = 1_048_576;
fn actions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    workspace::list::<D, Action, MAX_ACTIONS>(d)
}
fn annotations<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<ExampleAction>, D::Error> {
    workspace::list::<D, ExampleAction, MAX_ACTIONS>(d)
}
fn concepts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    fn title<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        prose_text::<D, 100>(d)
    }
    fn body<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        prose_text::<D, 1200>(d)
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        #[serde(deserialize_with = "title")]
        title: String,
        #[serde(deserialize_with = "body")]
        text: String,
    }
    Ok(workspace::list::<D, Input, 6>(d)?
        .into_iter()
        .map(|c| Concept {
            title: c.title,
            text: c.text,
        })
        .collect())
}
fn outcomes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<bool>, D::Error> {
    workspace::list::<D, bool, 32>(d)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleAction {
    pub action: Action,
    #[serde(deserialize_with = "explanation")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TryChange {
    #[serde(deserialize_with = "paragraph")]
    pub description: String,
    #[serde(deserialize_with = "actions")]
    pub actions: Vec<Action>,
    #[serde(deserialize_with = "outcomes")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "paragraph")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "concepts")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "paragraph")]
    pub example_description: String,
    pub fixture: State,
    #[serde(deserialize_with = "workspace::requirements")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "annotations")]
    pub example_actions: Vec<ExampleAction>,
    pub try_change: TryChange,
    #[serde(deserialize_with = "actions")]
    pub repair: Vec<Action>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Data {
            #[serde(deserialize_with = "paragraph")]
            prerequisite_note: String,
            #[serde(deserialize_with = "concepts")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "paragraph")]
            example_description: String,
            fixture: State,
            #[serde(deserialize_with = "workspace::requirements")]
            requirements: Vec<Requirement>,
            #[serde(deserialize_with = "annotations")]
            example_actions: Vec<ExampleAction>,
            try_change: TryChange,
            #[serde(deserialize_with = "actions")]
            repair: Vec<Action>,
        }
        let d = Data::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded OSPF teaching fields"))?;
        let lesson = Self {
            prerequisite_note: d.prerequisite_note,
            concepts: d.concepts,
            example_description: d.example_description,
            fixture: d.fixture,
            requirements: d.requirements,
            example_actions: d.example_actions,
            try_change: d.try_change,
            repair: d.repair,
        };
        lesson.validate().map_err(|_| {
            D::Error::custom("OSPF teaching actions/outcomes are not reachable or bounded")
        })?;
        Ok(lesson)
    }
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty()
        && s.len() <= max
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
fn ready(initial: &State, state: &State, rs: &[Requirement]) -> bool {
    workspace::checks(initial, state, rs).iter().all(|c| c.0)
}
fn replay(initial: &State, current: &mut State, actions: &[Action]) -> Result<(), String> {
    if actions.is_empty() || actions.len() > MAX_ACTIONS {
        return Err("OSPF example action count is outside the supported bound".into());
    }
    for action in actions {
        current
            .apply_against(initial, action)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        if !self.fixture.valid()
            || !workspace::valid_requirements(&self.fixture, &self.requirements)
            || !prose(&self.prerequisite_note, 1600)
            || !prose(&self.example_description, 2400)
            || self.concepts.is_empty()
            || self.concepts.len() > 6
            || self
                .concepts
                .iter()
                .any(|c| !prose(&c.title, 100) || !prose(&c.text, 1200))
            || self.example_actions.is_empty()
            || self.example_actions.len() > MAX_ACTIONS
            || self
                .example_actions
                .iter()
                .any(|a| !prose(&a.explanation, 800))
            || !prose(&self.try_change.description, 1600)
            || self.try_change.actions.is_empty()
            || self.try_change.actions.len() > MAX_ACTIONS
            || self.try_change.expected.len() != self.requirements.len()
            || !self.try_change.expected.iter().any(|p| !*p)
            || self.repair.is_empty()
            || self.repair.len() > MAX_ACTIONS
        {
            return Err("Invalid bounded OSPF prerequisite/concept/action/example metadata".into());
        }
        if ready(&self.fixture, &self.fixture, &self.requirements) {
            return Err("OSPF practice must require a substantive settings repair".into());
        }
        let mut worked = self.fixture.clone();
        for action in &self.example_actions {
            worked
                .apply_against(&self.fixture, &action.action)
                .map_err(|e| e.to_string())?;
        }
        if !ready(&self.fixture, &worked, &self.requirements) {
            return Err("OSPF worked settings do not reach every declared current outcome".into());
        }
        let mut changed = worked.clone();
        replay(&self.fixture, &mut changed, &self.try_change.actions)?;
        if changed == worked
            || workspace::checks(&self.fixture, &changed, &self.requirements)
                .iter()
                .map(|c| c.0)
                .collect::<Vec<_>>()
                != self.try_change.expected
        {
            return Err("OSPF try-change outcomes differ from the declared current checks".into());
        }
        replay(&self.fixture, &mut changed, &self.repair)?;
        if !ready(&self.fixture, &changed, &self.requirements) {
            return Err("OSPF example repair does not restore its current outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, initial: &State) -> bool {
        if !self.fixture.valid() || !initial.valid() {
            return false;
        }
        self.fixture
            .topology()
            .devices
            .keys()
            .all(|id| !initial.topology().devices.contains_key(id))
            && self
                .fixture
                .topology()
                .devices
                .values()
                .flat_map(|d| d.interfaces.values())
                .all(|p| {
                    !initial
                        .topology()
                        .devices
                        .values()
                        .flat_map(|d| d.interfaces.values())
                        .any(|other| {
                            p.link == other.link
                                || p.address
                                    .as_ref()
                                    .zip(other.address.as_ref())
                                    .is_some_and(|(a, b)| a.address == b.address)
                        })
                })
            && self
                .fixture
                .configuration()
                .routers
                .values()
                .filter_map(|c| c.process.router_id)
                .all(|id| {
                    !initial
                        .configuration()
                        .routers
                        .values()
                        .any(|c| c.process.router_id == Some(id))
                })
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u8,
    pub view: View,
    /// Immutable context metadata, not a saved success credential.
    pub fixture: State,
    pub simulation: State,
    pub draft: Draft,
    pub checked: bool,
}
impl<'de> Deserialize<'de> for Learning {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Data {
            version: u8,
            view: View,
            fixture: State,
            simulation: State,
            draft: Draft,
            checked: bool,
        }
        let data = Data::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded OSPF practice metadata"))?;
        let learning = Self {
            version: data.version,
            view: data.view,
            fixture: data.fixture,
            simulation: data.simulation,
            draft: data.draft,
            checked: data.checked,
        };
        if !learning.valid() {
            return Err(D::Error::custom(
                "Invalid OSPF practice context or editing shape",
            ));
        }
        Ok(learning)
    }
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate()?;
        let mut draft = Draft::new(&lesson.fixture)?;
        if let Some(first) = lesson.example_actions.first() {
            draft.selected = match &first.action {
                Action::SetProcess { router, .. } => crate::ospf::Endpoint::new(
                    router,
                    draft.routers[router].interfaces.keys().next().unwrap(),
                ),
                Action::SetPort { endpoint, .. }
                | Action::SetAddress { endpoint, .. }
                | Action::SetUp { endpoint, .. } => endpoint.clone(),
            };
        }
        Ok(Self {
            version: 1,
            view: View::Introduction,
            fixture: lesson.fixture.clone(),
            simulation: lesson.fixture.clone(),
            draft,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && workspace::same_context(&self.fixture, &self.simulation)
            && self.draft.same_shape(&self.simulation)
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && self.fixture == lesson.fixture
            && workspace::same_context(&lesson.fixture, &self.simulation)
    }
    pub fn from_json(text: &str, lesson: &Lesson) -> Result<Self, String> {
        if text.len() > MAX_LEARNING_BYTES {
            return Err("OSPF practice JSON exceeds 1 MiB".into());
        }
        let learning: Self = serde_json::from_str(text)
            .map_err(|_| "Invalid bounded OSPF practice data".to_owned())?;
        if !learning.valid_for(lesson) {
            return Err("OSPF practice origin differs from this lesson".into());
        }
        Ok(learning)
    }
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("OSPF practice context differs from this lesson".into());
        }
        lesson.validate()?;
        let mut worked = lesson.fixture.clone();
        for a in &lesson.example_actions {
            worked
                .apply_against(&lesson.fixture, &a.action)
                .map_err(|e| e.to_string())?;
        }
        let mut draft = Draft::new(&worked)?;
        draft.selected = self.draft.selected.clone();
        self.draft = draft;
        self.checked = false;
        Ok(())
    }
    pub fn apply_check(&mut self, lesson: &Lesson) -> Result<Vec<(bool, String)>, String> {
        if !self.valid_for(lesson) {
            return Err("OSPF practice identities differ from the immutable example".into());
        }
        self.checked = true;
        match self.draft.apply(&lesson.fixture, &mut self.simulation) {
            Ok(()) => Ok(self.checks(lesson)),
            Err(e) => {
                self.draft.notice = Some(workspace::notice(&e));
                Err(e)
            }
        }
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(lesson) {
            return vec![(
                false,
                "OSPF example context differs from this lesson".into(),
            )];
        }
        if !self.draft.matches(&self.simulation) {
            return vec![(false,"Example settings are pending or invalid; apply them before checking current outcomes".into())];
        }
        workspace::checks(&lesson.fixture, &self.simulation, &lesson.requirements)
    }
}

pub fn prepare(
    attempt: &mut crate::progress::Attempt,
    lesson: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    crate::workspace::reject_etherchannel_metadata(attempt)?;
    crate::workspace::reject_nfs_metadata(attempt)?;
    if !matches!(attempt.state, crate::engine::State::Ospf { .. }) {
        return Err("OSPF practice metadata belongs only to an OSPF workspace".into());
    }
    let w = attempt
        .workspace
        .as_mut()
        .ok_or("Prepare the workspace before OSPF teaching")?;
    if w.ospf_learning.is_none() {
        let mut l = Learning::new(lesson)?;
        if !new_attempt {
            l.view = View::Assignment;
        }
        w.ospf_learning = Some(l);
    }
    Ok(())
}
