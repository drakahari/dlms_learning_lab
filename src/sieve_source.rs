//! Bind a Sieve workspace to its immutable authored lesson; no shell entry point.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    sieve_workspace::{Action, Definition, Session},
};
pub const PREFIX: &str = "sieve-workspace ";
pub const HELP: &str = "Edit the Sieve filter and use Run filter or Stop. Only the supported subset described in this lesson is evaluated in process; no shell, host files, network or real mail delivery.";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::SieveFiltering { simulation }, Goal::SieveFiltering { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Filter source and workspace must agree.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the filter source identifier, version and independent initial workspaces."
                .into(),
        );
    }
    if s.guide.len() > 8
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 2048 || c.command.starts_with(PREFIX))
    {
        return Err(
            "Filter guidance needs bounded Sieve examples, not internal authoring actions.".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::SieveFiltering { simulation } = &a.state else {
        return Err("Saved work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved filter context differs; retain the original for recovery.".into());
    }
    d.bound(simulation)
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        if input.len() > crate::sieve_lex::MAX_SOURCE * 2 + PREFIX.len() {
            return Err("Filter authoring action exceeds its bounded size.".into());
        }
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native Sieve editor; this is not a command shell.")?;
        let action: Action = serde_json::from_str(text)
            .map_err(|e| format!("Invalid bounded filter action: {e}"))?;
        self.action(d, &action, false)?;
        Ok(if self.assignment.running {
            format!(
                "Run in progress: {}/{} samples evaluated.",
                self.assignment.results.len(),
                d.assignment.cases.len()
            )
        } else if let Some(error) = &self.assignment.error {
            error.clone()
        } else {
            format!(
                "Last run: {}/{} sample outcomes pass.",
                self.assignment.results.iter().filter(|r| r.passed).count(),
                d.assignment.cases.len()
            )
        })
    }
}
