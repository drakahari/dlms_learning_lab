//! Finite supplied binary-index snapshots; no host APT, acquisition or solver.
//! Selection changes eligibility immediately; refresh never installs software.
use crate::debian_packages::{Architecture, Version};
use crate::debian_packages_workspace as wire;
use crate::debian_sources::{self as sources, Key};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeSet, fmt};

pub const MAX_TARGETS: usize = 6;
pub const MAX_SNAPSHOTS: usize = 3;
pub const MAX_ROWS: usize = 24;
pub const MAX_TOTAL_TEXT: usize = 65536;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub name: String,
    pub version: Version,
    pub architecture: Architecture,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "wire::list::<_,Row,MAX_ROWS>")]
    pub rows: Vec<Row>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub key: Key,
    #[serde(deserialize_with = "wire::list::<_,Snapshot,MAX_SNAPSHOTS>")]
    pub snapshots: Vec<Snapshot>,
    #[serde(default, deserialize_with = "optional::<_,64>")]
    pub published: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub native: Architecture,
    #[serde(deserialize_with = "wire::list::<_,Target,MAX_TARGETS>")]
    pub targets: Vec<Target>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cached {
    pub key: Key,
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub snapshot: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(deserialize_with = "wire::string::<_,8192>")]
    pub saved: String,
    #[serde(deserialize_with = "wire::list::<_,Cached,MAX_TARGETS>")]
    pub cache: Vec<Cached>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Invalid,
    Limit,
    Selection,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: Code,
    pub message: &'static str,
    pub selection: Option<sources::Error>,
}
type Result<T> = std::result::Result<T, Error>;
pub const MAX_DEFINITION_JSON: usize = 262144;
pub const MAX_STATE_JSON: usize = 32768;
fn optional<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    Ok(Option::<wire::Text<N>>::deserialize(d)?.map(|s| s.0))
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(error) = &self.selection {
            error.fmt(f)
        } else {
            f.write_str(self.message)
        }
    }
}
impl std::error::Error for Error {}
fn fail<T>(code: Code, message: &'static str) -> Result<T> {
    Err(Error {
        code,
        message,
        selection: None,
    })
}
fn selection_error(error: sources::Error) -> Error {
    Error {
        code: Code::Selection,
        message: error.message,
        selection: Some(error),
    }
}
pub(crate) fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-+._".contains(&b))
}
impl Definition {
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_DEFINITION_JSON {
            return fail(Code::Limit, "Source definition exceeds 256 KiB");
        }
        let d: Self = serde_json::from_str(raw).map_err(|_| Error {
            selection: None,
            code: Code::Invalid,
            message: "Invalid or unbounded source definition",
        })?;
        d.validate()?;
        Ok(d)
    }

    pub fn validate(&self) -> Result<()> {
        if self.native == Architecture::All {
            return fail(Code::Invalid, "A native machine architecture is required");
        }
        if self.targets.is_empty() || self.targets.len() > MAX_TARGETS {
            return fail(Code::Limit, "Supply one to six source targets");
        }
        let mut targets = BTreeSet::new();
        let mut total = 0usize;
        for t in &self.targets {
            t.key.validate().map_err(|_| Error {
                selection: None,
                code: Code::Invalid,
                message: "Invalid canonical source target",
            })?;
            total += t.key.uri.len() + t.key.suite.len() + t.key.component.len();
            if !targets.insert(&t.key) {
                return fail(Code::Invalid, "Repeated supplied source target");
            }
            if t.snapshots.is_empty() || t.snapshots.len() > MAX_SNAPSHOTS {
                return fail(
                    Code::Limit,
                    "Supply one to three metadata snapshots per target",
                );
            }
            let mut snapshots = BTreeSet::new();
            for s in &t.snapshots {
                if !id(&s.id) || !snapshots.insert(&s.id) {
                    return fail(
                        Code::Invalid,
                        "Snapshot identifiers must be bounded and distinct within a source target",
                    );
                }
                if s.rows.len() > MAX_ROWS {
                    return fail(Code::Limit, "A snapshot has at most 24 metadata rows");
                }
                total += s.id.len();
                let mut rows = BTreeSet::new();
                for row in &s.rows {
                    if !id(&row.name) || row.name.len() < 2 || row.name.contains('_') {
                        return fail(Code::Invalid, "Invalid bounded binary package name");
                    }
                    // Version is already a validated Debian Version, not lexical comparison.
                    let architecture = match row.architecture {
                        Architecture::Amd64 => 0,
                        Architecture::Arm64 => 1,
                        Architecture::All => 2,
                    };
                    if !rows.insert((&row.name, row.version.as_str(), architecture)) {
                        return fail(Code::Invalid, "Repeated package/version/architecture row");
                    }
                    total += row.name.len() + row.version.as_str().len();
                }
            }
            if t.published.as_ref().is_some_and(|p| !snapshots.contains(p)) {
                return fail(
                    Code::Invalid,
                    "Published snapshot must belong to the same target",
                );
            }
        }
        if total > MAX_TOTAL_TEXT {
            return fail(Code::Limit, "Supplied source metadata exceeds 64 KiB");
        }
        Ok(())
    }
    fn target(&self, key: &Key) -> Option<&Target> {
        self.targets.iter().find(|t| &t.key == key)
    }
}
impl State {
    pub fn from_json(d: &Definition, raw: &str) -> Result<Self> {
        if raw.len() > MAX_STATE_JSON {
            return fail(Code::Limit, "Saved source workspace exceeds 32 KiB");
        }
        let s: Self = serde_json::from_str(raw).map_err(|_| Error {
            selection: None,
            code: Code::Invalid,
            message: "Invalid or unbounded saved source workspace",
        })?;
        s.validate(d)?;
        Ok(s)
    }

