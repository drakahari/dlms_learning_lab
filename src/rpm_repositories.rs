//! Bounded, typed repository/cache simulation. No CLI, transport or package execution.
use crate::rpm_packages as rpm;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, SeqAccess, Visitor},
};
use std::{
    collections::BTreeSet,
    fmt,
    io::{self, Write},
    marker::PhantomData,
};

pub const MAX_RAW: usize = 131_072;
pub const MAX_SOURCE: usize = 32_768;
pub const MAX_STATE: usize = 49_152;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Malformed,
    Limit,
    Context,
    UnknownAlias,
    UnknownSource,
    Disabled,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: Code,
    pub message: &'static str,
}
pub type Result<T> = std::result::Result<T, Error>;
fn fail<T>(code: Code, message: &'static str) -> Result<T> {
    Err(Error { code, message })
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
}
fn uri(s: &str) -> bool {
    s.len() <= 160
        && (s.starts_with("file:///") || s.starts_with("https://"))
        && s.len() > 8
        && s.bytes()
            .all(|b| b.is_ascii_graphic() && !b"\\\"<>".contains(&b))
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    struct V<const N: usize>;
    impl<'de, const N: usize> Visitor<'de> for V<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "text <= {N} bytes")
        }
        fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > N {
                return Err(E::custom("Repository text byte bound"));
            }
            Ok(s.to_owned())
        }
    }
    d.deserialize_str(V::<N>)
}
fn two<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct V<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most two repository items")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            struct Next<T> {
                allowed: bool,
                marker: PhantomData<T>,
            }
            impl<'de, T: Deserialize<'de>> de::DeserializeSeed<'de> for Next<T> {
                type Value = T;
                fn deserialize<D: Deserializer<'de>>(
                    self,
                    d: D,
                ) -> std::result::Result<T, D::Error> {
                    if !self.allowed {
                        return Err(de::Error::custom("Repository item count bound"));
                    }
                    T::deserialize(d)
                }
            }
            let mut out = Vec::new();
            while let Some(item) = a.next_element_seed(Next {
                allowed: out.len() < 2,
                marker: PhantomData,
            })? {
                out.push(item);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(V(PhantomData))
}
struct Counter {
    used: usize,
    max: usize,
}
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.used = self
            .used
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("size overflow"))?;
        if self.used > self.max {
            return Err(io::Error::other("repository compact byte bound"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn bounded<T: Serialize>(value: &T, max: usize) -> Result<()> {
    serde_json::to_writer(Counter { used: 0, max }, value).map_err(|_| Error {
        code: Code::Limit,
        message: "Repository serialized byte bound",
    })
}

/// Supplied delivery fact; it represents neither GPG trust nor a real transport probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    Available,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "text::<_,160>")]
    pub uri: String,
    pub delivery: Delivery,
    pub published: rpm::SnapshotId,
    #[serde(deserialize_with = "two")]
    pub history: Vec<rpm::SnapshotId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub schema_version: u8,
    pub catalog: rpm::Catalog,
    #[serde(deserialize_with = "two")]
    pub endpoints: Vec<Endpoint>,
}
impl Source {
    pub fn validate(&self) -> Result<()> {
        self.catalog.validate().map_err(|_| Error {
            code: Code::Malformed,
            message: "Invalid supplied RPM catalog",
        })?;
        if self.schema_version != 1 || self.endpoints.is_empty() || self.endpoints.len() > 2 {
            return fail(Code::Malformed, "Repository source version/count differs");
        }
        if self.catalog.archives.len() > 12
            || self.catalog.snapshots.len() > 2
            || self.catalog.allowed_paths.len() > 32
            || self.catalog.target != rpm::Architecture::X86_64
            || self
                .catalog
                .archives
                .values()
                .any(|a| a.identity.arch != rpm::Architecture::Noarch)
            || self
                .catalog
                .archives
                .values()
                .map(|a| &a.identity.name)
                .collect::<BTreeSet<_>>()
                .len()
                > 8
        {
            return fail(Code::Limit, "Repository app-scale RPM universe exceeded");
        }
        for (i, e) in self.endpoints.iter().enumerate() {
            if !name(&e.id)
                || !uri(&e.uri)
                || self.endpoints[..i]
                    .iter()
                    .any(|p| p.id == e.id || p.uri == e.uri)
                || e.history.is_empty()
                || e.history.len() > 2
                || !e.history.contains(&e.published)
                || e.history.iter().enumerate().any(|(n, h)| {
                    !self.catalog.snapshots.contains_key(h.as_str()) || e.history[..n].contains(h)
                })
            {
                return fail(
                    Code::Context,
                    "Duplicate/foreign repository endpoint or snapshot",
                );
            }
        }
        bounded(self, MAX_SOURCE)
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_RAW {
            return fail(Code::Limit, "Repository raw source byte bound");
        }
        let source: Self = serde_json::from_slice(raw).map_err(|_| Error {
            code: Code::Malformed,
            message: "Invalid bounded repository source JSON",
        })?;
        source.validate()?;
        Ok(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "text::<_,32>")]
    pub alias: String,
    #[serde(deserialize_with = "text::<_,160>")]
    pub uri: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cached {
    #[serde(deserialize_with = "text::<_,32>")]
    pub alias: String,
    #[serde(deserialize_with = "text::<_,32>")]
    pub endpoint: String,
    pub snapshot: rpm::SnapshotId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    schema_version: u8,
    source: Source,
    #[serde(deserialize_with = "two")]
    configurations: Vec<Configuration>,
    #[serde(deserialize_with = "two")]
    caches: Vec<Cached>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    SetEnabled { alias: String, enabled: bool },
    SetUri { alias: String, uri: String },
    Refresh { alias: String },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheStatus {
    Missing,
    WrongSource,
    Older,
    Current,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inspection<'a> {
    pub configuration: &'a Configuration,
    pub endpoint: Option<&'a Endpoint>,
    pub cached: Option<&'a Cached>,
    pub cache_status: CacheStatus,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageRow<'a> {
    pub alias: &'a str,
    pub endpoint: &'a str,
    pub snapshot: &'a rpm::SnapshotId,
    pub archive_id: &'a str,
    pub archive: &'a rpm::Archive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Omission {
    Disabled,
    UnknownSource,
    MissingCache,
    WrongSource,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query<'a> {
    pub rows: Vec<PackageRow<'a>>,
    pub omitted: Vec<(&'a str, Omission)>,
}
#[derive(Serialize)]
struct Projection<'a> {
    schema_version: u8,
    source: &'a Source,
    configurations: &'a [Configuration],
    caches: &'a [Cached],
}
impl State {
    pub fn new(
        source: &Source,
        configurations: Vec<Configuration>,
        caches: Vec<Cached>,
    ) -> Result<Self> {
        source.validate()?;
        Self::runtime_valid(source, &configurations, &caches)?;
        bounded(
            &Projection {
                schema_version: 1,
                source,
                configurations: &configurations,
                caches: &caches,
            },
            MAX_STATE,
        )?;
        Ok(Self {
            schema_version: 1,
            source: source.clone(),
            configurations,
            caches,
        })
    }
    fn runtime_valid(
        source: &Source,
        configurations: &[Configuration],
        caches: &[Cached],
    ) -> Result<()> {
        if configurations.is_empty() || configurations.len() > 2 || caches.len() > 2 {
            return fail(Code::Limit, "Repository configuration/cache count bound");
        }
        for (i, c) in configurations.iter().enumerate() {
            if !name(&c.alias)
                || !uri(&c.uri)
                || configurations[..i].iter().any(|p| p.alias == c.alias)
            {
                return fail(
                    Code::Malformed,
                    "Invalid/duplicate repository configuration",
                );
            }
        }
        for (i, c) in caches.iter().enumerate() {
            if !name(&c.alias)
                || !name(&c.endpoint)
                || !configurations.iter().any(|x| x.alias == c.alias)
                || caches[..i].iter().any(|p| p.alias == c.alias)
                || !source
                    .endpoints
                    .iter()
                    .any(|e| e.id == c.endpoint && e.history.contains(&c.snapshot))
            {
                return fail(Code::Context, "Duplicate/foreign repository cache identity");
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.source.validate()?;
        if self.schema_version != 1 {
            return fail(Code::Malformed, "Repository state version differs");
        }
        Self::runtime_valid(&self.source, &self.configurations, &self.caches)?;
        bounded(self, MAX_STATE)
    }
    pub fn validate_context(&self, source: &Source) -> Result<()> {
        source.validate()?;
        self.validate()?;
        if &self.source != source {
            return fail(
                Code::Context,
                "Whole repository source differs from the assignment",
            );
        }
        Ok(())
    }
    pub fn decode(raw: &[u8], source: &Source) -> Result<Self> {
        if raw.len() > MAX_RAW {
            return fail(Code::Limit, "Repository raw state byte bound");
        }
        let state: Self = serde_json::from_slice(raw).map_err(|_| Error {
            code: Code::Malformed,
            message: "Invalid bounded repository state JSON",
        })?;
        state.validate_context(source)?;
        Ok(state)
    }
    fn inspect_inner(&self, index: usize) -> Inspection<'_> {
        let configuration = &self.configurations[index];
        let endpoint = self
            .source
            .endpoints
            .iter()
            .find(|e| e.uri == configuration.uri);
        let cached = self.caches.iter().find(|c| c.alias == configuration.alias);
        let cache_status = match (cached, endpoint) {
            (None, _) => CacheStatus::Missing,
            (Some(c), Some(e)) if c.endpoint == e.id => {
                if c.snapshot == e.published {
                    CacheStatus::Current
                } else {
                    CacheStatus::Older
                }
            }
            _ => CacheStatus::WrongSource,
        };
        Inspection {
            configuration,
            endpoint,
            cached,
            cache_status,
        }
    }
    fn index(&self, alias: &str) -> Result<usize> {
        if !name(alias) {
            return fail(Code::Malformed, "Invalid repository alias");
        }
        self.configurations
            .iter()
            .position(|c| c.alias == alias)
            .ok_or(Error {
                code: Code::UnknownAlias,
                message: "Repository alias is not configured",
            })
    }
    pub fn inspect(&self, source: &Source, alias: &str) -> Result<Inspection<'_>> {
        self.validate_context(source)?;
        Ok(self.inspect_inner(self.index(alias)?))
    }
    pub fn inspect_all(&self, source: &Source) -> Result<Vec<Inspection<'_>>> {
        self.validate_context(source)?;
        Ok((0..self.configurations.len())
            .map(|i| self.inspect_inner(i))
            .collect())
    }
    /// Exact-name detailed repository rows. This is not a complete Zypper search command.
    pub fn query_exact(&self, source: &Source, package: &str) -> Result<Query<'_>> {
        self.validate_context(source)?;
        if !name(package) {
            return fail(Code::Malformed, "Invalid literal package name");
        }
        let mut query = Query {
            rows: Vec::new(),
            omitted: Vec::new(),
        };
        for i in 0..self.configurations.len() {
            let inspection = self.inspect_inner(i);
            let c = inspection.configuration;
            let reason = if !c.enabled {
                Some(Omission::Disabled)
            } else if inspection.endpoint.is_none() {
                Some(Omission::UnknownSource)
            } else {
                match inspection.cache_status {
                    CacheStatus::Missing => Some(Omission::MissingCache),
                    CacheStatus::WrongSource => Some(Omission::WrongSource),
                    CacheStatus::Older | CacheStatus::Current => None,
                }
            };
            if let Some(reason) = reason {
                query.omitted.push((&c.alias, reason));
                continue;
            }
            // Source and runtime validation above establish both references, including stale caches.
            if let (Some(cache), Some(endpoint)) = (inspection.cached, inspection.endpoint) {
                for archive_id in &self.source.catalog.snapshots[cache.snapshot.as_str()].archives {
                    let archive = &self.source.catalog.archives[archive_id];
                    if archive.identity.name == package {
                        query.rows.push(PackageRow {
                            alias: &c.alias,
                            endpoint: &endpoint.id,
                            snapshot: &cache.snapshot,
                            archive_id,
                            archive,
                        });
                    }
                }
            }
        }
        Ok(query)
    }
    pub fn apply(&mut self, source: &Source, action: &Action) -> Result<()> {
        self.validate_context(source)?;
        let alias = match action {
            Action::SetEnabled { alias, .. }
            | Action::SetUri { alias, .. }
            | Action::Refresh { alias } => alias,
        };
        let index = self.index(alias)?;
        // Default refresh excludes disabled sources even when explicitly named.
        // This is a typed simulator outcome, not a claimed Zypper exit status.
        if matches!(action, Action::Refresh { .. }) && !self.configurations[index].enabled {
            return fail(
                Code::Disabled,
                "Repository is disabled; default refresh left its cache unchanged. Enable it before retrying",
            );
        }
        if let Action::SetUri { uri: next, .. } = action
            && !uri(next)
        {
            return fail(Code::Malformed, "Invalid bounded repository URI text");
        }
        // Only two tiny records are cloned, never the immutable archive catalog or payloads.
        let mut configurations = self.configurations.clone();
        let mut caches = self.caches.clone();
        match action {
            Action::SetEnabled { enabled, .. } => configurations[index].enabled = *enabled,
            Action::SetUri { uri, .. } => configurations[index].uri.clone_from(uri),
            Action::Refresh { .. } => {
                let endpoint = source
                    .endpoints
                    .iter()
                    .find(|e| e.uri == configurations[index].uri)
                    .ok_or(Error {
                        code: Code::UnknownSource,
                        message: "URI has no supplied simulated repository",
                    })?;
                if endpoint.delivery != Delivery::Available {
                    return fail(
                        Code::Unavailable,
                        "Supplied repository is unavailable; existing cache retained",
                    );
                }
                let next = Cached {
                    alias: alias.clone(),
                    endpoint: endpoint.id.clone(),
                    snapshot: endpoint.published.clone(),
                };
                if let Some(old) = caches.iter_mut().find(|c| c.alias == *alias) {
                    *old = next;
                } else {
                    caches.push(next);
                }
            }
        }
        Self::runtime_valid(source, &configurations, &caches)?;
        bounded(
            &Projection {
                schema_version: 1,
                source,
                configurations: &configurations,
                caches: &caches,
            },
            MAX_STATE,
        )?;
        self.configurations = configurations;
        self.caches = caches;
        Ok(())
    }
}
