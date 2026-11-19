//! Source-bound chrony repair tasks with separate saved text and simulated runtime.
use crate::chrony::{self, Observation, Source};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const PREFIX: &str = "CHRONY SOURCE WORK\n";
type Checks = Vec<(bool, String)>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Source,8>")]
    pub inventory: Vec<Source>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub initial: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub reference: String,
    pub primary: Ipv4Addr,
    pub backup: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub timezone: String,
}

fn text_shape(s: &str) -> bool {
    s.len() <= chrony::MAX_CONFIG && !s.contains('\0')
}

impl Task {
    fn report(&self, source: &str) -> Result<Observation, String> {
        chrony::observe(source, &self.inventory)
    }

    fn outcome(&self, report: &Observation) -> Checks {
        let primary = (
            report.selected == Some(self.primary),
            report.selected.map_or_else(
                || {
                    let reason = report
                        .configured
                        .iter()
                        .find(|source| source.reach != 0)
                        .or_else(|| report.configured.first())
                        .map(|source| format!("{}: {}", source.address, source.assessment.detail()))
                        .unwrap_or_default();
                    format!(
                        "No source is selected. {reason} The task requires {}.",
                        self.primary
                    )
                },
                |ip| format!("Selected {ip}. The task requires {}.", self.primary),
            ),
        );
        let backup = (
            report
                .configured
                .iter()
                .any(|source| source.address == self.backup),
            format!(
                "Keep the documented backup {} configured, even when it has no replies in this sample.",
                self.backup
            ),
        );
        let retired = (
            report
                .configured
                .iter()
                .all(|source| [self.primary, self.backup].contains(&source.address)),
            "Remove the retired source. Keep the approved primary and documented backup.".into(),
        );
        vec![primary, backup, retired]
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 64
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.timezone.is_empty()
            || self.timezone.len() > 64
            || self.timezone.chars().any(char::is_control)
            || self.primary == self.backup
            || !text_shape(&self.initial)
            || !text_shape(&self.reference)
        {
            return Err("Invalid time-source task metadata.".into());
        }
        let starter = self.report(&self.initial)?;
        let reference = self.report(&self.reference)?;
        if self.outcome(&reference).iter().any(|row| !row.0)
            || self.outcome(&starter).iter().all(|row| row.0)
        {
            return Err("Supply an incomplete starter and a reachable clock-source repair.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inspection {
    Sources,
    Tracking,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub source: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub applied: String,
    #[serde(default, deserialize_with = "observed_source")]
    pub checked: Option<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub command: String,
    pub inspection: Option<Inspection>,
}

fn observed_source<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Bounded(#[serde(deserialize_with = "crate::cron::text::<_,2048>")] String);
    Option::<Bounded>::deserialize(deserializer).map(|value| value.map(|s| s.0))
}

impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            source: task.initial.clone(),
            applied: task.initial.clone(),
            checked: None,
            command: String::new(),
            inspection: None,
        }
    }

    fn shape(&self) -> Result<(), String> {
        if !text_shape(&self.source)
            || !text_shape(&self.applied)
            || self.command.len() > 256
            || self.command.contains('\0')
            || self
                .checked
                .as_ref()
                .is_some_and(|seen| seen != &self.source || seen != &self.applied)
        {
            return Err("Saved time-source edit or observation is incompatible.".into());
        }
        Ok(())
    }

    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.shape()?;
        task.report(&self.applied)?;
        Ok(())
    }

    pub fn edit(&mut self, source: String) -> Result<(), String> {
        if !text_shape(&source) {
            return Err("Keep the saved chrony draft within 2 KiB with no NUL.".into());
        }
        if self.source != source {
            self.source = source;
            self.checked = None;
        }
        Ok(())
    }

    pub fn apply(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        task.report(&self.source)?;
        self.applied = self.source.clone();
        self.checked = None;
        self.inspection = None;
        Ok(())
    }

    pub fn set_command(&mut self, command: String) -> Result<(), String> {
        if command.len() > 256 || command.contains('\0') {
            return Err("Keep the simulated chronyc command within 256 bytes.".into());
        }
        self.command = command;
        Ok(())
    }

    pub fn run(&mut self, task: &Task) -> Result<(), String> {
        self.observation(task)?;
        let text = self.command.trim();
        if text.contains(['\n', '\r']) || text.len() > 256 {
            return Err("Run one supported chronyc inspection command.".into());
        }
        let words: Vec<_> = text.split_ascii_whitespace().collect();
        let result = match words.as_slice() {
            ["chronyc", "sources"] | ["chronyc", "-n", "sources"] => Inspection::Sources,
            ["chronyc", "tracking"] | ["chronyc", "-n", "tracking"] => Inspection::Tracking,
            _ => return Err("Supported inspection: chronyc sources or chronyc tracking. Real chronyc and clock changes are outside this exercise.".into()),
        };
        self.inspection = Some(result);
        self.command.clear();
        Ok(())
    }

    pub fn output(&self, task: &Task) -> Result<Option<String>, String> {
        self.inspection
            .map(|inspection| {
                let report = self.observation(task)?;
                Ok(match inspection {
                    Inspection::Sources => report.report_sources(),
                    Inspection::Tracking => report.report_tracking(),
                })
            })
            .transpose()
    }

    pub fn observation(&self, task: &Task) -> Result<Observation, String> {
        self.shape()?;
        task.report(&self.applied)
    }

    pub fn test(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        if self.applied != self.source {
            return Err(
                "Apply the edited chrony configuration before checking the current source.".into(),
            );
        }
        self.checked = Some(self.source.clone());
        Ok(())
    }

    pub fn checks(&self, task: &Task) -> Checks {
        if self.checked.is_none() {
            return vec![(
                false,
                "Check the applied configuration after the final edit.".into(),
            )];
        }
        match self.observation(task) {
            Ok(report) => task.outcome(&report),
            Err(error) => vec![(false, error)],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            || self.initial.primary == self.example.primary
            || self.initial.backup == self.example.backup
        {
            return Err("Use separate source facts for practice and assignment.".into());
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub context: String,
    pub view: crate::script_teaching::View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(definition: &Definition) -> Self {
        Self {
            version: definition.version,
            context: definition.initial.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Draft::new(&definition.initial),
            practice: Draft::new(&definition.example),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.context.is_empty() || self.context.len() > 64 {
            return Err("Saved time-source activity metadata is incompatible.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }

    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.version != definition.version || self.context != definition.initial.id {
            return Err("Saved time-source activity belongs to another lesson.".into());
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
        let edited = input
            .strip_prefix(PREFIX)
            .ok_or("Use the time-source editor for this activity.")?;
        let mut next = self.assignment.clone();
        next.edit(edited.to_owned())?;
        next.apply(&definition.initial)?;
        next.test(&definition.initial)?;
        let checks = next.checks(&definition.initial);
        let passed = checks.iter().filter(|row| row.0).count();
        self.assignment = next;
        Ok(format!(
            "{passed}/{} time-source checks matched.",
            checks.len()
        ))
    }
}

fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (crate::engine::State::Chrony { simulation }, crate::engine::Goal::Chrony { definition }) =
        (&scenario.initial, &scenario.goal)
    else {
        return Err("This chrony task and its supplied workspace do not match.".into());
    };
    if scenario.track != "LPIC-1"
        || definition.initial.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)
    {
        return Err("The chrony task, track, instructions and fresh workspace must match.".into());
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
    let crate::engine::State::Chrony { simulation } = &attempt.state else {
        return Err("This saved time-source work belongs to another activity.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("This saved time-source work has incompatible progress metadata.".into());
    }
    simulation.bound(definition)
}
