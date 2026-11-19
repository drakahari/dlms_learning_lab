//! Separate bounded drafts; saved outputs/worlds/grades are rederived.
use super::*;
use crate::script_teaching::View;
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
            return Err("Use a different ID and supplied instants for worked practice and independent work.".into());
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
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub input: String,
    pub last_run: Option<Run>,
    pub checked: bool,
    pub attempted: bool,
}
fn full() -> Error {
    Error{kind:ErrorKind::Limit,message:"This simulated history reached96actions. Reset only this example or selected attempt to start again.".into()}
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            history: vec![],
            input: String::new(),
            last_run: None,
            checked: false,
            attempted: false,
        }
    }
    pub fn shape(&self) -> Result<()> {
        if self.task_id.len() > 64
            || self.history.len() > MAX_HISTORY
            || self.input.len() > 1024
            || self.input.contains('\0')
            || self.history.iter().any(|a| a.validate().is_err())
            || self
                .last_run
                .as_ref()
                .is_some_and(|r| r.command.text.len() > 1024 || r.command.text.contains('\0'))
            || self.checked && !self.attempted
        {
            return Err("Saved time-zone work exceeds supported bounds or completion provenance; original retained.".into());
        }
        Ok(())
    }
    pub fn world(&self, t: &Task) -> Result<World> {
        self.shape()?;
        if self.task_id != t.id {
            return Err("This draft belongs to a different time-zone activity.".into());
        }
        t.replay(&self.history)
    }
    pub fn last_output(&self, t: &Task) -> Result<Option<std::result::Result<Output, Error>>> {
        self.shape()?;
        let Some(r) = &self.last_run else {
            return Ok(None);
        };
        if r.before > self.history.len() {
            return Err("Saved action follows unavailable history.".into());
        }
        if r.before == MAX_HISTORY && !r.appended && self.history.len() == MAX_HISTORY {
            return Ok(Some(Err(full())));
        }
        if r.appended {
            if r.before + 1 != self.history.len() || self.history.get(r.before) != Some(&r.command)
            {
                return Err("Saved output disagrees with its command history.".into());
            }
        } else if r.before != self.history.len() {
            return Err("A failed command must follow the current history.".into());
        }
        let mut before = t.replay(&self.history[..r.before])?;
        let output = before.command(&r.command.text);
        if output.is_ok() != r.appended {
            return Err("Saved command outcome disagrees with source replay.".into());
        }
        Ok(Some(output))
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        let world = self.world(t)?;
        self.last_output(t)?;
        if self.attempted && self.checked != t.checks(&world).iter().all(|r| r.correct) {
            return Err("Saved completion disagrees with current time-zone outcomes.".into());
        }
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Result<Vec<Check>> {
        Ok(t.checks(&self.world(t)?))
    }
    pub fn run(&mut self, t: &Task) -> Result<std::result::Result<Output, Error>> {
        self.shape()?;
        self.last_output(t)?;
        let current = self.world(t)?;
        let mut next = current.clone();
        let command = Command::new(self.input.clone());
        let before = self.history.len();
        let output = if before == MAX_HISTORY {
            Err(full())
        } else {
            next.command(&command.text)
        };
        if output.is_ok() {
            self.history.push(command.clone());
            if next != current {
                self.checked = false;
                self.attempted = false;
            }
        }
        self.last_run = Some(Run {
            before,
            command,
            appended: output.is_ok(),
        });
        Ok(output)
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
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unknown time-zone workspace version; original retained.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        self.assignment.validate(&d.initial)?;
        self.practice.validate(&d.example)
    }
    pub fn solved(&self, d: &Definition) -> bool {
        d.validate().is_ok()
            && self.validate(d).is_ok()
            && self.assignment.checked
            && self
                .assignment
                .checks(&d.initial)
                .is_ok_and(|r| r.iter().all(|c| c.correct))
    }
    pub fn reset_practice(&mut self, d: &Definition) {
        self.practice = Draft::new(&d.example)
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Draft::new(&d.initial)
    }
}
