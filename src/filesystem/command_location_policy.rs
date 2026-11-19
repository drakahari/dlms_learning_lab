//! Bounded source/expected file-mode policy for optional command-location tasks.
//! No host actions. Source declarations are authority only when paired by the catalog.
use super::{Requirement, Tree};
use crate::command_location::{self as core, Mode, Query};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Write};
pub const VERSION: u32 = 1;
pub const MAX_DEFINITION_BYTES: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub path: String,
}
fn selected<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Selection>, D::Error> {
    crate::cron_workspace::sequence::<D, Selection, 4>(d)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Box<Tree>,
    pub expected: Box<Tree>,
    #[serde(deserialize_with = "selected")]
    pub selected: Vec<Selection>,
}
struct Size(usize);
impl Write for Size {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        if data.len() > MAX_DEFINITION_BYTES.saturating_sub(self.0) {
            return Err(std::io::Error::other(
                "Command-location definition exceeds1MiB",
            ));
        }
        self.0 += data.len();
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn preflight(tree: &Tree) -> Result<(), String> {
    if tree.nodes.len() > 64 || tree.files.len() > 64 || !tree.valid() {
        return Err(
            "Command-location tree needs valid bounded ordinary-user data (at most64entries)"
                .into(),
        );
    }
    Ok(())
}
fn mode_leaf(initial: &Tree, key: &str) -> bool {
    let Some(context) = &initial.command_location else {
        return false;
    };
    let Some(node) = initial.nodes.get(key) else {
        return false;
    };
    !node.directory
        && node.owner == initial.user
        && context
            .names
            .iter()
            .any(|name| key.rsplit('/').next() == Some(name.as_str()))
        && context.path.split(':').any(|dir| dir == super::parent(key))
}
/// Allowed CURRENT states need not be solved; only source-owned named file modes vary.
/// Fixed initial is supplied by the trusted Goal/lesson, never current.clone().
pub fn bound(initial: &Tree, current: &Tree) -> Result<(), String> {
    preflight(initial)?;
    preflight(current)?;
    let context = initial
        .command_location
        .as_ref()
        .ok_or("Missing held command-location context")?;
    context.validate_for(initial).map_err(|e| e.to_string())?;
    if current.command_location.as_ref() != Some(context)
        || current.users != initial.users
        || current.user != initial.user
        || current.home != initial.home
        || current.umask != initial.umask
    {
        return Err(
            "Command-location context, caller, home or mask differs from its held source".into(),
        );
    }
    if current.files != initial.files || !current.nodes.keys().eq(initial.nodes.keys()) {
        return Err(
            "File paths and bytes must remain exactly as supplied in this mode-repair activity"
                .into(),
        );
    }
    for (key, old) in &initial.nodes {
        let now = &current.nodes[key];
        if now.owner != old.owner || now.group != old.group || now.directory != old.directory {
            return Err("File ownership and kinds must remain exactly as supplied".into());
        }
        if now.mode != old.mode && !mode_leaf(initial, key) {
            return Err("Only permissions of supplied caller-owned regular command files may change; folders and unrelated files stay fixed".into());
        }
    }
    // project clears optional context, so this does not recurse through Tree.valid.
    context.validate_for(current).map_err(|e| e.to_string())?;
    Ok(())
}
fn semantic_same(a: &Tree, b: &Tree) -> bool {
    a.nodes == b.nodes
        && a.files == b.files
        && a.users == b.users
        && a.user == b.user
        && a.home == b.home
        && a.umask == b.umask
        && a.command_location == b.command_location
}
fn short(value: String) -> String {
    let mut end = value.len().min(512);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].into()
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION || self.selected.is_empty() || self.selected.len() > 4 {
            return Err("Invalid command-location definition version or selection count".into());
        }
        preflight(&self.initial)?;
        preflight(&self.expected)?;
        let context = self
            .initial
            .command_location
            .as_ref()
            .ok_or("Command-location definition needs an opted initial tree")?;
        context.shape().map_err(|e| e.to_string())?;
        let mut seen = BTreeSet::new();
        if self.selected.len() != context.names.len()
            || self.selected.iter().any(|s| {
                s.name.len() > 48
                    || !context.names.contains(&s.name)
                    || !seen.insert(s.name.as_str())
                    || !super::canonical(&s.path)
                    || !self
                        .expected
                        .nodes
                        .get(&s.path)
                        .is_some_and(|n| !n.directory)
            })
        {
            return Err(
                "Supply one distinct canonical selected-file outcome for every known command name"
                    .into(),
            );
        }
        // Borrowed structural checks BEFORE bounded serialization or any model clone.
        bound(&self.initial, &self.expected)?;
        serde_json::to_writer(&mut Size(0), self)
            .map_err(|_| "Command-location source/expected definition exceeds1MiB".to_string())?;
        for row in &self.selected {
            let output = core::evaluate(
                context,
                &self.initial,
                &self.expected,
                &Query {
                    mode: Mode::Which,
                    name: row.name.clone(),
                },
            )
            .map_err(|e| e.to_string())?;
            if output.paths.as_slice() != [row.path.as_str()] {
                return Err("Expected metadata does not select its declared command path".into());
            }
        }
        Ok(())
    }
    /// Source admission ONLY. A changed live state is deliberately not equal to initial.
    pub fn valid_for(&self, initial: &Tree) -> bool {
        self.validate().is_ok() && self.initial.as_ref() == initial
    }
    pub fn check(&self, current: &Tree) -> (bool, String) {
        if let Err(e) = self.validate() {
            return (
                false,
                short(format!("Invalid command-location objective: {e}")),
            );
        }
        if let Err(e) = bound(&self.initial, current) {
            return (false, short(e));
        }
        let context = self
            .initial
            .command_location
            .as_ref()
            .expect("validated source");
        for row in &self.selected {
            match core::evaluate(
                context,
                &self.initial,
                current,
                &Query {
                    mode: Mode::Which,
                    name: row.name.clone(),
                },
            ) {
                Ok(output) if output.paths.as_slice() == [row.path.as_str()] => {}
                Ok(output) => {
                    let actual = output
                        .paths
                        .first()
                        .map_or("no selectable file", String::as_str);
                    return (
                        false,
                        short(format!(
                            "{} currently selects {actual}. Inspect its file execute bits and the supplied PATH order.",
                            row.name
                        )),
                    );
                }
                Err(e) => return (false, short(format!("Lookup cannot be checked: {e}"))),
            }
        }
        if !semantic_same(current, &self.expected) {
            if let Some((path, _)) = self
                .expected
                .nodes
                .iter()
                .find(|(p, n)| current.nodes.get(*p) != Some(*n))
            {
                return (
                    false,
                    short(format!(
                        "File permissions differ from the handover requirement: {path}. Preserve all original bits except the stated change."
                    )),
                );
            }
            return (
                false,
                "Complete virtual metadata differs from the required handover".into(),
            );
        }
        (
            true,
            "Required command paths and complete preserved file metadata match".into(),
        )
    }
    pub fn decode(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION_BYTES {
            return Err("Command-location definition JSON exceeds1MiB".into());
        }
        let d: Self = serde_json::from_slice(raw)
            .map_err(|_| "Invalid bounded command-location definition JSON".to_string())?;
        d.validate()?;
        Ok(d)
    }
}
/// A caller must still validate this reference against the authored Scenario.initial.
pub fn unique(requirements: &[Requirement]) -> Result<Option<&Definition>, String> {
    if requirements.len() > 20 {
        return Err("Too many filesystem objective rows".into());
    }
    let mut found = None;
    for requirement in requirements {
        if let Requirement::CommandLocation { definition } = requirement {
            if found.is_some() {
                return Err("Only one command-location source reference is allowed".into());
            }
            definition.validate()?;
            found = Some(definition.as_ref());
        }
    }
    Ok(found)
}
/// Source/operational-role matrix for engine callers; no implicit old-goal fallthrough.
pub fn for_current<'a>(
    requirements: &'a [Requirement],
    current: &Tree,
) -> Result<Option<&'a Definition>, String> {
    match (unique(requirements)?, current.command_location.as_ref()) {
        (None, None) => Ok(None),
        (Some(definition), Some(_)) => {
            bound(&definition.initial, current)?;
            Ok(Some(definition))
        }
        _ => Err(
            "Command-location metadata and unique authored filesystem source do not match".into(),
        ),
    }
}
