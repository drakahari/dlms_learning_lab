//! Isolated DNS introductions, editable examples and current outcome checks.
use crate::{
    dns::Server,
    dns_workspace::{self, Draft, Requirement},
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: Server,
    pub example_solution: Vec<String>,
    pub outcomes: Vec<Requirement>,
    pub try_change: String,
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
            || self.example_solution.len() > 16
            || self
                .example_solution
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > dns_workspace::MAX_TEXT + 64)
            || !dns_workspace::valid_requirements(&self.outcomes, &self.fixture)
        {
            return Err("Invalid DNS teaching content".into());
        }
        let mut candidate = self.fixture.clone();
        for action in &self.example_solution {
            dns_workspace::adapter(&mut candidate, action)?;
        }
        if self.outcomes.iter().any(|r| !r.check(&candidate).0) {
            return Err("DNS example cannot reach its declared outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub server: Server,
    pub draft: Draft,
    pub checked: bool,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            server: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture),
            checked: false,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1 && self.server.validate().is_ok() && self.draft.valid()
    }
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        let mut candidate = lesson.fixture.clone();
        for action in &lesson.example_solution {
            dns_workspace::adapter(&mut candidate, action)?;
        }
        self.draft = Draft::new(&candidate);
        self.checked = false;
        Ok(())
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![];
        }
        let mut checks = lesson
            .outcomes
            .iter()
            .map(|r| r.check(&self.server))
            .collect::<Vec<_>>();
        if !self.draft.matches(&self.server) {
            checks.insert(0,(false,"The editor has unsaved changes. Save the zone before checking the applied results.".into()));
        }
        checks
    }
}
pub fn prepare(attempt: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    if matches!(attempt.state, crate::engine::State::Lvm { .. })
        || crate::workspace::is_physical_workspace(attempt)
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
    let w = attempt.workspace.as_mut().expect("workspace prepared");
    if w.dns_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        w.dns_learning = Some(learning);
    }
}
