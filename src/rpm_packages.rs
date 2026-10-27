//! Restricted, script-free RPM package observations and logical archive sets.
//! No RPM/DNF process, archive parser, CLI alias, resolver or transaction rollback.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::{self, Write},
    marker::PhantomData,
};
pub const MAX_RAW_STATE: usize = 2 * 1024 * 1024;
pub const MAX_COMPACT: usize = 1024 * 1024;
pub const MAX_STRINGS: usize = 128 * 1024;
pub const MAX_TOKENS: usize = 8192;
pub const MAX_ACTION: usize = 16 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Code {
    Malformed,
    Limit,
    Context,
    Unavailable,
    AlreadyInstalled,
    NotInstalled,
    Dependency,
    Ownership,
    UnsupportedSubset,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: Code,
    pub message: String,
}
type Result<T> = std::result::Result<T, Error>;
fn fail<T>(code: Code, message: &str) -> Result<T> {
    Err(Error {
        code,
        message: message.chars().take(300).collect(),
    })
}
fn limit(ok: bool) -> Result<()> {
    if ok {
        Ok(())
    } else {
        fail(
            Code::Limit,
            "Restricted RPM scalar, collection or combined byte budget exceeded",
        )
    }
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-+_.".contains(&c))
}
fn path(s: &str) -> bool {
    s.len() <= 160
        && s.starts_with('/')
        && s != "/"
        && s.is_ascii()
        && !s.bytes().any(|b| b < 0x20 || b == 0x7f)
        && s.split('/')
            .skip(1)
            .all(|c| !c.is_empty() && c != "." && c != "..")
}
fn account(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    struct V<const N: usize>;
    impl<'de, const N: usize> Visitor<'de> for V<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "string <= {N} UTF8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > N {
                Err(E::custom("RPM text bound"))
            } else {
                Ok(s.to_owned())
            }
        }
    }
    d.deserialize_str(V::<N>)
}
struct TextSeed<const N: usize>;
impl<'de, const N: usize> DeserializeSeed<'de> for TextSeed<N> {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
        text::<D, N>(d)
    }
}
struct Element<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Element<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("RPM count bound"));
        }
        T::deserialize(d)
    }
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct V<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for V<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "<= {N} elements")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(x) = a.next_element_seed(Element {
                allowed: v.len() < N,
                marker: PhantomData,
            })? {
                v.push(x);
            }
            Ok(v)
        }
    }
    d.deserialize_seq(V::<T, N>(PhantomData))
}
fn map<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize, const K: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, T>, D::Error> {
    struct V<T, const N: usize, const K: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize, const K: usize> Visitor<'de> for V<T, N, K> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "<= {N} unique keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut v = BTreeMap::new();
            while let Some(k) = a.next_key_seed(TextSeed::<K>)? {
                if v.len() >= N || v.contains_key(&k) {
                    return Err(A::Error::custom("RPM map count/duplicate key"));
                }
                v.insert(k, a.next_value()?);
            }
            Ok(v)
        }
    }
    d.deserialize_map(V::<T, N, K>(PhantomData))
}
fn strings<'de, D: Deserializer<'de>, const N: usize, const K: usize>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    struct V<const N: usize, const K: usize>;
    impl<'de, const N: usize, const K: usize> Visitor<'de> for V<N, K> {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "<= {N} bounded strings")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(s) = a.next_element_seed(StringElement::<K> {
                allowed: v.len() < N,
            })? {
                if v.contains(&s) {
                    return Err(A::Error::custom("RPM duplicate string"));
                }
                v.push(s);
            }
            Ok(v)
        }
    }
    struct StringElement<const K: usize> {
        allowed: bool,
    }
    impl<'de, const K: usize> DeserializeSeed<'de> for StringElement<K> {
        type Value = String;
        fn deserialize<X: Deserializer<'de>>(self, d: X) -> std::result::Result<String, X::Error> {
            if !self.allowed {
                return Err(X::Error::custom("RPM count bound"));
            }
            text::<X, K>(d)
        }
    }
    d.deserialize_seq(V::<N, K>)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Evr(String);
