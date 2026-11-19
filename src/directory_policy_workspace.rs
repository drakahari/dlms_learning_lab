//! Source-held tasks around the existing bounded directory-policy simulator.
//! Example and assignment drafts use separate replay-bound virtual worlds.
use crate::{
    filesystem::directory_policy::{CoreWorld, ErrorKind},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_HISTORY: usize = 128;
pub const MAX_SESSION: usize = 2 * 1024 * 1024;
fn path(p: &str) -> bool {
    p.starts_with('/')
        && p.len() <= 1024
        && (p == "/" || !p.ends_with('/'))
        && p.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        && p.split('/')
            .skip(1)
            .all(|s| !s.is_empty() && s != "." && s != "..")
}
fn parent(p: &str) -> &str {
    p.rsplit_once('/')
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Create { path: String },
    Remove { path: String },
    Move { source: String, destination: String },
    Copy { source: String, destination: String },
}
impl Operation {
    pub fn command(&self) -> String {
        match self {
            Self::Create { path } => format!("touch '{path}'"),
            Self::Remove { path } => format!("rm '{path}'"),
            Self::Move {
                source,
                destination,
            } => format!("mv '{source}' '{destination}'"),
            Self::Copy {
                source,
                destination,
            } => format!("cp '{source}' '{destination}'"),
        }
    }
    fn paths(&self) -> Vec<&str> {
        match self {
            Self::Create { path } | Self::Remove { path } => vec![path],
            Self::Move {
                source,
                destination,
            }
            | Self::Copy {
                source,
                destination,
            } => vec![source, destination],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Succeeds,
    PermissionDenied,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub label: String,
    pub actor: String,
    pub operation: Operation,
    pub expected: Expected,
}
impl Case {
    fn validate(&self, initial: &CoreWorld) -> Result<(), String> {
        if self.label.is_empty()
            || self.label.len() > 180
            || self.actor == "root"
            || !initial.tree.users.contains_key(&self.actor)
            || self.operation.paths().iter().any(|p| {
                !path(p)
                    || !initial
                        .tree
                        .nodes
                        .get(parent(p))
                        .is_some_and(|n| n.directory)
            })
        {
            return Err("Invalid fixed participant check".into());
        }
        Ok(())
    }
    pub fn check(&self, current: &CoreWorld) -> (bool, String) {
        match current.probe(&self.actor, &self.operation.command()) {
            Ok(actual) => {
                let ok = match self.expected {
                    Expected::Succeeds => actual.succeeded,
                    Expected::PermissionDenied => {
                        !actual.succeeded && actual.failure_kind == Some(ErrorKind::Permission)
                    }
                };
                let detail = if ok {
                    if actual.succeeded {
                        "The simulated action succeeded.".to_owned()
                    } else {
                        actual.diagnostic
                    }
                } else {
                    match self.expected {
                        Expected::Succeeds => {
                            format!("This action needs to work. {}", actual.diagnostic)
                        }
                        Expected::PermissionDenied if actual.succeeded => {
                            "This action needs to be blocked, but it currently succeeds.".to_owned()
                        }
                        Expected::PermissionDenied => format!(
                            "Permissions must block this action; a missing file or another operation error does not demonstrate that. {}",
                            actual.diagnostic
                        ),
                    }
                };
                (ok, format!("{} — {}", self.label, detail))
            }
            Err(e) => (
                false,
                format!("{} — This check could not run: {e}", self.label),
            ),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    Collaboration {
        directory: String,
        cases: Vec<Case>,
    },
    Handover {
        source: String,
        destination: String,
        retire: String,
        collision: String,
        cases: Vec<Case>,
    },
}
impl Task {
    pub fn cases(&self) -> &[Case] {
        match self {
            Self::Collaboration { cases, .. } | Self::Handover { cases, .. } => cases,
        }
    }
    pub fn validate(&self, initial: &CoreWorld) -> Result<(), String> {
        initial.validate().map_err(|e| e.to_string())?;
        if self.cases().is_empty() || self.cases().len() > 12 {
            return Err("Use one to twelve fixed participant checks".into());
        }
        let mut names = BTreeSet::new();
        for case in self.cases() {
            case.validate(initial)?;
            if !names.insert(&case.label) {
                return Err("Duplicate participant check label".into());
            }
        }
        match self {
            Self::Collaboration { directory, .. } => {
                if !path(directory)
                    || !initial
                        .tree
                        .nodes
                        .get(directory)
                        .is_some_and(|n| n.directory && n.owner == initial.tree.user)
                {
                    return Err(
                        "Collaboration task needs a supplied directory owned by the ordinary actor"
                            .into(),
                    );
                }
            }
            Self::Handover {
                source,
                destination,
                retire,
                collision,
                ..
            } => {
                if [source, destination, retire, collision]
                    .iter()
                    .any(|p| !path(p))
                    || BTreeSet::from([source, destination, retire, collision]).len() != 4
                    || initial.tree.nodes.contains_key(destination)
                    || !initial
                        .tree
                        .nodes
                        .get(parent(destination))
                        .is_some_and(|n| n.directory)
                    || [source, retire].iter().any(|p| {
                        !initial
                            .tree
                            .nodes
                            .get(*p)
                            .is_some_and(|n| !n.directory && n.owner == initial.tree.user)
                    })
                {
                    return Err("Handover needs two distinct owned regular source names and an unused destination under a supplied directory".into());
                }
                if !initial
                    .tree
                    .nodes
                    .get(collision)
                    .is_some_and(|n| !n.directory && n.owner != initial.tree.user)
                    || parent(collision) != parent(destination)
                    || [parent(source), parent(destination)].iter().any(|d| {
                        !initial.sticky.contains(*d)
                            || !initial
                                .tree
                                .nodes
                                .get(*d)
                                .is_some_and(|n| n.directory && n.owner != initial.tree.user)
                            || !initial
                                .ordinary_access(
                                    &initial.tree.user,
                                    d,
                                    crate::filesystem::Access::CreateEntry,
                                )
                                .unwrap_or(false)
                    })
                {
                    return Err("The collision task needs two searchable, collaborative sticky folders and a colleague-owned existing destination; directory-owner bypass is outside this assignment.".into());
                }
            }
        }
        Ok(())
    }
    pub fn checks(&self, initial: &CoreWorld, current: &CoreWorld) -> Vec<(bool, String)> {
        if current.validate().is_err() {
            return vec![(
                false,
                "The simulated file state is invalid; reload or reset this attempt.".into(),
            )];
        }
        let common = current.tree.users == initial.tree.users
            && current.tree.user == initial.tree.user
            && current.tree.home == initial.tree.home
            && current.tree.umask == initial.tree.umask
            && current.tree.command_location.is_none();
        let mut rows = vec![(
            common,
            "Keep the supplied participants, groups, home and creation policy unchanged.".into(),
        )];
        match self {
            Self::Collaboration { directory, .. } => {
                let intact = current.tree.nodes == initial.tree.nodes
                    && current.tree.files == initial.tree.files;
                let detail = if current.tree.nodes.get(directory).map(|n| n.mode)
                    != initial.tree.nodes.get(directory).map(|n| n.mode)
                {
                    "The shared folder’s ordinary permissions changed. Keep the supplied read/write/search rights; removing group access locks teammates out. Add restricted deletion without replacing those rights."
                } else if current.tree.files != initial.tree.files {
                    "Some supplied file contents changed. Restricted deletion protects names, not writable data; keep the original contents as well as the directory policy."
                } else {
                    "Keep all supplied files, ownership and ordinary read/write/search permissions unchanged; add restricted deletion to the shared directory."
                };
                rows.push((intact, detail.into()));
                let before: BTreeSet<_> =
                    initial.sticky.iter().filter(|p| *p != directory).collect();
                let after: BTreeSet<_> =
                    current.sticky.iter().filter(|p| *p != directory).collect();
                rows.push((current.sticky.contains(directory)&&before==after,format!("Enable restricted deletion on {directory} while retaining other directory policies.")));
            }
            Self::Handover {
                source,
                destination,
                retire,
                collision,
                ..
            } => {
                let mut expected_nodes = initial.tree.nodes.clone();
                let mut expected_files = initial.tree.files.clone();
                let node = expected_nodes.remove(source);
                let text = expected_files.remove(source);
                if let (Some(node), Some(text)) = (node, text) {
                    expected_nodes.insert(destination.clone(), node);
                    expected_files.insert(destination.clone(), text);
                }
                expected_nodes.remove(retire);
                expected_files.remove(retire);
                rows.push((current.tree.nodes.get(destination)==initial.tree.nodes.get(source)&&current.tree.files.get(destination)==initial.tree.files.get(source),format!("Publish the supplied file at {destination} with its contents, owner, group and permissions intact.")));
                rows.push((
                    !current.tree.nodes.contains_key(source)
                        && !current.tree.nodes.contains_key(retire),
                    "Retire only the supplied original and obsolete names.".into(),
                ));
                let retained = current.tree.nodes == expected_nodes
                    && current.tree.files == expected_files
                    && current.sticky == initial.sticky;
                let detail = if current.tree.files.get(collision)
                    != initial.tree.files.get(collision)
                {
                    format!(
                        "The colleague’s existing {collision} has different or missing contents. Copying into a writable file can change its data even when restricted deletion protects its name. Restore the original contents without weakening the folder policy."
                    )
                } else {
                    "Preserve the colleague’s existing destination, all other files and both directory policies.".into()
                };
                rows.push((retained, detail));
            }
        }
        rows.extend(self.cases().iter().map(|case| case.check(current)));
        rows
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub source_id: String,
    pub initial: CoreWorld,
    pub example: CoreWorld,
    pub goal: Task,
    pub example_goal: Task,
    pub example_commands: Vec<String>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_id.is_empty()
            || self.source_id.len() > 80
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.example_commands.is_empty()
            || self.example_commands.len() > 12
            || self
                .example_commands
                .iter()
                .any(|s| s.len() > 4096 || s.is_empty())
        {
            return Err("Invalid directory-policy source or example commands".into());
        }
        self.goal.validate(&self.initial)?;
        self.example_goal.validate(&self.example)?;
        if self.initial.tree.nodes == self.example.tree.nodes
            && self.initial.tree.files == self.example.tree.files
        {
            return Err("Use different task files or values in practice; a clock or navigation change alone is not a different example.".into());
        }
        Ok(())
    }
    pub fn bound(&self, session: &Session) -> Result<(), String> {
        self.validate()?;
        session.validate()?;
        if session.source_id != self.source_id
            || session.example_command >= self.example_commands.len()
        {
            return Err(
                "This saved work does not belong to the supplied task; prior work was retained."
                    .into(),
            );
        }
        for (scope, initial) in [
            (&session.assignment, &self.initial),
            (&session.practice, &self.example),
        ] {
            let mut replay = initial.clone();
            for action in &scope.history {
                match action {
                    Action::Command { input } => {
                        replay
                            .execute(input)
                            .map_err(|e| format!("Saved action cannot be replayed: {e}"))?;
                    }
                }
            }
            if replay != scope.world {
                return Err("Saved file state does not match this task’s actual bounded action history; prior work was retained.".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, session: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(session) {
            return vec![(false, e)];
        }
        if practice {
            self.example_goal
                .checks(&self.example, &session.practice.world)
        } else {
            self.goal.checks(&self.initial, &session.assignment.world)
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        input: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: CoreWorld,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,128>")]
    pub history: Vec<Action>,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,40>")]
    pub transcript: Vec<String>,
    pub checked: bool,
}
impl Scope {
    fn new(world: &CoreWorld) -> Self {
        Self {
            world: world.clone(),
            history: vec![],
            command: String::new(),
            transcript: vec![],
            checked: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub source_id: String,
    pub view: View,
    pub example_command: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(definition: &Definition) -> Self {
        Self {
            version: 1,
            source_id: definition.source_id.clone(),
            view: View::Introduction,
            example_command: 0,
            assignment: Scope::new(&definition.initial),
            practice: Scope::new(&definition.example),
        }
    }
    pub fn decode(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Directory-policy saved data exceeds its raw input limit.".into());
        }
        let session: Self = serde_json::from_slice(raw)
            .map_err(|e| format!("Invalid directory-policy saved data: {e}"))?;
        session.validate()?;
        Ok(session)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.source_id.len() > 80 || self.example_command >= 12 {
            return Err("Directory-policy saved data exceeds its version or field limits.".into());
        }
        for scope in [&self.assignment, &self.practice] {
            if scope.history.len() > MAX_HISTORY
                || scope.command.len() > 4096
                || scope.transcript.len() > 40
                || scope.transcript.iter().any(|s| s.len() > 8192)
                || scope.history.iter().any(|a| match a {
                    Action::Command { input } => input.len() > 4096,
                })
            {
                return Err("Directory-policy draft, history or output exceeds its limit.".into());
            }
            scope.world.validate().map_err(|e| e.to_string())?;
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SESSION {
            return Err("Directory-policy saved data exceeds its aggregate limit.".into());
        }
        Ok(())
    }
    pub fn run(
        &mut self,
        definition: &Definition,
        input: &str,
        practice: bool,
    ) -> Result<String, String> {
        definition.bound(self)?;
        let mut candidate = self.clone();
        let scope = if practice {
            &mut candidate.practice
        } else {
            &mut candidate.assignment
        };
        let before = scope.world.clone();
        let output = scope.world.execute(input).map_err(|e| e.to_string())?;
        if scope.world != before {
            if scope.history.len() == MAX_HISTORY {
                return Err(
                    "Saved action limit reached; reset this attempt before more changes.".into(),
                );
            }
            scope.history.push(Action::Command {
                input: input.into(),
            });
        }
        scope.checked = false;
        candidate.validate()?;
        *self = candidate;
        Ok(output)
    }
    pub fn participant(
        &self,
        definition: &Definition,
        practice: bool,
        index: usize,
    ) -> Result<(bool, String), String> {
        definition.bound(self)?;
        let (scope, task) = if practice {
            (&self.practice, &definition.example_goal)
        } else {
            (&self.assignment, &definition.goal)
        };
        let case = task
            .cases()
            .get(index)
            .ok_or("Choose a participant check supplied by this lesson.")?;
        Ok(case.check(&scope.world))
    }
}
