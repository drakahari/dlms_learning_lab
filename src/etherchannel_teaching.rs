//! Persisted independent practice for the disclosed L2 LACP workspace.
use crate::{etherchannel as core, etherchannel_workspace as work, script_teaching::View};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_LESSON: usize = 1024 * 1024;
pub const MAX_LEARNING: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "work::string::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "work::string::<_,1200>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionStep {
    #[serde(deserialize_with = "work::string::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "work::string::<_,800>")]
    pub explanation: String,
    pub action: core::Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    #[serde(deserialize_with = "work::string::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "work::list::<_,core::Action,16>")]
    pub actions: Vec<core::Action>,
    #[serde(deserialize_with = "work::list::<_,bool,33>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lesson {
    pub definition: work::Definition,
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub example_actions: Vec<ActionStep>,
    pub try_change: Experiment,
    pub repair: Vec<core::Action>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            definition: work::Definition,
            #[serde(deserialize_with = "work::string::<_,1600>")]
            prerequisite_note: String,
            #[serde(deserialize_with = "work::list::<_,Concept,6>")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "work::string::<_,1600>")]
            example_description: String,
            #[serde(deserialize_with = "work::list::<_,ActionStep,16>")]
            example_actions: Vec<ActionStep>,
            try_change: Experiment,
            #[serde(deserialize_with = "work::list::<_,core::Action,16>")]
            repair: Vec<core::Action>,
        }
        let r = Raw::deserialize(d)?;
        let l = Self {
            definition: r.definition,
            prerequisite_note: r.prerequisite_note,
            concepts: r.concepts,
            example_description: r.example_description,
            example_actions: r.example_actions,
            try_change: r.try_change,
            repair: r.repair,
        };
        if !l.bounded() {
            return Err(D::Error::custom("LACP Lesson scalar/action/context bound"));
        }
        Ok(l)
    }
}
fn prose(s: &str, n: usize) -> bool {
    work::text(s, n) && !s.trim().is_empty()
}
pub fn separate(a: &core::State, b: &core::State) -> bool {
    let a = a.topology();
    let b = b.topology();
    a.fixture != b.fixture
        && a.devices.keys().all(|n| !b.devices.contains_key(n))
        && a.cables.keys().all(|n| !b.cables.contains_key(n))
        && a.devices.values().all(|d| {
            b.devices
                .values()
                .all(|p| d.mac != p.mac && d.groups.keys().all(|n| !p.groups.contains_key(n)))
        })
}
impl Lesson {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_LESSON {
            return Err("LACP Lesson raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded LACP Lesson".into())
    }
    pub fn bounded(&self) -> bool {
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.example_actions.is_empty()
            || self.example_actions.len() > 16
            || self.try_change.actions.is_empty()
            || self.try_change.actions.len() > 16
            || self.repair.is_empty()
            || self.repair.len() > 16
            || self.try_change.expected.len() != self.definition.row_count()
        {
            return false;
        }
        if !prose(&self.prerequisite_note, 1600)
            || !prose(&self.example_description, 1600)
            || !prose(&self.try_change.description, 1600)
            || self
                .concepts
                .iter()
                .any(|c| !prose(&c.title, 160) || !prose(&c.text, 1200))
            || self
                .example_actions
                .iter()
                .any(|a| !prose(&a.title, 160) || !prose(&a.explanation, 800))
        {
            return false;
        }
        self.definition.validate().is_ok()
            && self
                .example_actions
                .iter()
                .map(|s| &s.action)
                .chain(&self.try_change.actions)
                .chain(&self.repair)
                .all(|a| match a {
                    core::Action::Configure { configuration } => {
                        work::admit_configuration(&self.definition.initial, configuration).is_ok()
                    }
                })
            && work::within(self, MAX_LESSON)
    }
    pub fn separate_from(&self, s: &core::State) -> bool {
        self.bounded() && s.validate().is_ok() && separate(&self.definition.initial, s)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("LACP Lesson bound or supplied context differs".into());
        }
        let d = &self.definition;
        if d.checks(&d.initial).iter().all(|r| r.0) {
            return Err("LACP practice needs a substantive configuration change".into());
        }
        let mut s = d.initial.clone();
        for step in &self.example_actions {
            work::perform(d, &mut s, &step.action)?;
        }
        if !d.checks(&s).iter().all(|r| r.0) {
            return Err("LACP worked practice is not reachable".into());
        }
        for a in &self.try_change.actions {
            work::perform(d, &mut s, a)?;
        }
        let actual = d.checks(&s).iter().map(|r| r.0).collect::<Vec<_>>();
        if actual != self.try_change.expected || actual.iter().all(|v| *v) {
            return Err("LACP experiment needs an exact genuine adverse outcome vector".into());
        }
        for a in &self.repair {
            work::perform(d, &mut s, a)?;
        }
        if !d.checks(&s).iter().all(|r| r.0) {
            return Err("LACP practice repair does not restore useful outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub context: work::Definition,
    pub simulation: core::State,
    pub draft: work::Draft,
    pub selected_example: usize,
    pub checked: bool,
}
impl<'de> Deserialize<'de> for Learning {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            view: View,
            context: work::Definition,
            simulation: core::State,
            draft: work::Draft,
            selected_example: usize,
            checked: bool,
        }
        let r = Raw::deserialize(d)?;
        let l = Self {
            version: r.version,
            view: r.view,
            context: r.context,
            simulation: r.simulation,
            draft: r.draft,
            selected_example: r.selected_example,
            checked: r.checked,
        };
        if !l.valid() {
            return Err(D::Error::custom("LACP Learning bound or context differs"));
        }
        Ok(l)
    }
}
#[derive(Serialize)]
pub(crate) struct Proposed<'a, T: Serialize + ?Sized, D: Serialize + ?Sized> {
    version: u32,
    view: View,
    context: &'a work::Definition,
    simulation: &'a T,
    draft: &'a D,
    selected_example: usize,
    checked: bool,
}
impl Learning {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_LEARNING {
            return Err("LACP Learning raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded LACP Learning".into())
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.selected_example < 16
            && self.context.validate().is_ok()
            && self.simulation.validate().is_ok()
            && self.context.same_assignment(&self.simulation)
            && self
                .draft
                .valid_for_definition(&self.context, &self.simulation)
            && work::within(self, MAX_LEARNING)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        l.bounded()
            && self.valid()
            && self.context == l.definition
            && self.selected_example < l.example_actions.len()
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(
                false,
                "LACP practice differs from its exact supplied example".into(),
            )];
        }
        let mut rows = self.context.checks(&self.simulation);
        if !self.draft.matches(&self.simulation) {
            rows[0]=(false,"Practice configuration changes are pending; Apply the current draft before checking".into());
        }
        rows
    }
    pub fn new_size(l: &Lesson, new: bool) -> Option<usize> {
        Self::new_size_view(
            l,
            if new {
                View::Introduction
            } else {
                View::Assignment
            },
        )
    }
    pub(crate) fn new_size_view(l: &Lesson, view: View) -> Option<usize> {
        if !l.bounded() {
            return None;
        }
        work::fresh_draft_size(
            l.definition.initial.configuration(),
            l.definition.initial.topology(),
        )?;
        let draft = work::draft_view(l.definition.initial.configuration());
        work::size(
            &Proposed {
                version: 1,
                view,
                context: &l.definition,
                simulation: &l.definition.initial,
                draft: &draft,
                selected_example: 0,
                checked: false,
            },
            MAX_LEARNING,
        )
    }
    pub fn new(l: &Lesson, new: bool) -> Result<Self, String> {
        Self::new_view(
            l,
            if new {
                View::Introduction
            } else {
                View::Assignment
            },
        )
    }
    pub(crate) fn new_view(l: &Lesson, view: View) -> Result<Self, String> {
        Self::new_size_view(l, view).ok_or("LACP new practice aggregate bound")?;
        #[cfg(test)]
        NEW_LEARNING_COPIES.with(|n| n.set(n.get() + 1));
        let draft = work::Draft::from_state(&l.definition.initial)?;
        Ok(Self {
            version: 1,
            view,
            context: l.definition.clone(),
            simulation: l.definition.initial.clone(),
            draft,
            selected_example: 0,
            checked: false,
        })
    }
    pub(crate) fn proposed_size<T: Serialize + ?Sized, D: Serialize + ?Sized>(
        &self,
        simulation: &T,
        draft: &D,
        view: View,
        selected: usize,
        checked: bool,
    ) -> Option<usize> {
        work::size(
            &Proposed {
                version: self.version,
                view,
                context: &self.context,
                simulation,
                draft,
                selected_example: selected,
                checked,
            },
            MAX_LEARNING,
        )
    }
}
pub fn prepare(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    new: bool,
) -> Result<(), String> {
    work::prepare_attempt(a, s, new)
}

#[cfg(test)]
thread_local! {pub(crate) static NEW_LEARNING_COPIES:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
