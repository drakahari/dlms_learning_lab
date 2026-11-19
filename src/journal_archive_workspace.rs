//! Source-bound copied-journal exercises with isolated practice and assignment.
use crate::{cron, journal_archive as archive, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

pub const HELP: &str = "Use journalctl -D DIRECTORY --list-boots. To filter the copied records, supply -b BOOT_ID, -u NAME.service, -p LEVEL and -o cat. Long forms: --directory=, --boot=, --unit=, --priority= and --output=cat. A level includes itself and more urgent levels. Optional --no-pager is accepted. Only the supplied copy is available.";
const PREFIX: &str = "COPIED JOURNAL WORK\n";
type Checks = Vec<(bool, String)>;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "cron::text::<_,1600>")]
    pub instructions: String,
    pub archive: archive::Archive,
    #[serde(deserialize_with = "cron::text::<_,32>")]
    pub expected_boot: String,
    #[serde(deserialize_with = "cron::text::<_,71>")]
    pub expected_unit: String,
    pub expected_priority: u8,
}

impl Task {
    fn inspect(&self, command: &str) -> Result<archive::Observation<'_>, String> {
        self.archive.inspect(&archive::Query::parse(command)?)
    }

    fn validate(&self) -> Result<(), String> {
        self.archive.validate()?;
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.instructions.contains('\0')
        {
            return Err("Supply a copied-journal task identifier and learner instructions.".into());
        }
        let boot = self
            .archive
            .boots
            .iter()
            .find(|boot| boot.id == self.expected_boot);
        let Some(boot) = boot else {
            return Err("The incident boot must be in the copied journal.".into());
        };
        if self.expected_priority > 7
            || !boot.records.iter().any(|record| {
                record.unit == self.expected_unit && record.priority <= self.expected_priority
            })
        {
            return Err(
                "The incident filter must return a message for the specified service.".into(),
            );
        }
        Ok(())
    }

    pub fn boot_list(&self) -> String {
        let boots = &self.archive.boots;
        let mut output = format!(
            "{:>3} {:<32} {:<20} {}\n",
            "IDX", "BOOT ID", "FIRST ENTRY (UTC)", "LAST ENTRY (UTC)"
        );
        for (index, boot) in boots.iter().enumerate() {
            let relative = index as i32 + 1 - boots.len() as i32;
            if let (Some(first), Some(last)) = (boot.records.first(), boot.records.last()) {
                let _ = writeln!(
                    output,
                    "{relative:>3} {} {} {}",
                    boot.id, first.timestamp, last.timestamp
                );
            }
        }
        output
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "cron::text::<_,384>")]
    pub command: String,
    pub listed_boots: bool,
    pub ran_current: bool,
    pub checked: bool,
}

impl Draft {
    fn shape(&self) -> Result<(), String> {
        if self.command.len() > archive::MAX_COMMAND
            || self.command.contains(['\r', '\n', '\0'])
            || (self.checked && !self.ran_current)
        {
            return Err("Saved journal query or check is incompatible.".into());
        }
        Ok(())
    }

    pub fn edit(&mut self, command: String) -> Result<(), String> {
        if command.len() > archive::MAX_COMMAND || command.contains(['\r', '\n', '\0']) {
            return Err("Keep the journal query to one line and 384 bytes.".into());
        }
        if command != self.command {
            self.command = command;
            self.ran_current = false;
            self.checked = false;
        }
        Ok(())
    }

    fn bound(&self, task: &Task) -> Result<(), String> {
        self.shape()?;
        if self.ran_current {
            task.inspect(&self.command)?;
        }
        Ok(())
    }

    pub fn run(&mut self, task: &Task) -> Result<(), String> {
        self.shape()?;
        let observation = task.inspect(&self.command)?;
        self.listed_boots |= matches!(observation, archive::Observation::Boots(_));
        self.ran_current = true;
        self.checked = false;
        Ok(())
    }

    pub fn check(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        if !self.ran_current {
            return Err("Run the current query before checking it.".into());
        }
        self.checked = true;
        Ok(())
    }

    pub fn showing_boot_list(&self) -> bool {
        self.ran_current
            && archive::Query::parse(&self.command)
                .is_ok_and(|query| query.selection == archive::Selection::ListBoots)
    }

    pub fn ran_message_query(&self) -> bool {
        self.ran_current && !self.showing_boot_list()
    }

