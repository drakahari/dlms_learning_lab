//! Outcome grading for bounded virtual backup tasks, not command-history matching.
//! No host access; not yet registered in the normal catalog.
pub mod presentation;
use crate::engine::{Goal, State};
use crate::{file_backup::Tree, file_backup_terminal::Store};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const MAX_WIRE: usize = 2 * 1024 * 1024;
struct Count(usize);
impl std::io::Write for Count {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_WIRE)
            .ok_or_else(|| std::io::Error::other("Backup definition byte bound"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn fits<T: Serialize>(value: &T) -> bool {
    serde_json::to_writer(Count(0), value).is_ok()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RecoveryProbe {
    #[serde(deserialize_with = "text::<_,120>")]
    pub name: String,
    pub initial: Tree,
    pub expected: Tree,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Tree {
        #[serde(deserialize_with = "text::<_,120>")]
        name: String,
        #[serde(deserialize_with = "text::<_,1024>")]
        path: String,
        expected: Tree,
    },
    Recovery {
        #[serde(deserialize_with = "text::<_,120>")]
        name: String,
        #[serde(deserialize_with = "strings::<_,8>")]
        archives: Vec<String>,
        #[serde(deserialize_with = "text::<_,255>")]
        root: String,
        #[serde(deserialize_with = "list::<_,RecoveryProbe,4>")]
        probes: Vec<RecoveryProbe>,
    },
    Payloads {
        #[serde(deserialize_with = "text::<_,120>")]
        name: String,
        #[serde(deserialize_with = "text::<_,1024>")]
        archive: String,
        #[serde(deserialize_with = "text::<_,255>")]
        root: String,
        #[serde(deserialize_with = "strings::<_,64>")]
        files: Vec<String>,
        incremental: bool,
    },
    PreserveArtifacts {
        #[serde(deserialize_with = "text::<_,120>")]
        name: String,
        #[serde(deserialize_with = "strings::<_,8>")]
        archives: Vec<String>,
        #[serde(deserialize_with = "strings::<_,8>")]
        snapshots: Vec<String>,
    },
    Snapshot {
        #[serde(deserialize_with = "text::<_,120>")]
        name: String,
        #[serde(deserialize_with = "text::<_,1024>")]
        path: String,
        #[serde(deserialize_with = "text::<_,1024>")]
        source: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Store,
    #[serde(deserialize_with = "strings::<_,6>")]
    pub protected: Vec<String>,
    #[serde(deserialize_with = "list::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
fn short(text: &str, max: usize) -> bool {
    !text.trim().is_empty() && text.len() <= max && !text.chars().any(char::is_control)
}
fn artifact(store: &Store, p: &str) -> bool {
    p.len() <= 1024
        && p.split('/').skip(1).count() <= 64
        && p.rsplit_once('/').is_some_and(|(parent, n)| {
            parent == store.storage
                && short(n, 255)
                && n != "."
                && n != ".."
                && !n.chars().any(char::is_control)
        })
}
fn projection(tree: &Tree) -> BTreeMap<&str, (&crate::engine::Node, Option<&str>)> {
    tree.entries
        .iter()
        .map(|(p, e)| (p.as_str(), (&e.node, e.text.as_deref())))
        .collect()
}
fn tree_feedback(name: &str, actual: &Tree, expected: &Tree) -> (bool, String) {
    let a = projection(actual);
    let e = projection(expected);
    let missing = e.keys().filter(|p| !a.contains_key(**p)).count();
    let extra = a.keys().filter(|p| !e.contains_key(**p)).count();
    let contents = e
        .iter()
        .filter(|(p, (_, text))| a.get(**p).is_some_and(|(_, v)| v != text))
        .count();
    let metadata = e
        .iter()
        .filter(|(p, (node, _))| a.get(**p).is_some_and(|(n, _)| n != node))
        .count();
    let pass = missing + extra + contents + metadata == 0;
    (
        pass,
        if pass {
            format!("{name}: names, contents and permissions/ownership match")
        } else {
            format!(
                "{name}: {missing} missing, {extra} extra, {contents} wrong contents, {metadata} wrong file types/permissions/owners. Inspect the restored files and the archive members, then retry."
            )
        },
    )
}
fn bounded_tree(tree: &Tree) -> bool {
    tree.valid()
        && tree.entries.len() <= 64
        && tree
            .entries
            .values()
            .filter_map(|v| v.text.as_ref())
            .map(String::len)
            .sum::<usize>()
            <= 32768
}
fn relative_file(p: &str) -> bool {
    !p.is_empty()
        && p.len() <= 1024
        && p.split('/').count() <= 64
        && p.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.len() <= 255
                && !part.chars().any(char::is_control)
        })
}
impl Definition {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_WIRE {
            return Err("Backup assignment exceeds2MiB".into());
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid backup assignment: {e}"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !self.initial.valid()
            || self.protected.len() > 6
            || self.requirements.is_empty()
            || self.requirements.len() > 12
            || self
                .protected
                .iter()
                .any(|p| !self.initial.trees.contains_key(p))
            || self.protected.iter().collect::<BTreeSet<_>>().len() != self.protected.len()
            || !fits(self)
        {
            return Err("Invalid bounded backup assignment".into());
        }
        let mut names = BTreeSet::new();
        for requirement in &self.requirements {
            let name = match requirement {
                Requirement::Tree { name, .. }
                | Requirement::Recovery { name, .. }
                | Requirement::Payloads { name, .. }
                | Requirement::Snapshot { name, .. }
                | Requirement::PreserveArtifacts { name, .. } => name,
            };
            if !names.insert(name) {
                return Err("Duplicate backup objective name".into());
            }
            let valid = match requirement {
                Requirement::Tree {
                    name,
                    path,
                    expected,
                } => {
                    short(name, 120)
                        && self.initial.trees.contains_key(path)
                        && bounded_tree(expected)
                }
                Requirement::Recovery {
                    name,
                    archives,
                    root,
                    probes,
                } => {
                    short(name, 120)
                        && relative_file(root)
                        && !root.contains('/')
                        && !archives.is_empty()
                        && archives.len() <= 8
                        && archives.iter().all(|p| artifact(&self.initial, p))
                        && archives.iter().collect::<BTreeSet<_>>().len() == archives.len()
                        && !probes.is_empty()
                        && probes.len() <= 4
                        && probes.iter().all(|p| {
                            short(&p.name, 120)
                                && bounded_tree(&p.initial)
                                && bounded_tree(&p.expected)
                        })
                }
                Requirement::Payloads {
                    name,
                    archive,
                    files,
                    root,
                    ..
                } => {
                    short(name, 120)
                        && artifact(&self.initial, archive)
                        && files.len() <= 64
                        && relative_file(root)
                        && !root.contains('/')
                        && files.iter().all(|p| relative_file(p))
                        && files.iter().collect::<BTreeSet<_>>().len() == files.len()
                }
                Requirement::PreserveArtifacts {
                    name,
                    archives,
                    snapshots,
                } => {
                    short(name, 120)
                        && archives.len() <= 8
                        && snapshots.len() <= 8
                        && !(archives.is_empty() && snapshots.is_empty())
                        && archives
                            .iter()
                            .all(|p| self.initial.archives.contains_key(p))
                        && snapshots
                            .iter()
                            .all(|p| self.initial.snapshots.contains_key(p))
                        && archives.iter().collect::<BTreeSet<_>>().len() == archives.len()
                        && snapshots.iter().collect::<BTreeSet<_>>().len() == snapshots.len()
                }
                Requirement::Snapshot { name, path, source } => {
                    short(name, 120)
                        && artifact(&self.initial, path)
                        && self.initial.trees.contains_key(source)
                }
            };
            if !valid {
                return Err("Invalid backup outcome or independent recovery probe".into());
            }
        }
        Ok(())
    }
    pub fn bound_state(&self, state: &Store) -> bool {
        state.valid()
            && state.storage == self.initial.storage
            && state.trees.keys().eq(self.initial.trees.keys())
    }
    pub fn checks(&self, state: &Store) -> Vec<(bool, String)> {
        match CheckRun::new(self, state) {
            Ok(mut run) => {
                while !run.tick() {}
                run.rows
            }
            Err(message) => vec![(false, message)],
        }
    }
}
fn immediate_check(initial: &Store, state: &Store, requirement: &Requirement) -> (bool, String) {
    match requirement {
        Requirement::Tree {
            name,
            path,
            expected,
        } => tree_feedback(name, &state.trees[path], expected),
        Requirement::Payloads {
            name,
            archive,
            files,
            root,
            incremental,
        } => {
            let Some(value) = state.archives.get(archive) else {
                return (false, format!("{name}: archive is missing"));
            };
            let actual = value
                .entries
                .iter()
                .filter(|(_, e)| !e.node.directory)
                .map(|(p, _)| p.as_str())
                .collect::<BTreeSet<_>>();
            let expected = files.iter().map(String::as_str).collect::<BTreeSet<_>>();
            let pass =
                actual == expected && value.root == *root && value.incremental == *incremental;
            (
                pass,
                if pass {
                    format!("{name}: stored file names, top-level folder and backup type match")
                } else {
                    format!(
                        "{name}: stored file names, top-level folder or backup type differ. List the archive and check which snapshot you used; names alone do not verify file contents."
                    )
                },
            )
        }
        Requirement::PreserveArtifacts {
            name,
            archives,
            snapshots,
        } => {
            let changed_archives = archives
                .iter()
                .filter(|p| state.archives.get(*p) != initial.archives.get(*p))
                .count();
            let changed_snapshots = snapshots
                .iter()
                .filter(|p| state.snapshots.get(*p) != initial.snapshots.get(*p))
                .count();
            let pass = changed_archives + changed_snapshots == 0;
            (
                pass,
                if pass {
                    format!("{name}: the supplied backups and snapshots are unchanged")
                } else {
                    format!(
                        "{name}: {changed_archives} changed or missing backup archives, {changed_snapshots} changed or missing snapshots. Use a separate destination to preserve the supplied recovery history."
                    )
                },
            )
        }
        Requirement::Snapshot { name, path, source } => {
            let pass = state.snapshots.get(path).is_some_and(|s| {
                s.source == *source
                    && source.rsplit('/').next() == Some(s.snapshot.root.as_str())
                    && s.snapshot.cutoff == state.trees[source].clock
                    && s.snapshot.inventory
                        == state.trees[source]
                            .entries
                            .iter()
                            .map(|(p, e)| (p.clone(), e.node.directory))
                            .collect()
            });
            (
                pass,
                if pass {
                    format!("{name}: the snapshot remembers this source's current state")
                } else {
                    format!(
                        "{name}: snapshot is missing, belongs to another source or predates its current contents. Keep the base snapshot separate when making a branch."
                    )
                },
            )
        }
        Requirement::Recovery { .. } => unreachable!("Recovery advances independently in CheckRun"),
    }
}

/// A bounded, cancellable check over an admitted immutable source snapshot.
/// Each tick handles at most one preservation/outcome row or one archive restore.
/// Recovery probes get separate fresh targets; results never mutate learner files.
/// The UI must key its cache by the complete definition and current workspace;
/// changing work discards the run, so results never authorize edited state.
pub struct CheckRun {
    definition: Definition,
    state: Store,
    position: usize,
    total: usize,
    rows: Vec<(bool, String)>,
    probe: usize,
    archive: usize,
    recovery: Option<Tree>,
}
impl CheckRun {
    pub fn new(definition: &Definition, state: &Store) -> Result<Self, String> {
        definition.validate()?;
        if !definition.bound_state(state) || !fits(state) {
            return Err("Backup source/context differs or exceeds its bound; your previous work is retained".into());
        }
        Ok(Self {
            definition: definition.clone(),
            state: state.clone(),
            position: 0,
            total: definition.protected.len() + definition.requirements.len(),
            rows: Vec::with_capacity(54),
            probe: 0,
            archive: 0,
            recovery: None,
        })
    }
    pub fn tick(&mut self) -> bool {
        if self.position >= self.total {
            return true;
        }
        let position = self.position;
        if position < self.definition.protected.len() {
            let p = &self.definition.protected[position];
            self.rows.push(tree_feedback(
                &format!("Preserve {p}"),
                &self.state.trees[p],
                &self.definition.initial.trees[p],
            ));
            self.position += 1;
        } else {
            let requirement =
                &self.definition.requirements[position - self.definition.protected.len()];
            if let Requirement::Recovery {
                name,
                archives,
                root,
                probes,
            } = requirement
            {
                let probe = &probes[self.probe];
                let target = self.recovery.get_or_insert_with(|| probe.initial.clone());
                let path = &archives[self.archive];
                let failure = match self.state.archives.get(path) {
                    None => Some(format!("{path} is missing")),
                    Some(archive) if archive.root != *root => Some(format!(
                        "{path} contains a different top-level folder. List its members and select the backup for {root}."
                    )),
                    Some(archive) => archive.extract(target, true).err(),
                };
                self.archive += 1;
                if failure.is_some() || self.archive == archives.len() {
                    let title = format!("{name} · {}", probe.name);
                    self.rows.push(match failure {
                        Some(error) => (false, format!("{title}: {error}")),
                        None => tree_feedback(&title, target, &probe.expected),
                    });
                    self.probe += 1;
                    self.archive = 0;
                    self.recovery = None;
                    if self.probe == probes.len() {
                        self.probe = 0;
                        self.position += 1;
                    }
                }
            } else {
                self.rows.push(immediate_check(
                    &self.definition.initial,
                    &self.state,
                    requirement,
                ));
                self.position += 1;
            }
        }
        self.done()
    }
    pub fn rows(&self) -> &[(bool, String)] {
        &self.rows
    }
    pub fn done(&self) -> bool {
        self.position == self.total
    }
    pub fn success(&self) -> bool {
        self.done() && self.rows.iter().all(|r| r.0)
    }
}

pub(crate) fn text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<String, D::Error> {
    use serde::de::Visitor;
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "nonblank text <= {N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if !short(s, N) {
                return Err(E::custom("Backup label/path bound"));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(Text::<N>)
}
struct Text1024(String);
impl<'de> Deserialize<'de> for Text1024 {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        text::<D, 1024>(d).map(Self)
    }
}
fn strings<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<Vec<String>, D::Error> {
    let values = list::<D, Text1024, N>(d)?
        .into_iter()
        .map(|v| v.0)
        .collect::<Vec<_>>();
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(serde::de::Error::custom("Duplicate backup path"));
    }
    Ok(values)
}
pub(crate) fn list<'de, D: serde::Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<V>, D::Error> {
    use serde::de::{DeserializeSeed, Error as _, SeqAccess, Visitor};
    struct Next<V> {
        allowed: bool,
        marker: std::marker::PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("Backup row capacity exceeded before body"));
            }
            V::deserialize(d)
        }
    }
    struct Items<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Items<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most{N} backup rows")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next {
                allowed: out.len() < N,
                marker: std::marker::PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<V, N>(std::marker::PhantomData))
}

