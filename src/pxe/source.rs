//! Tie a native PXE workspace and its progress to the exact authored facts.
use super::{
    session::{Definition, Session},
    workspace::Action,
};
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};
use serde::Deserialize;

pub const PREFIX: &str = "pxe-workspace ";
pub const HELP: &str = "Use the native workspace to inspect supplied virtual boot files and client outcomes. Saved edits affect only the represented DHCP offers and PXELINUX configuration. No real DHCP, TFTP, firmware, network or host file runs.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}

pub fn execute(
    session: &mut Session,
    definition: &Definition,
    input: &str,
) -> Result<String, String> {
    let json = input.strip_prefix(PREFIX).ok_or(
        "Use the native network boot workspace. This action route is internal to the simulation.",
    )?;
    if json.len() > 16_384 {
        return Err("Keep one virtual network boot action within 16 KiB.".into());
    }
    let routed: Routed = serde_json::from_str(json)
        .map_err(|error| format!("Unsupported network boot action: {error}"))?;
    session.apply(definition, routed.practice, &routed.action)?;
    Ok("Updated the virtual network boot workspace.".into())
}

pub fn source(scenario: &Scenario) -> Result<&Definition, String> {
    let (State::Pxe { simulation }, Goal::Pxe { definition }) = (&scenario.initial, &scenario.goal)
    else {
        return Err("Use a matching network boot starter and grading definition.".into());
    };
    if scenario.track != "LPIC-2"
        || scenario.version != 1
        || scenario.id != definition.source_id
        || scenario.objective != definition.assignment.instructions
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the LPIC-2 lesson and its pristine, separate network boot workspaces.".into(),
        );
    }
    if scenario.guide.iter().any(|step| !step.commands.is_empty()) {
        return Err(
            "Network boot uses native editor and trace controls, with no terminal commands.".into(),
        );
    }
    Ok(definition)
}

pub fn context(attempt: &Attempt, scenario: &Scenario) -> Result<(), String> {
    let definition = source(scenario)?;
    let State::Pxe { simulation } = &attempt.state else {
        return Err("Saved work belongs to another activity.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.step >= scenario.steps.len()
        || attempt.hints > scenario.hints.len()
        || attempt.transcript.len() > 120
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("The saved attempt does not match its network boot lesson. The original work was retained.".into());
    }
    simulation.bind(definition)
}

pub fn checks(session: &Session, definition: &Definition) -> Vec<(bool, String)> {
    if let Err(detail) = session.bind(definition) {
        return vec![(false, detail)];
    }
    let work = &session.assignment.work;
    if !work.checked {
        return vec![(
            false,
            "Select Check configuration after saving the current edits.".into(),
        )];
    }
    definition
        .assignment
        .checks(work)
        .into_iter()
        .map(|check| (check.passed, format!("{}: {}", check.label, check.detail)))
        .collect()
}
