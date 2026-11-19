//! Bounded metadata-only shared-library resolution; no real ELF loading.
//! No ELF parsing/execution, filesystem, subprocess, environment or network API.
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const VERSION: u32 = 1;
pub const MAX_STATE: usize = 65_536;
pub const MAX_OBJECTS: usize = 24;
pub const MAX_PROGRAMS: usize = 8;
pub const MAX_DIRECTORIES: usize = 16;
pub const MAX_DEPENDENCIES: usize = 8;
pub const MAX_DEPTH: usize = 16;
pub const MAX_VISITS: usize = 128;
pub const MAX_CONFIG: usize = 4096;
pub const MAX_PATH_LIST: usize = 1024;
pub const MAX_REBUILDS: u32 = 64;
pub const MAX_OUTPUT: usize = 32_768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    NotFound,
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

fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    struct Bounded<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Bounded<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > N {
                Err(E::custom("Library model text limit"))
            } else {
                Ok(s.to_owned())
            }
        }
    }
    d.deserialize_str(Bounded::<N>)
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} items")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Vec<T>, A::Error> {
            let mut out = Vec::new();
            while out.len() < N {
                match a.next_element()? {
                    Some(x) => out.push(x),
                    None => return Ok(out),
                }
            }
            // Refuse before decoding or allocating an excess element.
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> std::result::Result<Self, D::Error> {
                    Err(serde::de::Error::custom("Library model item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
#[derive(Deserialize)]
struct Name(#[serde(deserialize_with = "text::<_,128>")] String);
fn names<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    list::<D, Name, N>(d).map(|v| v.into_iter().map(|n| n.0).collect())
}
#[derive(Deserialize)]
struct Path(#[serde(deserialize_with = "text::<_,256>")] String);
fn paths<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    list::<D, Path, N>(d).map(|v| v.into_iter().map(|n| n.0).collect())
}
fn optional_path_list<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Value(#[serde(deserialize_with = "text::<_,1024>")] String);
    Option::<Value>::deserialize(d).map(|v| v.map(|v| v.0))
}
fn fits<T: Serialize>(value: &T) -> bool {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= MAX_STATE)
                .ok_or_else(|| std::io::Error::other("Library state byte limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0), value).is_ok()
}
fn canonical(s: &str) -> bool {
    s.starts_with('/')
        && s.len() <= 256
        && s.len() > 1
        && !s.ends_with('/')
        && s[1..]
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._+-".contains(&b))
}
fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |p| p.0)
}
fn basename(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |p| p.1)
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.starts_with("lib")
        && s.contains(".so")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
}
fn identity(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn unique<T: Ord>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    items.into_iter().all(|v| seen.insert(v))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Elf32,
    Elf64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Machine {
    X86,
    X86_64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ByteOrder {
    Little,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Abi {
    pub class: Class,
    pub machine: Machine,
    pub byte_order: ByteOrder,
}
impl Abi {
    fn validate(&self) -> Result<()> {
        if matches!(
            (self.class, self.machine),
            (Class::Elf32, Machine::X86) | (Class::Elf64, Machine::X86_64)
        ) {
            Ok(())
        } else {
            Err(error(
                ErrorKind::Unsupported,
                "Only supplied little-endian x86 ELF32 and x86-64 ELF64 metadata are supported",
            ))
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    #[serde(deserialize_with = "text::<_,64>")]
    pub identity: String,
    #[serde(deserialize_with = "text::<_,256>")]
    pub path: String,
    pub abi: Abi,
    #[serde(deserialize_with = "text::<_,128>")]
    pub soname: String,
    #[serde(deserialize_with = "names::<_,8>")]
    pub needed: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    #[serde(deserialize_with = "text::<_,256>")]
    pub path: String,
    pub abi: Abi,
    #[serde(deserialize_with = "names::<_,8>")]
    pub needed: Vec<String>,
}
/// A supplied already-resolved symbolic name, not a learner filesystem operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    #[serde(deserialize_with = "text::<_,256>")]
    pub path: String,
    #[serde(deserialize_with = "text::<_,256>")]
    pub object: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub version: u32,
    #[serde(deserialize_with = "paths::<_,16>")]
    pub directories: Vec<String>,
    /// Explicit virtual-platform defaults; never read from the Fedora host.
    #[serde(deserialize_with = "paths::<_,8>")]
    pub trusted: Vec<String>,
    #[serde(deserialize_with = "list::<_,Object,24>")]
    pub objects: Vec<Object>,
    #[serde(deserialize_with = "list::<_,Program,8>")]
    pub programs: Vec<Program>,
    #[serde(deserialize_with = "list::<_,Alias,24>")]
    pub aliases: Vec<Alias>,
}
impl Catalog {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.directories.is_empty()
            || self.directories.len() > MAX_DIRECTORIES
            || self.trusted.is_empty()
            || self.trusted.len() > 8
            || self.objects.len() > MAX_OBJECTS
            || self.programs.is_empty()
            || self.programs.len() > MAX_PROGRAMS
            || self.aliases.len() > MAX_OBJECTS
            || !unique(self.directories.iter())
            || !unique(self.trusted.iter())
            || self.directories.iter().any(|p| !canonical(p))
            || self.trusted.iter().any(|p| !self.directories.contains(p))
            || !fits(self)
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid bounded virtual library catalog",
            ));
        }
        let dependency_names = |v: &Vec<String>| {
            v.len() <= MAX_DEPENDENCIES && unique(v.iter()) && v.iter().all(|n| name(n))
        };
        let mut occupied = BTreeSet::new();
        let mut identities = BTreeSet::new();
        for o in &self.objects {
            o.abi.validate()?;
            if !canonical(&o.path)
                || !self.directories.iter().any(|p| p == parent(&o.path))
                || !name(basename(&o.path))
                || !name(&o.soname)
                || !identity(&o.identity)
                || !dependency_names(&o.needed)
                || !occupied.insert(o.path.as_str())
                || !identities.insert(o.identity.as_str())
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid or duplicate supplied library metadata",
                ));
            }
        }
        let platform = self.programs[0].abi;
        for p in &self.programs {
            p.abi.validate()?;
            if p.abi != platform {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Each supplied virtual platform has one program ABI and one explicit trusted-directory policy",
                ));
            }
            if !canonical(&p.path)
                || p.needed.is_empty()
                || !dependency_names(&p.needed)
                || !occupied.insert(p.path.as_str())
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid or duplicate supplied dynamically linked program metadata",
                ));
            }
        }
        for a in &self.aliases {
            let Some(o) = self.objects.iter().find(|o| o.path == a.object) else {
                return Err(error(ErrorKind::Invalid, "Alias names an unknown object"));
            };
            if !canonical(&a.path)
                || parent(&a.path) != parent(&o.path)
                || !name(basename(&a.path))
                || !occupied.insert(a.path.as_str())
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Only distinct supplied aliases in the object's directory are supported",
                ));
            }
        }
        // Admit only fixtures whose existing SONAME links require no repair.
        for o in &self.objects {
            let soname_path = format!("{}/{}", parent(&o.path), o.soname);
            if soname_path != o.path
                && !self
                    .aliases
                    .iter()
                    .any(|a| a.path == soname_path && a.object == o.path)
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Supply the correct existing SONAME link; link creation/repair is outside this slice",
                ));
            }
        }
        Ok(())
    }
    fn object(&self, path: &str) -> Option<&Object> {
        let target = self
            .aliases
            .iter()
            .find(|a| a.path == path)
            .map_or(path, |a| a.object.as_str());
        self.objects.iter().find(|o| o.path == target)
    }
    fn path_list(&self, value: &str) -> Result<Vec<String>> {
        if value.len() > MAX_PATH_LIST {
            return Err(error(
                ErrorKind::Limit,
                "Library path list exceeds 1024 bytes",
            ));
        }
        let mut out = Vec::new();
        for p in value.split([':', ';']) {
            if p.is_empty() || !canonical(p) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Use supplied absolute directories; empty/current-directory, relative paths and expansion tokens are not supported",
                ));
            }
            if !self.directories.iter().any(|d| d == p) {
                return Err(error(
                    ErrorKind::NotFound,
                    format!("Directory {p} is not in this virtual inventory"),
                ));
            }
            if !out.iter().any(|d| d == p) {
                out.push(p.to_owned());
            }
            if out.len() > MAX_DIRECTORIES {
                return Err(error(ErrorKind::Limit, "Library search-directory limit"));
            }
        }
        Ok(out)
    }
    fn scan_directories(&self, configuration: &str) -> Result<Vec<String>> {
        if configuration.len() > MAX_CONFIG {
            return Err(error(
                ErrorKind::Limit,
                "Library configuration exceeds 4096 bytes",
            ));
        }
        let mut out = Vec::new();
        for line in configuration.lines() {
            let line = line.split('#').next().unwrap().trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("include") || !canonical(line) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This configuration subset accepts supplied absolute directories and comments; include/globs/tokens/options are not supported",
                ));
            }
            if !self.directories.iter().any(|d| d == line) {
                return Err(error(
                    ErrorKind::NotFound,
                    format!("Directory {line} is not in this virtual inventory"),
                ));
            }
            if !out.iter().any(|p| p == line) {
                out.push(line.to_owned());
            }
            if out.len() > MAX_DIRECTORIES {
                return Err(error(
                    ErrorKind::Limit,
                    "Library configuration directory limit",
                ));
            }
        }
        for p in &self.trusted {
            if !out.contains(p) {
                out.push(p.clone());
            }
        }
        if out.len() > MAX_DIRECTORIES {
            return Err(error(ErrorKind::Limit, "Combined cache-directory limit"));
        }
        Ok(out)
    }
    fn build_entries(&self, directories: &[String]) -> Result<Vec<CacheEntry>> {
        let mut entries = BTreeMap::new();
        for dir in directories {
            for o in self.objects.iter().filter(|o| parent(&o.path) == dir) {
                let key = (o.soname.clone(), o.abi);
                if entries.contains_key(&key) {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "Multiple cache candidates for one SONAME/architecture need version-selection rules not yet modeled; previous cache retained",
                    ));
                }
                entries.insert(
                    key,
                    CacheEntry {
                        name: o.soname.clone(),
                        path: format!("{}/{}", parent(&o.path), o.soname),
                        object: o.path.clone(),
                        abi: o.abi,
                    },
                );
            }
        }
        Ok(entries.into_values().collect())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheEntry {
    #[serde(deserialize_with = "text::<_,128>")]
    pub name: String,
    #[serde(deserialize_with = "text::<_,256>")]
    pub path: String,
    #[serde(deserialize_with = "text::<_,256>")]
    pub object: String,
    pub abi: Abi,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cache {
    pub generation: u32,
    #[serde(deserialize_with = "text::<_,4096>")]
    pub source_configuration: String,
    #[serde(deserialize_with = "paths::<_,16>")]
    pub directories: Vec<String>,
    #[serde(deserialize_with = "list::<_,CacheEntry,24>")]
    pub entries: Vec<CacheEntry>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub catalog: Catalog,
    #[serde(deserialize_with = "text::<_,4096>")]
    pub configuration: String,
    #[serde(deserialize_with = "optional_path_list")]
    pub library_path: Option<String>,
    pub cache: Option<Cache>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Environment {
    Configured,
    Clean,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Via {
    Environment(String),
    Cache,
    Trusted(String),
    Loaded,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub path: String,
    pub object: String,
    pub identity: String,
    pub via: Via,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub requester: String,
    pub needed: String,
    pub selected: Option<Selected>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub program: String,
    pub bindings: Vec<Binding>,
}
impl Resolution {
    /// A state property, deliberately NOT a claim about real ldd's exit status.
    pub fn complete(&self) -> bool {
        self.bindings.iter().all(|b| b.selected.is_some())
    }
    pub fn report(&self) -> Result<String> {
        if self.bindings.len() > MAX_VISITS {
            return Err(error(ErrorKind::Limit, "Dependency report edge limit"));
        }
        let mut out = String::from(
            "Simulated dependency metadata; no ELF code or memory addresses executed.\n",
        );
        for b in &self.bindings {
            if b.needed.len() > 128
                || b.requester.len() > 256
                || b.selected
                    .as_ref()
                    .is_some_and(|s| s.object.len() > 256 || s.path.len() > 256)
            {
                return Err(error(ErrorKind::Limit, "Dependency report field limit"));
            }
            let line = match &b.selected {
                Some(s) => format!(
                    "{} => {} [requested by {}]\n",
                    b.needed, s.path, b.requester
                ),
                None => format!("{} => not found [requested by {}]\n", b.needed, b.requester),
            };
            if out
                .len()
                .checked_add(line.len())
                .is_none_or(|n| n > MAX_OUTPUT)
            {
                return Err(error(
                    ErrorKind::Limit,
                    "Dependency report exceeds 32 KiB; no partial report published",
                ));
            }
            out.push_str(&line);
        }
        Ok(out)
    }
}
impl State {
    pub fn new(catalog: Catalog, configuration: &str) -> Result<Self> {
        if configuration.len() > MAX_CONFIG {
            return Err(error(ErrorKind::Limit, "Library configuration byte limit"));
        }
        let s = Self {
            version: VERSION,
            catalog,
            configuration: configuration.into(),
            library_path: None,
            cache: None,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION || !fits(self) {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid version or library-state byte limit",
            ));
        }
        self.catalog.validate()?;
        self.catalog.scan_directories(&self.configuration)?;
        if let Some(value) = &self.library_path {
            self.catalog.path_list(value)?;
        }
        if let Some(cache) = &self.cache {
            if cache.generation == 0
                || cache.generation > MAX_REBUILDS
                || cache.entries.len() > MAX_OBJECTS
            {
                return Err(error(ErrorKind::Invalid, "Invalid cache version/count"));
            }
            let directories = self.catalog.scan_directories(&cache.source_configuration)?;
            if directories != cache.directories
                || self.catalog.build_entries(&directories)? != cache.entries
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Saved cache differs from its supplied catalog and scanned configuration",
                ));
            }
        }
        Ok(())
    }
    fn publish(&mut self, next: Self) -> Result<()> {
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn save_configuration(&mut self, text: &str) -> Result<()> {
        self.validate()?;
        self.catalog.scan_directories(text)?;
        let mut next = self.clone();
        next.configuration = text.into();
        self.publish(next)
    }
    pub fn set_library_path(&mut self, value: Option<&str>) -> Result<()> {
        self.validate()?;
        if let Some(v) = value {
            self.catalog.path_list(v)?;
        }
        let mut next = self.clone();
        next.library_path = value.map(str::to_owned);
        self.publish(next)
    }
    pub fn rebuild_cache(&mut self) -> Result<()> {
        self.validate()?;
        let generation = self.cache.as_ref().map_or(1, |c| c.generation + 1);
        if generation > MAX_REBUILDS {
            return Err(error(
                ErrorKind::Limit,
                "This bounded activity permits 64 cache rebuilds; existing cache retained",
            ));
        }
        let directories = self.catalog.scan_directories(&self.configuration)?;
        let entries = self.catalog.build_entries(&directories)?;
        let cache = Cache {
            generation,
            source_configuration: self.configuration.clone(),
            directories,
            entries,
        };
        let mut next = self.clone();
        next.cache = Some(cache);
        self.publish(next)
    }
    pub fn cache_matches_configuration(&self) -> Result<bool> {
        self.validate()?;
        let directories = self.catalog.scan_directories(&self.configuration)?;
        Ok(self
            .cache
            .as_ref()
            .is_some_and(|c| c.directories == directories))
    }
    fn select(&self, needed: &str, abi: Abi, environment: &[String]) -> Result<Option<Selected>> {
        for (dirs, is_environment) in [
            (environment, true),
            (self.catalog.trusted.as_slice(), false),
        ] {
            // Cache belongs between the environment and default-directory phases.
            if !is_environment
                && let Some(e) = self
                    .cache
                    .as_ref()
                    .and_then(|c| c.entries.iter().find(|e| e.name == needed && e.abi == abi))
            {
                let o = self
                    .catalog
                    .object(&e.object)
                    .ok_or_else(|| error(ErrorKind::Invalid, "Unknown cached object"))?;
                return Ok(Some(Selected {
                    path: e.path.clone(),
                    object: o.path.clone(),
                    identity: o.identity.clone(),
                    via: Via::Cache,
                }));
            }
            for dir in dirs {
                let path = format!("{dir}/{needed}");
                if let Some(o) = self.catalog.object(&path) {
                    if o.abi != abi {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "A mismatched ELF class/machine candidate was encountered. Its fallback/error behavior needs the separately reviewed reference contract; no resolution was invented",
                        ));
                    }
                    return Ok(Some(Selected {
                        path,
                        object: o.path.clone(),
                        identity: o.identity.clone(),
                        via: if is_environment {
                            Via::Environment(dir.clone())
                        } else {
                            Via::Trusted(dir.clone())
                        },
                    }));
                }
            }
        }
        Ok(None)
    }
    pub fn resolve(&self, program: &str, mode: Environment) -> Result<Resolution> {
        self.validate()?;
        let p = self
            .catalog
            .programs
            .iter()
            .find(|p| p.path == program)
            .ok_or_else(|| {
                error(
                    ErrorKind::NotFound,
                    "Choose a supplied virtual program; no host path was inspected",
                )
            })?;
        let environment = if mode == Environment::Configured {
            self.library_path
                .as_deref()
                .map(|p| self.catalog.path_list(p))
                .transpose()?
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut queue = p
            .needed
            .iter()
            .map(|name| (p.path.clone(), name.clone(), 0))
            .collect::<VecDeque<_>>();
        let mut loaded = BTreeSet::new();
        let mut loaded_sonames = BTreeMap::new();
        let mut loaded_names: BTreeMap<String, Selected> = BTreeMap::new();
        let mut bindings = Vec::new();
        while let Some((requester, needed, depth)) = queue.pop_front() {
            if depth > MAX_DEPTH || bindings.len() >= MAX_VISITS {
                return Err(error(
                    ErrorKind::Limit,
                    "Bounded dependency traversal exceeded its depth/edge limit",
                ));
            }
            // Existing requested names and SONAMEs are reused before a new search.
            // Register names on first loading; later aliases do not replace an
            // earlier link-map match. No program or ELF instruction executes.
            let selected = if let Some(previous) = loaded_names.get(&needed) {
                let mut reused = previous.clone();
                reused.via = Via::Loaded;
                Some(reused)
            } else {
                self.select(&needed, p.abi, &environment)?
            };
            if let Some(s) = &selected {
                if loaded.insert(s.object.clone()) {
                    let object = self.catalog.object(&s.object).unwrap();
                    if loaded_sonames
                        .get(&object.soname)
                        .is_some_and(|previous| previous != &object.path)
                    {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Multiple loaded objects sharing a SONAME need link-map name reuse rules not yet modeled; no dependency result invented",
                        ));
                    }
                    loaded_sonames.insert(object.soname.clone(), object.path.clone());
                    loaded_names
                        .entry(object.soname.clone())
                        .or_insert_with(|| s.clone());
                    for child in &object.needed {
                        if queue.len() >= MAX_VISITS {
                            return Err(error(ErrorKind::Limit, "Bounded dependency queue limit"));
                        }
                        queue.push_back((object.path.clone(), child.clone(), depth + 1));
                    }
                }
                loaded_names
                    .entry(needed.clone())
                    .or_insert_with(|| s.clone());
            }
            bindings.push(Binding {
                requester,
                needed,
                selected,
            });
        }
        Ok(Resolution {
            program: p.path.clone(),
            bindings,
        })
    }
    pub fn environment_matches(&self, source: &Self) -> Result<bool> {
        self.validate()?;
        source.validate()?;
        Ok(self.version == source.version && self.catalog == source.catalog)
    }
    pub fn from_json(raw: &[u8], source: &Self) -> Result<Self> {
        if raw.len() > MAX_STATE {
            return Err(error(
                ErrorKind::Limit,
                "Saved library activity exceeds 64 KiB",
            ));
        }
        let s: Self = serde_json::from_slice(raw).map_err(|e| {
            error(
                ErrorKind::Invalid,
                format!("Invalid saved library activity: {e}"),
            )
        })?;
        s.validate()?;
        if !s.environment_matches(source)? {
            return Err(error(
                ErrorKind::Invalid,
                "Saved library work belongs to different supplied metadata; original retained",
            ));
        }
        Ok(s)
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|e| error(ErrorKind::Invalid, e.to_string()))
    }
}

#[cfg(test)]
mod tests;
