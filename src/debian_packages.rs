//! Logical script-free Debian package model. No real archive/command/rollback claim.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    io::Write,
    marker::PhantomData,
};
pub const MAX_JSON: usize = 4_194_304;
pub const MAX_NAMES: usize = 24;
pub const MAX_ARCHIVES: usize = 72;
pub const MAX_FILES: usize = 288;
pub const MAX_TEXT: usize = 262_144;
pub const MAX_VERSION: usize = 128;
pub const MAX_WORK: usize = 10_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    Invalid,
    Limit,
    Unsupported,
    Unknown,
    Unavailable,
    Architecture,
    Dependency,
    Collision,
    Conffile,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}
fn error(code: ErrorCode, text: impl AsRef<str>) -> Error {
    let s = text.as_ref();
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    let mut message = String::with_capacity(n);
    message.push_str(&s[..n]);
    Error { code, message }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_.+".contains(&c))
}
fn name(s: &str) -> bool {
    s.len() >= 2 && id(s) && s.as_bytes()[0].is_ascii_alphanumeric() && !s.contains('_')
}
fn path(s: &str) -> bool {
    s.len() <= 160
        && s.starts_with('/')
        && s.len() > 1
        && !s.chars().any(char::is_control)
        && s[1..]
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..")
}
fn text(s: &str, n: usize) -> bool {
    s.len() <= n && !s.contains('\0')
}
struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct T<const N: usize>;
        impl<const N: usize> Visitor<'_> for T<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most{N} text bytes")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                s: &str,
            ) -> std::result::Result<Self::Value, E> {
                if text(s, N) {
                    Ok(Text(s.into()))
                } else {
                    Err(E::custom("Package text limit"))
                }
            }
        }
        d.deserialize_str(T::<N>)
    }
}
fn scalar<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    Ok(Text::<N>::deserialize(d)?.0)
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize, const K: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize, const K: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize, const K: usize> Visitor<'de> for M<V, N, K> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} distinct bounded keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Text(k)) = a.next_key::<Text<K>>()? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom("Package map count/duplicate limit"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<V, N, K>(PhantomData))
}
fn list<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<V>, D::Error> {
    struct L<V, const N: usize>(PhantomData<V>);
    struct Next<V> {
        allowed: bool,
        v: PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("Package list count limit"));
            }
            V::deserialize(d)
        }
    }
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for L<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} package items")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next::<V> {
                allowed: out.len() < N,
                v: PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(L::<V, N>(PhantomData))
}
fn ids<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeSet<String>, D::Error> {
    let items = list::<D, Text<64>, N>(d)?;
    let mut out = BTreeSet::new();
    for Text(s) in items {
        if !id(&s) || !out.insert(s) {
            return Err(D::Error::custom("Invalid/duplicate package ID"));
        }
    }
    Ok(out)
}
fn paths<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeSet<String>, D::Error> {
    let items = list::<D, Text<160>, 32>(d)?;
    let mut out = BTreeSet::new();
    for Text(s) in items {
        if !path(&s) || !out.insert(s) {
            return Err(D::Error::custom("Invalid/duplicate package path"));
        }
    }
    Ok(out)
}
fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    struct Count {
        n: usize,
        cap: usize,
    }
    impl Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Package JSON limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, cap }, v).is_ok()
}

