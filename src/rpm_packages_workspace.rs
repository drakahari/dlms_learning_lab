//! Bounded current RPM outcomes over immutable supplied metadata. No host operations.
use crate::rpm_packages::{self as core, Architecture, Identity, State};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, io::Write};
pub const MAX_STATE: usize = 65_536;
pub const MAX_DEFINITION: usize = 98_304;
pub const MAX_SCENARIO: usize = 393_216;
pub const MAX_ATTEMPT: usize = 1_048_576;
pub const HELP: &str = "Restricted virtual RPM console: rpm -q/-ql/-qf/--path/-qp/-qpl; --provides/--requires/--whatprovides; one fixed epoch queryformat; local -i/-U/--reinstall/-e. Declared noarch script-free metadata only, no host RPM, resolver, scripts, signatures, multilib or rpm -V. Unmodeled results are explicitly SimulatorUnsupported without an invented RPM status. Check inspects actual current outcomes; console history is never credit.";
pub fn notice(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    let mut out = String::with_capacity(n);
    out.push_str(&s[..n]);
    out
}
pub(crate) fn size<T: Serialize>(v: &T, cap: usize) -> Option<usize> {
    struct C {
        n: usize,
        cap: usize,
    }
    impl Write for C {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("RPM app compact bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = C { n: 0, cap };
    serde_json::to_writer(&mut c, v).ok()?;
    Some(c.n)
}
pub(crate) fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    size(v, cap).is_some()
}
pub(crate) struct Text<const N: usize>(pub String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct T<const N: usize>;
        impl<const N: usize> Visitor<'_> for T<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "non-NUL text <= {N} bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if s.len() > N || s.contains('\0') {
                    return Err(E::custom("RPM text bound"));
                }
                Ok(Text(s.into()))
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
        p: std::marker::PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("RPM row bound before body"));
            }
            V::deserialize(d)
        }
    }
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for L<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} rows")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<V>, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next {
                allowed: out.len() < N,
                p: std::marker::PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(L::<V, N>(std::marker::PhantomData))
}
pub(crate) fn paths<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, String>, D::Error> {
    struct M;
    impl<'de> Visitor<'de> for M {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most12 unique archive paths")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Text(k)) = a.next_key::<Text<160>>()? {
                if out.len() >= 12 || out.contains_key(&k) {
                    return Err(A::Error::custom("RPM duplicate/extra path before body"));
                }
                let Text(v) = a.next_value::<Text<32>>()?;
                out.insert(k, v);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M)
}
pub(crate) fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-+_.".contains(&b))
}
pub(crate) fn path(s: &str) -> bool {
    s.len() > 1
        && s.len() <= 160
        && s.is_ascii()
        && s.starts_with('/')
        && !s.bytes().any(|b| b < 32 || b == 127)
        && s.split('/')
            .skip(1)
            .all(|p| !p.is_empty() && p != "." && p != "..")
}
fn identity(c: &core::Catalog, i: &Identity) -> bool {
    i.arch == Architecture::Noarch && name(&i.name) && c.archives.values().any(|a| a.identity == *i)
}
/// Conservative closure for EVERY supported future operation, before candidate clones.
/// Runtime files are a subset of current files plus supplied payloads; no WriteFile.
/// 8192 covers state/runtime keys and <=8 installed records (bounded ASCII IDs/names).
fn closure(s: &State) -> bool {
    let c = s.catalog();
    let r = s.runtime();
    if c.archives.len() > 12
        || r.installed.len() > 8
        || c.allowed_paths.len() > 32
        || c.snapshots.len() > 2
        || c.archives
            .values()
            .any(|a| a.identity.arch != Architecture::Noarch)
    {
        return false;
    }
    let mut names = Vec::<&str>::new();
    for a in c.archives.values() {
        if !names.contains(&a.identity.name.as_str()) {
            names.push(&a.identity.name);
        }
    }
    if names.len() > 8 {
        return false;
    }
    let mut n = 8192usize;
    for v in [
        size(c, MAX_STATE),
        size(&r.files, MAX_STATE),
        size(&r.available, MAX_STATE),
        size(&r.cached, MAX_STATE),
    ] {
        let Some(v) = v else {
            return false;
        };
        let Some(next) = n.checked_add(v) else {
            return false;
        };
        n = next;
    }
    for a in c.archives.values() {
        let Some(v) = size(&a.files, MAX_STATE) else {
            return false;
        };
        let Some(next) = n.checked_add(v) else {
            return false;
        };
        n = next;
    }
    n <= MAX_STATE
}
pub fn admitted(s: &State) -> bool {
    s.validate().is_ok() && closure(s) && within(s, MAX_STATE)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Installed {
        package: Identity,
        version: Option<core::Constraint>,
    },
    SatisfiesCapability {
        requirement: core::Requirement,
    },
    OwnedPayloadMatches {
        package: Identity,
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
    },
    Absent {
        package: Identity,
    },
    DependenciesHealthy {},
    PreserveInitialPackage {
        package: Identity,
    },
    PreserveInitialFile {
        #[serde(deserialize_with = "string::<_,160>")]
        path: String,
    },
}
impl Requirement {
    fn positive(&self) -> bool {
        matches!(
            self,
            Self::Installed { .. }
                | Self::SatisfiesCapability { .. }
                | Self::OwnedPayloadMatches { .. }
        )
    }
    fn valid(&self, i: &State) -> bool {
        let c = i.catalog();
        match self {
            Self::Installed { package, version } => {
                identity(c, package) && version.as_ref().is_none_or(|v| v.evr.as_str().len() <= 128)
            }
            Self::SatisfiesCapability { requirement: q } => {
                name(&q.capability)
                    && q.constraint
                        .as_ref()
                        .is_none_or(|v| v.evr.as_str().len() <= 128)
                    && c.archives.values().any(|a| {
                        a.identity.name == q.capability || a.provides.contains_key(&q.capability)
                    })
            }
            Self::OwnedPayloadMatches { package, path: p } => {
                identity(c, package)
                    && path(p)
                    && c.archives
                        .values()
                        .any(|a| a.identity == *package && a.files.contains_key(p))
            }
            Self::Absent { package } => identity(c, package),
            Self::DependenciesHealthy {} => true,
            Self::PreserveInitialFile { path: p } => path(p) && i.runtime().files.contains_key(p),
            Self::PreserveInitialPackage { package } => {
                identity(c, package) && i.runtime().installed.iter().any(|v| v.identity == *package)
            }
        }
    }
    fn check(&self, i: &State, s: &State, v: &core::ValidatedQueries<'_>) -> (bool, String) {
        let r = s.runtime();
        let ok = match self {
            Self::Installed { package, version } => {
                v.installed(package).ok().flatten().is_some_and(|a| {
                    version.as_ref().is_none_or(|q| {
                        let o = a.evr.compare(&q.evr);
                        match q.relation {
                            core::Relation::Lt => o.is_lt(),
                            core::Relation::Le => !o.is_gt(),
                            core::Relation::Eq => o.is_eq(),
                            core::Relation::Ge => !o.is_lt(),
                            core::Relation::Gt => o.is_gt(),
                        }
                    })
                })
            }
            Self::SatisfiesCapability { requirement } => v
                .installed_providers(requirement)
                .is_ok_and(|a| !a.is_empty()),
            Self::OwnedPayloadMatches { package, path } => {
                v.installed(package)
                    .ok()
                    .flatten()
                    .is_some_and(|a| a.files.contains_key(path))
                    && v.file_issues(package)
                        .is_ok_and(|rows| !rows.iter().any(|x| &x.path == path))
            }
            Self::Absent { package } => v.installed(package).is_ok_and(|x| x.is_none()),
            Self::DependenciesHealthy {} => v.dependency_issues().is_empty(),
            Self::PreserveInitialFile { path } => i
                .runtime()
                .files
                .get(path)
                .is_some_and(|f| r.files.get(path) == Some(f)),
            Self::PreserveInitialPackage { package } => i
                .runtime()
                .installed
                .iter()
                .find(|x| &x.identity == package)
                .is_some_and(|old| {
                    r.installed.iter().any(|x| x == old)
                        && i.catalog().archives[&old.archive]
                            .files
                            .keys()
                            .all(|p| r.files.get(p) == i.runtime().files.get(p))
                }),
        };
        let target = match self {
            Self::Installed { package, .. } => {
                format!("{} has the required installed version", package.name)
            }
            Self::SatisfiesCapability { requirement } => {
                format!("An installed provider satisfies {}", requirement.capability)
            }
            Self::OwnedPayloadMatches { path, .. } => {
                format!("Recorded owner and simulated payload match for {path}")
            }
            Self::Absent { package } => format!("{} is absent", package.name),
            Self::DependenciesHealthy {} => {
                "All declared installed requirements are satisfied".into()
            }
            Self::PreserveInitialPackage { package } => {
                format!("{} and its supplied files are preserved", package.name)
            }
            Self::PreserveInitialFile { path } => format!("The initial file {path} is preserved"),
        };
        (
            ok,
            notice(if ok {
                target
            } else {
                format!("Not yet: {target}")
            }),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Definition {
    pub initial: State,
    pub archive_paths: BTreeMap<String, String>,
    pub requirements: Vec<Requirement>,
}
impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            initial: State,
            #[serde(deserialize_with = "paths")]
            archive_paths: BTreeMap<String, String>,
            #[serde(deserialize_with = "list::<_,Requirement,24>")]
            requirements: Vec<Requirement>,
        }
        let r = Raw::deserialize(d)?;
        let v = Self {
            initial: r.initial,
            archive_paths: r.archive_paths,
            requirements: r.requirements,
        };
        v.validate().map_err(D::Error::custom)?;
        Ok(v)
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.requirements.is_empty()
            || self.requirements.len() > 24
            || self.archive_paths.len() > 12
            || self.archive_paths.len() != self.initial.catalog().archives.len()
            || !self.requirements.iter().any(Requirement::positive)
            || !self.requirements.iter().all(|r| r.valid(&self.initial))
        {
            return Err("RPM positive goal/requirement shape differs".into());
        }
        let mut ids = Vec::<&str>::new();
        for (p, id) in &self.archive_paths {
            if !path(p)
                || !p.ends_with(".rpm")
                || !name(id)
                || !self.initial.catalog().archives.contains_key(id)
                || ids.contains(&id.as_str())
                || self.initial.catalog().allowed_paths.contains(p)
            {
                return Err("RPM archive path context differs".into());
            }
            ids.push(id.as_str());
        }
        if !admitted(&self.initial) || !within(self, MAX_DEFINITION) {
            return Err("RPM Definition exceeds bounded app context".into());
        }
        Ok(())
    }
    pub fn same_assignment(&self, s: &State) -> bool {
        self.initial.catalog() == s.catalog()
    }
    pub fn checks(&self, s: &State) -> Vec<(bool, String)> {
        if self.validate().is_err() || !self.same_assignment(s) {
            return vec![(false, "RPM authored context differs".into())];
        }
        let Ok(v) = s.validated_queries() else {
            return vec![(false, "RPM current state is invalid".into())];
        };
        if !closure(s) || !within(s, MAX_STATE) {
            return vec![(false, "RPM current state exceeds app bounds".into())];
        }
        self.requirements
            .iter()
            .map(|r| r.check(&self.initial, s, &v))
            .collect()
    }
}
pub fn action_bounded(a: &core::Action) -> bool {
    match a {
        core::Action::InstallSet { archives } | core::Action::UpgradeSet { archives } => {
            !archives.is_empty() && archives.len() <= 8 && archives.iter().all(|s| name(s))
        }
        core::Action::ReinstallExact { archive } => name(archive),
        core::Action::EraseSet { packages } => {
            !packages.is_empty()
                && packages.len() <= 8
                && packages
                    .iter()
                    .all(|i| i.arch == Architecture::Noarch && name(&i.name))
        }
    }
}
pub fn apply(d: &Definition, s: &mut State, a: &core::Action) -> Result<core::Plan, String> {
    if !action_bounded(a) || d.validate().is_err() || !d.same_assignment(s) || !admitted(s) {
        return Err("RPM bounded action/context differs; current state retained".into());
    } // closure bounds ALL possible payload candidates before this clone.
    let mut candidate = s.clone();
    let p = candidate.apply(a).map_err(|e| notice(e.message))?;
    if !admitted(&candidate) {
        return Err("RPM app aggregate limit; previous state retained".into());
    }
    *s = candidate;
    Ok(p)
}

// Exhaustive classification: new typed metadata requires a deliberate admission decision.
pub(crate) fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        physical_link_draft,
        physical_link_learning,
        file_backup_learning,
        etherchannel_draft,
        etherchannel_learning,
        cron_session,
        cron_learning,
        nfs_session,
        nfs_learning,
        rpm_session: _,
        rpm_learning: _,
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
        || nfs_session.is_some()
        || nfs_learning.is_some()
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
