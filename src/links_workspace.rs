//! Source-bound link lessons over the existing inode VFS. Never host filesystem I/O.
use crate::{links as core, pack::Scenario, progress::Attempt, script_teaching::View};
use serde::{Deserialize, Serialize};
pub const HELP: &str = "Simulated terminal subset: pwd; ls [-i] [DIRECTORY]; cat PATH; stat [-L] PATH; readlink PATH; ln SOURCE NEW_NAME; ln -s TARGET NEW_LINK; cp SOURCE NEW_NAME; mv SOURCE NEW_NAME; rm PATH. Literal single/double quotes and -- are supported. One command at a time: no expansion, operators, pipelines or scripts. Destinations must be unused explicit names; overwriting and directory shorthand are unsupported. Files/links only; supplied directories and cwd stay fixed. stat shows only modeled inode/type/size/link-count fields. The optional native editor saves existing simulated files only. No permissions, mounts, directory mutation or host access. Inode IDs are simulator identities. ls -i lists visible immediate names with their own inode IDs, omitting dot names; no -a/-l or file operands. Bare ls retains its prior simplified all-names listing.";
const MAX_SESSION: usize = 256 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Shared { left: String, right: String },
    Independent { left: String, right: String },
    SymlinkTo { path: String, target: String },
    Content { path: String, text: String },
    Absent { path: String },
    Moved { source: String, destination: String },
    Preserved { path: String },
}
impl Requirement {
    pub fn check(&self, initial: &core::State, current: &core::State) -> (bool, String) {
        let identity = |path: &str| {
            current
                .stat(path)
                .ok()
                .filter(|s| s.kind == core::Kind::Regular)
                .map(|s| s.inode)
        };
        match self {
            Self::Shared { left, right } => {
                let mut pass = identity(left).is_some() && identity(left) == identity(right);
                for text in ["first future edit\n", "another future edit\n"] {
                    let mut probe = current.clone();
                    pass &= probe.write_file(left, text).is_ok()
                        && probe.read_file(right).as_deref() == Ok(text);
                }
                (
                    pass,
                    format!(
                        "{left} and {right} must be names for the same regular file. Equal text in two copies is not enough; compare stat inode IDs."
                    ),
                )
            }
            Self::Independent { left, right } => {
                let mut pass = identity(left).is_some()
                    && identity(right).is_some()
                    && identity(left) != identity(right);
                if let Ok(original) = current.read_file(right) {
                    for text in ["future change one\n", "future change two\n"] {
                        let mut probe = current.clone();
                        pass &= probe.write_file(left, text).is_ok()
                            && probe.read_file(right).as_ref() == Ok(&original);
                    }
                } else {
                    pass = false;
                }
                (
                    pass,
                    format!(
                        "{right} must remain an independent copy of {left}, not a shared inode or symbolic link."
                    ),
                )
            }
            Self::SymlinkTo { path, target } => {
                let to = current.stat_follow(target).ok();
                let via = current.stat_follow(path).ok();
                (
                    current.read_link(path).is_ok()
                        && to.is_some()
                        && via.map(|s| s.inode) == to.map(|s| s.inode),
                    format!(
                        "{path} must be a symbolic link resolving to {target}. A relative target starts in the link’s containing directory, not your working directory; inspect readlink and stat -L."
                    ),
                )
            }
            Self::Content { path, text } => (
                current.read_file(path).as_ref() == Ok(text),
                format!(
                    "Keep the required contents at {path}; copying or relinking must not damage the retained data."
                ),
            ),
            Self::Absent { path } => (
                current
                    .stat(path)
                    .is_err_and(|e| e.kind == core::ErrorKind::NotFound),
                format!(
                    "Retire the old name {path}. Removing one hard-link name must leave the retained alias intact."
                ),
            ),
            Self::Moved {
                source,
                destination,
            } => {
                let before = initial.stat(source).ok();
                let after = current.stat(destination).ok();
                (
                    before.is_some()
                        && before.map(|s| s.inode) == after.map(|s| s.inode)
                        && current
                            .stat(source)
                            .is_err_and(|e| e.kind == core::ErrorKind::NotFound),
                    format!(
                        "Move {source} to {destination} while retaining the original inode. Copy-then-delete does not preserve the file’s identity."
                    ),
                )
            }
            Self::Preserved { path } => (
                initial.stat(path).ok().map(|s| (s.inode, s.kind))
                    == current.stat(path).ok().map(|s| (s.inode, s.kind))
                    && initial.read_file(path).ok() == current.read_file(path).ok()
                    && initial.read_link(path).ok() == current.read_link(path).ok(),
                format!("Preserve {path}, including its type, identity and contents."),
            ),
        }
    }
    fn paths(&self) -> Vec<&str> {
        match self {
            Self::Shared { left, right } | Self::Independent { left, right } => vec![left, right],
            Self::SymlinkTo { path, target } => vec![path, target],
            Self::Moved {
                source,
                destination,
            } => vec![source, destination],
            Self::Content { path, .. } | Self::Absent { path } | Self::Preserved { path } => {
                vec![path]
            }
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: core::State,
    pub example: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,24>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,24>")]
    pub example_requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,12>")]
    pub example_commands: Vec<String>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.initial == self.example {
            return Err("Worked example must use separate initial files".into());
        }
        self.initial.validate().map_err(|e| e.to_string())?;
        self.example.validate().map_err(|e| e.to_string())?;
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 128 * 1024
            || self.requirements.is_empty()
            || self.example_requirements.is_empty()
            || self.example_commands.is_empty()
        {
            return Err("Link lesson definition is empty or exceeds its bound".into());
        }
        for r in self.requirements.iter().chain(&self.example_requirements) {
            if r.paths()
                .iter()
                .any(|p| !p.starts_with('/') || p.len() > 1024 || p.chars().any(char::is_control))
            {
                return Err("Link outcomes need bounded absolute virtual paths".into());
            }
            if let Requirement::Content { text, .. } = r
                && text.len() > core::MAX_FILE_BYTES
            {
                return Err("Link expected data exceeds file limit".into());
            }
        }
        for (world, requirements) in [
            (&self.initial, &self.requirements),
            (&self.example, &self.example_requirements),
        ] {
            for r in requirements {
                match r {
                    Requirement::Shared { left, right }
                    | Requirement::Independent { left, right }
                        if left == right =>
                    {
                        return Err("Link outcomes need different names".into());
                    }
                    Requirement::Preserved { path } | Requirement::Moved { source: path, .. }
                        if world.stat(path).is_err() =>
                    {
                        return Err("Retained link outcome must name an initial file".into());
                    }
                    _ => {}
                }
            }
        }
        if self.example_commands.iter().any(|s| s.len() > 4096) {
            return Err("Link example command exceeds limit".into());
        }
        let mut example = self.example.clone();
        for command in &self.example_commands {
            execute(&mut example, command)?;
        }
        if !self
            .example_requirements
            .iter()
            .all(|r| r.check(&self.example, &example).0)
        {
            return Err("Link worked example cannot reach its objectives".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        if s.example_command >= self.example_commands.len() {
            return Err("Unknown link practice command".into());
        }
        for (scope, initial) in [(&s.assignment, &self.initial), (&s.practice, &self.example)] {
            let mut replay = initial.clone();
            for action in &scope.history {
                apply(&mut replay, action)?;
            }
            if replay != scope.world {
                return Err("Saved link state differs from the authored inputs and actual edit history; prior work retained".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (scope, initial, requirements) = if practice {
            (&s.practice, &self.example, &self.example_requirements)
        } else {
            (&s.assignment, &self.initial, &self.requirements)
        };
        let mut rows = vec![];
        if scope.editor_loaded
            && scope
                .world
                .read_file(&scope.editor_open_path)
                .is_ok_and(|v| v != scope.editor_text)
        {
            rows.push((false,"File editor changes are not saved. Save or reopen the file before checking the configuration.".into()));
        }
        rows.extend(requirements.iter().map(|r| r.check(initial, &scope.world)));
        rows
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        input: String,
    },
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        text: String,
    },
}
fn apply(s: &mut core::State, action: &Action) -> Result<(), String> {
    match action {
        Action::Command { input } => execute(s, input).map(|_| ()),
        Action::Edit { path, text } => s.write_file(path, text).map_err(|e| e.to_string()),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,128>")]
    pub history: Vec<Action>,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,40>")]
    pub transcript: Vec<String>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub editor_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub editor_text: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub editor_open_path: String,
    pub editor_loaded: bool,
}
impl Scope {
    fn new(world: &core::State) -> Self {
        Self {
            world: world.clone(),
            history: vec![],
            command: String::new(),
            transcript: vec![],
            checked: false,
            editor_path: String::new(),
            editor_text: String::new(),
            editor_open_path: String::new(),
            editor_loaded: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub example_command: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            example_command: 0,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.example_command >= 12
            || self.version != 1
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err(
                "Link workspace exceeds its version or256KiB limit; prior work retained".into(),
            );
        }
        for s in [&self.assignment, &self.practice] {
            s.world.validate().map_err(|e| e.to_string())?;
            if s.history.len() > 128
                || s.command.len() > 4096
                || s.transcript.len() > 40
                || s.transcript.iter().any(|v| v.len() > 8192)
                || s.editor_open_path.len() > 1024
                || s.editor_path.len() > 1024
                || s.editor_text.len() > 8192
            {
                return Err("Link workspace text/history limit exceeded".into());
            }
        }
        Ok(())
    }
    pub fn run(&mut self, d: &Definition, input: &str, practice: bool) -> Result<String, String> {
        d.bound(self)?;
        let mut next = self.clone();
        let scope = if practice {
            &mut next.practice
        } else {
            &mut next.assignment
        };
        let before = scope.world.clone();
        let output = execute(&mut scope.world, input)?;
        if scope.world != before {
            if scope.history.len() == 128 {
                return Err(
                    "Saved edit-history limit reached; reset this attempt to start again".into(),
                );
            }
            scope.history.push(Action::Command {
                input: input.into(),
            });
        }
        scope.checked = false;
        next.validate()?;
        *self = next;
        Ok(output)
    }
    pub fn save_editor(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        let scope = if practice {
            &mut next.practice
        } else {
            &mut next.assignment
        };
        if !scope.editor_loaded || scope.editor_path != scope.editor_open_path {
            return Err("Open an existing simulated file before saving".into());
        }
        if scope.history.len() == 128 {
            return Err("Saved edit-history limit reached".into());
        }
        scope
            .world
            .write_file(&scope.editor_open_path, &scope.editor_text)
            .map_err(|e| e.to_string())?;
        scope.history.push(Action::Edit {
            path: scope.editor_open_path.clone(),
            text: scope.editor_text.clone(),
        });
        scope.checked = false;
        next.validate()?;
        *self = next;
        Ok(())
    }
}
/// Candidate narrow non-TTY inode-column output; exact GNU spacing still needs
/// the separately approved fixed comparison. No host inode IDs are invented.
fn ls_inode_text(world: &core::State, path: &str) -> Result<String, String> {
    let rows = world.directory_inode_entries(path).map_err(|e| {
        if e.kind == core::ErrorKind::NotDirectory {
            "Simulated ls -i accepts directory operands only; file operands are outside this subset.".into()
        } else {
            format!("ls: {e}")
        }
    })?;
    let visible = rows.iter().filter(|(name, _)| !name.starts_with('.'));
    let width = visible.clone().map(|(_, inode)| inode.to_string().len()).max().unwrap_or(0);
    let mut text = String::new();
    for (name, inode) in visible {
        let line = format!("{inode:>width$} {name}\n");
        if line.len() > core::MAX_OUTPUT_BYTES.saturating_sub(text.len()) {
            return Err("ls: Inode listing exceeds32KiB; no state changed.".into());
        }
        text.push_str(&line);
    }
    Ok(text)
}
pub fn execute(world: &mut core::State, input: &str) -> Result<String, String> {
    let words = crate::shell::literal_command(input)?;
    let args = words.iter().map(String::as_str).collect::<Vec<_>>();
    let Some((command, mut rest)) = args.split_first() else {
        return Ok(String::new());
    };
    let mut ended = rest.first() == Some(&"--");
    if ended {
        rest = &rest[1..];
    }
    let mut flag = "";
    if !ended
        && matches!(
            (*command, rest.first()),
            ("ln", Some(&"-s")) | ("stat", Some(&"-L")) | ("ls", Some(&"-i"))
        )
    {
        flag = rest[0];
        rest = &rest[1..];
        if rest.first() == Some(&"--") {
            ended = true;
            rest = &rest[1..];
        }
    }
    if !ended && rest.iter().any(|s| s.starts_with('-')) {
        return Err("Unsupported option: use only the disclosed flags, or -- before literal names beginning with a dash".into());
    }
    if *command == "ls" && flag == "-i" {
        let directory = match rest {
            [] => ".",
            [path] => path,
            _ => return Err("Simulated ls -i accepts at most one directory; other operands/formats are unsupported.".into()),
        };
        return ls_inode_text(world, directory);
    }
    let result: Result<String, core::Error> = match (*command, flag, rest) {
        ("help", "", []) => Ok(HELP.into()),
        ("pwd", "", []) => world.cwd().map(|s| format!("{s}\n")),
        ("ls", "", []) => world
            .directory_entries(".")
            .map(|n| format!("{}\n", n.join("\n"))),
        ("ls", "", [path]) => world
            .directory_entries(path)
            .map(|n| format!("{}\n", n.join("\n"))),
        ("cat", "", [path]) => world.read_file(path),
        ("readlink", "", [path]) => world.read_link(path).map(|s| format!("{s}\n")),
        ("stat", "", [path]) => world.stat(path).map(|s| stat_text(path, &s)),
        ("stat", "-L", [path]) => world.stat_follow(path).map(|s| stat_text(path, &s)),
        ("ln", "-s", [target, destination]) => world
            .symbolic_link(target, destination)
            .map(|_| String::new()),
        ("ln", "", [source, destination]) => {
            world.hard_link(source, destination).map(|_| String::new())
        }
        ("cp", "", [source, destination]) => {
            world.copy_file(source, destination).map(|_| String::new())
        }
        ("mv", "", [source, destination]) => {
            world.move_entry(source, destination).map(|_| String::new())
        }
        ("rm", "", [path]) => world.unlink(path).map(|_| String::new()),
        _ => {
            return Err(format!(
                "Unsupported command, option or argument count. {HELP}"
            ));
        }
    };
    result.map_err(|e| format!("{command}: {e}"))
}
fn stat_text(path: &str, s: &core::Stat) -> String {
    format!(
        "  File: {path}\n  Type: {}\n  Inode: {}   Links: {}\n  Size: {}\n",
        match s.kind {
            core::Kind::Directory => "directory",
            core::Kind::Regular => "regular file",
            core::Kind::SymbolicLink => "symbolic link",
        },
        s.inode,
        s.links,
        s.size_bytes
            .map(|n| n.to_string())
            .unwrap_or_else(|| "not modeled for directories".into())
    )
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (crate::engine::State::Links { simulation }, crate::engine::Goal::Links { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Link task and state must agree".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err("Link task must start from its declared independent worlds".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::Links { simulation } = &a.state else {
        return Err("Saved link work belongs to a different workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible link attempt metadata; original retained".into());
    }
    d.bound(simulation)
}
