//! Independent static-server example, no host command or stored response authority.
use crate::{
    apache_static::{Action, EntryKind, State},
    apache_static_workspace as work,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_LESSON: usize = 786_432;
pub const MAX_LEARNING: usize = 1_572_864;
fn concepts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct C {
        #[serde(deserialize_with = "work::text::<_,160>")]
        title: String,
        #[serde(deserialize_with = "work::text::<_,2400>")]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(deserialize_with = "work::text::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "work::text::<_,1600>")]
    pub explanation: String,
    pub action: Action,
}
fn steps<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Step>, D::Error> {
    work::list::<D, Step, 32>(d)
}
fn actions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    work::list::<D, Action, 32>(d)
}
fn expected<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<bool>, D::Error> {
    work::list::<D, bool, 32>(d)
}
fn ids<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(work::list::<D, work::Text<64>, 16>(d)?
        .into_iter()
        .map(|t| t.0)
        .collect())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TryChange {
    #[serde(deserialize_with = "work::text::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "actions")]
    pub actions: Vec<Action>,
    #[serde(deserialize_with = "expected")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Lesson {
    pub definition: work::Definition,
    pub prerequisites: Vec<String>,
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub example_actions: Vec<Step>,
    pub try_change: TryChange,
    pub repair: Vec<Action>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            definition: work::Definition,
            #[serde(deserialize_with = "ids")]
            prerequisites: Vec<String>,
            #[serde(deserialize_with = "work::text::<_,1600>")]
            prerequisite_note: String,
            #[serde(deserialize_with = "concepts")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "work::text::<_,2400>")]
            example_description: String,
            #[serde(deserialize_with = "steps")]
            example_actions: Vec<Step>,
            try_change: TryChange,
            #[serde(deserialize_with = "actions")]
            repair: Vec<Action>,
        }
        let r =
            Wire::deserialize(d).map_err(|_| D::Error::custom("Invalid bounded Apache lesson"))?;
        let v = Self {
            definition: r.definition,
            prerequisites: r.prerequisites,
            prerequisite_note: r.prerequisite_note,
            concepts: r.concepts,
            example_description: r.example_description,
            example_actions: r.example_actions,
            try_change: r.try_change,
            repair: r.repair,
        };
        if !v.admissible() {
            return Err(D::Error::custom("Apache lesson exceeds admission"));
        }
        Ok(v)
    }
}

