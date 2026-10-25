//! Independent bounded DHCP practice; context copies are metadata, never credit.
use crate::{
    dhcpv4::State,
    dhcpv4_workspace::{self as workspace, Action, Draft, Probe, Requirement},
    script_teaching::{Concept, View},
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, Visitor},
};
use std::fmt;
pub const MAX_LEARNING_BYTES: usize = 1_048_576;
pub const MAX_ACTIONS: usize = 16;
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty()
        && s.len() <= n
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
fn prose_text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct T<const N: usize>;
    impl<const N: usize> Visitor<'_> for T<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} bytes of teaching prose")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if prose(s, N) {
                Ok(s.into())
            } else {
                Err(E::custom("DHCP teaching prose bound"))
            }
        }
    }
    d.deserialize_str(T::<N>)
}
fn paragraph<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    prose_text::<D, 2400>(d)
}
fn explanation<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    prose_text::<D, 800>(d)
}
fn concepts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct C {
        #[serde(deserialize_with = "prose_text::<_,100>")]
        title: String,
        #[serde(deserialize_with = "prose_text::<_,1200>")]
        text: String,
    }
    Ok(workspace::list::<D, C, 6>(d)?
        .into_iter()
        .map(|c| Concept {
            title: c.title,
            text: c.text,
        })
        .collect())
}
fn actions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    workspace::list::<D, Action, MAX_ACTIONS>(d)
}
fn optional_notice<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<workspace::Text<512>>::deserialize(d)?.map(|s| s.0))
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
    #[serde(deserialize_with = "workspace::list::<_,bool,65>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: State,
    pub requirements: Vec<Requirement>,
    pub probes: Vec<Probe>,
    pub example_actions: Vec<ExampleAction>,
    pub try_change: TryChange,
    pub repair: Vec<Action>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            #[serde(deserialize_with = "paragraph")]
            prerequisite_note: String,
            #[serde(deserialize_with = "concepts")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "paragraph")]
            example_description: String,
            fixture: State,
            #[serde(deserialize_with = "workspace::requirements")]
            requirements: Vec<Requirement>,
            #[serde(deserialize_with = "workspace::probes")]
            probes: Vec<Probe>,
            #[serde(deserialize_with = "workspace::list::<_,ExampleAction,16>")]
            example_actions: Vec<ExampleAction>,
            try_change: TryChange,
            #[serde(deserialize_with = "actions")]
            repair: Vec<Action>,
        }
        let r = Raw::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded DHCP teaching data"))?;
        let l = Self {
            prerequisite_note: r.prerequisite_note,
            concepts: r.concepts,
            example_description: r.example_description,
            fixture: r.fixture,
            requirements: r.requirements,
            probes: r.probes,
            example_actions: r.example_actions,
            try_change: r.try_change,
            repair: r.repair,
        };
        l.validate().map_err(|_| {
            D::Error::custom("DHCP teaching actions/outcomes are not reachable or bounded")
        })?;
        Ok(l)
    }
}
impl Lesson {
    fn bounded(&self) -> bool {
        prose(&self.prerequisite_note, 2400)
            && prose(&self.example_description, 2400)
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 100) && prose(&c.text, 1200))
            && workspace::valid_goal(&self.fixture, &self.requirements, &self.probes)
            && !self.example_actions.is_empty()
            && self.example_actions.len() <= MAX_ACTIONS
            && self
                .example_actions
                .iter()
                .all(|a| prose(&a.explanation, 800) && a.action.valid(self.fixture.topology()))
            && prose(&self.try_change.description, 2400)
            && !self.try_change.actions.is_empty()
            && self.try_change.actions.len() <= MAX_ACTIONS
            && self
                .try_change
                .actions
                .iter()
                .all(|a| a.valid(self.fixture.topology()))
            && self.try_change.expected.len() <= 65
            && !self.repair.is_empty()
            && self.repair.len() <= MAX_ACTIONS
            && self.repair.iter().all(|a| a.valid(self.fixture.topology()))
            && workspace::within(self, MAX_LEARNING_BYTES)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid DHCP lesson bounds, context or outcomes".into());
        }
        let mut s = self.fixture.clone();
        for a in &self.example_actions {
            workspace::apply(&self.fixture, &mut s, &a.action)?;
        }
        if !workspace::checks(&self.fixture, &s, &self.requirements, &self.probes)
            .iter()
            .all(|c| c.0)
        {
            return Err(
                "DHCP worked example does not satisfy its own current/fresh objectives".into(),
            );
        }
        let before = s.clone();
        for a in &self.try_change.actions {
            workspace::apply(&self.fixture, &mut s, a)?;
        }
        let result: Vec<_> = std::iter::once(true)
            .chain(
                workspace::checks(&self.fixture, &s, &self.requirements, &self.probes)
                    .into_iter()
                    .map(|c| c.0),
            )
            .collect();
        if s == before || result != self.try_change.expected || result.iter().all(|b| *b) {
            return Err("DHCP experiment must change observable state and match declared incomplete outcomes".into());
        }
        for a in &self.repair {
            workspace::apply(&self.fixture, &mut s, a)?;
        }
        if !workspace::checks(&self.fixture, &s, &self.requirements, &self.probes)
            .iter()
            .all(|c| c.0)
        {
            return Err("DHCP example repair does not restore current/fresh outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, initial: &State) -> bool {
        initial.validate().is_ok()
            && self.fixture.validate().is_ok()
            && self.fixture.topology().server != initial.topology().server
            && self
                .fixture
                .topology()
                .clients
                .keys()
                .all(|k| !initial.topology().clients.contains_key(k))
            && self
                .fixture
                .topology()
                .segments
                .keys()
                .all(|k| !initial.topology().segments.contains_key(k))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Learning {
    pub version: u8,
    pub context: State,
    pub view: View,
    pub simulation: State,
    pub draft: Draft,
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
            context: State,
            view: View,
            simulation: State,
            draft: Draft,
            selected_action: usize,
            checked: bool,
            #[serde(default, deserialize_with = "optional_notice")]
            notice: Option<String>,
        }
        let r = Raw::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded DHCP practice data"))?;
        let l = Self {
            version: r.version,
            context: r.context,
            view: r.view,
            simulation: r.simulation,
            draft: r.draft,
            selected_action: r.selected_action,
            checked: r.checked,
            notice: r.notice,
        };
        if !l.valid() {
            return Err(D::Error::custom(
                "Invalid DHCP practice context/aggregate bounds",
            ));
        }
        Ok(l)
    }
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate()?;
        let l = Self {
            version: 1,
            context: lesson.fixture.clone(),
            view: View::Introduction,
            simulation: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture)?,
            selected_action: 0,
            checked: false,
            notice: None,
        };
        if !l.valid() {
            return Err("DHCP practice exceeds aggregate bounds".into());
        }
        Ok(l)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.context.validate().is_ok()
            && self
                .simulation
                .validate_context(self.context.topology())
                .is_ok()
            && self.draft.same_shape(&self.simulation)
            && self.selected_action < MAX_ACTIONS
            && self
                .notice
                .as_ref()
                .is_none_or(|s| s.len() <= 512 && !s.chars().any(char::is_control))
            && workspace::within(self, MAX_LEARNING_BYTES)
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && lesson.bounded()
            && self.context == lesson.fixture
            && self.selected_action < lesson.example_actions.len()
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(lesson) {
            return vec![(
                false,
                "DHCP example belongs to another authored context".into(),
            )];
        }
        let applied = self.draft.matches(&self.simulation);
        std::iter::once((
            applied,
            if applied {
                "Current example policy draft is applied"
            } else {
                "Apply the current example policy draft before checking it"
            }
            .into(),
        ))
        .chain(workspace::checks(
            &lesson.fixture,
            &self.simulation,
            &lesson.requirements,
            &lesson.probes,
        ))
        .collect()
    }
    pub fn execute(&mut self, lesson: &Lesson, action: &Action) -> Result<String, String> {
        if !self.valid_for(lesson) {
            return Err("Foreign DHCP practice; previous example retained".into());
        }
        let mut next = self.simulation.clone();
        let text = workspace::apply(&lesson.fixture, &mut next, action)?;
        let draft = if matches!(action, Action::ApplyPolicy { .. }) {
            let mut d = Draft::new(&next)?;
            d.selected_client = self.draft.selected_client.clone();
            d.advance_seconds = self.draft.advance_seconds.clone();
            d
        } else {
            self.draft.clone()
        };
        let candidate = Self {
            simulation: next,
            draft,
            checked: false,
            notice: Some(workspace::notice(&text)),
            ..self.clone()
        };
        if !candidate.valid_for(lesson) {
            return Err("DHCP practice aggregate bound; previous example retained".into());
        }
        *self = candidate;
        Ok(text)
    }
    pub fn apply_draft(&mut self, lesson: &Lesson) -> Result<(), String> {
        let p = self.draft.snapshot()?;
        self.execute(lesson, &Action::ApplyPolicy { policy: p })?;
        Ok(())
    }
    /// Fill changes only editing data. Exchange/time actions remain explicit controls.
    pub fn fill(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("Foreign DHCP practice; previous draft retained".into());
        }
        let Action::ApplyPolicy { policy } = &lesson.example_actions[self.selected_action].action
        else {
            return Ok(());
        };
        let state = State::new(lesson.fixture.topology().clone(), policy.clone())
            .map_err(|e| workspace::notice(e.message))?;
        let mut draft = Draft::new(&state)?;
        draft.selected_client = self.draft.selected_client.clone();
        draft.advance_seconds = self.draft.advance_seconds.clone();
        let mut candidate = self.clone();
        candidate.draft = draft;
        candidate.checked = false;
        candidate.notice = None;
        if !candidate.valid_for(lesson) {
            return Err("DHCP draft aggregate bound; previous draft retained".into());
        }
        *self = candidate;
        Ok(())
    }
}
pub fn prepare(
    a: &mut crate::progress::Attempt,
    lesson: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    if !matches!(a.state, crate::engine::State::DhcpV4 { .. }) {
        return Err("DHCP teaching requires the DHCP workspace".into());
    }
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if w.dhcpv4_learning.is_none() {
        let mut l = Learning::new(lesson)?;
        if !new_attempt {
            l.view = View::Assignment;
        }
        w.dhcpv4_learning = Some(l);
    }
    Ok(())
}
