//! Bind native filter work to immutable authored cases, never host commands.
use crate::{
    auth_filter_workspace::{Action, Definition, Session},
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};
use serde::Deserialize;
pub const PREFIX: &str = "auth-filter-workspace ";
pub const HELP: &str = "Offline numeric-IPv4 filter tests only. Edit and save the declared virtual filter; cat and the documented fail2ban-regex file/filter form read only supplied virtual text. Matched, ignored and missed describe actual line classification, not a daemon or a firewall ban. Python shorthand/lookarounds/backreferences, includes, date processing, multiline messages, name lookup and executable actions are unsupported. The bounded ASCII subset and resource limits are documented in AUTH_FILTER_SUBSET.md. Practice drafts/results are separate from the assignment.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let Some(raw) = input.strip_prefix(PREFIX) else {
        return Err("Use the native filter editor and virtual command bar; input never goes to a host shell or real Fail2ban service.".into());
    };
    if raw.len() > 8 * 1024 {
        return Err("Keep native filter actions within 8 KiB.".into());
    }
    let request: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native filter action: {e}"))?;
    s.action(d, &request.action, request.practice)?;
    Ok("Virtual filter workspace updated.".into())
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::AuthFilter { simulation }, Goal::AuthFilter { definition }) = (&s.initial, &s.goal)
    else {
        return Err("Use the matching authored filter state and task.".into());
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
        return Err("Internal native actions are not Linux commands; teach the actual declared virtual tools instead.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::AuthFilter { simulation } = &a.state else {
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
            "Saved filter context differs from its authored task; preserve the original progress."
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
