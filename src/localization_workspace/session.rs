//! Separate source-bound histories. No serialized world, output or grade is trusted.
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
            return Err(
                "Use different task IDs and supplied inputs for example and independent work."
                    .into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub before: usize,
    pub action: Action,
    pub appended: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task_id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,96>")]
    pub history: Vec<Action>,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub configuration: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub report_input: String,
    pub last_run: Option<Run>,
    pub checked: bool,
    pub attempted: bool,
}
fn full() -> Error {
    Error{kind:ErrorKind::Limit,message:"The simulated history reached96actions. Reset only this example or selected attempt to start over.".into()}
}
fn action_shape(action: &Action) -> bool {
    match action {
        Action::Command { text } => text.len() <= 1024,
        Action::SaveLocale { text } => {
            text.len() <= localization::MAX_CONFIG && !text.contains('\0')
        }
        Action::Report { input } => input.len() <= 128 && !input.contains('\0'),
        Action::NewSession {} => true,
    }
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        let report_input = t
            .cases
            .iter()
            .find_map(|c| {
                if let Case::Report { input, .. } = c {
                    Some(input.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                t.facts
                    .inputs
                    .first()
                    .map_or(String::new(), |f| f.path.clone())
            });
        Self {
            task_id: t.id.clone(),
            history: vec![],
            input: String::new(),
            configuration: t.initial.saved_locale.clone(),
            report_input,
            last_run: None,
            checked: false,
            attempted: false,
        }
    }
    pub fn shape(&self) -> Result<()> {
        if self.task_id.len() > 64
            || self.history.len() > MAX_HISTORY
            || self.input.len() > 1024
            || self.configuration.len() > localization::MAX_CONFIG
            || self.configuration.contains('\0')
            || self.report_input.len() > 128
            || self.report_input.contains('\0')
            || self.history.iter().any(|a| !action_shape(a))
            || self
                .last_run
                .as_ref()
                .is_some_and(|r| !action_shape(&r.action))
        {
            return Err(
                "Saved locale/encoding work exceeds supported bounds; original retained.".into(),
            );
        }
        if self.checked && !self.attempted {
            return Err("Completed work needs an actual check position.".into());
        }
        Ok(())
    }
    pub fn world(&self, t: &Task) -> Result<World> {
        self.shape()?;
        if self.task_id != t.id || !t.facts.inputs.iter().any(|f| f.path == self.report_input) {
            return Err(
                "This draft or selected input belongs to a different supplied activity.".into(),
            );
        }
        t.replay(&self.history)
    }
    pub fn last_output(&self, t: &Task) -> Result<Option<std::result::Result<Output, Error>>> {
        self.shape()?;
        let Some(record) = &self.last_run else {
            return Ok(None);
        };
        if record.before > self.history.len() {
            return Err("Last action follows an unavailable history.".into());
        }
        if record.before == MAX_HISTORY && !record.appended && self.history.len() == MAX_HISTORY {
            return Ok(Some(Err(full())));
        }
        if record.appended {
            if record.before + 1 != self.history.len()
                || self.history.get(record.before) != Some(&record.action)
            {
                return Err("Saved output provenance disagrees with this history.".into());
            }
        } else if record.before != self.history.len() {
            return Err("A failed action must follow the current history.".into());
        }
        let mut before = t.replay(&self.history[..record.before])?;
        let output = before.apply(&record.action);
        if output.is_ok() != record.appended {
            return Err("Saved action outcome disagrees with source replay.".into());
        }
        Ok(Some(output))
    }
    pub fn checks(&self, t: &Task) -> Result<Vec<Check>> {
        let world = self.world(t)?;
        let mut rows = t.checks(&world);
        if t.has_reports() && self.configuration != world.saved_locale() {
            rows.insert(0,Check{label:"Unsaved locale settings".into(),correct:false,feedback:"Save locale settings or restore the edited text. A successful older report does not validate unsaved configuration edits.".into(),expected:None,actual:None});
        }
        Ok(rows)
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        self.world(t)?;
        self.last_output(t)?;
        if self.attempted && self.checked != self.checks(t)?.iter().all(|r| r.correct) {
            return Err("Saved completion disagrees with the current work.".into());
        }
        Ok(())
    }
    pub fn edit_configuration(&mut self, text: String) -> Result<()> {
        if text.len() > localization::MAX_CONFIG || text.contains('\0') {
            return Err("Keep locale settings within2,048bytes with no zero byte.".into());
        }
        if self.configuration != text {
            self.configuration = text;
            self.checked = false;
            self.attempted = false;
        }
        Ok(())
    }
    pub fn perform(
        &mut self,
        t: &Task,
        action: Action,
    ) -> Result<std::result::Result<Output, Error>> {
        self.shape()?;
        self.last_output(t)?;
        if !action_shape(&action) {
            return Err("This action exceeds its supported source bounds.".into());
        }
        let current = self.world(t)?;
        let mut next = current.clone();
        let before = self.history.len();
        let output = if before == MAX_HISTORY {
            Err(full())
        } else {
            next.apply(&action)
        };
        if output.is_ok() {
            if let Action::SaveLocale { text } = &action {
                self.configuration = text.clone();
            }
            self.history.push(action.clone());
            if next != current {
                self.checked = false;
                self.attempted = false;
            }
        }
        self.last_run = Some(Run {
            before,
            action,
            appended: output.is_ok(),
        });
        Ok(output)
    }
    pub fn run(&mut self, t: &Task) -> Result<std::result::Result<Output, Error>> {
        self.perform(
            t,
            Action::Command {
                text: self.input.clone(),
            },
        )
    }
    pub fn save_configuration(&mut self, t: &Task) -> Result<std::result::Result<Output, Error>> {
        self.perform(
            t,
            Action::SaveLocale {
                text: self.configuration.clone(),
            },
        )
    }
    pub fn run_report(&mut self, t: &Task) -> Result<std::result::Result<Output, Error>> {
        self.perform(
            t,
            Action::Report {
                input: self.report_input.clone(),
            },
        )
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
            return Err("Unknown locale/encoding workspace version; original retained.".into());
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
                .is_ok_and(|r| r.iter().all(|r| r.correct))
    }
    pub fn reset_practice(&mut self, d: &Definition) {
        self.practice = Draft::new(&d.example);
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Draft::new(&d.initial);
    }
}
