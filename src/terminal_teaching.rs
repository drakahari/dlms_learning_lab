//! Declarative terminal introductions with separate, persisted, ungraded virtual trees.
use crate::{
    engine::{self, State},
    filesystem::Tree,
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
    pub fixture: Tree,
    pub outcomes: Vec<crate::filesystem::Requirement>,
}
impl Lesson {
    pub fn command_source(&self)->Result<Option<crate::filesystem::discovery_policy::Reference<'_>>,String> {
        let reference=crate::filesystem::discovery_policy::for_current(&self.outcomes,&self.fixture)?;
        if reference.is_some_and(|r|!r.valid_for(&self.fixture)) {
            return Err("Practice command-location reference differs from its original fixture".into());
        }
        Ok(reference)
    }
    pub fn validate(&self) -> Result<(), String> {
        let reference = self.command_source()?;
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
            return Err("Invalid terminal teaching content".into());
        }
        if self.outcomes.is_empty()
            || self.outcomes.len() > 20
            || self.outcomes.iter().any(|r| !r.valid_for(&self.fixture))
        {
            return Err("Invalid terminal example outcomes".into());
        }
        let mut tree = self.fixture.clone();
        for command in &self.example {
            (match reference {Some(r)=>r.execute(&mut tree,command),None=>tree.execute_in_context(&self.fixture,command)})
                .map_err(|e| format!("Terminal example cannot run: {e}"))?;
        }
        if self.outcomes.iter().any(|r| !r.check(&tree).0) {
            return Err("Terminal example does not reach its declared outcome".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub tree: Tree,
    #[serde(default)]
    pub example_command: usize,
    pub command: String,
    pub transcript: Vec<String>,
}
impl Learning {
    /// Source pairing is mandatory only for the opt-in diagnostic workspace.
    /// Existing None lessons retain their previous validity and execution path.
    pub fn valid_for_source(&self,lesson:&Lesson)->bool {
        let Ok(reference)=lesson.command_source()else{return false;};
        self.valid() && self.example_command<lesson.example.len()
            && match reference {
                None=>lesson.fixture.command_location.is_none()&&self.tree.command_location.is_none(),
                Some(r)=>r.bound(&self.tree).is_ok(),
            }
    }
    pub fn execute_for(&mut self,lesson:&Lesson)->Result<(),String> {
        if !self.valid_for_source(lesson){return Err("Practice source differs; your draft is retained".into());}
        let Some(reference)=lesson.command_source()?else{self.execute();return Ok(());};
        let input=std::mem::take(&mut self.command);
        if input.trim().is_empty(){return Ok(());}
        let output=reference.execute(&mut self.tree,&input).unwrap_or_else(|e|e);
        self.transcript.push(format!("alex $ {input}\n{output}"));
        if self.transcript.len()>40{self.transcript.remove(0);}
        Ok(())
    }
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            tree: lesson.fixture.clone(),
            example_command: 0,
            command: String::new(),
            transcript: Vec::new(),
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.example_command < 8
            && self.tree.valid()
            && self.command.len() <= 4096
            && self.transcript.len() <= 40
            && self.transcript.iter().all(|s| s.len() <= 40_000)
    }
    pub fn execute(&mut self) {
        // A source-free call cannot consume a draft from a source-bound activity.
        if self.tree.command_location.is_some() { return; }
        let input = std::mem::take(&mut self.command);
        if input.trim().is_empty() {
            return;
        }
        let mut state = State::Filesystem {
            tree: self.tree.clone(),
        };
        let output = engine::execute(&mut state, &input).unwrap_or_else(|e| e);
        if let State::Filesystem { tree } = state {
            self.tree = tree;
        }
        self.transcript.push(format!("alex $ {input}\n{output}"));
        if self.transcript.len() > 40 {
            self.transcript.remove(0);
        }
    }
}
pub fn prepare(attempt: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    if matches!(
        attempt.state,
        crate::engine::State::PartitionFs { .. }
            | crate::engine::State::Lvm { .. }
            | crate::engine::State::Ntp { .. }
            | crate::engine::State::Hsrp { .. }
            | crate::engine::State::Syslog { .. }
    ) || crate::workspace::is_physical_workspace(attempt)
    {
        return;
    }
    if crate::workspace::is_one_off_workspace(attempt) {
        return;
    }

    if crate::workspace::is_file_backup_workspace(attempt) {
        return;
    }
    if crate::workspace::is_etherchannel_workspace(attempt)
        || crate::workspace::is_cron_workspace(attempt)
    {
        return;
    }
    if crate::workspace::has_nfs_metadata(attempt) || crate::workspace::has_cron_metadata(attempt) {
        return;
    }
    let workspace = attempt.workspace.as_mut().expect("workspace prepared");
    if workspace.terminal_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        workspace.terminal_learning = Some(learning);
    }
}
