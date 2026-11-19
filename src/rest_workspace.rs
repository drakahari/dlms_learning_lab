//! Source-bound request replay, separate drafts, and current-state network grading.
use crate::{rest_controller as api, switching};
use serde::{Deserialize, Serialize};

pub const PREFIX: &str = "REST REQUEST\n";
pub const HELP: &str = "Send request changes only this lesson's simulated controller resources. GET reads; POST creates a profile; PUT replaces a representation; PATCH applies JSON Merge Patch; DELETE removes an unused profile. Use application/json for POST/PUT or application/merge-patch+json for PATCH. Supplied Bearer lab-reader/lab-operator values are simulated identities, not real credentials. Check network grades actual resources and Ethernet paths, not a status code. No server, socket, host command or network is used.";
type Checks = Vec<(bool, String)>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Network { requirement: switching::Requirement },
    PreserveProfile { name: String },
    PreserveDescription { name: String },
    AbsentProfile { name: String },
}
impl Requirement {
    fn valid(&self, task: &Task) -> bool {
        match self {
            Self::Network { requirement } => requirement.valid_for(&task.api.inventory.fabric),
            Self::PreserveProfile { name }
            | Self::PreserveDescription { name }
            | Self::AbsentProfile { name } => {
                api::identifier(name) && task.initial.profiles.contains_key(name)
            }
        }
    }
    fn check(&self, task: &Task, store: &api::Store, fabric: &switching::Fabric) -> (bool, String) {
        match self {
            Self::Network { requirement } => requirement.check(fabric),
            Self::PreserveProfile { name } => (
                store.profiles.get(name) == task.initial.profiles.get(name),
                format!("Keep profile {name} unchanged; it protects another workgroup."),
            ),
            Self::PreserveDescription { name } => (
                store
                    .profiles
                    .get(name)
                    .is_some_and(|p| p.description == task.initial.profiles[name].description),
                format!(
                    "Keep the existing description of {name}. PATCH retains omitted fields; PUT needs the full representation."
                ),
            ),
            Self::AbsentProfile { name } => (
                !store.profiles.contains_key(name),
                format!("Retire unused profile {name} after moving its port bindings."),
            ),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    pub api: api::Spec,
    pub initial: api::Store,
    pub starter: api::Request,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,api::Request,32>")]
    pub reference: Vec<api::Request>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
impl Task {
    pub fn outcome(&self, store: &api::Store) -> Checks {
        match self.api.project(store) {
            Ok(fabric) => self
                .requirements
                .iter()
                .map(|r| r.check(self, store, &fabric))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.api.project(&self.initial)?;
        self.starter.validate()?;
        if !api::identifier(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.instructions.chars().any(char::is_control)
            || !(3..=12).contains(&self.requirements.len())
            || self.requirements.iter().any(|r| !r.valid(self))
            || !(1..=32).contains(&self.reference.len())
        {
            return Err("Supply a named, bounded controller task with valid independent requirements and worked requests.".into());
        }
        let network: Vec<_> = self
            .requirements
            .iter()
            .filter_map(|r| {
                if let Requirement::Network { requirement } = r {
                    Some(requirement.clone())
                } else {
                    None
                }
            })
            .collect();
        if !switching::valid_requirements(&network, &self.api.inventory.fabric) {
            return Err("Controller tasks need positive network paths and protected endpoints for separation checks.".into());
        }
        if self.outcome(&self.initial).iter().all(|c| c.0) {
            return Err("The supplied assignment must need a meaningful change.".into());
        }
        let mut draft = Draft::new(self);
        for request in &self.reference {
            draft.edit(request.clone())?;
            let response = draft.send(self)?;
            if !(200..300).contains(&response.status) {
                return Err(format!(
                    "Worked request failed: {} {}",
                    response.status, response.body
                ));
            }
        }
        draft.test(self)?;
        if !draft.checks(self).iter().all(|c| c.0) {
            return Err("Worked requests must reach every current-state requirement.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub request: api::Request,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,api::Request,64>")]
    pub changes: Vec<api::Request>,
    pub checked: bool,
}
impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            request: task.starter.clone(),
            changes: Vec::new(),
            checked: false,
        }
    }
    fn shape(&self) -> Result<(), String> {
        self.request.validate()?;
        if self.changes.len() > 64
            || serde_json::to_vec(&self.changes)
                .map_err(|e| e.to_string())?
                .len()
                > 16384
        {
            return Err("Keep at most 64 applied requests / 16 KiB of request history. Reset this example or attempt to start again.".into());
        }
        for request in &self.changes {
            request.validate()?;
        }
        Ok(())
    }
    pub fn store(&self, task: &Task) -> Result<api::Store, String> {
        self.shape()?;
        let mut store = task.initial.clone();
        task.api.project(&store)?;
        for request in &self.changes {
            let response = api::send(&task.api, &mut store, request)?;
            if !(200..300).contains(&response.status) || !response.changed {
                return Err(
                    "Saved controller history does not match this task; original work retained."
                        .into(),
                );
            }
        }
        Ok(store)
    }
    pub fn edit(&mut self, request: api::Request) -> Result<(), String> {
        request.validate()?;
        if request != self.request {
            self.request = request;
            self.checked = false;
        }
        Ok(())
    }
    pub fn send(&mut self, task: &Task) -> Result<api::Response, String> {
        let mut store = self.store(task)?;
        let response = api::send(&task.api, &mut store, &self.request)?;
        if response.changed {
            let mut next = self.clone();
            next.changes.push(self.request.clone());
            next.checked = false;
            next.shape()?;
            *self = next;
        } else if !(200..300).contains(&response.status) {
            self.checked = false;
        }
        Ok(response)
    }
    pub fn test(&mut self, task: &Task) -> Result<(), String> {
        self.store(task)?;
        self.checked = true;
        Ok(())
    }
    pub fn checks(&self, task: &Task) -> Checks {
        match self.store(task) {
            Err(e)=>vec![(false,e)],
            Ok(store) if self.checked=>task.outcome(&store),
            Ok(_)=>vec![(false,"Network unchecked. Send changes, then Check network to test applied resources and paths.".into())],
        }
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
            || self.initial.initial == self.example.initial
            || self.initial.api == self.example.api
        {
            return Err(
                "Keep worked and assignment networks, resources and identities separate.".into(),
            );
        }
        self.initial.validate()?;
        self.example.validate()
    }
    pub fn checks(&self, session: &Session) -> Checks {
        if let Err(e) = session.bound(self) {
            return vec![(false, e)];
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
        if self.version != 1
            || !api::identifier(&self.context)
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 65536
        {
            return Err("Saved controller work has incompatible metadata or exceeds 64 KiB; original retained.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.context != definition.initial.id || self.version != definition.version {
            return Err("Saved controller work belongs to another task; original retained.".into());
        }
        self.assignment.store(&definition.initial)?;
        self.practice.store(&definition.example)?;
        Ok(())
    }
    pub fn execute_reference(
        &mut self,
        definition: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.bound(definition)?;
        if input.len() > 8192 {
            return Err("Controller request exceeds the bounded editor input.".into());
        }
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use this activity's native request editor.")?;
        let request: api::Request =
            serde_json::from_str(text).map_err(|e| format!("Request fields: {e}"))?;
        let mut next = self.assignment.clone();
        next.edit(request)?;
        let response = next.send(&definition.initial)?;
        if (200..300).contains(&response.status) {
            next.test(&definition.initial)?;
        }
        self.assignment = next;
        Ok(format!(
            "Simulated response {}\n{}",
            response.status, response.body
        ))
    }
}

fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::RestController { simulation },
        crate::engine::Goal::RestController { definition },
    ) = (&scenario.initial, &scenario.goal)
    else {
        return Err("The controller task and workspace do not match.".into());
    };
    if scenario.track != "CCNA"
        || definition.initial.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)
    {
        return Err("Match the CCNA controller task, instructions and initial workspace.".into());
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
    let crate::engine::State::RestController { simulation } = &attempt.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved controller progress metadata is incompatible.".into());
    }
    simulation.bound(definition)
}
