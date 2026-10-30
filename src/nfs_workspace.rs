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
    size(v, cap).is_some()
}
pub(crate) fn size<T: Serialize + ?Sized>(v: &T, cap: usize) -> Option<usize> {
    let mut c = Counter { used: 0, cap };
    serde_json::to_writer(&mut c, v).ok()?;
    Some(c.used)
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

pub(crate) fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    if w.one_off_session.is_some() || w.one_off_learning.is_some() {
        return true;
    }
    let crate::workspace::WorkspaceData {
        physical_link_draft,
        physical_link_learning,
        one_off_session: _,
        one_off_learning: _,
        file_backup_learning,
        etherchannel_draft,
        etherchannel_learning,
        cron_session,
        cron_learning,
        nfs_session: _,
        nfs_learning: _,
        rpm_session,
        rpm_learning,
        nat_draft,
        nat_learning,
        debian_packages_terminal,
        apache_static_draft,
        apache_static_learning,
        debian_packages_draft,
        debian_packages_learning,
        spanning_tree_draft,
        spanning_tree_learning,
        dhcpv4_draft,
        dhcpv4_learning,
        version: _,
        unit_manager_draft,
        unit_manager_learning,
        unit_manager_terminal,
        ospf_draft,
        ospf_learning,
        acl_draft,
        acl_learning,
        script_learning,
        script_history: _,
        script_history_more: _,
        terminal_learning,
        process_learning,
        accounts_learning,
        discovery_draft,
        discovery_learning,
        storage_draft,
        storage_learning,
        ipv6_draft,
        ipv6_learning,
        routing_v4_draft,
        routing_v4_learning,
        address_learning,
        service_learning,
        client_learning,
        seed_dns_learning,
        seed_mount_learning,
        legacy_access_learning,
        legacy_logs_learning,
        client_draft,
        dns_learning,
        dns_draft,
        switching_draft,
        switching_learning,
        mac_draft,
        mac_learning,
        command: _,
        service_port: _,
        dns_address: _,
        dns_serial: _,
        mount_device: _,
        checked: _,
        ipv4_check: _,
        practice_check: _,
        migrated_host: _,
        legacy_host_reviewed: _,
        legacy_practice_reviewed: _,
        migrated_practice_host: _,
    } = w;
    false
        || physical_link_draft.is_some()
        || physical_link_learning.is_some()
        || file_backup_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || cron_session.is_some()
        || cron_learning.is_some()
        || rpm_session.is_some()
        || rpm_learning.is_some()
        || nat_draft.is_some()
        || nat_learning.is_some()
        || debian_packages_terminal.is_some()
        || apache_static_draft.is_some()
        || apache_static_learning.is_some()
        || debian_packages_draft.is_some()
        || debian_packages_learning.is_some()
        || spanning_tree_draft.is_some()
        || spanning_tree_learning.is_some()
        || dhcpv4_draft.is_some()
        || dhcpv4_learning.is_some()
        || unit_manager_draft.is_some()
        || unit_manager_learning.is_some()
        || unit_manager_terminal.is_some()
        || ospf_draft.is_some()
        || ospf_learning.is_some()
        || acl_draft.is_some()
        || acl_learning.is_some()
        || script_learning.is_some()
        || terminal_learning.is_some()
        || process_learning.is_some()
        || accounts_learning.is_some()
        || discovery_draft.is_some()
        || discovery_learning.is_some()
        || storage_draft.is_some()
        || storage_learning.is_some()
        || ipv6_draft.is_some()
        || ipv6_learning.is_some()
        || routing_v4_draft.is_some()
        || routing_v4_learning.is_some()
        || address_learning.is_some()
        || service_learning.is_some()
        || client_learning.is_some()
        || seed_dns_learning.is_some()
        || seed_mount_learning.is_some()
        || legacy_access_learning.is_some()
        || legacy_logs_learning.is_some()
        || client_draft.is_some()
        || dns_learning.is_some()
        || dns_draft.is_some()
        || switching_draft.is_some()
        || switching_learning.is_some()
        || mac_draft.is_some()
        || mac_learning.is_some()
}

