//! Source-bound private-index editor/terminal simulation. No host I/O.
use crate::{
    cron_workspace as bounded, file_index as core,
    filesystem::{Access, Tree},
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const VERSION: u32 = 1;
pub const HELP: &str = "Private index: locate [-e] -d SUPPLIED_DB LITERAL; updatedb -l 0 -o SUPPLIED_DB -U SUPPLIED_ROOT --config-file SUPPLIED_CONFIG. Use the native Save configuration action to apply editor text; saving never rebuilds. Existing simulated pwd/cd/ls/cat/find/cp/mv/rm/touch/chmod/chgrp/id/umask grammar applies only within the held task preservation policy. Directories, accounts and unrelated files are protected; only declared ordinary leaves may change. The private database is an opaque typed artifact: generic cat/cp/rm/ls of it or its storage folder is not supported; use its native name/count/status row. No root/global database, scripts, pipes, expansion, host filesystem, subprocess or network execution. help is simulator reference, not a Linux command lesson.";
pub const MAX_DEFINITION: usize = 2 * 1024 * 1024;
pub const MAX_SESSION: usize = 1024 * 1024;
pub const MAX_TREE_TEXT: usize = 32 * 1024;
pub const MAX_FILE_TEXT: usize = 8192;
pub const MAX_HISTORY_BYTES: usize = 16 * 1024;
pub const MAX_ROW_OUTPUT: usize = 8192;
pub const MAX_COMMAND: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Assignment,
    Example,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    Source,
    Configuration,
    Filesystem,
    Policy,
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
fn error(kind: ErrorKind, message: &str) -> Error {
    let text = if message.len() > 640 {
        format!(
            "{}\n[Simulator: diagnostic shortened; use a narrower command.]",
            short(message, 560)
        )
    } else {
        message.to_owned()
    };
    Error {
        kind,
        message: text,
    }
}
fn from_core(e: core::Error) -> Error {
    let k = match e.kind {
        core::ErrorKind::Invalid => ErrorKind::Invalid,
        core::ErrorKind::Unsupported => ErrorKind::Unsupported,
        core::ErrorKind::Limit => ErrorKind::Limit,
        core::ErrorKind::SourceMismatch => ErrorKind::Source,
        _ => ErrorKind::Filesystem,
    };
    error(k, &e.message)
}
fn parse_cfg(s: &str, root: &str) -> Result<core::Pruning> {
    core::parse_config(s, root).map_err(|e| {
        if e.kind == core::ErrorKind::Invalid {
            error(ErrorKind::Configuration, &e.message)
        } else {
            from_core(e)
        }
    })
}
fn short(s: &str, n: usize) -> String {
    let mut end = s.len().min(n);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    crate::cron::text::<D, N>(d)
}
struct Path(String);
impl<'de> Deserialize<'de> for Path {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        text::<D, 1024>(d).map(Self)
    }
}
fn extra_paths<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    bounded::sequence::<D, Path, 4>(d).map(|v| v.into_iter().map(|p| p.0).collect())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub context: core::Context,
    pub tree: Tree,
    pub index: Option<core::Index>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Step {
    SaveConfig {
        #[serde(deserialize_with = "text::<_,4096>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "text::<_,4096>")]
        input: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainedStep {
    #[serde(deserialize_with = "text::<_,100>")]
    pub title: String,
    #[serde(deserialize_with = "text::<_,1200>")]
    pub explanation: String,
    pub step: Step,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Teaching {
    #[serde(deserialize_with = "text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "text::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "bounded::sequence::<_,ExplainedStep,8>")]
    pub worked: Vec<ExplainedStep>,
    #[serde(deserialize_with = "bounded::sequence::<_,ExplainedStep,8>")]
    pub try_steps: Vec<ExplainedStep>,
    #[serde(deserialize_with = "bounded::sequence::<_,ExplainedStep,8>")]
    pub repair: Vec<ExplainedStep>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub initial: Fixture,
    pub expected_tree: Tree,
    #[serde(deserialize_with = "text::<_,1024>")]
    pub handover: String,
    #[serde(default, deserialize_with = "extra_paths")]
    pub extra_paths: Vec<String>,
    /// Author-only reachability, never constructor/reset/load actions or stored learner answers.
    #[serde(deserialize_with = "bounded::sequence::<_,Step,8>")]
    pub reference: Vec<Step>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "text::<_,48>")]
    pub id: String,
    pub version: u32,
    pub assignment: Task,
    pub example: Task,
    pub teaching: Teaching,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Filesystem,
    Found,
    NoMatch,
    IndexRebuilt,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "text::<_,4096>")]
    pub input: String,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub output: String,
    pub outcome: Outcome,
    pub truncated: bool,
}
impl Row {
    fn bytes(&self) -> usize {
        self.input.len() + self.output.len() + 128
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Current {
    pub context: core::Context,
    pub tree: Tree,
    pub index: Option<core::Index>,
    /// Invalid pending text is permitted; no missing/erased resumed-field backfill.
    #[serde(deserialize_with = "text::<_,4096>")]
    pub config_draft: String,
    #[serde(deserialize_with = "text::<_,4096>")]
    pub command: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,20>")]
    pub transcript: Vec<Row>,
    /// Presentation flag only. Every success query recomputes the actual state.
    pub checked: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "text::<_,48>")]
    pub source_id: String,
    pub source_version: u32,
    pub assignment: Current,
    pub example: Current,
    pub view: View,
    pub selected_example: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckKey {
    PendingDraft,
    SavedPruning,
    FreshIndex,
    Handover,
    OtherFiles,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub key: CheckKey,
    pub passed: bool,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub checks: Vec<Check>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub outcome: Outcome,
    pub stdout: String,
    pub diagnostic: String,
    pub truncated_in_history: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub path: String,
    pub names: usize,
    pub fresh: bool,
}
fn canonical(p: &str) -> bool {
    p.starts_with('/')
        && p.len() <= 1024
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
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}
fn inside(p: &str, d: &str) -> bool {
    p == d || d == "/" || p.strip_prefix(d).is_some_and(|s| s.starts_with('/'))
}
fn safe_text(s: &str, n: usize) -> bool {
    !s.is_empty() && s.len() <= n && !s.contains('\0')
}
fn tree_size(t: &Tree) -> Result<()> {
    if t.nodes.len() > 64
        || t.files.len() > 64
        || t.users.len() > 16
        || t.files.values().any(|s| s.len() > MAX_FILE_TEXT)
        || t.files.values().map(String::len).sum::<usize>() > MAX_TREE_TEXT
    {
        return Err(error(
            ErrorKind::Limit,
            "This activity permits64entries,8KiB per file and32KiB total text",
        ));
    }
    if t.command_location.is_some() || t.nodes.keys().any(|p| !canonical(p)) || !t.valid() {
        return Err(error(
            ErrorKind::Source,
            "Invalid tree or metadata from a different activity; previous work retained",
        ));
    }
    Ok(())
}
fn allowed(t: &Task, p: &str) -> bool {
    p == t.initial.context.config || p == t.handover || t.extra_paths.iter().any(|x| x == p)
}
fn node_identity(t: &Task, p: &str, n: &crate::engine::Node) -> bool {
    let template = t
        .initial
        .tree
        .nodes
        .get(p)
        .or_else(|| t.expected_tree.nodes.get(p));
    let (owner, group) = template
        .map(|n| (n.owner.as_str(), n.group.as_str()))
        .unwrap_or_else(|| {
            (
                t.initial.tree.user.as_str(),
                t.initial.tree.users[&t.initial.tree.user][0].as_str(),
            )
        });
    !n.directory && n.owner == owner && n.group == group
}
/// This is an allowed-state policy, not a solved-state check. Partial/wrong handover and modes survive.
fn policy(t: &Task, current: &Tree) -> Result<()> {
    tree_size(current)?;
    let source = &t.initial.tree;
    if current.users != source.users
        || current.user != source.user
        || current.home != source.home
        || current.umask != source.umask
    {
        return Err(error(
            ErrorKind::Source,
            "Caller, groups, home or umask differ from the held source",
        ));
    }
    for (p, n) in &source.nodes {
        if (n.directory || !allowed(t, p))
            && (current.nodes.get(p) != Some(n) || current.files.get(p) != source.files.get(p))
        {
            return Err(error(
                ErrorKind::Policy,
                "An unrelated file, directory, mode or ownership changed; this command was not applied",
            ));
        }
    }
    for (p, n) in &current.nodes {
        if !source.nodes.contains_key(p) && !allowed(t, p) {
            return Err(error(
                ErrorKind::Policy,
                "Only the supplied handover and declared practice leaves may be added",
            ));
        }
        if allowed(t, p) && !node_identity(t, p, n) {
            return Err(error(
                ErrorKind::Policy,
                "Editable leaves must retain their supplied ordinary-file kind, owner and group",
            ));
        }
    }
    let cfg = &t.initial.context.config;
    if current.nodes.get(cfg) != source.nodes.get(cfg) || !current.files.contains_key(cfg) {
        return Err(error(
            ErrorKind::Policy,
            "Keep the supplied configuration file and its permissions; edit its text with Save configuration",
        ));
    }
    parse_cfg(&current.files[cfg], &t.initial.context.root)?;
    Ok(())
}
fn step_shape(s: &Step, t: &Task) -> Result<()> {
    match s {
        Step::SaveConfig { text } => {
            parse_cfg(text, &t.initial.context.root)?;
        }
        Step::Command { input } => {
            if !safe_text(input, MAX_COMMAND) {
                return Err(error(ErrorKind::Invalid, "Invalid author command"));
            }
            crate::shell::literal_command(input).map_err(|_| {
                error(
                    ErrorKind::Unsupported,
                    "Author actions must use the supported literal command subset",
                )
            })?;
        }
    }
    Ok(())
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        tree_size(&self.initial.tree)?;
        tree_size(&self.expected_tree)?;
        self.initial
            .context
            .validate_source(&self.initial.tree)
            .map_err(from_core)?;
        let c = &self.initial.context;
        if !canonical(&self.handover)
            || self.handover == c.config
            || inside(&self.handover, parent(&c.database))
            || parent(&c.database) == parent(&c.config)
            || parent(&c.database) == "/"
            || self.extra_paths.len() > 4
            || self.reference.is_empty()
            || self.reference.len() > 8
        {
            return Err(error(
                ErrorKind::Invalid,
                "Use separate private index storage and one declared ordinary handover leaf",
            ));
        }
        let mut unique = BTreeSet::new();
        for p in &self.extra_paths {
            if !canonical(p)
                || !inside(p, &c.root)
                || p == &c.root
                || p == &self.handover
                || p == &c.config
                || !unique.insert(p)
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid or duplicate declared practice leaf",
                ));
            }
        }
        for p in std::iter::once(&c.config)
            .chain(std::iter::once(&self.handover))
            .chain(self.extra_paths.iter())
        {
            if !self
                .initial
                .tree
                .nodes
                .get(parent(p))
                .is_some_and(|n| n.directory)
                || !self.initial.tree.access(
                    &self.initial.tree.user,
                    parent(p),
                    Access::CreateEntry,
                )
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Declared editable leaves need an existing writable/searchable supplied parent",
                ));
            }
            for tr in [&self.initial.tree, &self.expected_tree] {
                if tr
                    .nodes
                    .get(p)
                    .is_some_and(|n| n.directory || n.owner != self.initial.tree.user)
                {
                    return Err(error(
                        ErrorKind::Invalid,
                        "Declared editable leaves must be caller-owned ordinary files",
                    ));
                }
            }
        }
        // All potential example/assignment scan branches are accessible; no private visibility emulation.
        for (p, n) in &self.initial.tree.nodes {
            if n.directory
                && inside(p, &c.root)
                && (!self
                    .initial
                    .tree
                    .access(&self.initial.tree.user, p, Access::ListNames)
                    || !self.initial.tree.access(
                        &self.initial.tree.user,
                        p,
                        Access::SearchDirectory,
                    ))
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This unit requires ordinary readable/searchable supplied scan directories",
                ));
            }
        }
        policy(self, &self.expected_tree)?;
        if !self
            .expected_tree
            .nodes
            .get(&self.handover)
            .is_some_and(|n| !n.directory)
            || self
                .expected_tree
                .files
                .get(&self.handover)
                .is_none_or(|s| s.is_empty())
        {
            return Err(error(
                ErrorKind::Invalid,
                "Expected handover must name a nonempty ordinary file",
            ));
        }
        let expected = parse_cfg(&self.expected_tree.files[&c.config], &c.root)?;
        if expected.directories().iter().any(|p| {
            self.expected_tree
                .nodes
                .get(p)
                .is_some_and(|n| !n.directory)
        }) {
            return Err(error(
                ErrorKind::Unsupported,
                "Expected exclusions may name directories only",
            ));
        }
        let initial = self.initial.index.as_ref().ok_or_else(|| {
            error(
                ErrorKind::Source,
                "The supplied stale index is required; do not backfill missing snapshots",
            )
        })?;
        if core::matches_current(initial, c, Some(c), &self.initial.tree, &self.initial.tree)
            .map_err(from_core)?
        {
            return Err(error(
                ErrorKind::Invalid,
                "The authored initial index must genuinely be stale for this task",
            ));
        }
        for step in &self.reference {
            step_shape(step, self)?;
        }
        Ok(())
    }
}
impl Definition {
    /// Structural/read-only admission only. NEVER runs reference or teaching commands.
    pub fn validate(&self) -> Result<()> {
        if !safe_text(&self.id, 48)
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            || self.version == 0
            || !bounded::within(self, MAX_DEFINITION)
        {
            return Err(error(
                ErrorKind::Limit,
                "Definition identifier/version or2MiB source limit is invalid",
            ));
        }
        self.assignment.validate()?;
        self.example.validate()?;
        let a = &self.assignment.initial.context;
        let e = &self.example.initial.context;
        if a.source_id != format!("{}-assignment", self.id)
            || e.source_id != format!("{}-example", self.id)
            || inside(&a.root, &e.root)
            || inside(&e.root, &a.root)
            || a.database == e.database
            || a.config == e.config
            || self.assignment.handover == self.example.handover
            || !self.assignment.extra_paths.is_empty()
        {
            return Err(error(
                ErrorKind::Source,
                "Assignment and example need distinct held role IDs, roots, config, database and handover; extra experiment leaves belong to practice only",
            ));
        }
        let h = &self.teaching;
        let count = h.worked.len() + h.try_steps.len() + h.repair.len();
        if !safe_text(&h.prerequisite_note, 1600)
            || !safe_text(&h.description, 1600)
            || h.concepts.is_empty()
            || h.concepts.len() > 6
            || h.concepts
                .iter()
                .any(|c| !safe_text(&c.title, 100) || !safe_text(&c.text, 1200))
            || h.worked.is_empty()
            || h.try_steps.is_empty()
            || h.worked.len() > 8
            || h.try_steps.len() > 8
            || h.repair.len() > 8
            || count > 24
        {
            return Err(error(
                ErrorKind::Invalid,
                "Use concise prerequisites,1–6concepts and at most8explained actions per phase (24total)",
            ));
        }
        for action in h.worked.iter().chain(&h.try_steps).chain(&h.repair) {
            if !safe_text(&action.title, 100) || !safe_text(&action.explanation, 1200) {
                return Err(error(
                    ErrorKind::Invalid,
                    "Teaching actions need a short title and explanation",
                ));
            }
            step_shape(&action.step, &self.example)?;
        }
        Ok(())
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_DEFINITION {
            return Err(error(ErrorKind::Limit, "Definition exceeds2MiB"));
        }
        let d: Self = serde_json::from_slice(raw)
            .map_err(|_| error(ErrorKind::Invalid, "Malformed or overbound definition"))?;
        d.validate()?;
        Ok(d)
    }
    /// EXPLICIT AUTHORING ONLY. Pack validation may call this; constructors/render/load never do.
    pub fn reference_current(&self, scope: Scope) -> Result<Current> {
        self.validate()?;
        let t = self.task(scope);
        let mut c = initial_current(t);
        for s in &t.reference {
            apply_reference(t, &mut c, s)?;
        }
        Ok(c)
    }
    pub fn validate_reachability(&self) -> Result<()> {
        self.validate()?;
        for scope in [Scope::Assignment, Scope::Example] {
            let current = self.reference_current(scope)?;
            if !report(self.task(scope), &current)?.passed() {
                return Err(error(
                    ErrorKind::Invalid,
                    "Author reference does not meet every actual state objective",
                ));
            }
        }
        let mut example = initial_current(&self.example);
        for a in &self.teaching.worked {
            apply_reference(&self.example, &mut example, &a.step)?;
        }
        if !report(&self.example, &example)?.passed() {
            return Err(error(
                ErrorKind::Invalid,
                "Worked example does not achieve its own goals",
            ));
        }
        for a in &self.teaching.try_steps {
            apply_reference(&self.example, &mut example, &a.step)?;
        }
        for a in &self.teaching.repair {
            apply_reference(&self.example, &mut example, &a.step)?;
        }
        if !report(&self.example, &example)?.passed() {
            return Err(error(
                ErrorKind::Invalid,
                "Teaching repair does not restore actual example goals",
            ));
        }
        Ok(())
    }
    pub fn task(&self, scope: Scope) -> &Task {
        match scope {
            Scope::Assignment => &self.assignment,
            Scope::Example => &self.example,
        }
    }
}
fn initial_current(t: &Task) -> Current {
    Current {
        context: t.initial.context.clone(),
        tree: t.initial.tree.clone(),
        index: t.initial.index.clone(),
        config_draft: t.initial.tree.files[&t.initial.context.config].clone(),
        command: String::new(),
        transcript: vec![],
        checked: false,
    }
}
fn current_valid(t: &Task, c: &Current) -> Result<()> {
    if c.context != t.initial.context
        || c.config_draft.len() > 4096
        || c.command.len() > 4096
        || c.transcript.len() > 20
        || c.transcript
            .iter()
            .any(|r| r.input.len() > 4096 || r.output.len() > 8192 || r.output.contains('\0'))
        || c.transcript.iter().map(Row::bytes).sum::<usize>() > MAX_HISTORY_BYTES
    {
        return Err(error(
            ErrorKind::Source,
            "Wrong context or oversized workspace/history; previous work retained",
        ));
    }
    policy(t, &c.tree)?;
    let i = c.index.as_ref().ok_or_else(|| {
        error(
            ErrorKind::Source,
            "A resumed private-index snapshot is missing; it cannot be silently rebuilt",
        )
    })?;
    // Readonly admission checks index shape/source; false freshness is valid partial work.
    let _ = core::matches_current(
        i,
        &t.initial.context,
        Some(&c.context),
        &t.initial.tree,
        &c.tree,
    )
    .map_err(from_core)?;
    Ok(())
}
fn expected_other(t: &Task, c: &Current) -> bool {
    let cfg = &t.initial.context.config;
    let hand = &t.handover;
    for (p, n) in &c.tree.nodes {
        if p != cfg && p != hand && t.expected_tree.nodes.get(p) != Some(n) {
            return false;
        }
    }
    for (p, n) in &t.expected_tree.nodes {
        if p != cfg && p != hand && c.tree.nodes.get(p) != Some(n) {
            return false;
        }
    }
    for (p, s) in &c.tree.files {
        if p != cfg && p != hand && t.expected_tree.files.get(p) != Some(s) {
            return false;
        }
    }
    for (p, s) in &t.expected_tree.files {
        if p != cfg && p != hand && c.tree.files.get(p) != Some(s) {
            return false;
        }
    }
    true
}
fn report(t: &Task, c: &Current) -> Result<Report> {
    current_valid(t, c)?;
    let cfg = &t.initial.context.config;
    let saved = &c.tree.files[cfg];
    let desired = parse_cfg(&t.expected_tree.files[cfg], &t.initial.context.root)?;
    let actual = parse_cfg(saved, &t.initial.context.root)?;
    let pending = c.config_draft == *saved;
    let pruning = actual == desired;
    let fresh = core::matches_current(
        c.index.as_ref().unwrap(),
        &t.initial.context,
        Some(&c.context),
        &t.initial.tree,
        &c.tree,
    )
    .map_err(from_core)?;
    let hand = c.tree.nodes.get(&t.handover) == t.expected_tree.nodes.get(&t.handover)
        && c.tree.files.get(&t.handover) == t.expected_tree.files.get(&t.handover);
    let hand_message = if !c.tree.nodes.contains_key(&t.handover) {
        "The handover file is missing; copy the current source to the required destination."
    } else if c.tree.files.get(&t.handover) != t.expected_tree.files.get(&t.handover) {
        "The handover contains the wrong text. Compare the current source and the copied file; an archive can have the same name."
    } else if !hand {
        "The handover text is correct, but its permissions or ownership do not match the task."
    } else {
        "The handover has the required current contents and file metadata."
    };
    Ok(Report{checks:vec![Check{key:CheckKey::PendingDraft,passed:pending,message:if pending{"Configuration draft is saved."}else{"Configuration has pending edits. Save it explicitly before checking the applied result."}.into()},Check{key:CheckKey::SavedPruning,passed:pruning,message:if pruning{"Saved exclusions match the task."}else{"Saved exclusions still hide a required branch or omit a required exclusion. Check the intended scan scope, not just whether one query finds a name."}.into()},Check{key:CheckKey::FreshIndex,passed:fresh,message:if fresh{"The saved names match the current allowed tree and saved exclusions."}else{"The private index is stale. Saving text or copying a file does not refresh it; run the supplied updatedb command after the saved changes."}.into()},Check{key:CheckKey::Handover,passed:hand,message:hand_message.into()},Check{key:CheckKey::OtherFiles,passed:expected_other(t,c),message:if expected_other(t,c){"All unrelated files, directories and required final practice leaves are preserved."}else{"A declared practice leaf is not restored to its required final state; originals and neighboring data must remain unchanged."}.into()}]})
}
fn append(c: &mut Current, input: &str, stdout: &str, outcome: Outcome) -> bool {
    let notice = "\n[Simulator: saved output shortened; narrow the query for a complete view.]\n";
    let truncated = stdout.len() > MAX_ROW_OUTPUT;
    let output = if truncated {
        let mut s = short(stdout, MAX_ROW_OUTPUT - notice.len());
        s.push_str(notice);
        s
    } else {
        stdout.to_owned()
    };
    let row = Row {
        input: input.to_owned(),
        output,
        outcome,
        truncated,
    };
    while !c.transcript.is_empty()
        && (c.transcript.len() >= 20
            || c.transcript.iter().map(Row::bytes).sum::<usize>() + row.bytes() > MAX_HISTORY_BYTES)
    {
        c.transcript.remove(0);
    }
    c.transcript.push(row);
    truncated
}
fn normalized_path(cwd: &str, p: &str) -> Result<String> {
    if p.len() > 1024 || p.chars().any(char::is_control) {
        return Err(error(
            ErrorKind::Unsupported,
            "Unsupported path length or control character",
        ));
    }
    let mut parts: Vec<&str> = if p.starts_with('/') {
        vec![]
    } else {
        cwd.split('/').filter(|v| !v.is_empty()).collect()
    };
    for part in p.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            v => parts.push(v),
        }
    }
    if parts.len() > 64 {
        return Err(error(ErrorKind::Limit, "Path component limit"));
    }
    Ok(format!("/{}", parts.join("/")))
}
fn opaque_guard(t: &Task, c: &Current, words: &[String]) -> Result<()> {
    let Some(cmd) = words.first() else {
        return Ok(());
    };
    let db = &t.initial.context.database;
    let folder = parent(db);
    if cmd == "ls" && words.iter().skip(1).all(|w| w.starts_with('-')) && c.tree.cwd == folder {
        return Err(error(
            ErrorKind::Unsupported,
            "This folder holds an opaque simulated index; its artifact row shows its name/count. Generic ls of index storage is outside this activity",
        ));
    }
    if cmd == "find" {
        let start = words.get(1).map(String::as_str).unwrap_or(".");
        let p = normalized_path(&c.tree.cwd, start)?;
        if inside(db, &p) || inside(folder, &p) {
            return Err(error(
                ErrorKind::Unsupported,
                "Generic find from index storage or its ancestors is outside this activity. Search the supplied data root or use locate -d for saved names",
            ));
        }
    }
    for word in words.iter().skip(1).filter(|w| !w.starts_with('-')) {
        let p = normalized_path(&c.tree.cwd, word)?;
        if &p == db || (p == folder && cmd != "cd") {
            return Err(error(
                ErrorKind::Unsupported,
                "The private database is an opaque typed artifact, not ordinary file bytes. Generic cat/cp/rm/ls of the database or its storage folder is not supported",
            ));
        }
    }
    Ok(())
}
fn run_current(t: &Task, c: &mut Current, input: &str) -> Result<Run> {
    current_valid(t, c)?;
    if !safe_text(input, MAX_COMMAND) {
        return Err(error(
            ErrorKind::Invalid,
            "Enter one nonempty supported command of at most4KiB",
        ));
    }
    let words=crate::shell::literal_command(input).map_err(|_|error(ErrorKind::Unsupported,"Use one literal supported command; shell expansion, pipelines and scripts are not executed"))?;
    let mut candidate = c.clone();
    let clean = c.config_draft == c.tree.files[&t.initial.context.config];
    let (stdout, outcome) = if words.as_slice() == ["help"] {
        (HELP.to_owned(), Outcome::Filesystem)
    } else if words.as_slice() == ["show"] {
        return Err(error(
            ErrorKind::Unsupported,
            "Use the native artifact row for typed index status; generic filesystem show does not describe this database",
        ));
    } else if matches!(
        words.first().map(String::as_str),
        Some("locate" | "updatedb")
    ) {
        let out = core::execute_bound(
            &t.initial.context,
            Some(&candidate.context),
            &t.initial.tree,
            &candidate.tree,
            &mut candidate.index,
            input,
        )
        .map_err(from_core)?;
        let k = match out.status {
            core::Status::Found => Outcome::Found,
            core::Status::NoMatch => Outcome::NoMatch,
            core::Status::Rebuilt => Outcome::IndexRebuilt,
        };
        (out.stdout, k)
    } else {
        opaque_guard(t, c, &words)?;
        let out = candidate.tree.execute(input).map_err(|s| {
            error(
                if s.starts_with("Unsupported") || s.starts_with("Only ") {
                    ErrorKind::Unsupported
                } else {
                    ErrorKind::Filesystem
                },
                &s,
            )
        })?;
        policy(t, &candidate.tree)?;
        (out, Outcome::Filesystem)
    };
    if stdout.len() > core::MAX_OUTPUT {
        return Err(error(
            ErrorKind::Limit,
            "Output exceeds32KiB; previous work retained",
        ));
    }
    if clean {
        candidate.config_draft = candidate.tree.files[&t.initial.context.config].clone();
    }
    candidate.checked = false;
    candidate.command.clear();
    let truncated = append(&mut candidate, input, &stdout, outcome);
    current_valid(t, &candidate)?;
    *c = candidate;
    let diagnostic=match outcome{Outcome::IndexRebuilt=>"Private index refreshed from saved configuration.",Outcome::NoMatch=>"No saved names matched. A new file needs a rebuild; -e only filters existing saved names.",_=>""}.into();
    Ok(Run {
        outcome,
        stdout,
        diagnostic,
        truncated_in_history: truncated,
    })
}
fn save_current(t: &Task, c: &mut Current) -> Result<()> {
    current_valid(t, c)?;
    parse_cfg(&c.config_draft, &t.initial.context.root)?;
    let mut next = c.clone();
    next.tree
        .files
        .insert(t.initial.context.config.clone(), next.config_draft.clone());
    next.tree.clock = next
        .tree
        .clock
        .checked_add(1)
        .filter(|n| *n <= 1_000_000)
        .ok_or_else(|| error(ErrorKind::Limit, "Simulated modification counter is full"))?;
    next.tree
        .modified
        .insert(t.initial.context.config.clone(), next.tree.clock);
    next.checked = false;
    current_valid(t, &next)?;
    *c = next;
    Ok(())
}
fn apply_reference(t: &Task, c: &mut Current, step: &Step) -> Result<()> {
    match step {
        Step::SaveConfig { text } => {
            let mut next = c.clone();
            next.config_draft = text.clone();
            save_current(t, &mut next)?;
            *c = next;
        }
        Step::Command { input } => {
            let _ = run_current(t, c, input)?;
        }
    }
    Ok(())
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        let s = Self {
            version: VERSION,
            source_id: d.id.clone(),
            source_version: d.version,
            assignment: initial_current(&d.assignment),
            example: initial_current(&d.example),
            view: View::Introduction,
            selected_example: 0,
        };
        s.validate_for(d)?;
        Ok(s)
    }
    /// Authority-free safety/shape only. This never grades or authorizes imported source rights.
    pub fn validate_shape(&self) -> Result<()> {
        if self.version != VERSION
            || !safe_text(&self.source_id, 48)
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            || self.source_version == 0
            || self.selected_example >= 8
            || !bounded::within(self, MAX_SESSION)
        {
            return Err(error(
                ErrorKind::Source,
                "Malformed or oversized private-index session",
            ));
        }
        for c in [&self.assignment, &self.example] {
            tree_size(&c.tree)?;
            c.context.shape().map_err(from_core)?;
            let i = c.index.as_ref().ok_or_else(|| {
                error(
                    ErrorKind::Source,
                    "Missing resumed index; no backfill is permitted",
                )
            })?;
            if i.context() != &c.context
                || i.entries().len() > 64
                || i.pruned().len() > 16
                || i.entries().iter().map(String::len).sum::<usize>() > core::MAX_NAMES_BYTES
                || i.entries()
                    .iter()
                    .any(|p| !canonical(p) || !inside(p, &c.context.root))
                || c.config_draft.len() > 4096
                || c.command.len() > 4096
                || c.transcript.len() > 20
                || c.transcript.iter().any(|r| {
                    r.input.len() > 4096 || r.output.len() > 8192 || r.output.contains('\0')
                })
                || c.transcript.iter().map(Row::bytes).sum::<usize>() > MAX_HISTORY_BYTES
            {
                return Err(error(
                    ErrorKind::Source,
                    "Malformed context, snapshot or bounded drafts/history",
                ));
            }
        }
        Ok(())
    }
    pub fn validate_for(&self, d: &Definition) -> Result<()> {
        self.validate_shape()?;
        d.validate()?;
        if self.version != VERSION
            || self.source_id != d.id
            || self.source_version != d.version
            || self.selected_example >= d.teaching.try_steps.len()
            || !bounded::within(self, MAX_SESSION)
        {
            return Err(error(
                ErrorKind::Source,
                "Session does not match its held definition/version or saved-data bounds",
            ));
        }
        current_valid(&d.assignment, &self.assignment)?;
        current_valid(&d.example, &self.example)?;
        Ok(())
    }
    pub fn decode(raw: &[u8], d: &Definition) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err(error(ErrorKind::Limit, "Session exceeds1MiB"));
        }
        let s: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                ErrorKind::Invalid,
                "Malformed, erased or overbound saved private-index work",
            )
        })?;
        s.validate_for(d)?;
        Ok(s)
    }
    pub fn current(&self, scope: Scope) -> &Current {
        match scope {
            Scope::Assignment => &self.assignment,
            Scope::Example => &self.example,
        }
    }
    fn current_mut(&mut self, scope: Scope) -> &mut Current {
        match scope {
            Scope::Assignment => &mut self.assignment,
            Scope::Example => &mut self.example,
        }
    }
    fn publish(&mut self, d: &Definition, scope: Scope, next: Current) -> Result<()> {
        current_valid(d.task(scope), &next)?;
        #[derive(Serialize)]
        struct Borrowed<'a> {
            version: u32,
            source_id: &'a str,
            source_version: u32,
            assignment: &'a Current,
            example: &'a Current,
            view: View,
            selected_example: usize,
        }
        let assignment = if scope == Scope::Assignment {
            &next
        } else {
            &self.assignment
        };
        let example = if scope == Scope::Example {
            &next
        } else {
            &self.example
        };
        let prospective = Borrowed {
            version: self.version,
            source_id: &self.source_id,
            source_version: self.source_version,
            assignment,
            example,
            view: self.view,
            selected_example: self.selected_example,
        };
        if !bounded::within(&prospective, MAX_SESSION) {
            return Err(error(
                ErrorKind::Limit,
                "This complete session exceeds1MiB; previous work retained",
            ));
        }
        *self.current_mut(scope) = next;
        Ok(())
    }
    pub fn set_config_draft(&mut self, d: &Definition, scope: Scope, text: String) -> Result<()> {
        self.validate_for(d)?;
        if text.len() > 4096 {
            return Err(error(
                ErrorKind::Limit,
                "Keep pending configuration within4KiB",
            ));
        }
        let mut c = self.current(scope).clone();
        c.config_draft = text;
        c.checked = false;
        self.publish(d, scope, c)
    }
    pub fn set_command(&mut self, d: &Definition, scope: Scope, input: String) -> Result<()> {
        self.validate_for(d)?;
        if input.len() > 4096 {
            return Err(error(ErrorKind::Limit, "Keep the command draft within4KiB"));
        }
        let mut c = self.current(scope).clone();
        c.command = input;
        self.publish(d, scope, c)
    }
    pub fn save_config(&mut self, d: &Definition, scope: Scope) -> Result<()> {
        self.validate_for(d)?;
        let mut c = self.current(scope).clone();
        save_current(d.task(scope), &mut c)?;
        self.publish(d, scope, c)
    }
    pub fn run(&mut self, d: &Definition, scope: Scope) -> Result<Run> {
        self.validate_for(d)?;
        let mut c = self.current(scope).clone();
        let input = c.command.clone();
        let r = run_current(d.task(scope), &mut c, &input)?;
        self.publish(d, scope, c)?;
        Ok(r)
    }
    pub fn checks(&self, d: &Definition, scope: Scope) -> Result<Report> {
        self.validate_for(d)?;
        report(d.task(scope), self.current(scope))
    }
    pub fn check(&mut self, d: &Definition, scope: Scope) -> Result<Report> {
        let r = self.checks(d, scope)?;
        self.current_mut(scope).checked = true;
        Ok(r)
    }
    pub fn artifact(&self, d: &Definition, scope: Scope) -> Result<Artifact> {
        self.validate_for(d)?;
        let t = d.task(scope);
        let c = self.current(scope);
        let i = c.index.as_ref().unwrap();
        Ok(Artifact {
            path: t.initial.context.database.clone(),
            names: i.entries().len(),
            fresh: core::matches_current(
                i,
                &t.initial.context,
                Some(&c.context),
                &t.initial.tree,
                &c.tree,
            )
            .map_err(from_core)?,
        })
    }
    pub fn reset_scope(&mut self, d: &Definition, scope: Scope) -> Result<()> {
        self.validate_for(d)?;
        self.publish(d, scope, initial_current(d.task(scope)))
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<()> {
        self.reset_scope(d, Scope::Example)
    }
    pub fn set_view(&mut self, d: &Definition, view: View) -> Result<()> {
        self.validate_for(d)?;
        self.view = view;
        Ok(())
    }
}
