//! Bind native DKMS console actions to their authored facts; never a host shell.
use crate::{
    dkms_workspace::{Action, Definition, Session},
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};
use serde::Deserialize;
pub const PREFIX: &str = "dkms-workspace ";
pub const HELP: &str = "Use the supplied DKMS console and its displayed kernels/source inventory. Commands change a bounded in-process registry only. Added, built and installed are separate states; installation does not prove a loaded or working device. Practice and assignment own separate data. DKMS_SUBSET.md documents supported syntax, fixtures and excluded compilation/version-replacement effects.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let raw = input
        .strip_prefix(PREFIX)
        .ok_or("Use the native DKMS workspace; this internal action route is not Linux syntax.")?;
    if raw.len() > 8192 {
        return Err("Keep one native DKMS operation within 8 KiB.".into());
    }
    let r: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native DKMS action: {e}"))?;
    s.apply(d, r.practice, &r.action)
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::Dkms { simulation }, Goal::Dkms { definition }) = (&s.initial, &s.goal) else {
        return Err("Use matching authored DKMS state and requirements.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || s.objective != definition.assignment.instructions
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the LPIC-2 identity, requirements and pristine separate DKMS workspaces.".into(),
        );
    }
    if s.guide
        .iter()
        .flat_map(|g| &g.commands)
        .any(|c| c.command.starts_with(PREFIX))
    {
        return Err("Internal native DKMS actions are not Linux commands; teach the actual console controls.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::Dkms { simulation } = &a.state else {
        return Err("Saved work belongs to another activity; original progress retained.".into());
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
        return Err("Saved DKMS context differs from its task; original work retained.".into());
    }
    simulation.bind(d)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    if let Err(e) = s.bind(d) {
        return vec![(false, e)];
    }
    if !s.assignment.work.checked {
        return vec![(
            false,
            "Not checked · select Check configuration after changing module state.".into(),
        )];
    }
    d.assignment
        .checks(&s.assignment.work)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
        .collect()
}