fn prose(s: &str, n: usize) -> bool {
    s.len() <= n && !s.trim().is_empty() && !s.contains('\0')
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
fn bounded_actions(v: &[Action]) -> bool {
    !v.is_empty() && v.len() <= 32 && v.iter().all(|a| a.validate().is_ok())
}
impl Lesson {
    fn admissible(&self) -> bool {
        self.definition.validate().is_ok()
            && self.prerequisites.len() <= 16
            && self
                .prerequisites
                .iter()
                .enumerate()
                .all(|(i, s)| id(s) && !self.prerequisites[..i].contains(s))
            && prose(&self.prerequisite_note, 1600)
            && prose(&self.example_description, 2400)
            && !self.concepts.is_empty()
            && self.concepts.len() <= 8
            && self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 160) && prose(&c.text, 2400))
            && !self.example_actions.is_empty()
            && self.example_actions.len() <= 32
            && self.example_actions.iter().all(|s| {
                prose(&s.title, 160) && prose(&s.explanation, 1600) && s.action.validate().is_ok()
            })
            && prose(&self.try_change.description, 1600)
            && bounded_actions(&self.try_change.actions)
            && bounded_actions(&self.repair)
            && self.try_change.expected.len() == self.definition.requirements.len()
            && self.try_change.expected.iter().any(|b| !*b)
            && work::within(self, MAX_LESSON)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.admissible() {
            return Err("Invalid bounded Apache teaching/fixture/actions".into());
        }
        let initial = &self.definition.initial;
        if self.definition.checks(initial).iter().all(|r| r.0) {
            return Err("Apache example must need a substantive change".into());
        }
        let mut worked = initial.clone();
        for s in &self.example_actions {
            work::apply(initial, &mut worked, &s.action)?;
        }
        if !self.definition.checks(&worked).iter().all(|r| r.0) {
            return Err(
                "Apache worked example does not satisfy all current/next-start outcomes".into(),
            );
        }
        for a in &self.try_change.actions {
            work::apply(initial, &mut worked, a)?;
        }
        if self
            .definition
            .checks(&worked)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>()
            != self.try_change.expected
        {
            return Err("Apache experiment differs from declared objective rows".into());
        }
        for a in &self.repair {
            work::apply(initial, &mut worked, a)?;
        }
        if !self.definition.checks(&worked).iter().all(|r| r.0) {
            return Err("Apache example repair does not restore all outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, s: &State) -> bool {
        let a = &self.definition.initial.inventory;
        let b = &s.inventory;
        self.admissible()
            && work::admitted(s)
            && a.fixture != b.fixture
            && a.server != b.server
            && a.config_path != b.config_path
            && a.entries
                .iter()
                .filter(|(_, v)| **v == EntryKind::File)
                .all(|(p, _)| b.entries.get(p) != Some(&EntryKind::File))
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
        struct R {
            version: u8,
            fixture: State,
            simulation: State,
            draft: work::Draft,
            view: View,
            selected_action: usize,
            checked: bool,
            #[serde(default, deserialize_with = "work::option_text::<_,512>")]
            notice: Option<String>,
        }
        let r =
            R::deserialize(d).map_err(|_| D::Error::custom("Invalid bounded Apache practice"))?;
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
            return Err(D::Error::custom("Invalid bounded Apache practice"));
        }
        Ok(v)
    }
}
impl Learning {
    fn fits(&self, state: &State, draft: &work::Draft, checked: bool) -> bool {
        #[derive(Serialize)]
        struct Pending<'a> {
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
            &Pending {
                version: 1,
                fixture: &self.fixture,
                simulation: state,
                draft,
                view: self.view,
                selected_action: self.selected_action,
                checked,
                notice: &self.notice,
            },
            MAX_LEARNING,
        )
    }
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate()?;
        let fixture = &l.definition.initial;
        let draft = work::Draft::new(fixture)?;
        let v = Self {
            version: 1,
            fixture: fixture.clone(),
            simulation: fixture.clone(),
            draft,
            view: View::Introduction,
            selected_action: 0,
            checked: false,
            notice: None,
        };
        if !v.valid_for(l) {
            return Err("Apache practice exceeds aggregate admission".into());
        }
        Ok(v)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && work::same_context(&self.fixture, &self.simulation)
            && self.draft.valid_for(&self.simulation)
            && self.selected_action < 32
            && self.notice.as_ref().is_none_or(|n| n.len() <= 512)
            && work::within(self, MAX_LEARNING)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        l.admissible()
            && self.valid()
            && self.fixture == l.definition.initial
            && self.selected_action < l.example_actions.len()
    }
    /// Request edits are read-only; only unapplied config/file text prevents a pass.
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(
                false,
                "Apache practice differs from its authored context".into(),
            )];
        }
        let mut rows = vec![(
            self.draft.matches(&self.simulation),
            if self.draft.matches(&self.simulation) {
                "Current practice configuration/file text is saved".into()
            } else {
                "Save pending practice configuration/file edits before checking".into()
            },
        )];
        rows.extend(l.definition.checks(&self.simulation));
        rows
    }
    pub fn insert_selected(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign Apache practice".into());
        }
        let mut d = self.draft.clone();
        match &l.example_actions[self.selected_action].action {
            Action::SaveConfig { text } => d.config_text = text.clone(),
            Action::WriteFile { path, text } => {
                d.select_file(&self.simulation, path)?;
                d.file_text = text.clone()
            }
            _ => {
                return Err(
                    "This action has no editor text to insert; use its named virtual control"
                        .into(),
                );
            }
        }
        if !d.valid_for(&self.simulation) || !self.fits(&self.simulation, &d, false) {
            return Err("Example editor insertion exceeds admission".into());
        }
        self.draft = d;
        self.checked = false;
        self.notice = None;
        Ok(())
    }
    pub fn perform(&mut self, l: &Lesson, action: &Action) -> Result<bool, String> {
        action.validate().map_err(|e| work::notice(&e.message))?;
        if !self.valid_for(l) {
            return Err("Foreign Apache practice".into());
        }
        let mut candidate = self.simulation.clone();
        let changed = work::apply(&self.fixture, &mut candidate, action)?;
        let mut d = self.draft.clone();
        match action {
            Action::SaveConfig { .. } => d.config_text = candidate.saved.clone(),
            Action::WriteFile { path, .. } if d.selected_file.as_ref() == Some(path) => {
                d.file_text = candidate.files[path].clone()
            }
            _ => {}
        }
        if !self.fits(&candidate, &d, self.checked && !changed) {
            return Err(
                "Apache practice result exceeds aggregate admission; prior work retained".into(),
            );
        }
        self.simulation = candidate;
        self.draft = d;
        self.checked &= !changed;
        Ok(changed)
    }
}
pub fn prepare(a: &mut crate::progress::Attempt, l: &Lesson) -> Result<(), String> {
    if crate::workspace::is_one_off_workspace(a) {
        return Err("One-off workspace belongs to its own activity; previous work retained".into());
    }

    crate::workspace::reject_etherchannel_metadata(a)?;
    crate::workspace::reject_nfs_metadata(a)?;
    let crate::engine::State::ApacheStatic { simulation } = &a.state else {
        return Err("Apache teaching belongs to a different activity kind".into());
    };
    l.validate()?;
    if !l.separate_from(simulation) {
        return Err("Apache example needs separate fixed server/file identities".into());
    }
    if a.workspace.as_ref().is_some_and(|w| {
        !w.valid()
            || w.apache_static_draft
                .as_ref()
                .is_some_and(|d| !d.valid_for(simulation))
            || w.apache_static_learning
                .as_ref()
                .is_some_and(|p| !p.valid_for(l))
    }) {
        return Err(
            "Existing Apache practice/draft has another context; prior work retained".into(),
        );
    }
    let draft = if a
        .workspace
        .as_ref()
        .is_none_or(|w| w.apache_static_draft.is_none())
    {
        Some(work::Draft::new(simulation)?)
    } else {
        None
    };
    let learning = if a
        .workspace
        .as_ref()
        .is_none_or(|w| w.apache_static_learning.is_none())
    {
        Some(Learning::new(l)?)
    } else {
        None
    };
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if let Some(d) = draft {
        w.apache_static_draft = Some(d)
    }
    if let Some(p) = learning {
        w.apache_static_learning = Some(p)
    }
    Ok(())
}
