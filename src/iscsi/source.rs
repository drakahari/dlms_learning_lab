//! Keep the lab catalog and local progress tied to the exact exercise facts.
use super::{
    Result, command,
    session::{Definition, Session},
};
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};

pub const HELP: &str = command::HELP;

pub fn execute(session: &mut Session, definition: &Definition, input: &str) -> Result<String> {
    session.bind(definition)?;
    if input.len() > command::INPUT_LIMIT {
        return Err("Keep the virtual initiator command within 1,024 bytes.".into());
    }
    let work = &mut session.assignment.work;
    if input == "iscsi-workspace check" {
        work.checked = true;
        return Ok("Checked the virtual initiator.".into());
    }
    if input.trim().is_empty() {
        return Err("Enter a listed command.".into());
    }
    work.input = input.into();
    work.run(&definition.assignment);
    let run = work.history.last().ok_or("Enter a listed command.")?;
    if run.ok {
        Ok(run.output.clone())
    } else {
        Err(run.output.clone())
    }
}

pub fn source(scenario: &Scenario) -> Result<&Definition> {
    let (State::Iscsi { simulation }, Goal::Iscsi { definition }) =
        (&scenario.initial, &scenario.goal)
    else {
        return Err("Use a matching iSCSI starter and grading definition.".into());
    };
    if scenario.track != "LPIC-2"
        || scenario.version != 1
        || scenario.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err("Match the LPIC-2 lesson and its pristine, separate iSCSI workspaces.".into());
    }
    if scenario.guide.iter().any(|step| !step.commands.is_empty()) {
        return Err("Author guided commands in the isolated practice lesson, without populating the independent assignment.".into());
    }
    Ok(definition)
}

pub fn context(attempt: &Attempt, scenario: &Scenario) -> Result<()> {
    let definition = source(scenario)?;
    let State::Iscsi { simulation } = &attempt.state else {
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
        return Err(
            "The saved attempt does not match its iSCSI lesson. The original work was retained."
                .into(),
        );
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
            "Select Check configuration to grade the current virtual initiator.".into(),
        )];
    }
    definition
        .assignment
        .checks(&work.initiator)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.feedback)))
        .collect()
}
