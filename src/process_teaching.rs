//! Declarative terminal introductions with separate, persisted, ungraded virtual process tables.
use crate::{
    engine::{self, State},
    processes::Table,
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example: Vec<String>,
    pub inputs: String,
    pub try_change: String,
    pub fixture: Table,
    pub outcomes: Vec<crate::processes::Requirement>,
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        if !self.fixture.valid()
            || self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.len() > 100
                    || c.text.trim().is_empty()
                    || c.text.len() > 1200
            })
            || [&self.prerequisite_note, &self.inputs, &self.try_change]
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 1200)
            || self.example.is_empty()
            || self.example.len() > 8
            || self
                .example
                .iter()
                .any(|c| c.trim().is_empty() || c.len() > 4096)
        {
            return Err("Invalid process teaching content".into());
        }
        if self.outcomes.is_empty()
            || self.outcomes.len() > 20
            || self.outcomes.iter().any(|r| !r.valid(&self.fixture))
        {
            return Err("Invalid terminal example outcomes".into());
        }
        let mut table = self.fixture.clone();
        for command in &self.example {
            let output = table
                .execute(command)
                .map_err(|e| format!("Process example cannot run: {e}"))?;
            if output.status != 0 {
                return Err("Reference example command has a nonzero result; record an intentional failure explicitly before teaching it".into());
            }
        }
        if self.outcomes.iter().any(|r| !r.check(&table).0) {
            return Err("Process example does not reach its declared outcome".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub table: Table,
    #[serde(default)]
    pub example_command: usize,
    pub command: String,
    pub transcript: Vec<String>,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            table: lesson.fixture.clone(),
            example_command: 0,
            command: String::new(),
            transcript: Vec::new(),
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.example_command < 8
            && self.table.valid()
            && self.command.len() <= 4096
            && self.transcript.len() <= 40
            && self.transcript.iter().all(|s| s.len() <= 40_000)
    }
    pub fn execute(&mut self) {
        if self.command.len() > 4096 {
            let mut end = 4096;
            while !self.command.is_char_boundary(end) {
                end -= 1;
            }
            self.command.truncate(end);
        }
        let input = std::mem::take(&mut self.command);
        if input.trim().is_empty() {
            return;
        }
        let prompt = self.table.prompt();
        let mut state = State::Processes {
            table: self.table.clone(),
        };
        let output = engine::execute(&mut state, &input).unwrap_or_else(|e| e);
        if let State::Processes { table } = state {
            self.table = table;
        }
        self.transcript.push(format!("{prompt} {input}\n{output}"));
        if self.transcript.len() > 40 {
            self.transcript.remove(0);
        }
    }
}
pub fn prepare(attempt: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    if crate::workspace::is_etherchannel_workspace(attempt)
        || crate::workspace::is_cron_workspace(attempt)
    {
        return;
    }
    if crate::workspace::has_nfs_metadata(attempt) || crate::workspace::has_cron_metadata(attempt) {
        return;
    }
    let workspace = attempt.workspace.as_mut().expect("workspace prepared");
    if workspace.process_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        workspace.process_learning = Some(learning);
    }
}
