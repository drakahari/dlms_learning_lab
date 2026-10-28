//! Separate introduction/example/assignment state for native mount activities.
use crate::{
    pack::CommandExample,
    script_teaching::{Concept, View},
    storage::State,
    storage_workspace::{self, Action, Draft, Requirement},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: State,
    pub example_solution: Vec<String>,
    pub example_commands: Vec<CommandExample>,
    pub outcomes: Vec<Requirement>,
    pub try_change: String,
    pub try_actions: Vec<String>,
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.len() > 100
                    || c.text.trim().is_empty()
                    || c.text.len() > 1200
            })
            || [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 1600)
            || self.example_solution.is_empty()
            || self.example_solution.len() > 24
            || self
                .example_solution
                .iter()
                .any(|s| s.len() > storage_workspace::MAX_DRAFT_BYTES + 256)
            || self.example_commands.is_empty()
            || self.example_commands.len() > 8
            || self.example_commands.iter().any(|c| {
                c.command.trim().is_empty()
                    || c.command.len() > 4096
                    || c.explanation.trim().is_empty()
                    || c.explanation.len() > 800
                    || c.simulator_only
            })
            || self.try_actions.is_empty()
            || self.try_actions.len() > 8
            || self
                .try_actions
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > storage_workspace::MAX_DRAFT_BYTES + 256)
            || !storage_workspace::valid_requirements(&self.outcomes, self.fixture.inventory())
        {
            return Err("Invalid storage introduction or separate practice metadata".into());
        }
        let mut repaired = self.fixture.clone();
        for text in &self.example_solution {
            storage_workspace::adapter(&mut repaired, text)?;
        }
        if storage_workspace::checks(&repaired, &self.outcomes)
            .iter()
            .any(|c| !c.0)
        {
            return Err("Storage example cannot reach every declared outcome".into());
        }
        // Every visible example command really belongs to the bounded terminal.
        for example in &self.example_commands {
            let mut candidate = repaired.clone();
            storage_workspace::command(&mut candidate, &example.command)?;
        }
        let before = repaired.clone();
        for command in &self.try_actions {
            storage_workspace::adapter(&mut repaired, command)?;
        }
        if repaired == before {
            return Err("Storage practice change must change simulated state, not merely display another message".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub state: State,
    pub draft: Draft,
    pub checked: bool,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            state: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture),
            checked: false,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1 && self.draft.valid()
    }
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        let mut repaired = lesson.fixture.clone();
        for action in &lesson.example_solution {
            storage_workspace::adapter(&mut repaired, action)?;
        }
        // Project the worked saved file only. No mount, save, command, check or
        // assignment execution happens in the learner's practice state.
        self.draft.text = repaired.configuration().into();
        self.draft.notice=Some("Worked configuration filled in this example only. Save it, inspect the live mounts and run the relevant example commands; nothing ran automatically.".into());
        self.checked = false;
        Ok(())
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![];
        }
        let mut checks = storage_workspace::checks(&self.state, &lesson.outcomes);
        if !self.draft.matches(&self.state) {
            checks.insert(0,(false,"The example has unsaved editor changes. Save them before checking the live and fresh-session results.".into()));
        }
        checks
    }
}

/// Kept explicit for authoring tools; these actions are data, never shell code.
pub fn save_action(text: String) -> String {
    format!(
        "storage {}",
        serde_json::to_string(&Action::Save { text }).expect("serializable storage action")
    )
}

pub fn prepare(attempt: &mut crate::progress::Attempt, lesson: &Lesson, new_attempt: bool) {
    if crate::workspace::is_etherchannel_workspace(attempt)
        || crate::workspace::is_cron_workspace(attempt)
    {
        return;
    }
    if crate::workspace::has_nfs_metadata(attempt) || crate::workspace::has_cron_metadata(attempt) {
        return;
    }
    let w = attempt.workspace.as_mut().expect("workspace prepared");
    if w.storage_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        w.storage_learning = Some(learning);
    }
}
