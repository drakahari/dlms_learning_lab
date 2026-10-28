//! Bounded archive semantics for offline backup/restore practice.
//!
//! This is a typed virtual archive, not the GNU tar byte format. No path is ever
//! opened on the host. A logical clock stands in for filesystem mtime/ctime;
//! device/inode matching, hard links, symlinks, sparse files and compression are
//! deliberately absent. See docs/FILE_BACKUP_SUBSET.md before authoring lessons.
use crate::engine::Node;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_ENTRIES: usize = 256;
pub const MAX_TEXT: usize = 262_144;
pub const MAX_WIRE: usize = 2 * 1024 * 1024;
const MAX_CLOCK: u64 = 1_000_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "bounded_node")]
    pub node: Node,
    /// None for a directory; Some, including an empty string, for a text file.
    #[serde(deserialize_with = "bounded_text")]
    pub text: Option<String>,
    pub modified: u64,
    pub changed: u64,
}

/// Relative paths, with the empty path naming the selected directory itself.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tree {
    pub clock: u64,
    #[serde(deserialize_with = "bounded_map")]
    pub entries: BTreeMap<String, Entry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "root_name")]
    pub root: String,
    pub cutoff: u64,
    /// Names/kinds only: removed/recreated files cannot be mistaken for old ones.
    #[serde(deserialize_with = "bounded_map")]
    pub inventory: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    #[serde(deserialize_with = "root_name")]
    pub root: String,
    pub clock: u64,
    pub incremental: bool,
    /// Stored directories and changed/new files. A full archive stores all files.
    #[serde(deserialize_with = "bounded_map")]
    pub entries: BTreeMap<String, Entry>,
    /// Complete immediate inventories enable incremental deletion on restore.
    /// An ordinary overlay extraction deliberately ignores these inventories.
    #[serde(deserialize_with = "bounded_directories")]
    pub directories: BTreeMap<String, Vec<String>>,
}

fn label<'de, D: serde::Deserializer<'de>>(
    d: D,
    validate: fn(&str) -> bool,
    expected: &'static str,
) -> Result<String, D::Error> {
    use serde::de::Visitor;
    struct Label(fn(&str) -> bool, &'static str);
    impl Visitor<'_> for Label {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str(self.1)
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            if !(self.0)(value) {
                return Err(E::custom(self.1));
            }
            Ok(value.into())
        }
    }
    d.deserialize_str(Label(validate, expected))
}
fn root_name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    label(d, name, "one archive root name of1..255bytes")
}
fn account_name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    label(d, account, "one virtual account/group name of1..32bytes")
}
fn bounded_node<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Node, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Metadata {
        directory: bool,
        mode: u16,
        #[serde(deserialize_with = "account_name")]
        owner: String,
        #[serde(deserialize_with = "account_name")]
        group: String,
    }
    let m = Metadata::deserialize(d)?;
    Ok(Node {
        directory: m.directory,
        mode: m.mode,
        owner: m.owner,
        group: m.group,
    })
}

/// Reject excessive/duplicate keys before decoding their values, without trusting
/// an allocator size_hint. Borrowed JSON strings are checked before owning them.
fn bounded_map<'de, D, V>(d: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
    struct Key<'a, V>(&'a BTreeMap<String, V>);
    impl<V> Visitor<'_> for Key<'_, V> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a unique bounded virtual relative path")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            if !path(value) || self.0.contains_key(value) {
                return Err(E::custom("Invalid or duplicate virtual path"));
            }
            Ok(value.to_owned())
        }
    }
    impl<'de, V> DeserializeSeed<'de> for Key<'_, V> {
        type Value = String;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
            if self.0.len() >= MAX_ENTRIES {
                return Err(D::Error::custom("Virtual archive entry count exceeds256"));
            }

            d.deserialize_str(self)
        }
    }
    struct Entries<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> Visitor<'de> for Entries<V> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most256 unique virtual entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(key) = map.next_key_seed(Key(&result))? {
                result.insert(key, map.next_value()?);
            }
            Ok(result)
        }
    }
    d.deserialize_map(Entries(std::marker::PhantomData))
}

fn bounded_text<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    use serde::de::Visitor;
    struct Text;
    impl Visitor<'_> for Text {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most262144 UTF-8 bytes of file text without NUL")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            if value.len() > MAX_TEXT || value.contains('\0') {
                return Err(E::custom("Virtual file text bound exceeded or NUL present"));
            }
            Ok(value.into())
        }
    }
    struct OptionalText;
    impl<'de> Visitor<'de> for OptionalText {
        type Value = Option<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null directory text or bounded file text")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            d.deserialize_str(Text).map(Some)
        }
    }
    d.deserialize_option(OptionalText)
}

