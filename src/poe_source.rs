//! Native-only source binding for power plans. No host action or shell parser.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    poe_workspace::{Action, Definition, Session},
    progress::Attempt,
};
use serde::Deserialize;
pub const PREFIX: &str = "poe-workspace ";
pub const HELP: &str = "Configure only supplied virtual ports and power plans. Source-side class reservations, compatible port types and a shared source budget determine power. Listed ports reserve in order in this teaching model; actual switch admission/negotiation differs. No host hardware, electricity, IOS command, packet or network runs. Power is not proof of data connectivity. POE_SUBSET.md gives the scope. Practice and assignment plans are separate.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let raw = input.strip_prefix(PREFIX).ok_or(
        "Use the native connection and power controls; no shell or hardware command runs.",
    )?;
    if raw.len() > 8192 {
        return Err("Keep a native power edit within 8 KiB.".into());
    }
    let r: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native power action: {e}"))?;
    s.action(d, &r.action, r.practice)?;
    Ok("Virtual power plan updated.".into())
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::Poe { simulation }, Goal::Poe { definition }) = (&s.initial, &s.goal) else {
        return Err("Use the matching authored power state and task.".into());
    };
    definition.validate()?;
    if s.track != "CCNA"
        || s.version != 1
        || s.id != definition.source_id
        || s.objective != definition.assignment.instructions
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the CCNA identity, requirements and pristine separate power plans.".into(),
        );
    }
    if s.guide
        .iter()
        .flat_map(|g| &g.commands)
        .any(|c| c.command.starts_with(PREFIX))
    {
        return Err(
            "Internal native power actions are not equipment commands; teach the actual controls."
                .into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::Poe { simulation } = &a.state else {
        return Err("Saved work belongs to another activity; keep the original progress.".into());
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
        return Err(
            "Saved power context differs from the supplied task; preserve the original work."
                .into(),
        );
    }
    d.bound(simulation)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    d.checks(s)
}
