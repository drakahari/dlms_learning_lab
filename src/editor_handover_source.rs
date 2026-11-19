//! Source-bound composition, with declarative authoring events only.
use crate::{
    editor_handover::{Definition, Session},
    engine::{Goal, State},
    modal_workspace,
    pack::Scenario,
    progress::Attempt,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Setup {
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        source: String,
    },
    RunSetup {},
    OpenEditor {},
    File {
        event: modal_workspace::Event,
    },
    Check {},
}
impl Session {
    /// Authoring/reachability path. The native GUI uses a cooperative Job instead.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.context(self)?;
        if input.len() > 16384 {
            return Err("Editor-handover authoring event exceeds its bound.".into());
        }
        let raw = input.strip_prefix("editor-handover ").ok_or("Use the native setup script and virtual editor. editor-handover JSON is a simulator authoring event, not a Linux command.")?;
        let event: Event = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid declarative handover event: {e}"))?;
        match event {
            Event::Setup { source } => self.edit_setup(d, false, source)?,
            Event::RunSetup {} => {
                let source = self.setup.assignment.source.clone();
                self.begin_run(d, false)?;
                let record = crate::environment_workspace::run(&d.setup.assignment, &source)?;
                self.store_setup(d, false, record)?;
            }
            Event::OpenEditor {} => self.open_editor(d, false)?,
            Event::File { event } => self.apply_file(d, false, &event)?,
            Event::Check {} => self.apply_file(d, false, &modal_workspace::Event::Check {})?,
        }
        Ok("Simulated editor handover action applied.".into())
    }
}
fn admitted(s: &Scenario) -> Result<&Definition, String> {
    let (State::EditorHandover { simulation }, Goal::EditorHandover { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Editor setup and virtual-file task do not match.".into());
    };
    if s.track != "LPIC-1"
        || s.id != definition.id
        || s.version != definition.version
        || **simulation != definition.new_session()?
    {
        return Err(
            "Match the supplied editor-handover lesson, version and initial workspaces.".into(),
        );
    }
    Ok(definition)
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let d = admitted(s)?;
    d.validate_example()?;
    Ok(d)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = admitted(s)?;
    let State::EditorHandover { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved editor-handover progress metadata is incompatible.".into());
    }
    d.context(simulation)
}