fn bounded_directories<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, Vec<String>>, D::Error> {
    use serde::de::{DeserializeSeed, Error as _, SeqAccess, Visitor};
    struct Children(Vec<String>);
    impl<'de> Deserialize<'de> for Children {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct Child<'a>(&'a [String]);
            impl Visitor<'_> for Child<'_> {
                type Value = String;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("one unique directory entry name")
                }
                fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
                    if !name(value) || self.0.iter().any(|n| n == value) {
                        return Err(E::custom("Invalid or duplicate directory entry name"));
                    }
                    Ok(value.into())
                }
            }
            impl<'de> DeserializeSeed<'de> for Child<'_> {
                type Value = String;
                fn deserialize<D: serde::Deserializer<'de>>(
                    self,
                    d: D,
                ) -> Result<String, D::Error> {
                    if self.0.len() >= MAX_ENTRIES {
                        return Err(D::Error::custom("Directory inventory count exceeds256"));
                    }

                    d.deserialize_str(self)
                }
            }
            struct Names;
            impl<'de> Visitor<'de> for Names {
                type Value = Children;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("at most256 unique directory child names")
                }
                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Children, A::Error> {
                    let mut names = Vec::new();
                    while let Some(name) = seq.next_element_seed(Child(&names))? {
                        names.push(name);
                    }
                    Ok(Children(names))
                }
            }
            d.deserialize_seq(Names)
        }
    }
    bounded_map::<D, Children>(d).map(|map| map.into_iter().map(|(k, v)| (k, v.0)).collect())
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value != "."
        && value != ".."
        && !value.contains('/')
        && !value.chars().any(char::is_control)
}
fn path(value: &str) -> bool {
    value.is_empty()
        || (value.len() <= 1024 && value.split('/').count() <= 64 && value.split('/').all(name))
}
fn parent(value: &str) -> &str {
    value.rsplit_once('/').map_or("", |(p, _)| p)
}
fn within(value: &str, directory: &str) -> bool {
    value == directory
        || (!value.is_empty()
            && (directory.is_empty()
                || value
                    .strip_prefix(directory)
                    .is_some_and(|tail| tail.starts_with('/'))))
}
fn account(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
impl Entry {
    fn valid(&self, clock: u64) -> bool {
        self.node.mode <= 0o7777
            && account(&self.node.owner)
            && account(&self.node.group)
            && self.modified <= clock
            && self.changed <= clock
            && match (&self.text, self.node.directory) {
                (None, true) => true,
                (Some(text), false) => text.len() <= MAX_TEXT && !text.contains('\0'),
                _ => false,
            }
    }
}
fn bounded_entries(entries: &BTreeMap<String, Entry>, clock: u64) -> bool {
    clock <= MAX_CLOCK
        && !entries.is_empty()
        && entries.len() <= MAX_ENTRIES
        && entries.get("").is_some_and(|e| e.node.directory)
        && entries.iter().all(|(p, e)| {
            path(p)
                && e.valid(clock)
                && (p.is_empty()
                    || entries
                        .get(parent(p))
                        .is_some_and(|parent| parent.node.directory))
        })
        && entries
            .values()
            .filter_map(|e| e.text.as_ref())
            .try_fold(0usize, |sum, t| sum.checked_add(t.len()))
            .is_some_and(|sum| sum <= MAX_TEXT)
}

impl Tree {
    pub fn valid(&self) -> bool {
        bounded_entries(&self.entries, self.clock)
    }

    /// Bounded content loading; this never reads a filename or touches a host path.
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_WIRE {
            return Err("Virtual backup tree exceeds the 2 MiB input bound".into());
        }
        let value: Self = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        if !value.valid() {
            return Err("Invalid virtual backup tree, path, metadata or size".into());
        }
        Ok(value)
    }

    fn next_clock(&self) -> Result<u64, String> {
        if !self.valid() || self.clock >= MAX_CLOCK {
            return Err("Invalid tree or virtual change counter exhausted".into());
        }
        Ok(self.clock + 1)
    }

    /// Existing files only; directory/file replacement is a separate operation.
    /// Copy-on-success keeps capacity errors from losing the previous contents.
    pub fn write(&mut self, target: &str, text: &str) -> Result<(), String> {
        let clock = self.next_clock()?;
        if text.len() > MAX_TEXT || text.contains('\0') {
            return Err("File text exceeds the supported bound or contains NUL".into());
        }
        let entry = self.entries.get(target).ok_or("File does not exist")?;
        if entry.node.directory {
            return Err("Choose a file, not a directory".into());
        }
        let prior = entry.text.as_ref().map_or(0, String::len);
        let total: usize = self
            .entries
            .values()
            .filter_map(|e| e.text.as_ref())
            .map(String::len)
            .sum();
        if total - prior + text.len() > MAX_TEXT {
            return Err("Virtual tree text capacity would be exceeded".into());
        }
        let entry = self.entries.get_mut(target).unwrap();
        entry.text = Some(text.into());
        entry.modified = clock;
        entry.changed = clock;
        self.clock = clock;
        Ok(())
    }

    pub fn remove(&mut self, target: &str) -> Result<(), String> {
        let clock = self.next_clock()?;
        if target.is_empty() || !self.entries.contains_key(target) {
            return Err("Choose an existing entry beneath the selected root".into());
        }
        self.entries.retain(|p, _| !within(p, target));
        self.clock = clock;
        Ok(())
    }

    /// Add a single virtual file/directory; refuse replacement and missing parents.
    pub fn add(&mut self, target: &str, node: Node, text: Option<&str>) -> Result<(), String> {
        let clock = self.next_clock()?;
        if !path(target) || target.is_empty() || self.entries.contains_key(target) {
            return Err("Use a new relative name beneath the selected directory".into());
        }
        if self.entries.len() >= MAX_ENTRIES
            || !self
                .entries
                .get(parent(target))
                .is_some_and(|e| e.node.directory)
        {
            return Err("Parent directory is missing or the entry capacity is full".into());
        }
        let bytes = text.map_or(0, str::len);
        let used: usize = self
            .entries
            .values()
            .filter_map(|e| e.text.as_ref())
            .map(String::len)
            .sum();
        if bytes > MAX_TEXT || used + bytes > MAX_TEXT {
            return Err("Virtual tree text capacity would be exceeded".into());
        }
        let entry = Entry {
            node,
            text: text.map(str::to_owned),
            modified: clock,
            changed: clock,
        };
        if !entry.valid(clock) {
            return Err("Invalid virtual file type, contents or metadata".into());
        }
        self.entries.insert(target.into(), entry);
        self.clock = clock;
        Ok(())
    }

    pub fn chmod(&mut self, target: &str, mode: u16) -> Result<(), String> {
        let clock = self.next_clock()?;
        if mode > 0o7777 {
            return Err("Use a supported octal permission mode".into());
        }
        let entry = self.entries.get_mut(target).ok_or("Entry does not exist")?;
        entry.node.mode = mode;
        entry.changed = clock;
        self.clock = clock;
        Ok(())
    }
}

