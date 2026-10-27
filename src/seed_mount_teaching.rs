//! Independent teaching for the original Mount seed, not PersistentMounts.
use crate::{
    engine::{self, Goal, State},
    pack::CommandExample,
    progress::Attempt,
    script_teaching::{Concept, View},
    seed_dns_teaching::limits,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    pub capacity: u64,
    pub used: u64,
    pub cache: u64,
    pub mounted: bool,
    #[serde(deserialize_with = "limits::text256")]
    pub device: String,
    #[serde(deserialize_with = "limits::text256")]
    pub configured_device: String,
    pub writable: bool,
}
impl Volume {
    pub fn valid(&self) -> bool {
        self.capacity > 0
            && self.used <= self.capacity
            && self.cache <= self.used
            && self.device.starts_with("/dev/")
            && self.device.len() <= 256
            && !self.configured_device.is_empty()
            && self.configured_device.len() <= 256
    }
    pub fn from_state(s: &State) -> Option<Self> {
        let State::Mount {
            capacity,
            used,
            cache,
            mounted,
            device,
            configured_device,
            writable,
        } = s
        else {
            return None;
        };
        if *capacity == 0
            || used > capacity
            || cache > used
            || !device.starts_with("/dev/")
            || device.len() > 256
            || configured_device.is_empty()
            || configured_device.len() > 256
        {
            return None;
        }
        Some(Self {
            capacity: *capacity,
            used: *used,
            cache: *cache,
            mounted: *mounted,
            device: device.clone(),
            configured_device: configured_device.clone(),
            writable: *writable,
        })
    }
    pub fn state(&self) -> Result<State, String> {
        if !self.valid() {
            return Err("Invalid mount example".into());
        }
        Ok(State::Mount {
            capacity: self.capacity,
            used: self.used,
            cache: self.cache,
            mounted: self.mounted,
            device: self.device.clone(),
            configured_device: self.configured_device.clone(),
            writable: self.writable,
        })
    }
    pub fn same_problem(&self, fixture: &Self) -> bool {
        self.valid()
            && fixture.valid()
            && self.capacity == fixture.capacity
            && self.device == fixture.device
    }
    pub fn execute(&mut self, command: &str) -> Result<String, String> {
        if !self.valid() || command.len() > 4096 {
            return Err("Invalid mount example or command exceeds 4,096 bytes".into());
        }
        let mut candidate = self.state()?;
        let result = engine::execute(&mut candidate, command);
        if result.is_ok() {
            let next =
                Self::from_state(&candidate).ok_or("Mount command changed the example kind")?;
            if !next.same_problem(self) {
                return Err("Keep the supplied capacity and known device".into());
            }
            *self = next;
        }
        result.map_err(|e| limits::message(&e))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "limits::text256")]
    pub device: String,
}
impl Draft {
    pub fn new(v: &Volume) -> Result<Self, String> {
        if !v.valid() {
            return Err("Invalid mount example before preparing its draft".into());
        }
        Ok(Self {
            device: v.configured_device.clone(),
        })
    }
    pub fn valid(&self) -> bool {
        self.device.len() <= 256
    }
    pub fn matches(&self, v: &Volume) -> bool {
        self.device == v.configured_device
    }
    pub fn apply(&self, v: &Volume) -> Result<Volume, String> {
        if !self.valid() || !v.valid() {
            return Err("Invalid or oversized mount example settings".into());
        }
        let mut candidate = v.state()?;
        crate::workspace::edit_mount(&mut candidate, &self.device)
            .map_err(|e| limits::message(&e))?;
        Volume::from_state(&candidate).ok_or("Mount settings changed the example context".into())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Save {
        draft: Draft,
    },
    Command {
        #[serde(deserialize_with = "limits::command")]
        command: CommandExample,
    },
}
impl Action {
    fn bounded(&self, _fixture: &Volume) -> bool {
        match self {
            Self::Save { draft } => draft.valid(),
            Self::Command { command } => {
                if command.command.is_empty()
                    || command.command.len() > 256
                    || command.explanation.trim().is_empty()
                    || command.explanation.len() > 600
                    || command.simulator_only
                {
                    return false;
                }
                matches!(
                    command
                        .command
                        .split_ascii_whitespace()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    ["df", "-h"]
                        | ["du", "-sh", "/data/cache"]
                        | ["cat", "/etc/fstab"]
                        | ["mount", "/data"]
                        | ["mount", "-o", "remount,rw", "/data"]
                        | ["umount", "/data"]
                        | ["rm", "-r", "/data/cache"]
                )
            }
        }
    }
    pub fn perform(&self, z: &mut Volume) -> Result<(), String> {
        match self {
            Self::Save { draft } => {
                *z = draft.apply(z)?;
                Ok(())
            }
            Self::Command { command } => z.execute(&command.command).map(|_| ()),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleAction {
    #[serde(deserialize_with = "limits::text100")]
    pub title: String,
    #[serde(deserialize_with = "limits::text1200")]
    pub explanation: String,
    pub action: Action,
}
fn examples<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<ExampleAction>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, _, 12>(d)
}
fn experiments<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, _, 4>(d)
}
fn repairs<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, _, 6>(d)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "limits::text1600")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "limits::concepts")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "limits::text1600")]
    pub example_description: String,
    pub fixture: Volume,
    #[serde(deserialize_with = "examples")]
    pub example_actions: Vec<ExampleAction>,
    #[serde(deserialize_with = "limits::text1600")]
    pub try_change: String,
    #[serde(deserialize_with = "experiments")]
    pub try_actions: Vec<Action>,
    #[serde(deserialize_with = "repairs")]
    pub repair_actions: Vec<Action>,
}
impl Lesson {
    pub fn goal(&self) -> Result<Goal, String> {
        Ok(Goal::Mount)
    }
    pub fn bounded(&self) -> bool {
        self.fixture.valid()
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && self.concepts.iter().all(|c| {
                !c.title.trim().is_empty()
                    && c.title.len() <= 100
                    && !c.text.trim().is_empty()
                    && c.text.len() <= 1200
            })
            && [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .all(|s| !s.trim().is_empty() && s.len() <= 1600)
            && !self.example_actions.is_empty()
            && self.example_actions.len() <= 12
            && self.example_actions.iter().all(|a| {
                !a.title.trim().is_empty()
                    && a.title.len() <= 100
                    && !a.explanation.trim().is_empty()
                    && a.explanation.len() <= 1200
                    && a.action.bounded(&self.fixture)
            })
            && !self.try_actions.is_empty()
            && self.try_actions.len() <= 4
            && self.try_actions.iter().all(|a| a.bounded(&self.fixture))
            && !self.repair_actions.is_empty()
            && self.repair_actions.len() <= 6
            && self.repair_actions.iter().all(|a| a.bounded(&self.fixture))
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid or oversized seed mount teaching".into());
        }
        let mut z = self.fixture.clone();
        if engine::solved(&z.state()?, &self.goal()?) {
            return Err("mount example must begin incomplete".into());
        }
        for a in &self.example_actions {
            a.action.perform(&mut z)?;
        }
        if !engine::solved(&z.state()?, &self.goal()?) {
            return Err("mount example is not reachable".into());
        }
        for a in &self.try_actions {
            a.perform(&mut z)?;
        }
        if engine::solved(&z.state()?, &self.goal()?) {
            return Err("mount experiment must leave work remaining".into());
        }
        for a in &self.repair_actions {
            a.perform(&mut z)?;
        }
        if !engine::solved(&z.state()?, &self.goal()?) {
            return Err("mount repair does not restore the example".into());
        }
        Ok(())
    }
    pub fn valid_for(&self, initial: &State, goal: &Goal) -> bool {
        let Some(original) = Volume::from_state(initial) else {
            return false;
        };
        matches!(goal, Goal::Mount)
            && self.fixture.capacity != original.capacity
            && self.fixture.device != original.device
            && self.validate().is_ok()
    }
}
/// Physical context only: never freeze mutable used/cache, pointer or mount flags.
pub fn assignment_context(current: &State, initial: &State) -> bool {
    match (current, initial) {
        (
            State::Mount {
                capacity, device, ..
            },
            State::Mount {
                capacity: original_capacity,
                device: original_device,
                ..
            },
        ) => capacity == original_capacity && device == original_device,
        _ => false,
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u8,
    pub view: View,
    pub fixture: Volume,
    pub current: Volume,
    pub draft: Draft,
    #[serde(deserialize_with = "limits::text4096")]
    pub command: String,
    #[serde(deserialize_with = "limits::transcript")]
    pub transcript: Vec<String>,
    pub selected_action: usize,
    pub checked: bool,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            fixture: lesson.fixture.clone(),
            current: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture)?,
            command: String::new(),
            transcript: Vec::new(),
            selected_action: 0,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.current.same_problem(&self.fixture)
            && self.draft.valid()
            && self.command.len() <= 4096
            && limits::transcript_valid(&self.transcript)
            && self.selected_action < 12
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid()
            && l.bounded()
            && self.fixture == l.fixture
            && self.selected_action < l.example_actions.len()
    }
    pub fn insert(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("mount example context or selection differs".into());
        }
        match &l.example_actions[self.selected_action].action {
            Action::Save { draft } => {
                self.draft = draft.clone();
                self.checked = false;
            }
            Action::Command { command } => self.command = command.command.clone(),
        }
        Ok(())
    }
    pub fn save(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("mount example context differs".into());
        }
        let next = self.draft.apply(&self.current)?;
        if next != self.current {
            self.checked = false;
        }
        self.current = next;
        self.draft = Draft::new(&self.current)?;
        Ok(())
    }
    pub fn submit(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Invalid or foreign mount example".into());
        }
        let command = std::mem::take(&mut self.command);
        if command.trim().is_empty() {
            return Ok(());
        }
        let before = self.current.clone();
        let result = self.current.execute(&command);
        if self.draft.matches(&before) {
            self.draft = Draft::new(&self.current)?;
        }
        if before != self.current {
            self.checked = false;
        }
        limits::record(&mut self.transcript, &format!("practice # {command}"));
        match &result {
            Ok(output) => limits::record(&mut self.transcript, output),
            Err(error) => limits::record(&mut self.transcript, &format!("Command failed: {error}")),
        };
        result.map(|_| ())
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(false, "Invalid or foreign mount example".into())];
        }
        if !self.draft.matches(&self.current) {
            return vec![(
                false,
                "Unsaved example settings; Save configuration before checking these edits".into(),
            )];
        }
        engine::checks(
            &self.current.state().expect("validated mount example"),
            &l.goal().expect("bounded lesson"),
        )
        .into_iter()
        .enumerate()
        .map(|(index, (pass, message))| {
            if pass {
                return (pass, message);
            }
            let diagnosis = match index {
                0 => format!(
                    "Used space is {} of {} MiB, above the 80% practice limit. Inspect the space usage.",
                    self.current.used, self.current.capacity
                ),
                1 => "The saved mount entry points to a different device from the supplied data device. Inspect the persistent entry.".into(),
                2 if !self.current.mounted => "The data filesystem is not mounted. Inspect its current mount state and saved device.".into(),
                2 => "The data filesystem is mounted read-only. Inspect the mount mode before changing files.".into(),
                _ => message,
            };
            (pass, limits::message(&diagnosis))
        })
        .collect()
    }
}
pub fn prepare(a: &mut Attempt, l: &Lesson, new_attempt: bool) -> Result<(), String> {
    crate::workspace::reject_nfs_metadata(a)?;
    if !matches!(a.state, State::Mount { .. }) || l.validate().is_err() {
        return Err("Seed mount teaching needs the original mount activity".into());
    }
    let w = a.workspace.as_mut().ok_or("Prepare the workspace first")?;
    if w.seed_mount_learning
        .as_ref()
        .is_some_and(|x| !x.valid_for(l))
    {
        return Err("Foreign or invalid mount example".into());
    }
    if w.seed_mount_learning.is_none() {
        let mut x = Learning::new(l)?;
        if !new_attempt {
            x.view = View::Assignment;
        }
        w.seed_mount_learning = Some(x);
    }
    Ok(())
}
