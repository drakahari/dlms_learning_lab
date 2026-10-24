//! Isolated typed examples for the virtual manager. No assignment/progress access.
use crate::{
    script_teaching::{Concept, View},
    unit_manager::{self as core, State},
    unit_manager_workspace::{self as workspace, Definition, Draft, EditAction, Requirement},
};
use serde::{Deserialize, Deserializer, Serialize, de::Visitor};
use std::fmt;

pub const MAX_LEARNING_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_LESSON_BYTES: usize = 8 * 1024 * 1024;

fn concepts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    fn title<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        bounded_text::<D, 100>(d)
    }
    fn body<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        bounded_text::<D, 1200>(d)
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        #[serde(deserialize_with = "title")]
        title: String,
        #[serde(deserialize_with = "body")]
        text: String,
    }
    workspace::sequence::<D, Input, 6>(d).map(|v| {
        v.into_iter()
            .map(|c| Concept {
                title: c.title,
                text: c.text,
            })
            .collect()
    })
}
fn annotations<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<AnnotatedAction>, D::Error> {
    workspace::sequence::<D, AnnotatedAction, 8>(d)
}
fn transcript<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Message>, D::Error> {
    workspace::sequence::<D, Message, 100>(d)
}
fn bounded_text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                Err(E::custom("Virtual manager teaching text limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn prose<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    bounded_text::<D, 1600>(d)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotatedAction {
    #[serde(deserialize_with = "prose")]
    pub title: String,
    #[serde(deserialize_with = "prose")]
    pub explanation: String,
    pub action: EditAction,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "prose")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "concepts")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "prose")]
    pub example_description: String,
    pub definition: Definition,
    #[serde(deserialize_with = "workspace::actions")]
    pub solution: Vec<EditAction>,
    #[serde(deserialize_with = "annotations")]
    pub example_actions: Vec<AnnotatedAction>,
    #[serde(deserialize_with = "prose")]
    pub try_change: String,
    #[serde(deserialize_with = "workspace::actions")]
    pub try_actions: Vec<EditAction>,
    #[serde(deserialize_with = "workspace::requirements")]
    pub try_outcomes: Vec<Requirement>,
    #[serde(deserialize_with = "workspace::actions")]
    pub repair: Vec<EditAction>,
}
fn short(s: &str, max: usize) -> bool {
    s.len() <= max && !s.trim().is_empty()
}
fn run(state: &mut State, initial: &State, actions: &[EditAction]) -> Result<(), String> {
    if actions.is_empty()
        || actions.len() > workspace::MAX_ACTIONS
        || actions
            .iter()
            .any(|action| !workspace::valid_action(initial, action))
    {
        return Err("Invalid bounded example action sequence".into());
    }
    for action in actions {
        workspace::apply(state, initial, action)?;
    }
    Ok(())
}
impl Lesson {
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_LESSON_BYTES {
            return Err(
                "Virtual manager lesson JSON exceeds 8 MiB; outer pack limits may be smaller"
                    .into(),
            );
        }
        let lesson: Self = serde_json::from_str(text)
            .map_err(|_| "Invalid bounded virtual manager lesson data".to_owned())?;
        lesson.validate()?;
        Ok(lesson)
    }
    /// Cheap borrowed validation before every replay or retained-state/draft clone.
    /// Do not serialize metadata to measure it, or execute actions to validate bounds.
    pub fn validate_bounds(&self) -> Result<(), String> {
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self
                .concepts
                .iter()
                .any(|c| !short(&c.title, 100) || !short(&c.text, 1200))
            || [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .any(|s| !short(s, 1600))
            || self.solution.is_empty()
            || self.solution.len() > workspace::MAX_ACTIONS
            || self.try_actions.is_empty()
            || self.try_actions.len() > workspace::MAX_ACTIONS
            || self.repair.is_empty()
            || self.repair.len() > workspace::MAX_ACTIONS
            || self.example_actions.is_empty()
            || self.example_actions.len() > 8
            || self
                .example_actions
                .iter()
                .any(|a| !short(&a.title, 100) || !short(&a.explanation, 800))
            || self.try_outcomes.is_empty()
            || self.try_outcomes.len() > workspace::MAX_REQUIREMENTS
        {
            return Err(
                "Invalid virtual-manager introduction, example or experiment metadata".into(),
            );
        }
        self.definition.validate()?;
        let initial = &self.definition.initial;
        if !workspace::valid_requirements(&self.try_outcomes, initial, false)
            || self
                .solution
                .iter()
                .chain(&self.try_actions)
                .chain(&self.repair)
                .any(|a| !workspace::valid_action(initial, a))
            || self
                .example_actions
                .iter()
                .any(|a| !workspace::valid_action(initial, &a.action))
        {
            return Err("Invalid bounded example actions or experiment outcomes".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.validate_bounds()?;
        let initial = &self.definition.initial;
        let mut worked = initial.clone();
        run(&mut worked, initial, &self.solution)?;
        if !workspace::solved(&self.definition, &worked, &Draft::new(&worked)?) {
            return Err("Virtual-manager worked example does not meet its stated outcomes".into());
        }
        // Annotated actions are a genuinely runnable sequence, not decorative tips.
        let mut annotated = initial.clone();
        for a in &self.example_actions {
            workspace::apply(&mut annotated, initial, &a.action)?;
        }
        if !workspace::solved(&self.definition, &annotated, &Draft::new(&annotated)?) {
            return Err(
                "Annotated example actions do not teach a working route to the task".into(),
            );
        }
        let mut changed = worked.clone();
        run(&mut changed, initial, &self.try_actions)?;
        if changed == worked
            || self
                .try_outcomes
                .iter()
                .any(|r| !workspace::check_requirement(&changed, r).0)
            || workspace::solved(&self.definition, &changed, &Draft::new(&changed)?)
        {
            return Err("Virtual-manager experiment must make the promised meaningful change, with a retryable failed objective".into());
        }
        run(&mut changed, initial, &self.repair)?;
        if !workspace::solved(&self.definition, &changed, &Draft::new(&changed)?) {
            return Err("Virtual-manager experiment repair does not restore every outcome".into());
        }
        Ok(())
    }
    pub fn distinct_from(&self, assignment: &State) -> bool {
        self.validate_bounds().is_ok()
            && !self.definition.initial.same_context(assignment)
            && self.definition.initial.inventory().fixture != assignment.inventory().fixture
            && self.definition.initial.inventory().units != assignment.inventory().units
            && self.definition.initial.inventory().config_paths
                != assignment.inventory().config_paths
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub failed: bool,
    #[serde(deserialize_with = "core::text256")]
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    /// Exact authored practice origin, never a grading credential.
    pub fixture: State,
    pub state: State,
    pub draft: Draft,
    pub selected_action: usize,
    #[serde(deserialize_with = "transcript")]
    pub transcript: Vec<Message>,
    pub checked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<crate::unit_manager_terminal::Session>,
}
impl Learning {
    pub fn from_json(text: &str, lesson: &Lesson) -> Result<Self, String> {
        lesson.validate_bounds()?;
        if text.len() > MAX_LEARNING_BYTES {
            return Err("Virtual manager practice JSON exceeds 8 MiB".into());
        }
        let learning: Self = serde_json::from_str(text)
            .map_err(|_| "Invalid bounded virtual manager practice data".to_owned())?;
        if !learning.valid_for(lesson) {
            return Err("Virtual manager practice origin differs from this lesson".into());
        }
        Ok(learning)
    }
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate_bounds()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            fixture: lesson.definition.initial.clone(),
            state: lesson.definition.initial.clone(),
            draft: Draft::new(&lesson.definition.initial)?,
            selected_action: 0,
            transcript: vec![],
            checked: false,
            terminal: None,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.terminal.as_ref().is_none_or(|s| s.valid())
            && self.transcript.len() <= 100
            && self.selected_action < 8
            && self.fixture.validate().is_ok()
            && self.state.same_context(&self.fixture)
            && self.draft.same_shape(&self.state)
            && self.transcript.iter().all(|m| m.text.len() <= 256)
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && lesson.validate_bounds().is_ok()
            && self.fixture == lesson.definition.initial
            && self.selected_action < lesson.example_actions.len()
    }
    /// Fill only the example editor with worked saved source. No action is run in
    /// the learner's example State, and no assignment object is accepted here.
    pub fn fill_worked(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("Example fixture/draft does not match this lesson".into());
        }
        let mut worked = lesson.definition.initial.clone();
        run(&mut worked, &lesson.definition.initial, &lesson.solution)?;
        self.draft.documents.clone_from(worked.documents());
        self.draft.configuration.clone_from(worked.configuration());
        self.checked = false;
        Ok(())
    }
    pub fn execute(
        &mut self,
        lesson: &Lesson,
        action: &EditAction,
    ) -> Result<Option<core::Report>, String> {
        if !self.valid_for(lesson) || !workspace::valid_action(&lesson.definition.initial, action) {
            return Err("Example fixture/draft/action does not match this bounded lesson".into());
        }
        let before = self.state.clone();
        let result = workspace::apply(&mut self.state, &lesson.definition.initial, action);
        // Preserve unsaved example work; synchronize only a previously clean draft.
        if self.draft.matches(&before) {
            self.draft.documents.clone_from(self.state.documents());
            self.draft
                .configuration
                .clone_from(self.state.configuration());
        }
        let text = match &result {
            Ok(Some(r)) => format!(
                "Simulator action completed with {} job results; inspect current state separately.",
                r.jobs.len()
            ),
            Ok(None) => {
                "Example editor text saved; manager/runtime/enablement remain separate.".into()
            }
            Err(e) => e.clone(),
        };
        let mut end = text.len().min(256);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        self.transcript.push(Message {
            failed: result.is_err(),
            text: text[..end].into(),
        });
        if self.transcript.len() > 100 {
            self.transcript.remove(0);
        }
        result
    }
    pub fn save_draft(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("Example fixture/draft does not match this bounded lesson".into());
        }
        let action = EditAction::Save {
            draft: self.draft.clone(),
        };
        self.execute(lesson, &action).map(|_| ())
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![];
        }
        if !self.valid_for(lesson) {
            return vec![(false, "Example state differs from its fixed lesson".into())];
        }
        workspace::checks(&lesson.definition, &self.state, &self.draft)
    }
}

/// Prepare optional metadata without changing another kind or replacing saved work.
pub fn prepare(
    attempt: &mut crate::progress::Attempt,
    lesson: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    let crate::engine::State::UnitManager { simulation } = &attempt.state else {
        return Err("Virtual unit practice belongs only to its own workspace".into());
    };
    lesson.validate_bounds()?;
    if !lesson.distinct_from(simulation) {
        return Err("Virtual unit assignment/example origins are not separate".into());
    }
    let workspace = attempt
        .workspace
        .as_mut()
        .ok_or("Prepare the workspace before its unit-manager example")?;
    if let Some(learning) = &workspace.unit_manager_learning {
        if !learning.valid_for(lesson) {
            return Err("Saved unit-manager practice origin differs; work retained".into());
        }
    } else {
        let mut learning = Learning::new(lesson)?;
        if !new_attempt {
            learning.view = View::Assignment;
        }
        workspace.unit_manager_learning = Some(learning);
    }
    Ok(())
}
