//! Strict authored-source binding. JSON actions are declarative authoring data,
//! not IOS commands, shell scripts or learner-host execution.
use crate::{
    engine::{Goal, State},
    network_transfer_workspace::{Definition, Draft, Session},
    pack::Scenario,
    progress::Attempt,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Edit { draft: Draft },
    Transfer {},
    Check {},
}
pub const PREFIX: &str = "file-transfer ";
/// Bound escaped JSON without excluding a bounded native ACL document.
pub const MAX_ACTION: usize = 6 * crate::acl_editor::MAX_DRAFT_TOTAL + 4096;
pub const HELP: &str = "Use the native supplied server, direction and virtual-file controls; Transfer file performs one bounded simulated copy, then Check work evaluates every sample's actual files and current policy. The optional router document uses the documented ACL subset. No FTP/TFTP client, shell, real target or host file is executed. file-transfer JSON is authoring data, not a network-device command.";
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.context(self)?;
        if input.len() > MAX_ACTION {
            return Err("Transfer authoring action exceeds its bounded document size.".into());
        }
        let raw=input.strip_prefix(PREFIX).ok_or("Use the native virtual-file transfer workspace. file-transfer JSON is a simulator authoring action, not an IOS or Linux command.")?;
        let event: Event = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid declarative transfer action: {e}"))?;
        match event {
            Event::Edit { draft } => {
                self.assignment.edit(&d.assignment, draft)?;
                Ok("Transfer draft updated; no file was copied.".into())
            }
            Event::Transfer {} => self.assignment.transfer(&d.assignment).map(|r| r.detail),
            Event::Check {} => {
                self.assignment.check(&d.assignment)?;
                Ok(
                    "Actual virtual files and current policy checked across the supplied cases."
                        .into(),
                )
            }
        }
    }
}
fn admitted(s: &Scenario) -> Result<&Definition, String> {
    let (State::NetworkTransfer { simulation }, Goal::NetworkTransfer { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Transfer workspace and supplied task do not match.".into());
    };
    if s.track != "CCNA"
        || s.id != definition.id
        || s.version != definition.version
        || **simulation != definition.new_session()?
    {
        return Err(
            "Match the supplied transfer lesson, version and independent initial workspaces."
                .into(),
        );
    }
    Ok(definition)
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let d = admitted(s)?;
    d.validate()?;
    Ok(d)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = admitted(s)?;
    let State::NetworkTransfer { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err(
            "Saved transfer progress metadata is incompatible; original data retained.".into(),
        );
    }
    d.context(simulation)
}
