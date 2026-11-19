//! Separate JSON drafts and applied virtual port inventories.
use crate::{json_inventory as inventory, switching};
use serde::{Deserialize, Serialize};

pub const PREFIX: &str = "JSON INVENTORY\n";
pub const HELP: &str = "This lab's JSON document describes only the supplied access ports. Use strings for names, true/false for up, and integers written with digits for vlan. Include each supplied switch and editable port once. The note may be a short string or null. Apply inventory parses and applies a complete document atomically to the simulated switches. Check network tests the current document's actual Ethernet behavior. No Cisco device, host file, command, or network is used. This is a lab-specific schema; JSON itself supports additional numeric forms.";
type Checks = Vec<(bool, String)>;
fn text_ok(value: &str) -> bool {
    value.len() <= inventory::MAX_DOCUMENT && !value.contains('\0')
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    pub inventory: inventory::Spec,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub initial_applied: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub reference: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,switching::Requirement,12>")]
    pub requirements: Vec<switching::Requirement>,
}
impl Task {
    pub fn outcome(&self, source: &str) -> Checks {
        match self.inventory.apply(source) {
            Ok(fabric) => self
                .requirements
                .iter()
                .map(|requirement| requirement.check(&fabric))
                .collect(),
            Err(error) => vec![(false, error)],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.inventory.validate()?;
        if !identifier(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.instructions.chars().any(char::is_control)
            || !text_ok(&self.starter)
            || !text_ok(&self.initial_applied)
            || !text_ok(&self.reference)
            || !(3..=12).contains(&self.requirements.len())
        {
            return Err("Supply a named inventory task with bounded documents and at least three behavior requirements.".into());
        }
        let initial = self.inventory.apply(&self.initial_applied)?;
        let reference = self.inventory.apply(&self.reference)?;
        if !switching::valid_requirements(&self.requirements, &reference)
            || self
                .requirements
                .iter()
                .any(|requirement| !requirement.check(&reference).0)
            || self
                .requirements
                .iter()
                .all(|requirement| requirement.check(&initial).0)
            || self.inventory.apply(&self.starter).is_ok_and(|fabric| {
                self.requirements
                    .iter()
                    .all(|requirement| requirement.check(&fabric).0)
            })
        {
            return Err("Keep an incomplete inventory starter, protected useful paths and a reachable worked repair.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub source: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub applied: String,
    pub checked: bool,
}
impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            source: task.starter.clone(),
            applied: task.initial_applied.clone(),
            checked: false,
        }
    }
    fn shape(&self) -> Result<(), String> {
        if !text_ok(&self.source)
            || !text_ok(&self.applied)
            || (self.checked && self.source != self.applied)
        {
            return Err(
                "Saved inventory work is oversized or its current result is out of date.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.shape()?;
        task.inventory.apply(&self.applied).map(|_| ())
    }
    pub fn edit(&mut self, source: String) -> Result<(), String> {
        if !text_ok(&source) {
            return Err("Keep the JSON inventory within 8 KiB with no NUL.".into());
        }
        if source != self.source {
            self.source = source;
            self.checked = false;
        }
        Ok(())
    }
    pub fn apply(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        task.inventory.apply(&self.source)?;
        self.applied = self.source.clone();
        self.checked = false;
        Ok(())
    }
    pub fn test(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        if self.source != self.applied {
            return Err("Apply the edited inventory before checking its network behavior.".into());
        }
        self.checked = true;
        Ok(())
    }
    pub fn checks(&self, task: &Task) -> Checks {
        if let Err(error) = self.bound(task) {
            return vec![(false, error)];
        }
        if !self.checked {
            return vec![(
                false,
                "Network unchecked. Apply the current document, then Check network.".into(),
            )];
        }
        task.outcome(&self.applied)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            || self.initial.inventory == self.example.inventory
        {
            return Err("Keep distinct worked and assignment inventory facts.".into());
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
            version: 1,
            context: definition.initial.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Draft::new(&definition.initial),
            practice: Draft::new(&definition.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !identifier(&self.context) {
            return Err("Saved inventory lesson metadata is incompatible.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.version != definition.version || self.context != definition.initial.id {
            return Err("Saved inventory work belongs to another lesson.".into());
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
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the JSON editor for this activity.")?;
        let mut next = self.assignment.clone();
        next.edit(text.into())?;
        next.apply(&definition.initial)?;
        next.test(&definition.initial)?;
        let checks = next.checks(&definition.initial);
        self.assignment = next;
        Ok(format!(
            "{}/{} network requirements passed.",
            checks.iter().filter(|check| check.0).count(),
            checks.len()
        ))
    }
}
fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::JsonInventory { simulation },
        crate::engine::Goal::JsonInventory { definition },
    ) = (&scenario.initial, &scenario.goal)
    else {
        return Err("The inventory task and workspace do not match.".into());
    };
    if scenario.track != "CCNA"
        || definition.initial.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)
    {
        return Err("Match the CCNA inventory task, instructions and initial workspace.".into());
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
    let crate::engine::State::JsonInventory { simulation } = &attempt.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved inventory progress metadata is incompatible.".into());
    }
    simulation.bound(definition)
}
