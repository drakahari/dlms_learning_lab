//! Virtual GPT allocation and blank-partition preparation. No disk I/O.
//! Fixed 512-byte sectors and 128-entry GPT geometry, not raw GPT images.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: u32 = 1;
pub const SECTOR_BYTES: u64 = 512;
pub const BLOCK_BYTES: u64 = 4096;
pub const FIRST_USABLE: u64 = 34;
pub const MAX_SECTORS: u64 = 16 * 1024 * 1024 * 1024 * 1024 / SECTOR_BYTES;
pub const MAX_DISKS: usize = 4;
pub const MAX_PARTITIONS: usize = 16;
pub const MAX_FILES: usize = 16;
pub const MAX_FILE_BYTES: usize = 16_384;
pub const MAX_TOTAL_FILE_BYTES: usize = 131_072;
pub const MAX_STATE_BYTES: usize = 262_144;
pub const MAX_COMMAND_BYTES: usize = 2048;
pub const HELP: &str = "Virtual preparation only: parted -s /dev/vdX mklabel gpt; parted -s /dev/vdX unit s mkpart NAME ext4|linux-swap STARTs ENDs; parted -s /dev/vdX unit s print; parted -s /dev/vdX align-check optimal NUMBER; mkfs.ext4 [-b 4096] [-L LABEL] [-U UUID] PARTITION (also mkfs -t ext4 or mke2fs -t ext4); mkswap [-L LABEL] [-U UUID] PARTITION; blkid [PARTITION]. Exact positive sectors and supplied /dev/vd[a-z] paths only. GPT type hints do not format partitions. Reports are simulator reports. Only blank disks/partitions can be initialized; this simulator policy is not real-tool protection. No MBR, logical/extended partitions, deletion, resizing, force, other filesystems, raw GPT bytes/CRC, mounts, swap activation, shell or host commands.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Unknown,
    Occupied,
    Overlap,
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
fn err(kind: ErrorKind, message: &str) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn invalid(s: &str) -> Error {
    err(ErrorKind::Invalid, s)
}
fn limited(s: &str) -> Error {
    err(ErrorKind::Limit, s)
}

