//! Source binding for represented hosts and routing contexts, never host execution.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    virtual_isolation_workspace::{Definition, Session},
};
use serde::{Deserialize, Serialize};
pub const PREFIX: &str = "virtual-net ";
pub const MAX_ACTION: usize = 24 * 1024;
pub const HELP: &str = "Use the native placement, interface-context and experiment controls. Hosts, guest images, kernels, links and endpoint addresses are supplied simulation facts. Nothing boots or runs on your computer: no real VM, container runtime, host shell or network is used. The workload budget is an exercise planning policy, not a universal hypervisor rule. VRF practice uses one router and connected routes only; no route leaking, MPLS, BGP or full IOS command syntax.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Edit {
        configuration: crate::virtual_isolation_workspace::Draft,
    },
    Trial {
        selection: crate::virtual_isolation_workspace::Selection,
    },
    Check {},
}

impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.context(self)?;
        if input.len() > MAX_ACTION {
            return Err("Represented-network authoring action exceeds its size bound.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native represented-network workspace; no host command is executed.")?;
        let e: Event = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid represented-network authoring data: {e}"))?;
        let a = &mut self.assignment;
        let t = &d.assignment;
        match e {
            Event::Edit { configuration } => a.edit(t, configuration)?,
            Event::Trial { selection } => {
                a.choose(t, selection)?;
                a.run(t)?;
            }
            Event::Check {} => a.check(t)?,
        }
        Ok(
            "Simulated represented-network workspace updated; check the current task outcomes."
                .into(),
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.version == 0
            || self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 128 * 1024
        {
            return Err(
                "Unknown represented-network workspace version/identity or saved work exceeds its bound."
                    .into(),
            );
        }

        Ok(())
    }
    pub fn checks(&self, d: &Definition, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = d.context(self) {
            return vec![(false, e)];
        }
        let (a, t) = if practice {
            (&self.practice, &d.example)
        } else {
            (&self.assignment, &d.assignment)
        };
        if !a.checked {
            return vec![(
                false,
                "Unchecked · check the full represented-network task after making your changes."
                    .into(),
            )];
        }
        t.checks(&a.configuration)
            .into_iter()
            .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
            .collect()
    }
}
fn admitted(s: &Scenario) -> Result<&Definition, String> {
    let (State::VirtualIsolation { simulation }, Goal::VirtualIsolation { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Represented-network source and workspace do not match.".into());
    };
    if s.track != "CCNA"
        || s.id != definition.id
        || s.version != definition.version
        || **simulation != definition.new_session()?
    {
        return Err(
            "Match the authored represented-network lesson, version and separate initial workspaces.".into(),
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
    let State::VirtualIsolation { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved represented-network metadata differs; original data retained.".into());
    }
    d.context(simulation)
}