    pub fn checks(&self, task: &Task) -> Checks {
        if !self.checked || !self.ran_current {
            return vec![(false, "Run and check the incident query.".into())];
        }
        let selection = match archive::Query::parse(&self.command) {
            Ok(query) => query.selection,
            Err(error) => return vec![(false, error)],
        };
        let archive::Selection::Messages {
            boot,
            unit,
            priority,
        } = selection
        else {
            return vec![(
                false,
                "You listed the boots. Now query the incident boot’s service messages.".into(),
            )];
        };
        vec![
            (
                self.listed_boots,
                "List the boots in the copied journal before submitting the incident evidence.".into(),
            ),
            (
                boot.eq_ignore_ascii_case(&task.expected_boot),
                "Use the boot whose time window contains the incident. Compare its full ID with the copied boot list.".into(),
            ),
            (
                unit == task.expected_unit,
                format!(
                    "The query selects {unit}. The task asks for messages from {}.",
                    task.expected_unit
                ),
            ),
            (
                priority == task.expected_priority,
                format!(
                    "The {} threshold includes levels 0–{priority}. The task asks for {}, which includes levels 0–{}.",
                    crate::linux_logging::SEVERITIES[usize::from(priority)],
                    crate::linux_logging::SEVERITIES
                        .get(usize::from(task.expected_priority))
                        .unwrap_or(&"specified in the task"),
                    task.expected_priority,
                ),
            ),
        ]
    }

    pub fn output(&self, task: &Task) -> Result<Option<String>, String> {
        if !self.ran_current {
            return Ok(None);
        }
        let observation = task.inspect(&self.command)?;
        let output = match observation {
            archive::Observation::Boots(_) => task.boot_list(),
            archive::Observation::Messages { records, .. } if records.is_empty() => {
                "-- No entries --".into()
            }
            archive::Observation::Messages { records, .. } => records
                .iter()
                .map(|record| record.message.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        };
        Ok(Some(output))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}

impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.initial.id == self.example.id
            || self.initial.archive.directory == self.example.archive.directory
            || self.initial.expected_boot == self.example.expected_boot
        {
            return Err("Use separate copied journals for practice and assignment.".into());
        }
        self.initial.validate()?;
        self.example.validate()
    }

    pub fn checks(&self, session: &Session) -> Checks {
        if let Err(error) = session.bound(self) {
            return vec![(false, error)];
        }
        session.assignment.checks(&self.initial)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "cron::text::<_,64>")]
    pub context: String,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}

impl Session {
    pub fn new(definition: &Definition) -> Self {
        Self {
            version: definition.version,
            context: definition.initial.id.clone(),
            view: View::Introduction,
            assignment: Draft::default(),
            practice: Draft::default(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.context.is_empty() || self.context.len() > 64 {
            return Err("Saved copied-journal activity metadata is incompatible.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }

    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.version != definition.version || self.context != definition.initial.id {
            return Err("This saved query belongs to another lesson.".into());
        }
        self.assignment.bound(&definition.initial)?;
        self.practice.bound(&definition.example)
    }

    pub fn execute_reference(
        &mut self,
        definition: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.bound(definition)?;
        let command = input
            .strip_prefix(PREFIX)
            .ok_or("Use the copied-journal console for this activity.")?;
        let mut next = self.assignment.clone();
        next.edit(format!(
            "journalctl -D {} --list-boots",
            definition.initial.archive.directory
        ))?;
        next.run(&definition.initial)?;
        next.edit(command.to_owned())?;
        next.run(&definition.initial)?;
        next.check(&definition.initial)?;
        let checks = next.checks(&definition.initial);
        let passed = checks.iter().filter(|row| row.0).count();
        self.assignment = next;
        Ok(format!(
            "{passed}/{} incident query checks matched.",
            checks.len()
        ))
    }
}

fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::CopiedJournal { simulation },
        crate::engine::Goal::CopiedJournal { definition },
    ) = (&scenario.initial, &scenario.goal)
    else {
        return Err("The copied-journal task does not match its workspace.".into());
    };
    if scenario.track != "LPIC-1"
        || definition.initial.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)
    {
        return Err(
            "The copied-journal task, track, instructions and initial workspace must match.".into(),
        );
    }
    Ok(definition)
}

pub fn source(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let definition = admitted(scenario)?;
    definition.validate()?;
    Ok(definition)
}

pub fn context(
    attempt: &crate::progress::Attempt,
    scenario: &crate::pack::Scenario,
) -> Result<(), String> {
    let definition = admitted(scenario)?;
    let crate::engine::State::CopiedJournal { simulation } = &attempt.state else {
        return Err("This saved query belongs to another activity.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("This saved query has incompatible progress metadata.".into());
    }
    simulation.bound(definition)
}
