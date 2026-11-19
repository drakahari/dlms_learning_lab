//! Bounded sparse before-images for fixed-length virtual records.
//! Prerequisite model only: not yet admitted by LVM state, UI or authored lessons.
//! No filesystem, device, subprocess or host I/O.
use super::*;

pub const MAX_SNAPSHOTS: usize = 4;
pub const MAX_IMAGES: usize = 128;
pub const MAX_ORIGIN_BYTES: u64 = 16 * 1024 * MIB;
pub const MAX_BANK_BYTES: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub offset: u64,
    pub length: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    pub offset: u32,
    #[serde(deserialize_with = "text::<_,65536>")]
    data: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Image {
    #[serde(deserialize_with = "map::<_,Fragment,32>")]
    fragments: BTreeMap<String, Fragment>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    id: u64,
    capacity_bytes: u64,
    chunk_bytes: u32,
    valid: bool,
    // String keys use the existing streaming bounded map deserializer.
    // Canonical decimal chunk indexes are validated before any use.
    #[serde(deserialize_with = "map::<_,Image,128>")]
    images: BTreeMap<String, Image>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    next_snapshot_id: u64,
    byte_len: u64,
    #[serde(deserialize_with = "map::<_,Slot,32>")]
    layout: BTreeMap<String, Slot>,
    #[serde(deserialize_with = "map::<_,File,32>")]
    current: BTreeMap<String, File>,
    #[serde(deserialize_with = "map::<_,Snapshot,4>")]
    snapshots: BTreeMap<String, Snapshot>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub capacity_bytes: u64,
    pub chunk_bytes: u32,
    pub metadata_chunks: usize,
    pub usable_chunks: u64,
    pub copied_chunks: usize,
    pub valid: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteResult {
    /// Newly invalidated snapshots. The origin write still succeeds.
    pub invalidated: Vec<String>,
}
fn ascii_record(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= MAX_FILE_BYTES
        && s.bytes()
            .all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\n'))
}
fn chunk_size(n: u32) -> bool {
    (4096..=512 * 1024).contains(&n) && n.is_power_of_two()
}
fn capacity(bytes: u64, chunk: u32) -> Result<u64> {
    if !chunk_size(chunk) || bytes > MAX_ORIGIN_BYTES || !bytes.is_multiple_of(512) {
        return Err(invalid(
            "Use sector-aligned COW capacity and power-of-two chunks from4 through512KiB",
        ));
    }
    // Explicit teaching accounting, NOT actual LVM metadata overhead.
    (bytes / u64::from(chunk))
        .checked_sub(1)
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            invalid("This model reserves one chunk for metadata and needs at least one data chunk")
        })
}
fn end(slot: &Slot) -> Option<u64> {
    slot.offset.checked_add(u64::from(slot.length))
}
fn intersects(slot: &Slot, start: u64, limit: u64) -> bool {
    slot.offset < limit && end(slot).is_some_and(|e| e > start)
}
impl Bank {
    pub fn new(byte_len: u64, records: BTreeMap<String, (Slot, String)>) -> Result<Self> {
        // Check count before splitting/copying owned caller data.
        if records.len() > MAX_FILES {
            return Err(limit("Too many virtual records"));
        }
        let mut layout = BTreeMap::new();
        let mut current = BTreeMap::new();
        for (id, (slot, contents)) in records {
            current.insert(id.clone(), File { contents });
            layout.insert(id, slot);
        }
        let bank = Self {
            next_snapshot_id: 1,
            byte_len,
            layout,
            current,
            snapshots: BTreeMap::new(),
        };
        bank.validate()?;
        Ok(bank)
    }
    pub fn byte_len(&self) -> u64 {
        self.byte_len
    }
    pub fn layout(&self) -> &BTreeMap<String, Slot> {
        &self.layout
    }
    pub fn current(&self, record: &str) -> Result<&str> {
        self.current
            .get(record)
            .map(|r| r.contents.as_str())
            .ok_or_else(|| error(ErrorKind::Unknown, "No supplied virtual record"))
    }
    pub fn status(&self, id: &str) -> Result<Status> {
        let s = self
            .snapshots
            .get(id)
            .ok_or_else(|| error(ErrorKind::Unknown, "No such virtual snapshot"))?;
        Ok(Status {
            capacity_bytes: s.capacity_bytes,
            chunk_bytes: s.chunk_bytes,
            metadata_chunks: 1,
            usable_chunks: capacity(s.capacity_bytes, s.chunk_bytes)?,
            copied_chunks: s.images.len(),
            valid: s.valid,
        })
    }
    pub fn snapshot_names(&self) -> impl Iterator<Item = &str> {
        self.snapshots.keys().map(String::as_str)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_BANK_BYTES {
            return Err(limit("Sparse COW model exceeds1MiB before decoding"));
        }
        let bank: Self = serde_json::from_slice(raw)
            .map_err(|_| invalid("Malformed bounded sparse COW model"))?;
        bank.validate()?;
        Ok(bank)
    }
    pub fn validate(&self) -> Result<()> {
        if self.next_snapshot_id == 0
            || self.byte_len == 0
            || self.byte_len > MAX_ORIGIN_BYTES
            || !self.byte_len.is_multiple_of(512)
            || self.layout.is_empty()
            || self.layout.len() > MAX_FILES
            || self.layout.keys().ne(self.current.keys())
            || self.snapshots.len() > MAX_SNAPSHOTS
        {
            return Err(invalid(
                "Invalid virtual origin geometry or record/snapshot inventory",
            ));
        }
        let mut total = 0usize;
        for (id, slot) in &self.layout {
            let text = &self.current[id].contents;
            if !name(id)
                || slot.length == 0
                || slot.length as usize != text.len()
                || !ascii_record(text)
                || end(slot).is_none_or(|e| e > self.byte_len)
            {
                return Err(invalid(
                    "Virtual records need distinct simple names, fixed-length printable ASCII content and ranges inside the origin",
                ));
            }
            total = total
                .checked_add(text.len())
                .ok_or_else(|| limit("Record byte overflow"))?;
            if total > MAX_TOTAL_FILE_BYTES {
                return Err(limit("Virtual origin record content exceeds128KiB"));
            }
            for (other, range) in &self.layout {
                if other < id && intersects(range, slot.offset, end(slot).unwrap()) {
                    return Err(invalid("Virtual record ranges overlap"));
                }
            }
        }
        let mut identities = BTreeSet::new();
        for (id, s) in &self.snapshots {
            if s.id == 0
                || s.id >= self.next_snapshot_id
                || !identities.insert(s.id)
                || !provision::valid_volume_name(id)
                || s.images.len() > MAX_IMAGES
                || s.images.len() as u64 > capacity(s.capacity_bytes, s.chunk_bytes)?
            {
                return Err(invalid(
                    "Invalid snapshot name, capacity or exception count",
                ));
            }
            let chunk = u64::from(s.chunk_bytes);
            for (key, image) in &s.images {
                let index = key
                    .parse::<u64>()
                    .ok()
                    .filter(|n| n.to_string() == *key)
                    .ok_or_else(|| {
                        invalid("Snapshot chunk indexes must be canonical decimal values")
                    })?;
                let start = index
                    .checked_mul(chunk)
                    .filter(|n| *n < self.byte_len)
                    .ok_or_else(|| invalid("Snapshot chunk is outside the origin"))?;
                let limit = start
                    .checked_add(chunk)
                    .ok_or_else(|| invalid("Chunk boundary overflow"))?;
                let expected = self
                    .layout
                    .iter()
                    .filter(|(_, slot)| intersects(slot, start, limit));
                let mut count = 0;
                for (record, slot) in expected {
                    count += 1;
                    let f = image.fragments.get(record).ok_or_else(|| {
                        invalid("A copied chunk must contain every intersecting record fragment")
                    })?;
                    let from = slot.offset.max(start);
                    let to = end(slot).unwrap().min(limit);
                    if u64::from(f.offset) != from - start
                        || f.data.len() as u64 != to - from
                        || !ascii_record(&f.data)
                    {
                        return Err(invalid(
                            "Snapshot fragment geometry or ASCII payload is invalid",
                        ));
                    }
                }
                if count == 0 || image.fragments.len() != count {
                    return Err(invalid("Foreign or empty sparse chunk image"));
                }
            }
        }
        wire_size(self)?;
        if serde_json::to_vec(self)
            .map_err(|_| invalid("Invalid COW serialization"))?
            .len()
            > MAX_BANK_BYTES
        {
            return Err(limit("Sparse COW model exceeds1MiB"));
        }
        Ok(())
    }
    fn publish(&mut self, next: Self) -> Result<()> {
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn create(&mut self, id: &str, bytes: u64, chunk: u32) -> Result<()> {
        self.validate()?;
        capacity(bytes, chunk)?;
        if !provision::valid_volume_name(id) {
            return Err(invalid("Use a supported simple snapshot name"));
        }
        if self.snapshots.contains_key(id) {
            return Err(error(ErrorKind::Occupied, "Snapshot name already exists"));
        }
        if self.snapshots.len() >= MAX_SNAPSHOTS {
            return Err(limit("At most four virtual snapshots per origin"));
        }
        let mut next = self.clone();
        let identity = next.next_snapshot_id;
        next.next_snapshot_id = identity
            .checked_add(1)
            .ok_or_else(|| limit("Snapshot identity counter exhausted"))?;
        next.snapshots.insert(
            id.into(),
            Snapshot {
                id: identity,
                capacity_bytes: bytes,
                chunk_bytes: chunk,
                valid: true,
                images: BTreeMap::new(),
            },
        );
        self.publish(next)
    }
    pub fn extend(&mut self, id: &str, bytes: u64) -> Result<()> {
        self.validate()?;
        let s = self
            .snapshots
            .get(id)
            .ok_or_else(|| error(ErrorKind::Unknown, "No such virtual snapshot"))?;
        capacity(bytes, s.chunk_bytes)?;
        if !s.valid {
            return Err(invalid(
                "An invalid snapshot cannot be revived by adding space",
            ));
        }
        if bytes <= s.capacity_bytes {
            return Err(invalid("Only a larger COW store is supported"));
        }
        let mut next = self.clone();
        next.snapshots.get_mut(id).unwrap().capacity_bytes = bytes;
        self.publish(next)
    }
    pub fn remove(&mut self, id: &str) -> Result<()> {
        self.validate()?;
        if !self.snapshots.contains_key(id) {
            return Err(error(ErrorKind::Unknown, "No such virtual snapshot"));
        }
        let mut next = self.clone();
        next.snapshots.remove(id);
        self.publish(next)
    }
    fn capture(&self, index: u64, chunk: u32) -> Image {
        let start = index * u64::from(chunk);
        let limit = start + u64::from(chunk);
        let fragments = self
            .layout
            .iter()
            .filter(|(_, s)| intersects(s, start, limit))
            .map(|(id, slot)| {
                let from = slot.offset.max(start);
                let to = end(slot).unwrap().min(limit);
                (
                    id.clone(),
                    Fragment {
                        offset: (from - start) as u32,
                        data: self.current[id].contents
                            [(from - slot.offset) as usize..(to - slot.offset) as usize]
                            .into(),
                    },
                )
            })
            .collect();
        Image { fragments }
    }
    /// Structural value admission; no write, exception capture or data mutation.
    pub fn validate_record_value(&self, record: &str, data: &str) -> Result<()> {
        let slot = self
            .layout
            .get(record)
            .ok_or_else(|| error(ErrorKind::Unknown, "No supplied virtual record"))?;
        if data.len() != slot.length as usize || !ascii_record(data) {
            return Err(invalid(
                "This virtual record requires exactly its supplied byte count as printable ASCII text (spaces, tabs and newlines allowed); variable-length or new records are unsupported",
            ));
        }
        Ok(())
    }
    /// An operation sequence cannot rewind IDs or reuse a retired view's identity.
    pub fn continues(&self, before: &Self) -> bool {
        self.byte_len == before.byte_len
            && self.layout == before.layout
            && self.next_snapshot_id >= before.next_snapshot_id
            && self.snapshots.iter().all(|(name, s)| {
                before.snapshots.contains_key(name) || s.id >= before.next_snapshot_id
            })
    }
    /// Preserve the same readable point in time, not merely its current name.
    pub fn keeps_snapshot(&self, before: &Self, id: &str, allow_growth: bool) -> bool {
        if !self.continues(before)
            || self.byte_len != before.byte_len
            || self.layout != before.layout
            || self.validate().is_err()
            || before.validate().is_err()
        {
            return false;
        }
        let (Some(old), Some(now)) = (before.snapshots.get(id), self.snapshots.get(id)) else {
            return false;
        };
        if !old.valid {
            return old == now;
        }
        if now.id != old.id
            || !now.valid
            || old.chunk_bytes != now.chunk_bytes
            || if allow_growth {
                now.capacity_bytes < old.capacity_bytes
            } else {
                now.capacity_bytes != old.capacity_bytes
            }
            || old
                .images
                .iter()
                .any(|(key, image)| now.images.get(key) != Some(image))
        {
            return false;
        }
        before.layout.keys().all(
            |record| match (self.read(id, record), before.read(id, record)) {
                (Ok(a), Ok(b)) => a == b,
                _ => false,
            },
        )
    }
    /// Explicit simulator stimulus: overwrite one supplied fixed-size record.
    /// It touches the complete record range even if some bytes are unchanged.
    pub fn write(&mut self, record: &str, data: &str) -> Result<WriteResult> {
        self.validate()?;
        self.validate_record_value(record, data)?;
        let slot = &self.layout[record];
        let mut next = self.clone();
        let mut invalidated = Vec::new();
        for (id, snap) in &mut next.snapshots {
            if !snap.valid {
                continue;
            }
            let chunk = u64::from(snap.chunk_bytes);
            let indexes = (slot.offset / chunk)..=((end(slot).unwrap() - 1) / chunk);
            let missing = indexes
                .filter(|n| !snap.images.contains_key(&n.to_string()))
                .collect::<Vec<_>>();
            if (snap.images.len() + missing.len()) as u64
                > capacity(snap.capacity_bytes, snap.chunk_bytes)?
            {
                snap.valid = false;
                invalidated.push(id.clone());
                continue;
            }
            for index in missing {
                snap.images
                    .insert(index.to_string(), self.capture(index, snap.chunk_bytes));
            }
        }
        next.current.get_mut(record).unwrap().contents = data.into();
        self.publish(next)?;
        Ok(WriteResult { invalidated })
    }
    pub fn read(&self, snapshot: &str, record: &str) -> Result<String> {
        self.validate()?;
        let s = self
            .snapshots
            .get(snapshot)
            .ok_or_else(|| error(ErrorKind::Unknown, "No such virtual snapshot"))?;
        if !s.valid {
            return Err(invalid(
                "Snapshot is invalid; its point-in-time data is unavailable",
            ));
        }
        let slot = self
            .layout
            .get(record)
            .ok_or_else(|| error(ErrorKind::Unknown, "No supplied virtual record"))?;
        let chunk = u64::from(s.chunk_bytes);
        let last = end(slot).unwrap();
        let mut output = String::with_capacity(slot.length as usize);
        for index in slot.offset / chunk..=(last - 1) / chunk {
            let start = index * chunk;
            let from = slot.offset.max(start);
            let to = last.min(start + chunk);
            if let Some(image) = s.images.get(&index.to_string()) {
                output.push_str(&image.fragments[record].data);
            } else {
                output.push_str(
                    &self.current[record].contents
                        [(from - slot.offset) as usize..(to - slot.offset) as usize],
                );
            }
        }
        Ok(output)
    }
}
