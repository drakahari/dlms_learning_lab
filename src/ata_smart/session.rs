//! Separate saved diagnostic/configuration work and bind it to authored inventory.
use super::{
    Result,
    workspace::{Task, Work},
};
use crate::script_teaching::View;
use serde::{Deserialize, Serialize};

const TASK_LIMIT: usize = 24576;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
}

impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.source_id.is_empty()
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || std::mem::discriminant(&self.assignment.goal)
                != std::mem::discriminant(&self.example.goal)
            || self.assignment.inventory == self.example.inventory
        {
            return Err("Use a versioned SMART exercise with independent practice inventory and the same task type.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
}

fn snapshot(task: &Task) -> Result<String> {
    let value = serde_json::to_string(task).map_err(|e| e.to_string())?;
    if value.len() > TASK_LIMIT {
        return Err("Keep the authored SMART inventory and requirements within 24 KiB.".into());
    }
    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,24576>")]
    source: String,
    pub work: Work,
}

impl Scope {
    fn new(task: &Task) -> Result<Self> {
        Ok(Self {
            source: snapshot(task)?,
            work: Work::new(task),
        })
    }

    fn bind(&self, task: &Task) -> Result<()> {
        if self.source != snapshot(task)? {
            return Err("This saved SMART work has different exercise inputs.".into());
        }
        self.work.validate(task)
    }

    fn task(&self) -> Result<Task> {
        if self.source.len() > TASK_LIMIT {
            return Err("The saved SMART inventory exceeds 24 KiB.".into());
        }
        serde_json::from_str(&self.source)
            .map_err(|error| format!("Cannot read the saved SMART inventory: {error}"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn new(definition: &Definition) -> Result<Self> {
        definition.validate()?;
        Ok(Self {
            version: definition.version,
            source_id: definition.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&definition.assignment)?,
            example: Scope::new(&definition.example)?,
        })
    }

    pub fn bind(&self, definition: &Definition) -> Result<()> {
        if self.version != 1
            || self.version != definition.version
            || self.source_id != definition.source_id
        {
            return Err("This saved SMART work belongs to a different lesson or version.".into());
        }
        self.assignment.bind(&definition.assignment)?;
        self.example.bind(&definition.example)
    }

    pub fn validate_shape(&self) -> Result<()> {
        let definition = Definition {
            version: self.version,
            source_id: self.source_id.clone(),
            assignment: self.assignment.task()?,
            example: self.example.task()?,
        };
        definition.validate()?;
        self.bind(&definition)
    }

    pub fn reset_example(&mut self, definition: &Definition) -> Result<()> {
        self.bind(definition)?;
        self.example = Scope::new(&definition.example)?;
        Ok(())
    }
}
