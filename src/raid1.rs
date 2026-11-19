//! Bounded capability; authored integration and runtime acceptance are separate.
//! Original bounded RAID1 teaching model; no OS/device/process/network access.
//! Text segments are sample replication units, not RAID1 stripe/chunk geometry.
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: u32 = 1;
pub const MAX_STATE_BYTES: usize = 512 * 1024;
pub const MAX_DEVICES: usize = 8;
pub const MAX_ARRAYS: usize = 2;
pub const MAX_SEGMENTS: usize = 128;
pub const MAX_SEGMENT_BYTES: usize = 256;
pub const MAX_SECTORS: u64 = 1 << 35;
pub const HELP: &str = "Simulated two-member RAID1 only: mdadm --detail /dev/mdN; mdadm /dev/mdN --fail /dev/DEVICE; mdadm /dev/mdN --remove /dev/DEVICE; mdadm /dev/mdN --add /dev/DEVICE. --manage before the array and -D for detail are also supported. Add accepts supplied fresh blank media only. Use the separate Advance simulated recovery button to copy bounded sample data; this is not a Linux command or elapsed real time. Ordinary --detail succeeds even when degraded; read its state. No --test, create, force, replace, re-add, reshape, metadata erasure, host devices, arbitrary paths, shell syntax, concurrent writes or actual kernel execution. Protected-media refusal is a simulator safeguard, not a real mdadm guarantee.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Operation,
    Limit,
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
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn invalid(message: &str) -> Error {
    error(ErrorKind::Invalid, message)
}
fn unsupported(message: &str) -> Error {
    error(ErrorKind::Unsupported, message)
}
fn operation(message: &str) -> Error {
    error(ErrorKind::Operation, message)
}

