//! Closed replay, separate drafts and current success. No serialized runtime
//! world, query output or imported case results are trusted.
use super::*;
use crate::script_teaching::View;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id || self.initial.facts == self.example.facts {
            return Err("Use different supplied DNS data for guided practice and the independent assignment.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Client(#[serde(deserialize_with = "crate::cron::text::<_,16>")] pub String);
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
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,64>")]
    pub history: Vec<Action>,
    pub editor: Config,
    pub query_client: Client,
    pub query_name: Name,
    pub query_type: QueryType,
    pub recursive: bool,
    pub transfer_client: Client,
    pub transfer_zone: Name,
    pub checked: bool,
    pub attempted: bool,
    pub last: Option<Run>,
}
impl Draft {
    pub fn new(task: &Task) -> Result<Self, String> {
        let world = task.initial()?;
        Ok(Self {
            task_id: task.id.clone(),
            history: task.starter.clone(),
            editor: Config(world.saved_text().into()),
            query_client: Client(String::new()),
            query_name: Name(String::new()),
            query_type: QueryType::A,
            recursive: true,
            transfer_client: Client(String::new()),
            transfer_zone: Name(String::new()),
            checked: false,
            attempted: false,
            last: None,
        })
    }
    pub(super) fn shape(&self) -> Result<(), String> {
        if self.history.len() > MAX_HISTORY
            || self.editor.0.len() > 8192
            || self.query_client.0.len() > 16
            || self.transfer_client.0.len() > 16
            || self.query_name.0.len() > 254
            || self.transfer_zone.0.len() > 254
        {
            return Err("A saved DNS draft exceeds the supported size limit.".into());
        }
        if self.checked && !self.attempted {
            return Err("A completed DNS check must have an actual check position.".into());
        }
        Ok(())
    }
    pub fn world(&self, task: &Task) -> Result<World, String> {
        self.shape()?;
        if self.task_id != task.id {
            return Err("This DNS draft belongs to a different exercise.".into());
        }
        task.replay(&self.history)
    }
    pub fn set_editor(&mut self, text: String) -> Result<(), String> {
        if text.len() > 8192 {
            return Err("Keep the configuration within 8 KiB.".into());
        }
        if self.editor.0 != text {
            self.editor = Config(text);
            self.checked = false;
            self.attempted = false;
        }
        Ok(())
    }
    pub fn query_action(&self) -> Result<Action, String> {
        Ok(Action::Query {
            client: self
                .query_client
                .0
                .parse()
                .map_err(|_| "Query client: enter a complete simulated IPv4 address.")?,
            name: self.query_name.clone(),
            record_type: self.query_type,
            recursive: self.recursive,
        })
    }
    pub fn transfer_action(&self) -> Result<Action, String> {
        Ok(Action::Transfer {
            client: self
                .transfer_client
                .0
                .parse()
                .map_err(|_| "Transfer source: enter a complete simulated IPv4 address.")?,
            zone: self.transfer_zone.clone(),
        })
    }
    pub fn last_output(&self, task: &Task) -> Result<Option<Result<String, String>>, String> {
        let Some(record) = &self.last else {
            return Ok(None);
        };
        if record.before > self.history.len() || record.before < task.starter.len() {
            return Err("Last DNS action does not belong to this draft history.".into());
        }
        if record.before == MAX_HISTORY && !record.appended && record.before == self.history.len() {
            return Ok(Some(Err(
                "Action history limit reached (64); reset this attempt to start again.".into(),
            )));
        }
        if record.appended {
            if record.before + 1 != self.history.len()
                || self.history.get(record.before) != Some(&record.action)
            {
                return Err("Last DNS action does not match the saved successful action.".into());
            }
        } else if record.before != self.history.len() {
            return Err("Last failed DNS action must follow the current history.".into());
        }
        let mut before = task.replay(&self.history[..record.before])?;
        let output = apply(&mut before, &record.action);
        if output.is_ok() != record.appended {
            return Err("Imported DNS output provenance disagrees with replay.".into());
        }
        Ok(Some(output))
    }
    pub fn validate(&self, task: &Task) -> Result<(), String> {
        let world = self.world(task)?;
        self.last_output(task)?;
        let correct =
            self.editor.0 == world.saved_text() && task.grade(&world).iter().all(|r| r.correct);
        if self.attempted && self.checked != correct {
            return Err(
                "Saved DNS check disagrees with the current editor, policy or behavior.".into(),
            );
        }

        Ok(())
    }
    pub fn run(&mut self, task: &Task, action: Action) -> Result<Result<String, String>, String> {
        let world = self.world(task)?;
        self.last_output(task)?;
        let before = self.history.len();
        let mut next = world.clone();
        let output = if before == MAX_HISTORY {
            Err("Action history limit reached (64); reset this attempt to start again.".into())
        } else {
            apply(&mut next, &action)
        };
        if output.is_ok() {
            self.history.push(action.clone());
            if world.saved_text() != next.saved_text()
                || world.live() != next.live()
                || world.zones() != next.zones()
            {
                self.checked = false;
                self.attempted = false;
            }
        }
        self.last = Some(Run {
            before,
            action,
            appended: output.is_ok(),
        });
        if self.checked
            && (self.editor.0 != next.saved_text() || task.grade(&next).iter().any(|r| !r.correct))
        {
            self.checked = false;
            self.attempted = false;
        }
        Ok(output)
    }
    pub fn checks(&self, task: &Task) -> Result<Vec<Check>, String> {
        let world = self.world(task)?;
        let mut rows = vec![row(
            "Editor saved",
            self.editor.0 == world.saved_text(),
            "The current editor matches the saved configuration.",
            "The editor has unsaved changes. Save it, then reload the saved rules before checking their effect.",
        )];
        rows.extend(task.grade(&world));
        Ok(rows)
    }
    pub fn check(&mut self, task: &Task) -> Result<Vec<Check>, String> {
        self.checked = false;
        self.attempted = false;
        self.last_output(task)?;
        let rows = self.checks(task)?;
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
    pub fn new(def: &Definition) -> Result<Self, String> {
        Ok(Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&def.initial)?,
            practice: Draft::new(&def.example)?,
        })
    }
    pub fn validate(&self, def: &Definition) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported DNS workspace version.".into());
        }
        self.assignment.validate(&def.initial)?;
        self.practice.validate(&def.example)?;
        Ok(())
    }
    pub fn solved(&self, def: &Definition) -> bool {
        self.validate(def).is_ok() && self.assignment.checked
    }
    pub fn reset_assignment(&mut self, def: &Definition) -> Result<(), String> {
        self.assignment = Draft::new(&def.initial)?;
        Ok(())
    }
    pub fn reset_practice(&mut self, def: &Definition) -> Result<(), String> {
        self.practice = Draft::new(&def.example)?;
        Ok(())
    }
}