impl<'de> Deserialize<'de> for Evr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = text::<D, 128>(d)?;
        Self::parse(&s).map_err(|_| D::Error::custom("Invalid restricted RPM EVR"))
    }
}
impl Evr {
    pub fn parse(s: &str) -> Result<Self> {
        limit(s.len() <= 128)?;
        let rest = if let Some((epoch, rest)) = s.split_once(':') {
            if epoch.is_empty()
                || !epoch.bytes().all(|b| b.is_ascii_digit())
                || epoch.parse::<u32>().is_err()
            {
                return fail(
                    Code::Malformed,
                    "RPM epoch needs a bounded nonnegative integer",
                );
            }
            rest
        } else {
            s
        };
        let Some((v, r)) = rest.split_once('-') else {
            return fail(
                Code::UnsupportedSubset,
                "This subset requires complete version-release",
            );
        };
        let component = |x: &str| {
            !x.is_empty()
                && x.bytes().any(|b| b.is_ascii_alphanumeric())
                && x.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._+~^".contains(&b))
        };
        if !component(v) || !component(r) {
            return fail(
                Code::Malformed,
                "Restricted RPM version/release characters differ",
            );
        }
        Ok(Self(s.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    fn parts(&self) -> (u32, &str, &str) {
        let (e, t) = self
            .0
            .split_once(':')
            .map_or((0, self.0.as_str()), |(e, t)| (e.parse().unwrap(), t));
        let (v, r) = t.split_once('-').unwrap();
        (e, v, r)
    }
    pub fn compare(&self, other: &Self) -> Ordering {
        let (a, v, r) = self.parts();
        let (b, w, s) = other.parts();
        a.cmp(&b)
            .then_with(|| component_cmp(v, w))
            .then_with(|| component_cmp(r, s))
    }
}
// Independently written from the behavioral version manual, not upstream GPL code.
fn component_cmp(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0, 0);
    loop {
        while i < a.len() && !a[i].is_ascii_alphanumeric() && !b"~^".contains(&a[i]) {
            i += 1;
        }
        while j < b.len() && !b[j].is_ascii_alphanumeric() && !b"~^".contains(&b[j]) {
            j += 1;
        }
        if a.get(i) == Some(&b'~') || b.get(j) == Some(&b'~') {
            match (a.get(i) == Some(&b'~'), b.get(j) == Some(&b'~')) {
                (true, true) => {
                    i += 1;
                    j += 1;
                    continue;
                }
                (true, false) => return Ordering::Less,
                _ => return Ordering::Greater,
            }
        }
        if a.get(i) == Some(&b'^') || b.get(j) == Some(&b'^') {
            if i == a.len() {
                return Ordering::Less;
            }
            if j == b.len() {
                return Ordering::Greater;
            }
            match (a.get(i) == Some(&b'^'), b.get(j) == Some(&b'^')) {
                (true, true) => {
                    i += 1;
                    j += 1;
                    continue;
                }
                (true, false) => return Ordering::Less,
                _ => return Ordering::Greater,
            }
        }
        if i == a.len() || j == b.len() {
            return (a.len() - i).cmp(&(b.len() - j));
        }
        let numeric = a[i].is_ascii_digit();
        if numeric != b[j].is_ascii_digit() {
            return if numeric {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
        let (mut x, mut y) = (i, j);
        while i < a.len() && a[i].is_ascii_digit() == numeric && a[i].is_ascii_alphanumeric() {
            i += 1;
        }
        while j < b.len() && b[j].is_ascii_digit() == numeric && b[j].is_ascii_alphanumeric() {
            j += 1;
        }
        if numeric {
            while x < i && a[x] == b'0' {
                x += 1;
            }
            while y < j && b[y] == b'0' {
                y += 1;
            }
            let c = (i - x).cmp(&(j - y));
            if c != Ordering::Equal {
                return c;
            }
        }
        let c = a[x..i].cmp(&b[y..j]);
        if c != Ordering::Equal {
            return c;
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "noarch")]
    Noarch,
}
impl Architecture {
    fn label(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Noarch => "noarch",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[serde(deserialize_with = "text::<_,32>")]
    pub name: String,
    pub arch: Architecture,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relation {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}
impl Relation {
    fn accepts(self, c: Ordering) -> bool {
        match self {
            Self::Lt => c == Ordering::Less,
            Self::Le => c != Ordering::Greater,
            Self::Eq => c == Ordering::Equal,
            Self::Ge => c != Ordering::Less,
            Self::Gt => c == Ordering::Greater,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Lt => "Lt",
            Self::Le => "Le",
            Self::Eq => "Eq",
            Self::Ge => "Ge",
            Self::Gt => "Gt",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub relation: Relation,
    pub evr: Evr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    #[serde(deserialize_with = "text::<_,32>")]
    pub capability: String,
    pub constraint: Option<Constraint>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Ordinary,
    Config,
    ConfigNoreplace,
}
impl FileKind {
    fn label(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::Config => "config",
            Self::ConfigNoreplace => "config_noreplace",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(deserialize_with = "text::<_,4096>")]
    pub contents: String,
    pub mode: u16,
    #[serde(deserialize_with = "text::<_,32>")]
    pub owner: String,
    #[serde(deserialize_with = "text::<_,32>")]
    pub group: String,
    pub mtime: u64,
}
impl File {
    fn shape(&self) -> Result<()> {
        limit(self.contents.len() <= 4096 && self.owner.len() <= 32 && self.group.len() <= 32)?;
        if self.mode > 0o777 || !account(&self.owner) || !account(&self.group) {
            return fail(Code::Malformed, "Restricted regular file metadata differs");
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub file: File,
    pub kind: FileKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub identity: Identity,
    pub evr: Evr,
    #[serde(deserialize_with = "list::<_,Requirement,8>")]
    pub requires: Vec<Requirement>,
    #[serde(deserialize_with = "map::<_,Option<Evr>,8,32>")]
    pub provides: BTreeMap<String, Option<Evr>>,
    #[serde(deserialize_with = "map::<_,Payload,16,160>")]
    pub files: BTreeMap<String, Payload>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "strings::<_,72,32>")]
    pub archives: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    #[serde(deserialize_with = "text::<_,32>")]
    pub fixture: String,
    pub target: Architecture,
    pub published: SnapshotId,
    #[serde(deserialize_with = "map::<_,Archive,72,32>")]
    pub archives: BTreeMap<String, Archive>,
    #[serde(deserialize_with = "map::<_,Snapshot,3,32>")]
    pub snapshots: BTreeMap<String, Snapshot>,
    #[serde(deserialize_with = "strings::<_,128,160>")]
    pub allowed_paths: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installed {
    pub identity: Identity,
    #[serde(deserialize_with = "text::<_,32>")]
    pub archive: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    #[serde(deserialize_with = "list::<_,Installed,24>")]
    pub installed: Vec<Installed>,
    #[serde(deserialize_with = "strings::<_,72,32>")]
    pub available: Vec<String>,
    pub cached: Option<SnapshotId>,
    #[serde(deserialize_with = "map::<_,File,128,160>")]
    pub files: BTreeMap<String, File>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SnapshotId(String);
impl SnapshotId {
    pub fn new(s: &str) -> Result<Self> {
        if !name(s) {
            return fail(Code::Malformed, "Snapshot identifier differs");
        }
        Ok(Self(s.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for SnapshotId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = text::<D, 32>(d)?;
        Self::new(&s).map_err(|_| D::Error::custom("Invalid snapshot ID"))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    schema_version: u32,
    catalog: Catalog,
    runtime: Runtime,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema_version: u32,
            catalog: Catalog,
            runtime: Runtime,
        }
        let r = Raw::deserialize(d)?;
        if r.schema_version != 1 {
            return Err(D::Error::custom("Future RPM state"));
        }
        State::new(r.catalog, r.runtime).map_err(|e| D::Error::custom(e.message))
    }
}
// Counts every serialized string occurrence, including static keys and enum tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub strings: usize,
    pub tokens: usize,
}
impl Budget {
    fn token(&mut self) -> Result<()> {
        self.tokens = self.tokens.checked_add(1).ok_or_else(|| Error {
            code: Code::Limit,
            message: "RPM token overflow".into(),
        })?;
        limit(self.tokens <= MAX_TOKENS)
    }
    fn string(&mut self, s: &str) -> Result<()> {
        self.token()?;
        self.strings = self.strings.checked_add(s.len()).ok_or_else(|| Error {
            code: Code::Limit,
            message: "RPM byte overflow".into(),
        })?;
        limit(self.strings <= MAX_STRINGS)
    }
    fn object(&mut self, keys: &[&str]) -> Result<()> {
        self.token()?;
        for k in keys {
            self.string(k)?;
        }
        Ok(())
    }
    fn identity(&mut self, i: &Identity) -> Result<()> {
        self.object(&["name", "arch"])?;
        self.string(&i.name)?;
        self.string(i.arch.label())
    }
    fn file(&mut self, f: &File) -> Result<()> {
        self.object(&["contents", "mode", "owner", "group", "mtime"])?;
        self.string(&f.contents)?;
        self.token()?;
        self.string(&f.owner)?;
        self.string(&f.group)?;
        self.token()
    }
    fn catalog(&mut self, c: &Catalog) -> Result<()> {
        self.object(&[
            "fixture",
            "target",
            "published",
            "archives",
            "snapshots",
            "allowed_paths",
        ])?;
        self.string(&c.fixture)?;
        self.string(c.target.label())?;
        self.string(c.published.as_str())?;
        self.token()?;
        for (id, a) in &c.archives {
            self.string(id)?;
            self.object(&["identity", "evr", "requires", "provides", "files"])?;
            self.identity(&a.identity)?;
            self.string(a.evr.as_str())?;
            self.token()?;
            for q in &a.requires {
                self.object(&["capability", "constraint"])?;
                self.string(&q.capability)?;
                if let Some(k) = &q.constraint {
                    self.object(&["relation", "evr"])?;
                    self.string(k.relation.label())?;
                    self.string(k.evr.as_str())?;
                } else {
                    self.token()?;
                }
            }
            self.token()?;
            for (k, v) in &a.provides {
                self.string(k)?;
                if let Some(v) = v {
                    self.string(v.as_str())?;
                } else {
                    self.token()?;
                }
            }
            self.token()?;
            for (p, f) in &a.files {
                self.string(p)?;
                self.object(&["file", "kind"])?;
                self.file(&f.file)?;
                self.string(f.kind.label())?;
            }
        }
        self.token()?;
        for (k, s) in &c.snapshots {
            self.string(k)?;
            self.object(&["archives"])?;
            self.token()?;
            for id in &s.archives {
                self.string(id)?;
            }
        }
        self.token()?;
        for p in &c.allowed_paths {
            self.string(p)?;
        }
        Ok(())
    }
    fn runtime(&mut self, r: &Runtime) -> Result<()> {
        self.runtime_parts(
            r.installed
                .iter()
                .map(|x| (&x.identity, x.archive.as_str())),
            &r.available,
            r.cached.as_ref(),
            r.files.iter().map(|(p, f)| (p.as_str(), f)),
        )
    }
    fn runtime_parts<'a>(
        &mut self,
        installed: impl Iterator<Item = (&'a Identity, &'a str)>,
        available: &[String],
        cached: Option<&SnapshotId>,
        files: impl Iterator<Item = (&'a str, &'a File)>,
    ) -> Result<()> {
        self.object(&["installed", "available", "cached", "files"])?;
        self.token()?;
        for (i, a) in installed {
            self.object(&["identity", "archive"])?;
            self.identity(i)?;
            self.string(a)?;
        }
        self.token()?;
        for a in available {
            self.string(a)?;
        }
        if let Some(s) = cached {
            self.string(s.as_str())?;
        } else {
            self.token()?;
        }
        self.token()?;
        for (p, f) in files {
            self.string(p)?;
            self.file(f)?;
        }
        Ok(())
    }
}
struct Count {
    n: usize,
    cap: usize,
}
impl Write for Count {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.n = self
            .n
            .checked_add(b.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| io::Error::other("RPM compact byte bound"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn compact<T: Serialize>(v: &T, cap: usize) -> Result<()> {
    serde_json::to_writer(Count { n: 0, cap }, v).map_err(|_| Error {
        code: Code::Limit,
        message: "RPM compact byte bound".into(),
    })
}

impl Catalog {
    fn shape(&self) -> Result<()> {
        limit(
            self.fixture.len() <= 32
                && self.archives.len() <= 72
                && self.snapshots.len() <= 3
                && self.allowed_paths.len() <= 128,
        )?;
        for (id, a) in &self.archives {
            limit(
                id.len() <= 32
                    && a.identity.name.len() <= 32
                    && a.requires.len() <= 8
                    && a.provides.len() <= 8
                    && a.files.len() <= 16,
            )?;
            for q in &a.requires {
                limit(q.capability.len() <= 32)?;
            }
            for k in a.provides.keys() {
                limit(k.len() <= 32)?;
            }
            for (p, f) in &a.files {
                limit(p.len() <= 160)?;
                f.file.shape()?;
            }
        }
        for (k, s) in &self.snapshots {
            limit(k.len() <= 32 && s.archives.len() <= 72)?;
            for a in &s.archives {
                limit(a.len() <= 32)?;
            }
        }
        for p in &self.allowed_paths {
            limit(p.len() <= 160)?;
        }
        Ok(())
    }
    fn semantic(&self) -> Result<()> {
        if !name(&self.fixture) || self.target != Architecture::X86_64 {
            return fail(
                Code::UnsupportedSubset,
                "This closed RPM catalog targets x86_64 plus noarch",
            );
        }
        if self.allowed_paths.iter().any(|p| !path(p))
            || self
                .allowed_paths
                .iter()
                .enumerate()
                .any(|(n, p)| self.allowed_paths[..n].contains(p))
        {
            return fail(Code::Malformed, "Duplicate or invalid allowed path");
        }
        let mut identities = BTreeMap::<&Identity, usize>::new();
        for (id, a) in &self.archives {
            if !name(id) || !name(&a.identity.name) {
                return fail(
                    Code::Malformed,
                    "Invalid supplied RPM archive/package identifier",
                );
            }
            let n = identities.entry(&a.identity).or_default();
            *n += 1;
            limit(*n <= 3)?;
            for (other_id, b) in &self.archives {
                if other_id < id
                    && a.identity == b.identity
                    && a.evr.compare(&b.evr) == Ordering::Equal
                {
                    return fail(
                        Code::Malformed,
                        "Equivalent-EVR artifacts for one identity are ambiguous",
                    );
                }
            }
            if a.requires
                .iter()
                .enumerate()
                .any(|(n, q)| !name(&q.capability) || a.requires[..n].contains(q))
                || a.provides.keys().any(|c| !name(c) || c == &a.identity.name)
            {
                return fail(
                    Code::UnsupportedSubset,
                    "Invalid/duplicate literal capabilities or explicit self-provide",
                );
            }
            for p in a.files.keys() {
                if !path(p) || !self.allowed_paths.contains(p) {
                    return fail(Code::Context, "Archive path outside supplied text universe");
                }
            }
        }
        limit(identities.len() <= 24)?;
        if !self.snapshots.contains_key(self.published.as_str()) {
            return fail(Code::Context, "Published snapshot is not supplied");
        }
        for (k, s) in &self.snapshots {
            if !name(k)
                || s.archives
                    .iter()
                    .enumerate()
                    .any(|(n, id)| !self.archives.contains_key(id) || s.archives[..n].contains(id))
            {
                return fail(
                    Code::Context,
                    "Repository snapshot has duplicate/foreign archive",
                );
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.shape()?;
        let mut b = Budget {
            strings: 0,
            tokens: 0,
        };
        b.catalog(self)?;
        compact(self, MAX_COMPACT)?;
        self.semantic()
    }
}
impl Runtime {
    fn shape(&self) -> Result<()> {
        limit(self.installed.len() <= 24 && self.available.len() <= 72 && self.files.len() <= 128)?;
        for x in &self.installed {
            limit(x.identity.name.len() <= 32 && x.archive.len() <= 32)?;
        }
        for a in &self.available {
            limit(a.len() <= 32)?;
        }
        for (p, f) in &self.files {
            limit(p.len() <= 160)?;
            f.shape()?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    InstallSet {
        #[serde(deserialize_with = "strings::<_,8,32>")]
        archives: Vec<String>,
    },
    UpgradeSet {
        #[serde(deserialize_with = "strings::<_,8,32>")]
        archives: Vec<String>,
    },
    ReinstallExact {
        #[serde(deserialize_with = "text::<_,32>")]
        archive: String,
    },
    EraseSet {
        #[serde(deserialize_with = "list::<_,Identity,8>")]
        packages: Vec<Identity>,
    },
}
impl Action {
    pub fn decode(raw: &[u8]) -> Result<Self> {
        limit(raw.len() <= MAX_ACTION)?;
        let a: Self = serde_json::from_slice(raw).map_err(|_| Error {
            code: Code::Malformed,
            message: "Invalid bounded logical RPM action".into(),
        })?;
        a.shape()?;
        Ok(a)
    }
    fn shape(&self) -> Result<()> {
        match self {
            Self::InstallSet { archives } | Self::UpgradeSet { archives } => {
                limit(
                    !archives.is_empty()
                        && archives.len() <= 8
                        && archives.iter().all(|s| s.len() <= 32),
                )?;
                if archives
                    .iter()
                    .enumerate()
                    .any(|(n, s)| !name(s) || archives[..n].contains(s))
                {
                    return fail(Code::Malformed, "Duplicate/invalid explicit archive set");
                }
            }
            Self::ReinstallExact { archive } => {
                limit(archive.len() <= 32)?;
                if !name(archive) {
                    return fail(Code::Malformed, "Invalid explicit archive ID");
                }
            }
            Self::EraseSet { packages } => {
                limit(
                    !packages.is_empty()
                        && packages.len() <= 8
                        && packages.iter().all(|p| p.name.len() <= 32),
                )?;
                if packages
                    .iter()
                    .enumerate()
                    .any(|(n, p)| !name(&p.name) || packages[..n].contains(p))
                {
                    return fail(Code::Malformed, "Duplicate/invalid explicit package set");
                }
            }
        }
        compact(self, MAX_ACTION)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyIssue {
    pub package: Identity,
    pub requirement: Requirement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difference {
    Missing,
    Contents,
    Mode,
    Owner,
    Group,
    Mtime,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileIssue {
    pub path: String,
    pub differences: Vec<Difference>,
    pub configuration: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub identity: Identity,
    pub before: Option<String>,
    pub after: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub changes: Vec<Change>,
}
impl State {
    pub fn new(catalog: Catalog, runtime: Runtime) -> Result<Self> {
        let s = Self {
            schema_version: 1,
            catalog,
            runtime,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn empty(catalog: Catalog) -> Result<Self> {
        Self::new(
            catalog,
            Runtime {
                installed: Vec::new(),
                available: Vec::new(),
                cached: None,
                files: BTreeMap::new(),
            },
        )
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        limit(raw.len() <= MAX_RAW_STATE)?;
        serde_json::from_slice(raw).map_err(|_| Error {
            code: Code::Malformed,
            message: "Invalid bounded RPM State".into(),
        })
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    pub fn validate_context(&self, catalog: &Catalog) -> Result<()> {
        catalog.validate()?;
        self.validate()?;
        if &self.catalog != catalog {
            return fail(
                Code::Context,
                "Supplied RPM catalog differs from immutable original",
            );
        }
        Ok(())
    }
    pub fn budget(&self) -> Result<Budget> {
        self.catalog.shape()?;
        self.runtime.shape()?;
        let mut b = Budget {
            strings: 0,
            tokens: 0,
        };
        b.object(&["schema_version", "catalog", "runtime"])?;
        b.token()?;
        b.catalog(&self.catalog)?;
        b.runtime(&self.runtime)?;
        compact(self, MAX_COMPACT)?;
        Ok(b)
    }
    pub fn validate(&self) -> Result<()> {
        #[cfg(test)]
        QUERY_VALIDATIONS.with(|n| n.set(n.get() + 1));
        if self.schema_version != 1 {
            return fail(Code::Malformed, "Future RPM State");
        }
        self.budget()?;
        self.catalog.semantic()?;
        let r = &self.runtime;
        let c = &self.catalog;
        if r.available
            .iter()
            .enumerate()
            .any(|(n, a)| !c.archives.contains_key(a) || r.available[..n].contains(a))
            || r.cached
                .as_ref()
                .is_some_and(|s| !c.snapshots.contains_key(s.as_str()))
        {
            return fail(
                Code::Context,
                "Foreign/duplicate local archive or cached snapshot",
            );
        }
        for (n, x) in r.installed.iter().enumerate() {
            let Some(a) = c.archives.get(&x.archive) else {
                return fail(
                    Code::Context,
                    "Installed record has unknown supplied archive",
                );
            };
            if x.identity != a.identity {
                return fail(
                    Code::Context,
                    "Installed identity differs from recorded archive",
                );
            }
            if r.installed[..n]
                .iter()
                .any(|v| v.identity.name == x.identity.name)
            {
                return fail(
                    Code::UnsupportedSubset,
                    "Only one installed artifact per NAME; no multilib or parallel versions",
                );
            }
        }
        let mut owners = BTreeSet::new();
        for x in &r.installed {
            for p in c.archives[&x.archive].files.keys() {
                if !owners.insert(p) {
                    return fail(
                        Code::Ownership,
                        "Multiple installed manifests claim one virtual path",
                    );
                }
            }
        }
        if r.files
            .keys()
            .any(|p| !path(p) || !c.allowed_paths.contains(p))
        {
            return fail(
                Code::Context,
                "Current file is outside the supplied universe",
            );
        }
        // Missing/changed payload and unmet Requires are operational states, not shape errors.
        Ok(())
    }
    fn provides(a: &Archive, q: &Requirement) -> bool {
        let version = if a.identity.name == q.capability {
            Some(Some(&a.evr))
        } else {
            a.provides.get(&q.capability).map(|v| v.as_ref())
        };
        match (&q.constraint, version) {
            (None, Some(_)) => true,
            (Some(c), Some(Some(v))) => c.relation.accepts(v.compare(&c.evr)),
            _ => false,
        }
    }
    fn requirement_shape(q: &Requirement) -> Result<()> {
        limit(q.capability.len() <= 32)?;
        if !name(&q.capability) {
            return fail(
                Code::UnsupportedSubset,
                "Only literal bounded RPM capability requirements are admitted",
            );
        }
        Ok(())
    }
    fn identity_shape(identity: &Identity) -> Result<()> {
        limit(identity.name.len() <= 32)?;
        if !name(&identity.name) {
            return fail(Code::Malformed, "Invalid package identity");
        }
        Ok(())
    }
    pub fn installed(&self, identity: &Identity) -> Result<Option<&Archive>> {
        Self::identity_shape(identity)?;
        self.validate()?;
        Ok(self.installed_inner(identity))
    }
    fn installed_inner(&self, identity: &Identity) -> Option<&Archive> {
        self.runtime
            .installed
            .iter()
            .find(|x| &x.identity == identity)
            .map(|x| &self.catalog.archives[&x.archive])
    }
    pub fn local_archive(&self, id: &str) -> Result<&Archive> {
        limit(id.len() <= 32)?;
        self.validate()?;
        self.local_archive_inner(id)
    }
    fn local_archive_inner(&self, id: &str) -> Result<&Archive> {
        if !self.runtime.available.iter().any(|s| s == id) {
            return fail(
                Code::Unavailable,
                "Supplied archive is not locally available",
            );
        }
        self.catalog.archives.get(id).ok_or_else(|| Error {
            code: Code::Context,
            message: "Unknown archive ID".into(),
        })
    }
    pub fn installed_providers(&self, q: &Requirement) -> Result<Vec<&Archive>> {
        Self::requirement_shape(q)?;
        self.validate()?;
        Ok(self.installed_providers_inner(q))
    }
    fn installed_providers_inner(&self, q: &Requirement) -> Vec<&Archive> {
        self.runtime
            .installed
            .iter()
            .map(|x| &self.catalog.archives[&x.archive])
            .filter(|a| Self::provides(a, q))
            .collect()
    }
    pub fn available_providers(&self, q: &Requirement) -> Result<Vec<&Archive>> {
        Self::requirement_shape(q)?;
        self.validate()?;
        let Some(s) = &self.runtime.cached else {
            return fail(Code::Unavailable, "No cached RPM repository metadata");
        };
        Ok(self.catalog.snapshots[s.as_str()]
            .archives
            .iter()
            .map(|id| &self.catalog.archives[id])
            .filter(|a| Self::provides(a, q))
            .collect())
    }
    fn dependency_issues_inner(&self) -> Vec<DependencyIssue> {
        let mut out = Vec::new();
        for x in &self.runtime.installed {
            for q in &self.catalog.archives[&x.archive].requires {
                if !self
                    .runtime
                    .installed
                    .iter()
                    .any(|i| Self::provides(&self.catalog.archives[&i.archive], q))
                {
                    out.push(DependencyIssue {
                        package: x.identity.clone(),
                        requirement: q.clone(),
                    });
                }
            }
        }
        out
    }
    pub fn dependency_issues(&self) -> Result<Vec<DependencyIssue>> {
        self.validate()?;
        Ok(self.dependency_issues_inner())
    }
    pub fn recorded_owners(&self, p: &str) -> Result<Vec<Identity>> {
        limit(p.len() <= 160)?;
        if !path(p) {
            return fail(Code::Malformed, "Expected supplied absolute virtual path");
        }
        self.validate()?;
        if !self.catalog.allowed_paths.iter().any(|x| x == p) {
            return fail(Code::Context, "Path outside supplied universe");
        }
        Ok(self
            .runtime
            .installed
            .iter()
            .filter(|x| self.catalog.archives[&x.archive].files.contains_key(p))
            .map(|x| x.identity.clone())
            .collect())
    }
    pub fn file_issues(&self, identity: &Identity) -> Result<Vec<FileIssue>> {
        let Some(a) = self.installed(identity)? else {
            return fail(Code::NotInstalled, "Package is not installed");
        };
        Ok(self.file_issues_inner(a))
    }
    fn file_issues_inner(&self, a: &Archive) -> Vec<FileIssue> {
        let mut out = Vec::new();
        for (p, expected) in &a.files {
            let mut differences = Vec::new();
            if let Some(f) = self.runtime.files.get(p) {
                if f.contents != expected.file.contents {
                    differences.push(Difference::Contents);
                }
                if f.mode != expected.file.mode {
                    differences.push(Difference::Mode);
                }
                if f.owner != expected.file.owner {
                    differences.push(Difference::Owner);
                }
                if f.group != expected.file.group {
                    differences.push(Difference::Group);
                }
                if f.mtime != expected.file.mtime {
                    differences.push(Difference::Mtime);
                }
            } else {
                differences.push(Difference::Missing);
            }
            if !differences.is_empty() {
                out.push(FileIssue {
                    path: p.clone(),
                    differences,
                    configuration: expected.kind != FileKind::Ordinary,
                });
            }
        }
        out
    }
    fn config_manifest(a: &Archive) -> BTreeMap<&str, FileKind> {
        a.files
            .iter()
            .filter(|(_, p)| p.kind != FileKind::Ordinary)
            .map(|(p, f)| (p.as_str(), f.kind))
            .collect()
    }
    fn check_config(&self, old: &Archive, new: Option<&Archive>) -> Result<()> {
        if new.is_some_and(|a| Self::config_manifest(old) != Self::config_manifest(a)) {
            return fail(
                Code::UnsupportedSubset,
                "Configuration manifest changes need the deferred RPM sidecar table",
            );
        }
        for (p, f) in &old.files {
            if f.kind != FileKind::Ordinary && self.runtime.files.get(p) != Some(&f.file) {
                return fail(
                    Code::UnsupportedSubset,
                    "Modified or missing configuration needs the deferred RPM sidecar table",
                );
            }
        }
        Ok(())
    }
    fn candidate(&self, action: &Action) -> Result<(Self, Plan)> {
        // All borrowed shape/aggregate/action bounds precede candidate clones and derived plans.
        action.shape()?;
        self.validate()?;
        let mut updates = Vec::<(Option<&Installed>, Option<&str>)>::new();
        match action {
            Action::InstallSet { archives } | Action::UpgradeSet { archives } => {
                for id in archives {
                    let a = self.local_archive_inner(id)?;
                    let old = self
                        .runtime
                        .installed
                        .iter()
                        .find(|i| i.identity.name == a.identity.name);
                    if updates.iter().any(|(_, v)| {
                        v.is_some_and(|v| self.catalog.archives[v].identity.name == a.identity.name)
                    }) {
                        return fail(
                            Code::UnsupportedSubset,
                            "One archive per package NAME in a logical set",
                        );
                    }
                    if let Some(old) = old {
                        if old.identity.arch != a.identity.arch {
                            return fail(
                                Code::UnsupportedSubset,
                                "Cross-architecture replacement is not multilib support",
                            );
                        }
                        if matches!(action, Action::InstallSet { .. }) {
                            return fail(
                                Code::AlreadyInstalled,
                                "InstallSet does not replace an installed package",
                            );
                        }
                        let old_a = &self.catalog.archives[&old.archive];
                        match a.evr.compare(&old_a.evr) {
                            Ordering::Greater => {}
                            Ordering::Equal => {
                                return fail(
                                    Code::AlreadyInstalled,
                                    "UpgradeSet needs a newer RPM EVR",
                                );
                            }
                            Ordering::Less => {
                                return fail(
                                    Code::UnsupportedSubset,
                                    "Downgrade is outside this logical subset",
                                );
                            }
                        }
                        self.check_config(old_a, Some(a))?;
                    }
                    updates.push((old, Some(id)));
                }
            }
            Action::ReinstallExact { archive } => {
                let a = self.local_archive_inner(archive)?;
                let Some(old) = self
                    .runtime
                    .installed
                    .iter()
                    .find(|x| x.archive == *archive && x.identity == a.identity)
                else {
                    return fail(
                        Code::NotInstalled,
                        "Reinstall requires the exact installed archive",
                    );
                };
                self.check_config(a, Some(a))?;
                updates.push((Some(old), Some(archive)));
            }
            Action::EraseSet { packages } => {
                for identity in packages {
                    let Some(old) = self
                        .runtime
                        .installed
                        .iter()
                        .find(|x| &x.identity == identity)
                    else {
                        return fail(Code::NotInstalled, "Exact erase identity is not installed");
                    };
                    self.check_config(&self.catalog.archives[&old.archive], None)?;
                    updates.push((Some(old), None));
                }
            }
        }
        // Build only bounded borrowed references before copying catalog/payload text.
        let mut installed: Vec<(&Identity, &str)> = self
            .runtime
            .installed
            .iter()
            .filter(|x| {
                !updates
                    .iter()
                    .any(|(old, _)| old.is_some_and(|i| i.identity.name == x.identity.name))
            })
            .map(|x| (&x.identity, x.archive.as_str()))
            .collect();
        for (_, new) in &updates {
            if let Some(id) = new {
                let a = &self.catalog.archives[*id];
                installed.push((&a.identity, id));
            }
        }
        limit(installed.len() <= 24)?;
        installed.sort_by(|a, b| a.0.cmp(b.0));
        let mut owners = BTreeMap::new();
        for (i, id) in &installed {
            for p in self.catalog.archives[*id].files.keys() {
                if owners.insert(p.as_str(), *i).is_some() {
                    return fail(
                        Code::Ownership,
                        "Final manifests claim one virtual path more than once",
                    );
                }
            }
        }
        let mut files: BTreeMap<&str, &File> = self
            .runtime
            .files
            .iter()
            .map(|(p, f)| (p.as_str(), f))
            .collect();
        for (old, _) in &updates {
            if let Some(old) = old {
                for p in self.catalog.archives[&old.archive].files.keys() {
                    files.remove(p.as_str());
                }
            }
        }
        for (_, new) in &updates {
            if let Some(id) = new {
                let a = &self.catalog.archives[*id];
                for (p, f) in &a.files {
                    let previous = self
                        .runtime
                        .installed
                        .iter()
                        .find(|x| self.catalog.archives[&x.archive].files.contains_key(p));
                    if previous.is_some_and(|x| x.identity.name != a.identity.name)
                        || previous.is_none() && self.runtime.files.contains_key(p)
                    {
                        return fail(
                            Code::UnsupportedSubset,
                            "Ownership transfer/adopting unmanaged files is outside this logical set",
                        );
                    }
                    files.insert(p.as_str(), &f.file);
                }
            }
        }
        limit(files.len() <= 128)?;
        let mut budget = Budget {
            strings: 0,
            tokens: 0,
        };
        budget.object(&["schema_version", "catalog", "runtime"])?;
        budget.token()?;
        budget.catalog(&self.catalog)?;
        budget.runtime_parts(
            installed.iter().copied(),
            &self.runtime.available,
            self.runtime.cached.as_ref(),
            files.iter().map(|(p, f)| (*p, *f)),
        )?;
        // Combined S/N proof gives <=999424 compact bytes, before ANY full content clone.
        for (_, id) in &installed {
            for q in &self.catalog.archives[*id].requires {
                if !installed
                    .iter()
                    .any(|(_, id)| Self::provides(&self.catalog.archives[*id], q))
                {
                    return fail(
                        Code::UnsupportedSubset,
                        "This simulator accepts changes only when all installed requirements are met. This selection leaves requirements unmet. That whole-database rule is not a universal RPM install rule; compare the client's requirements with the archive's provided capabilities.",
                    );
                }
            }
        }
        let runtime = Runtime {
            installed: installed
                .iter()
                .map(|(i, a)| Installed {
                    identity: (*i).clone(),
                    archive: (*a).to_owned(),
                })
                .collect(),
            available: self.runtime.available.clone(),
            cached: self.runtime.cached.clone(),
            files: files
                .iter()
                .map(|(p, f)| ((*p).to_owned(), (*f).clone()))
                .collect(),
        };
        let next = Self::new(self.catalog.clone(), runtime)?;
        let changes = updates
            .iter()
            .map(|(old, new)| Change {
                identity: old
                    .map(|x| &x.identity)
                    .unwrap_or_else(|| &self.catalog.archives[new.unwrap()].identity)
                    .clone(),
                before: old.map(|x| x.archive.clone()),
                after: new.map(str::to_owned),
            })
            .collect();
        Ok((next, Plan { changes }))
    }
    pub fn plan(&self, action: &Action) -> Result<Plan> {
        self.candidate(action).map(|(_, p)| p)
    }
    pub fn apply(&mut self, action: &Action) -> Result<Plan> {
        let (next, plan) = self.candidate(action)?;
        *self = next;
        Ok(plan)
    }
}

// A checked borrow, never a serialized credential. Only this module constructs it.
pub(crate) struct ValidatedQueries<'a> {
    state: &'a State,
}
impl State {
    pub(crate) fn validated_queries(&self) -> Result<ValidatedQueries<'_>> {
        self.validate()?;
        Ok(ValidatedQueries { state: self })
    }
}
impl<'a> ValidatedQueries<'a> {
    pub(crate) fn installed(&self, id: &Identity) -> Result<Option<&'a Archive>> {
        State::identity_shape(id)?;
        Ok(self.state.installed_inner(id))
    }
    pub(crate) fn installed_providers(&self, q: &Requirement) -> Result<Vec<&'a Archive>> {
        State::requirement_shape(q)?;
        Ok(self.state.installed_providers_inner(q))
    }
    pub(crate) fn dependency_issues(&self) -> Vec<DependencyIssue> {
        self.state.dependency_issues_inner()
    }
    pub(crate) fn file_issues(&self, id: &Identity) -> Result<Vec<FileIssue>> {
        let Some(a) = self.installed(id)? else {
            return fail(Code::NotInstalled, "Package is not installed");
        };
        Ok(self.state.file_issues_inner(a))
    }
}
#[cfg(test)]
std::thread_local! { static QUERY_VALIDATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

#[cfg(test)]
mod validated_query_tests {
    use super::*;
    fn fixture() -> State {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/rpm-workspace-preview.json"))
                .unwrap();
        State::decode(&serde_json::to_vec(&v["scenarios"][0]["initial"]["simulation"]).unwrap())
            .unwrap()
    }
    fn id(n: &str) -> Identity {
        Identity {
            name: n.into(),
            arch: Architecture::Noarch,
        }
    }
    #[test]
    fn query_view_matches_complete_public_values_and_validates_once() {
        let mut s = fixture();
        s.apply(&Action::InstallSet {
            archives: vec!["provider-a".into()],
        })
        .unwrap();
        let expected_ids = ["desk-client", "quiet-neighbor", "ink-bridge", "missing"].map(|n| {
            let i = id(n);
            (
                i.clone(),
                s.installed(&i).unwrap().cloned(),
                s.file_issues(&i),
            )
        });
        let requirements = [
            Requirement {
                capability: "desk-api".into(),
                constraint: None,
            },
            Requirement {
                capability: "desk-api".into(),
                constraint: Some(Constraint {
                    relation: Relation::Ge,
                    evr: Evr::parse("2.0-1").unwrap(),
                }),
            },
            Requirement {
                capability: "ink-bridge".into(),
                constraint: None,
            },
        ];
        let providers = requirements
            .iter()
            .map(|q| {
                s.installed_providers(q)
                    .unwrap()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let dep = s.dependency_issues().unwrap();
        let before = s.clone();
        QUERY_VALIDATIONS.with(|n| n.set(0));
        let v = s.validated_queries().unwrap();
        assert_eq!(QUERY_VALIDATIONS.with(|n| n.get()), 1);
        assert!(std::ptr::eq(
            v.installed(&id("desk-client")).unwrap().unwrap(),
            &s.catalog.archives["client-a"]
        ));
        for _ in 0..4 {
            for (i, archive, issues) in &expected_ids {
                assert_eq!(v.installed(i).unwrap().cloned(), *archive);
                match issues {
                    Ok(e) => assert_eq!(v.file_issues(i).unwrap(), *e),
                    Err(e) => {
                        let got = v.file_issues(i).unwrap_err();
                        assert_eq!(got.code, e.code);
                        assert_eq!(got.message, e.message);
                    }
                }
            }
            for (q, expected) in requirements.iter().zip(&providers) {
                assert_eq!(
                    v.installed_providers(q)
                        .unwrap()
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>(),
                    *expected
                );
            }
            assert_eq!(v.dependency_issues(), dep);
        }
        assert_eq!(QUERY_VALIDATIONS.with(|n| n.get()), 1);
        assert!(s == before, "Queries must not mutate caller");
    }
    #[test]
    fn shared_file_loop_preserves_every_difference_order_and_configuration_flag() {
        let mut s = fixture();
        let f = s.runtime.files.get_mut("/etc/desk.conf").unwrap();
        f.contents = "changed".into();
        f.mode = 0o600;
        f.owner = "operator".into();
        f.group = "team".into();
        f.mtime += 1;
        s.runtime.files.remove("/usr/share/desk/report");
        let expected = s.file_issues(&id("desk-client")).unwrap();
        let v = s.validated_queries().unwrap();
        assert_eq!(v.file_issues(&id("desk-client")).unwrap(), expected);
        assert_eq!(
            expected[0].differences,
            vec![
                Difference::Contents,
                Difference::Mode,
                Difference::Owner,
                Difference::Group,
                Difference::Mtime
            ]
        );
        assert!(expected[0].configuration);
        assert_eq!(expected[1].differences, vec![Difference::Missing]);
    }
    #[test]
    fn public_query_argument_errors_still_precede_invalid_state_and_no_unchecked_view() {
        let mut s = fixture();
        s.schema_version = 99;
        let large = id(&"x".repeat(33));
        assert_eq!(s.installed(&large).unwrap_err().code, Code::Limit);
        assert_eq!(s.file_issues(&large).unwrap_err().code, Code::Limit);
        assert_eq!(s.installed(&id("Upper")).unwrap_err().code, Code::Malformed);
        assert_eq!(
            s.installed(&id("Upper")).unwrap_err().message,
            "Invalid package identity"
        );
        assert_eq!(
            s.file_issues(&id("missing")).unwrap_err().message,
            "Future RPM State"
        );
        let q = Requirement {
            capability: "Upper".into(),
            constraint: None,
        };
        assert_eq!(
            s.installed_providers(&q).unwrap_err().code,
            Code::UnsupportedSubset
        );
        assert_eq!(
            s.file_issues(&id("missing")).unwrap_err().code,
            Code::Malformed
        );
        assert!(s.validated_queries().is_err());
    }
}
