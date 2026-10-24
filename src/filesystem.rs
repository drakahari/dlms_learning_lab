//! Bounded Unix file model. All paths name in-memory entries, never host files.
mod find;
mod permissions;
use crate::engine::{self, Node};
pub use find::{FindDiagnostic, FindDiagnosticKind, FindReport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const MAX_ENTRIES: usize = 256;
const MAX_DATA: usize = 262_144;
const MAX_OUTPUT: usize = 32_768;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tree {
    pub nodes: BTreeMap<String, Node>,
    pub files: BTreeMap<String, String>,
    pub users: BTreeMap<String, Vec<String>>,
    pub user: String,
    pub home: String,
    pub cwd: String,
    #[serde(default)]
    pub previous: Option<String>,
    pub umask: u16,
    #[serde(default)]
    pub clock: u64,
    #[serde(default)]
    pub modified: BTreeMap<String, u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Entry {
        path: String,
        directory: bool,
    },
    Contents {
        path: String,
        text: String,
    },
    Absent {
        path: String,
    },
    Directory {
        path: String,
    },
    Mode {
        path: String,
        mode: u16,
    },
    Mask {
        mask: u16,
    },
    Ownership {
        path: String,
        owner: String,
        group: String,
    },
    Access {
        path: String,
        user: String,
        operation: Access,
        allowed: bool,
    },
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    ReadFile,
    WriteFile,
    ListNames,
    SearchDirectory,
    CreateEntry,
}
impl Access {
    pub(crate) fn directory(self) -> bool {
        !matches!(self, Self::ReadFile | Self::WriteFile)
    }
    fn mask(self) -> u16 {
        match self {
            Self::ReadFile | Self::ListNames => 4,
            Self::WriteFile => 2,
            Self::SearchDirectory => 1,
            Self::CreateEntry => 3,
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::ReadFile => "read the file",
            Self::WriteFile => "write the file",
            Self::ListNames => "list names in the directory",
            Self::SearchDirectory => "search the directory",
            Self::CreateEntry => "create entries in the directory",
        }
    }
}
fn canonical(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= 1024
        && path.split('/').filter(|p| !p.is_empty()).count() <= 64
        && !path.chars().any(char::is_control)
        && (path == "/"
            || (!path.ends_with('/')
                && path[1..]
                    .split('/')
                    .all(|p| !p.is_empty() && p != "." && p != ".." && p.len() <= 255)))
}
fn parent(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}
fn child(dir: &str, name: &str) -> String {
    format!(
        "{}{name}",
        if dir == "/" {
            "/".into()
        } else {
            format!("{dir}/")
        }
    )
}
#[derive(Debug)]
enum PathError {
    Missing(String),
    NotDirectory(String),
    Invalid(String),
    Permission(String),
}
impl From<PathError> for String {
    fn from(value: PathError) -> Self {
        match value {
            PathError::Missing(path) => format!("No such file or directory: {path}"),
            PathError::NotDirectory(path) => format!("Not a directory: {path}"),
            PathError::Invalid(message) | PathError::Permission(message) => message,
        }
    }
}
impl Requirement {
    pub fn valid(&self) -> bool {
        if let Self::Mask { mask } = self {
            return *mask <= 0o777;
        }
        let path = match self {
            Self::Entry { path, .. }
            | Self::Contents { path, .. }
            | Self::Absent { path }
            | Self::Directory { path }
            | Self::Mode { path, .. }
            | Self::Access { path, .. }
            | Self::Ownership { path, .. } => path,
            Self::Mask { .. } => unreachable!(),
        };
        canonical(path)
            && match self {
                Self::Contents { text, .. } => text.len() <= 131_072 && !text.contains('\0'),
                Self::Mode { mode, .. } => *mode <= 0o777,
                Self::Ownership { owner, group, .. } => [owner, group].iter().all(|s| {
                    !s.is_empty()
                        && s.len() <= 32
                        && s.bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                }),
                Self::Access { user, .. } => {
                    !user.is_empty()
                        && user.len() <= 32
                        && user != "root"
                        && user
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                }
                _ => true,
            }
    }
    pub fn valid_for(&self, tree: &Tree) -> bool {
        self.valid()
            && match self {
                Self::Access { user, .. } => tree.users.contains_key(user),
                Self::Ownership { owner, group, .. } => {
                    tree.users.contains_key(owner) && tree.users.values().any(|g| g.contains(group))
                }
                _ => true,
            }
    }
    pub fn check(&self, tree: &Tree) -> (bool, String) {
        match self {
            Self::Ownership { path, owner, group } => (
                self.valid_for(tree)
                    && tree
                        .nodes
                        .get(path)
                        .is_some_and(|n| n.owner == *owner && n.group == *group),
                format!("{path} belongs to {owner}, group {group}"),
            ),
            Self::Access {
                path,
                user,
                operation,
                allowed,
            } => {
                let applicable = self.valid_for(tree)
                    && tree
                        .nodes
                        .get(path)
                        .is_some_and(|n| n.directory == operation.directory());
                let actual = tree.access(user, path, *operation);
                (
                    applicable && actual == *allowed,
                    format!(
                        "{user} {} {}: {path}",
                        if *allowed { "can" } else { "cannot" },
                        operation.description()
                    ),
                )
            }
            Self::Mask { mask } => (tree.umask == *mask, format!("Creation mask is {mask:03o}")),
            Self::Entry { path, directory } => (
                tree.nodes
                    .get(path)
                    .is_some_and(|n| n.directory == *directory),
                format!(
                    "{path} exists as a {}",
                    if *directory { "directory" } else { "file" }
                ),
            ),
            Self::Contents { path, text } => (
                tree.files.get(path) == Some(text),
                format!("{path} contains the required text"),
            ),
            Self::Absent { path } => (
                !tree.nodes.contains_key(path),
                format!("{path} has been removed"),
            ),
            Self::Directory { path } => (tree.cwd == *path, format!("Working directory is {path}")),
            Self::Mode { path, mode } => (
                tree.nodes.get(path).is_some_and(|n| n.mode == *mode),
                format!("{path} permissions are {mode:03o}"),
            ),
        }
    }
}
impl Tree {
    pub fn valid(&self) -> bool {
        self.user != "root"
            && self.users.contains_key(&self.user)
            && !self.users.is_empty()
            && self.users.len() <= 16
            && self.users.iter().all(|(n, groups)| {
                !n.is_empty()
                    && n.len() <= 32
                    && n.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                    && !groups.is_empty()
                    && groups.len() <= 16
                    && groups.iter().all(|g| {
                        !g.is_empty()
                            && g.len() <= 32
                            && g.bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                    })
            })
            && self.nodes.len() <= MAX_ENTRIES
            && self.nodes.get("/").is_some_and(|n| n.directory)
            && self.nodes.get(&self.home).is_some_and(|n| n.directory)
            && self.nodes.get(&self.cwd).is_some_and(|n| n.directory)
            && self.previous.as_ref().is_none_or(|p| canonical(p))
            && self.nodes.iter().all(|(p, n)| {
                canonical(p)
                    && n.mode <= 0o777
                    && self.users.contains_key(&n.owner)
                    && self.users.values().any(|g| g.contains(&n.group))
                    && (p == "/" || self.nodes.get(parent(p)).is_some_and(|n| n.directory))
                    && (n.directory || self.files.contains_key(p))
            })
            && self.files.iter().all(|(p, t)| {
                self.nodes.get(p).is_some_and(|n| !n.directory)
                    && t.len() <= 131_072
                    && !t.contains('\0')
            })
            && self.files.values().map(String::len).sum::<usize>() <= MAX_DATA
            && self.umask <= 0o777
            && self.clock <= 1_000_000
            && self
                .modified
                .iter()
                .all(|(p, t)| self.nodes.contains_key(p) && *t <= self.clock)
    }
    fn allows(&self, user: &str, path: &str, mask: u16) -> bool {
        let Some(groups) = self.users.get(user) else {
            return false;
        };
        self.nodes.get(path).is_some_and(|node| {
            let shift = if node.owner == user {
                6
            } else if groups.contains(&node.group) {
                3
            } else {
                0
            };
            ((node.mode >> shift) & mask) == mask
        })
    }
    pub fn access(&self, user: &str, path: &str, operation: Access) -> bool {
        if user == "root"
            || !canonical(path)
            || !self.users.contains_key(user)
            || !self
                .nodes
                .get(path)
                .is_some_and(|n| n.directory == operation.directory())
        {
            return false;
        }
        let mut ancestor = parent(path);
        if path != "/" {
            loop {
                if !self.nodes.get(ancestor).is_some_and(|n| n.directory)
                    || !self.allows(user, ancestor, 1)
                {
                    return false;
                }
                if ancestor == "/" {
                    break;
                }
                ancestor = parent(ancestor);
            }
        }
        self.allows(user, path, operation.mask())
    }
    fn permitted(&self, path: &str, mask: u16) -> Result<(), String> {
        if self.allows(&self.user, path, mask) {
            Ok(())
        } else {
            Err(format!(
                "Permission denied: {path}. Check directory search permission and the access this operation needs."
            ))
        }
    }
    fn resolve(&self, path: &str, missing_leaf: bool) -> Result<String, PathError> {
        if path.is_empty() {
            return Err(PathError::Invalid(
                "Empty path: enter a file or directory name.".into(),
            ));
        }
        if path.len() > 1024 || path.chars().any(char::is_control) {
            return Err(PathError::Invalid(
                "Unsupported path: at most 1,024 bytes and no control characters.".into(),
            ));
        }
        let parts: Vec<_> = path.split('/').filter(|p| !p.is_empty()).collect();
        if parts.len() > 64 || parts.iter().any(|p| p.len() > 255) {
            return Err(PathError::Invalid(
                "Path exceeds simulator component limits.".into(),
            ));
        }
        let mut current = if path.starts_with('/') {
            "/".into()
        } else {
            self.cwd.clone()
        };
        for (i, part) in parts.iter().enumerate() {
            let node = self
                .nodes
                .get(&current)
                .ok_or_else(|| PathError::Missing(current.clone()))?;
            if !node.directory {
                return Err(PathError::NotDirectory(current));
            }
            self.permitted(&current, 1).map_err(PathError::Permission)?;
            current = match *part {
                "." => current,
                ".." => parent(&current).into(),
                part => child(&current, part),
            };
            if (!missing_leaf || i + 1 < parts.len() || path.ends_with('/'))
                && !self.nodes.contains_key(&current)
            {
                return Err(PathError::Missing(current));
            }
        }
        if path.ends_with('/') && !self.nodes.get(&current).is_some_and(|n| n.directory) {
            return Err(PathError::NotDirectory(current));
        }
        Ok(current)
    }
    fn changed(&mut self, path: &str) {
        self.clock += 1;
        self.modified.insert(path.into(), self.clock);
    }
    fn new_node(&mut self, path: &str, directory: bool, mode: u16) -> Result<(), String> {
        if self.nodes.len() >= MAX_ENTRIES {
            return Err("Virtual entry limit reached (256).".into());
        }
        self.permitted(parent(path), 3)?;
        self.nodes.insert(
            path.into(),
            Node {
                mode,
                owner: self.user.clone(),
                group: self.users[&self.user][0].clone(),
                directory,
            },
        );
        if !directory {
            self.files.insert(path.into(), String::new());
        }
        self.changed(path);
        self.changed(parent(path));
        Ok(())
    }
    fn destination(&self, source: &str, arg: &str) -> Result<String, String> {
        let mut dest = self.resolve(arg, true)?;
        if self.nodes.get(&dest).is_some_and(|n| n.directory) {
            self.permitted(&dest, 1)?;
            dest = child(&dest, source.rsplit('/').next().unwrap());
        }
        if dest == source {
            return Err("Source and destination identify the same file; nothing changed.".into());
        }
        if self.nodes.get(&dest).is_some_and(|n| n.directory) {
            return Err(format!(
                "Destination is a directory: {dest}. Recursive copying/moving is unsupported."
            ));
        }
        Ok(dest)
    }
    /// Transactional bounded commands: no host I/O, no executable script dispatch.
    pub fn execute(&mut self, input: &str) -> Result<String, String> {
        if !self.valid() {
            return Err("Invalid virtual filesystem state; no command executed.".into());
        }
        let words = crate::shell::literal_command(input)?;
        // Read-only search retains partial results on traversal errors and does
        // not enter generic option stripping or clone file contents.
        if words.first().is_some_and(|word| word == "find") {
            return self.find(&words[1..])?.terminal();
        }
        let mut candidate = self.clone();
        let output = candidate.command(&words)?;
        if !candidate.valid() {
            return Err(
                "Virtual filesystem resource limit reached; command was not applied.".into(),
            );
        }
        if output.len() > MAX_OUTPUT {
            return Err(
                "Output limit reached (32 KiB); inspect a smaller file or directory.".into(),
            );
        }
        *self = candidate;
        Ok(output)
    }
    fn command(&mut self, words: &[String]) -> Result<String, String> {
        let Some(cmd) = words.first().map(String::as_str) else {
            return Ok(String::new());
        };
        let mut args = &words[1..];
        let mut ended_options = args.first().is_some_and(|s| s == "--");
        if ended_options {
            args = &args[1..];
        }
        let unsupported = || {
            format!(
                "Unsupported {cmd} command or arguments. Use help for this virtual filesystem’s supported subset; nothing ran on your computer."
            )
        };
        match cmd {
            "pwd" if args.is_empty() || matches!(args, [a] if a == "-P" || a == "-L") => {
                Ok(format!("{}\n", self.cwd))
            }
            "cd" if args.len() <= 1 => {
                if !ended_options && args.first().is_some_and(|s| s.starts_with('-') && s != "-") {
                    return Err(unsupported());
                }
                let previous = args.first().is_some_and(|s| s == "-");
                let target = if args.is_empty() {
                    self.home.clone()
                } else if previous {
                    self.previous
                        .clone()
                        .ok_or("OLDPWD is not set; visit another directory first.")?
                } else {
                    args[0].clone()
                };
                let path = self.resolve(&target, false)?;
                if !self.nodes[&path].directory {
                    return Err(format!("Not a directory: {path}"));
                }
                self.permitted(&path, 1)?;
                self.previous = Some(std::mem::replace(&mut self.cwd, path));
                Ok(if previous {
                    format!("{}\n", self.cwd)
                } else {
                    String::new()
                })
            }
            "ls" => self.list(&words[1..]),
            "id" => {
                let names = !ended_options
                    && matches!(args.first().map(String::as_str), Some("-Gn" | "-nG"));
                let operands = if names { &args[1..] } else { args };
                if operands.len() > 1
                    || operands
                        .first()
                        .is_some_and(|s| s.starts_with('-') && !ended_options)
                {
                    return Err(unsupported());
                }
                let user = operands.first().unwrap_or(&self.user);
                let groups = self
                    .users
                    .get(user)
                    .ok_or_else(|| format!("Unknown simulated user: {user}"))?;
                let mut seen = std::collections::BTreeSet::new();
                let groups: Vec<_> = groups.iter().filter(|g| seen.insert(*g)).collect();
                if names {
                    return Ok(format!(
                        "{}\n",
                        groups
                            .iter()
                            .map(|g| g.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                let primary = groups[0];
                Ok(format!(
                    "uid={}({user}) gid={}({primary}) groups={}\n",
                    engine::user_id(&self.users, user),
                    engine::group_id(&self.users, primary),
                    groups
                        .iter()
                        .map(|g| format!("{}({g})", engine::group_id(&self.users, g)))
                        .collect::<Vec<_>>()
                        .join(",")
                ))
            }
            "chmod" | "chgrp" if args.len() == 2 => {
                if (!ended_options && (args[0].starts_with('-') || args[1].starts_with('-')))
                    || args[0].is_empty()
                {
                    return Err(unsupported());
                }
                let path = self.resolve(&args[1], false)?;
                if self.nodes[&path].owner != self.user {
                    return Err(format!(
                        "Permission denied: only the owner can change {path} in this ordinary-user simulation."
                    ));
                }
                if cmd == "chmod" {
                    let new = permissions::mode(&args[0], self.nodes[&path].mode)?;
                    self.nodes.get_mut(&path).unwrap().mode = new;
                } else {
                    if !self.users.values().any(|gs| gs.contains(&args[0])) {
                        return Err(format!("Unknown simulated group: {}", args[0]));
                    }
                    if !self.users[&self.user].contains(&args[0]) {
                        return Err(format!(
                            "Permission denied: {} is not a member of group {}.",
                            self.user, args[0]
                        ));
                    }
                    self.nodes.get_mut(&path).unwrap().group = args[0].clone();
                }
                self.changed(&path);
                Ok(String::new())
            }

            "umask" if args.len() <= 1 => {
                if let Some(mask) = args.first() {
                    if mask.is_empty()
                        || mask.len() > 4
                        || !mask.bytes().all(|b| (b'0'..=b'7').contains(&b))
                    {
                        return Err(
                            "Use an octal mask such as 022; symbolic umask is unsupported.".into(),
                        );
                    }
                    self.umask = u16::from_str_radix(mask, 8)
                        .ok()
                        .filter(|n| *n <= 0o777)
                        .ok_or("Mask must be between 000 and 777")?;
                    return Ok(String::new());
                }
                Ok(format!("{:04o}\n", self.umask))
            }
            "cat" if args.len() == 1 => {
                if args[0] == "-" {
                    return Err("Reading terminal input with cat - is unsupported; use ./- for a file named -.".into());
                }
                if args[0].starts_with('-') && !ended_options {
                    return Err(unsupported());
                }
                let path = self.resolve(&args[0], false)?;
                if self.nodes[&path].directory {
                    return Err(format!("Is a directory: {path}"));
                }
                self.permitted(&path, 4)?;
                Ok(self.files[&path].clone())
            }
            "mkdir" | "touch" if args.len() == 1 => {
                if args[0].starts_with('-') && !ended_options {
                    return Err(unsupported());
                }
                if cmd == "touch" && args[0] == "-" {
                    return Err("touch - uses an output file descriptor on Linux, which this model does not support; use ./- for a file named -.".into());
                }
                let operand = if cmd == "mkdir" {
                    args[0].trim_end_matches('/')
                } else {
                    &args[0]
                };
                let operand = if operand.is_empty() && args[0].starts_with('/') {
                    "/"
                } else {
                    operand
                };
                let path = self.resolve(operand, true)?;
                if let Some(node) = self.nodes.get(&path) {
                    if cmd == "mkdir" {
                        return Err(format!("File exists: {path}"));
                    }
                    if node.owner != self.user {
                        self.permitted(&path, 2)?;
                    }
                    self.changed(&path);
                    return Ok(String::new());
                }
                let directory = cmd == "mkdir";
                self.new_node(
                    &path,
                    directory,
                    (if directory { 0o777 } else { 0o666 }) & !self.umask,
                )?;
                Ok(String::new())
            }
            "cp" | "mv" if args.len() == 2 => {
                if !ended_options && args.iter().any(|s| s.starts_with('-')) {
                    return Err(unsupported());
                }
                let source = self.resolve(&args[0], false)?;
                if self.nodes[&source].directory {
                    return Err("Only ordinary-file copy/move is supported; directory recursion is not modeled.".into());
                }
                let dest = self.destination(&source, &args[1])?;
                if cmd == "cp" {
                    self.permitted(&source, 4)?;
                    if self.nodes.contains_key(&dest) {
                        self.permitted(&dest, 2)?;
                    } else {
                        self.new_node(&dest, false, self.nodes[&source].mode & !self.umask)?;
                    }
                    self.files.insert(dest.clone(), self.files[&source].clone());
                    self.changed(&dest);
                } else {
                    self.permitted(parent(&source), 3)?;
                    self.permitted(parent(&dest), 3)?;
                    let node = self.nodes.remove(&source).unwrap();
                    let text = self.files.remove(&source).unwrap();
                    self.nodes.insert(dest.clone(), node);
                    self.files.insert(dest.clone(), text);
                    if let Some(time) = self.modified.remove(&source) {
                        self.modified.insert(dest.clone(), time);
                    } else {
                        self.modified.remove(&dest);
                    }
                    self.changed(parent(&source));
                    self.changed(parent(&dest));
                }
                Ok(String::new())
            }
            "rm" | "rmdir" => {
                let force =
                    !ended_options && cmd == "rm" && args.first().is_some_and(|s| s == "-f");
                if force {
                    args = &args[1..];
                    if args.first().is_some_and(|s| s == "--") {
                        args = &args[1..];
                        ended_options = true;
                    }
                }
                if args.len() != 1 || (args[0].starts_with('-') && !ended_options) {
                    return Err(unsupported());
                }
                let path = match self.resolve(&args[0], force) {
                    Ok(path) => path,
                    Err(PathError::Missing(_) | PathError::NotDirectory(_)) if force => {
                        return Ok(String::new());
                    }
                    Err(error) => return Err(error.into()),
                };
                let Some(node) = self.nodes.get(&path) else {
                    return Ok(String::new());
                };
                if path == "/" {
                    return Err("The virtual root cannot be removed.".into());
                }
                if node.directory != (cmd == "rmdir") {
                    return Err(if node.directory {
                        "Is a directory; recursive rm is unsupported."
                    } else {
                        "Not a directory."
                    }
                    .into());
                }
                if node.directory
                    && (self.cwd == path
                        || self.cwd.starts_with(&format!("{path}/"))
                        || self.home == path)
                {
                    return Err("Removing a current/home directory is unsupported in this persistent model.".into());
                }
                if node.directory
                    && self
                        .nodes
                        .keys()
                        .any(|p| p != &path && p.starts_with(&format!("{path}/")))
                {
                    return Err(format!("Directory not empty: {path}"));
                }
                self.permitted(parent(&path), 3)?;
                self.nodes.remove(&path);
                self.files.remove(&path);
                self.modified.remove(&path);
                self.changed(parent(&path));
                Ok(String::new())
            }
            _ => Err(unsupported()),
        }
    }
    fn list(&self, args: &[String]) -> Result<String, String> {
        let mut all = false;
        let mut long = false;
        let mut directory = false;
        let mut operands = Vec::new();
        let mut options = true;
        for arg in args {
            if options && arg == "--" {
                options = false;
                continue;
            }
            if options && arg.starts_with('-') && arg != "-" {
                for flag in arg[1..].chars() {
                    match flag {
                        'a' => all = true,
                        'l' => long = true,
                        'd' => directory = true,
                        _ => {
                            return Err(format!(
                                "Unsupported ls option: {flag}. Supported: -a, -l, -d and --."
                            ));
                        }
                    }
                }
            } else {
                operands.push(arg.as_str());
            }
        }
        if operands.len() > 1 {
            return Err("ls supports one path at a time in this model.".into());
        }
        let arg = operands.first().copied().unwrap_or(".");
        let path = self.resolve(arg, false)?;
        let node = &self.nodes[&path];
        let mut entries: Vec<(String, String)> = Vec::new();
        if node.directory && !directory {
            self.permitted(&path, if long { 5 } else { 4 })?;
            if all {
                entries.push((".".into(), path.clone()));
                entries.push(("..".into(), parent(&path).into()));
            }
            for p in self
                .nodes
                .keys()
                .filter(|p| p.as_str() != "/" && parent(p) == path)
            {
                let name = p.rsplit('/').next().unwrap();
                if all || !name.starts_with('.') {
                    entries.push((name.into(), p.clone()));
                }
            }
        } else {
            entries.push((arg.into(), path));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let rows: Vec<_> = entries
            .iter()
            .map(|(label, p)| {
                if !long {
                    return label.clone();
                }
                let n = &self.nodes[p];
                let size = self.files.get(p).map_or(0, String::len);
                let tick = self.modified.get(p).copied().unwrap_or(0);
                let links = if n.directory {
                    2 + self
                        .nodes
                        .iter()
                        .filter(|(q, n)| q.as_str() != "/" && parent(q) == p && n.directory)
                        .count()
                } else {
                    1
                };
                format!(
                    "{} {links} {} {} {size:>5} Jan {:>2} {:02}:{:02} {label}",
                    engine::mode_string(n),
                    n.owner,
                    n.group,
                    1 + tick / 86400,
                    (tick / 3600) % 24,
                    (tick / 60) % 60
                )
            })
            .collect();
        Ok(if rows.is_empty() {
            String::new()
        } else {
            format!("{}\n", rows.join("\n"))
        })
    }
}
pub const HELP: &str = "pwd [-L|-P] • cd [PATH|-] • ls [-a|-l|-d] [PATH] • cat PATH • mkdir PATH • touch PATH • cp FILE DEST • mv FILE DEST • rm [-f] FILE • rmdir EMPTY_DIR • umask [OCTAL] • chmod MODE PATH • chgrp GROUP PATH • id [-Gn|-nG] [USER]. chmod: 000–777 or explicit u/g/o/a +-= r/w/x clauses. Owner-only changes; chgrp requires membership; synthetic account IDs. -- ends options; quote spaces. find START [-maxdepth N] [-name QUOTED_PATTERN|-type f|d ...] [-print] searches one virtual tree; quote * and ?. No file mutation by find; readable nonsearchable directory descent is unsupported and reported incomplete. -- applies to the older commands, not find. No expansion, pipelines, redirection, recursive mutation or host access. show is a simulator helper.";
