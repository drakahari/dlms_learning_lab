//! App admission, current outcomes and typed logical actions; no package CLI.
use crate::debian_packages::{self as core, Action, Catalog, Constraint, State, Status, Version};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{fmt, io::Write};
pub const MAX_APP_STATE_BYTES: usize = 262_144;
pub const MAX_DEFINITION_BYTES: usize = 524_288;
pub const MAX_DRAFT_BYTES: usize = 393_216;
pub const MAX_ACTION_BYTES: usize = 16_384;
pub const MAX_QUERY_BYTES: usize = 32_768;
pub const HELP: &str = "Use the logical package inventory, cached-index, supplied-archive, owner and payload queries, then explicitly perform a logical Refresh index, Install, Reinstall, Remove or Purge action. These restricted script-free actions are not dpkg/APT/RPM commands or real package transactions. Proposed action/query drafts do not change installed state. Only current package/file outcomes and expressly named preservation are graded.";
pub fn notice(v: impl AsRef<str>) -> String {
    let s = v.as_ref();
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    let mut out = String::with_capacity(n);
    out.push_str(&s[..n]);
    out
}
pub(crate) fn within<T: Serialize>(v: &T, cap: usize) -> bool {
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
                .ok_or_else(|| std::io::Error::other("Package app JSON bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, cap }, v).is_ok()
}
pub(crate) struct Text<const N: usize>(pub String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct T<const N: usize>;
        impl<const N: usize> Visitor<'_> for T<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} non-NUL UTF8 bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if s.len() <= N && !s.contains('\0') {
                    Ok(Text(s.into()))
                } else {
                    Err(E::custom("Package app text bound"))
                }
            }
        }
        d.deserialize_str(T::<N>)
    }
}
pub(crate) fn string<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    Ok(Text::<N>::deserialize(d)?.0)
}
pub(crate) fn list<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<V>, D::Error> {
    struct L<V, const N: usize>(std::marker::PhantomData<V>);
    struct Next<V> {
        allowed: bool,
        v: std::marker::PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("Package app list count"));
            }
            V::deserialize(d)
        }
    }
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for L<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} package app rows")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next::<V> {
                allowed: out.len() < N,
                v: std::marker::PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(L::<V, N>(std::marker::PhantomData))
}
pub(crate) fn action_bounded(a: &Action) -> bool {
    match a {
        Action::RefreshIndex {} => true,
        Action::Install { packages } => {
            !packages.is_empty()
                && packages.len() <= 8
                && packages.iter().all(|p| {
                    bounded_raw(&p.name, 64)
                        && p.version.as_ref().is_none_or(|v| v.as_str().len() <= 128)
                })
        }
        Action::Reinstall { name } | Action::Remove { name } | Action::Purge { name } => {
            bounded_raw(name, 64)
        }
    }
}
fn bounded_raw(s: &str, n: usize) -> bool {
    s.len() <= n && !s.chars().any(char::is_control)
}
fn package(c: &Catalog, n: &str) -> bool {
    n.len() <= 64 && c.archives.values().any(|a| a.name == n)
}
fn file(c: &Catalog, p: &str) -> bool {
    p.len() <= 160
        && (c.extra_paths.contains(p) || c.archives.values().any(|a| a.files.contains_key(p)))
}
pub fn admitted(s: &State) -> bool {
    s.validate().is_ok() && within(s, MAX_APP_STATE_BYTES)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Installed {
        #[serde(deserialize_with = "string::<_,64>")]
        package: String,
        constraint: Option<Constraint>,
    },
    RetainedConfig {
        #[serde(deserialize_with = "string::<_,64>")]
        package: String,
        version: Option<Version>,
    },
    NoRecord {
        #[serde(deserialize_with = "string::<_,64>")]
        package: String,
    },
    PayloadIntact {
        #[serde(deserialize_with = "string::<_,64>")]
        package: String,
    },
    DependenciesSatisfied {},
    FileMatches {
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
        #[serde(deserialize_with = "string::<_,2048>")]
        contents: String,
    },
    FileAbsent {
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
    },
    FilePreserved {
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
    },
    PackagePreserved {
        #[serde(deserialize_with = "string::<_,64>")]
        package: String,
    },
    Cached {
        #[serde(deserialize_with = "optional_name")]
        snapshot: Option<String>,
    },
    ArchiveAvailable {
        #[serde(deserialize_with = "string::<_,64>")]
        archive: String,
        available: bool,
    },
}
fn optional_name<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<Text<64>>::deserialize(d)?.map(|s| s.0))
}
fn meets(c: &Constraint, v: &Version) -> bool {
    let o = v.compare(&c.version);
    use core::Relation::*;
    match c.relation {
        Lt => o.is_lt(),
        Le => o.is_le(),
        Eq => o.is_eq(),
        Ge => o.is_ge(),
        Gt => o.is_gt(),
    }
}
impl Requirement {
    fn valid(&self, i: &State) -> bool {
        let c = i.catalog();
        match self {
            Self::Installed { package: n, .. }
            | Self::RetainedConfig { package: n, .. }
            | Self::NoRecord { package: n }
            | Self::PayloadIntact { package: n } => package(c, n),
            Self::DependenciesSatisfied {} => true,
            Self::FileMatches { path: p, contents } => {
                contents.len() <= 2048 && !contents.contains('\0') && file(c, p)
            }
            Self::FileAbsent { path: p } | Self::FilePreserved { path: p } => file(c, p),
            Self::PackagePreserved { package: n } => {
                package(c, n) && i.runtime().records.contains_key(n)
            }
            Self::Cached { snapshot } => snapshot
                .as_ref()
                .is_none_or(|s| s.len() <= 64 && c.publications.contains_key(s)),
            Self::ArchiveAvailable { archive, .. } => {
                archive.len() <= 64 && c.archives.contains_key(archive)
            }
        }
    }
    fn check(&self, i: &State, s: &State) -> (bool, String) {
        let r = s.runtime();
        let c = s.catalog();
        match self {
            Self::Installed {
                package,
                constraint,
            } => {
                let current = r.records.get(package);
                let ok = current.is_some_and(|p| {
                    p.status == Status::Installed
                        && constraint
                            .as_ref()
                            .is_none_or(|v| meets(v, &c.archives[&p.archive].version))
                });
                let detail = current.map_or("no record".into(), |p| {
                    format!(
                        "{:?}, version {}",
                        p.status,
                        c.archives[&p.archive].version.as_str()
                    )
                });
                (
                    ok,
                    format!(
                        "{package}: current {detail}; an installed record must meet the requested version constraint"
                    ),
                )
            }
            Self::RetainedConfig { package, version } => {
                let ok = r.records.get(package).is_some_and(|p| {
                    p.status == Status::ConfigFiles
                        && version
                            .as_ref()
                            .is_none_or(|v| v.compare(&c.archives[&p.archive].version).is_eq())
                });
                (
                    ok,
                    format!("{package}: retain the requested configuration-only package record"),
                )
            }
            Self::NoRecord { package } => (
                !r.records.contains_key(package),
                format!(
                    "{package}: no package database record is requested; file requirements are checked separately"
                ),
            ),
            Self::PayloadIntact { package } => match s.payload_issues(package) {
                Ok(v) if v.is_empty() => (
                    true,
                    format!("{package}: ordinary payload text matches its installed artifact"),
                ),
                Ok(v) => (
                    false,
                    format!(
                        "{package}: {} ordinary payload file(s) are missing or differ; inspect payload findings",
                        v.len()
                    ),
                ),
                Err(_) => (
                    false,
                    format!(
                        "{package}: an installed package is needed before ordinary payload integrity can pass"
                    ),
                ),
            },
            Self::DependenciesSatisfied {} => match s.dependency_issues() {
                Ok(v) => (
                    v.is_empty(),
                    format!("{} unsatisfied installed Depends entries", v.len()),
                ),
                Err(_) => (false, "Installed dependency context is invalid".into()),
            },
            Self::FileMatches { path, contents } => (
                r.files.get(path) == Some(contents),
                format!("{path}: current virtual text must match the explicitly requested content"),
            ),
            Self::FileAbsent { path } => (
                !r.files.contains_key(path),
                format!("{path}: the requested virtual file must be absent"),
            ),
            Self::FilePreserved { path } => (
                r.files.get(path) == i.runtime().files.get(path),
                format!("{path}: preserve its supplied content or supplied absence"),
            ),
            Self::PackagePreserved { package } => (
                r.records.get(package) == i.runtime().records.get(package),
                format!("{package}: preserve the supplied package record"),
            ),
            Self::Cached { snapshot } => (
                r.cached == *snapshot,
                format!(
                    "Cached metadata snapshot must be {}",
                    snapshot.as_deref().unwrap_or("absent")
                ),
            ),
            Self::ArchiveAvailable { archive, available } => (
                r.local_archives.contains(archive) == *available,
                format!(
                    "Local artifact {archive} must be {}",
                    if *available { "available" } else { "absent" }
                ),
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Definition {
    pub initial: State,
    pub requirements: Vec<Requirement>,
}
impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct R {
            initial: State,
            #[serde(deserialize_with = "list::<_,Requirement,32>")]
            requirements: Vec<Requirement>,
        }
        let r = R::deserialize(d)?;
        let v = Self {
            initial: r.initial,
            requirements: r.requirements,
        };
        v.validate().map_err(D::Error::custom)?;
        Ok(v)
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.requirements.is_empty() || self.requirements.len() > 32 || !admitted(&self.initial)
        {
            return Err("Package definition exceeds app state/requirement/JSON bounds".into());
        }
        // Direct scalar/identity checks precede serialization and comparisons.
        if !self.requirements.iter().all(|r| r.valid(&self.initial)) {
            return Err("Package requirement field or known identity is invalid".into());
        }
        if self
            .requirements
            .iter()
            .enumerate()
            .any(|(n, r)| self.requirements[..n].contains(r))
            || !self
                .requirements
                .iter()
                .any(|r| matches!(r, Requirement::Installed { .. }))
        {
            return Err("Package goals need known unique predicates and a positive installed-package outcome".into());
        }
        if !within(self, MAX_DEFINITION_BYTES) {
            return Err("Package definition exceeds compact JSON bounds".into());
        }
        Ok(())
    }
    pub fn same_assignment(&self, s: &State) -> bool {
        self.validate().is_ok() && admitted(s) && s.validate_context(self.initial.catalog()).is_ok()
    }
    pub fn checks(&self, s: &State) -> Vec<(bool, String)> {
        if !self.same_assignment(s) {
            return vec![(
                false,
                "Package assignment catalog or bounded current state differs".into(),
            )];
        }
        self.requirements
            .iter()
            .map(|r| {
                let (ok, text) = r.check(&self.initial, s);
                (ok, notice(text))
            })
            .collect()
    }
}
pub fn apply(d: &Definition, s: &mut State, a: &Action) -> Result<core::Plan, String> {
    if !d.same_assignment(s) {
        return Err("Foreign package catalog; current state retained".into());
    }
    if !action_bounded(a) {
        return Err("Package action exceeds bounded fields/count; original retained".into());
    }
    let mut candidate = s.clone();
    let report = candidate.apply(a).map_err(|e| notice(e.message))?;
    if !admitted(&candidate) {
        return Err(
            "Logical operation exceeds this app's state admission; original retained".into(),
        );
    }
    *s = candidate;
    Ok(report)
}
pub fn execute(d: &Definition, s: &mut State, input: &str) -> Result<String, String> {
    if input.len() > MAX_ACTION_BYTES {
        return Err("Package authoring action exceeds16KiB".into());
    }
    let action = input
        .strip_prefix("packages ")
        .ok_or("Use typed logical package controls, not a package-manager command alias")?;
    let action = Action::decode(action.as_bytes()).map_err(|e| notice(e.message))?;
    let p = apply(d, s, &action)?;
    Ok(notice(format!(
        "Logical operation complete: {} package records and {} virtual files changed; no host transaction",
        p.packages.len(),
        p.changed_files.len()
    )))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    RefreshIndex,
    Install,
    Reinstall,
    Remove,
    Purge,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryKind {
    Inventory,
    CachedCandidate,
    LocalArchive,
    RecordedOwner,
    Payload,
    Dependencies,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestDraft {
    #[serde(deserialize_with = "string::<_,64>")]
    pub package: String,
    #[serde(deserialize_with = "string::<_,128>")]
    pub version: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub version: u32,
    pub context: Catalog,
    pub operation: Operation,
    pub package: String,
    pub requests: Vec<RequestDraft>,
    pub query: QueryKind,
    pub query_argument: String,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct R {
            version: u32,
            context: Catalog,
            operation: Operation,
            #[serde(deserialize_with = "string::<_,64>")]
            package: String,
            #[serde(deserialize_with = "list::<_,RequestDraft,8>")]
            requests: Vec<RequestDraft>,
            query: QueryKind,
            #[serde(deserialize_with = "string::<_,160>")]
            query_argument: String,
        }
        let r = R::deserialize(d)?;
        let v = Self {
            version: r.version,
            context: r.context,
            operation: r.operation,
            package: r.package,
            requests: r.requests,
            query: r.query,
            query_argument: r.query_argument,
        };
        if !v.valid() {
            return Err(D::Error::custom(
                "Invalid bounded package action/query draft",
            ));
        }
        Ok(v)
    }
}
impl Draft {
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.requests.len() <= 8
            && bounded_raw(&self.package, 64)
            && bounded_raw(&self.query_argument, 160)
            && self
                .requests
                .iter()
                .all(|r| bounded_raw(&r.package, 64) && bounded_raw(&r.version, 128))
            && self.context.validate().is_ok()
            && within(&self.context, MAX_APP_STATE_BYTES)
            && within(self, MAX_DRAFT_BYTES)
    }
    pub fn valid_for(&self, s: &State) -> bool {
        self.valid() && admitted(s) && s.catalog() == &self.context
    }
    pub fn new(s: &State) -> Result<Self, String> {
        if !admitted(s) {
            return Err("Package state exceeds app admission".into());
        }
        let package = s
            .runtime()
            .records
            .keys()
            .next()
            .or_else(|| s.catalog().archives.values().next().map(|a| &a.name))
            .unwrap()
            .clone();
        let d = Self {
            version: 1,
            context: s.catalog().clone(),
            operation: Operation::Reinstall,
            package: package.clone(),
            requests: vec![RequestDraft {
                package,
                version: String::new(),
            }],
            query: QueryKind::Inventory,
            query_argument: String::new(),
        };
        if !d.valid_for(s) {
            return Err("Package draft aggregate bound".into());
        }
        Ok(d)
    }
    pub fn action(&self) -> Result<Action, String> {
        if !self.valid() {
            return Err("Invalid package draft bounds; previous state retained".into());
        }
        let a = match self.operation {
            Operation::RefreshIndex => Action::RefreshIndex {},
            Operation::Install => Action::Install {
                packages: self
                    .requests
                    .iter()
                    .map(|r| {
                        Ok(core::Request {
                            name: r.package.clone(),
                            version: if r.version.is_empty() {
                                None
                            } else {
                                Some(Version::parse(&r.version).map_err(|e| notice(e.message))?)
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            },
            Operation::Reinstall => Action::Reinstall {
                name: self.package.clone(),
            },
            Operation::Remove => Action::Remove {
                name: self.package.clone(),
            },
            Operation::Purge => Action::Purge {
                name: self.package.clone(),
            },
        };
        Action::decode(&serde_json::to_vec(&a).map_err(|_| "Package action encoding")?)
            .map_err(|e| notice(e.message))
    }
    pub fn fill(&mut self, s: &State, a: &Action) -> Result<(), String> {
        if !self.valid_for(s) {
            return Err("Foreign package action draft".into());
        }
        if !action_bounded(a) {
            return Err("Package action exceeds bounded fields/count".into());
        }
        Action::decode(&serde_json::to_vec(a).map_err(|_| "Package action encoding")?)
            .map_err(|e| notice(e.message))?;
        let mut next = self.clone();
        match a {
            Action::RefreshIndex {} => next.operation = Operation::RefreshIndex,
            Action::Install { packages } => {
                next.operation = Operation::Install;
                next.requests = packages
                    .iter()
                    .map(|r| RequestDraft {
                        package: r.name.clone(),
                        version: r
                            .version
                            .as_ref()
                            .map_or(String::new(), |v| v.as_str().into()),
                    })
                    .collect();
            }
            Action::Reinstall { name } => {
                next.operation = Operation::Reinstall;
                next.package = name.clone();
            }
            Action::Remove { name } => {
                next.operation = Operation::Remove;
                next.package = name.clone();
            }
            Action::Purge { name } => {
                next.operation = Operation::Purge;
                next.package = name.clone();
            }
        }
        if !next.valid_for(s) {
            return Err("Filled package action exceeds app admission".into());
        }
        *self = next;
        Ok(())
    }
    pub fn query(&self, s: &State) -> Result<QueryResult, String> {
        if !self.valid_for(s) {
            return Err("Foreign package query draft".into());
        }
        query(s, self.query, &self.query_argument)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackageRow {
    pub package: String,
    pub archive: String,
    pub status: Status,
    pub version: Version,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CandidateRow {
    pub archive: String,
    pub package: String,
    pub version: Version,
    pub architecture: core::Architecture,
    pub dependencies: Vec<core::Dependency>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum QueryResult {
    Inventory {
        cached: Option<String>,
        records: Vec<PackageRow>,
        local_archives: Vec<String>,
    },
    CachedCandidate {
        candidate: Option<CandidateRow>,
    },
    LocalArchive {
        archive: core::Archive,
    },
    RecordedOwner {
        owner: Option<core::Owner>,
    },
    Payload {
        findings: Vec<core::IntegrityIssue>,
    },
    Dependencies {
        findings: Vec<core::DependencyIssue>,
    },
}
pub fn query(s: &State, kind: QueryKind, arg: &str) -> Result<QueryResult, String> {
    if !bounded_raw(arg, 160) || !admitted(s) {
        return Err("Invalid package query bounds/context".into());
    }
    let r = s.runtime();
    let out = match kind {
        QueryKind::Inventory => QueryResult::Inventory {
            cached: r.cached.clone(),
            records: r
                .records
                .iter()
                .map(|(n, p)| PackageRow {
                    package: n.clone(),
                    archive: p.archive.clone(),
                    status: p.status,
                    version: s.catalog().archives[&p.archive].version.clone(),
                })
                .collect(),
            local_archives: r.local_archives.iter().cloned().collect(),
        },
        QueryKind::CachedCandidate => {
            let c = s.cached_candidate(arg).map_err(|e| notice(e.message))?;
            QueryResult::CachedCandidate {
                candidate: c.map(|c| CandidateRow {
                    archive: c.archive.into(),
                    package: c.name.into(),
                    version: c.version.clone(),
                    architecture: c.architecture,
                    dependencies: c.depends.to_vec(),
                }),
            }
        }
        QueryKind::LocalArchive => {
            let archive = s.archive(arg).map_err(|e| notice(e.message))?;
            #[derive(Serialize)]
            #[serde(tag = "kind")]
            enum Borrowed<'a> {
                LocalArchive { archive: &'a core::Archive },
            }
            if !within(&Borrowed::LocalArchive { archive }, MAX_QUERY_BYTES) {
                return Err("Package local-artifact query exceeds32KiB; no result cloned".into());
            }
            QueryResult::LocalArchive {
                archive: archive.clone(),
            }
        }
        QueryKind::RecordedOwner => QueryResult::RecordedOwner {
            owner: s.recorded_owner(arg).map_err(|e| notice(e.message))?,
        },
        QueryKind::Payload => QueryResult::Payload {
            findings: s.payload_issues(arg).map_err(|e| notice(e.message))?,
        },
        QueryKind::Dependencies => QueryResult::Dependencies {
            findings: s.dependency_issues().map_err(|e| notice(e.message))?,
        },
    };
    if !within(&out, MAX_QUERY_BYTES) {
        return Err("Package query exceeds32KiB result limit; no partial result published".into());
    }
    Ok(out)
}
