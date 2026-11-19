//! Bind typed mail work to its authored source. Never a host shell entry point.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    smtp_workspace::{self as work, Definition, Session},
};
pub const PREFIX: &str = "smtp-workspace ";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::SmtpRouting { simulation }, Goal::SmtpRouting { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Mail source and workspace must agree.".into());
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
            c.command.len() > 4096 || c.explanation.len() > 1024 || c.command.starts_with(PREFIX)
        })
    {
        return Err("Mail guidance needs bounded example commands; internal authoring actions are not learner Linux syntax.".into());
    }
    if s.track != "LPIC-2"
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the mail lesson identifier and its independent initial workspaces.".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::SmtpRouting { simulation } = &a.state else {
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
        return Err("Saved mail lesson context differs; original work must be retained.".into());
    }
    d.bound(simulation)
}
impl Session {
    /// Closed authoring data for tests/solutions, never advertised as Linux syntax.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        if input.len() > work::MAX_ACTION + PREFIX.len() {
            return Err("Mail authoring action exceeds its bounded size.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native mail workspace; this is not a host command interpreter.")?;
        let action: work::Action =
            serde_json::from_str(raw).map_err(|e| format!("Invalid bounded mail action: {e}"))?;
        let rows = self.action(d, &action, false)?;
        if matches!(action, work::Action::Check) {
            Ok(format!(
                "{}/{} required mail outcomes pass.",
                rows.iter().filter(|r| r.passed).count(),
                rows.len()
            ))
        } else {
            Ok(self.assignment.output.clone())
        }
    }
}