// Bound map keys/counts before allocating the excess value, reject duplicate keys.
fn map<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, T>, D::Error> {
    use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
    struct Key;
    impl<'de> DeserializeSeed<'de> for Key {
        type Value = String;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
            crate::cron::text::<D, 64>(d)
        }
    }
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} distinct RAID inventory entries")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(key) = a.next_key_seed(Key)? {
                if out.len() >= N || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "RAID map limit or duplicate inventory key",
                    ));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Items::<T, N>(std::marker::PhantomData))
}
fn device_path(s: &str) -> bool {
    s.strip_prefix("/dev/").is_some_and(|name| {
        !name.is_empty()
            && name.len() <= 48
            && name != "."
            && name != ".."
            && !name.starts_with(['-', '.'])
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-+.".contains(&b))
    })
}
fn array_path(s: &str) -> bool {
    device_path(s) && s.strip_prefix("/dev/md").is_some_and(|n| !n.is_empty())
}
fn file_path(s: &str) -> bool {
    s.strip_prefix('/').is_some_and(|name| {
        !name.is_empty()
            && name.len() <= 63
            && name != "."
            && name != ".."
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub file: String,
    pub index: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Media {
    Blank,
    Foreign,
    Md {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        array: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub raw_sectors: u64,
    /// Supplied usable MD data sectors, after opaque metadata/offset constraints.
    pub usable_sectors: u64,
    pub available: bool,
    pub read_only: bool,
    pub protected: bool,
    pub media: Media,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Segment,128>")]
    pub content: Vec<Segment>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Role {
    Synced,
    Faulty,
    Spare,
    Recovering { copied: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub device: String,
    /// Faulty retains its former slot; that is not an active/in-sync claim.
    pub slot: Option<u8>,
    pub role: Role,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Array {
    pub component_sectors: u64,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Member,3>")]
    pub members: Vec<Member>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    #[serde(deserialize_with = "map::<_,Device,8>")]
    pub devices: BTreeMap<String, Device>,
    #[serde(deserialize_with = "map::<_,Array,2>")]
    pub arrays: BTreeMap<String, Array>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Healthy,
    Degraded,
    Rebuilding { copied: u16, total: u16 },
}
impl Health {
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Degraded => "Degraded",
            Self::Rebuilding { .. } => "Rebuilding (still degraded)",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Fail {
        array: String,
        device: String,
    },
    Remove {
        array: String,
        device: String,
    },
    Add {
        array: String,
        device: String,
    },
    /// Native simulator control only; never parsed as an mdadm command.
    Advance {
        array: String,
        units: u16,
    },
}
// Preserve the convenient closed tagged action wire shape without an internally
// tagged generic Content buffer allocating unknown/excess fields before bounds.
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
        struct Text<const N: usize>;
        impl<'de, const N: usize> DeserializeSeed<'de> for Text<N> {
            type Value = String;
            fn deserialize<D: Deserializer<'de>>(
                self,
                d: D,
            ) -> std::result::Result<String, D::Error> {
                crate::cron::text::<D, N>(d)
            }
        }
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = Action;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("one closed bounded RAID action")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Action, A::Error> {
                let (mut kind, mut array, mut device, mut units) = (None, None, None, None);
                let mut seen = BTreeSet::new();
                while let Some(key) = a.next_key_seed(Text::<16>)? {
                    if seen.len() >= 4 || !seen.insert(key.clone()) {
                        return Err(A::Error::custom("Duplicate or excess RAID action field"));
                    }
                    match key.as_str() {
                        "kind" => kind = Some(a.next_value_seed(Text::<16>)?),
                        "array" => array = Some(a.next_value_seed(Text::<64>)?),
                        "device" => device = Some(a.next_value_seed(Text::<64>)?),
                        "units" => units = Some(a.next_value::<u16>()?),
                        _ => return Err(A::Error::custom("Unknown RAID action field")),
                    }
                }
                let kind = kind.ok_or_else(|| A::Error::custom("Missing RAID action kind"))?;
                let array = array.ok_or_else(|| A::Error::custom("Missing RAID action array"))?;
                match kind.as_str() {
                    "fail" | "remove" | "add" if units.is_none() => {
                        let device =
                            device.ok_or_else(|| A::Error::custom("Missing RAID action device"))?;
                        Ok(match kind.as_str() {
                            "fail" => Action::Fail { array, device },
                            "remove" => Action::Remove { array, device },
                            _ => Action::Add { array, device },
                        })
                    }
                    "advance" if device.is_none() => Ok(Action::Advance {
                        array,
                        units: units
                            .ok_or_else(|| A::Error::custom("Missing RAID advance units"))?,
                    }),
                    _ => Err(A::Error::custom(
                        "Unsupported RAID action kind or extra operand",
                    )),
                }
            }
        }
        d.deserialize_map(Fields)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub status: u8,
    pub output: String,
}

fn content_valid(content: &[Segment]) -> bool {
    if content.len() > MAX_SEGMENTS {
        return false;
    }
    let mut files = 0usize;
    let mut previous: Option<&Segment> = None;
    for part in content {
        if !file_path(&part.file)
            || part.index >= 16
            || part.text.len() > MAX_SEGMENT_BYTES
            || !part.text.is_ascii()
            || part
                .text
                .bytes()
                .any(|b| b == 0 || b == 127 || (b < 32 && !b"\n\r\t".contains(&b)))
        {
            return false;
        }
        match previous {
            Some(p) if p.file == part.file => {
                if part.index != p.index + 1 {
                    return false;
                }
            }
            Some(p) if p.file < part.file => {
                files += 1;
                if part.index != 0 {
                    return false;
                }
            }
            None => {
                files += 1;
                if part.index != 0 {
                    return false;
                }
            }
            _ => return false,
        }
        previous = Some(part);
    }
    files <= 16
}

impl State {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.devices.is_empty()
            || self.devices.len() > MAX_DEVICES
            || self.arrays.is_empty()
            || self.arrays.len() > MAX_ARRAYS
            || !crate::cron_workspace::within(self, MAX_STATE_BYTES)
        {
            return Err(invalid(
                "RAID state version, inventory count or encoded-size limit",
            ));
        }
        for (path, d) in &self.devices {
            if !device_path(path)
                || self.arrays.contains_key(path)
                || d.raw_sectors == 0
                || d.raw_sectors > MAX_SECTORS
                || d.usable_sectors == 0
                || d.usable_sectors > d.raw_sectors
                || !content_valid(&d.content)
                || d.content.iter().map(|p| p.text.len() as u64).sum::<u64>()
                    > d.usable_sectors * 512
            {
                return Err(invalid(
                    "Invalid supplied device identity, geometry or sample content",
                ));
            }
            match &d.media {
                Media::Blank if !d.content.is_empty() => {
                    return Err(invalid("Blank media cannot contain retained sample data"));
                }
                Media::Md { array } if !self.arrays.contains_key(array) => {
                    return Err(invalid("Unknown retained MD metadata identity"));
                }
                _ => {}
            }
        }
        let mut attached = BTreeSet::new();
        for (path, a) in &self.arrays {
            if !array_path(path)
                || a.component_sectors == 0
                || a.component_sectors > MAX_SECTORS
                || a.members.is_empty()
                || a.members.len() > 3
            {
                return Err(invalid("Invalid bounded two-slot array"));
            }
            let mut slots = BTreeSet::new();
            let mut source: Option<&[Segment]> = None;
            let mut synced = 0usize;
            let mut spares = 0usize;
            let mut recovering = 0usize;
            let mut faulty = 0usize;
            for member in &a.members {
                let d = self
                    .devices
                    .get(&member.device)
                    .ok_or_else(|| invalid("Array member is outside supplied device inventory"))?;
                if !attached.insert(&member.device)
                    || d.media
                        != (Media::Md {
                            array: path.clone(),
                        })
                    || d.usable_sectors < a.component_sectors
                    || member.slot.is_some_and(|s| s >= 2)
                    || d.content.iter().map(|p| p.text.len() as u64).sum::<u64>()
                        > a.component_sectors * 512
                {
                    return Err(invalid(
                        "Duplicate member, wrong array metadata, capacity or slot",
                    ));
                }
                match member.role {
                    Role::Synced => {
                        if member.slot.is_none()
                            || !slots.insert(member.slot)
                            || !d.available
                            || d.read_only
                            || d.content.is_empty()
                        {
                            return Err(invalid(
                                "Synchronized members need a unique active slot and readable complete data",
                            ));
                        }
                        if source.is_some_and(|s| s != d.content.as_slice()) {
                            return Err(invalid("Synchronized replicas disagree"));
                        }
                        source = Some(&d.content);
                        synced += 1;
                    }
                    Role::Faulty => {
                        if member.slot.is_none() {
                            return Err(invalid("A failed member retains its former slot"));
                        }
                        faulty += 1;
                    }
                    Role::Spare => {
                        if member.slot.is_some()
                            || !d.available
                            || d.read_only
                            || d.protected
                            || !d.content.is_empty()
                        {
                            return Err(invalid(
                                "An idle spare must be available fresh data capacity without an active slot",
                            ));
                        }
                        spares += 1;
                    }
                    Role::Recovering { .. } => {
                        if member.slot.is_none()
                            || !slots.insert(member.slot)
                            || !d.available
                            || d.read_only
                            || d.protected
                        {
                            return Err(invalid(
                                "A recovery target needs a unique slot and writable available media",
                            ));
                        }
                        recovering += 1;
                    }
                }
            }
            if synced == 0
                || synced > 2
                || spares > 1
                || recovering > 1
                || faulty > 1
                || synced + recovering > 2
                || (synced == 1 && spares != 0)
                || (recovering != 0 && spares != 0)
            {
                return Err(invalid(
                    "Unsupported failed-array or ambiguous spare/recovery state",
                ));
            }
            let source = source.ok_or_else(|| invalid("No synchronized source"))?;
            for member in &a.members {
                if let Role::Recovering { copied } = member.role {
                    let copied = usize::from(copied);
                    let target = &self.devices[&member.device].content;
                    if copied >= source.len() || target.as_slice() != &source[..copied] {
                        return Err(invalid(
                            "Recovery cursor must describe actual copied sample segments, not claimed progress",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "RAID JSON exceeds bounded capacity",
            ));
        }
        let state: Self = serde_json::from_str(raw)
            .map_err(|e| invalid(&format!("Invalid bounded RAID state: {e}")))?;
        state.validate()?;
        Ok(state)
    }
    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| invalid(&e.to_string()))
    }

    /// Compare immutable source facts. The held authored Definition/Goal is still
    /// authority: this helper alone does not prove a learner outcome or history.
    pub fn environment_matches(&self, source: &Self) -> bool {
        if self.validate().is_err()
            || source.validate().is_err()
            || self.devices.keys().ne(source.devices.keys())
            || self.arrays.keys().ne(source.arrays.keys())
        {
            return false;
        }
        for (path, initial) in &source.devices {
            let current = &self.devices[path];
            if (
                current.raw_sectors,
                current.usable_sectors,
                current.available,
                current.read_only,
                current.protected,
            ) != (
                initial.raw_sectors,
                initial.usable_sectors,
                initial.available,
                initial.read_only,
                initial.protected,
            ) {
                return false;
            }
            if matches!(initial.media, Media::Foreign) && current != initial {
                return false;
            }
            if matches!(initial.media, Media::Blank) && matches!(current.media, Media::Foreign) {
                return false;
            }
            if let Media::Md { array } = &initial.media
                && current.media
                    != (Media::Md {
                        array: array.clone(),
                    })
            {
                return false;
            }
            if initial.protected {
                if current != initial {
                    return false;
                }
                let locate = |s: &Self| {
                    s.arrays.iter().find_map(|(a, v)| {
                        v.members
                            .iter()
                            .find(|m| m.device == *path)
                            .map(|m| (a.clone(), m.clone()))
                    })
                };
                if locate(self) != locate(source) {
                    return false;
                }
            }
        }
        source
            .arrays
            .iter()
            .all(|(path, a)| self.arrays[path].component_sectors == a.component_sectors)
    }

    fn array(&self, path: &str) -> Result<&Array> {
        self.arrays
            .get(path)
            .ok_or_else(|| operation("Choose an array from the supplied virtual inventory"))
    }
    pub fn health(&self, path: &str) -> Result<Health> {
        self.validate()?;
        let a = self.array(path)?;
        Ok(
            if let Some(copied) = a.members.iter().find_map(|m| match m.role {
                Role::Recovering { copied } => Some(copied),
                _ => None,
            }) {
                Health::Rebuilding {
                    copied,
                    total: self.replica_content(path)?.len() as u16,
                }
            } else if a.members.iter().filter(|m| m.role == Role::Synced).count() == 2 {
                Health::Healthy
            } else {
                Health::Degraded
            },
        )
    }
    pub fn replica_content(&self, path: &str) -> Result<&[Segment]> {
        self.validate()?;
        let member = self
            .array(path)?
            .members
            .iter()
            .find(|m| m.role == Role::Synced)
            .ok_or_else(|| operation("No readable synchronized member"))?;
        Ok(&self.devices[&member.device].content)
    }
    pub fn read_file(&self, path: &str, file: &str) -> Result<String> {
        if !file_path(file) {
            return Err(unsupported(
                "Sample files use one supplied absolute leaf name, without path traversal",
            ));
        }
        let parts = self.replica_content(path)?;
        if !parts.iter().any(|p| p.file == file) {
            return Err(operation(
                "That file is absent from the retained sample data",
            ));
        }
        Ok(parts
            .iter()
            .filter(|p| p.file == file)
            .map(|p| p.text.as_str())
            .collect())
    }

    pub fn detail(&self, path: &str) -> Result<Output> {
        self.validate()?;
        let a = self.array(path)?;
        let synced = a.members.iter().filter(|m| m.role == Role::Synced).count();
        let failed = a.members.iter().filter(|m| m.role == Role::Faulty).count();
        let spare = a
            .members
            .iter()
            .filter(|m| matches!(m.role, Role::Spare | Role::Recovering { .. }))
            .count();
        let mut output = format!(
            "{path}:\n    Raid Level : raid1\n    Array Size : {} KiB\n  Raid Devices : 2\n Total Devices : {}\n         State : {} (simulated)\nActive Devices : {synced}\nWorking Devices : {}\nFailed Devices : {failed}\n Spare Devices : {spare}\n\n RaidDevice  State                 Device\n",
            a.component_sectors / 2,
            a.members.len(),
            self.health(path)?.label(),
            a.members.len() - failed
        );
        for m in &a.members {
            let role = match m.role {
                Role::Synced => "active sync",
                Role::Faulty => "faulty",
                Role::Spare => "spare",
                Role::Recovering { .. } => "spare rebuilding",
            };
            let slot = if matches!(m.role, Role::Synced | Role::Recovering { .. }) {
                m.slot.map(|n| n.to_string()).unwrap_or_else(|| "-".into())
            } else {
                "-".into()
            };
            output.push_str(&format!(" {slot:>10}  {role:<21} {}\n", m.device));
            if let Role::Recovering { copied } = m.role {
                output.push_str(&format!(
                    "             Sample segments copied: {copied}/{}\n",
                    self.replica_content(path)?.len()
                ));
            }
        }
        Ok(Output { status: 0, output })
    }

    /// One bounded clone only after validating admission. Errors never publish it.
    pub fn apply(&mut self, action: &Action) -> Result<Output> {
        self.validate()?;
        let mut next = self.clone();
        let message = next.mutate(action)?;
        next.validate()?;
        *self = next;
        Ok(Output {
            status: 0,
            output: message,
        })
    }
    fn maybe_start_recovery(&mut self, path: &str) -> Result<()> {
        let a = self.array(path)?;
        let filled: BTreeSet<u8> = a
            .members
            .iter()
            .filter(|m| matches!(m.role, Role::Synced | Role::Recovering { .. }))
            .filter_map(|m| m.slot)
            .collect();
        if let Some(slot) = (0..2u8).find(|n| !filled.contains(n))
            && let Some(index) = a.members.iter().position(|m| m.role == Role::Spare)
        {
            let a = self
                .arrays
                .get_mut(path)
                .ok_or_else(|| invalid("Array disappeared"))?;
            a.members[index].slot = Some(slot);
            a.members[index].role = Role::Recovering { copied: 0 };
        }
        Ok(())
    }
    fn mutate(&mut self, action: &Action) -> Result<String> {
        let (path, device) = match action {
            Action::Fail { array, device }
            | Action::Remove { array, device }
            | Action::Add { array, device } => (array.as_str(), Some(device.as_str())),
            Action::Advance { array, .. } => (array.as_str(), None),
        };
        if !array_path(path) {
            return Err(unsupported("Use a supplied literal virtual array path"));
        }
        self.array(path)?;
        if let Some(device) = device {
            if !device_path(device) {
                return Err(unsupported(
                    "Use one supplied literal device path; no selectors, paths or shell expansion",
                ));
            }
            let d = self
                .devices
                .get(device)
                .ok_or_else(|| operation("That device is not in the supplied virtual inventory"))?;
            if d.protected {
                return Err(operation(
                    "This device holds protected data; choose the intended member or fresh replacement. Real mdadm does not provide this simulator safeguard.",
                ));
            }
        }
        match action {
            Action::Fail { device, .. } => {
                let a = self.array(path)?;
                let index = a
                    .members
                    .iter()
                    .position(|m| m.device == *device)
                    .ok_or_else(|| operation("This device is not attached to that array"))?;
                if a.members[index].role != Role::Synced {
                    return Err(unsupported(
                        "This slice supports deliberately failing a synchronized member only; failed/spare/recovering failure cases are not modeled",
                    ));
                }
                if a.members.iter().filter(|m| m.role == Role::Synced).count() < 2 {
                    return Err(unsupported(
                        "Failing the last complete source, including during recovery, is outside this exercise. This is not real Linux data-loss protection.",
                    ));
                }
                if a.members.iter().any(|m| m.role == Role::Faulty) {
                    return Err(unsupported(
                        "This subset retains only one failed member at a time. Remove the old failed member before exploring another failure; real mdadm has broader behavior.",
                    ));
                }
                self.arrays
                    .get_mut(path)
                    .ok_or_else(|| invalid("Array disappeared"))?
                    .members[index]
                    .role = Role::Faulty;
                self.maybe_start_recovery(path)?;
                Ok(format!("mdadm: set {device} faulty in {path}\n"))
            }
            Action::Remove { device, .. } => {
                let a = self.array(path)?;
                let index=a.members.iter().position(|m|m.device==*device).ok_or_else(||operation("This device is not attached; a missing slot has no attached member to remove"))?;
                match a.members[index].role {
                    Role::Synced => {
                        return Err(operation(
                            "This member is active. Inspect the array; an active member cannot simply be removed.",
                        ));
                    }
                    Role::Recovering { .. } => {
                        return Err(unsupported(
                            "Removing an in-progress recovery target is outside this subset",
                        ));
                    }
                    Role::Faulty | Role::Spare => {}
                }
                self.arrays
                    .get_mut(path)
                    .ok_or_else(|| invalid("Array disappeared"))?
                    .members
                    .remove(index);
                self.maybe_start_recovery(path)?;
                Ok(format!("mdadm: hot removed {device} from {path}\n"))
            }
            Action::Add { device, .. } => {
                let d = &self.devices[device];
                if !d.available {
                    return Err(operation("That replacement is not currently available"));
                }
                if d.read_only {
                    return Err(operation(
                        "That replacement is read-only; recovery needs writable media",
                    ));
                }
                if d.media != Media::Blank {
                    return Err(unsupported(
                        "Only supplied fresh blank media is supported. Existing MD metadata/re-add and foreign-data overwrite are outside this slice.",
                    ));
                }
                if d.usable_sectors < self.array(path)?.component_sectors {
                    return Err(operation(
                        "Replacement usable MD capacity is too small for the array's component size; raw disk size alone is insufficient.",
                    ));
                }
                let a = self.array(path)?;
                if a.members.len() >= 3
                    || a.members
                        .iter()
                        .any(|m| matches!(m.role, Role::Spare | Role::Recovering { .. }))
                {
                    return Err(unsupported(
                        "This subset allows one idle spare or one recovery target per array; inspect existing membership first",
                    ));
                }
                self.devices
                    .get_mut(device)
                    .ok_or_else(|| invalid("Device disappeared"))?
                    .media = Media::Md { array: path.into() };
                self.arrays
                    .get_mut(path)
                    .ok_or_else(|| invalid("Array disappeared"))?
                    .members
                    .push(Member {
                        device: device.clone(),
                        slot: None,
                        role: Role::Spare,
                    });
                self.maybe_start_recovery(path)?;
                Ok(format!("mdadm: added {device}\n"))
            }
            Action::Advance { units, .. } => {
                if *units == 0 || usize::from(*units) > MAX_SEGMENTS {
                    return Err(error(
                        ErrorKind::Limit,
                        "Advance between1and128 sample segments",
                    ));
                }
                let a = self.array(path)?;
                let index = a
                    .members
                    .iter()
                    .position(|m| matches!(m.role, Role::Recovering { .. }))
                    .ok_or_else(|| {
                        operation(
                            "No recovery is currently running; inspect the array before advancing",
                        )
                    })?;
                let copied = match a.members[index].role {
                    Role::Recovering { copied } => usize::from(copied),
                    _ => return Err(invalid("Recovery disappeared")),
                };
                let device = a.members[index].device.clone();
                let total = self.replica_content(path)?.len();
                let end = (copied + usize::from(*units)).min(total);
                let parts = self.replica_content(path)?[copied..end].to_vec();
                self.devices
                    .get_mut(&device)
                    .ok_or_else(|| invalid("Target disappeared"))?
                    .content
                    .extend(parts);
                let finished = end == total;
                self.arrays
                    .get_mut(path)
                    .ok_or_else(|| invalid("Array disappeared"))?
                    .members[index]
                    .role = if finished {
                    Role::Synced
                } else {
                    Role::Recovering { copied: end as u16 }
                };
                Ok(if finished {
                    "Simulated recovery completed; sample replicas now agree. Inspect array health and protected data.\n".into()
                } else {
                    format!(
                        "Simulated recovery advanced to {end} copied sample segments; redundancy is not restored yet.\n"
                    )
                })
            }
        }
    }

    /// Command errors remain honest: unsupported/invalid syntax returns Err;
    /// supported operational refusal returns status1 without changing any state.
    pub fn execute(&mut self, command: &str) -> Result<Output> {
        if command.len() > 512 || !command.is_ascii() || command.contains(['\n', '\r', '\0']) {
            return Err(unsupported(
                "Use one supported command, up to512 ASCII bytes, without shell syntax",
            ));
        }
        let tokens: Vec<&str> = command.split_ascii_whitespace().collect();
        if tokens.contains(&"--test") {
            return Err(unsupported(
                "The --test option is not implemented in this simulator. Use ordinary --detail and read the displayed health; exit status0 alone does not mean the mirror is healthy.",
            ));
        }

        let parsed = match tokens.as_slice() {
            ["mdadm", "--detail" | "-D", array] => return self.detail(array),
            ["mdadm", array, verb, device] => Some((*array, *verb, *device)),
            ["mdadm", "--manage", array, verb, device] => Some((*array, *verb, *device)),
            _ => None,
        };
        let (array, verb, device) = parsed.ok_or_else(|| unsupported("This simulator supports array inspection and one --fail, --remove or --add member operation at a time. Open Optional help for exact forms; other syntax and host commands are not executed."))?;
        let action = match verb {
            "--fail" => Action::Fail {
                array: array.into(),
                device: device.into(),
            },
            "--remove" => Action::Remove {
                array: array.into(),
                device: device.into(),
            },
            "--add" => Action::Add {
                array: array.into(),
                device: device.into(),
            },
            _ => {
                return Err(unsupported(
                    "This simulator supports array inspection and one --fail, --remove or --add member operation at a time. Open Optional help for exact forms; other syntax and host commands are not executed.",
                ));
            }
        };
        match self.apply(&action) {
            Err(e) if e.kind == ErrorKind::Operation => Ok(Output {
                status: 1,
                output: format!("mdadm: {}\n", e.message),
            }),
            other => other,
        }
    }
}