/// Complete native-attempt admission remains2MiB, including both scopes/history.
pub const MAX_ATTEMPT: usize = 2 * 1024 * 1024;
pub fn has_metadata(a: &crate::progress::Attempt) -> bool {
    a.workspace
        .as_ref()
        .is_some_and(|w| w.file_backup_learning.is_some())
}
pub const MAX_SCENARIO: usize = 256 * 1024;
pub fn size<T: Serialize>(value: &T, cap: usize) -> Option<usize> {
    struct Counter {
        used: usize,
        cap: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.used = self
                .used
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Backup attempt byte bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { used: 0, cap };
    serde_json::to_writer(&mut counter, value).ok()?;
    Some(counter.used)
}
pub fn within<T: Serialize>(value: &T, cap: usize) -> bool {
    size(value, cap).is_some()
}

// No ..: every added metadata field needs an explicit admission decision.
pub(crate) fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        etherchannel_draft,
        etherchannel_learning,
        file_backup_learning: _,
        cron_session,
        cron_learning,
        nfs_session,
        nfs_learning,
        nat_learning,
        rpm_session,
        rpm_learning,
        nat_draft,
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
        script_history,
        script_history_more,
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
        service_port,
        dns_address,
        dns_serial,
        mount_device,
        checked: _,
        ipv4_check,
        practice_check,
        migrated_host,
        legacy_host_reviewed,
        legacy_practice_reviewed,
        migrated_practice_host,
    } = w;
    cron_session.is_some()
        || cron_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || nfs_session.is_some()
        || nfs_learning.is_some()
        || nat_learning.is_some()
        || rpm_session.is_some()
        || rpm_learning.is_some()
        || nat_draft.is_some()
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
        || script_history.is_some()
        || !script_history_more.is_empty()
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
        || service_port.is_some()
        || dns_address.is_some()
        || dns_serial.is_some()
        || mount_device.is_some()
        || ipv4_check.is_some()
        || practice_check.is_some()
        || *migrated_host
        || *legacy_host_reviewed
        || *legacy_practice_reviewed
        || *migrated_practice_host
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Assignment,
    Practice,
}
/// Borrowed admission only; no grade computation or caller mutation.
pub fn context<'a>(
    a: &crate::progress::Attempt,
    s: &'a crate::pack::Scenario,
) -> Result<&'a Definition, String> {
    crate::progress::validate_file_backup_workspace(a)?;
    if !source_shape(s)
        || a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || !within(s, MAX_SCENARIO)
    {
        return Err(
            "Saved backup task version or instructions differ; previous work retained".into(),
        );
    }
    let (
        crate::engine::State::FileBackup { simulation },
        crate::engine::State::FileBackup {
            simulation: authored,
        },
        crate::engine::Goal::FileBackup { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return Err("Backup task and workspace do not match".into());
    };
    if authored != &definition.initial
        || definition.validate().is_err()
        || !definition.bound_state(simulation)
    {
        return Err("Saved backup files do not match the supplied task".into());
    }
    let Some(lesson) = &s.file_backup_teaching else {
        return Err("Backup introduction and separate example are missing".into());
    };
    if !lesson.separate_from(authored)
        || a.workspace
            .as_ref()
            .and_then(|w| w.file_backup_learning.as_ref())
            .is_some_and(|l| !l.valid_for(lesson))
    {
        return Err(
            "Saved backup example does not match this lesson; previous work retained".into(),
        );
    }
    Ok(definition)
}
/// One bounded transaction over the complete attempt. Refusal retains both
/// scopes, drafts, transcript, walkthrough and visit clock, not just the tree.
/// The native caller must reserve8MiB transient space before invoking this path.
pub fn perform(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    scope: Scope,
    input: &str,
) -> Result<String, String> {
    if input.len() > 4096 || input.trim().is_empty() {
        return Err("Type one command of at most4096bytes first".into());
    }
    context(a, s)?;
    let mut proposed = a.clone();
    let output = match scope {
        Scope::Assignment => {
            let out = crate::engine::execute_for_goal(&mut proposed.state, &s.goal, input)?;
            let workspace = proposed
                .workspace
                .get_or_insert_with(crate::workspace::WorkspaceData::new);
            workspace.checked = false;
            proposed
                .transcript
                .push(format!("simulated root $ {input}\n{out}"));
            while proposed.transcript.len() > 120
                || proposed.transcript.iter().map(String::len).sum::<usize>() > 262144
            {
                proposed.transcript.remove(0);
            }
            out
        }
        Scope::Practice => {
            let lesson = s.file_backup_teaching.as_ref().unwrap();
            let learning = proposed
                .workspace
                .as_mut()
                .and_then(|w| w.file_backup_learning.as_mut())
                .ok_or("Open the separate example before running its command")?;
            learning.command = input.into();
            learning.execute(lesson)?
        }
    };
    if !within(&proposed, MAX_ATTEMPT) {
        return Err("This command would exceed the saved backup workspace limit; previous work and drafts retained".into());
    }
    crate::progress::validate_file_backup_workspace(&proposed)?;
    *a = proposed;
    Ok(output)
}

