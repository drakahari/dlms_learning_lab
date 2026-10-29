//! Isolated native configuration practice. Pack adapters only prepare example drafts.
use crate::{
    ipv4_routing::{Requirement, Topology},
    ipv4_routing_workspace::{self, Draft},
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(default)]
    pub start_with_routes: bool,
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
            || !crate::ipv4_routing::valid_requirements(&self.outcomes, &self.fixture)
        {
            return Err("Invalid IPv4 teaching content".into());
        }
        let mut candidate = self.fixture.clone();
        for action in &self.example_solution {
            ipv4_routing_workspace::adapter(&mut candidate, action)?;
        }
        if self.outcomes.iter().any(|r| !r.check(&candidate).0) {
            return Err("IPv4 example does not reach its declared outcomes".into());
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
            draft: {
                let mut draft = Draft::new(&lesson.fixture);
                draft.show_routes = lesson.start_with_routes;
                if let Some(device) = preferred_device(&lesson.outcomes) {
                    draft.selected_device = device;
                }
                draft
            },
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
            ipv4_routing_workspace::adapter(&mut candidate, action)?;
        }
        let selected = self.draft.selected_device.clone();
        let show_routes = self.draft.show_routes;
        self.draft = Draft::new(&candidate);
        if self.draft.devices.contains_key(&selected) {
            self.draft.selected_device = selected;
        }
        self.draft.show_routes = show_routes;
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
    let workspace = attempt.workspace.as_mut().expect("workspace prepared");
    if new_attempt && let Some(draft) = &mut workspace.routing_v4_draft {
        draft.show_routes = lesson.start_with_routes;
    }
    if workspace.routing_v4_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        workspace.routing_v4_learning = Some(learning);
    }
}

/// Select the first device named by the meaningful outcomes, without a correctness hint.
pub fn preferred_device(requirements: &[Requirement]) -> Option<String> {
    requirements.first().map(|r| match r {
        Requirement::AddressIn { device, .. }
        | Requirement::PrivateSubnet { device, .. }
        | Requirement::SameSubnet {
            left_device: device,
            ..
        }
        | Requirement::KeepAddress { device, .. }
        | Requirement::Forwarding { device, .. }
        | Requirement::StaticRouteFor { device, .. } => device.clone(),
        Requirement::Reachable { from, .. } => from.clone(),
    })
}
