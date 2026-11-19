//! Authored-source/progress binding; native configuration actions are never shell commands.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    vpn_point_workspace::{Action, Definition, Session},
};
use serde::Deserialize;
pub const PREFIX: &str = "vpn-point-workspace ";
pub const HELP: &str = "Offline two-peer IPv4 UDP/TUN configuration subset. Save virtual peer profiles, Apply profiles and test supplied LAN paths. Identity validity, authority, purpose and key-pair values are supplied metadata; no certificates, signatures, TLS handshake, tunnel, daemon, packet or host file is created or read. Explicit supported routes use the modeled tunnel. Unknown directives, includes, scripts and plugins fail honestly. Limits and the OpenVPN 2.6 reference are documented in VPN_POINT_SUBSET.md. Examples remain separate and ungraded.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let Some(raw) = input.strip_prefix(PREFIX) else {
        return Err("Use the native peer editors and test controls. Profile text is parsed only inside the bounded virtual model.".into());
    };
    if raw.len() > 64 * 1024 {
        return Err("Keep native configuration actions within 64 KiB.".into());
    }
    let request: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native VPN action: {e}"))?;
    s.action(d, &request.action, request.practice)?;
    Ok("Virtual peer workspace updated.".into())
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::VpnPoint { simulation }, Goal::VpnPoint { definition }) = (&s.initial, &s.goal)
    else {
        return Err("Use the matching source-owned VPN state and task.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err("Match the authored identity/version and pristine separate practice/assignment workspaces.".into());
    }
    if s.guide.iter().any(|g| !g.commands.is_empty()) {
        return Err("Teach profile directives in the native editor; internal actions are not Linux commands.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::VpnPoint { simulation } = &a.state else {
        return Err("Saved work belongs to another activity; prior work retained.".into());
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
            "Saved VPN context differs from the source-owned activity; prior work retained.".into(),
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