pub fn source_shape(s: &crate::pack::Scenario) -> bool {
    let crate::pack::Scenario {
        file_backup_teaching,
        etherchannel_teaching,
        cron_teaching,
        nfs_teaching,
        nat_teaching,
        rpm_packages_teaching,
        apache_static_teaching,
        spanning_tree_teaching,
        dhcpv4_teaching,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
        teaching,
        terminal_teaching,
        process_teaching,
        accounts_teaching,
        unit_manager_teaching,
        ospf_teaching,
        discovery_teaching,
        storage_teaching,
        acl_teaching,
        ipv6_teaching,
        ipv4_routing_teaching,
        address_teaching,
        service_teaching,
        client_teaching,
        seed_dns_teaching,
        seed_mount_teaching,
        legacy_access_teaching,
        legacy_logs_teaching,
        dns_teaching,
        switching_teaching,
        mac_teaching,
        debian_packages_teaching,
    } = s;
    file_backup_teaching.is_some()
        && cron_teaching.is_none()
        && etherchannel_teaching.is_none()
        && nfs_teaching.is_none()
        && nat_teaching.is_none()
        && rpm_packages_teaching.is_none()
        && apache_static_teaching.is_none()
        && spanning_tree_teaching.is_none()
        && dhcpv4_teaching.is_none()
        && teaching.is_none()
        && terminal_teaching.is_none()
        && process_teaching.is_none()
        && accounts_teaching.is_none()
        && unit_manager_teaching.is_none()
        && ospf_teaching.is_none()
        && discovery_teaching.is_none()
        && storage_teaching.is_none()
        && acl_teaching.is_none()
        && ipv6_teaching.is_none()
        && ipv4_routing_teaching.is_none()
        && address_teaching.is_none()
        && service_teaching.is_none()
        && client_teaching.is_none()
        && seed_dns_teaching.is_none()
        && seed_mount_teaching.is_none()
        && legacy_access_teaching.is_none()
        && legacy_logs_teaching.is_none()
        && dns_teaching.is_none()
        && switching_teaching.is_none()
        && mac_teaching.is_none()
        && debian_packages_teaching.is_none()
        && matches!(
            (&s.initial, &s.goal),
            (State::FileBackup { .. }, Goal::FileBackup { .. })
        )
}
