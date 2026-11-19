//! Capacity accounting over bounded supplied virtual facts, not a disk filesystem.
//! No df/fsck CLI, host I/O, mounts, real ext4 allocation or repair is implemented.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: u32 = 1;
pub const MAX_FILES: usize = 128;
pub const MAX_NAME_BYTES: usize = 48;
pub const MAX_LOGICAL_BYTES: u64 = 1 << 40;
pub const MAX_POOL: u64 = 1 << 32;
pub const MAX_STATE_BYTES: usize = 32_768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Limit,
    NoBlocks,
    NoInodes,
    Protected,
    Unknown,
    Stale,
}
pub type Result<T> = std::result::Result<T, Error>;
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "Invalid virtual filesystem facts or path",
            Self::Limit => "Virtual capacity model resource limit",
            Self::NoBlocks => "Insufficient blocks available to ordinary files",
            Self::NoInodes => "No free inode for another file",
            Self::Protected => "Supplied retained file cannot be changed or removed",
            Self::Unknown => "Unknown virtual file",
            Self::Stale => "Preview does not match the entire current filesystem state",
        })
    }
}
impl std::error::Error for Error {}

fn component(s: &str) -> bool {
    !s.is_empty()
        && s.len() < MAX_NAME_BYTES
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn path(s: &str) -> bool {
    s.len() <= MAX_NAME_BYTES && s.strip_prefix('/').is_some_and(component)
}
fn bounded_text<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    struct Text;
    impl serde::de::Visitor<'_> for Text {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {MAX_NAME_BYTES} UTF8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> std::result::Result<String, E> {
            if v.len() > MAX_NAME_BYTES || v.contains('\0') {
                return Err(E::custom("capacity text bound before copy"));
            }
            Ok(v.to_owned())
        }
    }
    d.deserialize_str(Text)
}
fn bounded_map<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, u64>, D::Error> {
    use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
    struct Key;
    impl<'de> DeserializeSeed<'de> for Key {
        type Value = String;
        fn deserialize<T: serde::Deserializer<'de>>(
            self,
            d: T,
        ) -> std::result::Result<String, T::Error> {
            bounded_text(d)
        }
    }
    struct Entries;
    impl<'de> Visitor<'de> for Entries {
        type Value = BTreeMap<String, u64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {MAX_FILES} unique capacity file entries")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some(key) = a.next_key_seed(Key)? {
                if values.len() == MAX_FILES || values.contains_key(&key) {
                    return Err(A::Error::custom(
                        "capacity map bound/duplicate before value",
                    ));
                }
                values.insert(key, a.next_value()?);
            }
            Ok(values)
        }
    }
    d.deserialize_map(Entries)
}

/// Immutable supplied environment; fixed allocation is an explicit teaching fact,
/// not an ext4 metadata calculation. Reserved blocks remain free but unavailable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    #[serde(deserialize_with = "bounded_text")]
    pub id: String,
    pub block_bytes: u64,
    pub total_blocks: u64,
    pub fixed_used_blocks: u64,
    pub root_reserved_blocks: u64,
    pub total_inodes: u64,
    pub fixed_used_inodes: u64,
    #[serde(deserialize_with = "bounded_map")]
    pub retained: BTreeMap<String, u64>,
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        if !component(&self.id)
            || !matches!(self.block_bytes, 1024 | 2048 | 4096)
            || self.total_blocks == 0
            || self.total_blocks > MAX_POOL
            || self.fixed_used_blocks > self.total_blocks
            || self.root_reserved_blocks > self.total_blocks
            || self.total_inodes == 0
            || self.total_inodes > MAX_POOL
            || self.fixed_used_inodes > self.total_inodes
            || self.retained.len() > MAX_FILES
            || self
                .retained
                .iter()
                .any(|(n, b)| !path(n) || *b > MAX_LOGICAL_BYTES)
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressure {
    Neither,
    OrdinaryBlocks,
    Inodes,
    Both,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub block_bytes: u64,
    pub total_blocks: u64,
    pub used_blocks: u64,
    pub free_blocks: u64,
    pub available_blocks: u64,
    pub total_inodes: u64,
    pub used_inodes: u64,
    pub free_inodes: u64,
    pub file_logical_bytes: u64,
    pub file_data_blocks: u64,
    pub used_percent: Option<u8>,
    pub inode_percent: Option<u8>,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
}
impl Usage {
    pub fn pressure(self) -> Pressure {
        match (self.available_blocks == 0, self.free_inodes == 0) {
            (false, false) => Pressure::Neither,
            (true, false) => Pressure::OrdinaryBlocks,
            (false, true) => Pressure::Inodes,
            (true, true) => Pressure::Both,
        }
    }
}
fn percent(used: u64, denominator: u64) -> Option<u8> {
    (denominator != 0).then(|| ((used * 100).div_ceil(denominator)) as u8)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    version: u32,
    profile: Profile,
    generation: u64,
    files: BTreeMap<String, u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u32,
    profile: Profile,
    generation: u64,
    #[serde(deserialize_with = "bounded_map")]
    files: BTreeMap<String, u64>,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let w = Wire::deserialize(d)?;
        let state = Self {
            version: w.version,
            profile: w.profile,
            generation: w.generation,
            files: w.files,
        };
        state.validate().map_err(serde::de::Error::custom)?;
        Ok(state)
    }
}

