//! Separate one-off practice, never assignment credit or host execution.
//! Owns independent example simulation and persisted practice drafts.
use crate::{
    one_off_workspace::{self as work, Action, Assignment, Definition, Session},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotatedAction {
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub explanation: String,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub example_description: String,
    pub definition: Definition,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,AnnotatedAction,16>")]
    pub example_actions: Vec<AnnotatedAction>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub try_change: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,16>")]
    pub try_actions: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,bool,35>")]
    pub try_rows: Vec<bool>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,16>")]
    pub repair: Vec<Action>,
}
fn short(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.contains('\0')
}
fn run(
    d: &Definition,
    a: &mut Assignment,
    s: &mut Session,
    actions: impl IntoIterator<Item = Action>,
) -> Result<(), String> {
    for action in actions {
        let whole = crate::cron_workspace::size(a, work::MAX_ATTEMPT)
            .and_then(|n| n.checked_add(crate::cron_workspace::size(s, work::MAX_ATTEMPT)?))
            .and_then(|n| n.checked_add(256))
            .ok_or("One-off example capacity")?;
        if let Action::Run { end } = action {
            let mut run = work::PendingRun::begin(d, a, s, end, whole)?;
            while !run.done() {
                run.tick(crate::one_off::MAX_TICK)
            }
            *a = run
                .take_result()
                .ok_or("One-off example has no completed candidate")??;
        } else {
            work::apply(d, a, s, &action, whole)?;
        }
    }
    Ok(())
}
impl Lesson {
    /// Raw-admitted standalone teaching codec. Nested new tagged enums stream;
    /// legacy outer engine State/Goal envelopes have their separate raw bounds.
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > work::MAX_CONTEXT {
            return Err("One-off lesson exceeds2MiB before decoding".into());
        }
        let lesson: Self =
            serde_json::from_slice(raw).map_err(|_| "Malformed/excessive one-off lesson")?;
        lesson.validate_context()?;
        Ok(lesson)
    }
    pub fn validate_context(&self) -> Result<(), String> {
        self.definition.validate()?;
        if !crate::cron_workspace::within(self, work::MAX_CONTEXT)
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
            return Err("Invalid bounded one-off introduction/example".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.validate_context()?;
        let mut a = Assignment::new(&self.definition.initial)?;
        let mut s = Session::from_state(&self.definition, &a)?;
        run(
            &self.definition,
            &mut a,
            &mut s,
            self.example_actions.iter().map(|r| r.action.clone()),
        )?;
        if work::checks_with_session(&self.definition, &a, &s)
            .iter()
            .any(|r| !r.0)
        {
            return Err("One-off worked example does not reach its complete behavior".into());
        }
        run(&self.definition, &mut a, &mut s, self.try_actions.clone())?;
        if work::checks_with_session(&self.definition, &a, &s)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>()
            != self.try_rows
        {
            return Err("One-off practice change differs from authored row vector".into());
        }
        run(&self.definition, &mut a, &mut s, self.repair.clone())?;
        if work::checks_with_session(&self.definition, &a, &s)
            .iter()
            .any(|r| !r.0)
        {
            return Err("One-off practice repair cannot restore behavior".into());
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
                .filter(|u| u.as_str() != "root")
                .all(|u| !d.initial.users.contains(u))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub definition: Definition,
    pub assignment: Assignment,
    pub session: Session,
    pub selected_example: usize,
}
impl Learning {
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate_context()?;
        if work::budget::initial_learning_size(l).is_none() {
            return Err("Separate one-off example exceeds its saved bound before copy".into());
        }
        let assignment = Assignment::new(&l.definition.initial)?;
        let session = Session::from_state(&l.definition, &assignment)?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            definition: l.definition.clone(),
            assignment,
            session,
            selected_example: 0,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.definition.validate().is_ok()
            && self.definition.bound(&self.assignment)
            && self.session.valid_for(&self.definition)
            && crate::cron_workspace::within(self, work::MAX_ATTEMPT)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid()
            && self.definition == l.definition
            && self.selected_example < l.example_actions.len()
    }
}
