//! Bounded NFS application definitions and display sessions. No host operations.
use crate::nfs as core;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, io::Write, marker::PhantomData};
pub const MAX_DEFINITION: usize = 384 * 1024;
pub const MAX_SESSION: usize = 64 * 1024;
pub const MAX_SCENARIO: usize = 1024 * 1024;
pub const MAX_ATTEMPT: usize = 2 * 1024 * 1024;
pub fn notice(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].into()
}
struct Counter {
    used: usize,
    cap: usize,
}
impl Write for Counter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.used = self
            .used
            .checked_add(b.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| std::io::Error::other("NFS app compact limit"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn within<T: Serialize + ?Sized>(v: &T, cap: usize) -> bool {
    serde_json::to_writer(Counter { used: 0, cap }, v).is_ok()
}
pub(crate) fn text(s: &str, n: usize) -> bool {
    s.len() <= n && !s.contains('\0')
}
pub(crate) fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub(crate) fn path(s: &str, absolute: bool) -> bool {
    if s.is_empty() || s.len() > 160 || s.starts_with('/') != absolute {
        return false;
    }
    let tail = if absolute { &s[1..] } else { s };
    !tail.is_empty()
        && tail.split('/').count() <= 8
        && tail.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        })
}
pub(crate) fn string<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct S<const N: usize>;
    impl<const N: usize> Visitor<'_> for S<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "non-NUL text <= {N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if text(s, N) {
                Ok(s.into())
            } else {
                Err(E::custom("NFS app text bound"))
            }
        }
    }
    d.deserialize_str(S::<N>)
}
struct Item<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Item<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom(
                "NFS app collection limit before typed body",
            ));
        }
        T::deserialize(d)
    }
}
pub(crate) fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct L<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for L<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "<= {N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<T>, A::Error> {
            let mut v = Vec::new();
            while let Some(x) = a.next_element_seed(Item {
                allowed: v.len() < N,
                marker: PhantomData,
            })? {
                v.push(x);
            }
            Ok(v)
        }
    }
    d.deserialize_seq(L::<T, N>(PhantomData))
}
fn unique<T: PartialEq>(v: &[T]) -> bool {
    v.iter().enumerate().all(|(n, x)| !v[..n].contains(x))
}
fn client<'a>(i: &'a core::Inventory, name: &str) -> Option<&'a core::Client> {
    i.clients.iter().find(|c| c.id == name)
}
fn target(i: &core::Inventory, c: &str, m: &str) -> bool {
    id(c) && path(m, true) && client(i, c).is_some_and(|c| c.mountpoints.iter().any(|p| p == m))
}
fn server(i: &core::Inventory, s: &str) -> bool {
    id(s) && i.servers.iter().any(|v| v.id == s)
}
fn file(i: &core::Inventory, s: &str, p: &str) -> bool {
    server(i, s)
        && path(p, true)
        && i.roots
            .iter()
            .any(|r| r.server == s && r.files.iter().any(|f| f.path == p))
}
fn spec(i: &core::Inventory, c: &str, m: &core::MountSpec) -> bool {
    target(i, c, &m.mountpoint)
        && server(i, &m.server)
        && path(&m.root, true)
        && i.roots
            .iter()
            .any(|r| r.server == m.server && r.path == m.root)
}
fn access(i: &core::Inventory, c: &str, m: &str, p: &str, relative: &str) -> bool {
    target(i, c, m)
        && id(p)
        && client(i, c).is_some_and(|c| c.profiles.iter().any(|n| n == p))
        && path(relative, false)
        && i.roots.iter().any(|r| {
            r.files.iter().any(|f| {
                f.path
                    .strip_prefix(&r.path)
                    .is_some_and(|s| s.strip_prefix('/') == Some(relative))
            })
        })
}
fn document(s: &str) -> bool {
    text(s, core::MAX_DOCUMENT)
        && s.lines().all(|l| l.len() <= 512)
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\t' | '\r' | '\n'))
        && s.as_bytes()
            .iter()
            .enumerate()
            .all(|(n, b)| *b != b'\r' || s.as_bytes().get(n + 1) == Some(&b'\n'))
}
/// Borrowed scalar/context preflight before counting, parsing or candidate copies.
pub(crate) fn action_shape(i: &core::Inventory, a: &core::Action) -> bool {
    match a {
        core::Action::SetExportsDraft { server: s, text } => server(i, s) && document(text),
        core::Action::SaveExportsDraft { server: s }
        | core::Action::ApplySavedExports { server: s } => server(i, s),
        core::Action::SetFstabDraft { client: c, text } => {
            id(c) && client(i, c).is_some() && document(text)
        }
        core::Action::SaveFstabDraft { client: c } => id(c) && client(i, c).is_some(),
        core::Action::MountExplicit {
            client: c,
            specification,
        } => spec(i, c, specification),
        core::Action::MountSaved {
            client: c,
            mountpoint,
        }
        | core::Action::Unmount {
            client: c,
            mountpoint,
        } => target(i, c, mountpoint),
        core::Action::ReadExisting {
            client: c,
            mountpoint,
            profile,
            relative,
        } => access(i, c, mountpoint, profile, relative),
        core::Action::WriteExisting {
            client: c,
            mountpoint,
            profile,
            relative,
            text: t,
        } => text(t, core::MAX_FILE) && access(i, c, mountpoint, profile, relative),
    }
}
fn expected_shape(e: &core::Expected) -> bool {
    match e {
        core::Expected::Read { text: t } => text(t, core::MAX_FILE),
        _ => true,
    }
}
fn request_shape(i: &core::Inventory, r: &core::Request, e: &core::Expected) -> bool {
    match r {
        core::Request::Read {
            client,
            mountpoint,
            profile,
            relative,
        } => {
            access(i, client, mountpoint, profile, relative)
                && matches!(
                    e,
                    core::Expected::Read { .. } | core::Expected::Denied { .. }
                )
                && expected_shape(e)
        }
        core::Request::WritePermission {
            client,
            mountpoint,
            profile,
            relative,
        } => {
            access(i, client, mountpoint, profile, relative)
                && matches!(e, core::Expected::Write {} | core::Expected::Denied { .. })
        }
        core::Request::MountPresent { .. } => false,
    }
}
fn trial_shape(i: &core::Inventory, t: &core::Trial) -> bool {
    if t.steps.is_empty() || t.steps.len() > 16 {
        return false;
    }
    if let core::TrialBase::Saved { servers, client: c } = &t.base
        && (servers.is_empty()
            || servers.len() > 2
            || !unique(servers)
            || !servers.iter().all(|s| server(i, s))
            || !id(c)
            || client(i, c).is_none())
    {
        return false;
    }
    t.steps.iter().all(|s| {
        action_shape(i, &s.action)
            && expected_shape(&s.expected)
            && matches!(
                s.action,
                core::Action::MountExplicit { .. }
                    | core::Action::MountSaved { .. }
                    | core::Action::Unmount { .. }
                    | core::Action::ReadExisting { .. }
                    | core::Action::WriteExisting { .. }
            )
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    CurrentMount {
        #[serde(deserialize_with = "string::<_,32>")]
        client: String,
        specification: core::MountSpec,
    },
    NoCurrentMount {
        #[serde(deserialize_with = "string::<_,32>")]
        client: String,
        #[serde(deserialize_with = "string::<_,160>")]
        mountpoint: String,
    },
    CurrentFileText {
        #[serde(deserialize_with = "string::<_,32>")]
        server: String,
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
        #[serde(deserialize_with = "string::<_,4096>")]
        text: String,
    },
    CurrentAccess {
        request: core::Request,
        expected: core::Expected,
    },
    ServerDocumentsCommitted {
        #[serde(deserialize_with = "string::<_,32>")]
        server: String,
    },
    ClientSavedMount {
        #[serde(deserialize_with = "string::<_,32>")]
        client: String,
        specification: core::MountSpec,
    },
    ClientSavedMountAbsent {
        #[serde(deserialize_with = "string::<_,32>")]
        client: String,
        #[serde(deserialize_with = "string::<_,160>")]
        mountpoint: String,
    },
    PreserveInitialMount {
        #[serde(deserialize_with = "string::<_,32>")]
        client: String,
        #[serde(deserialize_with = "string::<_,160>")]
        mountpoint: String,
    },
    PreserveInitialFile {
        #[serde(deserialize_with = "string::<_,32>")]
        server: String,
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
    },
    PreserveInitialServerLayers {
        #[serde(deserialize_with = "string::<_,32>")]
        server: String,
    },
    PreserveInitialClientLayers {
        #[serde(deserialize_with = "string::<_,32>")]
        client: String,
    },
}
fn mount<'a>(s: &'a core::State, c: &str, p: &str) -> Option<&'a core::Mount> {
    s.runtime()
        .mounts
        .iter()
        .find(|m| m.client == c && m.specification.mountpoint == p)
}
fn contents<'a>(s: &'a core::State, server: &str, p: &str) -> Option<&'a str> {
    s.runtime()
        .contents
        .iter()
        .find(|f| f.server == server && f.path == p)
        .map(|f| f.text.as_str())
}
impl Requirement {
    fn valid(&self, s: &core::State) -> bool {
        let i = s.inventory();
        match self {
            Self::CurrentMount {
                client,
                specification,
            }
            | Self::ClientSavedMount {
                client,
                specification,
            } => spec(i, client, specification),
            Self::NoCurrentMount { client, mountpoint }
            | Self::ClientSavedMountAbsent { client, mountpoint } => target(i, client, mountpoint),
            Self::PreserveInitialMount { client, mountpoint } => {
                target(i, client, mountpoint) && mount(s, client, mountpoint).is_some()
            }
            Self::CurrentFileText {
                server: s,
                path: p,
                text: t,
            } => text(t, core::MAX_FILE) && file(i, s, p),
            Self::PreserveInitialFile { server: s, path: p } => file(i, s, p),
            Self::CurrentAccess { request, expected } => request_shape(i, request, expected),
            Self::ServerDocumentsCommitted { server: s }
            | Self::PreserveInitialServerLayers { server: s } => server(i, s),
            Self::PreserveInitialClientLayers { client: c } => id(c) && client(i, c).is_some(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Definition {
    pub initial: core::State,
    pub requirements: Vec<Requirement>,
    pub fresh: Vec<core::Trial>,
}
impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            initial: core::State,
            #[serde(deserialize_with = "list::<_,Requirement,32>")]
            requirements: Vec<Requirement>,
            #[serde(deserialize_with = "list::<_,core::Trial,4>")]
            fresh: Vec<core::Trial>,
        }
        let r = Raw::deserialize(d)?;
        let v = Self {
            initial: r.initial,
            requirements: r.requirements,
            fresh: r.fresh,
        };
        v.validate().map_err(D::Error::custom)?;
        Ok(v)
    }
}
fn denial(stage: core::DenialStage) -> &'static str {
    match stage {
        core::DenialStage::MountAbsent => "no current mount is present",
        core::DenialStage::ExportAdmission => "the export does not admit this client",
        core::DenialStage::ClientReadOnly => "the client mount is read-only",
        core::DenialStage::ExportReadOnly => "the export is read-only",
        core::DenialStage::DirectorySearch => "directory search permission is denied",
        core::DenialStage::FilePermission => "file permissions deny this operation",
    }
}
/// Actual decision matching only; faults are deliberately not accepted here.
pub fn matches(e: &core::Expected, d: &core::Decision) -> bool {
    match (e, d) {
        (core::Expected::Mounted {}, core::Decision::Mounted {})
        | (core::Expected::Unmounted {}, core::Decision::Unmounted {})
        | (core::Expected::Write {}, core::Decision::AllowedWrite { .. }) => true,
        (core::Expected::Read { text: a }, core::Decision::AllowedRead { text: b, .. }) => a == b,
        (core::Expected::Denied { stage: a }, core::Decision::Denied { stage: b, .. }) => a == b,
        _ => false,
    }
}
fn positive_fresh(t: &core::Trial) -> bool {
    t.steps.iter().any(|s| {
        matches!(
            (&s.action, &s.expected),
            (
                core::Action::ReadExisting { .. },
                core::Expected::Read { .. }
            ) | (core::Action::WriteExisting { .. }, core::Expected::Write {})
        )
    })
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("NFS definition raw limit".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NFS definition".into())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.requirements.is_empty()
            || self.requirements.len() > 32
            || self.fresh.is_empty()
            || self.fresh.len() > 4
            || !self.requirements.iter().all(|r| r.valid(&self.initial))
            || !self
                .fresh
                .iter()
                .all(|t| trial_shape(self.initial.inventory(), t))
        {
            return Err("NFS goal scalar/count/context differs".into());
        }
        let positive_read = self.requirements.iter().any(|r| {
            matches!(
                r,
                Requirement::CurrentAccess {
                    request: core::Request::Read { .. },
                    expected: core::Expected::Read { .. }
                }
            )
        });
        let mounted = self
            .requirements
            .iter()
            .any(|r| matches!(r, Requirement::CurrentMount { .. }));
        let published = self
            .requirements
            .iter()
            .any(|r| matches!(r, Requirement::CurrentFileText { .. }));
        if !(positive_read || (mounted && published)) || !self.fresh.iter().any(positive_fresh) {
            return Err("NFS goal needs useful current and fresh file outcomes".into());
        }
        #[derive(Serialize)]
        struct Plan<'a> {
            current: Vec<&'a core::Request>,
            fresh: &'a [core::Trial],
        }
        let plan = Plan {
            current: self
                .requirements
                .iter()
                .filter_map(|r| {
                    if let Requirement::CurrentAccess { request, .. } = r {
                        Some(request)
                    } else {
                        None
                    }
                })
                .collect(),
            fresh: &self.fresh,
        };
        if !within(&plan, core::MAX_ACTION)
            || !within(self, MAX_DEFINITION)
            || !unique(&self.requirements)
            || !unique(&self.fresh)
        {
            return Err("NFS goal aggregate/duplicate bound".into());
        }
        self.initial.validate().map_err(|e| notice(e.message))
    }
    pub fn same_assignment(&self, s: &core::State) -> bool {
        self.initial.inventory() == s.inventory()
    }
    pub fn row_count(&self) -> usize {
        1 + self.requirements.len() + self.fresh.len()
    }
    pub fn checks(&self, s: &core::State) -> Vec<(bool, String)> {
        let fail = |m: &str| vec![(false, notice(m))];
        if self.validate().is_err() || !self.same_assignment(s) {
            return fail("NFS authored inventory or goal differs");
        }
        // Cache only bounded saved parses relevant to direct rows. Core owns independent fresh trials.
        let mut exports = BTreeMap::new();
        let mut mounts = BTreeMap::new();
        for r in &self.requirements {
            match r {
                Requirement::ServerDocumentsCommitted { server } => {
                    if let Some(l) = s.runtime().servers.iter().find(|l| l.server == *server) {
                        exports.entry(server.as_str()).or_insert_with(|| {
                            core::parse_exports(s.inventory(), server, &l.saved)
                        });
                    }
                }
                Requirement::ClientSavedMount { client, .. }
                | Requirement::ClientSavedMountAbsent { client, .. } => {
                    if let Some(l) = s.runtime().clients.iter().find(|l| l.client == *client) {
                        mounts
                            .entry(client.as_str())
                            .or_insert_with(|| core::parse_fstab(s.inventory(), client, &l.saved));
                    }
                }
                _ => {}
            }
        }
        // All copied request fields are already shape/count/compact admitted; at most32+4*16.
        let plan = core::Evaluation {
            current: self
                .requirements
                .iter()
                .filter_map(|r| {
                    if let Requirement::CurrentAccess { request, .. } = r {
                        Some(request.clone())
                    } else {
                        None
                    }
                })
                .collect(),
            fresh: self.fresh.clone(),
        };
        let report = match s.evaluate_batch(&plan) {
            Ok(r) => r,
            Err(e) => return fail(&e.message),
        };
        let mut rows = vec![(true, "Current NFS context is valid".into())];
        let mut access_index = 0;
        for r in &self.requirements {
            let (ok, message) = match r {
                Requirement::CurrentMount {
                    client,
                    specification,
                } => (
                    mount(s, client, &specification.mountpoint)
                        .is_some_and(|m| m.specification == *specification),
                    format!("Required current mount for {client} is present"),
                ),
                Requirement::NoCurrentMount { client, mountpoint } => (
                    mount(s, client, mountpoint).is_none(),
                    format!("Retired current mount for {client} is absent"),
                ),
                Requirement::CurrentFileText { server, path, text } => (
                    contents(s, server, path) == Some(text.as_str()),
                    format!("Required current file contents at {server}:{path} are present"),
                ),
                Requirement::CurrentAccess { expected, .. } => {
                    let o = &report.current[access_index];
                    access_index += 1;
                    let ok = match o {
                        core::Observation::Decision { decision } => matches(expected, decision),
                        core::Observation::Fault { .. } => false,
                    };
                    let message = match o {
                        core::Observation::Fault { fault } => format!(
                            "Current request is unsupported or invalid: {}",
                            fault.message
                        ),
                        core::Observation::Decision {
                            decision: core::Decision::Denied { stage, .. },
                        } => format!("Current access: {}",denial(*stage)),
                        core::Observation::Decision { decision:core::Decision::AllowedRead{..} } if !ok=>"The current read returned different file contents or was expected to be denied".into(),
                        core::Observation::Decision { decision:core::Decision::AllowedWrite{..} } if !ok=>"The current write is permitted but this request was expected to be denied".into(),
                        _ => "Current access has the required outcome".into(),
                    };
                    (ok, message)
                }
                Requirement::ServerDocumentsCommitted { server } => {
                    let l = s.runtime().servers.iter().find(|l| l.server == *server);
                    let ok = l.is_some_and(|l| {
                        l.draft == l.saved
                            && exports.get(server.as_str()).is_some_and(|r| {
                                r.as_ref().is_ok_and(|e| same_exports(e, &l.applied))
                            })
                    });
                    (ok, format!("Exports for {server} are saved and applied"))
                }
                Requirement::ClientSavedMount {
                    client,
                    specification,
                } => {
                    let l = s.runtime().clients.iter().find(|l| l.client == *client);
                    let ok = l.is_some_and(|l| l.draft == l.saved)
                        && mounts
                            .get(client.as_str())
                            .is_some_and(|r| r.as_ref().is_ok_and(|m| m.contains(specification)));
                    (
                        ok,
                        format!("Saved mount for {client} has the required source and mode"),
                    )
                }
                Requirement::ClientSavedMountAbsent { client, mountpoint } => {
                    let l = s.runtime().clients.iter().find(|l| l.client == *client);
                    let ok = l.is_some_and(|l| l.draft == l.saved)
                        && mounts.get(client.as_str()).is_some_and(|r| {
                            r.as_ref()
                                .is_ok_and(|m| m.iter().all(|s| s.mountpoint != *mountpoint))
                        });
                    (ok, format!("Retired saved mount for {client} is absent"))
                }
                Requirement::PreserveInitialMount { client, mountpoint } => (
                    mount(&self.initial, client, mountpoint)
                        .is_some_and(|m| mount(s, client, mountpoint) == Some(m)),
                    format!("Initial mount for {client} is preserved"),
                ),
                Requirement::PreserveInitialFile { server, path } => (
                    contents(&self.initial, server, path) == contents(s, server, path),
                    format!("Initial file at {server}:{path} is preserved"),
                ),
                Requirement::PreserveInitialServerLayers { server } => (
                    self.initial
                        .runtime()
                        .servers
                        .iter()
                        .find(|l| l.server == *server)
                        == s.runtime().servers.iter().find(|l| l.server == *server),
                    format!("Initial export layers for {server} are preserved"),
                ),
                Requirement::PreserveInitialClientLayers { client } => (
                    self.initial
                        .runtime()
                        .clients
                        .iter()
                        .find(|l| l.client == *client)
                        == s.runtime().clients.iter().find(|l| l.client == *client),
                    format!("Initial saved and draft mounts for {client} are preserved"),
                ),
            };
            rows.push((
                ok,
                notice(if ok {
                    format!("Verified: {message}")
                } else {
                    format!("Needs attention: {message}")
                }),
            ));
        }
        for (n, t) in report.fresh.iter().enumerate() {
            rows.push((
                t.passed,
                notice(format!(
                    "New-mount test {} {}",
                    n + 1,
                    if t.passed {
                        "has the required outcomes"
                    } else {
                        "does not yet have the required outcomes"
                    }
                )),
            ));
        }
        rows
    }
}
fn same_exports(a: &[core::Export], b: &[core::Export]) -> bool {
    a.len() == b.len()
        && a.iter().all(|x| {
            b.iter().any(|y| {
                x.root == y.root
                    && x.clients.len() == y.clients.len()
                    && x.clients.iter().all(|c| y.clients.contains(c))
            })
        })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "string::<_,160>")]
    pub operation: String,
    #[serde(deserialize_with = "string::<_,4096>")]
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Session {
    pub version: u32,
    pub server: String,
    pub client: String,
    pub mountpoint: String,
    pub profile: String,
    pub relative: String,
    pub write_text: String,
    pub entries: Vec<Entry>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            version: 1,
            server: String::new(),
            client: String::new(),
            mountpoint: String::new(),
            profile: String::new(),
            relative: String::new(),
            write_text: String::new(),
            entries: Vec::new(),
        }
    }
}
impl<'de> Deserialize<'de> for Session {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            #[serde(deserialize_with = "string::<_,32>")]
            server: String,
            #[serde(deserialize_with = "string::<_,32>")]
            client: String,
            #[serde(deserialize_with = "string::<_,160>")]
            mountpoint: String,
            #[serde(deserialize_with = "string::<_,32>")]
            profile: String,
            #[serde(deserialize_with = "string::<_,160>")]
            relative: String,
            #[serde(deserialize_with = "string::<_,4096>")]
            write_text: String,
            #[serde(deserialize_with = "list::<_,Entry,16>")]
            entries: Vec<Entry>,
        }
        let r = Raw::deserialize(d)?;
        let s = Self {
            version: r.version,
            server: r.server,
            client: r.client,
            mountpoint: r.mountpoint,
            profile: r.profile,
            relative: r.relative,
            write_text: r.write_text,
            entries: r.entries,
        };
        if !s.valid() {
            return Err(D::Error::custom("NFS Session bound"));
        }
        Ok(s)
    }
}
impl Session {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("NFS Session raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NFS Session".into())
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && text(&self.server, 32)
            && text(&self.client, 32)
            && text(&self.mountpoint, 160)
            && text(&self.profile, 32)
            && text(&self.relative, 160)
            && text(&self.write_text, 4096)
            && self.entries.len() <= 16
            && self.entries.iter().all(|e| {
                text(&e.operation, 160) && !e.operation.trim().is_empty() && text(&e.detail, 4096)
            })
            && within(self, MAX_SESSION)
    }
    pub fn valid_for(&self, s: &core::State) -> bool {
        self.valid()
            && (self.server.is_empty() || server(s.inventory(), &self.server))
            && (self.client.is_empty() || client(s.inventory(), &self.client).is_some())
            && (self.profile.is_empty()
                || client(s.inventory(), &self.client)
                    .is_some_and(|c| c.profiles.contains(&self.profile)))
            && (self.mountpoint.is_empty() || target(s.inventory(), &self.client, &self.mountpoint))
    }
    fn recorded(&self, a: &core::Action, d: &core::Decision) -> Result<Self, String> {
        let operation = match a {
            core::Action::SetExportsDraft { .. } => "Edit exports",
            core::Action::SaveExportsDraft { .. } => "Save exports",
            core::Action::ApplySavedExports { .. } => "Apply exports",
            core::Action::SetFstabDraft { .. } => "Edit saved-mount draft",
            core::Action::SaveFstabDraft { .. } => "Save mount document",
            core::Action::MountExplicit { .. } => "Mount explicitly",
            core::Action::MountSaved { .. } => "Mount saved entry",
            core::Action::Unmount { .. } => "Unmount",
            core::Action::ReadExisting { .. } => "Read existing file",
            core::Action::WriteExisting { .. } => "Write existing file",
        };
        let generated;
        let detail = if let core::Decision::AllowedRead { text, .. } = d {
            text.as_str()
        } else {
            generated = match d {
                core::Decision::Denied { stage, .. } => {
                    format!("Simulated access denied: {}", denial(*stage))
                }
                core::Decision::AllowedWrite { .. } => "Existing virtual file written".into(),
                core::Decision::Mounted {} => "Virtual mount is present".into(),
                core::Decision::Unmounted {} => "Virtual mount removed".into(),
                core::Decision::Changed { changed } => {
                    format!("Declared operation changed data: {changed}")
                }
                core::Decision::AllowedRead { .. } => unreachable!("borrowed read above"),
            };
            &generated
        };
        #[derive(Serialize)]
        struct NewEntry<'a> {
            operation: &'a str,
            detail: &'a str,
        }
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Item<'a> {
            Old(&'a Entry),
            New(NewEntry<'a>),
        }
        #[derive(Serialize)]
        struct Proposed<'a> {
            version: u32,
            server: &'a str,
            client: &'a str,
            mountpoint: &'a str,
            profile: &'a str,
            relative: &'a str,
            write_text: &'a str,
            entries: Vec<Item<'a>>,
        }
        let mut first = usize::from(self.entries.len() == 16);
        loop {
            let mut entries = self.entries[first..]
                .iter()
                .map(Item::Old)
                .collect::<Vec<_>>();
            entries.push(Item::New(NewEntry { operation, detail }));
            let proposed = Proposed {
                version: self.version,
                server: &self.server,
                client: &self.client,
                mountpoint: &self.mountpoint,
                profile: &self.profile,
                relative: &self.relative,
                write_text: &self.write_text,
                entries,
            };
            if within(&proposed, MAX_SESSION) {
                break;
            }
            if first == self.entries.len() {
                return Err("NFS Session aggregate bound".into());
            }
            first += 1;
        }
        // Exact borrowed proposed JSON was admitted before cloning retained entries or read text.
        let mut entries = self.entries[first..].to_vec();
        entries.push(Entry {
            operation: operation.into(),
            detail: detail.into(),
        });
        Ok(Self {
            version: self.version,
            server: self.server.clone(),
            client: self.client.clone(),
            mountpoint: self.mountpoint.clone(),
            profile: self.profile.clone(),
            relative: self.relative.clone(),
            write_text: self.write_text.clone(),
            entries,
        })
    }
}
pub(crate) fn propose(
    d: &Definition,
    s: &core::State,
    session: &Session,
    a: &core::Action,
) -> Result<(core::State, Session, core::Decision), String> {
    if !action_shape(s.inventory(), a) || !within(a, core::MAX_ACTION) {
        return Err("NFS action scalar/context/aggregate bound".into());
    }
    d.validate()?;
    if !d.same_assignment(s) || !session.valid_for(s) {
        return Err("NFS immutable context or Session differs".into());
    }
    s.validate().map_err(|e| notice(e.message))?;
    let mut candidate = s.clone();
    let decision = candidate.apply(a).map_err(|e| notice(e.message))?;
    let next = session.recorded(a, &decision)?;
    Ok((candidate, next, decision))
}
pub fn perform(
    d: &Definition,
    s: &mut core::State,
    session: &mut Session,
    a: &core::Action,
) -> Result<core::Decision, String> {
    let (candidate, next, decision) = propose(d, s, session, a)?;
    *s = candidate;
    *session = next;
    Ok(decision)
}
