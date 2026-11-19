//! Keep the practice draft and the independent assignment bound to their source.
use super::workspace::{Action, Task, Work};
use crate::script_teaching::View;
use serde::{Deserialize, Serialize};

const MAX_SOURCE: usize = 48 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
}

impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.source_id != self.assignment.id
            || self.source_id == self.example.id
            || std::mem::discriminant(&self.assignment.mode)
                != std::mem::discriminant(&self.example.mode)
            || self.assignment.inventory == self.example.inventory
        {
            return Err("Use a versioned lesson with separate assignment and different-input practice tasks.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
}

fn source_snapshot(task: &Task) -> Result<String, String> {
    let text = serde_json::to_string(task).map_err(|error| error.to_string())?;
    if text.len() > MAX_SOURCE {
        return Err("Keep the network boot source within 48 KiB.".into());
    }
    Ok(text)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,49152>")]
    source_task: String,
    pub work: Work,
}

impl Scope {
    fn new(task: &Task) -> Result<Self, String> {
        Ok(Self {
            source_task: source_snapshot(task)?,
            work: task.start()?,
        })
    }

    fn bind(&self, task: &Task) -> Result<(), String> {
        if self.source_task != source_snapshot(task)? {
            return Err("The saved network boot work belongs to different exercise inputs.".into());
        }
        task.effective_inventory(&self.work)?;
        Ok(())
    }

    fn decoded_source(&self) -> Result<Task, String> {
        if self.source_task.len() > MAX_SOURCE {
            return Err("Saved network boot source exceeds 48 KiB.".into());
        }
        serde_json::from_str(&self.source_task)
            .map_err(|error| format!("Cannot read the saved exercise inputs: {error}"))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub example: Scope,
}

impl Session {
    pub fn new(definition: &Definition) -> Result<Self, String> {
        definition.validate()?;
        Ok(Self {
            version: 1,
            source_id: definition.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&definition.assignment)?,
            example: Scope::new(&definition.example)?,
        })
    }

    pub fn bind(&self, definition: &Definition) -> Result<(), String> {
        if self.version != 1 || self.source_id != definition.source_id || definition.version != 1 {
            return Err("The saved network boot work belongs to another lesson or version.".into());
        }
        self.assignment.bind(&definition.assignment)?;
        self.example.bind(&definition.example)
    }

    pub fn validate_shape(&self) -> Result<(), String> {
        let definition = Definition {
            version: self.version,
            source_id: self.source_id.clone(),
            assignment: self.assignment.decoded_source()?,
            example: self.example.decoded_source()?,
        };
        definition.validate()?;
        self.bind(&definition)
    }

    pub fn apply(
        &mut self,
        definition: &Definition,
        practice: bool,
        action: &Action,
    ) -> Result<(), String> {
        self.bind(definition)?;
        let (task, scope) = if practice {
            (&definition.example, &mut self.example)
        } else {
            (&definition.assignment, &mut self.assignment)
        };
        task.apply(&mut scope.work, action)
    }
}