    pub fn validate(&self, d: &Definition) -> Result<()> {
        d.validate()?;
        sources::select(&sources::parse(&self.saved).map_err(selection_error)?)
            .map_err(selection_error)?;
        if self.cache.len() > MAX_TARGETS {
            return fail(
                Code::Limit,
                "At most six matching supplied indexes can be cached",
            );
        }
        let mut keys = BTreeSet::new();
        for c in &self.cache {
            c.key.validate().map_err(|_| Error {
                selection: None,
                code: Code::Invalid,
                message: "Invalid cached source key",
            })?;
            if c.snapshot.len() > 64 || !keys.insert(&c.key) {
                return fail(Code::Invalid, "Repeated or invalid cached target");
            }
            let Some(t) = d.target(&c.key) else {
                return fail(Code::Invalid, "Cached index needs a supplied source target");
            };
            if !t.snapshots.iter().any(|s| s.id == c.snapshot) {
                return fail(
                    Code::Invalid,
                    "Cached snapshot does not belong to this source target",
                );
            }
        }
        Ok(())
    }
    /// Invalid edits are atomic. Retained cache bytes are not active by themselves.
    pub fn save(&mut self, d: &Definition, text: &str) -> Result<()> {
        self.validate(d)?;
        sources::select(&sources::parse(text).map_err(selection_error)?)
            .map_err(selection_error)?;
        self.saved = text.into();
        Ok(())
    }
    pub fn active(&self, d: &Definition) -> Result<Vec<Key>> {
        self.validate(d)?;
        Ok(
            sources::select(&sources::parse(&self.saved).expect("validated saved fields"))
                .expect("validated target selection")
                .keys,
        )
    }
    /// Each active target refreshes independently from its supplied publication.
    /// Unknown/unavailable targets report failure without erasing old matching data.
    pub fn refresh(&mut self, d: &Definition) -> Result<Vec<Refresh>> {
        let keys = self.active(d)?;
        let mut results = vec![];
        for key in keys {
            let publication = d.target(&key).and_then(|t| t.published.as_ref());
            let status = if let Some(snapshot) = publication {
                if let Some(old) = self.cache.iter_mut().find(|c| c.key == key) {
                    old.snapshot = snapshot.clone();
                } else {
                    self.cache.push(Cached {
                        key: key.clone(),
                        snapshot: snapshot.clone(),
                    });
                }
                RefreshStatus::Updated
            } else {
                RefreshStatus::Unavailable
            };
            results.push(Refresh { key, status });
        }
        self.validate(d)?;
        Ok(results)
    }
    pub fn indexes(&self, d: &Definition) -> Result<Vec<Index>> {
        let keys = self.active(d)?;
        let mut result = vec![];
        for key in keys {
            let target = d.target(&key);
            let cached = self.cache.iter().find(|c| c.key == key);
            let status = match cached {
                None => IndexStatus::Missing,
                Some(c) => {
                    if target.and_then(|t| t.published.as_ref()) == Some(&c.snapshot) {
                        IndexStatus::Current
                    } else {
                        IndexStatus::Retained
                    }
                }
            };
            result.push(Index {
                key,
                status,
                snapshot: cached.map(|c| c.snapshot.clone()),
            });
        }
        Ok(result)
    }
    /// Exact-name evidence, not an APT-policy candidate resolver. Order is source
    /// selection then supplied snapshot order; duplicate provenance is retained.
    pub fn query(&self, d: &Definition, name: &str) -> Result<Vec<Evidence>> {
        if !id(name) || name.len() < 2 || name.contains('_') {
            return fail(Code::Invalid, "Enter one ordinary binary package name");
        }
        let mut rows = vec![];
        for index in self.indexes(d)? {
            let Some(snapshot) = index.snapshot else {
                continue;
            };
            let target = d
                .target(&index.key)
                .expect("validated cache belongs to supplied target");
            let s = target
                .snapshots
                .iter()
                .find(|s| s.id == snapshot)
                .expect("validated snapshot belongs to target");
            for row in &s.rows {
                if row.name == name
                    && (row.architecture == d.native || row.architecture == Architecture::All)
                {
                    rows.push(Evidence {
                        key: index.key.clone(),
                        snapshot: snapshot.clone(),
                        row: row.clone(),
                    });
                }
            }
        }
        Ok(rows)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshStatus {
    Updated,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refresh {
    pub key: Key,
    pub status: RefreshStatus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexStatus {
    Missing,
    Current,
    Retained,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    pub key: Key,
    pub status: IndexStatus,
    pub snapshot: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub key: Key,
    pub snapshot: String,
    pub row: Row,
}
