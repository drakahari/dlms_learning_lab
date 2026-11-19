//! Source-bound virtual application-documentation placement. Never executes files.
use super::Tree;
use crate::whereis::{self, Category, Context, Query};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Write};
const VERSION: u32 = 1;
const MAX_BYTES: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub category: Category,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub path: String,
}
fn slots<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Slot(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] String);
    crate::cron_workspace::sequence::<D, Slot, 4>(d).map(|v| v.into_iter().map(|s| s.0).collect())
}
fn selected<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Selection>, D::Error> {
    crate::cron_workspace::sequence::<D, Selection, 12>(d)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Box<Tree>,
    pub expected: Box<Tree>,
    pub context: Context,
    #[serde(deserialize_with = "slots")]
    pub mutable_files: Vec<String>,
    #[serde(deserialize_with = "selected")]
    pub selected: Vec<Selection>,
}
struct Size(usize);
impl Write for Size {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if b.len() > MAX_BYTES.saturating_sub(self.0) {
            return Err(std::io::Error::other("Deployment definition exceeds1MiB"));
        }
        self.0 += b.len();
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bounded(t: &Tree) -> bool {
    t.nodes.len() <= 64
        && t.files.len() <= 64
        && t.modified.len() <= 64
        && t.files
            .values()
            .try_fold(0usize, |n, s| n.checked_add(s.len()))
            .is_some_and(|n| n <= 256 * 1024)
        && t.valid()
}
fn same(a: &Tree, b: &Tree) -> bool {
    a.nodes == b.nodes
        && a.files == b.files
        && a.users == b.users
        && a.user == b.user
        && a.home == b.home
        && a.umask == b.umask
        && a.command_location == b.command_location
}
fn short(s: String) -> String {
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].into()
}
impl Definition {
    fn shape(&self) -> Result<(), String> {
        if self.version != VERSION
            || self.mutable_files.is_empty()
            || self.mutable_files.len() > 4
            || !bounded(&self.initial)
            || !bounded(&self.expected)
        {
            return Err("Invalid bounded deployment source/version".into());
        }
        self.context
            .validate_for(&self.initial)
            .map_err(|e| e.to_string())?;
        let marker = self
            .initial
            .command_location
            .as_ref()
            .ok_or("Deployment needs its supplied command-search marker")?;
        marker
            .validate_for(&self.initial)
            .map_err(|e| e.to_string())?;
        if marker.source_id != self.context.source_id
            || marker.names != self.context.names
            || self.context.directories.len() != 3
        {
            return Err(
                "Deployment category source differs from its supplied command-search context"
                    .into(),
            );
        }
        let Some(binary) = self
            .context
            .directories
            .iter()
            .find(|d| d.category == Category::Binary)
        else {
            return Err("Deployment requires its supplied binary category".into());
        };
        if !marker.path.split(':').any(|p| p == binary.path) {
            return Err(
                "Supplied binary folder must belong to the fixed command-search path".into(),
            );
        }
        let mut seen = BTreeSet::new();
        for p in &self.mutable_files {
            if !super::canonical(p)
                || p == "/"
                || !seen.insert(p)
                || !self
                    .initial
                    .nodes
                    .get(super::parent(p))
                    .is_some_and(|n| n.directory)
                || self
                    .initial
                    .nodes
                    .get(p)
                    .is_some_and(|n| n.directory || n.owner != self.initial.user)
                || self
                    .expected
                    .nodes
                    .get(p)
                    .is_some_and(|n| n.directory || n.owner != self.initial.user)
                || marker.path.split(':').any(|d| super::parent(p) == d)
            {
                return Err("Declare distinct caller-owned regular file slots outside every protected PATH folder".into());
            }
        }
        Ok(())
    }
    /// Valid partial work changes only declared ordinary leaves; all other data is fixed.
    pub fn bound(&self, current: &Tree) -> Result<(), String> {
        self.shape()?;
        if !bounded(current)
            || current.command_location != self.initial.command_location
            || current.users != self.initial.users
            || current.user != self.initial.user
            || current.home != self.initial.home
            || current.umask != self.initial.umask
        {
            return Err("Deployment source, caller, home, creation mask or resource bounds differ; previous work retained".into());
        }
        for (p, old) in &self.initial.nodes {
            if self.mutable_files.contains(p) {
                continue;
            }
            if current.nodes.get(p) != Some(old)
                || current.files.get(p) != self.initial.files.get(p)
            {
                return Err(short(format!(
                    "Preserve supplied folders, application binary and unrelated files: {p}"
                )));
            }
        }
        for (p, now) in &current.nodes {
            if self.mutable_files.contains(p) {
                if now.directory || now.owner != self.initial.user {
                    return Err(
                        "Editable deployment slots must remain caller-owned regular files".into(),
                    );
                }
            } else if !self.initial.nodes.contains_key(p) {
                return Err(short(format!(
                    "This task permits changes only to the named deployment files; unexpected path: {p}"
                )));
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.bound(&self.expected)?;
        let expected_count = self.context.names.len() * self.context.directories.len();
        if self.selected.len() != expected_count || self.selected.len() > 12 {
            return Err("Supply one selected path per known name and category".into());
        }
        let mut seen = BTreeSet::new();
        for s in &self.selected {
            if !self.context.names.contains(&s.name)
                || !seen.insert((s.category, &s.name))
                || !super::canonical(&s.path)
                || !self
                    .expected
                    .nodes
                    .get(&s.path)
                    .is_some_and(|n| !n.directory)
            {
                return Err("Invalid or duplicate deployment category outcome".into());
            }
            let o = whereis::evaluate(
                &self.context,
                &self.initial,
                &self.expected,
                &Query {
                    category: s.category,
                    name: s.name.clone(),
                },
            )
            .map_err(|e| e.to_string())?;
            if o.paths.as_slice() != [s.path.as_str()] {
                return Err(
                    "Expected deployment does not select its declared category path".into(),
                );
            }
        }
        if same(&self.initial, &self.expected) {
            return Err("Deployment task must require a meaningful file repair".into());
        }
        serde_json::to_writer(&mut Size(0), self)
            .map_err(|_| "Deployment source definition exceeds1MiB".to_string())?;
        Ok(())
    }
    pub fn valid_for(&self, initial: &Tree) -> bool {
        self.initial.as_ref() == initial && self.validate().is_ok()
    }
    pub fn check(&self, current: &Tree) -> (bool, String) {
        if let Err(e) = self.validate() {
            return (false, short(format!("Invalid deployment objective: {e}")));
        }
        if let Err(e) = self.bound(current) {
            return (false, short(e));
        }
        for s in &self.selected {
            let query = Query {
                category: s.category,
                name: s.name.clone(),
            };
            match whereis::evaluate(&self.context, &self.initial, current, &query) {
                Ok(o) if o.paths.as_slice() == [s.path.as_str()] => {}
                Ok(o) => {
                    return (
                        false,
                        short(format!(
                            "{:?} lookup for {} currently finds {}. Check the supplied search folder and filename; finding a name does not verify its release contents.",
                            s.category,
                            s.name,
                            if o.paths.is_empty() {
                                "no matching name".into()
                            } else {
                                o.paths.join(", ")
                            }
                        )),
                    );
                }
                Err(e) => {
                    return (
                        false,
                        short(format!("Cannot check the {:?} category: {e}", s.category)),
                    );
                }
            }
        }
        if !same(&self.expected, current) {
            for p in &self.mutable_files {
                if self.expected.nodes.get(p) != current.nodes.get(p)
                    || self.expected.files.get(p) != current.files.get(p)
                {
                    let why = if self.expected.nodes.contains_key(p) {
                        "Check the release contents and permission bits; a matching filename alone is not enough."
                    } else {
                        "The explicitly retired file is still present. Preserve the neighboring documentation."
                    };
                    return (false, short(format!("Deployment differs at {p}. {why}")));
                }
            }
            return (false, "Required complete file handover differs".into());
        }
        (true,"Current binary, manual and source are correctly located; complete protected state is preserved".into())
    }
    pub fn decode(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_BYTES {
            return Err("Deployment JSON exceeds1MiB".into());
        }
        let d: Self =
            serde_json::from_slice(raw).map_err(|_| "Invalid deployment definition JSON")?;
        d.validate()?;
        Ok(d)
    }
    /// All queries and mutations are in-process; never evaluates the contents of a file.
    pub fn execute(&self, current: &mut Tree, input: &str) -> Result<String, String> {
        self.validate()?;
        self.bound(current)?;
        if input.len() > 4096 {
            return Err("Command exceeds4,096bytes".into());
        }
        let words = crate::shell::literal_command(input)?;
        match words.first().map(String::as_str) {
            Some("whereis") => {
                return whereis::execute_bound(
                    &self.context,
                    Some(&self.context),
                    &self.initial,
                    current,
                    input,
                )
                .map(|o| o.stdout)
                .map_err(|e| e.to_string());
            }
            Some("which" | "type") => {
                let c = self
                    .initial
                    .command_location
                    .as_ref()
                    .expect("validated marker");
                let o = crate::command_location::execute_bound(
                    c,
                    current.command_location.as_ref(),
                    &self.initial,
                    current,
                    input,
                )
                .map_err(|e| e.to_string())?;
                return if o.success() {
                    Ok(o.stdout)
                } else {
                    Err(o.diagnostic)
                };
            }
            Some("help") if words.len() == 1 => {
                return Ok(format!(
                    "{}\nThis activity adds whereis -b -B DIR -f NAME, -m -M DIR and -s -S DIR for supplied categories only. Discovery names do not prove executable permission or release contents. Only the task’s explicitly named deployment files may change. help/show are simulator references.",
                    super::HELP
                ));
            }
            Some("show") if words.len() == 1 => {
                return Ok(format!(
                    "Simulator state · User: {} · Working directory: {}\nFixed PATH: {}\nSupplied category folders:\n{}",
                    current.user,
                    current.cwd,
                    self.initial.command_location.as_ref().unwrap().path,
                    self.context
                        .directories
                        .iter()
                        .map(|d| format!("{:?}: {}", d.category, d.path))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
            Some("find") => return current.find(&words[1..])?.terminal(),
            _ => {}
        }
        let mut candidate = current.clone();
        let out = candidate.command(&words)?;
        self.bound(&candidate)?;
        if out.len() > super::MAX_OUTPUT {
            return Err("Output exceeds32KiB; previous work retained".into());
        }
        *current = candidate;
        Ok(out)
    }
}
