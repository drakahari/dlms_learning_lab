//! Bind native rule editing to authored decoded records, never host traffic.
use crate::{
    engine::{Goal, State},
    ids_rule_workspace::{Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
use serde::Deserialize;
pub const PREFIX: &str = "ids-rule-workspace ";
pub const HELP: &str = "Offline rule decisions over supplied decoded IPv4 records only. Edit and save the declared virtual rule, then Replay record set. The bounded subset supports one alert tcp/udp header, numeric IPv4 hosts/CIDRs or any, decimal ports or any, -> or <>, sid/msg/rev and ASCII content literals with optional nocase. All content conditions must match. No live traffic, packet decoding, stream reconstruction, daemon, scanner or firewall action is modeled. Unsupported syntax fails explicitly. Practice is separate from assignment work; IDS_RULE_SUBSET.md records the limits.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let Some(raw) = input.strip_prefix(PREFIX) else {
        return Err("Use the native rule editor and replay controls; no host shell, Snort service or live traffic is involved.".into());
    };
    if raw.len() > 8 * 1024 {
        return Err("Keep native rule actions within 8 KiB.".into());
    }
    let request: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native rule action: {e}"))?;
    s.action(d, &request.action, request.practice)?;
    Ok("Virtual rule workspace updated.".into())
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::IdsRule { simulation }, Goal::IdsRule { definition }) = (&s.initial, &s.goal)
    else {
        return Err("Use the matching authored rule state and task.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err("Match the source identity/version and pristine separate example/assignment workspaces.".into());
    }
    if s.guide
        .iter()
        .flat_map(|g| &g.commands)
        .any(|c| c.command.starts_with(PREFIX))
    {
        return Err("Internal native actions are not Linux commands; teach the native editor and replay controls instead.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::IdsRule { simulation } = &a.state else {
        return Err(
            "Saved work belongs to another activity; preserve the original progress.".into(),
        );
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
            "Saved rule context differs from its authored task; preserve the original progress."
                .into(),
        );
    }
    d.bound(simulation)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    if let Err(e) = d.bound(s) {
        return vec![(false, e)];
    }
    s.assignment
        .checks(&d.assignment)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
        .collect()
}
