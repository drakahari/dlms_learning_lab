//! Original bounded consistency graph, not an ext4 image/parser or host fsck.
//! All findings are derived; reports are never imported as completion authority.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};

pub const VERSION: u32 = 1;
pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_FILES: usize = 32;
pub const MAX_ENTRIES: usize = 64;
pub const MAX_BLOCKS: u32 = 256;
pub const MAX_INODES: u32 = 128;
pub const FIRST_DATA_BLOCK: u32 = 8;
pub const FIRST_FILE_INODE: u32 = 12;
pub const BLOCK_BYTES: usize = 4096;
pub const MAX_CONTENT: usize = 16384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    Mounted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}

pub(crate) fn sequence<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Values<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Values<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} integrity items")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = vec![];
            while out.len() < N {
                let Some(v) = a.next_element()? else {
                    return Ok(out);
                };
                out.push(v);
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> std::result::Result<Self, D::Error> {
                    Err(D::Error::custom("Integrity item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Values::<T, N>(PhantomData))
}
fn mapping<
    'de,
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
    const N: usize,
>(
    d: D,
) -> std::result::Result<BTreeMap<K, V>, D::Error> {
    struct Values<K, V, const N: usize>(PhantomData<(K, V)>);
    impl<'de, K: Deserialize<'de> + Ord, V: Deserialize<'de>, const N: usize> Visitor<'de>
        for Values<K, V, N>
    {
        type Value = BTreeMap<K, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} distinct integrity keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while out.len() < N {
                let Some(key) = a.next_key()? else {
                    return Ok(out);
                };
                if out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate integrity key"));
                }
                out.insert(key, a.next_value()?);
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> std::result::Result<Self, D::Error> {
                    Err(D::Error::custom("Integrity key count limit"))
                }
            }
            let _ = a.next_key::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_map(Values::<K, V, N>(PhantomData))
}
fn bitmap<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeSet<u32>, D::Error> {
    let values = sequence::<D, u32, N>(d)?;
    let n = values.len();
    let set: BTreeSet<_> = values.into_iter().collect();
    if set.len() != n {
        return Err(D::Error::custom("Duplicate integrity bitmap value"));
    }
    Ok(set)
}
fn entries<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, u32>, D::Error> {
    #[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,80>")] String);
    mapping::<D, Name, u32, MAX_ENTRIES>(d).map(|m| m.into_iter().map(|(k, v)| (k.0, v)).collect())
}
// Tagged scenario enums buffer JSON object keys as strings. Decode the same
// canonical decimal inode keys both directly and through that buffered content.
fn files<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeMap<u32, Inode>, D::Error> {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Key(u32);
    impl<'de> Deserialize<'de> for Key {
        fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
            struct Decimal;
            impl Visitor<'_> for Decimal {
                type Value = Key;
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a canonical decimal inode key")
                }
                fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<Key, E> {
                    if s.is_empty()
                        || s.len() > 3
                        || !s.bytes().all(|b| b.is_ascii_digit())
                        || (s.len() > 1 && s.starts_with('0'))
                    {
                        return Err(E::custom("Invalid canonical inode key"));
                    }
                    let value = s.parse::<u32>().map_err(E::custom)?;
                    if !(FIRST_FILE_INODE..=MAX_INODES).contains(&value) {
                        return Err(E::custom("Inode key outside supported bounds"));
                    }
                    Ok(Key(value))
                }
            }
            d.deserialize_str(Decimal)
        }
    }
    mapping::<D, Key, Inode, MAX_FILES>(d)
        .map(|values| values.into_iter().map(|(k, v)| (k.0, v)).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inode {
    #[serde(deserialize_with = "crate::cron::text::<_,MAX_CONTENT>")]
    pub content: String,
    #[serde(deserialize_with = "sequence::<_,u32,4>")]
    pub blocks: Vec<u32>,
    pub links: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    pub version: u32,
    pub block_count: u32,
    pub inode_count: u32,
    #[serde(deserialize_with = "files")]
    pub files: BTreeMap<u32, Inode>,
    #[serde(deserialize_with = "entries")]
    pub entries: BTreeMap<String, u32>,
    #[serde(deserialize_with = "bitmap::<_,256>")]
    pub allocated_blocks: BTreeSet<u32>,
    #[serde(deserialize_with = "bitmap::<_,128>")]
    pub allocated_inodes: BTreeSet<u32>,
    pub free_blocks: u32,
    pub free_inodes: u32,
    pub mounted: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    MissingInode {
        name: String,
        inode: u32,
    },
    MultiplyClaimed {
        block: u32,
        owners: Vec<u32>,
    },
    Unattached {
        inode: u32,
        empty: bool,
    },
    LinkCount {
        inode: u32,
        recorded: u16,
        actual: u16,
    },
    BlockBitmap {
        missing: Vec<u32>,
        extra: Vec<u32>,
    },
    InodeBitmap {
        missing: Vec<u32>,
        extra: Vec<u32>,
    },
    FreeBlocks {
        recorded: u32,
        actual: u32,
    },
    FreeInodes {
        recorded: u32,
        actual: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    ReadOnly,
    Preen,
    Yes,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Correction {
    Reconnected { inode: u32, name: String },
    ClearedEmpty { inode: u32 },
    LinkCount { inode: u32, value: u16 },
    BlockBitmap,
    InodeBitmap,
    FreeBlocks,
    FreeInodes,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub status: u8,
    pub before: Vec<Finding>,
    pub after: Vec<Finding>,
    pub corrections: Vec<Correction>,
    pub blocked: Option<String>,
}

fn name(s: &str) -> bool {
    let leaf = s.strip_prefix("lost+found/").unwrap_or(s);
    !leaf.is_empty()
        && leaf.len() <= 64
        && !leaf.starts_with('.')
        && leaf
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-#".contains(&b))
}
impl Volume {
    /// Structural admission allows modeled inconsistency, not unbounded/corrupt formats.
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(error(
                ErrorKind::Unsupported,
                "Unsupported virtual integrity version",
            ));
        }
        if !(16..=MAX_BLOCKS).contains(&self.block_count)
            || !(16..=MAX_INODES).contains(&self.inode_count)
            || self.files.len() > MAX_FILES
            || self.entries.len() > MAX_ENTRIES
            || self.allocated_blocks.len() > MAX_BLOCKS as usize
            || self.allocated_inodes.len() > MAX_INODES as usize
            || !crate::cron_workspace::within(self, MAX_BYTES)
        {
            return Err(error(
                ErrorKind::Limit,
                "Virtual integrity version/size/count bound exceeded",
            ));
        }
        for (ino, f) in &self.files {
            if !(FIRST_FILE_INODE..=self.inode_count).contains(ino)
                || f.content.len() > MAX_CONTENT
                || f.blocks.len() > 4
            {
                return Err(error(
                    ErrorKind::Limit,
                    "File inode/content/reference bound exceeded",
                ));
            }
            if f.blocks.len() != f.content.len().div_ceil(BLOCK_BYTES)
                || f.blocks
                    .iter()
                    .any(|b| !(FIRST_DATA_BLOCK..self.block_count).contains(b))
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Only complete nonsparse regular-file contents with in-range data references are modeled; no extent, truncation or out-of-range repair is guessed",
                ));
            }
        }
        if self
            .entries
            .iter()
            .any(|(n, i)| !name(n) || !(FIRST_FILE_INODE..=self.inode_count).contains(i))
            || self.allocated_blocks.iter().any(|b| *b >= self.block_count)
            || self
                .allocated_inodes
                .iter()
                .any(|i| *i == 0 || *i > self.inode_count)
        {
            return Err(error(
                ErrorKind::Invalid,
                "Unknown directory shape or out-of-range bitmap/inode value",
            ));
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(error(ErrorKind::Limit, "Integrity input exceeds1MiB"));
        }
        let volume: Self = serde_json::from_slice(bytes)
            .map_err(|e| error(ErrorKind::Invalid, format!("Invalid integrity data: {e}")))?;
        volume.validate()?;
        Ok(volume)
    }
    fn references(&self, ino: u32) -> u16 {
        self.entries.values().filter(|n| **n == ino).count() as u16
    }
    fn expected_blocks(&self) -> BTreeSet<u32> {
        (0..FIRST_DATA_BLOCK)
            .chain(self.files.values().flat_map(|f| f.blocks.iter().copied()))
            .collect()
    }
    fn expected_inodes(&self) -> BTreeSet<u32> {
        (1..FIRST_FILE_INODE)
            .chain(self.files.keys().copied())
            .collect()
    }
    pub fn findings(&self) -> Result<Vec<Finding>> {
        self.validate()?;
        let mut out = vec![];
        for (n, i) in &self.entries {
            if !self.files.contains_key(i) {
                out.push(Finding::MissingInode {
                    name: n.clone(),
                    inode: *i,
                })
            }
        }
        let mut owners: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for (i, f) in &self.files {
            for b in &f.blocks {
                owners.entry(*b).or_default().push(*i)
            }
            let refs = self.references(*i);
            if refs == 0 {
                out.push(Finding::Unattached {
                    inode: *i,
                    empty: f.content.is_empty(),
                })
            }
            if refs != f.links {
                out.push(Finding::LinkCount {
                    inode: *i,
                    recorded: f.links,
                    actual: refs,
                })
            }
        }
        for (block, owners) in owners {
            if owners.len() > 1 {
                out.push(Finding::MultiplyClaimed { block, owners })
            }
        }
        let blocks = self.expected_blocks();
        let inodes = self.expected_inodes();
        if self.allocated_blocks != blocks {
            out.push(Finding::BlockBitmap {
                missing: blocks.difference(&self.allocated_blocks).copied().collect(),
                extra: self.allocated_blocks.difference(&blocks).copied().collect(),
            })
        }
        if self.allocated_inodes != inodes {
            out.push(Finding::InodeBitmap {
                missing: inodes.difference(&self.allocated_inodes).copied().collect(),
                extra: self.allocated_inodes.difference(&inodes).copied().collect(),
            })
        }
        let free = self.block_count - blocks.len() as u32;
        if self.free_blocks != free {
            out.push(Finding::FreeBlocks {
                recorded: self.free_blocks,
                actual: free,
            })
        }
        let free = self.inode_count - inodes.len() as u32;
        if self.free_inodes != free {
            out.push(Finding::FreeInodes {
                recorded: self.free_inodes,
                actual: free,
            })
        }
        Ok(out)
    }
    /// Strict exercise policy: no mounted findings or guessed unsupported repair.
    pub fn check(&mut self, mode: Mode) -> Result<Report> {
        self.validate()?;
        if self.mounted {
            return Err(error(
                ErrorKind::Mounted,
                "Unmount the virtual volume before checking. This exercise refuses mounted checks: real read-only mounted findings are unreliable, and repair is unsafe.",
            ));
        }
        let before = self.findings()?;
        if mode == Mode::ReadOnly || before.is_empty() {
            return Ok(Report {
                status: if before.is_empty() { 0 } else { 4 },
                after: before.clone(),
                before,
                corrections: vec![],
                blocked: None,
            });
        }
        if before.iter().any(|f| {
            matches!(
                f,
                Finding::MissingInode { .. } | Finding::MultiplyClaimed { .. }
            )
        }) {
            return Ok(Report{status:4,after:before.clone(),before,corrections:vec![],blocked:Some("This graph has missing inode data or multiply claimed blocks. The simulator cannot determine the original bytes; this repair is unsupported and leaves the volume unchanged.".into())});
        }
        let mut next = self.clone();
        let mut corrections = vec![];
        let ids: Vec<_> = next.files.keys().copied().collect();
        for ino in ids {
            if next.references(ino) == 0 {
                if next.files[&ino].content.is_empty() {
                    next.files.remove(&ino);
                    corrections.push(Correction::ClearedEmpty { inode: ino });
                    continue;
                }
                if mode == Mode::Preen {
                    let after = next.findings()?;
                    let status = if corrections.is_empty() { 4 } else { 5 };
                    *self = next;
                    return Ok(Report {
                        status,
                        before,
                        after,
                        corrections,
                        blocked: Some(format!(
                            "Inode {ino} retains data but has no directory name. Preen cannot make that recovery decision; no later bitmap repairs were attempted."
                        )),
                    });
                }
                if next.entries.len() == MAX_ENTRIES {
                    return Err(error(
                        ErrorKind::Limit,
                        "No modeled directory slot remains for recovery; volume unchanged",
                    ));
                }
                let base = format!("lost+found/#{ino}");
                let mut target = None;
                for suffix in 0..=MAX_ENTRIES {
                    let n = if suffix == 0 {
                        base.clone()
                    } else {
                        format!("{base}.{suffix}")
                    };
                    if !next.entries.contains_key(&n) {
                        target = Some(n);
                        break;
                    }
                }
                let target = target.ok_or_else(|| {
                    error(
                        ErrorKind::Limit,
                        "Recovery name bound exceeded; volume unchanged",
                    )
                })?;
                next.entries.insert(target.clone(), ino);
                corrections.push(Correction::Reconnected {
                    inode: ino,
                    name: target,
                });
            }
            let refs = next.references(ino);
            if next.files[&ino].links != refs {
                next.files.get_mut(&ino).unwrap().links = refs;
                corrections.push(Correction::LinkCount {
                    inode: ino,
                    value: refs,
                })
            }
        }
        let blocks = next.expected_blocks();
        let inodes = next.expected_inodes();
        if next.allocated_blocks != blocks {
            next.allocated_blocks = blocks;
            corrections.push(Correction::BlockBitmap)
        }
        if next.allocated_inodes != inodes {
            next.allocated_inodes = inodes;
            corrections.push(Correction::InodeBitmap)
        }
        let free = next.block_count - next.allocated_blocks.len() as u32;
        if next.free_blocks != free {
            next.free_blocks = free;
            corrections.push(Correction::FreeBlocks)
        }
        let free = next.inode_count - next.allocated_inodes.len() as u32;
        if next.free_inodes != free {
            next.free_inodes = free;
            corrections.push(Correction::FreeInodes)
        }
        let after = next.findings()?;
        let status = if after.is_empty() {
            u8::from(!corrections.is_empty())
        } else {
            4 | u8::from(!corrections.is_empty())
        };
        *self = next;
        Ok(Report {
            status,
            before,
            after,
            corrections,
            blocked: None,
        })
    }
}