pub const PREFIX: &str = "nfs ";
pub const HELP: &str = "Supplied virtual NFS exports/fstab editors and explicit Save/Apply/Mount/Unmount/Read/Write actions. Internal simulator-only nfs JSON is an authored/test codec, not a Linux command. No host mount, RPC, daemon, network or trusted credentials.";
pub fn execute(d: &Definition, s: &mut core::State, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + core::MAX_ACTION {
        return Err("NFS input exceeds raw action bound before trimming".into());
    }
    d.validate()?;
    if !d.same_assignment(s) {
        return Err("NFS supplied inventory differs".into());
    }
    match input.trim() {
        "help" => return Ok(HELP.into()),
        "show" => {
            return Ok(format!(
                "Virtual NFS: {} current mounts, {} existing text files",
                s.runtime().mounts.len(),
                s.runtime().contents.len()
            ));
        }
        _ => {}
    }
    let raw = input
        .trim()
        .strip_prefix(PREFIX)
        .ok_or("NFS requires its labeled simulator-only typed action")?;
    let a = core::Action::from_json(raw).map_err(|e| notice(e.message))?;
    let decision = perform(d, s, &mut Session::default(), &a)?;
    Ok(match decision {
        core::Decision::AllowedRead { text, .. } => text,
        core::Decision::Denied { stage, .. } => format!("Simulated NFS denial: {}", denial(stage)),
        core::Decision::AllowedWrite { .. } => "Existing virtual file written".into(),
        core::Decision::Mounted {} => "Virtual mount added".into(),
        core::Decision::Unmounted {} => "Virtual mount removed".into(),
        core::Decision::Changed { changed } => {
            format!("Declared NFS operation changed data: {changed}")
        }
    })
}
/// Full authored source admission, shared by current checks and writable NFS entry points.
pub(crate) fn context<'a>(
    a: &crate::progress::Attempt,
    s: &'a crate::pack::Scenario,
) -> Result<&'a Definition, String> {
    if s.physical_link_teaching.is_some() {
        return Err("Physical-link teaching belongs to its own activity".into());
    }
    crate::workspace::reject_file_backup_workspace(a)?;
    if s.one_off_teaching.is_some() {
        return Err("One-off scheduling teaching belongs to its scheduling activity".into());
    }
    if s.cron_teaching.is_some() {
        return Err("Scheduling teaching belongs to its scheduling activity".into());
    }
    if a.scenario_version != s.version || !within(s, MAX_SCENARIO) {
        return Err("NFS source version or complete Scenario bound differs".into());
    }
    crate::progress::validate_nfs_metadata(a)?;
    let (
        crate::engine::State::Nfs { simulation },
        crate::engine::State::Nfs {
            simulation: authored,
        },
        crate::engine::Goal::Nfs { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return Err("NFS State/Goal kind differs".into());
    };
    if authored != &definition.initial
        || !definition.same_assignment(simulation)
        || definition.validate().is_err()
    {
        return Err("NFS exact authored initial or full Inventory differs".into());
    }
    if s.nfs_teaching
        .as_ref()
        .is_some_and(|l| !l.bounded() || !l.separate_from(authored))
        || a.workspace
            .as_ref()
            .and_then(|w| w.nfs_learning.as_ref())
            .is_some_and(|l| {
                s.nfs_teaching
                    .as_ref()
                    .is_none_or(|source| !l.valid_for(source))
            })
    {
        return Err("NFS practice differs from its exact authored context".into());
    }
    Ok(definition.as_ref())
}
#[derive(Serialize)]
#[serde(tag = "kind")]
enum NfsRef<'a> {
    Nfs { simulation: &'a core::State },
}
fn field_bytes(name: &str, value: Option<usize>) -> Option<usize> {
    match value {
        None => Some(0),
        Some(n) => n.checked_add(name.len() + 4),
    }
}
/// Exact serde-size substitution. Mandatory Workspace fields guarantee a nonempty object.
/// This is admission math, not a cached validator or completion credential.
pub(crate) fn projected_attempt_size(
    a: &crate::progress::Attempt,
    simulation: &core::State,
    session: Option<usize>,
    learning: Option<usize>,
) -> Option<usize> {
    if !matches!(a.state, crate::engine::State::Nfs { .. }) {
        return None;
    }
    let old = size(a, MAX_ATTEMPT)?;
    let state_old = size(&a.state, MAX_ATTEMPT)?;
    let state_new = size(&NfsRef::Nfs { simulation }, MAX_ATTEMPT)?;
    let blank = crate::workspace::WorkspaceData::new();
    let w = a.workspace.as_ref().unwrap_or(&blank);
    let old_w = size(w, MAX_ATTEMPT)?;
    let old_session = w
        .nfs_session
        .as_ref()
        .map(|x| size(x, MAX_ATTEMPT))
        .transpose_option()?;
    let old_learning = w
        .nfs_learning
        .as_ref()
        .map(|x| size(x, MAX_ATTEMPT))
        .transpose_option()?;
    let next_w = old_w
        .checked_sub(field_bytes("nfs_session", old_session)?)?
        .checked_sub(field_bytes("nfs_learning", old_learning)?)?
        .checked_add(field_bytes("nfs_session", session)?)?
        .checked_add(field_bytes("nfs_learning", learning)?)?;
    let next = old.checked_sub(state_old)?.checked_add(state_new)?;
    if a.workspace.is_some() {
        next.checked_sub(old_w)?.checked_add(next_w)
    } else {
        next.checked_add("workspace".len() + 4)?.checked_add(next_w)
    }
}
trait OptionalSize {
    fn transpose_option(self) -> Option<Option<usize>>;
}
impl OptionalSize for Option<Option<usize>> {
    fn transpose_option(self) -> Option<Option<usize>> {
        match self {
            None => Some(None),
            Some(Some(n)) => Some(Some(n)),
            Some(None) => None,
        }
    }
}
fn retained_learning_size(a: &crate::progress::Attempt) -> Option<Option<usize>> {
    a.workspace
        .as_ref()
        .and_then(|w| w.nfs_learning.as_ref())
        .map(|l| size(l, crate::nfs_teaching::MAX_LEARNING))
        .transpose_option()
}
fn admitted_projection(n: Option<usize>) -> Result<(), String> {
    if n.is_some_and(|n| n <= MAX_ATTEMPT) {
        Ok(())
    } else {
        Err("NFS whole Attempt exceeds2MiB; previous work retained".into())
    }
}
/// Common void workspace prepare delegates here before any legacy preservation/mutation.
pub(crate) fn prepare_session(a: &mut crate::progress::Attempt) -> Result<(), String> {
    crate::progress::validate_nfs_workspace(a)?;
    let crate::engine::State::Nfs { simulation } = &a.state else {
        return Err("Not an NFS attempt".into());
    };
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.nfs_session.is_some())
    {
        return Ok(());
    }
    let session = Session::default();
    admitted_projection(projected_attempt_size(
        a,
        simulation,
        Some(size(&session, MAX_SESSION).ok_or("NFS Session bound")?),
        retained_learning_size(a).ok_or("NFS Learning bound")?,
    ))?;
    a.workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .nfs_session = Some(session);
    Ok(())
}
pub fn perform_attempt(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    action: &core::Action,
) -> Result<core::Decision, String> {
    let d = context(a, s)?;
    let crate::engine::State::Nfs { simulation } = &a.state else {
        return Err("NFS kind differs".into());
    };
    let blank = Session::default();
    let session = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_session.as_ref())
        .unwrap_or(&blank);
    let (candidate, next, decision) = propose(d, simulation, session, action)?;
    admitted_projection(projected_attempt_size(
        a,
        &candidate,
        Some(size(&next, MAX_SESSION).ok_or("NFS Session bound")?),
        retained_learning_size(a).ok_or("NFS Learning bound")?,
    ))?;
    a.state = crate::engine::State::Nfs {
        simulation: candidate,
    };
    a.workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .nfs_session = Some(next);
    Ok(decision)
}
pub(crate) fn prepare_attempt(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    new: bool,
) -> Result<(), String> {
    context(a, s)?;
    let crate::engine::State::Nfs { simulation } = &a.state else {
        return Err("NFS kind differs".into());
    };
    let blank = Session::default();
    let session = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_session.as_ref())
        .unwrap_or(&blank);
    let learning = a.workspace.as_ref().and_then(|w| w.nfs_learning.as_ref());
    let learned_size = if let Some(old) = learning {
        Some(size(old, crate::nfs_teaching::MAX_LEARNING).ok_or("NFS retained practice bound")?)
    } else if let Some(l) = &s.nfs_teaching {
        Some(crate::nfs_teaching::Learning::new_size(l, new).ok_or("NFS new practice bound")?)
    } else {
        None
    };
    admitted_projection(projected_attempt_size(
        a,
        simulation,
        Some(size(session, MAX_SESSION).ok_or("NFS Session bound")?),
        learned_size,
    ))?;
    let fresh = if learning.is_none() {
        s.nfs_teaching
            .as_ref()
            .map(|l| {
                let mut p = crate::nfs_teaching::Learning::new(l)?;
                if !new {
                    p.view = crate::script_teaching::View::Assignment;
                }
                Ok::<_, String>(p)
            })
            .transpose()?
    } else {
        None
    };
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if w.nfs_session.is_none() {
        w.nfs_session = Some(blank);
    }
    if let Some(f) = fresh {
        w.nfs_learning = Some(f);
    }
    Ok(())
}
pub fn perform_practice(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    action: &core::Action,
) -> Result<core::Decision, String> {
    context(a, s)?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_learning.as_ref())
        .ok_or("NFS practice is not prepared")?;
    let (candidate, next, decision) = propose(&l.context, &l.simulation, &l.session, action)?;
    let bytes = l
        .proposed_size(&candidate, &next)
        .ok_or("NFS proposed practice bound")?;
    let crate::engine::State::Nfs { simulation } = &a.state else {
        return Err("NFS kind differs".into());
    };
    let session = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_session.as_ref())
        .map(|x| size(x, MAX_SESSION))
        .transpose_option()
        .ok_or("NFS retained Session bound")?;
    admitted_projection(projected_attempt_size(a, simulation, session, Some(bytes)))?;
    a.workspace
        .as_mut()
        .unwrap()
        .nfs_learning
        .as_mut()
        .unwrap()
        .commit(candidate, next)?;
    Ok(decision)
}
pub fn fill_practice(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let source = s.nfs_teaching.as_ref().ok_or("NFS lesson absent")?;
    let old = size(a, MAX_ATTEMPT).ok_or("NFS Attempt bound")?;
    let old_l = size(
        a.workspace
            .as_ref()
            .and_then(|w| w.nfs_learning.as_ref())
            .ok_or("NFS practice unprepared")?,
        crate::nfs_teaching::MAX_LEARNING,
    )
    .ok_or("NFS practice bound")?;
    a.workspace
        .as_mut()
        .unwrap()
        .nfs_learning
        .as_mut()
        .unwrap()
        .fill_with_budget(source, |new| {
            old.checked_sub(old_l)
                .and_then(|n| n.checked_add(new))
                .is_some_and(|n| n <= MAX_ATTEMPT)
        })
}
pub fn reset_practice(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let source = s.nfs_teaching.as_ref().ok_or("NFS lesson absent")?;
    let crate::engine::State::Nfs { simulation } = &a.state else {
        return Err("NFS kind differs".into());
    };
    let session = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_session.as_ref())
        .map(|x| size(x, MAX_SESSION))
        .transpose_option()
        .ok_or("NFS Session bound")?;
    // Reset opens Example; new_size_view computes that exact borrowed wire before any fixture copy.
    let bytes =
        crate::nfs_teaching::Learning::new_size_view(source, crate::script_teaching::View::Example)
            .ok_or("NFS reset practice bound")?;
    admitted_projection(projected_attempt_size(a, simulation, session, Some(bytes)))?;
    let mut fresh = crate::nfs_teaching::Learning::new(source)?;
    fresh.view = crate::script_teaching::View::Example;
    a.workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .nfs_learning = Some(fresh);
    Ok(())
}

