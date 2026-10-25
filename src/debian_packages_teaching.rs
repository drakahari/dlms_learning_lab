//! Separate bounded practice for logical package operations; no host command alias.
use crate::{
    debian_packages::{Action, State},
    debian_packages_workspace::{self as work, Definition, Draft},
    script_teaching::View,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_LESSON_BYTES: usize = 1_048_576;
pub const MAX_LEARNING_BYTES: usize = 1_048_576;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "work::string::<_,128>")]
    pub title: String,
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleAction {
    pub action: Action,
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub explanation: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub description: String,
    #[serde(deserialize_with = "work::list::<_,Action,32>")]
    pub actions: Vec<Action>,
    #[serde(deserialize_with = "work::list::<_,bool,32>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lesson {
    pub definition: Definition,
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub example_actions: Vec<ExampleAction>,
    pub try_change: Experiment,
    pub repair: Vec<Action>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct R {
            definition: Definition,
            #[serde(deserialize_with = "work::string::<_,2048>")]
            prerequisite_note: String,
            #[serde(deserialize_with = "work::list::<_,Concept,8>")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "work::string::<_,4096>")]
            example_description: String,
            #[serde(deserialize_with = "work::list::<_,ExampleAction,32>")]
            example_actions: Vec<ExampleAction>,
            try_change: Experiment,
            #[serde(deserialize_with = "work::list::<_,Action,32>")]
            repair: Vec<Action>,
        }
        let r = R::deserialize(d)?;
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
            return Err(D::Error::custom("Invalid package lesson bounds"));
        }
        Ok(l)
    }
}
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.contains('\0')
}
impl Lesson {
    pub fn bounded(&self) -> bool {
        if self.concepts.is_empty()
            || self.concepts.len() > 8
            || self.example_actions.is_empty()
            || self.example_actions.len() > 32
            || self.try_change.actions.is_empty()
            || self.try_change.actions.len() > 32
            || self.repair.is_empty()
            || self.repair.len() > 32
            || self.try_change.expected.len() > 32
        {
            return false;
        }
        prose(&self.prerequisite_note, 2048)
            && prose(&self.example_description, 4096)
            && prose(&self.try_change.description, 2048)
            && self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 128) && prose(&c.text, 2048))
            && self
                .example_actions
                .iter()
                .all(|a| prose(&a.explanation, 2048) && work::action_bounded(&a.action))
            && self
                .try_change
                .actions
                .iter()
                .chain(&self.repair)
                .all(work::action_bounded)
            && self.definition.validate().is_ok()
            && self.try_change.expected.len() == self.definition.requirements.len()
            && self.try_change.expected.iter().any(|v| !*v)
            && work::within(self, MAX_LESSON_BYTES)
    }
    pub fn separate_from(&self, assignment: &State) -> bool {
        self.bounded() && separate_contexts(&self.definition.initial, assignment)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err(
                "Package lesson bounded metadata/positive goal/experiment shape differs".into(),
            );
        }
        let mut worked = self.definition.initial.clone();
        for action in &self.example_actions {
            work::apply(&self.definition, &mut worked, &action.action)?;
        }
        if !self.definition.checks(&worked).iter().all(|c| c.0) {
            return Err("Package worked example does not meet its current outcomes".into());
        }
        let mut changed = worked.clone();
        for action in &self.try_change.actions {
            work::apply(&self.definition, &mut changed, action)?;
        }
        if changed == worked
            || self
                .definition
                .checks(&changed)
                .iter()
                .map(|c| c.0)
                .collect::<Vec<_>>()
                != self.try_change.expected
        {
            return Err(
                "Package experiment must actually change state and match its declared outcomes"
                    .into(),
            );
        }
        for action in &self.repair {
            work::apply(&self.definition, &mut changed, action)?;
        }
        if !self.definition.checks(&changed).iter().all(|c| c.0) {
            return Err("Package experiment repair does not meet its current outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub context: State,
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
        struct R {
            version: u32,
            view: View,
            context: State,
            simulation: State,
            draft: Draft,
            selected_action: usize,
            checked: bool,
            #[serde(default, deserialize_with = "optional_notice")]
            notice: Option<String>,
        }
        let r = R::deserialize(d)?;
        let l = Self {
            version: r.version,
            view: r.view,
            context: r.context,
            simulation: r.simulation,
            draft: r.draft,
            selected_action: r.selected_action,
            checked: r.checked,
            notice: r.notice,
        };
        if !l.valid() {
            return Err(D::Error::custom("Invalid bounded package practice"));
        }
        Ok(l)
    }
}
fn optional_notice<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<work::Text<512>>::deserialize(d)?.map(|s| s.0))
}
impl Learning {
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.selected_action < 32
            && self
                .notice
                .as_ref()
                .is_none_or(|s| s.len() <= 512 && !s.contains('\0'))
            && work::admitted(&self.context)
            && work::admitted(&self.simulation)
            && self.simulation.catalog() == self.context.catalog()
            && self.draft.valid_for(&self.simulation)
            && work::within(self, MAX_LEARNING_BYTES)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        l.bounded()
            && self.valid()
            && self.context == l.definition.initial
            && self.selected_action < l.example_actions.len()
    }
    pub fn new(l: &Lesson) -> Result<Self, String> {
        if !l.bounded() {
            return Err("Package lesson bounds prevent practice creation".into());
        }
        let draft = Draft::new(&l.definition.initial)?;
        let v = Self {
            version: 1,
            view: View::Introduction,
            context: l.definition.initial.clone(),
            simulation: l.definition.initial.clone(),
            draft,
            selected_action: 0,
            checked: false,
            notice: None,
        };
        if !v.valid_for(l) {
            return Err("Package practice exceeds aggregate app admission".into());
        }
        Ok(v)
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(
                false,
                "Foreign package practice fixture or draft context".into(),
            )];
        }
        l.definition.checks(&self.simulation)
    }
    pub fn fill(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Invalid package practice; original retained".into());
        }
        let mut next = self.draft.clone();
        next.fill(
            &self.simulation,
            &l.example_actions[self.selected_action].action,
        )?;
        self.save_draft(l, &next)?;
        self.notice = None;
        Ok(())
    }
    pub fn save_draft(&mut self, l: &Lesson, draft: &Draft) -> Result<(), String> {
        if !self.valid_for(l) || !draft.valid_for(&self.simulation) {
            return Err("Foreign/oversized package practice draft; original retained".into());
        }
        #[derive(Serialize)]
        struct B<'a> {
            version: u32,
            view: View,
            context: &'a State,
            simulation: &'a State,
            draft: &'a Draft,
            selected_action: usize,
            checked: bool,
            notice: &'a Option<String>,
        }
        if !work::within(
            &B {
                version: self.version,
                view: self.view,
                context: &self.context,
                simulation: &self.simulation,
                draft,
                selected_action: self.selected_action,
                checked: self.checked,
                notice: &self.notice,
            },
            MAX_LEARNING_BYTES,
        ) {
            return Err("Package practice draft aggregate bound".into());
        }
        self.draft = draft.clone();
        Ok(())
    }
    pub fn execute(
        &mut self,
        l: &Lesson,
        a: &Action,
    ) -> Result<crate::debian_packages::Plan, String> {
        if !self.valid_for(l) || !work::action_bounded(a) {
            return Err("Invalid package practice action/context; original retained".into());
        }
        let mut state = self.simulation.clone();
        let report = work::apply(&l.definition, &mut state, a)?;
        #[derive(Serialize)]
        struct B<'a> {
            version: u32,
            view: View,
            context: &'a State,
            simulation: &'a State,
            draft: &'a Draft,
            selected_action: usize,
            checked: bool,
            notice: Option<String>,
        }
        let message = work::notice(format!(
            "Logical operation: {} package record(s) and {} file(s) changed",
            report.packages.len(),
            report.changed_files.len()
        ));
        if !work::within(
            &B {
                version: self.version,
                view: self.view,
                context: &self.context,
                simulation: &state,
                draft: &self.draft,
                selected_action: self.selected_action,
                checked: false,
                notice: Some(message.clone()),
            },
            MAX_LEARNING_BYTES,
        ) {
            return Err("Package practice result aggregate bound; original retained".into());
        }
        self.simulation = state;
        self.checked = false;
        self.notice = Some(message);
        Ok(report)
    }
}
pub fn prepare(
    a: &mut crate::progress::Attempt,
    l: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    let crate::engine::State::DebianPackages { simulation } = &a.state else {
        return Err(
            "Package practice needs a logical package assignment; original retained".into(),
        );
    };
    if !l.separate_from(simulation) {
        return Err("Package practice needs separate admitted catalogs; original retained".into());
    }
    if let Some(existing) = a
        .workspace
        .as_ref()
        .and_then(|w| w.debian_packages_learning.as_ref())
    {
        if !existing.valid_for(l) {
            return Err("Foreign package practice fixture; original retained".into());
        }
        return Ok(());
    }
    let mut learning = Learning::new(l)?;
    if !new_attempt {
        learning.view = View::Assignment;
    }
    a.workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .debian_packages_learning = Some(learning);
    Ok(())
}

pub fn separate_contexts(practice: &State, assignment: &State) -> bool {
    if !work::admitted(practice) || !work::admitted(assignment) {
        return false;
    }
    let p = practice.catalog();
    let a = assignment.catalog();
    p != a
        && p.archives.keys().all(|id| !a.archives.contains_key(id))
        && p.archives.values().all(|x| {
            a.archives
                .values()
                .all(|y| x.name != y.name && x.files.keys().all(|path| !y.files.contains_key(path)))
        })
        && p.publications
            .keys()
            .all(|id| !a.publications.contains_key(id))
        && p.extra_paths.iter().all(|path| {
            !a.extra_paths.contains(path)
                && a.archives.values().all(|y| !y.files.contains_key(path))
        })
        && a.extra_paths
            .iter()
            .all(|path| p.archives.values().all(|y| !y.files.contains_key(path)))
}
