//! Authored task binding and grading for the offline Apache HTTPS editor.
use super::{
    Result,
    session::{Definition, Session},
    workspace::Action,
};
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};

pub const PREFIX: &str = "apache-https-workspace ";
pub const HELP: &str = "Use the Apache HTTPS editor to save and apply the virtual configuration. Inspect the supplied client trials before checking.";

pub fn execute(session: &mut Session, definition: &Definition, input: &str) -> Result<String> {
    let data = input
        .strip_prefix(PREFIX)
        .ok_or("Use the native Apache HTTPS workspace.")?;
    if data.len() > 8192 {
        return Err("Keep one virtual Apache action within 8 KiB.".into());
    }
    session.bind(definition)?;
    let action: Action = serde_json::from_str(data)
        .map_err(|error| format!("Unsupported HTTPS workspace action: {error}"))?;
    let mut next = session.assignment.work.clone();
    next.apply(&definition.assignment, action)?;
    next.validate(&definition.assignment)?;
    session.assignment.work = next;
    Ok("Updated the supplied HTTPS workspace.".into())
}

pub fn source(scenario: &Scenario) -> Result<&Definition> {
    let (State::ApacheHttps { simulation }, Goal::ApacheHttps { definition }) =
        (&scenario.initial, &scenario.goal)
    else {
        return Err("Use matching supplied Apache HTTPS work and its grading definition.".into());
    };
    if scenario.track != "LPIC-2"
        || scenario.version != 1
        || scenario.id != definition.source_id
        || **simulation != Session::new(definition)?
        || scenario.guide.iter().any(|step| !step.commands.is_empty())
    {
        return Err(
            "Bind the Apache HTTPS lesson to its two independent virtual activities.".into(),
        );
    }
    Ok(definition)
}

pub fn context(attempt: &Attempt, scenario: &Scenario) -> Result<()> {
    let definition = source(scenario)?;
    let State::ApacheHttps { simulation } = &attempt.state else {
        return Err("Saved work belongs to a different activity.".into());
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
            "The saved work does not match this HTTPS lesson. The original work was retained."
                .into(),
        );
    }
    simulation.bind(definition)
}

pub fn checks(session: &Session, definition: &Definition) -> Vec<(bool, String)> {
    if let Err(error) = session.bind(definition) {
        return vec![(false, error)];
    }
    if !session.assignment.work.checked {
        return vec![(
            false,
            "Select Check configuration to check the saved and applied setup.".into(),
        )];
    }
    session
        .assignment
        .work
        .checks(&definition.assignment)
        .into_iter()
        .map(|outcome| {
            (
                outcome.passed,
                format!("{}. {}", outcome.label, outcome.feedback),
            )
        })
        .collect()
}
