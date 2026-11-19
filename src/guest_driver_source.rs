//! Bind authored guest-image actions to their source. Never a host command route.
use crate::{
    engine::{Goal, State},
    guest_driver_workspace::{self as work, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "guest-image ";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::GuestDrivers { simulation }, Goal::GuestDrivers { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Guest image source and workspace must agree.".into());
    };
    definition.validate()?;
    let commands = s
        .guide
        .iter()
        .flat_map(|g| g.commands.iter())
        .collect::<Vec<_>>();
    if s.guide.len() > 8
        || commands.len() > 24
        || commands.iter().any(|c| {
            c.command.len() > 4096
                || c.explanation.len() > 1024
                || (!c.simulator_only && c.command.starts_with(PREFIX))
        })
    {
        return Err("Guest guidance needs bounded real example commands; internal authoring helpers are not Linux syntax.".into());
    }
    if s.track != "LPIC-1"
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the guest lesson identifier and its independent initial workspaces.".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::GuestDrivers { simulation } = &a.state else {
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
        return Err("Saved guest lesson context differs; original work must be retained.".into());
    }
    d.bound(simulation)
}
impl Session {
    /// Closed authoring data for source solutions/tests, not Linux syntax.
    /// The learner terminal wraps ordinary input as Action::Run instead.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        if input.len() > work::MAX_ACTION {
            return Err("Guest authoring action exceeds its bounded size.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native guest workspace; this is not a host command interpreter.")?;
        let action: work::Action =
            serde_json::from_str(raw).map_err(|e| format!("Invalid bounded guest action: {e}"))?;
        let rows = self.action(d, &action, false)?;
        if matches!(action, work::Action::Check) {
            Ok(format!(
                "{}/{} required guest outcomes pass.",
                rows.iter().filter(|r| r.passed).count(),
                rows.len()
            ))
        } else {
            Ok(self.assignment.output.clone())
        }
    }
}