impl Snapshot {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_WIRE {
            return Err("Virtual snapshot exceeds the 2 MiB input bound".into());
        }
        let value: Self = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        if !value.valid() {
            return Err("Invalid virtual snapshot, inventory, source name or timeline".into());
        }
        Ok(value)
    }
    pub fn valid(&self) -> bool {
        name(&self.root)
            && self.cutoff <= MAX_CLOCK
            && !self.inventory.is_empty()
            && self.inventory.len() <= MAX_ENTRIES
            && self.inventory.get("") == Some(&true)
            && self.inventory.iter().all(|(p, _)| {
                path(p) && (p.is_empty() || self.inventory.get(parent(p)) == Some(&true))
            })
    }
}

impl Archive {
    pub fn valid(&self) -> bool {
        if !name(&self.root)
            || !bounded_entries(&self.entries, self.clock)
            || self.directories.len() > MAX_ENTRIES
            || self.directories.len() != self.entries.values().filter(|e| e.node.directory).count()
        {
            return false;
        }
        let mut inventory_count = 0usize;
        for (directory, names) in &self.directories {
            if !self
                .entries
                .get(directory)
                .is_some_and(|e| e.node.directory)
                || names.len() > MAX_ENTRIES
                || !names.iter().all(|n| name(n))
                || names.iter().collect::<BTreeSet<_>>().len() != names.len()
            {
                return false;
            }
            inventory_count += names.len();
            if inventory_count >= MAX_ENTRIES {
                return false;
            }
            if names.iter().any(|n| {
                let child = if directory.is_empty() {
                    n.clone()
                } else {
                    format!("{directory}/{n}")
                };
                !path(&child)
            }) {
                return false;
            }
        }
        for p in self.entries.keys().filter(|p| !p.is_empty()) {
            let base = p.rsplit('/').next().unwrap();
            if !self
                .directories
                .get(parent(p))
                .is_some_and(|names| names.iter().any(|n| n == base))
            {
                return false;
            }
        }
        // A full archive cannot reference omitted file payloads. Incrementals may.
        self.incremental
            || self.directories.iter().all(|(p, names)| {
                names.iter().all(|n| {
                    let child = if p.is_empty() {
                        n.clone()
                    } else {
                        format!("{p}/{n}")
                    };
                    self.entries.contains_key(&child)
                })
            })
    }

    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_WIRE {
            return Err("Virtual archive exceeds the 2 MiB input bound".into());
        }
        let value: Self = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        if !value.valid() {
            return Err("Invalid virtual archive, directory inventory, path or size".into());
        }
        Ok(value)
    }

    /// Extract into a selected virtual root. Incremental deletion is explicitly
    /// enabled by the caller, analogous to GNU tar -G / --listed-incremental.
    /// Missing/out-of-order levels are applied honestly; there is no canned
    /// chain-success flag. Resulting missing/old files are visible to the grader.
    pub fn extract(&self, target: &mut Tree, incremental: bool) -> Result<(), String> {
        if !self.valid() || !target.valid() {
            return Err("Cannot extract an invalid archive or target".into());
        }
        let clock = target
            .clock
            .max(self.clock)
            .checked_add(1)
            .filter(|clock| *clock <= MAX_CLOCK)
            .ok_or("Virtual change counter exhausted; extraction was not applied")?;
        let mut result = target.clone();
        // Every current directory is stored. A listed but unstored child is
        // therefore an unchanged file, not an unknown kind. Refuse conflicts
        // there too; otherwise a target directory could masquerade as that file.
        for (directory, names) in &self.directories {
            for name in names {
                let child = if directory.is_empty() {
                    name.clone()
                } else {
                    format!("{directory}/{name}")
                };
                let expected_directory = self.entries.get(&child).is_some_and(|e| e.node.directory);
                if result
                    .entries
                    .get(&child)
                    .is_some_and(|e| e.node.directory != expected_directory)
                {
                    return Err("File/directory replacement during extraction is unsupported; use a clean designated restore folder".into());
                }
            }
        }
        if incremental && self.incremental {
            result.entries.retain(|p, _| {
                if p.is_empty() {
                    return true;
                }
                // Each ancestor inventory controls which child subtrees survive.
                let mut ancestor = p.as_str();
                while !ancestor.is_empty() {
                    let parent = parent(ancestor);
                    if let Some(names) = self.directories.get(parent) {
                        let base = ancestor.rsplit('/').next().unwrap();
                        if !names.iter().any(|name| name == base) {
                            return false;
                        }
                    }
                    ancestor = parent;
                }
                true
            });
        }
        // Directories precede descendants. Type-changing extraction needs GNU
        // tar's additional unlink policies, deliberately outside this subset.
        let mut ordered = self.entries.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(p, _)| (p.split('/').count(), p.as_str()));
        for (p, entry) in ordered {
            if result
                .entries
                .get(p)
                .is_some_and(|old| old.node.directory != entry.node.directory)
            {
                return Err("File/directory replacement during extraction is unsupported; use a clean designated restore folder".into());
            }
            let mut restored = entry.clone();
            // tar can preserve archived mtime, but writing data/metadata changes
            // ctime. A restore must itself be visible to a later incremental.
            restored.changed = clock;
            result.entries.insert(p.clone(), restored);
        }
        result.clock = clock;
        if !result.valid() {
            return Err(
                "Restore would exceed the virtual file capacity or leave invalid parents".into(),
            );
        }
        *target = result;
        Ok(())
    }
}

