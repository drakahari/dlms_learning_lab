//! Bounded linear LVM growth over virtual devices only. No host I/O or subprocesses.
//! Filesystem capacity is independent of LV capacity. See LVM_SUBSET.md.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MIB: u64 = 1024 * 1024;
pub const DATA_START: u64 = MIB;
pub const BLOCK_BYTES: u64 = 4096;
pub const MAX_DEVICES: usize = 8;
pub const MAX_GROUPS: usize = 4;
pub const MAX_VOLUMES: usize = 16;
pub const MAX_EXTENTS: usize = 4096;
pub const MAX_FILES: usize = 32;
pub const MAX_FILE_BYTES: usize = 65_536;
pub const MAX_TOTAL_FILE_BYTES: usize = 131_072;
pub const MAX_STATE_BYTES: usize = 2 * 1024 * 1024;
const MAX_DEVICE_BYTES: u64 = 1024 * 1024 * MIB;
const MAX_COMMAND_BYTES: usize = 2048;

pub const HELP: &str = "Virtual LVM growth only: pvs; vgs; lvs; pvcreate /dev/NAME...; vgextend VG /dev/NAME...; lvextend -l N|+N|+100%FREE VG/LV; lvextend -L N|+N[M|G] VG/LV; resize2fs VG/LV [BLOCKS|SIZE[M|G]]. /dev/VG/LV is also accepted. Integer LVM size without a suffix means MiB; resize2fs size without a suffix means 4096-byte blocks. M/G are binary and case-insensitive. LV sizes round upward to whole extents; resize2fs changes the separate modeled ext4 capacity. pvs/vgs/lvs use a fixed simulator report with explicit byte columns. No shell syntax, force, shrink, deletion, creation of VGs/LVs/filesystems, snapshots, activation, RAID, thin provisioning, mounts, real devices or host commands. Real LVM has additional and potentially destructive operations; this simulator's refusal is not a real-system safety guarantee.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Unknown,
    Occupied,
    NoSpace,
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
fn error(kind: ErrorKind, text: impl Into<String>) -> Error {
    Error {
        kind,
        message: text.into(),
    }
}
fn invalid(text: &str) -> Error {
    error(ErrorKind::Invalid, text)
}
fn limit(text: &str) -> Error {
    error(ErrorKind::Limit, text)
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && !s.starts_with(['-', '.'])
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-+.".contains(&c))
}
fn device_name(s: &str) -> bool {
    s.strip_prefix("/dev/").is_some_and(name)
}
fn file_name(s: &str) -> bool {
    s.strip_prefix('/').is_some_and(name)
}
fn volume_name(s: &str) -> Result<(&str, &str)> {
    let s = s.strip_prefix("/dev/").unwrap_or(s);
    let (vg, lv) = s
        .split_once('/')
        .ok_or_else(|| invalid("Use VG/LV or /dev/VG/LV"))?;
    if !name(vg) || !name(lv) {
        return Err(invalid(
            "Use ordinary supplied VG/LV names; paths and aliases are not expanded",
        ));
    }
    Ok((vg, lv))
}

