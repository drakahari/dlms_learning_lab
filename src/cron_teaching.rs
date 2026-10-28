//! Skippable scheduling introductions and isolated ungraded practice.
use crate::{
    cron::{self as core, State},
    cron_workspace::{self as work, Action, Definition, Document, Session},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "core::text::<_,100>")]
    pub title: String,
    #[serde(deserialize_with = "core::text::<_,1600>")]
    pub text: String,
}
pub const MAX_LEARNING: usize = 2 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotatedAction {
    #[serde(deserialize_with = "core::text::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "core::text::<_,1600>")]
    pub explanation: String,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "core::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "work::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "core::text::<_,1600>")]
    pub example_description: String,
    pub definition: Definition,
    #[serde(deserialize_with = "work::sequence::<_,AnnotatedAction,16>")]
    pub example_actions: Vec<AnnotatedAction>,
    #[serde(deserialize_with = "core::text::<_,1600>")]
    pub try_change: String,
    #[serde(deserialize_with = "work::sequence::<_,Action,16>")]
    pub try_actions: Vec<Action>,
    #[serde(deserialize_with = "work::sequence::<_,bool,34>")]
    pub try_rows: Vec<bool>,
    #[serde(deserialize_with = "work::sequence::<_,Action,16>")]
    pub repair: Vec<Action>,
}
fn short(s: &str, cap: usize) -> bool {
    !s.trim().is_empty() && s.len() <= cap && !s.contains('\0')
}
fn run(
    d: &Definition,
    s: &mut State,
    w: &mut Session,
    actions: impl IntoIterator<Item = Action>,
) -> Result<(), String> {
    for action in actions {
        work::apply(d, s, w, &action)?;
    }
    Ok(())
}
impl Lesson {
    pub fn validate_context(&self) -> Result<(), String> {
        self.definition.validate()?;
        if !work::within(self, MAX_LEARNING)
            || !short(&self.prerequisite_note, 1600)
            || !short(&self.example_description, 1600)
            || !short(&self.try_change, 1600)
            || self.concepts.is_empty()
            || self.concepts.len() > 6
            || self
                .concepts
                .iter()
                .any(|c| !short(&c.title, 100) || !short(&c.text, 1600))
            || self.example_actions.is_empty()
            || self.example_actions.len() > 16
            || self
                .example_actions
                .iter()
                .any(|a| !short(&a.title, 160) || !short(&a.explanation, 1600))
            || self.try_actions.is_empty()
            || self.try_actions.len() > 16
            || self.repair.is_empty()
            || self.repair.len() > 16
            || self.try_rows.len() != 1 + self.definition.row_count()
        {
            return Err("Invalid bounded scheduling introduction/practice".into());
        }
        Ok(())
    }
    /// Loader/authoring verification; native frames use the continuation grader.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_context()?;
        let mut s = self.definition.initial.clone();
        let mut w = Session::new(&self.definition)?;
        run(
            &self.definition,
            &mut s,
            &mut w,
            self.example_actions.iter().map(|a| a.action.clone()),
        )?;
        if !w.applied(&self.definition, &s)
            || work::checks_with_session(&self.definition, &s, &w)
                .iter()
                .any(|r| !r.0)
        {
            return Err("Worked scheduling example does not reach its complete outcomes".into());
        }
        run(&self.definition, &mut s, &mut w, self.try_actions.clone())?;
        let actual: Vec<_> = work::checks_with_session(&self.definition, &s, &w)
            .iter()
            .map(|r| r.0)
            .collect();
        if actual != self.try_rows {
            return Err(
                "Scheduling experiment differs from its explicitly reviewed row outcomes".into(),
            );
        }
        run(&self.definition, &mut s, &mut w, self.repair.clone())?;
        if !w.applied(&self.definition, &s)
            || work::checks_with_session(&self.definition, &s, &w)
                .iter()
                .any(|r| !r.0)
        {
            return Err("Scheduling example repair cannot recover its outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, d: &Definition) -> bool {
        self.definition != *d
            && self
                .definition
                .file_paths()
                .all(|p| !d.file_paths().any(|other| other == p))
            && self
                .definition
                .initial
                .users
                .iter()
                .filter(|n| n.as_str() != "root")
                .all(|n| !d.initial.users.contains(n))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub definition: Definition,
    pub simulation: State,
    pub session: Session,
    pub selected_example: usize,
}
impl Learning {
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate_context()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            definition: l.definition.clone(),
            simulation: l.definition.initial.clone(),
            session: Session::new(&l.definition)?,
            selected_example: 0,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.definition.validate().is_ok()
            && self.definition.bound_state(&self.simulation)
            && self.session.valid_for(&self.definition)
            && self
                .session
                .last_run
                .as_ref()
                .is_none_or(|r| r.end <= self.simulation.clock)
            && self.selected_example < 16
            && work::within(self, MAX_LEARNING)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid()
            && self.definition == l.definition
            && self.selected_example < l.example_actions.len()
    }
    pub fn reset(&mut self, l: &Lesson) -> Result<(), String> {
        *self = Self::new(l)?;
        self.view = View::Example;
        Ok(())
    }
    /// Fill/selection never installs a table, changes policy, advances time or runs.
    pub fn fill(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Scheduling example context differs; draft retained".into());
        }
        match &l.example_actions[self.selected_example].action {
            Action::Edit { .. } | Action::Actor { .. } | Action::Select { .. } => work::apply(
                &self.definition,
                &mut self.simulation,
                &mut self.session,
                &l.example_actions[self.selected_example].action,
            ),
            Action::Install { target } | Action::Remove { target } => work::apply(
                &self.definition,
                &mut self.simulation,
                &mut self.session,
                &Action::Select {
                    target: target.clone(),
                },
            ),
            Action::Run { end } => {
                self.session.window_end = end.text();
                Ok(())
            }
        }
    }
    pub fn selected_document(&self) -> &Document {
        &self.session.selected
    }
}
