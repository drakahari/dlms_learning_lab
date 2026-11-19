//! Bind native early-userspace console actions to their authored facts; never a host shell.
use crate::{
    engine::{Goal, State},
    initramfs_workspace::{Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
use serde::Deserialize;
pub const PREFIX: &str = "initramfs-workspace ";
pub const HELP: &str = "Edit the supplied virtual driver-selection fragment and save it. The bounded console models only explicit-kernel dracut generation and lsinitrd driver inspection. Supplied generic base: hostonly=no; known NVMe/ext4 module trees and early userspace. A generation result does not prove root handoff: Check configuration reads the actual selected entry and image. No real dracut, archive/compression, shell, firmware, device or host file runs. Practice and assignment are separate; INITRAMFS_SUBSET.md discloses the limits.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let raw = input.strip_prefix(PREFIX).ok_or(
        "Use the native early-userspace workspace; this internal action route is not Linux syntax.",
    )?;
    if raw.len() > 16384 {
        return Err("Keep one native early-userspace operation within 16 KiB.".into());
    }
    let r: Routed = serde_json::from_str(raw)
        .map_err(|e| format!("Malformed native early-userspace action: {e}"))?;
    s.apply(d, r.practice, &r.action)
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::Initramfs { simulation }, Goal::Initramfs { definition }) = (&s.initial, &s.goal)
    else {
        return Err("Use matching authored early-userspace state and requirements.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || s.objective != definition.assignment.instructions
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the LPIC-2 identity, requirements and pristine separate early-userspace workspaces.".into(),
        );
    }
    if s.guide
        .iter()
        .flat_map(|g| &g.commands)
        .any(|c| c.command.starts_with(PREFIX))
    {
        return Err("Internal native early-userspace actions are not Linux commands; teach the actual console controls.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::Initramfs { simulation } = &a.state else {
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
            "Saved early-userspace context differs from its task; original work retained.".into(),
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
            "Not checked · select Check configuration after changing saved choices or selected boot files.".into(),
        )];
    }
    d.assignment
        .checks(&s.assignment.work)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
        .collect()
}