// Fixed-schema streaming admission: refuse excess keys/items before their bodies.
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
                    return Err(E::custom("Partition text bound before copy"));
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
            write!(f, "at most {N} unique entries")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            use serde::de::Error;
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key_seed(Text::<64>)? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom(
                        "Partition map count/duplicate before value",
                    ));
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
            write!(f, "at most {N} items")
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
                    Err(D::Error::custom("Partition item bound before excess body"))
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
fn disk_path(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 8 && &b[..7] == b"/dev/vd" && b[7].is_ascii_lowercase()
}
fn name(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        && !s.starts_with(['.', '-'])
}
fn label(s: &str) -> bool {
    s.len() <= 16
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
        && s.bytes().any(|b| b != b'0' && b != b'-')
}
fn split_partition(s: &str) -> Result<(&str, u8)> {
    if s.len() < 9 || s.get(..8).is_none_or(|prefix| !disk_path(prefix)) {
        return Err(invalid("Use an exact supplied /dev/vdX partition path"));
    }
    let n = &s[8..];
    if n.starts_with('0') || !n.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("Partition aliases/leading zeros are not expanded"));
    }
    let number = n
        .parse::<u8>()
        .map_err(|_| invalid("Invalid partition number"))?;
    if number == 0 || usize::from(number) > MAX_PARTITIONS {
        return Err(invalid("Partition number outside modeled slots1..16"));
    }
    Ok((&s[..8], number))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    /// Simulator identity sequence; not entropy or a real-disk UUID generator.
    pub next_uuid: u64,
    #[serde(deserialize_with = "map::<_,Disk,MAX_DISKS>")]
    pub disks: BTreeMap<String, Disk>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    pub sectors: u64,
    pub sector_bytes: u64,
    pub optimal_alignment: u64,
    pub writable: bool,
    pub in_use: bool,
    /// Opaque pre-existing unpartitioned data; never erased by this subset.
    pub occupied_without_table: bool,
    pub table: Option<Gpt>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gpt {
    #[serde(deserialize_with = "sequence::<_,Partition,MAX_PARTITIONS>")]
    pub partitions: Vec<Partition>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeHint {
    LinuxData,
    LinuxSwap,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Partition {
    pub number: u8,
    #[serde(deserialize_with = "text::<_,36>")]
    pub name: String,
    pub start: u64,
    pub end: u64,
    pub type_hint: TypeHint,
    pub content: Content,
}
impl Partition {
    pub fn bytes(&self) -> Result<u64> {
        self.end
            .checked_sub(self.start)
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(SECTOR_BYTES))
            .ok_or_else(|| invalid("Invalid/overflowing inclusive partition extent"))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Content {
    Blank,
    Foreign {
        #[serde(deserialize_with = "text::<_,64>")]
        description: String,
    },
    Ext4 {
        #[serde(deserialize_with = "text::<_,16>")]
        label: String,
        #[serde(deserialize_with = "text::<_,36>")]
        uuid: String,
        blocks: u64,
        #[serde(deserialize_with = "map::<_,File,MAX_FILES>")]
        files: BTreeMap<String, File>,
    },
    Swap {
        #[serde(deserialize_with = "text::<_,16>")]
        label: String,
        #[serde(deserialize_with = "text::<_,36>")]
        uuid: String,
        usable_pages: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(deserialize_with = "text::<_,MAX_FILE_BYTES>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    CreateGpt {
        disk: String,
    },
    AddPartition {
        disk: String,
        name: String,
        start: u64,
        end: u64,
        type_hint: TypeHint,
    },
    FormatExt4 {
        partition: String,
        label: String,
        uuid: Option<String>,
    },
    InitializeSwap {
        partition: String,
        label: String,
        uuid: Option<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub status: u8,
}
impl Output {
    fn success(stdout: String) -> Self {
        Self { stdout, status: 0 }
    }
}

// Count escaped JSON bytes without allocating an unbounded serialization buffer.
fn wire_size(state: &State) -> Result<()> {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= MAX_STATE_BYTES)
                .ok_or_else(|| std::io::Error::other("Partition wire limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter(0), state)
        .map_err(|_| limited("Escaped partition state exceeds256KiB"))
}
impl State {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.disks.is_empty()
            || self.disks.len() > MAX_DISKS
            || self.next_uuid == 0
            || self.next_uuid > 0xffff_ffff_ffff
        {
            return Err(invalid(
                "Invalid partition state version/count/identity sequence",
            ));
        }
        let mut identities = BTreeSet::new();
        let mut total = 0usize;
        for (id, d) in &self.disks {
            if !disk_path(id)
                || d.sector_bytes != SECTOR_BYTES
                || d.sectors < 128
                || d.sectors > MAX_SECTORS
                || !d.optimal_alignment.is_power_of_two()
                || d.optimal_alignment > 8192
            {
                return Err(invalid(
                    "Supplied disks need128 sectors through16TiB,512-byte logical sectors and bounded power-of-two alignment",
                ));
            }
            if let Some(t) = &d.table {
                if d.occupied_without_table || t.partitions.len() > MAX_PARTITIONS {
                    return Err(invalid(
                        "Invalid GPT/opaque-content combination or slot count",
                    ));
                }
                let mut numbers = BTreeSet::new();
                for (i, p) in t.partitions.iter().enumerate() {
                    if p.number == 0
                        || usize::from(p.number) > MAX_PARTITIONS
                        || !numbers.insert(p.number)
                        || !name(&p.name, 36)
                        || p.start < FIRST_USABLE
                        || p.end > d.sectors - 34
                        || p.start > p.end
                    {
                        return Err(invalid("Invalid GPT name,number or reserved-sector range"));
                    }
                    if t.partitions[..i]
                        .iter()
                        .any(|q| p.start <= q.end && q.start <= p.end)
                    {
                        return Err(err(ErrorKind::Overlap, "GPT partition ranges overlap"));
                    }
                    let bytes = p.bytes()?;
                    match &p.content {
                        Content::Blank => {}
                        Content::Foreign { description } => {
                            if description.is_empty()
                                || description.len() > 64
                                || !description
                                    .bytes()
                                    .all(|b| b.is_ascii_graphic() || b == b' ')
                            {
                                return Err(invalid("Invalid opaque content description"));
                            }
                        }
                        Content::Ext4 {
                            label: l,
                            uuid: u,
                            blocks,
                            files,
                        } => {
                            if !label(l)
                                || !uuid(u)
                                || !identities.insert(u.as_str())
                                || *blocks != bytes / BLOCK_BYTES
                                || bytes < 1024 * 1024
                                || files.len() > MAX_FILES
                            {
                                return Err(invalid("Invalid ext4 identity/capacity/file count"));
                            }
                            let mut used = 0u64;
                            for (path, f) in files {
                                if !path.strip_prefix('/').is_some_and(|s| name(s, 63))
                                    || f.text.len() > MAX_FILE_BYTES
                                    || f.text.contains('\0')
                                {
                                    return Err(invalid("Invalid retained flat file path/text"));
                                }
                                total = total
                                    .checked_add(f.text.len())
                                    .ok_or_else(|| limited("File byte overflow"))?;
                                used = used
                                    .checked_add(f.text.len() as u64)
                                    .ok_or_else(|| limited("File byte overflow"))?;
                                if total > MAX_TOTAL_FILE_BYTES || used > blocks * BLOCK_BYTES {
                                    return Err(limited(
                                        "Retained file bytes exceed bounded modeled capacity",
                                    ));
                                }
                            }
                        }
                        Content::Swap {
                            label: l,
                            uuid: u,
                            usable_pages,
                        } => {
                            if !label(l)
                                || !uuid(u)
                                || !identities.insert(u.as_str())
                                || bytes / BLOCK_BYTES < 10
                                || *usable_pages != bytes / BLOCK_BYTES - 1
                            {
                                return Err(invalid("Invalid swap identity/page capacity"));
                            }
                        }
                    }
                }
            }
        }
        wire_size(self)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(limited("Partition raw JSON exceeds256KiB before decoding"));
        }
        let out: Self =
            serde_json::from_str(raw).map_err(|_| invalid("Invalid bounded partition JSON"))?;
        out.validate()?;
        Ok(out)
    }
    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| invalid("Could not encode validated virtual state"))
    }
    pub fn environment_matches(&self, source: &Self) -> bool {
        self.validate().is_ok()
            && source.validate().is_ok()
            && self.disks.len() == source.disks.len()
            && self.disks.iter().all(|(id, d)| {
                source.disks.get(id).is_some_and(|s| {
                    d.sectors == s.sectors
                        && d.sector_bytes == s.sector_bytes
                        && d.optimal_alignment == s.optimal_alignment
                        && d.writable == s.writable
                        && d.in_use == s.in_use
                        && d.occupied_without_table == s.occupied_without_table
                })
            })
    }
    pub fn reset_from(&mut self, source: &Self) -> Result<()> {
        self.validate()?;
        source.validate()?;
        if !self.environment_matches(source) {
            return Err(invalid("Reset source has different supplied device facts"));
        }
        *self = source.clone();
        Ok(())
    }
    fn disk(&self, id: &str) -> Result<&Disk> {
        if !disk_path(id) {
            return Err(invalid("Exact supplied /dev/vdX path required; no aliases"));
        }
        self.disks
            .get(id)
            .ok_or_else(|| err(ErrorKind::Unknown, "No such supplied virtual disk"))
    }
    fn partition(&self, id: &str) -> Result<(&Disk, &Partition)> {
        let (disk, n) = split_partition(id)?;
        let d = self.disk(disk)?;
        let p = d
            .table
            .as_ref()
            .and_then(|t| t.partitions.iter().find(|p| p.number == n))
            .ok_or_else(|| err(ErrorKind::Unknown, "No such supplied virtual partition"))?;
        Ok((d, p))
    }
    fn writable(d: &Disk) -> Result<()> {
        if !d.writable || d.in_use {
            Err(err(
                ErrorKind::Occupied,
                "Supplied disk is read-only or in use",
            ))
        } else {
            Ok(())
        }
    }
    fn uuid_exists(&self, u: &str) -> bool {
        self.disks
            .values()
            .filter_map(|d| d.table.as_ref())
            .flat_map(|t| &t.partitions)
            .any(|p| matches!(&p.content,Content::Ext4{uuid,..}|Content::Swap{uuid,..} if uuid==u))
    }
    fn new_uuid(&self, given: &Option<String>) -> Result<(String, u64)> {
        if let Some(u) = given {
            if !uuid(u) {
                return Err(invalid("Use a nonzero canonical lowercase UUID"));
            }
            if self.uuid_exists(u) {
                return Err(err(
                    ErrorKind::Occupied,
                    "Filesystem/swap UUID already exists",
                ));
            }
            return Ok((u.clone(), self.next_uuid));
        }
        let mut n = self.next_uuid;
        for _ in 0..=MAX_DISKS * MAX_PARTITIONS {
            if n >= 0xffff_ffff_ffff {
                return Err(limited("Simulator UUID sequence exhausted"));
            }
            let u = format!("00000000-0000-4000-8000-{n:012x}");
            n += 1;
            if !self.uuid_exists(&u) {
                return Ok((u, n));
            }
        }
        Err(limited("Simulator identity allocation exhausted"))
    }
    fn preflight(&self, a: &Action) -> Result<Option<(String, u64)>> {
        match a {
            Action::CreateGpt { disk } => {
                let d = self.disk(disk)?;
                Self::writable(d)?;
                if d.table.is_some() || d.occupied_without_table {
                    return Err(err(
                        ErrorKind::Occupied,
                        "Preparation-only model refuses replacement of any table or existing data",
                    ));
                }
            }
            Action::AddPartition {
                disk,
                name: n,
                start,
                end,
                ..
            } => {
                let d = self.disk(disk)?;
                Self::writable(d)?;
                let t = d
                    .table
                    .as_ref()
                    .ok_or_else(|| invalid("Create GPT before allocating a partition"))?;
                if !name(n, 36) || *start < FIRST_USABLE || *end > d.sectors - 34 || start > end {
                    return Err(invalid(
                        "Name or inclusive sector range invalid; keep GPT reserved sectors free",
                    ));
                }
                if t.partitions.len() >= MAX_PARTITIONS {
                    return Err(limited("All16 modeled GPT slots occupied"));
                }
                if t.partitions
                    .iter()
                    .any(|p| *start <= p.end && p.start <= *end)
                {
                    return Err(err(
                        ErrorKind::Overlap,
                        "Requested partition overlaps an existing partition",
                    ));
                }
            }
            Action::FormatExt4 {
                partition,
                label: l,
                uuid: u,
            }
            | Action::InitializeSwap {
                partition,
                label: l,
                uuid: u,
            } => {
                let (d, p) = self.partition(partition)?;
                Self::writable(d)?;
                if p.content != Content::Blank {
                    return Err(err(
                        ErrorKind::Occupied,
                        "Preparation-only simulator refuses overwriting any signature/data; real utilities can be destructive",
                    ));
                }
                if !label(l) {
                    return Err(invalid("Label must be at most16 ASCII letters/digits/._-"));
                }
                let bytes = p.bytes()?;
                if matches!(a, Action::FormatExt4 { .. }) && bytes < 1024 * 1024 {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "This ext4 teaching profile requires at least1MiB; real small-filesystem feature behavior is not modeled",
                    ));
                }
                if matches!(a, Action::InitializeSwap { .. }) && bytes / BLOCK_BYTES < 10 {
                    return Err(invalid("Swap needs at least10 modeled4096-byte pages"));
                }
                return self.new_uuid(u).map(Some);
            }
        }
        Ok(None)
    }
    pub fn apply(&mut self, a: Action) -> Result<Output> {
        self.validate()?;
        let identity = self.preflight(&a)?; // Admission before copying current state.
        let mut next = self.clone();
        match a {
            Action::CreateGpt { disk } => {
                next.disks.get_mut(&disk).expect("preflight disk").table = Some(Gpt {
                    partitions: Vec::new(),
                })
            }
            Action::AddPartition {
                disk,
                name,
                start,
                end,
                type_hint,
            } => {
                let t = next
                    .disks
                    .get_mut(&disk)
                    .expect("preflight disk")
                    .table
                    .as_mut()
                    .expect("preflight GPT");
                let number = (1..=MAX_PARTITIONS as u8)
                    .find(|n| !t.partitions.iter().any(|p| p.number == *n))
                    .expect("preflight free slot");
                t.partitions.push(Partition {
                    number,
                    name,
                    start,
                    end,
                    type_hint,
                    content: Content::Blank,
                });
                t.partitions.sort_by_key(|p| p.number);
            }
            a @ (Action::FormatExt4 { .. } | Action::InitializeSwap { .. }) => {
                let swap = matches!(a, Action::InitializeSwap { .. });
                let (partition, label) = match a {
                    Action::FormatExt4 {
                        partition, label, ..
                    }
                    | Action::InitializeSwap {
                        partition, label, ..
                    } => (partition, label),
                    _ => unreachable!(),
                };
                let (disk, n) = split_partition(&partition)?;
                let (uuid, serial) = identity.expect("preflight identity");
                let p = next
                    .disks
                    .get_mut(disk)
                    .expect("preflight disk")
                    .table
                    .as_mut()
                    .expect("preflight table")
                    .partitions
                    .iter_mut()
                    .find(|p| p.number == n)
                    .expect("preflight partition");
                let blocks = p.bytes()? / BLOCK_BYTES;
                p.content = if swap {
                    Content::Swap {
                        label,
                        uuid,
                        usable_pages: blocks - 1,
                    }
                } else {
                    Content::Ext4 {
                        label,
                        uuid,
                        blocks,
                        files: BTreeMap::new(),
                    }
                };
                next.next_uuid = serial;
            }
        }
        next.validate()?;
        *self = next;
        Ok(Output::success(
            "Virtual preparation transaction completed\n".into(),
        ))
    }
    pub fn execute(&mut self, input: &str) -> Result<Output> {
        self.validate()?;
        if input.is_empty()
            || input.len() > MAX_COMMAND_BYTES
            || !input.bytes().all(|b| b.is_ascii_graphic() || b == b' ')
            || input.bytes().any(|b| b";|&<>`$\\\"'(){}[]*?!".contains(&b))
        {
            return Err(err(
                ErrorKind::Unsupported,
                "Use one bounded literal command; no shell syntax/control characters",
            ));
        }
        let v: Vec<&str> = input.split_ascii_whitespace().collect();
        match v.as_slice() {
            ["parted", "-s", d, "mklabel", "gpt"] => {
                self.apply(Action::CreateGpt { disk: (*d).into() })
            }
            ["parted", "-s", d, "unit", "s", "mkpart", n, t, start, end] => {
                let type_hint = match *t {
                    "ext4" => TypeHint::LinuxData,
                    "linux-swap" => TypeHint::LinuxSwap,
                    _ => return Err(err(ErrorKind::Unsupported, HELP)),
                };
                self.apply(Action::AddPartition {
                    disk: (*d).into(),
                    name: (*n).into(),
                    start: sector(start)?,
                    end: sector(end)?,
                    type_hint,
                })
            }
            ["parted", "-s", d, "unit", "s", "print"] => self.report_disk(d),
            ["parted", "-s", d, "align-check", "optimal", n] => {
                let disk = self.disk(d)?;
                let number = integer(n)?;
                let p = disk
                    .table
                    .as_ref()
                    .and_then(|t| t.partitions.iter().find(|p| u64::from(p.number) == number))
                    .ok_or_else(|| err(ErrorKind::Unknown, "Unknown partition number"))?;
                Ok(Output {
                    stdout: String::new(),
                    status: u8::from(!p.start.is_multiple_of(disk.optimal_alignment)),
                })
            }
            ["blkid"] => self.report_blkid(None),
            ["blkid", p] => self.report_blkid(Some(p)),
            _ => {
                let (swap, options) = match v.as_slice() {
                    ["mkfs.ext4", rest @ ..] => (false, rest),
                    ["mkfs" | "mke2fs", "-t", "ext4", rest @ ..] => (false, rest),
                    ["mkswap", rest @ ..] => (true, rest),
                    _ => return Err(err(ErrorKind::Unsupported, HELP)),
                };
                let (partition, label, uuid) = format_options(options, swap)?;
                self.apply(if swap {
                    Action::InitializeSwap {
                        partition,
                        label,
                        uuid,
                    }
                } else {
                    Action::FormatExt4 {
                        partition,
                        label,
                        uuid,
                    }
                })
            }
        }
    }
    pub fn report_disk(&self, id: &str) -> Result<Output> {
        self.validate()?;
        let d = self.disk(id)?;
        let mut s = format!(
            "SIMULATOR GPT report: {id}; {} sectors; logical512B; optimal{} sectors\n",
            d.sectors, d.optimal_alignment
        );
        if let Some(t) = &d.table {
            s.push_str(&format!(
                "Usable inclusive LBAs:34..{}; Number Start End Sectors Name TypeHint Signature\n",
                d.sectors - 34
            ));
            for p in &t.partitions {
                s.push_str(&format!(
                    "{} {} {} {} {} {:?} {}\n",
                    p.number,
                    p.start,
                    p.end,
                    p.end - p.start + 1,
                    p.name,
                    p.type_hint,
                    content_type(&p.content)
                ));
            }
        } else {
            s.push_str(if d.occupied_without_table {
                "No modeled table; opaque existing data\n"
            } else {
                "Blank disk; no partition table\n"
            });
        }
        Ok(Output::success(s))
    }
    pub fn report_blkid(&self, only: Option<&str>) -> Result<Output> {
        self.validate()?;
        if let Some(p) = only {
            self.partition(p)?;
        }
        let mut s = String::from("SIMULATOR signature inventory (not host blkid output)\n");
        let mut found = false;
        for (id, d) in &self.disks {
            if let Some(t) = &d.table {
                for p in &t.partitions {
                    let path = format!("{id}{}", p.number);
                    if only.is_some_and(|n| n != path) {
                        continue;
                    }
                    match &p.content {
                        Content::Ext4 { uuid, label, .. } | Content::Swap { uuid, label, .. } => {
                            found = true;
                            s.push_str(&format!(
                                "{path}: UUID=\"{uuid}\" LABEL=\"{label}\" TYPE=\"{}\"\n",
                                content_type(&p.content)
                            ));
                        }
                        Content::Foreign { description } => {
                            found = true;
                            s.push_str(&format!(
                                "{path}: opaque supplied content ({description})\n"
                            ));
                        }
                        Content::Blank => {}
                    }
                }
            }
        }
        Ok(Output {
            stdout: s,
            status: if found { 0 } else { 2 },
        })
    }
}
fn content_type(c: &Content) -> &'static str {
    match c {
        Content::Blank => "none",
        Content::Foreign { .. } => "foreign",
        Content::Ext4 { .. } => "ext4",
        Content::Swap { .. } => "swap",
    }
}
fn integer(s: &str) -> Result<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || (s.len() > 1 && s.starts_with('0'))
    {
        return Err(invalid("Use canonical unsigned decimal integers"));
    }
    s.parse().map_err(|_| invalid("Integer overflow"))
}
fn sector(s: &str) -> Result<u64> {
    integer(s.strip_suffix('s').ok_or_else(|| {
        invalid("Use exact sector suffix s; no percentages/negative/unit rounding")
    })?)
}
fn format_options(v: &[&str], swap: bool) -> Result<(String, String, Option<String>)> {
    let (last, options) = v
        .split_last()
        .ok_or_else(|| invalid("Missing supplied partition path"))?;
    split_partition(last)?;
    let mut l = None;
    let mut u = None;
    let mut block = false;
    let mut i = 0;
    while i < options.len() {
        let value = options
            .get(i + 1)
            .ok_or_else(|| invalid("Missing formatting option value"))?;
        match options[i] {
            "-L" if l.is_none() => {
                if !label(value) {
                    return Err(invalid("Invalid bounded label"));
                }
                l = Some((*value).into());
            }
            "-U" if u.is_none() => {
                if !uuid(value) {
                    return Err(invalid("Invalid canonical UUID"));
                }
                u = Some((*value).into());
            }
            "-b" if !swap && !block && *value == "4096" => block = true,
            _ => {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Only nonduplicate -L/-U and ext4 -b4096 are supported; no force/overwrite/features",
                ));
            }
        }
        i += 2;
    }
    Ok(((*last).into(), l.unwrap_or_default(), u))
}
