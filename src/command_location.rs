//! Pure typed command-location diagnostics over current virtual metadata.
//! Never loads/runs a file, invokes a shell, or observes host PATH/files/processes.
use crate::filesystem::{
    Tree,
    path_lookup::{self, Candidate, Decision, Profile},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const VERSION: u32 = 1;
pub const MAX_CONTEXT_BYTES: usize = 64 * 1024;
pub const MAX_INPUT_BYTES: usize = 2048;
pub const MAX_OUTPUT_BYTES: usize = 32 * 1024;
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
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: &str) -> Error {
    let mut end = message.len().min(512);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    Error {
        kind,
        message: message[..end].into(),
    }
}
fn from_lookup(e: path_lookup::Error) -> Error {
    let kind = match e.kind {
        path_lookup::ErrorKind::Invalid => ErrorKind::Invalid,
        path_lookup::ErrorKind::Unsupported => ErrorKind::Unsupported,
        path_lookup::ErrorKind::Limit => ErrorKind::Limit,
    };
    error(kind, &e.message)
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
/// Metadata belongs to a held source; structural validity never authenticates it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub version: u32,
    #[serde(deserialize_with = "text::<_,64>")]
    pub source_id: String,
    #[serde(deserialize_with = "text::<_,16384>")]
    pub path: String,
    #[serde(deserialize_with = "names")]
    pub names: Vec<String>,
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
fn basename(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
impl Context {
    /// Borrowed shape preflight; reserved names are admitted by existing Profile validation.
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
        {
            return Err(error(
                ErrorKind::Invalid,
                "Command-location source version, ID or name count is invalid",
            ));
        }
        let mut seen = BTreeSet::new();
        if self.names.iter().any(|n| !basename(n) || !seen.insert(n)) {
            return Err(error(
                ErrorKind::Invalid,
                "Supply one to four distinct ASCII external names",
            ));
        }
        let _ = self.components()?;
        Ok(())
    }
    fn components(&self) -> Result<Vec<&str>> {
        if self.path.len() > path_lookup::MAX_PATH {
            return Err(error(ErrorKind::Limit, "Supplied PATH exceeds16KiB"));
        }
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for part in self.path.split(':') {
            if out.len() == 16 {
                return Err(error(ErrorKind::Limit, "Supplied PATH exceeds16components"));
            }
            if !canonical(part) || part.is_empty() || !seen.insert(part) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This command-location activity needs distinct nonempty canonical absolute PATH components; no default or current-directory search is invented",
                ));
            }
            for name in &self.names {
                let candidate = if part == "/" {
                    format!("/{name}")
                } else {
                    format!("{part}/{name}")
                };
                if !canonical(&candidate) {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "A supplied PATH/name candidate exceeds canonical path bounds",
                    ));
                }
            }
            out.push(part);
        }
        Ok(out)
    }
    pub fn validate_for(&self, source: &Tree) -> Result<()> {
        self.shape()?;
        preflight(source)?;
        project(source, &self.names)?
            .validate()
            .map_err(from_lookup)
    }
    pub fn decode(raw: &[u8], source: &Tree) -> Result<Self> {
        if raw.len() > MAX_CONTEXT_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Command-location context JSON exceeds64KiB",
            ));
        }
        let c: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                ErrorKind::Invalid,
                "Invalid bounded command-location context JSON",
            )
        })?;
        c.validate_for(source)?;
        Ok(c)
    }
}
fn preflight(tree: &Tree) -> Result<()> {
    // Check small limits before walking text and BEFORE any metadata clone.
    if tree.nodes.len() > 64 || tree.files.len() > 64 {
        return Err(error(
            ErrorKind::Limit,
            "Command-location tree exceeds64nodes or files",
        ));
    }
    if !tree.valid() {
        return Err(error(
            ErrorKind::Invalid,
            "Command-location needs a valid supplied ordinary-user tree",
        ));
    }
    Ok(())
}
fn project(tree: &Tree, names: &[String]) -> Result<Profile> {
    preflight(tree)?;
    // Construct empty metadata values directly. Never clone large file contents to discard.
    let files: BTreeMap<String, String> = tree
        .files
        .keys()
        .map(|p| (p.clone(), String::new()))
        .collect();
    Ok(Profile {
        version: path_lookup::VERSION,
        tree: Tree {
            nodes: tree.nodes.clone(),
            files,
            users: tree.users.clone(),
            user: tree.user.clone(),
            home: tree.home.clone(),
            cwd: tree.cwd.clone(),
            previous: tree.previous.clone(),
            umask: tree.umask,
            clock: tree.clock,
            modified: tree.modified.clone(),
            command_location: None,
        },
        trusted_directories: vec![],
        names: names.to_vec(),
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Which,
    TypeAllPaths,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub mode: Mode,
    #[serde(deserialize_with = "text::<_,48>")]
    pub name: String,
}
impl Query {
    pub fn parse(context: &Context, input: &str) -> Result<Self> {
        context.shape()?;
        if input.len() > MAX_INPUT_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Command-location input exceeds2KiB",
            ));
        }
        let words=crate::shell::literal_command(input).map_err(|_|error(ErrorKind::Unsupported,"Only one literal command-location query is supported; no expansion, redirection, pipelines or command composition"))?;
        let (mode, name) = match words.as_slice() {
            [cmd, name] if cmd == "which" => (Mode::Which, name),
            [cmd, a, p, name] if cmd == "type" && a == "-a" && p == "-p" => {
                (Mode::TypeAllPaths, name)
            }
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Supported queries: which NAME or type -a -p NAME, for one supplied external name",
                ));
            }
        };
        if !context.names.contains(name) {
            return Err(error(
                ErrorKind::Unsupported,
                "That name is not a supplied query in this activity",
            ));
        }
        Ok(Self {
            mode,
            name: name.clone(),
        })
    }
}
/// Semantic result only: NOT an invented raw Bash/which exit code or kernel status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Status {
    Found,
    Missing,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Output {
    pub context: Context,
    pub query: Query,
    pub status: Status,
    /// Only actual selected paths, one per line. Empty for a supported missing result.
    pub stdout: String,
    /// Labeled simulator guidance, not a verbatim utility stderr promise.
    pub diagnostic: String,
    pub paths: Vec<String>,
    pub candidates: Vec<Candidate>,
    /// Existing resolver candidate-entry evaluations, not wallclock or complete CPU accounting.
    pub steps: u32,
}
impl Output {
    pub fn success(&self) -> bool {
        self.status == Status::Found
    }
    pub fn matches(&self, held: &Context, source: &Tree, current: &Tree) -> bool {
        self.context == *held
            && self.stdout.len() <= MAX_OUTPUT_BYTES
            && self.diagnostic.len() <= 512
            && self.paths.len() <= 16
            && self.candidates.len() <= 16
            && self.paths.iter().all(|p| p.len() <= 1024)
            && evaluate(held, source, current, &self.query).is_ok_and(|o| o == *self)
    }
}
/// Standalone query using CALLER-HELD context and original tree. No mutation.
/// This validates context/caller foundation, not arbitrary assignment file-edit policy.
pub fn evaluate(held: &Context, source: &Tree, current: &Tree, query: &Query) -> Result<Output> {
    held.validate_for(source)?;
    preflight(current)?;
    if current.user != source.user || current.users != source.users || current.home != source.home {
        return Err(error(
            ErrorKind::SourceMismatch,
            "Command-location caller/home/groups differ from the held source",
        ));
    }
    if !held.names.contains(&query.name) {
        return Err(error(
            ErrorKind::Unsupported,
            "That name is not a supplied query in this activity",
        ));
    }
    let mut profile = project(current, &held.names)?;
    profile.names = vec![query.name.clone()];
    profile.validate().map_err(from_lookup)?;
    let mut paths = Vec::new();
    let mut candidates = Vec::new();
    let mut steps = 0u32;
    match query.mode {
        Mode::Which => {
            let mut observation =
                path_lookup::resolve(&profile, Some(&held.path)).map_err(from_lookup)?;
            steps = observation.steps;
            let report = observation.reports.pop().ok_or_else(|| {
                error(
                    ErrorKind::Invalid,
                    "Resolver did not return its supplied query",
                )
            })?;
            if let Decision::Selected { path } = report.decision {
                paths.push(path);
            }
            candidates = report.candidates;
        }
        Mode::TypeAllPaths => {
            // Reuse the existing permission/candidate implementation. Old resolve stays byte-for-byte unchanged.
            // A single-directory query has at most one candidate; accumulating16 calls is finite.
            for directory in held.components()? {
                let mut observation =
                    path_lookup::resolve(&profile, Some(directory)).map_err(from_lookup)?;
                steps = steps
                    .checked_add(observation.steps)
                    .filter(|n| *n <= path_lookup::MAX_STEPS)
                    .ok_or_else(|| {
                        error(
                            ErrorKind::Limit,
                            "All-path lookup traversal budget exceeded",
                        )
                    })?;
                let report = observation.reports.pop().ok_or_else(|| {
                    error(
                        ErrorKind::Invalid,
                        "Resolver did not return its supplied query",
                    )
                })?;
                if let Decision::Selected { path } = report.decision {
                    paths.push(path);
                }
                candidates.extend(report.candidates);
            }
        }
    }
    let status = if paths.is_empty() {
        Status::Missing
    } else {
        Status::Found
    };
    let stdout = paths.iter().map(|p| format!("{p}\n")).collect::<String>();
    if stdout.len() > MAX_OUTPUT_BYTES || paths.len() > 16 || candidates.len() > 16 {
        return Err(error(
            ErrorKind::Limit,
            "Command-location output exceeds its bound",
        ));
    }
    let diagnostic = if status == Status::Missing {
        format!(
            "Simulator lookup: no executable file named {} is selectable in the supplied PATH; inspect file execute bits and ancestor search permissions.",
            query.name
        )
    } else {
        String::new()
    };
    Ok(Output {
        context: held.clone(),
        query: query.clone(),
        status,
        stdout,
        diagnostic,
        paths,
        candidates,
        steps,
    })
}
/// Imported optional metadata must match caller-held context BEFORE any projection.
/// Neither carried context nor original tree supplied by an import authenticates itself.
pub fn execute_bound(
    held: &Context,
    carried: Option<&Context>,
    source: &Tree,
    current: &Tree,
    input: &str,
) -> Result<Output> {
    held.shape()?;
    let carried = carried.ok_or_else(|| {
        error(
            ErrorKind::SourceMismatch,
            "Command-location source metadata is missing",
        )
    })?;
    carried.shape()?;
    if carried != held {
        return Err(error(
            ErrorKind::SourceMismatch,
            "Command-location source metadata differs from the held activity",
        ));
    }
    let query = Query::parse(held, input)?;
    evaluate(held, source, current, &query)
}
