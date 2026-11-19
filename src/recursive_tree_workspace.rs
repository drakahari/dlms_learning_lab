//! Source-held ordinary-directory tasks over the opt-in Tree reducer.
//! Saved virtual files are state, not reports or executable scripts.
use crate::{
    filesystem::{Tree, recursive as core},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Write};
pub const MAX_SESSION: usize = 4 * 1024 * 1024;
pub const CHECK_ACTION: &str = "@tree-check";
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
fn path(p: &str) -> bool {
    p.starts_with('/')
        && p != "/"
        && p.len() <= 1024
        && !p.ends_with('/')
        && p.split('/').skip(1).all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.len() <= 255
                && !p.chars().any(char::is_control)
        })
}
fn inside(root: &str, p: &str) -> bool {
    p == root || p.strip_prefix(root).is_some_and(|s| s.starts_with('/'))
}
fn bytes<T: Serialize + ?Sized>(value: &T, cap: usize) -> bool {
    struct Count(usize, usize);
    impl Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Recursive workspace encoding limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0, cap), value).is_ok()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Assignment,
    Practice,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    Stage {
        source: String,
        destination: String,
        private_defaults: bool,
    },
    Retire {
        source: String,
        destination: String,
    },
    Collect {
        source: String,
        destination: String,
        criterion: crate::filesystem::size_predicate::Criterion,
        private_defaults: bool,
    },
}
impl Task {
    pub fn source(&self) -> &str {
        match self {
            Self::Stage { source, .. }
            | Self::Retire { source, .. }
            | Self::Collect { source, .. } => source,
        }
    }
    pub fn destination(&self) -> &str {
        match self {
            Self::Stage { destination, .. }
            | Self::Retire { destination, .. }
            | Self::Collect { destination, .. } => destination,
        }
    }
    fn private(&self) -> bool {
        matches!(
            self,
            Self::Stage {
                private_defaults: true,
                ..
            } | Self::Collect {
                private_defaults: true,
                ..
            }
        )
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub root: String,
    pub initial: Tree,
    pub task: Task,
}
impl Activity {
    pub fn command_help(&self) -> String {
        if matches!(self.task, Task::Collect { .. }) {
            format!(
                "{}\n{}",
                core::HELP,
                crate::filesystem::size_predicate::HELP
            )
        } else {
            core::HELP.into()
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let src = self.task.source();
        let dest = self.task.destination();
        if !self.initial.valid()
            || self.initial.command_location.is_some()
            || self.initial.nodes.len() > 96
            || !path(&self.root)
            || !path(src)
            || !path(dest)
            || src == dest
            || inside(src, dest)
            || inside(dest, src)
            || !inside(&self.root, src)
            || !inside(&self.root, dest)
            || src == self.root
            || dest == self.root
            || !self.initial.nodes.get(src).is_some_and(|n| n.directory)
            || !self
                .initial
                .nodes
                .get(&self.root)
                .is_some_and(|n| n.directory && n.owner == self.initial.user)
        {
            return Err("Recursive task needs valid separate ordinary source/destination paths inside its supplied work folder.".into());
        }
        if self.task.private() && self.initial.umask == 0o077 {
            return Err("Private-default task must start with a different supplied mask.".into());
        }
        if matches!(self.task, Task::Stage { .. }) && self.initial.nodes.contains_key(dest) {
            return Err("Staging starts with an unused independent destination.".into());
        }
        if matches!(self.task, Task::Retire { .. })
            && !self.initial.nodes.get(dest).is_some_and(|n| n.directory)
        {
            return Err(
                "Retirement needs an existing handover directory to inspect and repair.".into(),
            );
        }
        if let Task::Collect { criterion, .. } = self.task {
            if !self
                .initial
                .nodes
                .get(dest)
                .is_some_and(|n| n.directory && n.owner == self.initial.user)
            {
                return Err(
                    "Size collection needs its supplied caller-owned destination folder.".into(),
                );
            }
            let files: Vec<_> = self
                .initial
                .files
                .iter()
                .filter(|(p, _)| inside(src, p))
                .collect();
            let count = files
                .iter()
                .filter(|(_, s)| criterion.matches(s.len()))
                .count();
            if count == 0 || count == files.len() {
                return Err("A size-selection task needs both qualifying files and meaningful nonmatching files.".into());
            }
        }
        if !self.initial.files.keys().any(|p| inside(src, p)) || !bytes(self, 1024 * 1024) {
            return Err("Supply bounded real source-file content for the tree task.".into());
        }
        self.bound(&self.initial)
    }
    /// Domain/authority only. Incorrect but valid work remains resumable.
    pub fn bound(&self, current: &Tree) -> Result<(), String> {
        if !current.valid()
            || current.command_location.is_some()
            || current.user != self.initial.user
            || current.users != self.initial.users
            || current.home != self.initial.home
        {
            return Err("Recursive work's caller, accounts, home or activity context differ; original retained.".into());
        }
        for (p, n) in &self.initial.nodes {
            if !inside(&self.root, p)
                && (current.nodes.get(p) != Some(n)
                    || current.files.get(p) != self.initial.files.get(p))
            {
                return Err(format!(
                    "This activity keeps {p} outside its work folder unchanged."
                ));
            }
        }
        if current
            .nodes
            .keys()
            .any(|p| !self.initial.nodes.contains_key(p) && !inside(&self.root, p))
        {
            return Err(format!(
                "Create or change files inside {} only; unrelated virtual folders stay fixed.",
                self.root
            ));
        }
        Ok(())
    }
    pub fn checks(&self, current: &Tree) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|_| self.bound(current)) {
            return vec![(false, e)];
        }
        let src = self.task.source();
        let dest = self.task.destination();
        let mut selected = BTreeSet::new();
        if let Task::Collect { criterion, .. } = self.task {
            selected.insert(src.to_owned());
            for (p, body) in &self.initial.files {
                if inside(src, p) && criterion.matches(body.len()) {
                    let mut path = p.as_str();
                    while path != src {
                        selected.insert(path.to_owned());
                        path = path.rsplit_once('/').map(|r| r.0).unwrap_or("");
                    }
                }
            }
        }
        let expected: Vec<_> = self
            .initial
            .nodes
            .iter()
            .filter(|(p, _)| {
                inside(src, p) && (selected.is_empty() || selected.contains(p.as_str()))
            })
            .map(|(p, n)| (format!("{dest}{}", &p[src.len()..]), p, n))
            .collect();
        let expected_names: BTreeSet<&str> = expected.iter().map(|(p, _, _)| p.as_str()).collect();
        let actual_names: BTreeSet<&str> = current
            .nodes
            .keys()
            .filter(|p| inside(dest, p))
            .map(String::as_str)
            .collect();
        let missing = expected_names
            .difference(&actual_names)
            .take(3)
            .copied()
            .collect::<Vec<_>>();
        let extra = actual_names
            .difference(&expected_names)
            .take(3)
            .copied()
            .collect::<Vec<_>>();
        let shape = expected_names == actual_names
            && expected.iter().all(|(p, _, n)| {
                current
                    .nodes
                    .get(p)
                    .is_some_and(|a| a.directory == n.directory)
            });
        let contents = expected
            .iter()
            .filter(|(_, _, n)| !n.directory)
            .all(|(p, old, _)| current.files.get(p) == self.initial.files.get(*old));
        let owner = &self.initial.user;
        let group = &self.initial.users[owner][0];
        let ownership = expected.iter().all(|(p, _, _)| {
            current
                .nodes
                .get(p)
                .is_some_and(|n| &n.owner == owner && &n.group == group)
        });
        let modes = expected.iter().all(|(p, _, n)| {
            current.nodes.get(p).is_some_and(|a| {
                a.mode
                    == if self.task.private() {
                        n.mode & 0o700
                    } else {
                        n.mode
                    }
            })
        });
        let source_ok = match self.task {
            Task::Stage { .. } | Task::Collect { .. } => {
                self.initial
                    .nodes
                    .iter()
                    .filter(|(p, _)| inside(src, p))
                    .all(|(p, n)| {
                        current.nodes.get(p) == Some(n)
                            && current.files.get(p) == self.initial.files.get(p)
                    })
                    && current.nodes.keys().filter(|p| inside(src, p)).count()
                        == self.initial.nodes.keys().filter(|p| inside(src, p)).count()
            }
            Task::Retire { .. } => !current.nodes.keys().any(|p| inside(src, p)),
        };
        let outside = self
            .initial
            .nodes
            .iter()
            .filter(|(p, _)| !inside(src, p) && !inside(dest, p))
            .all(|(p, n)| {
                current.nodes.get(p) == Some(n) && current.files.get(p) == self.initial.files.get(p)
            });
        let mut final_names: BTreeSet<&str> = self
            .initial
            .nodes
            .keys()
            .filter(|p| {
                !inside(dest, p) && (!matches!(self.task, Task::Retire { .. }) || !inside(src, p))
            })
            .map(String::as_str)
            .collect();
        final_names.extend(expected_names.iter().copied());
        let no_scratch = current
            .nodes
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == final_names;
        let mut rows=vec![
            (shape,if shape{format!("{dest} contains every nested, hidden and empty entry, with no extra entries.")}else{format!("Inspect the complete handover tree. Missing: {}. Unexpected: {}. Copying only visible top-level files is not enough.",if missing.is_empty(){"none".into()}else{missing.join(", ")},if extra.is_empty(){"none".into()}else{extra.join(", ")})}),
            (contents,if contents{"Every transferred file retains its exact supplied content, including empty/final-newline differences.".into()}else{"One or more transferred files are missing or have different content. Compare nested files with the original before retiring it.".into()}),
            (ownership,format!("Transferred entries must belong to {owner}, primary group {group}; retaining another source owner's name is not a normal new copy.")),
            (modes,if self.task.private(){"Keep the source owner permission bits on the private copy and remove group/other access. A permissive copy or an unreadable zero-mode copy does not meet this task.".into()}else{"Keep the supplied source permissions on the handover. Content alone is not enough.".into()}),
            (source_ok,match self.task{Task::Stage{..}|Task::Collect{..}=>format!("Keep the original {src} tree, contents and permissions intact."),Task::Retire{..}=>format!("Retire only {src}, including its hidden and nested entries. Verify the complete handover before removing it.")}),
            (outside&&no_scratch,"Keep unrelated supplied entries intact and leave only the requested handover/retirement changes; remove any temporary scratch copies.".into()),
        ];
        if let Task::Collect { criterion, .. } = self.task {
            let comparison = match criterion.compare {
                crate::filesystem::size_predicate::Compare::Equal => "exactly",
                crate::filesystem::size_predicate::Compare::Less => "strictly less than",
                crate::filesystem::size_predicate::Compare::Greater => "strictly greater than",
            };
            let explanation = if criterion.unit.bytes() == 1 {
                format!(
                    "Count UTF-8 bytes, not characters; keep files {comparison} {} bytes.",
                    criterion.quantity
                )
            } else {
                format!(
                    "Measure bytes, round up to {}-byte units, then keep files using {comparison} {} units. A nonempty file smaller than one unit still uses one unit.",
                    criterion.unit.bytes(),
                    criterion.quantity
                )
            };
            rows[0].1 = if shape {
                format!(
                    "{dest} contains exactly the size-selected files and their required parent folders."
                )
            } else {
                format!(
                    "Collect only the requested size matches. Missing: {}. Unexpected: {}. {explanation}",
                    if missing.is_empty() {
                        "none".into()
                    } else {
                        missing.join(", ")
                    },
                    if extra.is_empty() {
                        "none".into()
                    } else {
                        extra.join(", ")
                    }
                )
            };
            // Membership already checks every destination name. Keep unrelated/scratch
            // feedback independent; source preservation plus these checks implies the
            // same complete final roster as before, without double-reporting an extra.
            let no_other = current
                .nodes
                .keys()
                .filter(|p| !inside(src, p) && !inside(dest, p))
                .all(|p| self.initial.nodes.contains_key(p));
            rows[5]=(outside&&no_other,"Keep unrelated supplied entries intact and leave no scratch entries outside the source/evidence folders.".into());
        }
        if self.task.private() {
            rows.push((current.umask==0o077,"Set the virtual session's creation mask to 077 so later ordinary files/directories also default to owner-only access. It does not repair permissions on existing copies.".into()));
        }
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub role: Role,
    pub tree: Tree,
    pub command: String,
    pub transcript: Vec<String>,
    pub checked: bool,
}
impl Scope {
    fn new(role: Role, a: &Activity) -> Self {
        Self {
            role,
            tree: a.initial.clone(),
            command: String::new(),
            transcript: Vec::new(),
            checked: false,
        }
    }
    fn bound(&self, role: Role, a: &Activity) -> Result<(), String> {
        if self.role != role
            || self.command.len() > 4096
            || self.transcript.len() > 40
            || self.transcript.iter().any(|s| s.len() > 8192)
        {
            return Err("Recursive draft role or field limits differ; original retained.".into());
        }
        a.bound(&self.tree)
    }
    pub fn checks(&self, a: &Activity) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![(
                false,
                "Not checked · use Check work to inspect the current handover.".into(),
            )];
        }
        a.checks(&self.tree)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub source_id: String,
    pub assignment: Activity,
    pub example: Activity,
    pub reference: Vec<String>,
    pub example_commands: Vec<String>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !name(&self.source_id) || !bytes(self, 2 * 1024 * 1024) {
            return Err("Unknown recursive task version/identity or excessive source size.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if std::mem::discriminant(&self.assignment.task)
            != std::mem::discriminant(&self.example.task)
        {
            return Err("Practice and assignment must teach the same file-operation family using separate supplied files.".into());
        }
        if inside(&self.assignment.root, &self.example.root)
            || inside(&self.example.root, &self.assignment.root)
        {
            return Err("Practice and assignment need separate virtual folders.".into());
        }
        for (commands, max) in [(&self.reference, 16), (&self.example_commands, 8)] {
            if commands.is_empty()
                || commands.len() > max
                || commands
                    .iter()
                    .any(|c| c.trim().is_empty() || c.len() > 4096 || c == CHECK_ACTION)
            {
                return Err("Use bounded real example/reference commands, with Check as a separate native action.".into());
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        if s.version != 1
            || s.origin != self.source_id
            || s.example_command >= self.example_commands.len()
            || !bytes(s, MAX_SESSION)
        {
            return Err(
                "Recursive session source/version, selection or size differs; original retained."
                    .into(),
            );
        }
        s.assignment.bound(Role::Assignment, &self.assignment)?;
        s.practice.bound(Role::Practice, &self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|_| self.bound(s)) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.assignment)
    }
    pub fn validate_reachability(&self) -> Result<(), String> {
        self.validate()?;
        for (practice, commands) in [(false, &self.reference), (true, &self.example_commands)] {
            let mut s = Session::new(self)?;
            for c in commands {
                s.run(self, c, practice)?;
            }
            s.check(self, practice)?;
            let a = if practice {
                &self.example
            } else {
                &self.assignment
            };
            let scope = if practice { &s.practice } else { &s.assignment };
            if !scope.checks(a).iter().all(|r| r.0) {
                return Err(format!(
                    "Recursive {} reference does not reach its outcomes: {:?}",
                    if practice { "example" } else { "assignment" },
                    scope.checks(a)
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub origin: String,
    pub view: View,
    pub example_command: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.source_id.clone(),
            view: View::Introduction,
            example_command: 0,
            assignment: Scope::new(Role::Assignment, &d.assignment),
            practice: Scope::new(Role::Practice, &d.example),
        })
    }
    pub fn shape(&self) -> bool {
        self.version == 1
            && name(&self.origin)
            && self.example_command < 8
            && self.assignment.tree.valid()
            && self.practice.tree.valid()
            && self.assignment.role == Role::Assignment
            && self.practice.role == Role::Practice
            && bytes(self, MAX_SESSION)
    }
    pub fn decode(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Recursive saved input exceeds 4 MiB before decoding.".into());
        }
        let s: Self = serde_json::from_slice(raw)
            .map_err(|e| format!("Invalid recursive saved input: {e}"))?;
        if !s.shape() {
            return Err("Invalid recursive saved shape; original retained.".into());
        }
        Ok(s)
    }
    pub fn run(&mut self, d: &Definition, input: &str, practice: bool) -> Result<String, String> {
        d.validate()?;
        d.bound(self)?;
        let activity = if practice { &d.example } else { &d.assignment };
        let old = if practice {
            &self.practice
        } else {
            &self.assignment
        };
        let mut next = self.clone();
        let scope = if practice {
            &mut next.practice
        } else {
            &mut next.assignment
        };
        let output = if matches!(activity.task, Task::Collect { .. }) {
            crate::filesystem::size_predicate::execute(&mut scope.tree, input)?
        } else {
            core::execute(&mut scope.tree, input).map_err(|e| e.to_string())?
        };
        activity.bound(&scope.tree)?;
        if scope.tree.nodes != old.tree.nodes
            || scope.tree.files != old.tree.files
            || scope.tree.umask != old.tree.umask
        {
            scope.checked = false;
        }
        if !bytes(&next, MAX_SESSION) {
            return Err("Recursive saved-size limit reached; previous work retained.".into());
        }
        *self = next;
        Ok(output)
    }
    pub fn check(&mut self, d: &Definition, practice: bool) -> Result<Vec<(bool, String)>, String> {
        d.validate()?;
        d.bound(self)?;
        let scope = if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        };
        let a = if practice { &d.example } else { &d.assignment };
        scope.checked = true;
        Ok(scope.checks(a))
    }
}
