//! Bounded optional-media dependency projection. Virtual facts only; never host operations.
use crate::storage as core;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, core::Error>;
fn error(kind: core::ErrorKind, line: Option<usize>, message: impl Into<String>) -> core::Error {
    core::Error {
        kind,
        line,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub mount: core::Entry,
    pub optional: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    entries: Vec<Entry>,
    normalized: String,
}
impl Policy {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// Exact existing-core text, with optional dependency metadata kept in this
    /// policy. Actor-aware activities can reuse storage without changing its wire.
    pub fn core_configuration(&self) -> &str {
        &self.normalized
    }
    pub fn parse(raw: &str) -> Result<Self> {
        if raw.len() > core::MAX_CONFIG_BYTES || raw.lines().count() > 128 {
            return Err(error(
                core::ErrorKind::Limit,
                None,
                "Use at most16KiB/128lines of virtual mount configuration",
            ));
        }
        if raw
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err(error(
                core::ErrorKind::Invalid,
                None,
                "Mount text contains an unsupported control character",
            ));
        }
        let mut normalized = String::with_capacity(raw.len());
        let mut optional = BTreeMap::new();
        for (n, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                normalized.push_str(line);
                normalized.push('\n');
                continue;
            }
            let fields = line.split_ascii_whitespace().take(7).collect::<Vec<_>>();
            if fields.len() != 6 {
                return Err(error(
                    core::ErrorKind::Invalid,
                    Some(n + 1),
                    "Use six fields: source, mountpoint, filesystem, options, dump and check order",
                ));
            }
            let mut remaining = vec![];
            let mut nofail = false;
            for (index, token) in fields[3].split(',').enumerate() {
                if index >= 16 {
                    return Err(error(
                        core::ErrorKind::Limit,
                        Some(n + 1),
                        "Too many mount options",
                    ));
                }
                if token == "nofail" {
                    nofail = true;
                } else {
                    remaining.push(token);
                }
            }
            optional.insert(n + 1, nofail);
            let options = if remaining.is_empty() {
                "defaults".into()
            } else {
                remaining.join(",")
            };
            normalized.push_str(&format!(
                "{} {} {} {} {} {}\n",
                fields[0], fields[1], fields[2], options, fields[4], fields[5]
            ));
        }
        // Never truncate a field to fit a budget; reject a too-large normalized form.
        if normalized.len() == core::MAX_CONFIG_BYTES + 1 && normalized.ends_with('\n') {
            normalized.pop();
        }
        if normalized.len() > core::MAX_CONFIG_BYTES {
            return Err(error(
                core::ErrorKind::Limit,
                None,
                "Normalized virtual configuration exceeds16KiB; shorten comments or whitespace",
            ));
        }
        let entries=core::parse(&normalized).map_err(|mut e|{
   if e.kind==core::ErrorKind::Unsupported&&e.message.starts_with("Supported options:"){e.message="Supported options here: defaults, ro, rw, auto, noauto and nofail. Other options are outside this subset".into();}e
  })?.into_iter().map(|mount|Entry{optional:optional[&mount.line],mount}).collect();
        Ok(Self {
            entries,
            normalized,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub inventory: core::Inventory,
    #[serde(deserialize_with = "present_media")]
    pub present: BTreeSet<String>,
}
fn present_media<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<String>, D::Error> {
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,48>")] String);
    let list = crate::cron_workspace::sequence::<D, Name, 8>(d)?;
    let mut out = BTreeSet::new();
    for Name(name) in list {
        if !out.insert(name) {
            return Err(serde::de::Error::custom(
                "Duplicate present-medium identity",
            ));
        }
    }
    Ok(out)
}
impl Environment {
    pub fn validate(&self) -> Result<()> {
        if self.inventory.swap().is_some() {
            return Err(error(core::ErrorKind::Unsupported, None,
                "Removable-media/caller-policy activities do not enable swap inventory; use the source-enabled storage workspace for that separate subset"));
        }
        if self.present.len() > core::MAX_VOLUMES
            || self
                .present
                .iter()
                .any(|id| !self.inventory.volumes().contains_key(id))
        {
            return Err(error(
                core::ErrorKind::Invalid,
                None,
                "Present media must refer only to the supplied bounded inventory",
            ));
        }
        Ok(())
    }
    pub fn blkid(&self) -> Result<String> {
        self.validate()?;
        Ok(self
            .inventory
            .devices()
            .iter()
            .filter(|(_, id)| self.present.contains(*id))
            .map(|(device, id)| {
                let v = &self.inventory.volumes()[id];
                format!(
                    "{device}: UUID=\"{}\" TYPE=\"{}\"",
                    v.uuid,
                    v.filesystem.name()
                )
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }
    fn availability(&self, entry: &core::Entry) -> Result<()> {
        let id = self.inventory.resolve(&entry.source).map_err(|mut e| {
            e.line = Some(entry.line);
            if e.kind==core::ErrorKind::UnknownSource{e.message="The configured UUID or device alias does not match a supplied filesystem. Compare the source with Supplied media sample.".into();}
            e
        })?;
        if !self.present.contains(id) {
            return Err(error(
                core::ErrorKind::UnknownSource,
                Some(entry.line),
                "This filesystem's device is absent in the selected sample",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dependency {
    NotEnrolled,
    Wanted,
    Required,
}
impl Entry {
    pub fn dependency(&self) -> Dependency {
        if !self.mount.automatic {
            Dependency::NotEnrolled
        } else if self.optional {
            Dependency::Wanted
        } else {
            Dependency::Required
        }
    }
    pub fn ordered_before_local_fs(&self) -> bool {
        self.dependency() == Dependency::Required
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Mounted,
    Manual,
    Failed(core::Error),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub target: String,
    pub dependency: Dependency,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub state: core::State,
    pub rows: Vec<Row>,
}
impl Projection {
    /// Only the modeled table's required attempts, never a claim of boot completion.
    pub fn required_attempts_satisfied(&self) -> bool {
        self.rows
            .iter()
            .filter(|r| r.dependency == Dependency::Required)
            .all(|r| r.outcome == Outcome::Mounted)
    }
}
impl Policy {
    pub fn startup(&self, env: &Environment) -> Result<Projection> {
        env.validate()?;
        let mut state = core::State::new(
            env.inventory.clone(),
            self.normalized.clone(),
            BTreeMap::new(),
        )?;
        let mut rows = vec![];
        for entry in &self.entries {
            let dependency = entry.dependency();
            let outcome = if dependency == Dependency::NotEnrolled {
                Outcome::Manual
            } else {
                match env
                    .availability(&entry.mount)
                    .and_then(|_| state.mount(&entry.mount.target))
                {
                    Ok(_) => Outcome::Mounted,
                    Err(e) => Outcome::Failed(e),
                }
            };
            rows.push(Row {
                target: entry.mount.target.clone(),
                dependency,
                outcome,
            });
        }
        Ok(Projection { state, rows })
    }
    pub fn manual_mount(
        &self,
        env: &Environment,
        state: &mut core::State,
        target: &str,
    ) -> Result<core::MountOutcome> {
        env.validate()?;
        if state.inventory() != &env.inventory || state.configuration() != self.normalized.as_str()
        {
            return Err(error(
                core::ErrorKind::Invalid,
                None,
                "Manual mount needs the matching supplied context and current saved policy",
            ));
        }
        if state
            .live()
            .values()
            .any(|m| !env.present.contains(&m.volume))
        {
            return Err(error(
                core::ErrorKind::Unsupported,
                None,
                "Hot removal of an already mounted medium is outside this settled-state exercise",
            ));
        }
        let entry = self
            .entries
            .iter()
            .find(|e| e.mount.target == target)
            .ok_or_else(|| {
                error(
                    core::ErrorKind::Invalid,
                    None,
                    "No saved virtual entry names that mountpoint",
                )
            })?;
        env.availability(&entry.mount)?;
        state.mount(target)
    }
}
