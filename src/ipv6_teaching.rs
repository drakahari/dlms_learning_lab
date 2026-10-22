//! Isolated native configuration practice. Pack adapters only prepare example drafts.
use crate::{
    ipv6::{Requirement, Topology},
    ipv6_workspace::{self, Draft},
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
    pub fixture: Topology,
    pub example_solution: Vec<String>,
    pub outcomes: Vec<Requirement>,
    pub try_change: String,
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
                .any(|s| s.trim().is_empty() || s.len() > 4096)
            || self.outcomes.is_empty()
            || self.outcomes.len() > 20
            || self.outcomes.iter().any(|r| !r.valid_for(&self.fixture))
        {
            return Err("Invalid IPv6 teaching content".into());
        }
        let mut candidate = self.fixture.clone();
        for action in &self.example_solution {
            ipv6_workspace::adapter(&mut candidate, action)?;
        }
        if self.outcomes.iter().any(|r| !r.check(&candidate).0) {
            return Err("IPv6 example does not reach its declared outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub topology: Topology,
    pub draft: Draft,
    pub checked: bool,
    pub notice: Option<String>,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            topology: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture),
            checked: false,
            notice: None,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.topology.valid()
            && self.draft.same_shape(&self.topology)
            && self.notice.as_ref().is_none_or(|s| s.len() <= 16000)
    }
    /// Explicitly replaces only practice drafts, leaving its applied and assignment state intact.
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        let mut candidate = lesson.fixture.clone();
        for action in &lesson.example_solution {
            ipv6_workspace::adapter(&mut candidate, action)?;
        }
        self.draft = Draft::new(&candidate);
        self.checked = false;
        self.notice = None;
        Ok(())
    }
    pub fn apply(&mut self) -> Result<(), String> {
        let result = self.draft.apply(&mut self.topology);
        self.checked = result.is_ok();
        self.notice = result.as_ref().err().cloned();
        result
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.checked || !self.draft.matches(&self.topology) {
            return Vec::new();
        }
        lesson
            .outcomes
            .iter()
            .map(|r| r.check(&self.topology))
            .collect()
    }
}
pub fn prepare(attempt: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    let workspace = attempt.workspace.as_mut().expect("workspace prepared");
    if workspace.ipv6_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        workspace.ipv6_learning = Some(learning);
    }
}
