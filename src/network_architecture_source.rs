//! Declarative developer actions and immutable source binding; never device commands.
use crate::{
    engine::{Goal, State},
    network_architecture::Configuration,
    network_architecture_workspace::{Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
use serde::{Deserialize, Serialize};
pub const PREFIX: &str = "campus-layout ";
pub const MAX_ACTION: usize = 16 * 1024;
pub const HELP: &str = "Use the supplied cable endpoints and stated-outage selector, then Check answers. Structural paths assume already-configured routed adjacencies; they do not prove IP forwarding, STP, convergence or throughput. campus-layout JSON is developer authoring data, not an IOS command. No real device, packet, host cable or network is changed.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Edit { draft: Configuration },
    Check {},
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.context(self)?;
        if input.len() > MAX_ACTION {
            return Err("Campus authoring action exceeds its size bound.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native cable table; no host or device command is executed.")?;
        let e: Event =
            serde_json::from_str(raw).map_err(|e| format!("Invalid campus authoring data: {e}"))?;
        match e {
            Event::Edit { draft } => {
                self.assignment.edit(&d.assignment, draft)?;
                Ok(
                    "Cable endpoints changed; previous checks cleared if the layout changed."
                        .into(),
                )
            }
            Event::Check {} => {
                self.assignment.check(&d.assignment)?;
                Ok("Current cable layout checked against every stated outage and role requirement.".into())
            }
        }
    }
}
fn admitted(s: &Scenario) -> Result<&Definition, String> {
    let (State::CampusArchitecture { simulation }, Goal::CampusArchitecture { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Campus source and workspace do not match.".into());
    };
    if s.track != "CCNA"
        || s.id != definition.id
        || s.version != definition.version
        || **simulation != definition.new_session()?
    {
        return Err(
            "Match the authored campus lesson, version and separate initial workspaces.".into(),
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
    let State::CampusArchitecture { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved campus progress metadata differs; original data retained.".into());
    }
    d.context(simulation)
}
