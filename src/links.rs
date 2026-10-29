//! Separate inode/path teaching VFS. Never delegates a learner path to the host.
//! Supplied directories are immutable; see LINKS_SUBSET.md for the exact API scope.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
pub const VERSION: u32 = 1;
pub const MAX_ENTRIES: usize = 128;
pub const MAX_DIRECTORIES: usize = 32;
pub const MAX_FILE_BYTES: usize = 8_192;
pub const MAX_DATA_BYTES: usize = 65_536;
pub const MAX_PATH_BYTES: usize = 1_024;
pub const MAX_COMPONENTS: usize = 64;
pub const MAX_SYMLINK_HOPS: usize = 40;
pub const MAX_RESOLVE_STEPS: usize = 4_096;
pub const MAX_PENDING_COMPONENTS: usize = 256;
pub const MAX_STATE_BYTES: usize = 1_048_576;
pub const MAX_OUTPUT_BYTES: usize = 32_768;
pub const HELP: &str = "Typed virtual-filesystem API only, not a host or shell command runner. Models regular hard links, independent copies, absolute/relative/dangling symbolic links, physical path resolution, unlink, no-replace file/symlink moves, cat/readlink/stat observations and writes to existing regular files. Supplied directories, one device and physical cwd are fixed. No permissions, arbitrary byte filenames, mounts, directory mutation, hard links to directories/symlinks, ln directory shorthand, options parser, shell expansion, real device IDs/timestamps or host filesystem access. stat defaults to no final dereference; explicitly requested following corresponds to stat -L. A relative target is relative to the symlink's containing directory, not cwd.";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    NotFound,
    Exists,
    NotDirectory,
    IsDirectory,
    NotRegular,
    NotSymlink,
    Loop,
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
fn valid_input(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= MAX_PATH_BYTES
        && !s.chars().any(char::is_control)
        && s.split('/').filter(|s| !s.is_empty()).count() <= MAX_COMPONENTS
        && s.split('/').all(|s| s.len() <= 255)
}
fn canonical(s: &str) -> bool {
    valid_input(s)
        && s.starts_with('/')
        && (s == "/"
            || (!s.ends_with('/')
                && s[1..]
                    .split('/')
                    .all(|p| !p.is_empty() && p != "." && p != "..")))
}
fn parent(s: &str) -> &str {
    let p = s.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
    if p.is_empty() { "/" } else { p }
}
fn joined(dir: &str, name: &str) -> Result<String> {
    let n = dir.len() + usize::from(dir != "/") + name.len();
    if n > MAX_PATH_BYTES {
        return Err(error(
            ErrorKind::Limit,
            "Expanded virtual path exceeds1024 bytes.",
        ));
    }
    Ok(if dir == "/" {
        format!("/{name}")
    } else {
        format!("{dir}/{name}")
    })
}
fn limited<'de, D: serde::Deserializer<'de>>(
    d: D,
    max: usize,
    path: bool,
) -> std::result::Result<String, D::Error> {
    struct V {
        max: usize,
        path: bool,
    }
    impl serde::de::Visitor<'_> for V {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a string bounded to{} bytes", self.max)
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > self.max || (self.path && !valid_input(s)) {
                return Err(E::custom("Invalid or oversized virtual string"));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(V { max, path })
}
fn data<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    limited(d, MAX_FILE_BYTES, false)
}
fn path<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    limited(d, MAX_PATH_BYTES, true)
}
struct PathKey(String);
impl<'de> Deserialize<'de> for PathKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        path(d).map(Self)
    }
}
fn entries<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, u32>, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = BTreeMap<String, u32>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("up to128 unique virtual paths")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut r = BTreeMap::new();
            while let Some(PathKey(k)) = a.next_key()? {
                if r.len() == MAX_ENTRIES || r.contains_key(&k) {
                    return Err(serde::de::Error::custom(
                        "Virtual entry bound or duplicate path",
                    ));
                }
                r.insert(k, a.next_value()?);
            }
            Ok(r)
        }
    }
    d.deserialize_map(V)
}
fn inodes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<u32, Inode>, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = BTreeMap<u32, Inode>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("up to128 unique virtual inodes")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut r = BTreeMap::new();
            while let Some(k) = a.next_key::<u32>()? {
                if k == 0 || r.len() == MAX_ENTRIES || r.contains_key(&k) {
                    return Err(serde::de::Error::custom(
                        "Virtual inode bound, zero or duplicate ID",
                    ));
                }
                r.insert(k, a.next_value()?);
            }
            Ok(r)
        }
    }
    d.deserialize_map(V)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Inode {
    Directory,
    Regular {
        #[serde(deserialize_with = "data")]
        data: String,
    },
    SymbolicLink {
        #[serde(deserialize_with = "path")]
        target: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    version: u32,
    #[serde(deserialize_with = "path")]
    cwd: String,
    #[serde(deserialize_with = "entries")]
    entries: BTreeMap<String, u32>,
    #[serde(deserialize_with = "inodes")]
    inodes: BTreeMap<u32, Inode>,
    next_inode: u32,
    revision: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Directory,
    Regular,
    SymbolicLink,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    pub physical_path: String,
    /// A synthetic identity within this one virtual device, not a host inode number.
    pub inode: u32,
    pub kind: Kind,
    pub links: usize,
    /// Directory storage size is deliberately unmodeled.
    pub size_bytes: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub physical_path: String,
    pub inode: u32,
}
fn components(s: &str) -> VecDeque<String> {
    let mut r: VecDeque<_> = s
        .split('/')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();
    // A trailing slash requires a directory; don't erase that condition.
    if s.ends_with('/') {
        r.push_back(".".into());
    }
    r
}
impl State {
    pub fn new(directories: &[&str], cwd: &str) -> Result<Self> {
        if directories.is_empty() || directories.len() > MAX_DIRECTORIES {
            return Err(error(
                ErrorKind::Limit,
                "Supply1–32 fixed virtual directories.",
            ));
        }
        if !canonical(cwd) {
            return Err(error(
                ErrorKind::Invalid,
                "Physical cwd must be a bounded canonical absolute path.",
            ));
        }
        let mut s = Self {
            version: VERSION,
            cwd: cwd.into(),
            entries: BTreeMap::new(),
            inodes: BTreeMap::new(),
            next_inode: 1,
            revision: 0,
        };
        for d in directories {
            if !canonical(d) || s.entries.contains_key(*d) {
                return Err(error(
                    ErrorKind::Invalid,
                    "Directories must have distinct canonical absolute paths.",
                ));
            }
            let id = s.allocate(Inode::Directory)?;
            s.entries.insert((*d).into(), id);
        }
        s.validate()?;
        Ok(s)
    }
    pub fn cwd(&self) -> Result<&str> {
        self.validate()?;
        Ok(&self.cwd)
    }
    pub fn revision(&self) -> Result<u64> {
        self.validate()?;
        Ok(self.revision)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(error(
                ErrorKind::Invalid,
                "Unsupported virtual-links state version.",
            ));
        }
        if self.entries.is_empty()
            || self.entries.len() > MAX_ENTRIES
            || self.inodes.is_empty()
            || self.inodes.len() > MAX_ENTRIES
        {
            return Err(error(
                ErrorKind::Limit,
                "Virtual entry/inode limit exceeded.",
            ));
        }
        if self.next_inode == 0
            || self
                .inodes
                .keys()
                .any(|id| *id == 0 || *id >= self.next_inode)
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid monotonic virtual inode allocator.",
            ));
        }
        let node = |p: &str| self.entries.get(p).and_then(|id| self.inodes.get(id));
        if !matches!(node("/"), Some(Inode::Directory))
            || !canonical(&self.cwd)
            || !matches!(node(&self.cwd), Some(Inode::Directory))
        {
            return Err(error(
                ErrorKind::Invalid,
                "Virtual root and fixed physical cwd must be existing directories.",
            ));
        }
        let mut refs = BTreeMap::<u32, usize>::new();
        for (p, id) in &self.entries {
            if !canonical(p)
                || !self.inodes.contains_key(id)
                || (p != "/" && !matches!(node(parent(p)), Some(Inode::Directory)))
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Every canonical entry needs an existing inode and a physical directory parent.",
                ));
            }
            *refs.entry(*id).or_default() += 1;
        }
        let mut dirs = 0;
        let mut bytes = 0usize;
        for (id, inode) in &self.inodes {
            let count = refs.get(id).copied().unwrap_or(0);
            if count == 0 {
                return Err(error(ErrorKind::Invalid, "Orphan virtual inode refused."));
            }
            match inode {
                Inode::Directory => {
                    dirs += 1;
                    if count != 1 {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Directory aliases/hard links are not supported.",
                        ));
                    }
                }
                Inode::Regular { data } => {
                    if data.len() > MAX_FILE_BYTES {
                        return Err(error(
                            ErrorKind::Limit,
                            "Regular file exceeds8192 UTF-8 bytes.",
                        ));
                    }
                    bytes = bytes
                        .checked_add(data.len())
                        .ok_or_else(|| error(ErrorKind::Limit, "Virtual data overflow."))?;
                }
                Inode::SymbolicLink { target } => {
                    if count != 1 {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Hard links to symlinks are not modeled; no automatic -P/-L approximation.",
                        ));
                    }
                    if !valid_input(target) {
                        return Err(error(
                            ErrorKind::Invalid,
                            "Invalid or oversized symbolic-link text.",
                        ));
                    }
                }
            }
        }
        if dirs > MAX_DIRECTORIES || bytes > MAX_DATA_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Directory or aggregate regular-data limit exceeded.",
            ));
        }
        Ok(())
    }
    fn resolve_raw(&self, path: &str, follow_final: bool) -> Result<Resolved> {
        if !valid_input(path) {
            return Err(error(
                ErrorKind::Invalid,
                "Paths must be nonempty UTF-8 text, at most1024 bytes/64 components, without control characters.",
            ));
        }
        let mut current = if path.starts_with('/') {
            "/".into()
        } else {
            self.cwd.clone()
        };
        let mut pending = components(path);
        let mut hops = 0;
        let mut steps = 0;
        while let Some(part) = pending.pop_front() {
            steps += 1;
            if steps > MAX_RESOLVE_STEPS {
                return Err(error(
                    ErrorKind::Limit,
                    "Virtual path-resolution work bound exceeded.",
                ));
            }
            if part == "." {
                continue;
            }
            if part == ".." {
                current = parent(&current).into();
                continue;
            }
            let candidate = joined(&current, &part)?;
            let id = *self.entries.get(&candidate).ok_or_else(|| {
                error(
                    ErrorKind::NotFound,
                    format!("No virtual entry at {candidate}."),
                )
            })?;
            let inode = &self.inodes[&id];
            if let Inode::SymbolicLink { target } = inode
                && (follow_final || !pending.is_empty())
            {
                hops += 1;
                if hops > MAX_SYMLINK_HOPS {
                    return Err(error(
                        ErrorKind::Loop,
                        "ELOOP: more than40 symbolic-link traversals in one virtual path lookup.",
                    ));
                }
                let mut prefix = components(target);
                if prefix.len() + pending.len() > MAX_PENDING_COMPONENTS {
                    return Err(error(
                        ErrorKind::Limit,
                        "Expanded virtual path queue bound exceeded.",
                    ));
                }
                if target.starts_with('/') {
                    current = "/".into();
                }
                prefix.append(&mut pending);
                pending = prefix;
                continue;
            }
            if !pending.is_empty() && !matches!(inode, Inode::Directory) {
                return Err(error(
                    ErrorKind::NotDirectory,
                    format!("{candidate} is not a virtual directory."),
                ));
            }
            current = candidate;
        }
        let inode = *self
            .entries
            .get(&current)
            .ok_or_else(|| error(ErrorKind::NotFound, "Resolved virtual entry is absent."))?;
        Ok(Resolved {
            physical_path: current,
            inode,
        })
    }
    pub fn resolve(&self, path: &str, follow_final: bool) -> Result<Resolved> {
        self.validate()?;
        self.resolve_raw(path, follow_final)
    }
    fn destination(&self, path: &str) -> Result<String> {
        if !valid_input(path) {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid virtual destination path.",
            ));
        }
        if path.ends_with('/') {
            return Err(error(
                ErrorKind::Unsupported,
                "Modifying paths with trailing slashes/directory shorthand are not supported; give an explicit entry name.",
            ));
        }
        let (dir, name) = path
            .rsplit_once('/')
            .map(|(a, b)| (if a.is_empty() { "/" } else { a }, b))
            .unwrap_or((".", path));
        if name == "." || name == ".." {
            return Err(error(
                ErrorKind::Unsupported,
                "Directory-name modification is not supported.",
            ));
        }
        let parent = self.resolve_raw(dir, true)?;
        if !matches!(self.inodes[&parent.inode], Inode::Directory) {
            return Err(error(
                ErrorKind::NotDirectory,
                "Virtual destination parent is not a directory.",
            ));
        }
        joined(&parent.physical_path, name)
    }
    fn vacant(&self, path: &str, needs_slot: bool) -> Result<()> {
        if let Some(id) = self.entries.get(path) {
            return Err(if matches!(self.inodes.get(id), Some(Inode::Directory)) {
                error(
                    ErrorKind::Unsupported,
                    "Directory-destination shorthand is unsupported; supply the complete new entry path.",
                )
            } else {
                error(
                    ErrorKind::Exists,
                    "Virtual destination already exists; replacement is not supported.",
                )
            });
        }
        if needs_slot && self.entries.len() >= MAX_ENTRIES {
            return Err(error(ErrorKind::Limit, "Virtual entry limit reached."));
        }
        Ok(())
    }
    fn allocate(&mut self, inode: Inode) -> Result<u32> {
        if self.inodes.len() >= MAX_ENTRIES {
            return Err(error(ErrorKind::Limit, "Virtual inode limit reached."));
        }
        let id = self.next_inode;
        self.next_inode = id
            .checked_add(1)
            .ok_or_else(|| error(ErrorKind::Limit, "Virtual inode counter exhausted."))?;
        self.inodes.insert(id, inode);
        Ok(id)
    }
    fn change(&mut self, f: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        self.validate()?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| error(ErrorKind::Limit, "Virtual event counter exhausted."))?;
        let mut next = self.clone();
        f(&mut next)?;
        next.revision = revision;
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn create_file(&mut self, destination: &str, text: &str) -> Result<()> {
        if text.len() > MAX_FILE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Regular file exceeds8192 UTF-8 bytes.",
            ));
        }
        self.change(|s| {
            let p = s.destination(destination)?;
            s.vacant(&p, true)?;
            let id = s.allocate(Inode::Regular { data: text.into() })?;
            s.entries.insert(p, id);
            Ok(())
        })
    }
    /// Editor save to an existing regular inode; a dangling target is an error, not auto-created.
    pub fn write_file(&mut self, path: &str, text: &str) -> Result<()> {
        if text.len() > MAX_FILE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Regular file exceeds8192 UTF-8 bytes.",
            ));
        }
        self.change(|s| {
            let r = s.resolve_raw(path, true)?;
            match s.inodes.get_mut(&r.inode).unwrap() {
                Inode::Regular { data } => {
                    *data = text.into();
                    Ok(())
                }
                Inode::Directory => Err(error(ErrorKind::IsDirectory, "Cannot edit a directory.")),
                _ => Err(error(ErrorKind::NotRegular, "Not a regular virtual file.")),
            }
        })
    }
    pub fn hard_link(&mut self, source: &str, destination: &str) -> Result<()> {
        self.change(|s|{
            let r=s.resolve_raw(source,false)?;
            match s.inodes[&r.inode] { Inode::Directory=>return Err(error(ErrorKind::Unsupported,"Hard links to directories are unsupported.")),Inode::SymbolicLink{..}=>return Err(error(ErrorKind::Unsupported,"Hard links to symlinks are unsupported; GNU -P/-L differ and this subset will not silently follow either.")),Inode::Regular{..}=>{} }
            let p=s.destination(destination)?;s.vacant(&p, true)?;s.entries.insert(p,r.inode);Ok(())
        })
    }
    pub fn symbolic_link(&mut self, target: &str, destination: &str) -> Result<()> {
        if !valid_input(target) {
            return Err(error(
                ErrorKind::Invalid,
                "Symbolic-link targets must be nonempty bounded UTF-8 path text.",
            ));
        }
        self.change(|s| {
            let p = s.destination(destination)?;
            s.vacant(&p, true)?;
            let id = s.allocate(Inode::SymbolicLink {
                target: target.into(),
            })?;
            s.entries.insert(p, id);
            Ok(())
        })
    }
    pub fn copy_file(&mut self, source: &str, destination: &str) -> Result<()> {
        self.change(|s| {
            let r = s.resolve_raw(source, true)?;
            let data = match &s.inodes[&r.inode] {
                Inode::Regular { data } => data.clone(),
                _ => {
                    return Err(error(
                        ErrorKind::NotRegular,
                        "Copy source must resolve to a regular virtual file.",
                    ));
                }
            };
            let p = s.destination(destination)?;
            s.vacant(&p, true)?;
            let id = s.allocate(Inode::Regular { data })?;
            s.entries.insert(p, id);
            Ok(())
        })
    }
    pub fn unlink(&mut self, path: &str) -> Result<()> {
        self.change(|s| {
            let p = s.destination(path)?;
            let id = *s
                .entries
                .get(&p)
                .ok_or_else(|| error(ErrorKind::NotFound, "Virtual entry is absent."))?;
            if matches!(s.inodes[&id], Inode::Directory) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Directory removal is unsupported.",
                ));
            }
            s.entries.remove(&p);
            if !s.entries.values().any(|v| *v == id) {
                s.inodes.remove(&id);
            }
            Ok(())
        })
    }
    /// Move only this file/symlink entry, preserving identity/text; never replace an existing entry.
    pub fn move_entry(&mut self, source: &str, destination: &str) -> Result<()> {
        self.change(|s| {
            let from = s.destination(source)?;
            let id = *s
                .entries
                .get(&from)
                .ok_or_else(|| error(ErrorKind::NotFound, "Move source is absent."))?;
            if matches!(s.inodes[&id], Inode::Directory) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Directory moves are unsupported.",
                ));
            }
            let to = s.destination(destination)?;
            s.vacant(&to, false)?;
            s.entries.remove(&from);
            s.entries.insert(to, id);
            Ok(())
        })
    }
    pub fn read_file(&self, path: &str) -> Result<String> {
        self.validate()?;
        let r = self.resolve_raw(path, true)?;
        match &self.inodes[&r.inode] {
            Inode::Regular { data } => Ok(data.clone()),
            Inode::Directory => Err(error(
                ErrorKind::IsDirectory,
                "Cannot read a directory as regular text.",
            )),
            _ => Err(error(ErrorKind::NotRegular, "Not a regular virtual file.")),
        }
    }
    pub fn read_link(&self, path: &str) -> Result<String> {
        self.validate()?;
        let r = self.resolve_raw(path, false)?;
        match &self.inodes[&r.inode] {
            Inode::SymbolicLink { target } => Ok(target.clone()),
            _ => Err(error(
                ErrorKind::NotSymlink,
                "readlink requires a symbolic link; it does not canonicalize ordinary paths.",
            )),
        }
    }
    /// Default stat observation: do not follow a final symlink (except a directory-required trailing slash).
    pub fn stat(&self, path: &str) -> Result<Stat> {
        self.stat_with_follow(path, false)
    }
    /// Explicit stat -L-style following; metadata is still virtual and partial.
    pub fn stat_follow(&self, path: &str) -> Result<Stat> {
        self.stat_with_follow(path, true)
    }
    fn stat_with_follow(&self, path: &str, follow_final: bool) -> Result<Stat> {
        self.validate()?;
        let r = self.resolve_raw(path, follow_final)?;
        let (kind, size_bytes, links) = match &self.inodes[&r.inode] {
            Inode::Directory => (
                Kind::Directory,
                None,
                2 + self
                    .entries
                    .iter()
                    .filter(|(p, id)| {
                        p.as_str() != "/"
                            && parent(p) == r.physical_path
                            && matches!(self.inodes.get(*id), Some(Inode::Directory))
                    })
                    .count(),
            ),
            Inode::Regular { data } => (
                Kind::Regular,
                Some(data.len()),
                self.entries.values().filter(|id| **id == r.inode).count(),
            ),
            Inode::SymbolicLink { target } => (Kind::SymbolicLink, Some(target.len()), 1),
        };
        Ok(Stat {
            physical_path: r.physical_path,
            inode: r.inode,
            kind,
            links,
            size_bytes,
        })
    }
    pub fn directory_entries(&self, path: &str) -> Result<Vec<String>> {
        self.validate()?;
        let r = self.resolve_raw(path, true)?;
        if !matches!(self.inodes[&r.inode], Inode::Directory) {
            return Err(error(ErrorKind::NotDirectory, "Not a virtual directory."));
        }
        let mut result = Vec::new();
        let mut bytes = 0;
        for p in self
            .entries
            .keys()
            .filter(|p| p.as_str() != "/" && parent(p) == r.physical_path)
        {
            let name = p.rsplit('/').next().unwrap();
            bytes += name.len() + 1;
            if bytes > MAX_OUTPUT_BYTES {
                return Err(error(
                    ErrorKind::Limit,
                    "Virtual listing exceeds output limit.",
                ));
            }
            result.push(name.into());
        }
        Ok(result)
    }
    /// Only fixed directory identity/cwd; not permission to alter task-protected files.
    pub fn environment_matches(&self, source: &Self) -> Result<bool> {
        self.validate()?;
        source.validate()?;
        let dirs = |s: &Self| {
            s.entries
                .iter()
                .filter(|(_, id)| matches!(s.inodes.get(*id), Some(Inode::Directory)))
                .map(|(p, id)| (p.clone(), *id))
                .collect::<BTreeMap<_, _>>()
        };
        Ok(self.cwd == source.cwd && dirs(self) == dirs(source))
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(ErrorKind::Limit, "Virtual links JSON exceeds1MiB."));
        }
        let s: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                ErrorKind::Invalid,
                "Malformed bounded virtual-links JSON (including duplicate/unknown fields).",
            )
        })?;
        s.validate()?;
        Ok(s)
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let raw = serde_json::to_vec(self)
            .map_err(|_| error(ErrorKind::Invalid, "Cannot encode virtual-links state."))?;
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Virtual links encoding exceeds1MiB.",
            ));
        }
        Ok(raw)
    }
}
