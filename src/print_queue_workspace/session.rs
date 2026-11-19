//! Separate finite histories; serialized worlds, output and grades are never trusted.
use super::*;
use crate::{
    boot_entry::{Error, ErrorKind},
    print_queues::command::Output,
    script_teaching::View,
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id || self.initial.facts == self.example.facts {
            return Err("Use different supplied printers/documents for the example and independent assignment.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub before: usize,
    pub command: Command,
    pub appended: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task_id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,96>")]
    pub history: Vec<Command>,
    pub input: Command,
    pub last_run: Option<Run>,
    pub checked: bool,
    pub attempted: bool,
}
fn full() -> Error {
    Error {
        kind: ErrorKind::Limit,
        message: "Print history limit reached (96). Reset only this attempt to start over.".into(),
    }
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            history: vec![],
            input: Command(String::new()),
            last_run: None,
            checked: false,
            attempted: false,
        }
    }
    fn shape(&self) -> Result<()> {
        if self.task_id.len() > 64
            || self.history.len() > MAX_HISTORY
            || self.input.0.len() > 1024
            || self.history.iter().any(|c| c.0.len() > 1024)
            || self
                .last_run
                .as_ref()
                .is_some_and(|r| r.command.0.len() > 1024)
        {
            return Err(
                "Saved print input/history exceeds its supported bounds; original retained.".into(),
            );
        }
        if self.checked && !self.attempted {
            return Err("A completed print check must have an actual check position.".into());
        }
        Ok(())
    }
    pub fn world(&self, t: &Task) -> Result<World> {
        self.shape()?;
        if self.task_id != t.id {
            return Err("This print draft belongs to a different exercise.".into());
        }
        t.replay(&self.history)
    }
    pub fn last_output(&self, t: &Task) -> Result<Option<std::result::Result<Output, Error>>> {
        let Some(record) = &self.last_run else {
            return Ok(None);
        };
        self.shape()?;
        if record.before > self.history.len() {
            return Err("Last print action follows an unavailable history.".into());
        }
        if record.before == MAX_HISTORY && !record.appended && record.before == self.history.len() {
            return Ok(Some(Err(full())));
        }
        if record.appended {
            if record.before + 1 != self.history.len()
                || self.history.get(record.before) != Some(&record.command)
            {
                return Err(
                    "Saved print output provenance disagrees with the current history.".into(),
                );
            }
        } else if record.before != self.history.len() {
            return Err("A failed print action must follow the current history.".into());
        }
        let mut before = t.replay(&self.history[..record.before])?;
        let output = command::run(&mut before, &t.facts, &record.command.0);
        if output.is_ok() != record.appended {
            return Err("The recorded print action result disagrees with replay.".into());
        }
        Ok(Some(output))
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        let current = self.world(t)?;
        self.last_output(t)?;
        if self.attempted && self.checked != t.grade(&current).iter().all(|r| r.correct) {
            return Err("Saved print success disagrees with the current simulated state.".into());
        }
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<std::result::Result<Output, Error>> {
        let current = self.world(t)?;
        self.last_output(t)?;
        let mut next = current.clone();
        let before = self.history.len();
        let output = if before == MAX_HISTORY {
            Err(full())
        } else {
            command::run(&mut next, &t.facts, &self.input.0)
        };
        if output.is_ok() {
            self.history.push(self.input.clone());
            if next != current {
                self.checked = false;
                self.attempted = false;
            }
        }
        self.last_run = Some(Run {
            before,
            command: self.input.clone(),
            appended: output.is_ok(),
        });
        Ok(output)
    }
    pub fn checks(&self, t: &Task) -> Result<Vec<Check>> {
        Ok(t.grade(&self.world(t)?))
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<Check>> {
        self.checked = false;
        self.attempted = false;
        self.last_output(t)?;
        let rows = self.checks(t)?;
        self.checked = rows.iter().all(|r| r.correct);
        self.attempted = true;
        Ok(rows)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        }
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported print-workspace version; original retained.".into());
        }
        self.assignment.validate(&d.initial)?;
        self.practice.validate(&d.example)
    }
}

impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unknown print-workspace version; original retained.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn solved(&self, d: &Definition) -> bool {
        d.validate().is_ok()
            && self.validate(d).is_ok()
            && self.assignment.checked
            && self
                .assignment
                .checks(&d.initial)
                .is_ok_and(|rows| rows.iter().all(|r| r.correct))
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Draft::new(&d.initial);
    }
    pub fn reset_practice(&mut self, d: &Definition) {
        self.practice = Draft::new(&d.example);
    }
    /// Developer-only authored reachability route. The native terminal runs its
    /// scoped Draft directly; @print-check is never taught as a Linux command.
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String> {
        d.validate()?;
        self.validate(d)?;
        if raw.len() > 1024 {
            return Err("Print command exceeds1024bytes.".into());
        }
        let mut next = self.clone();
        let output = if raw == "@print-check" {
            next.assignment.check(&d.initial)?;
            "Checked the simulated print requirements.".into()
        } else {
            next.assignment.input = Command(raw.into());
            next.assignment
                .run(&d.initial)?
                .map_err(|e| e.to_string())?
                .stdout
        };
        next.validate(d)?;
        *self = next;
        Ok(output)
    }
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(error) = self.validate().and_then(|()| s.validate(self)) {
            return vec![(false, error)];
        }
        let (draft, task) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !draft.attempted {
            return vec![(false,"Not checked. Investigate and repair the supplied queues, then choose Check print work.".into())];
        }
        match draft.checks(task) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| (r.correct, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
}
