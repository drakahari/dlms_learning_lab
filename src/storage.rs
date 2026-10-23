//! Bounded virtual volume identity, saved mount policy and live mount state.
//! No host paths are opened, and no kernel, subprocess or device API is used.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_VOLUMES: usize = 8;
pub const MAX_CONFIG_BYTES: usize = 16_384;
pub const MAX_STATE_BYTES: usize = 65_536;
const MAX_LINES: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Filesystem {
    Ext4,
    Xfs,
}
impl Filesystem {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ext4 => "ext4",
            Self::Xfs => "xfs",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    pub uuid: String,
    pub filesystem: Filesystem,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    UnknownSource,
    TypeMismatch,
    Occupied,
    NotMounted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: Option<usize>,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        line: None,
        message: message.into(),
    }
}
fn at(mut err: Error, line: usize) -> Error {
    err.line = Some(line);
    err
}
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn path(s: &str) -> bool {
    s.starts_with('/')
        && s.len() <= 128
        && !s.ends_with('/')
        && s[1..].split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        })
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
fn bounded_config(text: &str) -> Result<()> {
    if text.len() > MAX_CONFIG_BYTES || text.lines().count() > MAX_LINES {
        return Err(error(
            ErrorKind::Limit,
            "Mount configuration exceeds the simulator's text or line limit.",
        ));
    }
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(error(
            ErrorKind::Invalid,
            "Mount configuration contains an unsupported control character.",
        ));
    }
    Ok(())
}

// Reject duplicate keys and bound allocations before accepting a snapshot.
fn map<'de, D: serde::Deserializer<'de>, V: Deserialize<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Entries<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> Visitor<'de> for Entries<V> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {MAX_VOLUMES} unique virtual storage keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key::<String>()? {
                if out.len() >= MAX_VOLUMES || k.len() > 128 || out.contains_key(&k) {
                    return Err(A::Error::custom(
                        "Duplicate/oversized virtual storage mapping",
                    ));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries(std::marker::PhantomData))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "InventoryData")]
