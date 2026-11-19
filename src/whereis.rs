//! Original bounded virtual whereis lookup; no host I/O or file execution.
use crate::filesystem::{Access, Tree};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const VERSION: u32 = 1;
pub const MAX_CONTEXT_BYTES: usize = 16 * 1024;
pub const MAX_INPUT_BYTES: usize = 2048;
pub const MAX_OUTPUT_BYTES: usize = 32 * 1024;
const MAX_NODES: usize = 64;
const MAX_DATA_BYTES: usize = 256 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    SourceMismatch,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: &'static str) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    crate::cron::text::<D, N>(d)
}
struct Name(String);
impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        text::<D, 48>(d).map(Self)
    }
}
fn names<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    crate::cron_workspace::sequence::<D, Name, 4>(d).map(|v| v.into_iter().map(|n| n.0).collect())
}
fn directories<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Directory>, D::Error> {
    crate::cron_workspace::sequence::<D, Directory, 3>(d)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Binary,
    Manual,
    Source,
}
impl Category {
    fn options(self) -> (&'static str, &'static str) {
        match self {
            Self::Binary => ("-b", "-B"),
            Self::Manual => ("-m", "-M"),
            Self::Source => ("-s", "-S"),
        }
    }
    fn expected_name(self, name: &str) -> String {
        match self {
            Self::Binary => name.into(),
            Self::Manual => format!("{name}.1"),
            Self::Source => format!("{name}.c"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Directory {
    pub category: Category,
    #[serde(deserialize_with = "text::<_,1024>")]
    pub path: String,
}
/// Held metadata describes allowed queries; it does not authorize tree edits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub version: u32,
    #[serde(deserialize_with = "text::<_,64>")]
    pub source_id: String,
    #[serde(deserialize_with = "names")]
    pub names: Vec<String>,
    #[serde(deserialize_with = "directories")]
    pub directories: Vec<Directory>,
}
fn canonical(p: &str) -> bool {
    p.starts_with('/')
        && p.len() <= 1024
        && !p.chars().any(char::is_control)
        && (p == "/"
            || (!p.ends_with('/')
                && p[1..].split('/').count() <= 64
                && p[1..]
                    .split('/')
                    .all(|s| !s.is_empty() && s != "." && s != ".." && s.len() <= 255)))
}
fn basename(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 48
        && n.as_bytes()[0].is_ascii_alphabetic()
        && n.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn child(dir: &str, name: &str) -> String {
    if dir == "/" {
        format!("/{name}")
    } else {
        format!("{dir}/{name}")
    }
}
fn parent(p: &str) -> &str {
    p.rsplit_once('/')
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}
impl Context {
    pub fn shape(&self) -> Result<()> {
        if self.version != VERSION
            || self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            || self.names.is_empty()
            || self.names.len() > 4
            || self.directories.is_empty()
            || self.directories.len() > 3
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid whereis source version, ID or query roster bounds",
            ));
        }
        let mut seen = BTreeSet::new();
        if self.names.iter().any(|n| !basename(n) || !seen.insert(n)) {
            return Err(error(
                ErrorKind::Invalid,
                "Supply one to four distinct ASCII dot-free basenames",
            ));
        }
        let mut categories = BTreeSet::new();
        for d in &self.directories {
            if !canonical(&d.path) || !categories.insert(d.category) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Supply one canonical absolute directory for each distinct category",
                ));
            }
            for n in &self.names {
                if !canonical(&child(&d.path, &d.category.expected_name(n))) {
                    return Err(error(
                        ErrorKind::Limit,
                        "A supplied directory/name candidate exceeds canonical path bounds",
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn validate_for(&self, source: &Tree) -> Result<()> {
        self.shape()?;
        preflight(source)
    }
    pub fn decode(raw: &[u8], source: &Tree) -> Result<Self> {
        if raw.len() > MAX_CONTEXT_BYTES {
            return Err(error(ErrorKind::Limit, "Whereis context JSON exceeds16KiB"));
        }
        let c: Self = serde_json::from_slice(raw)
            .map_err(|_| error(ErrorKind::Invalid, "Invalid bounded whereis context JSON"))?;
        c.validate_for(source)?;
        Ok(c)
    }
    fn directory(&self, category: Category) -> Result<&str> {
        self.directories
            .iter()
            .find(|d| d.category == category)
            .map(|d| d.path.as_str())
            .ok_or_else(|| {
                error(
                    ErrorKind::Unsupported,
                    "This category is not supplied by the held activity",
                )
            })
    }
}
fn preflight(tree: &Tree) -> Result<()> {
    if tree.nodes.len() > MAX_NODES
        || tree.files.len() > MAX_NODES
        || tree.modified.len() > MAX_NODES
    {
        return Err(error(
            ErrorKind::Limit,
            "Whereis virtual tree exceeds64nodes, files or modification records",
        ));
    }
    let mut bytes = 0usize;
    for text in tree.files.values() {
        bytes = bytes
            .checked_add(text.len())
            .ok_or_else(|| error(ErrorKind::Limit, "Virtual file byte budget exceeded"))?;
        if bytes > MAX_DATA_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Whereis virtual file contents exceed256KiB",
            ));
        }
    }
    if !tree.valid() {
        return Err(error(
            ErrorKind::Invalid,
            "Whereis needs a valid supplied ordinary-user virtual tree",
        ));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub category: Category,
    #[serde(deserialize_with = "text::<_,48>")]
    pub name: String,
}
impl Query {
    pub fn parse(held: &Context, input: &str) -> Result<Self> {
        held.shape()?;
        if input.len() > MAX_INPUT_BYTES {
            return Err(error(ErrorKind::Limit, "Whereis query exceeds2KiB"));
        }
        let words=crate::shell::literal_command(input).map_err(|_|error(ErrorKind::Unsupported,"Only one literal whereis query is supported; no expansion, redirects, pipelines or composition"))?;
        if let [cmd, small, big, dir, f, name] = words.as_slice()
            && cmd == "whereis"
            && f == "-f"
        {
            for category in [Category::Binary, Category::Manual, Category::Source] {
                let (a, b) = category.options();
                if small == a
                    && big == b
                    && dir == held.directory(category)?
                    && held.names.contains(name)
                {
                    return Ok(Self {
                        category,
                        name: name.clone(),
                    });
                }
            }
        }
        Err(error(
            ErrorKind::Unsupported,
            "Use whereis -b -B DIR -f NAME, -m -M, or -s -S with the supplied category, directory and name",
        ))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Status {
    Found,
    Missing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Scan {
    Listed,
    MissingDirectory,
    NotDirectory,
    InaccessibleDirectory,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Output {
    pub context: Context,
    pub query: Query,
    pub status: Status,
    pub scan: Scan,
    pub paths: Vec<String>,
    pub stdout: String,
    /// Normal missing query also returns0: source-grounded, fixed observation pending.
    pub exit_status: u8,
    /// Simulator explanation, not verbatim whereis stderr.
    pub diagnostic: String,
    /// Immediate entries inspected; not elapsed time or complete CPU accounting.
    pub inspected_entries: u32,
}
impl Output {
    pub fn command_succeeded(&self) -> bool {
        self.exit_status == 0
    }
    pub fn matches(&self, held: &Context, source: &Tree, current: &Tree) -> bool {
        self.stdout.len() <= MAX_OUTPUT_BYTES
            && self.diagnostic.len() <= 512
            && self.paths.len() <= 1
            && self.context == *held
            && evaluate(held, source, current, &self.query).is_ok_and(|o| o == *self)
    }
}
/// Conservative possible-match envelope, not a port of the utility matcher.
/// Possibly significant unimplemented names are diagnosed, never silently missed.
fn potential(category: Category, name: &str, entry: &str) -> bool {
    if category == Category::Binary {
        return entry == name;
    }
    let mut e = entry;
    if category == Category::Source {
        while let Some(rest) = e.strip_prefix("s.") {
            e = rest;
        }
    }
    e == name
        || e.strip_prefix(name)
            .is_some_and(|tail| tail.starts_with('.'))
}
pub fn evaluate(held: &Context, source: &Tree, current: &Tree, query: &Query) -> Result<Output> {
    held.validate_for(source)?;
    preflight(current)?;
    if current.user != source.user
        || current.users != source.users
        || current.home != source.home
        || current.command_location != source.command_location
    {
        return Err(error(
            ErrorKind::SourceMismatch,
            "Whereis caller or operational marker differs from the held source",
        ));
    }
    if !held.names.contains(&query.name) {
        return Err(error(
            ErrorKind::Unsupported,
            "That basename is not supplied by this activity",
        ));
    }
    let dir = held.directory(query.category)?;
    let scan = match current.nodes.get(dir) {
        None => Scan::MissingDirectory,
        Some(n) if !n.directory => Scan::NotDirectory,
        Some(_) if !current.access(&current.user, dir, Access::ListNames) => {
            Scan::InaccessibleDirectory
        }
        Some(_) => Scan::Listed,
    };
    let mut paths = Vec::new();
    let mut inspected_entries = 0u32;
    if scan == Scan::Listed {
        let wanted = query.category.expected_name(&query.name);
        for p in current.nodes.keys() {
            if p == "/" || parent(p) != dir {
                continue;
            }
            inspected_entries += 1;
            let name = p.rsplit('/').next().unwrap_or("");
            if !potential(query.category, &query.name, name) {
                continue;
            }
            if name != wanted {
                return Err(error(
                    ErrorKind::Unsupported,
                    "A possibly matching source prefix, suffix, manual section or compressed name is outside this activity; no partial answer was returned",
                ));
            }
            if !paths.is_empty() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Multiple matches need actual directory-entry ordering, which this activity does not model",
                ));
            }
            paths.push(p.clone());
        }
    }
    let status = if paths.is_empty() {
        Status::Missing
    } else {
        Status::Found
    };
    let stdout = if let Some(p) = paths.first() {
        format!("{}: {p}\n", query.name)
    } else {
        format!("{}:\n", query.name)
    };
    if stdout.len() > MAX_OUTPUT_BYTES {
        return Err(error(ErrorKind::Limit, "Whereis output exceeds32KiB"));
    }
    let diagnostic=match scan {
        Scan::Listed if status==Status::Missing=>"Simulator: no supported matching name in the supplied category directory.",
        Scan::Listed=>"Simulator: this name was found; discovery alone does not prove version, executable permission, or file contents.",
        Scan::MissingDirectory=>"Simulator: the supplied directory is missing; the normal query still succeeds with no matching path.",
        Scan::NotDirectory=>"Simulator: the supplied path is not a directory; no matching path was searched.",
        Scan::InaccessibleDirectory=>"Simulator: directory names could not be read with this caller's ancestor-search and directory-read permissions.",
    }.into();
    Ok(Output {
        context: held.clone(),
        query: query.clone(),
        status,
        scan,
        paths,
        stdout,
        exit_status: 0,
        diagnostic,
        inspected_entries,
    })
}
/// Caller supplies original source and context from trusted lesson/Goal.
/// Carried metadata never authenticates itself; higher layers enforce edit policy.
pub fn execute_bound(
    held: &Context,
    carried: Option<&Context>,
    source: &Tree,
    current: &Tree,
    input: &str,
) -> Result<Output> {
    held.shape()?;
    let carried =
        carried.ok_or_else(|| error(ErrorKind::SourceMismatch, "Whereis metadata is missing"))?;
    carried.shape()?;
    if carried != held {
        return Err(error(
            ErrorKind::SourceMismatch,
            "Whereis metadata differs from the held activity",
        ));
    }
    let query = Query::parse(held, input)?;
    evaluate(held, source, current, &query)
}