/// Borrowed request/view edits, never a file write or document publication.
/// None retains the existing value; an explicit value wins over dependent clearing.
#[derive(Clone, Copy, Default)]
pub struct SessionEdit<'a> {
    pub server: Option<&'a str>,
    pub client: Option<&'a str>,
    pub mountpoint: Option<&'a str>,
    pub profile: Option<&'a str>,
    pub relative: Option<&'a str>,
    pub write_text: Option<&'a str>,
}
#[derive(Serialize)]
struct BorrowedSession<'a> {
    version: u32,
    server: &'a str,
    client: &'a str,
    mountpoint: &'a str,
    profile: &'a str,
    relative: &'a str,
    write_text: &'a str,
    entries: &'a [Entry],
}
impl BorrowedSession<'_> {
    fn owned(&self) -> Session {
        #[cfg(test)]
        SESSION_EDIT_COPIES.with(|n| n.set(n.get() + 1));
        Session {
            version: self.version,
            server: self.server.into(),
            client: self.client.into(),
            mountpoint: self.mountpoint.into(),
            profile: self.profile.into(),
            relative: self.relative.into(),
            write_text: self.write_text.into(),
            entries: self.entries.to_vec(),
        }
    }
}
#[cfg(test)]
thread_local! { static SESSION_EDIT_COPIES: std::cell::Cell<usize> = const {std::cell::Cell::new(0)}; }
fn edit_session(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    edit: SessionEdit<'_>,
    practice: bool,
) -> Result<(), String> {
    for (v, cap) in [
        (edit.server, 32),
        (edit.client, 32),
        (edit.mountpoint, 160),
        (edit.profile, 32),
        (edit.relative, 160),
        (edit.write_text, 4096),
    ] {
        if v.is_some_and(|v| !text(v, cap)) {
            return Err(
                "NFS request field exceeds its non-NUL byte bound; previous fields retained".into(),
            );
        }
    }
    context(a, s)?;
    let blank = Session::default();
    let (simulation, old) = if practice {
        let l = a
            .workspace
            .as_ref()
            .and_then(|w| w.nfs_learning.as_ref())
            .ok_or("NFS practice unprepared")?;
        (&l.simulation, &l.session)
    } else {
        let crate::engine::State::Nfs { simulation } = &a.state else {
            return Err("NFS kind differs".into());
        };
        (
            simulation,
            a.workspace
                .as_ref()
                .and_then(|w| w.nfs_session.as_ref())
                .unwrap_or(&blank),
        )
    };
    let selected_client = edit.client.unwrap_or(&old.client);
    let changed_client = selected_client != old.client;
    let profile = edit.profile.unwrap_or_else(|| {
        if changed_client
            && !old.profile.is_empty()
            && !client(simulation.inventory(), selected_client)
                .is_some_and(|c| c.profiles.contains(&old.profile))
        {
            ""
        } else {
            &old.profile
        }
    });
    let mountpoint = edit.mountpoint.unwrap_or_else(|| {
        if changed_client
            && !old.mountpoint.is_empty()
            && !target(simulation.inventory(), selected_client, &old.mountpoint)
        {
            ""
        } else {
            &old.mountpoint
        }
    });
    let proposed = BorrowedSession {
        version: old.version,
        server: edit.server.unwrap_or(&old.server),
        client: selected_client,
        mountpoint,
        profile,
        relative: edit.relative.unwrap_or(&old.relative),
        write_text: edit.write_text.unwrap_or(&old.write_text),
        entries: &old.entries,
    };
    if (!proposed.server.is_empty() && !server(simulation.inventory(), proposed.server))
        || (!proposed.client.is_empty()
            && client(simulation.inventory(), proposed.client).is_none())
        || (!proposed.mountpoint.is_empty()
            && !target(simulation.inventory(), proposed.client, proposed.mountpoint))
        || (!proposed.profile.is_empty()
            && !client(simulation.inventory(), proposed.client)
                .is_some_and(|c| c.profiles.iter().any(|p| p == proposed.profile)))
    {
        return Err("NFS request selection differs from the supplied client/context".into());
    }
    let session_size = size(&proposed, MAX_SESSION)
        .ok_or("NFS proposed Session aggregate; previous fields retained")?;
    let crate::engine::State::Nfs {
        simulation: assignment,
    } = &a.state
    else {
        return Err("NFS kind differs".into());
    };
    if practice {
        let l = a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap();
        #[derive(Serialize)]
        struct Proposed<'a> {
            version: u32,
            view: crate::script_teaching::View,
            context: &'a Definition,
            simulation: &'a core::State,
            session: &'a BorrowedSession<'a>,
            selected_example: usize,
            checked: bool,
        }
        let bytes = size(
            &Proposed {
                version: l.version,
                view: l.view,
                context: &l.context,
                simulation: &l.simulation,
                session: &proposed,
                selected_example: l.selected_example,
                checked: l.checked,
            },
            crate::nfs_teaching::MAX_LEARNING,
        )
        .ok_or("NFS proposed Learning aggregate; previous fields retained")?;
        let retained = a
            .workspace
            .as_ref()
            .and_then(|w| w.nfs_session.as_ref())
            .map(|t| size(t, MAX_SESSION))
            .transpose_option()
            .ok_or("NFS retained Session bound")?;
        admitted_projection(projected_attempt_size(a, assignment, retained, Some(bytes)))?;
    } else {
        admitted_projection(projected_attempt_size(
            a,
            assignment,
            Some(session_size),
            retained_learning_size(a).ok_or("NFS retained Learning bound")?,
        ))?;
    }
    let owned = proposed.owned();
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if practice {
        w.nfs_learning.as_mut().unwrap().session = owned;
    } else {
        w.nfs_session = Some(owned);
    }
    Ok(())
}
pub fn edit_assignment_session(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    edit: SessionEdit<'_>,
) -> Result<(), String> {
    edit_session(a, s, edit, false)
}
pub fn edit_practice_session(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    edit: SessionEdit<'_>,
) -> Result<(), String> {
    edit_session(a, s, edit, true)
}
/// Bounded presentation-only metadata; no action, grading result or state is installed.
pub fn set_practice_presentation(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    view: Option<crate::script_teaching::View>,
    selected: Option<usize>,
) -> Result<(), String> {
    context(a, s)?;
    let source = s.nfs_teaching.as_ref().ok_or("NFS lesson absent")?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_learning.as_ref())
        .ok_or("NFS practice unprepared")?;
    let view = view.unwrap_or(l.view);
    let selected = selected.unwrap_or(l.selected_example);
    if selected >= source.example_actions.len() {
        return Err("NFS example selection differs".into());
    }
    let bytes = l
        .presentation_size(view, selected)
        .ok_or("NFS practice presentation bound")?;
    let crate::engine::State::Nfs { simulation } = &a.state else {
        return Err("NFS kind differs".into());
    };
    let session = a
        .workspace
        .as_ref()
        .and_then(|w| w.nfs_session.as_ref())
        .map(|t| size(t, MAX_SESSION))
        .transpose_option()
        .ok_or("NFS Session bound")?;
    admitted_projection(projected_attempt_size(a, simulation, session, Some(bytes)))?;
    let l = a.workspace.as_mut().unwrap().nfs_learning.as_mut().unwrap();
    l.view = view;
    l.selected_example = selected;
    Ok(())
}