/// This private bounded snapshot is a virtual preview, not a Linux transaction.
#[derive(Debug)]
pub struct RemovalPlan {
    before: State,
    names: Vec<String>,
    projected: Usage,
}
impl RemovalPlan {
    pub fn projected(&self) -> Usage {
        self.projected
    }
}
impl State {
    pub fn new(profile: Profile, files: BTreeMap<String, u64>) -> Result<Self> {
        let state = Self {
            version: VERSION,
            profile,
            generation: 0,
            files,
        };
        state.validate()?;
        Ok(state)
    }
    pub fn validate(&self) -> Result<()> {
        self.usage().map(|_| ())
    }
    pub fn usage(&self) -> Result<Usage> {
        self.profile.validate()?;
        if self.version != VERSION || self.files.len() > MAX_FILES {
            return Err(Error::Invalid);
        }
        let mut logical = 0_u64;
        let mut data = 0_u64;
        for (name, &bytes) in &self.files {
            if !path(name) || bytes > MAX_LOGICAL_BYTES {
                return Err(Error::Invalid);
            }
            logical = logical.checked_add(bytes).ok_or(Error::Limit)?;
            data = data
                .checked_add(bytes.div_ceil(self.profile.block_bytes))
                .ok_or(Error::Limit)?;
        }
        if logical > MAX_LOGICAL_BYTES {
            return Err(Error::Limit);
        }
        for (name, bytes) in &self.profile.retained {
            if self.files.get(name) != Some(bytes) {
                return Err(Error::Protected);
            }
        }
        let used_blocks = self
            .profile
            .fixed_used_blocks
            .checked_add(data)
            .ok_or(Error::Limit)?;
        let used_inodes = self
            .profile
            .fixed_used_inodes
            .checked_add(self.files.len() as u64)
            .ok_or(Error::Limit)?;
        let free_blocks = self
            .profile
            .total_blocks
            .checked_sub(used_blocks)
            .ok_or(Error::Invalid)?;
        let free_inodes = self
            .profile
            .total_inodes
            .checked_sub(used_inodes)
            .ok_or(Error::Invalid)?;
        let available_blocks = free_blocks.saturating_sub(self.profile.root_reserved_blocks);
        Ok(Usage {
            block_bytes: self.profile.block_bytes,
            total_blocks: self.profile.total_blocks,
            used_blocks,
            free_blocks,
            available_blocks,
            total_inodes: self.profile.total_inodes,
            used_inodes,
            free_inodes,
            file_logical_bytes: logical,
            file_data_blocks: data,
            used_percent: percent(used_blocks, used_blocks + available_blocks),
            inode_percent: percent(used_inodes, self.profile.total_inodes),
            total_bytes: self
                .profile
                .total_blocks
                .checked_mul(self.profile.block_bytes)
                .ok_or(Error::Limit)?,
            free_bytes: free_blocks
                .checked_mul(self.profile.block_bytes)
                .ok_or(Error::Limit)?,
            available_bytes: available_blocks
                .checked_mul(self.profile.block_bytes)
                .ok_or(Error::Limit)?,
        })
    }
    pub fn files(&self) -> Result<&BTreeMap<String, u64>> {
        self.validate()?;
        Ok(&self.files)
    }
    pub fn profile(&self) -> Result<&Profile> {
        self.validate()?;
        Ok(&self.profile)
    }
    pub fn generation(&self) -> Result<u64> {
        self.validate()?;
        Ok(self.generation)
    }
    pub fn same_environment(&self, source: &Self) -> bool {
        self.validate().is_ok() && source.validate().is_ok() && self.profile == source.profile
    }
    pub fn put_file(&mut self, name: &str, bytes: u64) -> Result<()> {
        let now = self.usage()?;
        if !path(name) {
            return Err(Error::Invalid);
        }
        if bytes > MAX_LOGICAL_BYTES {
            return Err(Error::Limit);
        }
        if self.profile.retained.contains_key(name) {
            return Err(Error::Protected);
        }
        if self.files.get(name) == Some(&bytes) {
            return Ok(());
        }
        let previous = self.files.get(name).copied();
        if previous.is_none() {
            if self.files.len() == MAX_FILES {
                return Err(Error::Limit);
            }
            if now.free_inodes == 0 {
                return Err(Error::NoInodes);
            }
        }
        let previous_blocks = previous.unwrap_or(0).div_ceil(self.profile.block_bytes);
        let next_blocks = bytes.div_ceil(self.profile.block_bytes);
        if next_blocks.saturating_sub(previous_blocks) > now.available_blocks {
            return Err(Error::NoBlocks);
        }
        let next_logical = now.file_logical_bytes - previous.unwrap_or(0);
        if next_logical
            .checked_add(bytes)
            .is_none_or(|n| n > MAX_LOGICAL_BYTES)
        {
            return Err(Error::Limit);
        }
        let generation = self.generation.checked_add(1).ok_or(Error::Limit)?;
        let mut candidate = self.clone();
        candidate.files.insert(name.to_owned(), bytes);
        candidate.generation = generation;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    pub fn preview_remove(&self, names: &[&str]) -> Result<RemovalPlan> {
        self.validate()?;
        if names.is_empty() || names.len() > MAX_FILES {
            return Err(Error::Limit);
        }
        let mut seen = BTreeSet::new();
        for &name in names {
            if !path(name) || !seen.insert(name) {
                return Err(Error::Invalid);
            }
            if self.profile.retained.contains_key(name) {
                return Err(Error::Protected);
            }
            if !self.files.contains_key(name) {
                return Err(Error::Unknown);
            }
        }
        self.generation.checked_add(1).ok_or(Error::Limit)?;
        let mut candidate = self.clone();
        for &name in names {
            candidate.files.remove(name);
        }
        let projected = candidate.usage()?;
        Ok(RemovalPlan {
            before: self.clone(),
            names: names.iter().map(|s| (*s).to_owned()).collect(),
            projected,
        })
    }
    pub fn apply_remove(&mut self, plan: &RemovalPlan) -> Result<()> {
        self.validate()?;
        if self != &plan.before {
            return Err(Error::Stale);
        }
        let generation = self.generation.checked_add(1).ok_or(Error::Limit)?;
        let mut candidate = self.clone();
        for name in &plan.names {
            candidate.files.remove(name);
        }
        candidate.generation = generation;
        if candidate.usage()? != plan.projected {
            return Err(Error::Invalid);
        }
        *self = candidate;
        Ok(())
    }
    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        let raw = serde_json::to_string(self).map_err(|_| Error::Invalid)?;
        if raw.len() > MAX_STATE_BYTES {
            return Err(Error::Limit);
        }
        Ok(raw)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(Error::Limit);
        }
        serde_json::from_str(raw).map_err(|_| Error::Invalid)
    }
}