// Streaming text/map/sequence admission bounds allocations before excess values.
struct Text<const N: usize>;
impl<'de, const N: usize> serde::de::DeserializeSeed<'de> for Text<N> {
    type Value = String;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        d: D,
    ) -> std::result::Result<String, D::Error> {
        struct V<const N: usize>;
        impl<const N: usize> serde::de::Visitor<'_> for V<N> {
            type Value = String;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "text up to {N} bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
                if s.len() > N || s.contains('\0') {
                    return Err(E::custom("LVM text bound before copy"));
                }
                Ok(s.into())
            }
        }
        d.deserialize_str(V::<N>)
    }
}
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    use serde::de::DeserializeSeed;
    Text::<N>.deserialize(d)
}
fn map<'de, D: serde::Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct Entries<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Entries<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} unique LVM entries")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            use serde::de::Error;
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key_seed(Text::<64>)? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom("LVM map count/duplicate before value"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries::<V, N>(std::marker::PhantomData))
}
fn sequence<'de, D: serde::Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<V>, D::Error> {
    struct Items<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Items<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} LVM items")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            use serde::de::{DeserializeSeed, Error};
            struct Excess;
            impl<'de> DeserializeSeed<'de> for Excess {
                type Value = ();
                fn deserialize<D: serde::Deserializer<'de>>(
                    self,
                    _: D,
                ) -> std::result::Result<(), D::Error> {
                    Err(D::Error::custom("LVM sequence bound before excess body"))
                }
            }
            let mut out = Vec::new();
            loop {
                if out.len() == N {
                    a.next_element_seed(Excess)?;
                    return Ok(out);
                }
                match a.next_element()? {
                    Some(v) => out.push(v),
                    None => return Ok(out),
                }
            }
        }
    }
    d.deserialize_seq(Items::<V, N>(std::marker::PhantomData))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceRole {
    Blank,
    Foreign,
    PhysicalVolume,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub bytes: u64,
    pub role: DeviceRole,
    // Empty string means ungrouped. Kept bounded even on direct serde decoding.
    #[serde(deserialize_with = "text::<_,32>")]
    pub group: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extent {
    #[serde(deserialize_with = "text::<_,64>")]
    pub pv: String,
    pub index: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(deserialize_with = "text::<_,65536>")]
    pub contents: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filesystem {
    pub blocks: u64,
    #[serde(deserialize_with = "map::<_,File,32>")]
    pub files: BTreeMap<String, File>,
}
impl Filesystem {
    pub fn bytes(&self) -> Result<u64> {
        self.blocks
            .checked_mul(BLOCK_BYTES)
            .ok_or_else(|| limit("Filesystem byte overflow"))
    }
    pub fn used_blocks(&self) -> Result<u64> {
        // Deliberate teaching accounting, not ext4's real metadata/inode layout.
        self.files.values().try_fold(1_u64, |n, f| {
            n.checked_add((f.contents.len() as u64).div_ceil(BLOCK_BYTES))
                .ok_or_else(|| limit("Filesystem use overflow"))
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalVolume {
    #[serde(deserialize_with = "sequence::<_,Extent,4096>")]
    pub extents: Vec<Extent>,
    pub filesystem: Option<Filesystem>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeGroup {
    pub extent_bytes: u64,
    #[serde(deserialize_with = "map::<_,LogicalVolume,16>")]
    pub volumes: BTreeMap<String, LogicalVolume>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    #[serde(deserialize_with = "map::<_,Device,8>")]
    pub devices: BTreeMap<String, Device>,
    #[serde(deserialize_with = "map::<_,VolumeGroup,4>")]
    pub groups: BTreeMap<String, VolumeGroup>,
}
fn wire_size<T: Serialize>(value: &T) -> Result<()> {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= MAX_STATE_BYTES)
                .ok_or_else(|| std::io::Error::other("LVM wire bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0), value).map_err(|_| limit("LVM snapshot exceeds 2 MiB"))
}
impl State {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(limit("LVM snapshot exceeds 2 MiB before decoding"));
        }
        let state: Self = serde_json::from_slice(raw)
            .map_err(|e| invalid(&format!("Invalid LVM snapshot: {e}")))?;
        state.validate()?;
        Ok(state)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.devices.len() > MAX_DEVICES || self.groups.len() > MAX_GROUPS {
            return Err(limit("Unsupported LVM version or inventory count"));
        }
        for (p, d) in &self.devices {
            if !device_name(p)
                || d.bytes <= DATA_START
                || d.bytes > MAX_DEVICE_BYTES
                || d.bytes % 512 != 0
                || (!d.group.is_empty() && (!name(&d.group) || !self.groups.contains_key(&d.group)))
                || (d.role != DeviceRole::PhysicalVolume && !d.group.is_empty())
                || (d.role == DeviceRole::PhysicalVolume && d.bytes < DATA_START + MIB)
            {
                return Err(invalid(
                    "Invalid virtual device, signature, sector size or VG ownership",
                ));
            }
        }
        let mut occupied = BTreeSet::new();
        let mut lv_count = 0;
        let mut file_count = 0;
        let mut file_bytes = 0usize;
        for (vg, g) in &self.groups {
            if !name(vg)
                || !(MIB..=64 * MIB).contains(&g.extent_bytes)
                || !g.extent_bytes.is_power_of_two()
                || g.volumes.len() > MAX_VOLUMES
                || !self.devices.values().any(|d| d.group == *vg)
            {
                return Err(invalid(
                    "VG needs supplied physical volumes and a supported power-of-two extent size",
                ));
            }
            for d in self.devices.values().filter(|d| d.group == *vg) {
                self.pv_extents_for(d, g)?;
            }
            lv_count += g.volumes.len();
            if lv_count > MAX_VOLUMES {
                return Err(limit("At most 16 logical volumes are modeled"));
            }
            for (lv, v) in &g.volumes {
                if !name(lv) || v.extents.is_empty() || v.extents.len() > MAX_EXTENTS {
                    return Err(invalid(
                        "Invalid logical volume name or bounded nonempty extent map",
                    ));
                }
                for e in &v.extents {
                    let d = self
                        .devices
                        .get(&e.pv)
                        .ok_or_else(|| invalid("LV refers to an unknown physical volume"))?;
                    if d.role != DeviceRole::PhysicalVolume
                        || d.group != *vg
                        || e.index >= self.pv_extents_for(d, g)?
                        || !occupied.insert((e.pv.as_str(), e.index))
                    {
                        return Err(invalid(
                            "Overlapping, foreign or out-of-range physical extent",
                        ));
                    }
                }
                if let Some(fs) = &v.filesystem {
                    if fs.files.len() > MAX_FILES
                        || fs.blocks == 0
                        || fs.bytes()? > v.extents.len() as u64 * g.extent_bytes
                        || fs.used_blocks()? > fs.blocks
                    {
                        return Err(invalid(
                            "Filesystem capacity/use exceeds its logical volume",
                        ));
                    }
                    file_count += fs.files.len();
                    if file_count > MAX_FILES {
                        return Err(limit("At most 32 virtual files are modeled"));
                    }
                    for (path, f) in &fs.files {
                        if !file_name(path)
                            || f.contents.len() > MAX_FILE_BYTES
                            || f.contents.contains('\0')
                        {
                            return Err(invalid("Invalid flat virtual file or content bound"));
                        }
                        file_bytes = file_bytes
                            .checked_add(f.contents.len())
                            .ok_or_else(|| limit("Virtual data overflow"))?;
                        if file_bytes > MAX_TOTAL_FILE_BYTES {
                            return Err(limit("Stored virtual file data exceeds 128 KiB"));
                        }
                    }
                }
            }
        }
        wire_size(self)
    }
    fn pv_extents_for(&self, d: &Device, g: &VolumeGroup) -> Result<u32> {
        let n = (d.bytes - DATA_START) / g.extent_bytes;
        if n == 0 || n > MAX_EXTENTS as u64 {
            return Err(limit(
                "Each grouped PV needs 1..4096 physical extents in this simulator",
            ));
        }
        Ok(n as u32)
    }
    fn used(&self) -> BTreeSet<(&str, u32)> {
        self.groups
            .values()
            .flat_map(|g| g.volumes.values())
            .flat_map(|v| v.extents.iter())
            .map(|e| (e.pv.as_str(), e.index))
            .collect()
    }
    pub fn volume(&self, id: &str) -> Result<(&VolumeGroup, &LogicalVolume)> {
        self.validate()?;
        let (vg, lv) = volume_name(id)?;
        let g = self
            .groups
            .get(vg)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown supplied VG"))?;
        Ok((
            g,
            g.volumes
                .get(lv)
                .ok_or_else(|| error(ErrorKind::Unknown, "Unknown supplied LV"))?,
        ))
    }
    pub fn lv_bytes(&self, id: &str) -> Result<u64> {
        let (g, v) = self.volume(id)?;
        Ok(v.extents.len() as u64 * g.extent_bytes)
    }
    pub fn filesystem_bytes(&self, id: &str) -> Result<u64> {
        self.volume(id)?
            .1
            .filesystem
            .as_ref()
            .ok_or_else(|| {
                error(
                    ErrorKind::Unsupported,
                    "This LV has no modeled ext4 filesystem",
                )
            })?
            .bytes()
    }
    pub fn read_file(&self, id: &str, path: &str) -> Result<&str> {
        let fs = self
            .volume(id)?
            .1
            .filesystem
            .as_ref()
            .ok_or_else(|| error(ErrorKind::Unsupported, "No modeled filesystem"))?;
        fs.files
            .get(path)
            .map(|f| f.contents.as_str())
            .ok_or_else(|| error(ErrorKind::Unknown, "No such virtual file in this LV"))
    }
    pub fn free_extents(&self, vg: &str) -> Result<u32> {
        self.validate()?;
        let g = self
            .groups
            .get(vg)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown supplied VG"))?;
        let all = self
            .devices
            .values()
            .filter(|d| d.group == vg)
            .try_fold(0u32, |n, d| {
                n.checked_add(self.pv_extents_for(d, g)?)
                    .ok_or_else(|| limit("VG count overflow"))
            })?;
        let used = g
            .volumes
            .values()
            .map(|v| v.extents.len() as u32)
            .sum::<u32>();
        Ok(all - used)
    }
    pub fn execute(&mut self, input: &str) -> Result<String> {
        self.validate()?;
        let action = parse(input)?;
        if let Action::Report(kind) = action {
            return self.report(kind);
        }
        // All checked commands publish one validated candidate. Partial mutation
        // of this teaching transaction is never presented as real LVM rollback.
        let mut next = self.clone();
        let output = next.apply(action)?;
        next.validate()?;
        *self = next;
        Ok(output)
    }
    fn apply(&mut self, action: Action) -> Result<String> {
        match action {
            Action::PvCreate(paths) => {
                for p in paths {
                    let d = self.devices.get_mut(&p).ok_or_else(|| {
                        error(
                            ErrorKind::Unknown,
                            format!("No supplied virtual device {p}"),
                        )
                    })?;
                    if d.role != DeviceRole::Blank {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Grow-only simulator does not reinitialize existing PVs or occupied devices; real force/wipe behavior is not modeled",
                        ));
                    }
                    if d.bytes < DATA_START + MIB {
                        return Err(limit(
                            "This teaching profile requires at least 1 MiB of PV data after the 1 MiB metadata reservation",
                        ));
                    }
                    d.role = DeviceRole::PhysicalVolume;
                }
                Ok("Supplied virtual devices initialized as physical volumes".into())
            }
            Action::VgExtend { vg, paths } => {
                if !self.groups.contains_key(&vg) {
                    return Err(error(ErrorKind::Unknown, "No supplied VG"));
                }
                for p in paths {
                    let d = self.devices.get_mut(&p).ok_or_else(|| {
                        error(
                            ErrorKind::Unknown,
                            format!("No supplied virtual device {p}"),
                        )
                    })?;
                    if !d.group.is_empty() {
                        return Err(error(
                            ErrorKind::Occupied,
                            "PV already belongs to a VG; no migration or reinitialization is modeled",
                        ));
                    }
                    if d.role == DeviceRole::Foreign {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Occupied virtual device; no force/wipe behavior is modeled",
                        ));
                    }
                    // Real vgextend can initialize a previously blank PV.
                    d.role = DeviceRole::PhysicalVolume;
                    d.group = vg.clone();
                }
                Ok(format!(
                    "Volume group {vg} extended using supplied virtual devices"
                ))
            }
            Action::LvExtend { id, growth } => {
                let (vg, lv) = volume_name(&id)?;
                let (g, v) = self.volume(&id)?;
                let current = v.extents.len() as u64;
                let pe = g.extent_bytes;
                let target = match growth {
                    Growth::Extents { amount, relative } => {
                        if relative {
                            current.checked_add(amount)
                        } else {
                            Some(amount)
                        }
                    }
                    Growth::Bytes { amount, relative } => {
                        let n = amount.div_ceil(pe);
                        if relative {
                            current.checked_add(n)
                        } else {
                            Some(n)
                        }
                    }
                    Growth::AllFree => current.checked_add(self.free_extents(vg)? as u64),
                }
                .ok_or_else(|| limit("LV extent arithmetic overflow"))?;
                if target <= current {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "Only extension is modeled; choose a larger LV size. Real shrink/remove operations can destroy data and are not represented",
                    ));
                }
                if target > MAX_EXTENTS as u64 {
                    return Err(limit("LV exceeds 4096 extent limit"));
                }
                let needed = (target - current) as usize;
                if needed > self.free_extents(vg)? as usize {
                    return Err(error(
                        ErrorKind::NoSpace,
                        format!(
                            "VG {vg} has insufficient free physical extents; adding LV size does not add physical storage"
                        ),
                    ));
                }
                let used = self.used();
                let mut allocation = Vec::with_capacity(needed);
                'pvs: for (p, d) in self.devices.iter().filter(|(_, d)| d.group == vg) {
                    for index in 0..self.pv_extents_for(d, &self.groups[vg])? {
                        if !used.contains(&(p.as_str(), index)) {
                            allocation.push(Extent {
                                pv: p.clone(),
                                index,
                            });
                            if allocation.len() == needed {
                                break 'pvs;
                            }
                        }
                    }
                }
                if allocation.len() != needed {
                    return Err(invalid("Extent allocation invariant differs"));
                }
                self.groups
                    .get_mut(vg)
                    .unwrap()
                    .volumes
                    .get_mut(lv)
                    .unwrap()
                    .extents
                    .extend(allocation);
                Ok(format!(
                    "Logical volume {vg}/{lv} now has {target} extents; its filesystem capacity has not changed"
                ))
            }
            Action::ResizeFs { id, blocks } => {
                let (vg, lv) = volume_name(&id)?;
                let (g, v) = self.volume(&id)?;
                let max = v.extents.len() as u64 * g.extent_bytes / BLOCK_BYTES;
                let fs = v.filesystem.as_ref().ok_or_else(|| {
                    error(
                        ErrorKind::Unsupported,
                        "resize2fs requires the supplied modeled ext4 filesystem",
                    )
                })?;
                let target = blocks.unwrap_or(max);
                if target < fs.blocks {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "This simulator models filesystem growth only; real offline resize2fs can shrink filesystems, with additional constraints not modeled here",
                    ));
                }
                if target > max {
                    return Err(error(
                        ErrorKind::NoSpace,
                        "Filesystem cannot grow beyond its LV; extend the LV first",
                    ));
                }
                self.groups
                    .get_mut(vg)
                    .unwrap()
                    .volumes
                    .get_mut(lv)
                    .unwrap()
                    .filesystem
                    .as_mut()
                    .unwrap()
                    .blocks = target;
                Ok(format!(
                    "Modeled ext4 filesystem on {vg}/{lv}: {target} blocks of 4096 bytes; existing virtual files retained"
                ))
            }
            Action::Report(kind) => self.report(kind),
        }
    }
    fn report(&self, kind: Report) -> Result<String> {
        let mut out = String::new();
        match kind {
            Report::Pvs => {
                out.push_str("PV VG Fmt PSize(B) PFree(B)\n");
                let used = self.used();
                for (p, d) in self
                    .devices
                    .iter()
                    .filter(|(_, d)| d.role == DeviceRole::PhysicalVolume)
                {
                    let (size, free) = if d.group.is_empty() {
                        (d.bytes - DATA_START, d.bytes - DATA_START)
                    } else {
                        let g = &self.groups[&d.group];
                        let all = self.pv_extents_for(d, g)? as u64;
                        let used = used.iter().filter(|(pv, _)| *pv == p.as_str()).count() as u64;
                        (all * g.extent_bytes, (all - used) * g.extent_bytes)
                    };
                    out.push_str(&format!(
                        "{p} {} lvm2 {size} {free}\n",
                        if d.group.is_empty() { "-" } else { &d.group }
                    ));
                }
            }
            Report::Vgs => {
                out.push_str("VG #PV #LV VSize(B) VFree(B) Extent(B)\n");
                for (vg, g) in &self.groups {
                    let count = self.devices.values().filter(|d| d.group == *vg).count();
                    let free = self.free_extents(vg)? as u64;
                    let used = g
                        .volumes
                        .values()
                        .map(|v| v.extents.len() as u64)
                        .sum::<u64>();
                    out.push_str(&format!(
                        "{vg} {count} {} {} {} {}\n",
                        g.volumes.len(),
                        (free + used) * g.extent_bytes,
                        free * g.extent_bytes,
                        g.extent_bytes
                    ));
                }
            }
            Report::Lvs => {
                out.push_str("LV VG LSize(B) #LE Type\n");
                for (vg, g) in &self.groups {
                    for (lv, v) in &g.volumes {
                        out.push_str(&format!(
                            "{lv} {vg} {} {} linear\n",
                            v.extents.len() as u64 * g.extent_bytes,
                            v.extents.len()
                        ));
                    }
                }
            }
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Report {
    Pvs,
    Vgs,
    Lvs,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Growth {
    Extents { amount: u64, relative: bool },
    Bytes { amount: u64, relative: bool },
    AllFree,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    Report(Report),
    PvCreate(Vec<String>),
    VgExtend { vg: String, paths: Vec<String> },
    LvExtend { id: String, growth: Growth },
    ResizeFs { id: String, blocks: Option<u64> },
}
fn decimal(s: &str) -> Result<u64> {
    if s.is_empty() || s.len() > 20 || !s.bytes().all(|c| c.is_ascii_digit()) {
        return Err(invalid(
            "Use a nonnegative decimal integer in the supported size syntax",
        ));
    }
    s.parse().map_err(|_| limit("Size integer overflow"))
}
fn bytes(s: &str, default: u64) -> Result<u64> {
    let last = s.chars().last().ok_or_else(|| invalid("Give a size"))?;
    let num = s.strip_suffix(last).unwrap();
    let (n, unit) = match last {
        'm' | 'M' => (num, MIB),
        'g' | 'G' => (num, 1024 * MIB),
        '0'..='9' => (s, default),
        _ => {
            return Err(error(
                ErrorKind::Unsupported,
                "Only integer sizes with M/G suffix letters are supported here; no fractions or other units",
            ));
        }
    };
    decimal(n)?
        .checked_mul(unit)
        .ok_or_else(|| limit("Size byte overflow"))
}
fn paths(raw: &[&str]) -> Result<Vec<String>> {
    if raw.is_empty() || raw.len() > MAX_DEVICES {
        return Err(limit("Choose 1..8 supplied virtual devices"));
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for p in raw {
        if !device_name(p) || !seen.insert(*p) {
            return Err(invalid("Use distinct supplied flat /dev/NAME devices"));
        }
        out.push((*p).into());
    }
    Ok(out)
}
fn parse(input: &str) -> Result<Action> {
    if input.len() > MAX_COMMAND_BYTES {
        return Err(limit("LVM command exceeds 2048 bytes before parsing"));
    }
    if input.contains([
        '\0', '\n', '\r', ';', '|', '&', '<', '>', '$', '`', '(', ')', '\\', '\'', '"',
    ]) {
        return Err(error(
            ErrorKind::Unsupported,
            "Use one supported LVM command, without shell operators, quoting or expansions; no host shell runs",
        ));
    }
    let args = input.split_ascii_whitespace().collect::<Vec<_>>();
    match args.as_slice() {
        ["pvs"] => Ok(Action::Report(Report::Pvs)),
        ["vgs"] => Ok(Action::Report(Report::Vgs)),
        ["lvs"] => Ok(Action::Report(Report::Lvs)),
        ["pvcreate", tail @ ..] => Ok(Action::PvCreate(paths(tail)?)),
        ["vgextend", vg, tail @ ..] if name(vg) => Ok(Action::VgExtend {
            vg: (*vg).into(),
            paths: paths(tail)?,
        }),
        ["lvextend", flag, value, id] if matches!(*flag, "-l" | "--extents" | "-L" | "--size") => {
            volume_name(id)?;
            let relative = value.starts_with('+');
            let number = value.strip_prefix('+').unwrap_or(value);
            let growth = if matches!(*flag, "-l" | "--extents") && *value == "+100%FREE" {
                Growth::AllFree
            } else if matches!(*flag, "-l" | "--extents") {
                Growth::Extents {
                    amount: decimal(number)?,
                    relative,
                }
            } else {
                Growth::Bytes {
                    amount: bytes(number, MIB)?,
                    relative,
                }
            };
            Ok(Action::LvExtend {
                id: (*id).into(),
                growth,
            })
        }
        ["resize2fs", id] => {
            volume_name(id)?;
            Ok(Action::ResizeFs {
                id: (*id).into(),
                blocks: None,
            })
        }
        ["resize2fs", id, size] => {
            volume_name(id)?;
            let amount = bytes(size, BLOCK_BYTES)?;
            if amount % BLOCK_BYTES != 0 {
                return Err(invalid(
                    "Filesystem size must contain whole 4096-byte blocks",
                ));
            }
            Ok(Action::ResizeFs {
                id: (*id).into(),
                blocks: Some(amount / BLOCK_BYTES),
            })
        }
        _ => Err(error(ErrorKind::Unsupported, HELP)),
    }
}

/// Pure outcome checks for a future authored assignment; not catalog registration.
pub fn growth_checks(
    state: &State,
    id: &str,
    minimum_lv: u64,
    minimum_filesystem: u64,
    required_files: &BTreeMap<String, File>,
) -> Vec<(bool, String)> {
    if let Err(e) = state.validate() {
        return vec![(false, e.to_string())];
    }
    let (lv, fs) = (state.lv_bytes(id), state.filesystem_bytes(id));
    vec![
        (
            lv.is_ok_and(|n| n >= minimum_lv),
            "Logical volume has the requested backing capacity".into(),
        ),
        (
            fs.is_ok_and(|n| n >= minimum_filesystem),
            "Filesystem has grown to the requested capacity; LV growth alone is insufficient"
                .into(),
        ),
        (
            state.volume(id).is_ok_and(|(_, v)| {
                v.filesystem
                    .as_ref()
                    .is_some_and(|f| &f.files == required_files)
            }),
            "The complete required virtual file set and contents are preserved".into(),
        ),
    ]
}