/// Text identity is retained exactly; semantic comparison is deliberately separate
/// from Eq, because e.g. absent and explicit zero revisions compare equivalent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Version(String);
fn parts(s: &str) -> Result<(u32, &str, &str)> {
    if s.is_empty() || s.len() > MAX_VERSION || !s.is_ascii() {
        return Err(error(
            ErrorCode::Unsupported,
            "Version needs1..128 ASCII bytes",
        ));
    }
    let (epoch, rest) = if let Some((e, r)) = s.split_once(':') {
        if e.is_empty() || !e.bytes().all(|c| c.is_ascii_digit()) || r.contains(':') {
            return Err(error(
                ErrorCode::Unsupported,
                "Unsupported Debian epoch syntax",
            ));
        }
        (
            e.parse::<u32>()
                .map_err(|_| error(ErrorCode::Limit, "Epoch exceeds this model's u32 bound"))?,
            r,
        )
    } else {
        (0, s)
    };
    let (upstream, revision) = rest.rsplit_once('-').unwrap_or((rest, "0"));
    if upstream.is_empty()
        || !upstream.as_bytes()[0].is_ascii_digit()
        || !upstream
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".+-~".contains(&c))
        || revision.is_empty()
        || !revision
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".+~".contains(&c))
    {
        return Err(error(
            ErrorCode::Unsupported,
            "Use the numeric-start Debian version subset: epoch:upstream-revision; no RPM/SemVer alias",
        ));
    }
    Ok((epoch, upstream, revision))
}
fn order(c: Option<u8>) -> i32 {
    match c {
        Some(b'~') => -1,
        None => 0,
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_ascii_alphabetic() => i32::from(c),
        Some(c) => i32::from(c) + 256,
    }
}
fn component(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        while a.get(i).is_some_and(|c| !c.is_ascii_digit())
            || b.get(j).is_some_and(|c| !c.is_ascii_digit())
        {
            let cmp = order(a.get(i).copied()).cmp(&order(b.get(j).copied()));
            if cmp != Ordering::Equal {
                return cmp;
            }
            if i < a.len() {
                i += 1;
            }
            if j < b.len() {
                j += 1;
            }
        }
        while a.get(i) == Some(&b'0') {
            i += 1;
        }
        while b.get(j) == Some(&b'0') {
            j += 1;
        }
        let (ai, bj) = (i, j);
        while a.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        while b.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
        }
        let cmp = (i - ai).cmp(&(j - bj));
        if cmp != Ordering::Equal {
            return cmp;
        }
        let cmp = a[ai..i].cmp(&b[bj..j]);
        if cmp != Ordering::Equal {
            return cmp;
        }
    }
    Ordering::Equal
}
impl Version {
    pub fn parse(s: &str) -> Result<Self> {
        parts(s)?;
        Ok(Self(s.into()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn compare(&self, other: &Self) -> Ordering {
        let (a, u, r) = parts(&self.0).expect("validated version");
        let (b, v, s) = parts(&other.0).expect("validated version");
        a.cmp(&b)
            .then_with(|| component(u, v))
            .then_with(|| component(r, s))
    }
}
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let Text(s) = Text::<MAX_VERSION>::deserialize(d)?;
        parts(&s).map_err(|_| D::Error::custom("Unsupported bounded Debian version"))?;
        Ok(Self(s))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    Amd64,
    Arm64,
    All,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relation {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub relation: Relation,
    pub version: Version,
}
impl Constraint {
    fn accepts(&self, v: &Version) -> bool {
        let c = v.compare(&self.version);
        match self.relation {
            Relation::Lt => c.is_lt(),
            Relation::Le => !c.is_gt(),
            Relation::Eq => c.is_eq(),
            Relation::Ge => !c.is_lt(),
            Relation::Gt => c.is_gt(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub name: String,
    pub constraint: Option<Constraint>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    #[serde(deserialize_with = "scalar::<_,2048>")]
    pub contents: String,
    pub conffile: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub name: String,
    pub version: Version,
    pub architecture: Architecture,
    #[serde(deserialize_with = "list::<_,Dependency,8>")]
    pub depends: Vec<Dependency>,
    #[serde(deserialize_with = "map::<_,Payload,32,160>")]
    pub files: BTreeMap<String, Payload>,
}
fn candidates<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    Ok(map::<D, Text<64>, 24, 64>(d)?
        .into_iter()
        .map(|(k, Text(v))| (k, v))
        .collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    #[serde(deserialize_with = "candidates")]
    pub candidates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub architecture: Architecture,
    pub repository_reachable: bool,
    pub published: String,
    pub archives: BTreeMap<String, Archive>,
    pub publications: BTreeMap<String, Index>,
    pub extra_paths: BTreeSet<String>,
}
impl<'de> Deserialize<'de> for Catalog {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            architecture: Architecture,
            repository_reachable: bool,
            #[serde(deserialize_with = "scalar::<_,64>")]
            published: String,
            #[serde(deserialize_with = "map::<_,Archive,72,64>")]
            archives: BTreeMap<String, Archive>,
            #[serde(deserialize_with = "map::<_,Index,3,64>")]
            publications: BTreeMap<String, Index>,
            #[serde(deserialize_with = "paths")]
            extra_paths: BTreeSet<String>,
        }
        let r = Raw::deserialize(d)?;
        let c = Self {
            architecture: r.architecture,
            repository_reachable: r.repository_reachable,
            published: r.published,
            archives: r.archives,
            publications: r.publications,
            extra_paths: r.extra_paths,
        };
        c.validate()
            .map_err(|_| D::Error::custom("Invalid bounded Debian catalog"))?;
        Ok(c)
    }
}
impl Catalog {
    pub fn validate(&self) -> Result<()> {
        if self.architecture == Architecture::All
            || self.archives.is_empty()
            || self.archives.len() > MAX_ARCHIVES
            || self.publications.is_empty()
            || self.publications.len() > 3
            || self.extra_paths.len() > 32
            || !id(&self.published)
        {
            return Err(error(ErrorCode::Limit, "Catalog shape/architecture bound"));
        }
        // All collection/scalar/aggregate bounds precede derived sets or cloning.
        let (mut rows, mut bytes) = (0usize, 0usize);
        for (k, a) in &self.archives {
            if !id(k)
                || !name(&a.name)
                || a.depends.len() > 8
                || a.files.len() > 32
                || parts(a.version.as_str()).is_err()
            {
                return Err(error(ErrorCode::Invalid, "Invalid archive metadata bound"));
            }
            rows = rows
                .checked_add(a.files.len())
                .filter(|n| *n <= 256)
                .ok_or_else(|| error(ErrorCode::Limit, "Catalog exceeds256 payload rows"))?;
            for (p, v) in &a.files {
                if !path(p) || !text(&v.contents, 2048) {
                    return Err(error(
                        ErrorCode::Invalid,
                        "Invalid bounded regular text payload",
                    ));
                }
                bytes = bytes
                    .checked_add(v.contents.len())
                    .filter(|n| *n <= MAX_TEXT)
                    .ok_or_else(|| error(ErrorCode::Limit, "Catalog text exceeds256KiB"))?;
            }
            for d in &a.depends {
                if !name(&d.name) {
                    return Err(error(ErrorCode::Invalid, "Invalid dependency name"));
                }
            }
        }
        for (k, index) in &self.publications {
            if !id(k)
                || index.candidates.len() > MAX_NAMES
                || index.candidates.iter().any(|(n, a)| !name(n) || !id(a))
            {
                return Err(error(ErrorCode::Invalid, "Invalid cached-index shape"));
            }
        }
        if self.extra_paths.iter().any(|p| !path(p)) {
            return Err(error(ErrorCode::Invalid, "Invalid supplied extra path"));
        }
        let mut names: BTreeMap<&str, usize> = BTreeMap::new();
        for a in self.archives.values() {
            let n = names.entry(&a.name).or_default();
            *n += 1;
            if *n > 3 || names.len() > MAX_NAMES {
                return Err(error(
                    ErrorCode::Limit,
                    "Use at most24 names and3 artifacts per name",
                ));
            }
        }
        for (i, a) in self.archives.values().enumerate() {
            for b in self.archives.values().take(i) {
                if a.name == b.name
                    && a.architecture == b.architecture
                    && a.version.compare(&b.version) == Ordering::Equal
                {
                    return Err(error(
                        ErrorCode::Invalid,
                        "Equivalent version identities are ambiguous",
                    ));
                }
            }
        }
        if !self.publications.contains_key(&self.published) {
            return Err(error(ErrorCode::Unknown, "Unknown published index"));
        }
        for index in self.publications.values() {
            for (n, a) in &index.candidates {
                if self.archives.get(a).is_none_or(|a| &a.name != n) {
                    return Err(error(
                        ErrorCode::Unknown,
                        "Index candidate must reference its named supplied archive",
                    ));
                }
            }
        }
        if self
            .extra_paths
            .iter()
            .any(|p| self.archives.values().any(|a| a.files.contains_key(p)))
        {
            return Err(error(
                ErrorCode::Invalid,
                "Extra unmanaged paths must not overlap payload paths",
            ));
        }
        if !within(self, 2_097_152) {
            return Err(error(ErrorCode::Limit, "Catalog compact JSON exceeds2MiB"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Installed,
    ConfigFiles,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageRecord {
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub archive: String,
    pub status: Status,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    #[serde(deserialize_with = "optional_id")]
    pub cached: Option<String>,
    #[serde(deserialize_with = "ids::<_,72>")]
    pub local_archives: BTreeSet<String>,
    #[serde(deserialize_with = "map::<_,PackageRecord,24,64>")]
    pub records: BTreeMap<String, PackageRecord>,
    #[serde(deserialize_with = "runtime_files")]
    pub files: BTreeMap<String, String>,
}
fn optional_id<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Option<String>, D::Error> {
    let s = Option::<Text<64>>::deserialize(d)?.map(|s| s.0);
    if s.as_ref().is_none_or(|s| id(s)) {
        Ok(s)
    } else {
        Err(D::Error::custom("Invalid cached index ID"))
    }
}
fn runtime_files<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    Ok(map::<D, Text<2048>, MAX_FILES, 160>(d)?
        .into_iter()
        .map(|(k, Text(v))| (k, v))
        .collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    catalog: Catalog,
    runtime: Runtime,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            catalog: Catalog,
            runtime: Runtime,
        }
        let r = Raw::deserialize(d)?;
        Self::new(r.catalog, r.runtime)
            .map_err(|_| D::Error::custom("Invalid bounded Debian state"))
    }
}
impl State {
    pub fn new(catalog: Catalog, runtime: Runtime) -> Result<Self> {
        let s = Self { catalog, runtime };
        s.validate()?;
        Ok(s)
    }
    pub fn empty(catalog: Catalog) -> Result<Self> {
        Self::new(
            catalog,
            Runtime {
                cached: None,
                local_archives: BTreeSet::new(),
                records: BTreeMap::new(),
                files: BTreeMap::new(),
            },
        )
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    pub fn validate_context(&self, original: &Catalog) -> Result<()> {
        self.validate()?;
        original.validate()?;
        if &self.catalog != original {
            return Err(error(
                ErrorCode::Invalid,
                "Package catalog/architecture/source context differs",
            ));
        }
        Ok(())
    }
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() > MAX_JSON {
            return Err(error(ErrorCode::Limit, "Package state JSON exceeds4MiB"));
        }
        serde_json::from_slice(input)
            .map_err(|_| error(ErrorCode::Invalid, "Invalid bounded Debian state JSON"))
    }
    pub fn validate(&self) -> Result<()> {
        self.catalog.validate()?;
        let r = &self.runtime;
        if r.local_archives.len() > MAX_ARCHIVES
            || r.records.len() > MAX_NAMES
            || r.files.len() > MAX_FILES
            || r.cached
                .as_ref()
                .is_some_and(|s| !id(s) || !self.catalog.publications.contains_key(s))
        {
            return Err(error(ErrorCode::Limit, "Runtime collection/index bound"));
        }
        if r.local_archives
            .iter()
            .any(|s| !id(s) || !self.catalog.archives.contains_key(s))
        {
            return Err(error(ErrorCode::Unknown, "Unknown local archive"));
        }
        let mut total = 0usize;
        for (p, v) in &r.files {
            if !path(p) || !text(v, 2048) {
                return Err(error(ErrorCode::Invalid, "Invalid runtime text/path"));
            }
            total = total
                .checked_add(v.len())
                .filter(|n| *n <= MAX_TEXT)
                .ok_or_else(|| error(ErrorCode::Limit, "Runtime text exceeds256KiB"))?;
        }
        for (n, v) in &r.records {
            if !name(n)
                || !id(&v.archive)
                || self.catalog.archives.get(&v.archive).is_none_or(|a| {
                    a.name != *n || !compatible(self.catalog.architecture, a.architecture)
                })
                || (v.status == Status::ConfigFiles
                    && self
                        .catalog
                        .archives
                        .get(&v.archive)
                        .is_some_and(|a| !a.files.values().any(|f| f.conffile)))
            {
                return Err(error(
                    ErrorCode::Invalid,
                    "Invalid installed record identity/architecture/config-only state",
                ));
            }
        }
        let allowed: BTreeSet<&str> = self
            .catalog
            .archives
            .values()
            .flat_map(|a| a.files.keys().map(String::as_str))
            .chain(self.catalog.extra_paths.iter().map(String::as_str))
            .collect();
        if r.files.keys().any(|p| !allowed.contains(p.as_str())) {
            return Err(error(
                ErrorCode::Unknown,
                "File path is outside the supplied virtual universe",
            ));
        }
        // Unmet Depends and absent/modified payloads are operational findings, not
        // structural import errors. Ambiguous multi-owner payloads are unsupported.
        owners(&self.catalog, &view_records(&r.records))?;
        if !within(self, MAX_JSON) {
            return Err(error(ErrorCode::Limit, "State compact JSON exceeds4MiB"));
        }
        Ok(())
    }
}
fn compatible(target: Architecture, artifact: Architecture) -> bool {
    artifact == Architecture::All || target == artifact
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RecordRef<'a> {
    archive: &'a str,
    status: Status,
}
fn view_records(records: &BTreeMap<String, PackageRecord>) -> BTreeMap<&str, RecordRef<'_>> {
    records
        .iter()
        .map(|(n, r)| {
            (
                n.as_str(),
                RecordRef {
                    archive: &r.archive,
                    status: r.status,
                },
            )
        })
        .collect()
}
fn owners<'a>(
    catalog: &'a Catalog,
    records: &BTreeMap<&'a str, RecordRef<'a>>,
) -> Result<BTreeMap<&'a str, &'a str>> {
    let mut out = BTreeMap::new();
    for (n, r) in records {
        let a = &catalog.archives[r.archive];
        for (p, v) in &a.files {
            if (r.status == Status::Installed || v.conffile) && out.insert(p.as_str(), *n).is_some()
            {
                return Err(error(
                    ErrorCode::Collision,
                    "Multiple package records claim the same supplied file; unsupported ownership state",
                ));
            }
        }
    }
    Ok(out)
}
fn satisfies(catalog: &Catalog, records: &BTreeMap<&str, RecordRef<'_>>, d: &Dependency) -> bool {
    records.get(d.name.as_str()).is_some_and(|r| {
        r.status == Status::Installed
            && d.constraint
                .as_ref()
                .is_none_or(|c| c.accepts(&catalog.archives[r.archive].version))
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub name: String,
    pub version: Option<Version>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    RefreshIndex {},
    Install {
        #[serde(deserialize_with = "list::<_,Request,8>")]
        packages: Vec<Request>,
    },
    Reinstall {
        #[serde(deserialize_with = "scalar::<_,64>")]
        name: String,
    },
    Remove {
        #[serde(deserialize_with = "scalar::<_,64>")]
        name: String,
    },
    Purge {
        #[serde(deserialize_with = "scalar::<_,64>")]
        name: String,
    },
}
impl Action {
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() > 16_384 {
            return Err(error(
                ErrorCode::Limit,
                "Logical package action exceeds16KiB",
            ));
        }
        let action: Self = serde_json::from_slice(input)
            .map_err(|_| error(ErrorCode::Invalid, "Invalid bounded typed package action"))?;
        action.validate()?;
        Ok(action)
    }
    fn validate(&self) -> Result<()> {
        let good = match self {
            Self::RefreshIndex {} => true,
            Self::Install { packages } => {
                !packages.is_empty()
                    && packages.len() <= 8
                    && packages.iter().all(|p| name(&p.name))
            }
            Self::Reinstall { name: n } | Self::Remove { name: n } | Self::Purge { name: n } => {
                name(n)
            }
        };
        if good {
            Ok(())
        } else {
            Err(error(
                ErrorCode::Limit,
                "Logical action identity/count bound",
            ))
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Change {
    pub package: String,
    pub before: Option<PackageRecord>,
    pub after: Option<PackageRecord>,
}
/// An explanatory result only. Apply takes an Action and recomputes; it cannot
/// accept this Plan as authority for current state or publication.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub packages: Vec<Change>,
    pub changed_files: Vec<String>,
    pub cached_before: Option<String>,
    pub cached_after: Option<String>,
    pub acquired_archives: Vec<String>,
}
struct Budget(usize);
impl Budget {
    fn charge(&mut self) -> Result<()> {
        if self.0 == 0 {
            return Err(error(ErrorCode::Limit, "Logical package work bound"));
        }
        self.0 -= 1;
        Ok(())
    }
}
struct Prepared<'a> {
    cached: Option<&'a str>,
    local: BTreeSet<&'a str>,
    records: BTreeMap<&'a str, RecordRef<'a>>,
    files: BTreeMap<&'a str, &'a str>,
}
impl Prepared<'_> {
    fn bounded(&self, catalog: &Catalog) -> Result<()> {
        if self.records.len() > MAX_NAMES
            || self.local.len() > MAX_ARCHIVES
            || self.files.len() > MAX_FILES
            || self
                .files
                .values()
                .try_fold(0usize, |n, v| n.checked_add(v.len()))
                .is_none_or(|n| n > MAX_TEXT)
        {
            return Err(error(
                ErrorCode::Limit,
                "Proposed runtime exceeds count/text bounds",
            ));
        }
        #[derive(Serialize)]
        struct R<'a, 'b> {
            cached: Option<&'a str>,
            local_archives: &'b BTreeSet<&'a str>,
            records: &'b BTreeMap<&'a str, RecordRef<'a>>,
            files: &'b BTreeMap<&'a str, &'a str>,
        }
        #[derive(Serialize)]
        struct S<'a, T> {
            catalog: &'a Catalog,
            runtime: T,
        }
        if !within(
            &S {
                catalog,
                runtime: R {
                    cached: self.cached,
                    local_archives: &self.local,
                    records: &self.records,
                    files: &self.files,
                },
            },
            MAX_JSON,
        ) {
            return Err(error(
                ErrorCode::Limit,
                "Proposed state exceeds compact JSON bound",
            ));
        }
        Ok(())
    }
    fn materialize(&self, catalog: &Catalog) -> Result<State> {
        self.bounded(catalog)?;
        State::new(
            catalog.clone(),
            Runtime {
                cached: self.cached.map(str::to_owned),
                local_archives: self.local.iter().map(|s| (*s).to_owned()).collect(),
                records: self
                    .records
                    .iter()
                    .map(|(n, r)| {
                        (
                            (*n).to_owned(),
                            PackageRecord {
                                archive: r.archive.into(),
                                status: r.status,
                            },
                        )
                    })
                    .collect(),
                files: self
                    .files
                    .iter()
                    .map(|(p, v)| ((*p).to_owned(), (*v).to_owned()))
                    .collect(),
            },
        )
    }
    fn report(&self, old: &State) -> Plan {
        let names: BTreeSet<_> = old
            .runtime
            .records
            .keys()
            .map(String::as_str)
            .chain(self.records.keys().copied())
            .collect();
        let packages = names
            .into_iter()
            .filter_map(|n| {
                let before = old.runtime.records.get(n);
                let after = self.records.get(n).map(|r| PackageRecord {
                    archive: r.archive.into(),
                    status: r.status,
                });
                if before == after.as_ref() {
                    None
                } else {
                    Some(Change {
                        package: n.into(),
                        before: before.cloned(),
                        after,
                    })
                }
            })
            .collect();
        let paths: BTreeSet<_> = old
            .runtime
            .files
            .keys()
            .map(String::as_str)
            .chain(self.files.keys().copied())
            .collect();
        let changed_files = paths
            .into_iter()
            .filter(|p| {
                old.runtime.files.get(*p).map(String::as_str) != self.files.get(*p).copied()
            })
            .map(str::to_owned)
            .collect();
        Plan {
            packages,
            changed_files,
            cached_before: old.runtime.cached.clone(),
            cached_after: self.cached.map(str::to_owned),
            acquired_archives: self
                .local
                .iter()
                .filter(|id| !old.runtime.local_archives.contains(**id))
                .map(|id| (*id).to_owned())
                .collect(),
        }
    }
}
impl State {
    fn candidate<'a>(&'a self, p: &Request) -> Result<&'a str> {
        let index = self.runtime.cached.as_ref().ok_or_else(|| {
            error(
                ErrorCode::Unavailable,
                "No cached package index; refresh the supplied source first",
            )
        })?;
        let id = self.catalog.publications[index]
            .candidates
            .get(&p.name)
            .ok_or_else(|| {
                error(
                    ErrorCode::Unavailable,
                    "Requested name is absent from the cached index",
                )
            })?;
        let a = &self.catalog.archives[id];
        if p.version
            .as_ref()
            .is_some_and(|v| v.compare(&a.version) != Ordering::Equal)
        {
            return Err(error(
                ErrorCode::Unavailable,
                "Requested version is not exposed by this cached index",
            ));
        }
        if self.runtime.records.get(&p.name).is_some_and(|old| {
            a.version
                .compare(&self.catalog.archives[&old.archive].version)
                .is_lt()
        }) {
            return Err(error(
                ErrorCode::Unsupported,
                "Downgrades need separately modeled policy; previous state retained",
            ));
        }
        if !compatible(self.catalog.architecture, a.architecture) {
            return Err(error(
                ErrorCode::Architecture,
                "Cached candidate has the wrong architecture",
            ));
        }
        Ok(id)
    }
    fn prepared<'a>(&'a self, action: &Action) -> Result<Prepared<'a>> {
        self.validate()?;
        action.validate()?;
        let mut budget = Budget(MAX_WORK);
        let mut p = Prepared {
            cached: self.runtime.cached.as_deref(),
            local: self
                .runtime
                .local_archives
                .iter()
                .map(String::as_str)
                .collect(),
            records: view_records(&self.runtime.records),
            files: self
                .runtime
                .files
                .iter()
                .map(|(p, v)| (p.as_str(), v.as_str()))
                .collect(),
        };
        match action {
            Action::RefreshIndex {} => {
                if !self.catalog.repository_reachable {
                    return Err(error(
                        ErrorCode::Unavailable,
                        "Supplied repository is unavailable; cached and installed data retained",
                    ));
                }
                p.cached = Some(&self.catalog.published);
            }
            Action::Install { packages } => {
                let mut selected: BTreeMap<&str, &str> = BTreeMap::new();
                let mut queue = VecDeque::new();
                for request in packages {
                    budget.charge()?;
                    let id = self.candidate(request)?;
                    let a = &self.catalog.archives[id];
                    if let Some(previous) = selected.insert(&a.name, id) {
                        if previous != id {
                            return Err(error(
                                ErrorCode::Dependency,
                                "Conflicting explicit package choices",
                            ));
                        }
                    } else {
                        queue.push_back(a.name.as_str());
                    }
                    p.records.insert(
                        &a.name,
                        RecordRef {
                            archive: id,
                            status: Status::Installed,
                        },
                    );
                }
                while let Some(n) = queue.pop_front() {
                    budget.charge()?;
                    let a = &self.catalog.archives[selected[n]];
                    for d in &a.depends {
                        budget.charge()?;
                        if satisfies(&self.catalog, &p.records, d) {
                            continue;
                        }
                        let id = self.candidate(&Request {
                            name: d.name.clone(),
                            version: None,
                        })?;
                        let candidate = &self.catalog.archives[id];
                        if d.constraint
                            .as_ref()
                            .is_some_and(|v| !v.accepts(&candidate.version))
                        {
                            return Err(error(
                                ErrorCode::Dependency,
                                "Cached candidate cannot satisfy a required version under this closed policy",
                            ));
                        }
                        if selected.get(d.name.as_str()).is_some_and(|old| *old != id) {
                            return Err(error(
                                ErrorCode::Dependency,
                                "One package cannot satisfy conflicting selected versions",
                            ));
                        }
                        if !selected.contains_key(d.name.as_str()) {
                            selected.insert(&candidate.name, id);
                            queue.push_back(candidate.name.as_str());
                        }
                        p.records.insert(
                            &candidate.name,
                            RecordRef {
                                archive: id,
                                status: Status::Installed,
                            },
                        );
                    }
                }
                for (n, r) in &p.records {
                    if r.status == Status::Installed {
                        for d in &self.catalog.archives[r.archive].depends {
                            budget.charge()?;
                            if !satisfies(&self.catalog, &p.records, d) {
                                return Err(error(
                                    ErrorCode::Dependency,
                                    format!(
                                        "{n}: a final dependency remains unsatisfied; no automatic peer removal or alternate solver"
                                    ),
                                ));
                            }
                        }
                    }
                }
                owners(&self.catalog, &p.records)?;
                for (n, id) in selected {
                    budget.charge()?;
                    if self
                        .runtime
                        .records
                        .get(n)
                        .is_some_and(|r| r.status == Status::Installed && r.archive == id)
                    {
                        continue;
                    }
                    self.acquire(&mut p, id)?;
                    self.install_files(&mut p, n, id, &mut budget)?;
                }
            }
            Action::Reinstall { name: n } => {
                let r = self
                    .runtime
                    .records
                    .get(n)
                    .filter(|r| r.status == Status::Installed)
                    .ok_or_else(|| {
                        error(
                            ErrorCode::Unavailable,
                            "Reinstall requires a currently installed record",
                        )
                    })?;
                self.acquire(&mut p, &r.archive)?;
                self.install_files(&mut p, n, &r.archive, &mut budget)?;
            }
            Action::Remove { name: n } | Action::Purge { name: n } => {
                let r = self.runtime.records.get(n).ok_or_else(|| {
                    error(ErrorCode::Unknown, "No package record for exact removal")
                })?;
                let a = &self.catalog.archives[&r.archive];
                let purge = matches!(action, Action::Purge { .. });
                if r.status == Status::Installed {
                    // Existing unrelated broken requirements stay representable, but a
                    // remaining installed dependent still blocks exact removal, even if its
                    // version constraint was already broken before this action.
                    p.records.remove(n.as_str());
                    for (other, record) in &p.records {
                        if record.status == Status::Installed {
                            for d in &self.catalog.archives[record.archive].depends {
                                budget.charge()?;
                                if d.name == *n {
                                    return Err(error(
                                        ErrorCode::Dependency,
                                        format!("{other} still requires the package being removed"),
                                    ));
                                }
                            }
                        }
                    }
                    if !purge && a.files.values().any(|v| v.conffile) {
                        p.records.insert(
                            &a.name,
                            RecordRef {
                                archive: &r.archive,
                                status: Status::ConfigFiles,
                            },
                        );
                    }
                }
                if purge {
                    p.records.remove(n.as_str());
                }
                for (path, payload) in &a.files {
                    budget.charge()?;
                    if (r.status == Status::Installed && (purge || !payload.conffile))
                        || (r.status == Status::ConfigFiles && purge && payload.conffile)
                    {
                        p.files.remove(path.as_str());
                    }
                }
            }
        }
        p.bounded(&self.catalog)?;
        Ok(p)
    }
    fn acquire<'a>(&'a self, p: &mut Prepared<'a>, id: &'a str) -> Result<()> {
        if p.local.contains(id) {
            return Ok(());
        }
        let exposed = self.runtime.cached.as_ref().is_some_and(|cache| {
            self.catalog.publications[cache]
                .candidates
                .values()
                .any(|v| v == id)
        });
        if !self.catalog.repository_reachable || !exposed {
            return Err(error(
                ErrorCode::Unavailable,
                "Exact artifact is not locally available or obtainable from this cached source",
            ));
        }
        p.local.insert(id);
        Ok(())
    }
    fn install_files<'a>(
        &'a self,
        p: &mut Prepared<'a>,
        n: &str,
        id: &'a str,
        budget: &mut Budget,
    ) -> Result<()> {
        let new = &self.catalog.archives[id];
        let old_record = self.runtime.records.get(n);
        let old = old_record.map(|r| &self.catalog.archives[&r.archive]);
        if let Some(old) = old {
            let before: BTreeSet<_> = old
                .files
                .iter()
                .filter(|(_, v)| v.conffile)
                .map(|(p, _)| p)
                .collect();
            let after: BTreeSet<_> = new
                .files
                .iter()
                .filter(|(_, v)| v.conffile)
                .map(|(p, _)| p)
                .collect();
            if before != after {
                return Err(error(
                    ErrorCode::Unsupported,
                    "Changing the conffile path set needs unmodeled obsolete-file handling",
                ));
            }
        }
        let previous_owners = owners(&self.catalog, &view_records(&self.runtime.records))?;
        if let Some(old) = old {
            for (path, payload) in &old.files {
                budget.charge()?;
                if !payload.conffile
                    && old_record.is_some_and(|r| r.status == Status::Installed)
                    && !new.files.contains_key(path)
                {
                    p.files.remove(path.as_str());
                }
            }
        }
        for (path, payload) in &new.files {
            budget.charge()?;
            if previous_owners
                .get(path.as_str())
                .is_some_and(|owner| *owner != n)
            {
                return Err(error(
                    ErrorCode::Collision,
                    "Another package owns a proposed payload path",
                ));
            }
            let current = self.runtime.files.get(path);
            if payload.conffile {
                if let Some(previous) = old.and_then(|a| a.files.get(path)) {
                    match current {
                        None => {
                            p.files.remove(path.as_str());
                        }
                        Some(current) if current == &previous.contents => {
                            p.files.insert(path, &payload.contents);
                        }
                        Some(_) if payload.contents == previous.contents => {}
                        Some(_) => {
                            return Err(error(
                                ErrorCode::Conffile,
                                "Both local and supplied conffile content changed; an unmodeled choice is required",
                            ));
                        }
                    }
                } else {
                    if current.is_some() {
                        return Err(error(
                            ErrorCode::Unsupported,
                            "Existing unowned conffile needs unmodeled adoption/prompt behavior",
                        ));
                    }
                    p.files.insert(path, &payload.contents);
                }
            } else {
                if current.is_some() && !previous_owners.contains_key(path.as_str()) {
                    return Err(error(
                        ErrorCode::Unsupported,
                        "Existing unowned payload collision is outside this subset",
                    ));
                }
                p.files.insert(path, &payload.contents);
            }
        }
        Ok(())
    }
    pub fn plan(&self, action: &Action) -> Result<Plan> {
        let p = self.prepared(action)?;
        Ok(p.report(self))
    }
    pub fn apply(&mut self, action: &Action) -> Result<Plan> {
        let p = self.prepared(action)?;
        let report = p.report(self);
        let next = p.materialize(&self.catalog)?;
        *self = next;
        Ok(report)
    }
    pub fn recorded_owner(&self, p: &str) -> Result<Option<Owner>> {
        self.validate()?;
        if !path(p) {
            return Err(error(
                ErrorCode::Invalid,
                "Use a bounded canonical absolute supplied path",
            ));
        }
        let records = view_records(&self.runtime.records);
        let o = owners(&self.catalog, &records)?;
        Ok(o.get(p).map(|n| Owner {
            package: (*n).into(),
            archive: records[n].archive.into(),
            status: records[n].status,
            present: self.runtime.files.contains_key(p),
        }))
    }
    pub fn archive(&self, archive_id: &str) -> Result<&Archive> {
        self.validate()?;
        if !id(archive_id) {
            return Err(error(ErrorCode::Invalid, "Invalid archive query identity"));
        }
        if !self.runtime.local_archives.contains(archive_id) {
            return Err(error(
                ErrorCode::Unavailable,
                "Archive is not in the supplied local archive cache",
            ));
        }
        self.catalog
            .archives
            .get(archive_id)
            .ok_or_else(|| error(ErrorCode::Unknown, "Unknown archive identity"))
    }
    pub fn cached_candidate(&self, n: &str) -> Result<Option<Candidate<'_>>> {
        self.validate()?;
        if !name(n) {
            return Err(error(ErrorCode::Invalid, "Invalid package query name"));
        }
        Ok(self
            .runtime
            .cached
            .as_ref()
            .and_then(|id| self.catalog.publications[id].candidates.get(n))
            .map(|id| {
                let a = &self.catalog.archives[id];
                Candidate {
                    archive: id,
                    name: &a.name,
                    version: &a.version,
                    architecture: a.architecture,
                    depends: &a.depends,
                }
            }))
    }
    pub fn dependency_issues(&self) -> Result<Vec<DependencyIssue>> {
        self.validate()?;
        let records = view_records(&self.runtime.records);
        let mut out = Vec::new();
        for (n, r) in &records {
            if r.status == Status::Installed {
                for d in &self.catalog.archives[r.archive].depends {
                    if !satisfies(&self.catalog, &records, d) {
                        out.push(DependencyIssue {
                            package: (*n).into(),
                            dependency: d.clone(),
                            installed: records
                                .get(d.name.as_str())
                                .filter(|r| r.status == Status::Installed)
                                .map(|r| self.catalog.archives[r.archive].version.clone()),
                        });
                    }
                }
            }
        }
        Ok(out)
    }
    pub fn payload_issues(&self, n: &str) -> Result<Vec<IntegrityIssue>> {
        self.validate()?;
        if !name(n) {
            return Err(error(
                ErrorCode::Invalid,
                "Invalid installed package query name",
            ));
        }
        let r = self
            .runtime
            .records
            .get(n)
            .filter(|r| r.status == Status::Installed)
            .ok_or_else(|| error(ErrorCode::Unknown, "No installed package to inspect"))?;
        Ok(self.catalog.archives[&r.archive]
            .files
            .iter()
            .filter(|(_, v)| !v.conffile)
            .filter_map(|(p, v)| {
                let kind = match self.runtime.files.get(p) {
                    None => Some(IntegrityKind::Missing),
                    Some(actual) if actual != &v.contents => Some(IntegrityKind::Changed),
                    _ => None,
                };
                kind.map(|kind| IntegrityIssue {
                    path: p.clone(),
                    kind,
                })
            })
            .collect())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Owner {
    pub package: String,
    pub archive: String,
    pub status: Status,
    pub present: bool,
}
pub struct Candidate<'a> {
    pub archive: &'a str,
    pub name: &'a str,
    pub version: &'a Version,
    pub architecture: Architecture,
    pub depends: &'a [Dependency],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DependencyIssue {
    pub package: String,
    pub dependency: Dependency,
    pub installed: Option<Version>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum IntegrityKind {
    Missing,
    Changed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IntegrityIssue {
    pub path: String,
    pub kind: IntegrityKind,
}
