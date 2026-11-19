//! Bind each exercise attempt to its authored inputs and version.
use super::{
    Result,
    workspace::{Attempt, Task},
};
use crate::script_teaching::View;
use serde::{Deserialize, Serialize};

const SOURCE_LIMIT: usize = 32 * 1024;

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
            || self.source_id.len() > 96
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.assignment.goal.automatic_at_startup != self.example.goal.automatic_at_startup
            || self.assignment.inventory == self.example.inventory
            || self.assignment.goal.target == self.example.goal.target
            || self.assignment.goal.wwn == self.example.goal.wwn
        {
            return Err("Use a versioned iSCSI lesson with different practice inputs and the same kind of task.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
}

fn snapshot(task: &Task) -> Result<String> {
    let text = serde_json::to_string(task).map_err(|e| e.to_string())?;
    if text.len() > SOURCE_LIMIT {
        return Err("Keep the authored iSCSI facts within 32 KiB.".into());
    }
    Ok(text)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    source: String,
    pub work: Attempt,
}

impl Scope {
    fn new(task: &Task) -> Result<Self> {
        Ok(Self {
            source: snapshot(task)?,
            work: Attempt::new(task),
        })
    }

    fn read_task(&self) -> Result<Task> {
        if self.source.len() > SOURCE_LIMIT {
            return Err("Saved exercise facts exceed the size limit.".into());
        }
        serde_json::from_str(&self.source)
            .map_err(|e| format!("Cannot read the saved exercise facts: {e}"))
    }

    fn bind(&self, task: &Task) -> Result<()> {
        if self.source != snapshot(task)? {
            return Err("The saved iSCSI work belongs to different exercise inputs.".into());
        }
        self.work.validate(task)
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
            return Err("The saved iSCSI work belongs to another lesson or version.".into());
        }
        self.assignment.bind(&definition.assignment)?;
        self.example.bind(&definition.example)
    }

    pub fn validate_shape(&self) -> Result<()> {
        let definition = Definition {
            version: self.version,
            source_id: self.source_id.clone(),
            assignment: self.assignment.read_task()?,
            example: self.example.read_task()?,
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
