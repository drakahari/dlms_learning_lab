//! Declarative development fixtures and source binding, never host Samba commands.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    samba_share_workspace::{Definition, Session},
};
use serde::{Deserialize, Serialize};
pub const PREFIX: &str = "smb-share ";
pub const MAX_ACTION: usize = 20 * 1024;
pub const HELP: &str = "Use the native configuration editor, supplied account controls and client trials. The virtual filesystem terminal supports ls, id, cat, pwd, cd, chmod and chgrp only. Account/Apply controls are simulator administration, not Linux commands. No host account, file, process, service or network is changed. SMB cryptography, AD membership, printers, CIFS mounting and cached open handles are outside this standalone subset.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Edit { text: String },
    Save {},
    Publish {},
    Account { user: String, enabled: bool },
    Command { command: String },
    Check {},
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.context(self)?;
        if input.len() > MAX_ACTION {
            return Err("Share authoring action exceeds its size bound.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native sharing workspace; no host command is executed.")?;
        let e: Event =
            serde_json::from_str(raw).map_err(|e| format!("Invalid share authoring data: {e}"))?;
        let a = &mut self.assignment;
        let t = &d.assignment;
        match e {
            Event::Edit { text } => a.edit(t, text)?,
            Event::Save {} => a.save(t)?,
            Event::Publish {} => a.publish(t)?,
            Event::Account { user, enabled } => a.account(t, &user, enabled)?,
            Event::Command { command } => return a.command(t, &command),
            Event::Check {} => a.check(t)?,
        }
        Ok("Simulated share workspace updated; check the current task outcomes.".into())
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
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 512 * 1024
        {
            return Err(
                "Unknown sharing workspace version/identity or saved work exceeds its bound."
                    .into(),
            );
        }
        for a in [&self.assignment, &self.practice] {
            a.world.validate().map_err(|e| e.detail)?;
            if a.editor.len() > 16384 || a.terminal.len() > 2048 {
                return Err("Saved sharing input exceeds its bound.".into());
            }
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
                "Unchecked · check the full sharing task after making your changes.".into(),
            )];
        }
        a.checks(t)
            .into_iter()
            .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
            .collect()
    }
}
fn admitted(s: &Scenario) -> Result<&Definition, String> {
    let (State::SambaShare { simulation }, Goal::SambaShare { definition }) = (&s.initial, &s.goal)
    else {
        return Err("Share source and workspace do not match.".into());
    };
    if s.track != "LPIC-2"
        || s.id != definition.id
        || s.version != definition.version
        || **simulation != definition.new_session()?
    {
        return Err(
            "Match the authored sharing lesson, version and separate initial workspaces.".into(),
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
    let State::SambaShare { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved sharing metadata differs; original data retained.".into());
    }
    d.context(simulation)
}
