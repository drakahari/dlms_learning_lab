//! Source-bound native kernel actions; no host shell, compiler or boot operation.
use crate::{
    engine::{Goal, State},
    kernel_workspace::{Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
use serde::Deserialize;

pub const PREFIX: &str = "kernel-workspace ";
pub const HELP: &str = "Edit the supplied five-setting configuration fragment, save it, and run a listed make operation against the selected simulated source. Builds are bounded typed records, not compiled code. Native staging and initramfs controls are simulator operations, not Linux commands. A boot trial checks supplied virtual files only. No real files, shell, compiler, firmware, processes or system changes are accessed. Practice and assignment work are separate. KERNEL_BUILD_SUBSET.md documents the finite model.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let raw = input
        .strip_prefix(PREFIX)
        .ok_or("Use the native kernel workspace; this action route is not a shell.")?;
    if raw.len() > 16384 {
        return Err("Keep one native kernel edit within 16 KiB.".into());
    }
    let r: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native kernel action: {e}"))?;
    s.apply(d, r.practice, &r.action)
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::KernelBuild { simulation }, Goal::KernelBuild { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Use matching authored kernel state and requirements.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || s.objective != definition.assignment.instructions
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the LPIC-2 identity, requirements and pristine independent kernel workspaces."
                .into(),
        );
    }
    if s.guide
        .iter()
        .flat_map(|g| &g.commands)
        .any(|c| c.command.starts_with(PREFIX))
    {
        return Err(
            "Internal native kernel actions are not Linux commands; teach the actual controls."
                .into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::KernelBuild { simulation } = &a.state else {
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
        return Err(
            "Saved kernel context differs from its supplied task; original work retained.".into(),
        );
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
            "Not checked · select Check build after your changes.".into(),
        )];
    }
    d.assignment
        .checks(&s.assignment.work)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
        .collect()
}
