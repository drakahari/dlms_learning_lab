//! Opt-in bounded restricted-deletion world. No host I/O or source authority.
//! The later caller-held lesson policy must bind roles and mutable slots.
use super::{Access, PathError, Tree, canonical, parent, permissions};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt};

pub const MAX_NODES: usize = 64;
pub const MAX_ORDINARY_USERS: usize = 4;
pub const MAX_USERS: usize = MAX_ORDINARY_USERS + 1; // Optional root metadata owner.
pub const MAX_STICKY: usize = 2;
pub const MAX_TEXT: usize = 131_072;
pub const MAX_RAW: usize = 1_048_576;
pub const MAX_COMMAND: usize = 4_096;
pub const HELP: &str = "Simulated directory policy: pwd [-L|-P], cd [PATH|-], id [-Gn|-nG] [USER], ls [-a|-l|-d] [PATH], cat FILE; chmod MODE PATH; touch FILE; cp FILE DEST; rm [-f] FILE; mv FILE DEST. -- ends options; literal quoting is supported. chmod: ordinary 000–777/explicit-class rwx, +t/-t or numeric 1000–1777 on directories only (one optional leading zero). Regular-file rename may replace an existing regular file only after both parents' write/search and both applicable sticky checks. No creating, removing or moving directories, links, root/capabilities, SUID/SGID, shell composition or host commands. The actor is the supplied ordinary user; participant probes run only on a discarded in-memory copy.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CoreWorld {
    pub tree: Tree,
    #[serde(deserialize_with = "sticky_paths")]
    pub sticky: BTreeSet<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Permission,
    Missing,
    Operation,
    Resource,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    let mut message = message.into();
    if message.len() > 1024 {
        let mut end = 1024;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
    }
    Error { kind, message }
}
fn unsupported() -> Error {
    error(
        ErrorKind::Unsupported,
        "Unsupported command or syntax in this directory-policy activity; use help. Nothing changed.",
    )
}
fn path_error(e: PathError) -> Error {
    let kind = match &e {
        PathError::Permission(_) => ErrorKind::Permission,
        PathError::Missing(_) => ErrorKind::Missing,
        PathError::NotDirectory(_) => ErrorKind::Operation,
        PathError::Invalid(_) => ErrorKind::Unsupported,
    };
    error(kind, String::from(e))
}
fn sticky_paths<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<String>, D::Error> {
    struct Paths;
    impl<'de> Visitor<'de> for Paths {
        type Value = BTreeSet<String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("at most two distinct canonical directory paths")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeSet::new();
            while let Some(p) = seq.next_element::<String>()? {
                if out.len() == MAX_STICKY || !canonical(&p) || !out.insert(p) {
                    return Err(de::Error::custom(
                        "Invalid, duplicate or excessive sticky directory path",
                    ));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Paths)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub succeeded: bool,
    pub failure_kind: Option<ErrorKind>,
    pub output: String,
    pub diagnostic: String,
}
impl CoreWorld {
    pub fn new(tree: Tree, sticky: BTreeSet<String>) -> Result<Self, Error> {
        let world = Self { tree, sticky };
        world.validate()?;
        Ok(world)
    }
    /// Raw-size admission is required before the existing generic Tree reader.
    /// Serde may allocate bounded raw-sized scratch; this is not preallocation-free.
    pub fn decode(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > MAX_RAW {
            return Err(error(
                ErrorKind::Resource,
                "Directory-policy data exceeds 1 MiB",
            ));
        }
        let world: Self = serde_json::from_slice(raw).map_err(|e| {
            error(
                ErrorKind::Invalid,
                format!("Invalid directory-policy data: {e}"),
            )
        })?;
        world.validate()?;
        Ok(world)
    }
    /// Structural admission only. Imported data cannot certify lesson authority.
    pub fn validate(&self) -> Result<(), Error> {
        if self.tree.clock > 1_000_000
            || self.tree.nodes.len() > MAX_NODES
            || self.tree.users.len() > MAX_USERS
            || self
                .tree
                .users
                .keys()
                .filter(|name| name.as_str() != "root")
                .count()
                > MAX_ORDINARY_USERS
            || self.sticky.len() > MAX_STICKY
            || self
                .tree
                .files
                .values()
                .try_fold(0usize, |n, s| n.checked_add(s.len()))
                .is_none_or(|n| n > MAX_TEXT)
        {
            return Err(error(
                ErrorKind::Resource,
                "Directory-policy world exceeds its node, user, text or sticky-directory limit",
            ));
        }
        if self.tree.command_location.is_some()
            || !self.tree.valid()
            || self
                .sticky
                .iter()
                .any(|p| !canonical(p) || !self.tree.nodes.get(p).is_some_and(|n| n.directory))
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid directory-policy world or mixed activity context",
            ));
        }
        Ok(())
    }
    pub fn effective_mode(&self, path: &str) -> Result<u16, Error> {
        self.validate()?;
        let n = self
            .tree
            .nodes
            .get(path)
            .ok_or_else(|| error(ErrorKind::Missing, "No such canonical virtual entry"))?;
        Ok(n.mode
            | if self.sticky.contains(path) {
                0o1000
            } else {
                0
            })
    }
    fn actor(&self, actor: &str) -> Result<(), Error> {
        if actor == "root" {
            return Err(error(
                ErrorKind::Unsupported,
                "Privileged users and capability overrides are outside this ordinary-user simulation",
            ));
        }
        if !self.tree.users.contains_key(actor) {
            return Err(error(ErrorKind::Invalid, "Unknown simulated participant"));
        }
        Ok(())
    }
    pub fn ordinary_access(
        &self,
        actor: &str,
        path: &str,
        operation: Access,
    ) -> Result<bool, Error> {
        self.validate()?;
        self.actor(actor)?;
        if !canonical(path) {
            return Err(error(
                ErrorKind::Unsupported,
                "Permission probes require a canonical virtual path",
            ));
        }
        Ok(self.tree.access(actor, path, operation))
    }
    /// Executes as tree.user. The later source-bound wrapper must also compare
    /// the candidate with the held allowed-change policy before publication.
    pub fn execute(&mut self, input: &str) -> Result<String, Error> {
        self.validate()?;
        self.actor(&self.tree.user)?;
        if input.len() > MAX_COMMAND {
            return Err(error(ErrorKind::Resource, "Command exceeds 4,096 bytes"));
        }
        let words =
            crate::shell::literal_command(input).map_err(|m| error(ErrorKind::Unsupported, m))?;
        let mut candidate = self.clone();
        let output = candidate.command(&words)?;
        candidate.validate()?;
        if output.len() > super::MAX_OUTPUT {
            return Err(error(
                ErrorKind::Resource,
                "Output exceeds 32 KiB; no command applied",
            ));
        }
        *self = candidate;
        Ok(output)
    }
    /// Real model execution on a bounded discarded clone, never host identities.
    /// Syntax/source/resource errors are errors, not permission-denial evidence.
    pub fn probe(&self, actor: &str, input: &str) -> Result<Probe, Error> {
        self.validate()?;
        self.actor(actor)?;
        if input.len() > MAX_COMMAND {
            return Err(error(
                ErrorKind::Resource,
                "Probe command exceeds 4,096 bytes",
            ));
        }
        let words =
            crate::shell::literal_command(input).map_err(|m| error(ErrorKind::Unsupported, m))?;
        if !words
            .first()
            .is_some_and(|s| s == "touch" || s == "cp" || s == "rm" || s == "mv")
        {
            return Err(error(
                ErrorKind::Unsupported,
                "Participant operation probes support touch/cp/rm/mv only; use ordinary_access for readonly permission questions",
            ));
        }
        let mut trial = self.clone();
        trial.tree.user = actor.into();
        match trial.execute(input) {
            Ok(output) => Ok(Probe { succeeded: true, failure_kind: None, output, diagnostic: "The simulated operation succeeded on a discarded copy; the working files did not change.".into() }),
            Err(e) if matches!(e.kind, ErrorKind::Permission | ErrorKind::Missing | ErrorKind::Operation) => Ok(Probe { succeeded: false, failure_kind: Some(e.kind), output: String::new(), diagnostic: e.message }),
            Err(e) => Err(e),
        }
    }
    fn parent_access(&self, path: &str) -> Result<(), Error> {
        let directory = parent(path);
        if self
            .tree
            .access(&self.tree.user, directory, Access::CreateEntry)
        {
            Ok(())
        } else {
            Err(error(
                ErrorKind::Permission,
                format!(
                    "Permission denied: {} needs write and search permission on {directory} and search permission on its ancestors",
                    self.tree.user
                ),
            ))
        }
    }
    fn sticky_access(&self, path: &str) -> Result<(), Error> {
        let directory = parent(path);
        if !self.sticky.contains(directory)
            || self.tree.nodes[path].owner == self.tree.user
            || self.tree.nodes[directory].owner == self.tree.user
        {
            Ok(())
        } else {
            Err(error(
                ErrorKind::Permission,
                format!(
                    "Restricted deletion: {} owns neither {path} nor its sticky parent {directory}; directory write/search alone cannot remove or replace this name",
                    self.tree.user
                ),
            ))
        }
    }
    fn regular(&self, path: &str) -> Result<(), Error> {
        if self.tree.nodes.get(path).is_some_and(|n| !n.directory) {
            Ok(())
        } else {
            Err(error(
                ErrorKind::Unsupported,
                "Only ordinary-file operations are supported; directory mutation, links and recursive operations are outside this activity",
            ))
        }
    }
    fn command(&mut self, words: &[String]) -> Result<String, Error> {
        let Some(cmd) = words.first().map(String::as_str) else {
            return Ok(String::new());
        };
        let mut args = &words[1..];
        let mut ended = args.first().is_some_and(|s| s == "--");
        if ended {
            args = &args[1..];
        }
        match cmd {
            "help" if args.is_empty() => Ok(HELP.into()),
            "ls" => self.list(&words[1..]),
            "pwd" if args.is_empty() || matches!(args, [a] if a == "-P" || a == "-L") => self
                .tree
                .command(words)
                .map_err(|m| error(ErrorKind::Operation, m)),
            "cd" if args.len() <= 1
                && (ended || args.first().is_none_or(|a| !a.starts_with('-') || a == "-")) =>
            {
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            "cat" if args.len() == 1 && args[0] != "-" && (ended || !args[0].starts_with('-')) => {
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            "id" => {
                let names = !ended && args.first().is_some_and(|a| a == "-Gn" || a == "-nG");
                let operands = if names { &args[1..] } else { args };
                if operands.len() > 1
                    || (!ended && operands.first().is_some_and(|a| a.starts_with('-')))
                {
                    return Err(unsupported());
                }
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            "chmod" if args.len() == 2 => {
                if !ended && args[1].starts_with('-') {
                    return Err(unsupported());
                }
                let path = self.tree.resolve(&args[1], false).map_err(path_error)?;
                let node = &self.tree.nodes[&path];
                if node.owner != self.tree.user {
                    return Err(error(
                        ErrorKind::Permission,
                        format!("Only the ordinary owner may change {path}"),
                    ));
                }
                let spec = args[0].as_str();
                let (mode, sticky) = if matches!(spec, "+t" | "-t") {
                    if !node.directory {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Sticky modes on regular files are outside this directory-only subset",
                        ));
                    }
                    (node.mode, spec == "+t")
                } else if !spec.is_empty() && spec.bytes().all(|b| matches!(b, b'0'..=b'7')) {
                    let numeric = u16::from_str_radix(spec, 8)
                        .ok()
                        .filter(|_| spec.len() <= 4 || (spec.len() == 5 && spec.starts_with('0')))
                        .filter(|n| *n <= 0o1777)
                        .ok_or_else(unsupported)?;
                    if numeric & 0o1000 != 0 && !node.directory {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Sticky modes on regular files are outside this directory-only subset",
                        ));
                    }
                    (numeric & 0o777, numeric & 0o1000 != 0)
                } else {
                    // Validate the entire unchanged ordinary grammar/count bound first.
                    permissions::mode(spec, node.mode)
                        .map_err(|m| error(ErrorKind::Unsupported, m))?;
                    let mut mode = node.mode;
                    let mut sticky = self.sticky.contains(&path);
                    for clause in spec.split(',') {
                        mode = permissions::mode(clause, mode)
                            .map_err(|m| error(ErrorKind::Unsupported, m))?;
                        // GNU '=' clears unmentioned sticky when selecting o/a.
                        // u=/g= leave it alone; ordinary +/- never mention it.
                        // Special-setting compound clauses remain unsupported.
                        if clause
                            .split_once('=')
                            .is_some_and(|(who, _)| who.bytes().any(|b| matches!(b, b'o' | b'a')))
                        {
                            sticky = false;
                        }
                    }
                    (mode, sticky)
                };
                self.tree.nodes.get_mut(&path).unwrap().mode = mode;
                if sticky {
                    self.sticky.insert(path.clone());
                } else {
                    self.sticky.remove(&path);
                }
                self.tree.changed(&path);
                Ok(String::new())
            }
            "touch" if args.len() == 1 => {
                if args[0] == "-" || (!ended && args[0].starts_with('-')) {
                    return Err(unsupported());
                }
                let path = self.tree.resolve(&args[0], true).map_err(path_error)?;
                if self.tree.nodes.contains_key(&path) {
                    self.regular(&path)?;
                    if self.tree.nodes[&path].owner != self.tree.user
                        && !self.tree.access(&self.tree.user, &path, Access::WriteFile)
                    {
                        return Err(error(
                            ErrorKind::Permission,
                            format!(
                                "Permission denied: changing this existing file's simulated timestamp needs ownership or write permission: {path}"
                            ),
                        ));
                    }
                } else {
                    self.parent_access(&path)?;
                }
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            "cp" if args.len() == 2 => {
                if !ended && args.iter().any(|s| s.starts_with('-')) {
                    return Err(unsupported());
                }
                let source = self.tree.resolve(&args[0], false).map_err(path_error)?;
                self.regular(&source)?;
                if !self.tree.access(&self.tree.user, &source, Access::ReadFile) {
                    return Err(error(
                        ErrorKind::Permission,
                        format!(
                            "Permission denied: reading the copy source needs file read permission: {source}"
                        ),
                    ));
                }
                let destination = self
                    .tree
                    .destination(&source, &args[1])
                    .map_err(|m| error(ErrorKind::Operation, m))?;
                if self.tree.nodes.contains_key(&destination) {
                    self.regular(&destination)?;
                    if !self
                        .tree
                        .access(&self.tree.user, &destination, Access::WriteFile)
                    {
                        return Err(error(
                            ErrorKind::Permission,
                            format!(
                                "Permission denied: overwriting existing file contents needs file write permission: {destination}"
                            ),
                        ));
                    }
                } else {
                    self.parent_access(&destination)?;
                }
                // Existing-file content overwrite does not remove its name:
                // sticky is not a file-content permission check.
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            "rm" => {
                let force = !ended && args.first().is_some_and(|s| s == "-f");
                if force {
                    args = &args[1..];
                    if args.first().is_some_and(|s| s == "--") {
                        args = &args[1..];
                        ended = true;
                    }
                }
                if args.len() != 1 || (!ended && args[0].starts_with('-')) {
                    return Err(unsupported());
                }
                let path = match self.tree.resolve(&args[0], force) {
                    Ok(p) => p,
                    Err(PathError::Missing(_) | PathError::NotDirectory(_)) if force => {
                        return Ok(String::new());
                    }
                    Err(e) => return Err(path_error(e)),
                };
                if !self.tree.nodes.contains_key(&path) && force {
                    return Ok(String::new());
                }
                self.regular(&path)?;
                self.parent_access(&path)?;
                self.sticky_access(&path)?;
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            "mv" if args.len() == 2 => {
                if !ended && args.iter().any(|s| s.starts_with('-')) {
                    return Err(unsupported());
                }
                let source = self.tree.resolve(&args[0], false).map_err(path_error)?;
                self.regular(&source)?;
                let destination = self
                    .tree
                    .destination(&source, &args[1])
                    .map_err(|m| error(ErrorKind::Operation, m))?;
                let replaces = self.tree.nodes.contains_key(&destination);
                if replaces {
                    self.regular(&destination)?;
                }
                self.parent_access(&source)?;
                self.parent_access(&destination)?;
                self.sticky_access(&source)?;
                if replaces {
                    self.sticky_access(&destination)?;
                }
                self.tree
                    .command(words)
                    .map_err(|m| error(ErrorKind::Operation, m))
            }
            _ => Err(unsupported()),
        }
    }
    // Same bounded entry/metadata representation as the parent; the only new
    // display is t/T for this world's directory flag. No fake inode/host dates.
    fn list(&self, args: &[String]) -> Result<String, Error> {
        let (mut all, mut long, mut directory, mut options) = (false, false, false, true);
        let mut operands = Vec::new();
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
                        _ => return Err(unsupported()),
                    }
                }
            } else {
                operands.push(arg.as_str());
            }
        }
        if operands.len() > 1 {
            return Err(unsupported());
        }
        let arg = operands.first().copied().unwrap_or(".");
        let path = self.tree.resolve(arg, false).map_err(path_error)?;
        let node = &self.tree.nodes[&path];
        let mut entries = Vec::new();
        if node.directory && !directory {
            self.tree
                .permitted(&path, if long { 5 } else { 4 })
                .map_err(|m| error(ErrorKind::Permission, m))?;
            if all {
                entries.push((".".into(), path.clone()));
                entries.push(("..".into(), parent(&path).into()));
            }
            for p in self
                .tree
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
        entries.sort_by(|a: &(String, String), b| a.0.cmp(&b.0));
        let mut output = String::new();
        for (label, p) in entries {
            let row = if !long {
                label
            } else {
                let n = &self.tree.nodes[&p];
                let mut mode = crate::engine::mode_string(n);
                if self.sticky.contains(&p) {
                    mode.replace_range(9..10, if n.mode & 1 != 0 { "t" } else { "T" });
                }
                let size = self.tree.files.get(&p).map_or(0, String::len);
                let tick = self.tree.modified.get(&p).copied().unwrap_or(0);
                let links = if n.directory {
                    2 + self
                        .tree
                        .nodes
                        .iter()
                        .filter(|(q, n)| q.as_str() != "/" && parent(q) == p && n.directory)
                        .count()
                } else {
                    1
                };
                format!(
                    "{mode} {links} {} {} {size:>5} Jan {:>2} {:02}:{:02} {label}",
                    n.owner,
                    n.group,
                    1 + tick / 86400,
                    (tick / 3600) % 24,
                    (tick / 60) % 60
                )
            };
            if output.len().saturating_add(row.len()).saturating_add(1) > super::MAX_OUTPUT {
                return Err(error(ErrorKind::Resource, "Listing exceeds 32 KiB"));
            }
            output.push_str(&row);
            output.push('\n');
        }
        Ok(output)
    }
}