/// Recency is presentation metadata too: a decimal digit boundary cannot exceed the Attempt cap.
pub fn record_visit(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    visited: u64,
) -> Result<(), String> {
    context(a, s)?;
    let old = size(a, MAX_ATTEMPT).ok_or("NFS saved capacity bound")?;
    let previous = size(&a.visited, 32).ok_or("NFS prior recency count")?;
    let next = size(&visited, 32).ok_or("NFS proposed recency count")?;
    admitted_projection(old.checked_sub(previous).and_then(|n|n.checked_add(next)))
      .map_err(|_|"Recency could not be updated within this NFS Attempt's saved capacity; all work and the earlier visit value are retained".to_string())?;
    a.visited = visited;
    Ok(())
}

#[cfg(test)]
mod attempt_projection_tests {
    use super::*;
    use crate::{
        engine::State,
        nfs_teaching as teach,
        pack::{self, Scenario},
        progress::{self, Attempt},
        workspace::WorkspaceData,
    };
    // Library-unit scope cannot import the UI-owned historical boundary. Keep
    // its complete new-source equality here; external catalog190 guards retain
    // the independent recipe/pack hashes and raw/full-typed unit inverse.
    fn ere_predecessor(all: &[Scenario]) -> &[Scenario] {
        let recipe: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../tests/fixtures/catalog193-registration.json"
        ))
        .unwrap();
        let new: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/shell-ere-evidence-v1.json")).unwrap();
        assert_eq!(all.len(), 193);
        assert_eq!(new.scenarios.len(), 3);
        assert_eq!(
            new.scenarios
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            [
                "shell-ere-event-view",
                "shell-ere-inventory-review",
                "shell-ere-incident-report"
            ]
        );
        assert_eq!(
            all[..190]
                .iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>(),
            *recipe["before_ids"].as_array().unwrap()
        );
        assert_eq!(
            serde_json::to_value(&all[190..]).unwrap(),
            serde_json::to_value(&new.scenarios).unwrap()
        );
        &all[..190]
    }
    fn ntp_predecessor(all: &[Scenario]) -> &[Scenario] {
        let all = ere_predecessor(all);
        let recipe: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../tests/fixtures/catalog190-registration.json"
        ))
        .unwrap();
        let new: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/ntp-client-server-v1.json")).unwrap();
        assert_eq!(all.len(), 190);
        assert_eq!(new.scenarios.len(), 4);
        assert_eq!(
            new.scenarios
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            [
                "ccna-ntp-client-chain",
                "ccna-ntp-cycle-repair",
                "ccna-ntp-asymmetric-delivery",
                "ccna-ntp-policy-handover",
            ]
        );
        assert_eq!(
            all[..186]
                .iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>(),
            *recipe["before_ids"].as_array().unwrap()
        );
        assert_eq!(
            serde_json::to_value(&all[186..]).unwrap(),
            serde_json::to_value(&new.scenarios).unwrap()
        );
        &all[..186]
    }
    fn scenario() -> Scenario {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/nfs-workbench-v1.json")).unwrap();
        let c = &v["cases"][0];
        serde_json::from_value(serde_json::json!({"id":"nfs-technical-progress","version":1,"track":"LPIC-2","topic":"Virtual NFS","unit":"","title":"Technical NFS adapter","summary":"Supplied existing-file model","objective":"Complete current and next-mount outcomes","retry_guidance":"Inspect the separate layers","difficulty":"Intermediate","minutes":15,"steps":["Inspect","Edit","Check"],"guide":[],"explanation":"No host service","hints":["Save and Apply differ"],"sources":["https://www.rfc-editor.org/rfc/rfc1813.html"],"simplifications":"Supplied virtual ACCESS-style decisions; no host mount","initial":{"kind":"Nfs","simulation":c["definition"]["initial"]},"goal":{"kind":"Nfs","definition":c["definition"]},"solution":[],"prerequisites":[],"nfs_teaching":c["lesson"]})).unwrap()
    }
    #[test]
    fn exact_projection_matches_independent_serde_all_optional_shapes_and_genuine_history() {
        let s = scenario();
        let source = s.nfs_teaching.as_ref().unwrap();
        let learning = teach::Learning::new(source).unwrap();
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/shell-v8-synthetic-writer-progress.json"
        ))
        .unwrap();
        let genuine = raw["attempts"]
            .as_object()
            .unwrap()
            .values()
            .filter_map(|a| a["state"].get("run").filter(|r| !r.is_null()))
            .take(24)
            .map(|v| serde_json::from_value::<crate::shell::Record>(v.clone()).unwrap())
            .collect::<Vec<_>>();
        assert!(!genuine.is_empty());
        for workspace in [false, true] {
            for old_session in [false, true] {
                for old_learning in [false, true] {
                    if !workspace && (old_session || old_learning) {
                        continue;
                    }
                    let mut a = Attempt::new(&s, 11);
                    if workspace {
                        let mut w = WorkspaceData::new();
                        if old_session {
                            w.nfs_session = Some(Session::default());
                        }
                        if old_learning {
                            w.nfs_learning = Some(learning.clone());
                        }
                        w.script_history = genuine.first().cloned();
                        w.script_history_more = genuine.iter().skip(1).cloned().collect();
                        assert!(w.valid());
                        a.workspace = Some(w);
                    }
                    let State::Nfs { simulation } = &a.state else {
                        panic!()
                    };
                    for new_session in [false, true] {
                        for new_learning in [false, true] {
                            let session = new_session.then(Session::default);
                            let l = new_learning.then(|| learning.clone());
                            let predicted = projected_attempt_size(
                                &a,
                                simulation,
                                session
                                    .as_ref()
                                    .map(|x| serde_json::to_vec(x).unwrap().len()),
                                l.as_ref().map(|x| serde_json::to_vec(x).unwrap().len()),
                            )
                            .unwrap();
                            let mut expected = a.clone();
                            let w = expected.workspace.get_or_insert_with(WorkspaceData::new);
                            w.nfs_session = session;
                            w.nfs_learning = l;
                            assert_eq!(
                                predicted,
                                serde_json::to_vec(&expected).unwrap().len(),
                                "full optional-field size equality"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    #[ignore = "Requires the retained accepted private actual9 capture; never fabricates a run"]
    fn actual9_profile_nfs_addition_keeps_original_records_and_raw_backup() {
        let raw = crate::actual9_test_evidence::original9();
        let mut all = pack::bundled().unwrap();
        assert_eq!(ntp_predecessor(&all).len(), 186);
        assert_eq!(ere_predecessor(&all).len(), 190);
        assert_eq!(all.len(), 193);
        let new: crate::pack::Pack =
            serde_json::from_str(include_str!("../scenarios/linux-one-off-v1.json")).unwrap();
        assert_eq!(
            serde_json::to_value(&all[176..180]).unwrap(),
            serde_json::to_value(&new.scenarios).unwrap()
        );
        assert_eq!(all[..176].len(), 176);
        let s = scenario();
        all.push(s.clone());
        let mut p = progress::decode(&raw, &all).unwrap();
        let old = serde_json::to_value(&p.attempts).unwrap();
        let mut a = Attempt::new(&s, 17);
        teach::prepare(&mut a, &s, true).unwrap();
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synthetic.json");
        std::fs::write(&path, &raw).unwrap();
        progress::save(&path, &p).unwrap();
        assert_eq!(
            std::fs::read(path.with_extension(format!(
                "before-shell-v{}.json",
                crate::shell::EVALUATOR_REVISION
            )))
            .unwrap(),
            raw
        );
        let loaded = progress::load(&path, &all).unwrap();
        let mut old_loaded = serde_json::to_value(&loaded.attempts).unwrap();
        old_loaded.as_object_mut().unwrap().remove(&s.id);
        assert!(old_loaded == old);
        assert!(
            serde_json::to_value(&loaded.attempts[&s.id]).unwrap()
                == serde_json::to_value(&p.attempts[&s.id]).unwrap()
        );
    }
    #[test]
    fn borrowed_session_edit_whole_attempt_refusal_precedes_actual_copy_site() {
        let source = scenario();
        let mut a = Attempt::new(&source, 1);
        teach::prepare(&mut a, &source, true).unwrap();
        let mut good = a.clone();
        let text = "\u{1}".repeat(4096);
        SESSION_EDIT_COPIES.with(|n| n.set(0));
        edit_assignment_session(
            &mut good,
            &source,
            SessionEdit {
                write_text: Some(&text),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(SESSION_EDIT_COPIES.with(|n| n.get()), 1);
        let mut lo = 0usize;
        let mut hi = MAX_ATTEMPT;
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            a.transcript = vec!["x".repeat(mid)];
            if serde_json::to_vec(&a).unwrap().len() <= MAX_ATTEMPT {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        a.transcript = vec!["x".repeat(lo)];
        let before = serde_json::to_value(&a).unwrap();
        SESSION_EDIT_COPIES.with(|n| n.set(0));
        assert!(
            edit_assignment_session(
                &mut a,
                &source,
                SessionEdit {
                    write_text: Some(&text),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(SESSION_EDIT_COPIES.with(|n| n.get()), 0);
        assert!(serde_json::to_value(&a).unwrap() == before);
    }
}