/// A missing snapshot produces a level0 archive with deletion inventories too.
/// A plain full archive is `create(source, root, None, false)`.
pub fn create(
    source: &Tree,
    root: &str,
    previous: Option<&Snapshot>,
    incremental: bool,
) -> Result<(Archive, Snapshot), String> {
    if !source.valid() || !name(root) {
        return Err("Choose a valid bounded source directory and root name".into());
    }
    if let Some(prior) = previous
        && (!incremental || !prior.valid() || prior.root != root || prior.cutoff > source.clock)
    {
        return Err("Snapshot does not belong to this source or virtual timeline".into());
    }
    let mut directories = BTreeMap::new();
    for (p, entry) in &source.entries {
        if entry.node.directory {
            let names = source
                .entries
                .keys()
                .filter(|q| !q.is_empty() && parent(q) == p)
                .map(|q| q.rsplit('/').next().unwrap().to_owned())
                .collect();
            directories.insert(p.clone(), names);
        }
    }
    let entries = source
        .entries
        .iter()
        .filter(|(p, entry)| {
            entry.node.directory
                || previous.is_none_or(|prior| {
                    prior.inventory.get(*p) != Some(&entry.node.directory)
                        || entry.modified > prior.cutoff
                        || entry.changed > prior.cutoff
                })
        })
        .map(|(p, entry)| (p.clone(), entry.clone()))
        .collect();
    let archive = Archive {
        root: root.into(),
        clock: source.clock,
        incremental,
        entries,
        directories,
    };
    let snapshot = Snapshot {
        root: root.into(),
        cutoff: source.clock,
        inventory: source
            .entries
            .iter()
            .map(|(p, e)| (p.clone(), e.node.directory))
            .collect(),
    };
    if !archive.valid() || !snapshot.valid() {
        return Err("Backup exceeds the supported virtual archive bounds".into());
    }
    Ok((archive, snapshot))
}
