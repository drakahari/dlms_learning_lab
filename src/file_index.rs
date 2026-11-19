//! Bounded pure indexed-discovery model; application integration is separate. No host I/O or utility execution.
//! Context is supplied by a caller-held source; imported metadata never authenticates itself.
use crate::filesystem::{Access, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const VERSION: u32 = 1;
pub const MAX_NODES: usize = 64;
pub const MAX_CONFIG: usize = 4096;
pub const MAX_CONTEXT_JSON: usize = 8192;
pub const MAX_INDEX_JSON: usize = 128 * 1024;
pub const MAX_PATH: usize = 1024;
pub const MAX_NAMES_BYTES: usize = 64 * 1024;
pub const MAX_OUTPUT: usize = 32 * 1024;
pub const MAX_WORK: usize = 16_384;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    SourceMismatch,
    Permission,
    Incomplete,
    MissingDatabase,
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
fn err(kind: ErrorKind, message: &str) -> Error {
    Error {
        kind,
        message: message.chars().take(384).collect(),
    }
}
fn field<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    crate::cron::text::<D, N>(d)
}
// Intentionally restrict the first fixture namespace to safe ASCII path text. No output quoting fiction.
fn canonical(p: &str) -> bool {
    p.starts_with('/')
        && p.len() <= MAX_PATH
        && p.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        && (p == "/"
            || (!p.ends_with('/')
                && p[1..].split('/').count() <= 32
                && p[1..]
                    .split('/')
                    .all(|s| !s.is_empty() && s != "." && s != ".." && s.len() <= 255)))
}
fn parent(p: &str) -> &str {
    p.rsplit_once('/')
        .map(|(d, _)| if d.is_empty() { "/" } else { d })
        .unwrap_or("/")
}
fn inside(path: &str, root: &str) -> bool {
    path == root
        || (root == "/" && path.starts_with('/'))
        || path
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
fn strict_child(path: &str, root: &str) -> bool {
    path != root && inside(path, root)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub version: u32,
    #[serde(deserialize_with = "field::<_,64>")]
    pub source_id: String,
    #[serde(deserialize_with = "field::<_,1024>")]
    pub root: String,
    #[serde(deserialize_with = "field::<_,1024>")]
    pub database: String,
    #[serde(deserialize_with = "field::<_,1024>")]
    pub config: String,
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
        {
            return Err(err(
                ErrorKind::Invalid,
                "Invalid indexed-discovery version or source identifier",
            ));
        }
        if [&self.root, &self.database, &self.config]
            .iter()
            .any(|p| !canonical(p))
        {
            return Err(err(
                ErrorKind::Unsupported,
                "This activity supports canonical absolute ASCII paths only",
            ));
        }
        // The private output and configuration are outside the scanned subtree, avoiding self-indexing.
        if self.root == "/"
            || self.database == self.config
            || inside(&self.database, &self.root)
            || inside(&self.config, &self.root)
            || self.database == "/"
            || self.config == "/"
        {
            return Err(err(
                ErrorKind::Unsupported,
                "Use a proper supplied subtree and distinct private config/database paths outside it",
            ));
        }
        Ok(())
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_CONTEXT_JSON {
            return Err(err(ErrorKind::Limit, "Context exceeds8KiB"));
        }
        let c: Self = serde_json::from_slice(raw).map_err(|_| {
            err(
                ErrorKind::Invalid,
                "Invalid or duplicate indexed-discovery context fields",
            )
        })?;
        c.shape()?;
        Ok(c)
    }
    /// Semantic authority is the caller's original source, not a carried/imported copy.
    pub fn validate_source(&self, source: &Tree) -> Result<()> {
        self.shape()?;
        if source.user == "root" {
            return Err(err(
                ErrorKind::Unsupported,
                "This private-index activity models an ordinary user, not privileged database visibility",
            ));
        }
        tree_bound(source)?;
        if !source.nodes.get(&self.root).is_some_and(|n| n.directory)
            || !source.nodes.get(&self.config).is_some_and(|n| !n.directory)
        {
            return Err(err(
                ErrorKind::Invalid,
                "Supplied scan root or configuration file is missing or has the wrong kind",
            ));
        }
        if source.nodes.contains_key(&self.database)
            || !source
                .nodes
                .get(parent(&self.database))
                .is_some_and(|n| n.directory)
        {
            return Err(err(
                ErrorKind::Invalid,
                "Private database identity must not overwrite a supplied file or directory",
            ));
        }
        if source
            .files
            .get(&self.config)
            .is_none_or(|s| s.len() > MAX_CONFIG)
        {
            return Err(err(
                ErrorKind::Limit,
                "Supplied configuration exceeds4KiB or is missing",
            ));
        }
        if !source.access(&source.user, &self.config, Access::ReadFile)
            || !source.access(&source.user, &self.config, Access::WriteFile)
            || !source.access(&source.user, parent(&self.database), Access::CreateEntry)
        {
            return Err(err(
                ErrorKind::Permission,
                "The ordinary user needs read/write access to the private config and write/search access to its database folder",
            ));
        }
        Ok(())
    }
}
fn tree_bound(tree: &Tree) -> Result<()> {
    // Check size fields before Tree::valid scans contents. No file bodies are cloned or parsed as code.
    if tree.nodes.len() > MAX_NODES
        || tree.files.len() > MAX_NODES
        || tree.users.len() > 16
        || tree.files.values().any(|s| s.len() > 131_072)
        || tree.files.values().map(String::len).sum::<usize>() > 262_144
    {
        return Err(err(
            ErrorKind::Limit,
            "Indexed-discovery tree exceeds64nodes or its existing text limits",
        ));
    }
    if tree.command_location.is_some() {
        return Err(err(
            ErrorKind::Unsupported,
            "Command-location metadata belongs to a different activity",
        ));
    }
    if tree.nodes.keys().any(|p| !canonical(p)) {
        return Err(err(
            ErrorKind::Unsupported,
            "Only bounded canonical ASCII fixture paths are supported",
        ));
    }
    if !tree.valid() {
        return Err(err(ErrorKind::Invalid, "Invalid virtual filesystem"));
    }
    Ok(())
}
fn bound(held: &Context, carried: Option<&Context>, source: &Tree, current: &Tree) -> Result<()> {
    held.validate_source(source)?;
    if carried != Some(held) {
        return Err(err(
            ErrorKind::SourceMismatch,
            "Indexed-discovery metadata differs from the caller-held source",
        ));
    }
    tree_bound(current)?;
    if current.users != source.users
        || current.user != source.user
        || current.home != source.home
        || current.umask != source.umask
    {
        return Err(err(
            ErrorKind::SourceMismatch,
            "Caller, group roster, home or umask differs from the held source",
        ));
    }
    if !current.nodes.get(&held.root).is_some_and(|n| n.directory)
        || !current
            .nodes
            .get(&held.config)
            .is_some_and(|n| !n.directory)
        || current.nodes.contains_key(&held.database)
        || !current
            .nodes
            .get(parent(&held.database))
            .is_some_and(|n| n.directory)
    {
        return Err(err(
            ErrorKind::SourceMismatch,
            "The supplied root/config/private database role is no longer valid",
        ));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pruning {
    directories: Vec<String>,
}
impl Pruning {
    pub fn directories(&self) -> &[String] {
        &self.directories
    }
}
/// One mandatory PRUNEPATHS quoted list; optional blank/comment lines. No shell evaluation.
pub fn parse_config(raw: &str, root: &str) -> Result<Pruning> {
    if raw.len() > MAX_CONFIG {
        return Err(err(ErrorKind::Limit, "Configuration exceeds4KiB"));
    }
    if !canonical(root) || root == "/" {
        return Err(err(ErrorKind::Invalid, "Invalid supplied subtree"));
    }
    if raw.contains('\0') || raw.contains('\r') || raw.contains('\\') {
        return Err(err(
            ErrorKind::Unsupported,
            "NUL, CR and escape syntax are not supported in this configuration",
        ));
    }
    let mut parsed = None;
    for line in raw.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let (key, value) = l.split_once('=').ok_or_else(|| {
            err(
                ErrorKind::Unsupported,
                "Use PRUNEPATHS = \"/supplied/directory ...\"; this is not a shell script",
            )
        })?;
        if key.trim() != "PRUNEPATHS" {
            return Err(err(
                ErrorKind::Unsupported,
                "Only PRUNEPATHS is supported; filesystem types and mount pruning are not modeled",
            ));
        }
        if parsed.is_some() {
            return Err(err(
                ErrorKind::Unsupported,
                "Repeated PRUNEPATHS assignments are outside this activity",
            ));
        }
        let rest = value.trim().strip_prefix('"').ok_or_else(|| {
            err(
                ErrorKind::Unsupported,
                "PRUNEPATHS needs one double-quoted list",
            )
        })?;
        let (value, tail) = rest
            .split_once('"')
            .ok_or_else(|| err(ErrorKind::Invalid, "Close the PRUNEPATHS quote"))?;
        if !tail.trim().is_empty() && !tail.trim().starts_with('#') {
            return Err(err(
                ErrorKind::Unsupported,
                "Unexpected text after the quoted PRUNEPATHS list",
            ));
        }
        if !value.is_ascii()
            || value
                .bytes()
                .any(|b| b.is_ascii_control() && !b.is_ascii_whitespace())
        {
            return Err(err(
                ErrorKind::Unsupported,
                "Use ASCII canonical directory paths separated by spaces",
            ));
        }
        let mut paths = BTreeSet::new();
        for p in value.split_ascii_whitespace() {
            if !canonical(p) || !strict_child(p, root) {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Prune only canonical directories strictly below the supplied scan root",
                ));
            }
            if paths.len() >= 16 && !paths.contains(p) {
                return Err(err(
                    ErrorKind::Limit,
                    "At most16pruned directories are supported",
                ));
            }
            paths.insert(p.to_string());
        }
        // Canonicalize redundant descendants as well as order/duplicates: same exclusion semantics.
        let mut directories: Vec<String> = Vec::new();
        for p in paths {
            if !directories.iter().any(|d| inside(&p, d)) {
                directories.push(p);
            }
        }
        parsed = Some(Pruning { directories });
    }
    parsed.ok_or_else(|| {
        err(
            ErrorKind::Invalid,
            "Add the required PRUNEPATHS = \"...\" setting; an empty quoted list is allowed",
        )
    })
}
struct PathText(String);
impl<'de> Deserialize<'de> for PathText {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        field::<D, 1024>(d).map(Self)
    }
}
fn saved_paths<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    crate::cron_workspace::sequence::<D, PathText, 64>(d)
        .map(|v| v.into_iter().map(|p| p.0).collect())
}
fn pruned_paths<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    crate::cron_workspace::sequence::<D, PathText, 16>(d)
        .map(|v| v.into_iter().map(|p| p.0).collect())
}
/// Saved names may honestly be stale. A parsed index is never proof of freshness or mastery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    version: u32,
    context: Context,
    #[serde(deserialize_with = "pruned_paths")]
    pruned: Vec<String>,
    #[serde(deserialize_with = "saved_paths")]
    entries: Vec<String>,
}
impl Index {
    pub fn context(&self) -> &Context {
        &self.context
    }
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
    pub fn pruned(&self) -> &[String] {
        &self.pruned
    }
    fn validate(&self, held: &Context) -> Result<()> {
        held.shape()?;
        if self.version != VERSION || &self.context != held {
            return Err(err(
                ErrorKind::SourceMismatch,
                "Saved index belongs to another source/root/config/database",
            ));
        }
        if self.entries.len() > MAX_NODES
            || self.pruned.len() > 16
            || self.entries.iter().map(String::len).sum::<usize>() > MAX_NAMES_BYTES
        {
            return Err(err(ErrorKind::Limit, "Saved index exceeds name limits"));
        }
        if self
            .pruned
            .iter()
            .any(|p| !canonical(p) || !strict_child(p, &held.root))
            || !strictly_sorted(&self.pruned)
            || self
                .entries
                .iter()
                .any(|p| !canonical(p) || !strict_child(p, &held.root))
        {
            return Err(err(
                ErrorKind::Invalid,
                "Saved paths must be canonical, ordered, unique and scoped to the held subtree",
            ));
        }
        if order_entries(&self.entries, &held.root)? != self.entries {
            return Err(err(
                ErrorKind::Invalid,
                "Saved names must follow directory-record order",
            ));
        }
        if self
            .pruned
            .iter()
            .enumerate()
            .any(|(i, p)| self.pruned[..i].iter().any(|d| inside(p, d)))
        {
            return Err(err(
                ErrorKind::Invalid,
                "Saved pruning must use canonical effective exclusions, without redundant descendants",
            ));
        }
        if self
            .entries
            .iter()
            .any(|p| self.pruned.iter().any(|d| strict_child(p, d)))
        {
            return Err(err(
                ErrorKind::Invalid,
                "Saved entries contradict their applied pruning policy",
            ));
        }
        Ok(())
    }
    pub fn decode(raw: &[u8], held: &Context, source: &Tree) -> Result<Self> {
        held.validate_source(source)?;
        if raw.len() > MAX_INDEX_JSON {
            return Err(err(ErrorKind::Limit, "Index JSON exceeds128KiB"));
        }
        let i: Self = serde_json::from_slice(raw).map_err(|_| {
            err(
                ErrorKind::Invalid,
                "Invalid, duplicate or overbound index data",
            )
        })?;
        i.validate(held)?;
        Ok(i)
    }
}
/// Names are emitted as a directory's sorted immediate children, then its child
/// directories are visited in the same order. The scan root itself is not a name.
/// This is an original bounded traversal of saved names, not plocate database code.
fn order_entries<'a>(entries: &'a [String], root: &'a str) -> Result<Vec<String>> {
    if entries.len() > MAX_NODES {
        return Err(err(ErrorKind::Limit, "Too many saved names"));
    }
    let names: BTreeSet<&str> = entries.iter().map(String::as_str).collect();
    if names.len() != entries.len()
        || names
            .iter()
            .any(|p| parent(p) != root && !names.contains(parent(p)))
    {
        return Err(err(
            ErrorKind::Invalid,
            "Saved names need unique entries and their parent names within the scan root",
        ));
    }
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for name in names {
        groups.entry(parent(name)).or_default().push(name);
    }
    let mut pending = vec![root];
    let mut ordered = Vec::with_capacity(entries.len());
    while let Some(directory) = pending.pop() {
        if let Some(children) = groups.get(directory) {
            ordered.extend(children.iter().map(|p| (*p).to_owned()));
            for &child in children.iter().rev() {
                if groups.contains_key(child) {
                    pending.push(child);
                }
            }
        }
    }
    if ordered.len() != entries.len() {
        return Err(err(
            ErrorKind::Invalid,
            "Saved names are disconnected from their scan root",
        ));
    }
    Ok(ordered)
}
fn strictly_sorted(v: &[String]) -> bool {
    v.windows(2).all(|w| w[0] < w[1])
}
fn spend(work: &mut usize, n: usize) -> Result<()> {
    *work = work
        .checked_add(n)
        .ok_or_else(|| err(ErrorKind::Limit, "Work counter overflow"))?;
    if *work > MAX_WORK {
        Err(err(
            ErrorKind::Limit,
            "Index operation exceeds its work limit",
        ))
    } else {
        Ok(())
    }
}
/// Builds a candidate only. The caller publishes it after success; failure leaves prior work untouched.
pub fn rebuild(
    held: &Context,
    carried: Option<&Context>,
    source: &Tree,
    current: &Tree,
) -> Result<Index> {
    bound(held, carried, source, current)?;
    if !current.access(&current.user, &held.config, Access::ReadFile)
        || !current.access(&current.user, parent(&held.database), Access::CreateEntry)
    {
        return Err(err(
            ErrorKind::Permission,
            "Cannot read the supplied config or write its private database folder",
        ));
    }
    let config = current
        .files
        .get(&held.config)
        .ok_or_else(|| err(ErrorKind::Invalid, "Configuration content is missing"))?;
    let pruning = parse_config(config, &held.root)?;
    for p in &pruning.directories {
        if current.nodes.get(p).is_some_and(|n| !n.directory) {
            return Err(err(
                ErrorKind::Unsupported,
                "PRUNEPATHS entries may name only directories, not ordinary files",
            ));
        }
    }
    let mut entries = Vec::new();
    let mut bytes = 0usize;
    let mut work = 0;
    for (p, node) in &current.nodes {
        spend(&mut work, 1)?;
        if !inside(p, &held.root) {
            continue;
        }
        let mut hidden = false;
        for d in &pruning.directories {
            spend(&mut work, 1)?;
            if strict_child(p, d) {
                hidden = true;
                break;
            }
        }
        if hidden {
            continue;
        }
        let pruned = pruning.directories.iter().any(|d| d == p);
        // A pruned directory's own name is retained; no access to its contents is requested.
        if node.directory
            && !pruned
            && (!current.access(&current.user, p, Access::ListNames)
                || !current.access(&current.user, p, Access::SearchDirectory))
        {
            return Err(err(
                ErrorKind::Incomplete,
                "Scan incomplete: a supplied directory needs read and search permission. Previous index retained",
            ));
        }
        // Check the root's scan access above, but store only names below it.
        if p == &held.root {
            continue;
        }
        spend(&mut work, p.split('/').count())?;
        bytes = bytes
            .checked_add(p.len())
            .ok_or_else(|| err(ErrorKind::Limit, "Name size overflow"))?;
        if entries.len() >= MAX_NODES || bytes > MAX_NAMES_BYTES {
            return Err(err(
                ErrorKind::Limit,
                "Index names exceed their resource limit",
            ));
        }
        entries.push(p.clone());
    }
    let index = Index {
        version: VERSION,
        context: held.clone(),
        pruned: pruning.directories,
        entries: order_entries(&entries, &held.root)?,
    };
    index.validate(held)?;
    Ok(index)
}
/// Semantic freshness only: no generation counter, saved observation or command history can certify it.
pub fn matches_current(
    index: &Index,
    held: &Context,
    carried: Option<&Context>,
    source: &Tree,
    current: &Tree,
) -> Result<bool> {
    index.validate(held)?;
    let actual = rebuild(held, carried, source, current)?;
    Ok(index == &actual)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Found,
    NoMatch,
    Rebuilt,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub status: Status,
    pub paths: Vec<String>,
    pub stdout: String,
}
fn literal(q: &str) -> Result<()> {
    if q.is_empty() || q.len() > 128 {
        return Err(err(
            ErrorKind::Limit,
            "Use one nonempty literal query of at most128bytes",
        ));
    }
    if !q.is_ascii()
        || q.bytes()
            .any(|b| b.is_ascii_control() || b"*?[]\\".contains(&b))
    {
        return Err(err(
            ErrorKind::Unsupported,
            "Only case-sensitive literal ASCII substring queries are supported; no globs, escapes or regex",
        ));
    }
    Ok(())
}
fn searchable_parent(tree: &Tree, p: &str) -> bool {
    let mut at = parent(p);
    loop {
        if !tree.access(&tree.user, at, Access::SearchDirectory) {
            return false;
        }
        if at == "/" {
            return true;
        }
        at = parent(at);
    }
}
pub fn locate(
    index: &Index,
    held: &Context,
    carried: Option<&Context>,
    source: &Tree,
    current: &Tree,
    query: &str,
    existing: bool,
) -> Result<Output> {
    bound(held, carried, source, current)?;
    index.validate(held)?;
    literal(query)?;
    let mut paths = Vec::new();
    let mut out = String::new();
    let mut work = 0;
    for p in &index.entries {
        spend(&mut work, p.len().saturating_mul(query.len().max(1)))?;
        if !p.contains(query) {
            continue;
        }
        if existing {
            if !current.nodes.contains_key(p) {
                continue;
            }
            if !searchable_parent(current, p) {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Existing-only lookup with inaccessible parent directories is outside this activity",
                ));
            }
        }
        if out.len() + p.len() + 1 > MAX_OUTPUT {
            return Err(err(
                ErrorKind::Limit,
                "Locate output exceeds32KiB; use a narrower literal query",
            ));
        }
        out.push_str(p);
        out.push('\n');
        paths.push(p.clone());
    }
    Ok(Output {
        status: if paths.is_empty() {
            Status::NoMatch
        } else {
            Status::Found
        },
        paths,
        stdout: out,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Locate { literal: String, existing: bool },
    Updatedb,
}
/// Parse only the fixed private forms. literal_command rejects shell composition, expansion and redirects.
pub fn parse_command(held: &Context, input: &str) -> Result<Command> {
    held.shape()?;
    if input.len() > 4096 {
        return Err(err(ErrorKind::Limit, "Command exceeds4KiB"));
    }
    let w = crate::shell::literal_command(input).map_err(|_| {
        err(
            ErrorKind::Unsupported,
            "Only one supported literal command is accepted; no shell composition or expansion",
        )
    })?;
    match w.first().map(String::as_str) {
        Some("locate") => {
            let mut db = None;
            let mut existing = false;
            let mut query = None;
            let mut n = 1;
            while n < w.len() {
                match w[n].as_str() {
                    "-d" => {
                        if db.is_some() || n + 1 >= w.len() {
                            return Err(err(
                                ErrorKind::Unsupported,
                                "Use locate [-e] -d SUPPLIED_DATABASE LITERAL",
                            ));
                        }
                        db = Some(w[n + 1].as_str());
                        n += 2;
                    }
                    "-e" => {
                        if existing {
                            return Err(err(
                                ErrorKind::Unsupported,
                                "Do not repeat locate options",
                            ));
                        }
                        existing = true;
                        n += 1;
                    }
                    v if v.starts_with('-') => {
                        return Err(err(
                            ErrorKind::Unsupported,
                            "Unsupported locate option; only -d and -e are available",
                        ));
                    }
                    v => {
                        if query.is_some() {
                            return Err(err(
                                ErrorKind::Unsupported,
                                "This activity supports one literal locate query",
                            ));
                        }
                        literal(v)?;
                        query = Some(v.to_string());
                        n += 1;
                    }
                }
            }
            if db != Some(held.database.as_str()) {
                return Err(err(
                    ErrorKind::SourceMismatch,
                    "Select only the supplied private database with -d",
                ));
            }
            Ok(Command::Locate {
                literal: query
                    .ok_or_else(|| err(ErrorKind::Unsupported, "Add one literal locate query"))?,
                existing,
            })
        }
        Some("updatedb") => {
            let mut seen = BTreeSet::new();
            let mut n = 1;
            while n < w.len() {
                let key = w[n].as_str();
                if !["-l", "-o", "-U", "--config-file"].contains(&key)
                    || !seen.insert(key)
                    || n + 1 >= w.len()
                {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "Use updatedb -l 0 -o SUPPLIED_DATABASE -U SUPPLIED_ROOT --config-file SUPPLIED_CONFIG",
                    ));
                }
                let expected = match key {
                    "-l" => "0",
                    "-o" => &held.database,
                    "-U" => &held.root,
                    "--config-file" => &held.config,
                    _ => unreachable!(),
                };
                if w[n + 1] != expected {
                    return Err(err(
                        ErrorKind::SourceMismatch,
                        "Only supplied private root/config/database and visibility-disabled -l 0 are supported",
                    ));
                }
                n += 2;
            }
            if seen.len() != 4 {
                return Err(err(
                    ErrorKind::Unsupported,
                    "All four fixed private updatedb options are required",
                ));
            }
            Ok(Command::Updatedb)
        }
        _ => Err(err(
            ErrorKind::Unsupported,
            "Indexed discovery supports locate and updatedb only; no host command ran",
        )),
    }
}
/// This adapter never dispatches filesystem actions or shells. Future workspace code owns those separately.
pub fn execute_bound(
    held: &Context,
    carried: Option<&Context>,
    source: &Tree,
    current: &Tree,
    saved: &mut Option<Index>,
    input: &str,
) -> Result<Output> {
    bound(held, carried, source, current)?;
    match parse_command(held, input)? {
        Command::Locate { literal, existing } => locate(
            saved.as_ref().ok_or_else(|| {
                err(
                    ErrorKind::MissingDatabase,
                    "Build the supplied private index first",
                )
            })?,
            held,
            carried,
            source,
            current,
            &literal,
            existing,
        ),
        Command::Updatedb => {
            let candidate = rebuild(held, carried, source, current)?;
            *saved = Some(candidate);
            Ok(Output {
                status: Status::Rebuilt,
                paths: vec![],
                stdout: String::new(),
            })
        }
    }
}