pub struct Inventory {
    volumes: BTreeMap<String, Volume>,
    devices: BTreeMap<String, String>,
    mountpoints: BTreeSet<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryData {
    #[serde(deserialize_with = "map")]
    volumes: BTreeMap<String, Volume>,
    #[serde(deserialize_with = "map")]
    devices: BTreeMap<String, String>,
    #[serde(deserialize_with = "points")]
    mountpoints: Vec<String>,
}
fn points<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Points;
    impl<'de> Visitor<'de> for Points {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {MAX_VOLUMES} virtual mountpoints")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(p) = a.next_element::<String>()? {
                if out.len() >= MAX_VOLUMES || !path(&p) || out.contains(&p) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate virtual mountpoint or collection limit",
                    ));
                }
                out.push(p);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Points)
}
impl TryFrom<InventoryData> for Inventory {
    type Error = Error;
    fn try_from(raw: InventoryData) -> Result<Self> {
        Self::new(raw.volumes, raw.devices, raw.mountpoints)
    }
}
impl Inventory {
    pub fn new(
        volumes: BTreeMap<String, Volume>,
        devices: BTreeMap<String, String>,
        mountpoints: Vec<String>,
    ) -> Result<Self> {
        if volumes.is_empty()
            || volumes.len() > MAX_VOLUMES
            || devices.len() != volumes.len()
            || mountpoints.is_empty()
            || mountpoints.len() > MAX_VOLUMES
            || volumes
                .iter()
                .any(|(id, v)| !key(id) || !uuid(&v.uuid) || v.uuid != v.uuid.to_ascii_lowercase())
            || devices
                .iter()
                .any(|(p, id)| !path(p) || !p.starts_with("/dev/") || !key(id))
            || mountpoints.iter().any(|p| !path(p))
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid or oversized virtual storage inventory.",
            ));
        }
        // Collection/scalar limits are checked before any derived allocations.
        let point_count = mountpoints.len();
        let points: BTreeSet<_> = mountpoints.into_iter().collect();
        let ids: BTreeSet<_> = devices.values().collect();
        let uuids: BTreeSet<_> = volumes.values().map(|v| &v.uuid).collect();
        if ids != volumes.keys().collect()
            || uuids.len() != volumes.len()
            || points.len() != point_count
            || points.iter().any(|a| {
                points
                    .iter()
                    .any(|b| a != b && b.starts_with(&format!("{a}/")))
            })
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid volume identities, device aliases or fixed mountpoints. This simulator requires unique UUIDs, one alias per volume and distinct non-nested mountpoints.",
            ));
        }
        Ok(Self {
            volumes,
            devices,
            mountpoints: points,
        })
    }
    pub fn volumes(&self) -> &BTreeMap<String, Volume> {
        &self.volumes
    }
    pub fn devices(&self) -> &BTreeMap<String, String> {
        &self.devices
    }
    pub fn mountpoints(&self) -> &BTreeSet<String> {
        &self.mountpoints
    }
    pub fn device_for(&self, volume: &str) -> Option<&str> {
        self.devices
            .iter()
            .find(|(_, id)| id.as_str() == volume)
            .map(|(dev, _)| dev.as_str())
    }
    pub fn resolve(&self, source: &Source) -> Result<&str> {
        match source {
            Source::Device(dev) => self.devices.get(dev).map(String::as_str),
            Source::Uuid(uuid) => self.volumes.iter().find(|(_,v)| &v.uuid == uuid).map(|(id,_)| id.as_str()),
        }.ok_or_else(|| error(ErrorKind::UnknownSource, "No exact source match in this simulator. Copy the device or UUID spelling shown by blkid's inventory; filesystem UUIDs here are lowercase."))
    }
    pub fn blkid(&self) -> String {
        self.devices
            .iter()
            .map(|(dev, id)| {
                let v = &self.volumes[id];
                format!(
                    "{dev}: UUID=\"{}\" TYPE=\"{}\"",
                    v.uuid,
                    v.filesystem.name()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Device(String),
    Uuid(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub line: usize,
    pub source: Source,
    pub target: String,
    pub filesystem: Filesystem,
    pub read_only: bool,
    pub automatic: bool,
    pub dump: u32,
    pub pass: u32,
}
/// Parse only the disclosed six-field subset, without resolving volume identities.
pub fn parse(text: &str) -> Result<Vec<Entry>> {
    bounded_config(text)?;
    let mut entries = Vec::new();
    let mut targets = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let number = index + 1;
        if line.contains(['\\', '#', '\'', '"']) {
            return Err(at(
                error(
                    ErrorKind::Unsupported,
                    "Escaped/quoted fields and inline comments are outside this mount subset. Use plain paths and whole-line comments.",
                ),
                number,
            ));
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.len() != 6 {
            return Err(at(
                error(
                    ErrorKind::Invalid,
                    "Use six fields: source, mountpoint, filesystem type, options, dump and check order.",
                ),
                number,
            ));
        }
        if entries.len() >= MAX_VOLUMES {
            return Err(at(
                error(
                    ErrorKind::Limit,
                    "Too many persistent mount entries for this simulator.",
                ),
                number,
            ));
        }
        let source = if let Some(id) = fields[0].strip_prefix("UUID=") {
            if !uuid(id) {
                return Err(at(
                    error(
                        ErrorKind::Invalid,
                        "UUID must contain 32 hexadecimal digits in the 8-4-4-4-12 pattern.",
                    ),
                    number,
                ));
            }
            Source::Uuid(id.into())
        } else if path(fields[0]) && fields[0].starts_with("/dev/") {
            Source::Device(fields[0].into())
        } else {
            return Err(at(
                error(
                    ErrorKind::Unsupported,
                    "Supported sources are an inventory /dev path or UUID= followed by a filesystem UUID; LABEL, network and other sources are not modeled.",
                ),
                number,
            ));
        };
        if !path(fields[1]) || !targets.insert(fields[1].to_owned()) {
            return Err(at(
                error(
                    ErrorKind::Invalid,
                    "Use a canonical absolute mountpoint once. Duplicate targets and stacked mounts are not modeled.",
                ),
                number,
            ));
        }
        let filesystem = match fields[2] {
            "ext4" => Filesystem::Ext4,
            "xfs" => Filesystem::Xfs,
            _ => {
                return Err(at(
                    error(
                        ErrorKind::Unsupported,
                        "This activity supports ext4 and xfs metadata only; it does not create or repair filesystems.",
                    ),
                    number,
                ));
            }
        };
        let mut read_only = false;
        let mut automatic = true;
        for option in fields[3].split(',') {
            match option {
                "defaults" => {}
                "ro" => read_only = true,
                "rw" => read_only = false,
                "auto" => automatic = true,
                "noauto" => automatic = false,
                _ => {
                    return Err(at(
                        error(
                            ErrorKind::Unsupported,
                            "Supported options: defaults, ro, rw, auto, noauto. Other options are not silently ignored.",
                        ),
                        number,
                    ));
                }
            }
        }
        let integer = |s: &str| {
            (!s.is_empty() && s.len() <= 10 && s.bytes().all(|b| b.is_ascii_digit()))
                .then(|| s.parse::<u32>().ok())
                .flatten()
        };
        let (Some(dump), Some(pass)) = (integer(fields[4]), integer(fields[5])) else {
            return Err(at(
                error(
                    ErrorKind::Invalid,
                    "Use nonnegative whole numbers within the simulator's 32-bit range for dump frequency and check order. These fields are stored; this simulator does not run dump or filesystem checks.",
                ),
                number,
            ));
        };
        entries.push(Entry {
            line: number,
            source,
            target: fields[1].into(),
            filesystem,
            read_only,
            automatic,
            dump,
            pass,
        });
    }
    Ok(entries)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    pub volume: String,
    pub read_only: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "StateData")]
pub struct State {
    inventory: Inventory,
    configuration: String,
    live: BTreeMap<String, Mount>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateData {
    inventory: Inventory,
    configuration: String,
    #[serde(deserialize_with = "map")]
    live: BTreeMap<String, Mount>,
}
impl TryFrom<StateData> for State {
    type Error = Error;
    fn try_from(raw: StateData) -> Result<Self> {
        Self::new(raw.inventory, raw.configuration, raw.live)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountOutcome {
    Mounted,
    AlreadyMounted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionOutcome {
    Mounted {
        target: String,
    },
    /// Skipped because of noauto; not evidence of valid/mountable configuration.
    Manual {
        target: String,
    },
    Failed {
        target: String,
        error: Error,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreshSession {
    pub state: State,
    pub outcomes: Vec<SessionOutcome>,
}
impl State {
    pub fn new(
        inventory: Inventory,
        configuration: String,
        live: BTreeMap<String, Mount>,
    ) -> Result<Self> {
        bounded_config(&configuration)?;
        if live.len() > MAX_VOLUMES {
            return Err(error(
                ErrorKind::Limit,
                "Too many live mounts for this simulator.",
            ));
        }
        let ids: BTreeSet<_> = live.values().map(|m| &m.volume).collect();
        if ids.len() != live.len()
            || live.iter().any(|(p, m)| {
                !inventory.mountpoints.contains(p) || !inventory.volumes.contains_key(&m.volume)
            })
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid live mount table. This simulator supports one volume per fixed mountpoint and one live mount per volume.",
            ));
        }
        Ok(Self {
            inventory,
            configuration,
            live,
        })
    }
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Virtual storage snapshot is too large.",
            ));
        }
        serde_json::from_str(text).map_err(|_| {
            error(
                ErrorKind::Invalid,
                "Invalid virtual storage snapshot; no host storage was accessed.",
            )
        })
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn configuration(&self) -> &str {
        &self.configuration
    }
    pub fn live(&self) -> &BTreeMap<String, Mount> {
        &self.live
    }
    pub fn same_inventory(&self, other: &Self) -> bool {
        self.inventory == other.inventory
    }
    /// Saving changes text only. Invalid work remains inspectable and cannot mount.
    pub fn save_configuration(&mut self, text: String) -> Result<()> {
        bounded_config(&text)?;
        self.configuration = text;
        Ok(())
    }
    fn resolve_entry<'a>(&'a self, entry: &Entry) -> Result<&'a str> {
        if !self.inventory.mountpoints.contains(&entry.target) {
            return Err(at(
                error(
                    ErrorKind::Invalid,
                    "This mountpoint is not present in the activity's fixed directory list.",
                ),
                entry.line,
            ));
        }
        let id = self
            .inventory
            .resolve(&entry.source)
            .map_err(|e| at(e, entry.line))?;
        if self.inventory.volumes[id].filesystem != entry.filesystem {
            return Err(at(
                error(
                    ErrorKind::TypeMismatch,
                    "The configured filesystem type does not match this volume's inventory. Check TYPE in blkid.",
                ),
                entry.line,
            ));
        }
        Ok(id)
    }
    /// Simulator validation, not mount -a, fsck, findmnt or a systemd boot check.
    pub fn validate_configuration(&self) -> Vec<Error> {
        let entries = match parse(&self.configuration) {
            Ok(v) => v,
            Err(e) => return vec![e],
        };
        let mut used = BTreeSet::new();
        let mut errors = Vec::new();
        for entry in entries {
            match self.resolve_entry(&entry) {
                Err(e) => errors.push(e),
                Ok(id) if !used.insert(id.to_owned()) => errors.push(at(error(ErrorKind::Unsupported, "Two entries identify the same volume, possibly through different aliases. Multiple mounts of one volume are outside this simulator subset."),entry.line)),
                _ => {},
            }
        }
        errors
    }
    pub fn mount(&mut self, target: &str) -> Result<MountOutcome> {
        let entries = parse(&self.configuration)?;
        let entry = entries.iter().find(|e| e.target == target).ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "No saved entry names that mountpoint. Save its configuration before mounting.",
            )
        })?;
        let id = self.resolve_entry(entry)?.to_owned();
        let requested = Mount {
            volume: id.clone(),
            read_only: entry.read_only,
        };
        if let Some(current) = self.live.get(target) {
            if current == &requested {
                return Ok(MountOutcome::AlreadyMounted);
            }
            return Err(error(
                ErrorKind::Occupied,
                "The mountpoint already has a different live volume or policy. Saved edits do not change it; this simulator requires unmounting before mounting the saved entry.",
            ));
        }
        if self.live.values().any(|m| m.volume == id) {
            return Err(error(
                ErrorKind::Occupied,
                "This volume is already mounted elsewhere. Multiple mounts of one volume are outside this simulator subset.",
            ));
        }
        self.live.insert(target.into(), requested);
        Ok(MountOutcome::Mounted)
    }
    pub fn umount(&mut self, target: &str) -> Result<()> {
        if self.live.remove(target).is_none() {
            return Err(error(
                ErrorKind::NotMounted,
                "No live volume is mounted at that location.",
            ));
        }
        Ok(())
    }
    /// Independent test copy: no changes to the learner's live session or devices.
    /// Device aliases are explicitly supplied, never probed from the host.
    pub fn fresh_session(&self, devices: BTreeMap<String, String>) -> Result<FreshSession> {
        if devices.keys().ne(self.inventory.devices.keys()) {
            return Err(error(
                ErrorKind::Invalid,
                "The fresh-session test must permute the existing device aliases without adding or removing them.",
            ));
        }
        let inventory = Inventory::new(
            self.inventory.volumes.clone(),
            devices,
            self.inventory.mountpoints.iter().cloned().collect(),
        )?;
        let entries = parse(&self.configuration)?;
        let mut state = Self::new(inventory, self.configuration.clone(), BTreeMap::new())?;
        let mut outcomes = Vec::new();
        for entry in entries {
            if !entry.automatic {
                outcomes.push(SessionOutcome::Manual {
                    target: entry.target,
                });
            } else {
                outcomes.push(match state.mount(&entry.target) {
                    Ok(_) => SessionOutcome::Mounted {
                        target: entry.target,
                    },
                    Err(error) => SessionOutcome::Failed {
                        target: entry.target,
                        error,
                    },
                });
            }
        }
        Ok(FreshSession { state, outcomes })
    }
    pub fn mount_listing(&self) -> String {
        self.live
            .iter()
            .map(|(target, m)| {
                format!(
                    "{} on {target} type {} ({})",
                    self.inventory
                        .device_for(&m.volume)
                        .expect("validated inventory"),
                    self.inventory.volumes[&m.volume].filesystem.name(),
                    if m.read_only { "ro" } else { "rw" }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
