//! Authored attribution, action routing and source-bound progress checks.
use super::{
    Result, command, monitor,
    session::{Definition, Session},
    workspace::{Assignment, Ticket, Work},
};
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};
use serde::{Deserialize, Serialize};

pub const PREFIX: &str = "ata-smart-workspace ";
pub const HELP: &str = command::HELP;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    Run {
        #[serde(deserialize_with = "crate::cron::text::<_,{command::INPUT_LIMIT}>")]
        input: String,
    },
    Advance,
    Record {
        ticket: Ticket,
    },
    Draft {
        #[serde(deserialize_with = "crate::cron::text::<_,{monitor::LIMIT}>")]
        contents: String,
    },
    Save,
    Check,
    Reset,
}

pub fn execute(session: &mut Session, definition: &Definition, input: &str) -> Result<String> {
    let data = input
        .strip_prefix(PREFIX)
        .ok_or("Use the native ATA SMART workspace.")?;
    if data.len() > 8192 {
        return Err("Keep one simulated SMART action within 8 KiB.".into());
    }
    session.bind(definition)?;
    let action: Action =
        serde_json::from_str(data).map_err(|e| format!("Unsupported SMART action: {e}"))?;
    apply(&definition.assignment, &mut session.assignment.work, action).map(str::to_string)
}

pub fn apply(
    task: &super::workspace::Task,
    work: &mut Work,
    action: Action,
) -> Result<&'static str> {
    let mut next = work.clone();
    let result = match (&task.goal, action) {
        (Assignment::Diagnostic(_), Action::Run { input }) => {
            next.command = input;
            next.run(task);
            "Recorded the virtual command result."
        }
        (Assignment::Diagnostic(_), Action::Advance) => {
            next.advance(task);
            "Advanced the active virtual disk tests."
        }
        (Assignment::Diagnostic(_), Action::Record { ticket }) => {
            if next.ticket != ticket {
                next.checked = false;
                next.ticket = ticket;
            }
            "Updated the maintenance ticket."
        }
        (Assignment::Monitoring(_), Action::Draft { contents }) => {
            if next.monitor_draft != contents {
                next.checked = false;
                next.monitor_draft = contents;
            }
            "Updated the monitoring draft."
        }
        (Assignment::Monitoring(_), Action::Save) => {
            next.save_monitor()?;
            "Saved the virtual monitoring configuration."
        }
        (_, Action::Check) => {
            next.checked = true;
            "Checked the virtual SMART workspace."
        }
        (_, Action::Reset) => {
            next = Work::new(task);
            "Reset this activity."
        }
        _ => return Err("That control belongs to a different SMART activity.".into()),
    };
    next.validate(task)?;
    *work = next;
    Ok(result)
}

pub fn source(scenario: &Scenario) -> Result<&Definition> {
    let (State::AtaSmart { simulation }, Goal::AtaSmart { definition }) =
        (&scenario.initial, &scenario.goal)
    else {
        return Err("Use a matching virtual ATA SMART starter and grading definition.".into());
    };
    if scenario.track != "LPIC-2"
        || scenario.version != 1
        || scenario.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err("Match the LPIC-2 lesson and its separate initial SMART workspaces.".into());
    }
    if scenario.guide.iter().any(|step| !step.commands.is_empty()) {
        return Err("Author guided actions in the ungraded SMART lesson only.".into());
    }
    Ok(definition)
}

pub fn context(attempt: &Attempt, scenario: &Scenario) -> Result<()> {
    let definition = source(scenario)?;
    let State::AtaSmart { simulation } = &attempt.state else {
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
            "The saved attempt does not match its SMART lesson. The original work was retained."
                .into(),
        );
    }
    simulation.bind(definition)
}

pub fn checks(session: &Session, definition: &Definition) -> Vec<(bool, String)> {
    if let Err(detail) = session.bind(definition) {
        return vec![(false, detail)];
    }
    if !session.assignment.work.checked {
        return vec![(false, "Select Check configuration to grade the current test or saved monitoring configuration.".into())];
    }
    session
        .assignment
        .work
        .checks(&definition.assignment)
        .into_iter()
        .map(|check| (check.passed, format!("{}: {}", check.label, check.detail)))
        .collect()
}
