use crate::{engine::State, pack::Scenario};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub scenario_version: u32,
    pub state: State,
    pub step: usize,
    pub challenge: bool,
    pub hints: usize,
    pub transcript: Vec<String>,
    pub visited: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<crate::ipv4::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<crate::workspace::WorkspaceData>,
}
impl Attempt {
    pub fn new(s: &Scenario, visited: u64) -> Self {
        Self {
            scenario_version: s.version,
            state: s.initial.clone(),
            step: 0,
            challenge: false,
            hints: 0,
            transcript: Vec::new(),
            visited,
            ipv4: None,
            workspace: None,
        }
    }
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub schema_version: u32,
    pub clock: u64,
    pub favorites: BTreeSet<String>,
    pub attempts: BTreeMap<String, Attempt>,
    /// Main-file transition provenance, never portable learning data or credit.
    #[serde(default, skip_serializing)]
    pub migration_receipts: Option<MigrationReceipts>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationReceipts {
    #[serde(default)]
    pub workspace: bool,
    #[serde(default, deserialize_with = "receipt_revisions")]
    pub shell: Vec<u32>,
}
const MAX_MIGRATION_RECEIPTS: usize = 32;

/// Do not trust a serialized sequence's size hint or deserialize an excess item.
/// Domain validity (combined history, record shape, revision order) stays separate.
pub(crate) fn bounded_saved_sequence<'de, D, T, const N: usize>(d: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    struct Excess<const N: usize>;
    impl<'de, const N: usize> Deserialize<'de> for Excess<N> {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Err(D::Error::custom(format!(
                "Saved sequence exceeds {N} items"
            )))
        }
    }
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} saved items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            if a.size_hint().is_some_and(|hint| hint > N) {
                return Err(A::Error::custom(format!(
                    "Saved sequence exceeds {N} items"
                )));
            }
            let mut items = Vec::new();
            while items.len() < N {
                match a.next_element()? {
                    Some(item) => items.push(item),
                    None => return Ok(items),
                }
            }
            // The sentinel rejects immediately if another element exists, before
            // decoding its record/strings or retaining one more vector entry.
            let _ = a.next_element::<Excess<N>>()?;
            Ok(items)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn receipt_revisions<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u32>, D::Error> {
    bounded_saved_sequence::<D, u32, MAX_MIGRATION_RECEIPTS>(d)
}
pub(crate) fn validate_provenance(p: &Progress) -> Result<(), String> {
    if let Some(receipts) = &p.migration_receipts
        && (receipts.shell.len() > MAX_MIGRATION_RECEIPTS
            || receipts
                .shell
                .iter()
                .any(|revision| *revision == 0 || *revision > crate::shell::EVALUATOR_REVISION)
            || receipts.shell.windows(2).any(|pair| pair[0] >= pair[1]))
    {
        return Err("Invalid or future migration provenance; original retained".into());
    }
    Ok(())
}
/// Original analysis credit is visible, but never stands in for a new requirement.
pub fn legacy_credit(a: &Attempt) -> bool {
    a.workspace.as_ref().is_some_and(|w| w.migrated_host)
        || matches!(&a.state, State::Subnet { second: None, .. })
            && crate::ipv4::Exercise::from_state(&a.state).is_some_and(|e| e.solved())
}
/// Earlier script completion remains distinguishable from current evaluator acceptance.
pub fn legacy_script_credit(a: &Attempt, s: &Scenario) -> bool {
    let crate::engine::Goal::Script { cases } = &s.goal else {
        return false;
    };
    let State::Script { run, .. } = &a.state else {
        return false;
    };
    run.as_ref()
        .into_iter()
        .chain(a.workspace.as_ref().and_then(|w| w.script_history.as_ref()))
        .chain(
            a.workspace
                .as_ref()
                .into_iter()
                .flat_map(|w| &w.script_history_more),
        )
        .any(|r| {
            r.engine_revision < crate::shell::EVALUATOR_REVISION
                && r.results.len() == cases.len()
                && r.results
                    .iter()
                    .zip(cases)
                    .all(|(result, case)| case.accepts(result))
        })
}
pub fn current_solved(a: &Attempt, s: &Scenario) -> bool {
    if let State::UnitManager { simulation } = &a.state {
        let (
            State::UnitManager {
                simulation: authored,
            },
            crate::engine::Goal::UnitManager { definition },
        ) = (&s.initial, &s.goal)
        else {
            return false;
        };
        if authored != &definition.initial || !definition.same_assignment(simulation) {
            return false;
        }
        if let Some(draft) = a
            .workspace
            .as_ref()
            .and_then(|w| w.unit_manager_draft.as_ref())
        {
            return crate::unit_manager_workspace::solved(definition, simulation, draft);
        }
        return crate::unit_manager_workspace::Draft::new(simulation).is_ok_and(|draft| {
            crate::unit_manager_workspace::solved(definition, simulation, &draft)
        });
    }
    if let State::Ospf { simulation } = &a.state {
        let (
            State::Ospf {
                simulation: authored,
            },
            crate::engine::Goal::Ospf {
                initial,
                requirements,
            },
        ) = (&s.initial, &s.goal)
        else {
            return false;
        };
        if authored != initial || !crate::ospf_workspace::same_context(initial, simulation) {
            return false;
        }
        if a.workspace
            .as_ref()
            .and_then(|w| w.ospf_draft.as_ref())
            .is_some_and(|d| !d.matches(simulation))
        {
            return false;
        }
        return crate::ospf_workspace::checks(initial, simulation, requirements)
            .iter()
            .all(|c| c.0);
    }
    if let State::Ipv4Acl { network } = &a.state {
        let crate::engine::Goal::Ipv4Acl { definition } = &s.goal else {
            return false;
        };
        if a.workspace
            .as_ref()
            .and_then(|w| w.acl_draft.as_ref())
            .is_some_and(|d| !d.same_shape(network))
        {
            return false;
        }
        let empty = crate::acl_editor::Drafts::default();
        let documents = a
            .workspace
            .as_ref()
            .and_then(|w| w.acl_draft.as_ref())
            .map(|d| &d.documents)
            .unwrap_or(&empty);
        return crate::acl_editor::grade(definition, network, documents).is_ok_and(|r| r.complete);
    }
    if let State::Accounts { simulation } = &a.state {
        let (
            State::Accounts {
                simulation: initial,
            },
            crate::engine::Goal::Accounts {
                initial: goal_initial,
                ..
            },
        ) = (&s.initial, &s.goal)
        else {
            return false;
        };
        if initial != goal_initial || !simulation.valid_against(initial) {
            return false;
        }
    }
    if let State::PersistentMounts { storage } = &a.state {
        let State::PersistentMounts { storage: initial } = &s.initial else {
            return false;
        };
        if !storage.same_inventory(initial)
            || a.workspace
                .as_ref()
                .and_then(|w| w.storage_draft.as_ref())
                .is_some_and(|d| !d.matches(storage))
        {
            return false;
        }
    }
    if let State::Discovery { simulation } = &a.state {
        let State::Discovery {
            simulation: initial,
        } = &s.initial
        else {
            return false;
        };
        if !simulation.topology().same_identity(initial.topology())
            || a.workspace
                .as_ref()
                .and_then(|w| w.discovery_draft.as_ref())
                .is_some_and(|d| !d.matches(simulation))
        {
            return false;
        }
    }
    if let State::Processes { table } = &a.state {
        let State::Processes { table: initial } = &s.initial else {
            return false;
        };
        if !table.same_problem(initial) {
            return false;
        }
    }
    if let State::AuthoritativeDns { server } = &a.state {
        let State::AuthoritativeDns { server: initial } = &s.initial else {
            return false;
        };
        if !crate::dns_workspace::same_problem(server, initial)
            || a.workspace
                .as_ref()
                .and_then(|w| w.dns_draft.as_ref())
                .is_some_and(|d| !d.matches(server))
        {
            return false;
        }
    }
    if let Some(current) = crate::client_teaching::Client::from_state(&a.state)
        && crate::client_teaching::Client::from_state(&s.initial)
            .is_none_or(|initial| !current.same_problem(&initial))
    {
        return false;
    }
    if s.seed_dns_teaching.is_some()
        && !crate::seed_dns_teaching::assignment_context(&a.state, &s.initial)
    {
        return false;
    }
    if s.seed_mount_teaching.is_some()
        && !crate::seed_mount_teaching::assignment_context(&a.state, &s.initial)
    {
        return false;
    }
    if crate::client_teaching::pending_draft(a) {
        return false;
    }
    if crate::workspace::pending_service_draft(a)
        || crate::workspace::pending_seed_configuration_draft(a)
    {
        return false;
    }
    if crate::address_teaching::goal(&s.initial).is_some()
        && !crate::address_teaching::same_assignment(&a.state, &s.initial)
    {
        return false;
    }

    if let State::MacLearning { simulation } = &a.state {
        let (
            State::MacLearning {
                simulation: initial,
            },
            crate::engine::Goal::MacLearning {
                initial: goal_initial,
                ..
            },
        ) = (&s.initial, &s.goal)
        else {
            return false;
        };
        if initial != goal_initial
            || !crate::mac_learning_workspace::same_problem(simulation, initial)
            || a.workspace
                .as_ref()
                .and_then(|w| w.mac_draft.as_ref())
                .is_some_and(|d| !d.matches(simulation))
        {
            return false;
        }
    }
    if let State::Switching { fabric } = &a.state
        && a.workspace
            .as_ref()
            .and_then(|w| w.switching_draft.as_ref())
            .is_some_and(|d| !d.matches(fabric))
    {
        return false;
    }
    if let State::Ipv6 { topology } = &a.state
        && a.workspace
            .as_ref()
            .and_then(|w| w.ipv6_draft.as_ref())
            .is_some_and(|d| !d.matches(topology))
    {
        return false;
    }
    if let State::Ipv4Routing { topology } = &a.state
        && a.workspace
            .as_ref()
            .and_then(|w| w.routing_v4_draft.as_ref())
            .is_some_and(|d| !d.matches(topology))
    {
        return false;
    }
    !matches!(&a.state, State::Subnet { second: None, .. })
        && a.workspace
            .as_ref()
            .is_none_or(|w| !w.migrated_host || w.legacy_host_reviewed)
        && crate::engine::solved(&a.state, &s.goal)
}
pub fn default_path() -> Result<PathBuf, String> {
    directories::ProjectDirs::from("org", "DLMS", "DLMS Learning Lab")
        .map(|d| d.data_local_dir().join("progress-v1.json"))
        .ok_or("No local application data directory".into())
}
pub fn load(path: &Path, scenarios: &[Scenario]) -> Result<Progress, String> {
    if !path.exists() {
        return Ok(Progress {
            schema_version: 1,
            ..Default::default()
        });
    }
    decode(&read_bounded(path, 16_000_000)?, scenarios)
}
pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    crate::local_file::read_bounded(path, limit)
}

pub fn decode(bytes: &[u8], scenarios: &[Scenario]) -> Result<Progress, String> {
    if bytes.len() > 16_000_000 {
        return Err("Progress exceeds 16 MB; original retained".into());
    }
    let p: Progress = serde_json::from_slice(bytes)
        .map_err(|e| format!("Progress unreadable; original retained: {e}"))?;
    if p.schema_version != 1 {
        return Err("Unsupported progress version; original retained".into());
    }
    validate_provenance(&p)?;
    if p.attempts.len() > scenarios.len()
        || p.favorites.len() > scenarios.len()
        || p.favorites
            .iter()
            .any(|id| !scenarios.iter().any(|s| &s.id == id))
    {
        return Err("Unknown or excessive saved labs; original retained".into());
    }
    for (id, a) in &p.attempts {
        let s = scenarios
            .iter()
            .find(|s| &s.id == id)
            .ok_or("Unknown saved scenario; original retained")?;
        if a.scenario_version != s.version
            || std::mem::discriminant(&a.state) != std::mem::discriminant(&s.initial)
            || a.step >= s.steps.len()
            || a.hints > s.hints.len()
            || a.transcript.len() > 120
        {
            return Err("Incompatible saved attempt; original retained".into());
        }
        validate_accounts_workspace(a)?;
        validate_ospf_workspace(a)?;
        validate_unit_manager_workspace(a)?;
        if let State::UnitManager { simulation } = &a.state {
            let (
                State::UnitManager {
                    simulation: initial,
                },
                crate::engine::Goal::UnitManager { definition },
            ) = (&s.initial, &s.goal)
            else {
                return Err(
                    "Saved virtual units belong to another activity; original retained".into(),
                );
            };
            if initial != &definition.initial || !definition.same_assignment(simulation) {
                return Err("Saved unit inventory/descriptor context differs from this activity; original retained".into());
            }
            if let Some(l) = a
                .workspace
                .as_ref()
                .and_then(|w| w.unit_manager_learning.as_ref())
                && s.unit_manager_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            {
                return Err("Saved unit practice origin differs from its authored fixture; original retained".into());
            }
        }
        if let State::Ospf { simulation } = &a.state {
            let State::Ospf {
                simulation: initial,
            } = &s.initial
            else {
                return Err("OSPF state belongs to a different activity; original retained".into());
            };
            if !crate::ospf_workspace::same_context(initial, simulation) {
                return Err("Saved OSPF topology/roles/host context differs from this activity; original retained".into());
            }
            if let Some(l) = a.workspace.as_ref().and_then(|w| w.ospf_learning.as_ref())
                && s.ospf_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            {
                return Err(
                    "Saved OSPF practice context differs from this lesson; original retained"
                        .into(),
                );
            }
        }
        if let State::Accounts { simulation } = &a.state {
            let State::Accounts {
                simulation: initial,
            } = &s.initial
            else {
                unreachable!()
            };
            if !simulation.valid_against(initial) {
                return Err("Saved account identities/protected context differ from this assignment; original retained".into());
            }
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.accounts_learning.as_ref())
            && s.accounts_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        {
            return Err("Saved account example differs from this lesson; original retained".into());
        }
        if let Some(learning) = a
            .workspace
            .as_ref()
            .and_then(|w| w.process_learning.as_ref())
            && (!matches!(a.state, State::Processes { .. })
                || s.process_teaching.as_ref().is_none_or(|lesson| {
                    !learning.table.same_problem(&lesson.fixture)
                        || learning.example_command >= lesson.example.len()
                }))
        {
            return Err("Saved process example differs from this lesson; original retained".into());
        }
        if let (State::Processes { table }, State::Processes { table: initial }) =
            (&a.state, &s.initial)
            && !table.same_problem(initial)
        {
            return Err(
                "Saved process identities differ from this assignment; original retained".into(),
            );
        }
        validate_mac_workspace(a)?;
        if let (
            State::MacLearning { simulation },
            State::MacLearning {
                simulation: initial,
            },
        ) = (&a.state, &s.initial)
            && !crate::mac_learning_workspace::same_problem(simulation, initial)
        {
            return Err(
                "Saved MAC identities differ from this assignment; original retained".into(),
            );
        }
        if let Some(l) = a.workspace.as_ref().and_then(|w| w.mac_learning.as_ref())
            && s.mac_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        {
            return Err("Saved MAC example belongs to another lesson; original retained".into());
        }
        if let (State::Switching { fabric }, State::Switching { fabric: initial }) =
            (&a.state, &s.initial)
            && (!fabric.same_shape(initial)
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.switching_draft.as_ref())
                    .is_some_and(|d| !d.same_shape(fabric)))
        {
            return Err("Saved switch/workstation/cable structure differs from this activity; original retained".into());
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.switching_learning.as_ref())
            && s.switching_teaching
                .as_ref()
                .is_none_or(|lesson| !l.fabric.same_shape(&lesson.fixture))
        {
            return Err(
                "Saved switching example structure differs from this lesson; original retained"
                    .into(),
            );
        }
        if let (State::Ipv6 { topology }, State::Ipv6 { topology: initial }) =
            (&a.state, &s.initial)
            && (!topology.same_shape(initial)
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.ipv6_draft.as_ref())
                    .is_some_and(|d| !d.same_shape(topology)))
        {
            return Err("Saved IPv6 device/interface/link structure differs from this scenario; original retained".into());
        }
        if let State::Ipv4Routing { topology } = &a.state {
            let State::Ipv4Routing { topology: initial } = &s.initial else {
                unreachable!()
            };
            if !topology.same_shape(initial) {
                return Err("Saved IPv4 device/interface/link structure differs from this activity; original retained".into());
            }
        }
        if crate::address_teaching::goal(&s.initial).is_some()
            && !crate::address_teaching::same_assignment(&a.state, &s.initial)
        {
            return Err(
                "Saved addressing assignment inputs differ from this lab; original retained".into(),
            );
        }
        validate_acl_workspace(a)?;
        if let State::Ipv4Acl { network } = &a.state {
            let State::Ipv4Acl { network: initial } = &s.initial else {
                unreachable!()
            };
            if !crate::acl_workspace::same_assignment(network, initial) {
                return Err("Saved ACL topology or host-role context differs from this assignment; original retained".into());
            }
            if let Some(learning) = a.workspace.as_ref().and_then(|w| w.acl_learning.as_ref())
                && s.acl_teaching
                    .as_ref()
                    .is_none_or(|lesson| !learning.same_problem(lesson))
            {
                return Err(
                    "Saved ACL practice context differs from this lesson; original retained".into(),
                );
            }
        }
        validate_storage_workspace(a)?;
        if let (State::PersistentMounts { storage }, State::PersistentMounts { storage: initial }) =
            (&a.state, &s.initial)
            && !storage.same_inventory(initial)
        {
            return Err("Saved storage identities, device aliases or mountpoints differ from this assignment; original retained".into());
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.storage_learning.as_ref())
            && s.storage_teaching
                .as_ref()
                .is_none_or(|lesson| !l.state.same_inventory(&lesson.fixture))
        {
            return Err(
                "Saved storage practice identities differ from this lesson; original retained"
                    .into(),
            );
        }
        validate_service_workspace(a)?;
        validate_client_workspace(a)?;
        validate_seed_workspace(a)?;
        if let Some(current) = crate::client_teaching::Client::from_state(&a.state)
            && crate::client_teaching::Client::from_state(&s.initial)
                .is_none_or(|initial| !current.same_problem(&initial))
        {
            return Err("Saved client router, LAN plan or remote target differ from this assignment; original retained".into());
        }

        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.client_learning.as_ref())
            && s.client_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        {
            return Err("Saved client example differs from this lesson; original retained".into());
        }
        if s.seed_dns_teaching.is_some()
            && !crate::seed_dns_teaching::assignment_context(&a.state, &s.initial)
        {
            return Err(
                "Saved seed DNS record names differ from this assignment; original retained".into(),
            );
        }
        if s.seed_mount_teaching.is_some()
            && !crate::seed_mount_teaching::assignment_context(&a.state, &s.initial)
        {
            return Err(
                "Saved seed mount capacity or known device differ; original retained".into(),
            );
        }
        if a.workspace
            .as_ref()
            .and_then(|w| w.seed_dns_learning.as_ref())
            .is_some_and(|l| {
                s.seed_dns_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            })
        {
            return Err(
                "Saved seed DNS example differs from this lesson; original retained".into(),
            );
        }
        if a.workspace
            .as_ref()
            .and_then(|w| w.seed_mount_learning.as_ref())
            .is_some_and(|l| {
                s.seed_mount_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            })
        {
            return Err(
                "Saved seed mount example differs from this lesson; original retained".into(),
            );
        }
        validate_discovery_workspace(a)?;
        if let (
            State::Discovery { simulation },
            State::Discovery {
                simulation: initial,
            },
        ) = (&a.state, &s.initial)
            && !simulation.topology().same_identity(initial.topology())
        {
            return Err("Saved discovery identities or attachments differ from this assignment; original retained".into());
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.discovery_learning.as_ref())
            && s.discovery_teaching.as_ref().is_none_or(|lesson| {
                !l.simulation
                    .topology()
                    .same_identity(lesson.fixture.topology())
            })
        {
            return Err(
                "Saved discovery practice differs from this lesson; original retained".into(),
            );
        }
        validate_process_workspace(a)?;
        validate_dns_workspace(a)?;
        if let (State::AuthoritativeDns { server }, State::AuthoritativeDns { server: initial }) =
            (&a.state, &s.initial)
            && !crate::dns_workspace::same_problem(server, initial)
        {
            return Err(
                "Saved DNS zone identity differs from this assignment; original retained".into(),
            );
        }
        if let Some(l) = a.workspace.as_ref().and_then(|w| w.dns_learning.as_ref())
            && s.dns_teaching.as_ref().is_none_or(|lesson| {
                !crate::dns_workspace::same_problem(&l.server, &lesson.fixture)
            })
        {
            return Err("Saved DNS example differs from this lesson; original retained".into());
        }
        validate_address_workspace(a)?;
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.address_learning.as_ref())
            && s.address_teaching.as_ref().is_none_or(|lesson| {
                !crate::address_teaching::same_problem(&l.state, &lesson.fixture)
            })
        {
            return Err(
                "Saved addressing example differs from this lesson; original retained".into(),
            );
        }
        if a.workspace
            .as_ref()
            .is_some_and(|w| w.service_learning.is_some())
            && s.service_teaching.is_none()
        {
            return Err(
                "Service practice metadata without authored teaching; original retained".into(),
            );
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.service_learning.as_ref())
            && s.service_teaching
                .as_ref()
                .is_none_or(|lesson| l.command_index >= lesson.commands.len())
        {
            return Err("Saved service example selection is incompatible with this lesson; original retained".into());
        }
        validate_routing_workspace(a)?;
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.routing_v4_learning.as_ref())
            && s.ipv4_routing_teaching
                .as_ref()
                .is_none_or(|lesson| !l.topology.same_shape(&lesson.fixture))
        {
            return Err(
                "Saved IPv4 practice structure differs from this lesson; original retained".into(),
            );
        }
        crate::pack::validate_state(&a.state)?;
        if !crate::workspace::can_preserve_script_history(a) {
            return Err("Saved script history is full; original retained. Keep a copy of the progress file; this profile needs a history migration.".into());
        }
        if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
            return Err("Invalid workspace data; original retained".into());
        }
        if let Some(learning) = a.workspace.as_ref().and_then(|w| w.ipv6_learning.as_ref())
            && s.ipv6_teaching
                .as_ref()
                .is_none_or(|lesson| !learning.topology.same_shape(&lesson.fixture))
        {
            return Err(
                "Saved IPv6 practice structure differs from this lesson; original retained".into(),
            );
        }
        if let Some(l) = &a.ipv4 {
            let original = crate::ipv4::Exercise::from_state(&a.state)
                .ok_or("IPv4 lesson metadata on a different state")?;
            if !crate::ipv4::validate_lesson(l, &original) {
                return Err("Invalid saved IPv4 lesson; original retained".into());
            }
        }
    }
    Ok(p)
}
fn validate_unit_manager_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else {
        return Ok(());
    };
    match &a.state {
        State::UnitManager { simulation } => {
            if w.unit_manager_draft
                .as_ref()
                .is_some_and(|d| !d.same_shape(simulation))
                || w.unit_manager_learning.as_ref().is_some_and(|l| {
                    !l.valid()
                        || l.fixture.inventory().fixture == simulation.inventory().fixture
                        || l.fixture.inventory().units == simulation.inventory().units
                        || l.fixture.inventory().config_paths == simulation.inventory().config_paths
                })
            {
                return Err(
                    "Invalid unit-manager draft/example context; previous progress retained".into(),
                );
            }
        }
        _ if w.unit_manager_draft.is_some()
            || w.unit_manager_learning.is_some()
            || w.unit_manager_terminal.is_some() =>
        {
            return Err(
                "Unit-manager metadata belongs to another activity; previous progress retained"
                    .into(),
            );
        }
        _ => {}
    }
    Ok(())
}
fn validate_ospf_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else {
        return Ok(());
    };
    match &a.state {
        State::Ospf { simulation } => {
            if w.ospf_draft
                .as_ref()
                .is_some_and(|d| !d.same_shape(simulation))
                || w.ospf_learning.as_ref().is_some_and(|l| {
                    !l.valid()
                        || l.simulation
                            .topology()
                            .devices
                            .keys()
                            .any(|id| simulation.topology().devices.contains_key(id))
                })
            {
                return Err("Invalid OSPF draft/example identities; original retained".into());
            }
        }
        _ if w.ospf_draft.is_some() || w.ospf_learning.is_some() => {
            return Err("OSPF metadata on another activity; original retained".into());
        }
        _ => {}
    }
    Ok(())
}
fn validate_acl_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else {
        return Ok(());
    };
    match &a.state {
        State::Ipv4Acl { network } => {
            if w.acl_draft.as_ref().is_some_and(|d| !d.same_shape(network))
                || w.acl_learning.as_ref().is_some_and(|l| !l.valid())
            {
                return Err("Invalid ACL draft/example metadata; original retained".into());
            }
        }
        _ if w.acl_draft.is_some() || w.acl_learning.is_some() => {
            return Err("ACL metadata on another activity; original retained".into());
        }
        _ => {}
    }
    Ok(())
}
fn validate_discovery_workspace(a: &Attempt) -> Result<(), String> {
    if let Some(w) = &a.workspace {
        match &a.state {
            State::Discovery { simulation } => {
                if w.discovery_draft
                    .as_ref()
                    .is_some_and(|d| !d.same_shape(simulation))
                {
                    return Err("Invalid discovery draft identities; original retained".into());
                }
            }
            _ if w.discovery_draft.is_some() || w.discovery_learning.is_some() => {
                return Err("Discovery metadata on another activity; original retained".into());
            }
            _ => {}
        }
    }
    Ok(())
}
fn validate_storage_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.storage_draft.is_some() || w.storage_learning.is_some())
        && !matches!(a.state, State::PersistentMounts { .. })
    {
        return Err("Storage workspace metadata on another activity; original retained".into());
    }
    Ok(())
}
fn validate_accounts_workspace(a: &Attempt) -> Result<(), String> {
    if let Some(l) = a
        .workspace
        .as_ref()
        .and_then(|w| w.accounts_learning.as_ref())
    {
        let State::Accounts { simulation } = &a.state else {
            return Err("Account example metadata on another activity; original retained".into());
        };
        // Only immutable context markers can be compared here. Learners may
        // legitimately create coincident names/IDs in the two independent
        // namespaces; authored initial disjointness is validated by the pack.
        if !l.valid()
            || l.simulation.policy().fixture == simulation.policy().fixture
            || l.simulation.policy().today == simulation.policy().today
        {
            return Err(
                "Invalid or assignment-context account example metadata; original retained".into(),
            );
        }
    }
    Ok(())
}
fn validate_process_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.process_learning.is_some())
        && !matches!(a.state, State::Processes { .. })
    {
        return Err("Process example metadata on a different activity; original retained".into());
    }
    Ok(())
}
fn validate_dns_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.dns_learning.is_some() || w.dns_draft.is_some())
        && !matches!(a.state, State::AuthoritativeDns { .. })
    {
        return Err("DNS editor/practice metadata on another activity; original retained".into());
    }
    Ok(())
}
fn validate_seed_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else { return Ok(()) };
    if let Some(l) = &w.seed_dns_learning {
        let State::Dns { records, .. } = &a.state else {
            return Err("Seed DNS metadata on another activity; original retained".into());
        };
        if !l.valid() || records.len() != 1 || records.contains_key(&l.fixture.name) {
            return Err("Invalid or assignment-context seed DNS example; original retained".into());
        }
    }
    if let Some(l) = &w.seed_mount_learning {
        let State::Mount {
            capacity, device, ..
        } = &a.state
        else {
            return Err("Seed mount metadata on another activity; original retained".into());
        };
        if !l.valid() || l.fixture.capacity == *capacity || l.fixture.device == *device {
            return Err(
                "Invalid or assignment-context seed mount example; original retained".into(),
            );
        }
    }
    Ok(())
}
fn validate_client_workspace(a: &Attempt) -> Result<(), String> {
    if let Some(w) = &a.workspace {
        if (w.client_draft.is_some() || w.client_learning.is_some())
            && !matches!(a.state, State::Network { .. })
        {
            return Err("Client metadata on another activity; original retained".into());
        }
        if w.client_draft.as_ref().is_some_and(|d| !d.valid())
            || w.client_learning.as_ref().is_some_and(|l| !l.valid())
        {
            return Err("Invalid client workspace; original retained".into());
        }
    }
    Ok(())
}
fn validate_service_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.service_learning.is_some())
        && !matches!(a.state, State::Service { .. })
    {
        return Err("Service practice metadata on another activity; original retained".into());
    }
    Ok(())
}
fn validate_address_workspace(a: &Attempt) -> Result<(), String> {
    if let Some(l) = a
        .workspace
        .as_ref()
        .and_then(|w| w.address_learning.as_ref())
        && (crate::address_teaching::goal(&a.state).is_none()
            || std::mem::discriminant(&a.state) != std::mem::discriminant(&l.state))
    {
        return Err(
            "Addressing example metadata on an incompatible activity; original retained".into(),
        );
    }
    Ok(())
}
fn validate_routing_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else {
        return Ok(());
    };
    if w.routing_v4_draft.is_none() && w.routing_v4_learning.is_none() {
        return Ok(());
    }
    let State::Ipv4Routing { topology } = &a.state else {
        return Err("IPv4 routing metadata on a different activity; original retained".into());
    };
    if w.routing_v4_draft
        .as_ref()
        .is_some_and(|d| !d.same_shape(topology))
    {
        return Err("Invalid IPv4 routing draft structure; original retained".into());
    }
    Ok(())
}
fn validate_mac_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else { return Ok(()) };
    if w.mac_draft.is_none() && w.mac_learning.is_none() {
        return Ok(());
    }
    let State::MacLearning { simulation } = &a.state else {
        return Err("MAC metadata on another activity; original retained".into());
    };
    if w.mac_draft
        .as_ref()
        .is_some_and(|d| !d.same_shape(simulation))
        || w.mac_learning.as_ref().is_some_and(|l| !l.valid())
    {
        return Err("Invalid MAC draft or practice; original retained".into());
    }
    Ok(())
}
#[derive(Debug)]
pub struct Saved {
    pub durability_warning: Option<String>,
}
fn validate_for_save(p: &Progress) -> Result<(), String> {
    if p.schema_version != 1 {
        return Err("Unsupported progress version; original retained".into());
    }
    validate_provenance(p)?;
    for a in p.attempts.values() {
        crate::pack::validate_state(&a.state)?;
        validate_mac_workspace(a)?;
        validate_acl_workspace(a)?;
        validate_accounts_workspace(a)?;
        validate_ospf_workspace(a)?;
        validate_unit_manager_workspace(a)?;
        validate_storage_workspace(a)?;
        validate_discovery_workspace(a)?;
        validate_process_workspace(a)?;
        validate_dns_workspace(a)?;
        validate_service_workspace(a)?;
        validate_client_workspace(a)?;
        validate_seed_workspace(a)?;
        validate_address_workspace(a)?;
        validate_routing_workspace(a)?;
        if let State::Switching { fabric } = &a.state
            && a.workspace
                .as_ref()
                .and_then(|w| w.switching_draft.as_ref())
                .is_some_and(|d| !d.same_shape(fabric))
        {
            return Err("Invalid switching draft structure; previous progress retained".into());
        }

        if let State::Ipv6 { topology } = &a.state
            && a.workspace
                .as_ref()
                .and_then(|w| w.ipv6_draft.as_ref())
                .is_some_and(|draft| !draft.same_shape(topology))
        {
            return Err("Invalid IPv6 draft structure; previous progress retained".into());
        }
        if !crate::workspace::can_preserve_script_history(a) {
            return Err("Saved script history is full; original retained. Keep a copy of the progress file; this profile needs a history migration.".into());
        }
        if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
            return Err("Invalid draft data; previous progress retained".into());
        }
    }
    Ok(())
}
#[derive(Serialize)]
struct MainProgress<'a> {
    #[serde(flatten)]
    progress: &'a Progress,
    migration_receipts: MigrationReceipts,
}
enum WriteMode<'a> {
    Normal,
    Restore(&'a [Scenario]),
}
pub fn save(path: &Path, p: &Progress) -> Result<Saved, String> {
    save_with_writer(path, p, WriteMode::Normal, |_| Ok(()), write_atomic)
}
pub(crate) fn save_for_restore(
    path: &Path,
    p: &Progress,
    scenarios: &[Scenario],
    before_replace: impl FnOnce(Option<&[u8]>) -> Result<(), String>,
) -> Result<Saved, String> {
    save_with_writer(
        path,
        p,
        WriteMode::Restore(scenarios),
        before_replace,
        write_atomic,
    )
}
fn compatible_target(bytes: &[u8], mode: &WriteMode<'_>) -> Result<Progress, String> {
    let old: Progress = serde_json::from_slice(bytes)
        .map_err(|error| format!("Original retained; current progress is unreadable: {error}"))?;
    validate_for_save(&old)?;
    if let WriteMode::Restore(scenarios) = mode {
        let _ = decode(bytes, scenarios)?;
    }
    Ok(old)
}
fn save_with_writer(
    path: &Path,
    p: &Progress,
    mode: WriteMode<'_>,
    before_replace: impl FnOnce(Option<&[u8]>) -> Result<(), String>,
    persist: impl FnOnce(&Path, &[u8]) -> Result<Saved, String>,
) -> Result<Saved, String> {
    validate_for_save(p)?;
    let original = match std::fs::symlink_metadata(path) {
        Ok(_) => Some(read_bounded(path, 16_000_000)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "Original retained; current progress cannot be read: {error}"
            ));
        }
    };
    let old = if let Some(bytes) = &original {
        match compatible_target(bytes, &mode) {
            Ok(old) => Some(old),
            Err(error) if matches!(mode, WriteMode::Normal) => return Err(error),
            Err(_) => None, // Explicit restore protects these incompatible RAW bytes separately.
        }
    } else {
        None
    };
    let has_workspace = |p: &Progress| {
        p.attempts
            .values()
            .any(|attempt| attempt.workspace.is_some())
    };
    let mut receipts = old
        .as_ref()
        .and_then(|old| old.migration_receipts.clone())
        .unwrap_or_default();
    let prior_workspace = old.as_ref().is_some_and(has_workspace);
    let shell_copy = old.is_some() && !receipts.shell.contains(&crate::shell::EVALUATOR_REVISION);
    let workspace_copy =
        old.is_some() && !receipts.workspace && !prior_workspace && has_workspace(p);
    // These are pending bytes only. No caller data is stamped, even if copying/persist fails.
    if !receipts.shell.contains(&crate::shell::EVALUATOR_REVISION) {
        receipts.shell.push(crate::shell::EVALUATOR_REVISION);
    }
    receipts.workspace |= prior_workspace || workspace_copy || (old.is_none() && has_workspace(p));
    let encoded = serde_json::to_vec(&MainProgress {
        progress: p,
        migration_receipts: receipts,
    })
    .map_err(|error| error.to_string())?;
    if encoded.len() > 16_000_000 {
        return Err("Progress exceeds 16 MB; original retained".into());
    }
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    if let Some(original) = &original {
        if shell_copy {
            preserve_copy(
                &path.with_extension(format!(
                    "before-shell-v{}.json",
                    crate::shell::EVALUATOR_REVISION
                )),
                original,
            )?;
        }
        if workspace_copy {
            preserve_copy(&path.with_extension("before-workspace.json"), original)?;
        }
    }
    before_replace(original.as_deref())?;
    persist(path, &encoded)
}
fn validate_saved_copy(path: &Path, original: &[u8]) -> Result<(), String> {
    let bytes = read_bounded(path, 16_000_000)?;
    if bytes != original {
        return Err(
            "Original retained; existing migration backup differs from the exact current file"
                .into(),
        );
    }
    Ok(())
}
fn preserve_copy(path: &Path, bytes: &[u8]) -> Result<(), String> {
    preserve_copy_with_race(path, bytes, || Ok(()))
}
fn preserve_copy_with_race(
    path: &Path,
    bytes: &[u8],
    before_persist: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if metadata.file_type().is_file() {
            return validate_saved_copy(path, bytes);
        }
        return Err("Original retained; migration backup path is not an ordinary file".into());
    }
    let mut copy =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing backup directory")?)
            .map_err(|e| e.to_string())?;
    copy.write_all(bytes)
        .and_then(|_| copy.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    before_persist()?;
    match copy.persist_noclobber(path) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            validate_saved_copy(path, bytes)
        }
        Err(error) => Err(format!(
            "Original retained; migration backup failed: {}",
            error.error
        )),
    }
}
pub(crate) fn write_atomic(path: &Path, encoded: &[u8]) -> Result<Saved, String> {
    write_atomic_with_sync(path, encoded, |parent| {
        #[cfg(unix)]
        {
            std::fs::File::open(parent)?.sync_all()
        }
        #[cfg(not(unix))]
        {
            let _ = parent;
            Ok(())
        }
    })
}
fn write_atomic_with_sync(
    path: &Path,
    encoded: &[u8],
    sync: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<Saved, String> {
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(encoded).map_err(|e| e.to_string())?;
    file.flush()
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    let durability_warning=sync(parent).err().map(|e|format!("The file was replaced, but storage durability could not be confirmed: {e}. Keep an additional backup before closing."));
    Ok(Saved { durability_warning })
}

/// Held for the app lifetime. OS locks are released on exit/crash; the lock file is never deleted.
/// Deleting it would let a second process lock another inode while the first still owns its lock.
pub struct ProfileLock {
    _file: std::fs::File,
}
pub fn lock_profile(path: &Path) -> Result<ProfileLock, String> {
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let file = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|e|format!("Progress is read-only because another app instance owns it or this storage cannot lock safely: {e}. Close other copies and reopen this app."))?;
    Ok(ProfileLock { _file: file })
}

#[cfg(test)]
mod durability_tests {
    #[test]
    fn committed_write_reports_sync_warning_instead_of_claiming_original_was_retained() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");
        std::fs::write(&path, b"before").unwrap();
        let saved = super::write_atomic_with_sync(&path, b"after", |_| {
            Err(std::io::Error::other("controlled sync failure"))
        })
        .unwrap();
        assert!(
            saved
                .durability_warning
                .unwrap()
                .contains("file was replaced")
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"after");
    }
}

#[cfg(test)]
mod migration_writer_tests {
    use super::*;
    fn profile() -> Progress {
        Progress {
            schema_version: 1,
            ..Default::default()
        }
    }
    #[test]
    fn first_copy_already_exists_race_compares_exact_bytes() {
        for same in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("migration.json");
            let original = serde_json::to_vec_pretty(&profile()).unwrap();
            let occupied = if same {
                original.clone()
            } else {
                serde_json::to_vec(&profile()).unwrap()
            };
            let result = preserve_copy_with_race(&path, &original, || {
                std::fs::write(&path, &occupied).map_err(|e| e.to_string())
            });
            assert_eq!(result.is_ok(), same);
            assert_eq!(std::fs::read(&path).unwrap(), occupied);
        }
    }
    #[test]
    fn failed_callback_and_main_write_leave_actual_target_and_caller_provenance_unchanged() {
        for fail_callback in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("progress.json");
            let original = serde_json::to_vec_pretty(&profile()).unwrap();
            std::fs::write(&path, &original).unwrap();
            let candidate = profile();
            let persisted = std::cell::Cell::new(false);
            let result = save_with_writer(
                &path,
                &candidate,
                WriteMode::Normal,
                |raw| {
                    assert_eq!(raw, Some(original.as_slice()));
                    if fail_callback {
                        Err("controlled pre-replace failure".into())
                    } else {
                        Ok(())
                    }
                },
                |_, _| {
                    persisted.set(true);
                    Err("controlled failed main persist".into())
                },
            );
            assert!(result.is_err());
            assert_eq!(persisted.get(), !fail_callback);
            assert_eq!(std::fs::read(&path).unwrap(), original);
            assert!(candidate.migration_receipts.is_none());
            assert_eq!(
                std::fs::read(path.with_extension(format!(
                    "before-shell-v{}.json",
                    crate::shell::EVALUATOR_REVISION
                )))
                .unwrap(),
                original
            );
            save(&path, &candidate).unwrap();
        }
    }
    #[test]
    fn committed_sync_warning_keeps_successful_receipt_and_allows_repeat_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        let candidate = profile();
        let original = serde_json::to_vec_pretty(&candidate).unwrap();
        std::fs::write(&path, &original).unwrap();
        let saved = save_with_writer(
            &path,
            &candidate,
            WriteMode::Normal,
            |_| Ok(()),
            |target, encoded| {
                write_atomic_with_sync(target, encoded, |_| {
                    Err(std::io::Error::other("controlled sync failure"))
                })
            },
        )
        .unwrap();
        assert!(
            saved
                .durability_warning
                .unwrap()
                .contains("file was replaced")
        );
        let disk: Progress = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            disk.migration_receipts.unwrap().shell,
            vec![crate::shell::EVALUATOR_REVISION]
        );
        assert!(candidate.migration_receipts.is_none());
        save(&path, &candidate).unwrap();
        assert_eq!(
            std::fs::read(path.with_extension(format!(
                "before-shell-v{}.json",
                crate::shell::EVALUATOR_REVISION
            )))
            .unwrap(),
            original
        );
    }
}

#[cfg(test)]
mod saved_sequence_reader_tests {
    use super::*;
    use std::cell::Cell;
    thread_local! { static DECODED: Cell<usize> = const { Cell::new(0) }; }
    struct Probe;
    impl<'de> Deserialize<'de> for Probe {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let _ = bool::deserialize(d)?;
            DECODED.with(|n| n.set(n.get() + 1));
            Ok(Self)
        }
    }
    fn probes<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Probe>, D::Error> {
        bounded_saved_sequence::<D, Probe, 32>(d)
    }
    #[derive(Deserialize)]
    struct Probes {
        #[serde(deserialize_with = "probes")]
        items: Vec<Probe>,
    }
    #[test]
    fn saved_sequence_never_decodes_or_retains_an_excess_typed_element() {
        DECODED.with(|n| n.set(0));
        let maximum = format!(
            r#"{{"items":[{}]}}"#,
            std::iter::repeat_n("true", 32)
                .collect::<Vec<_>>()
                .join(",")
        );
        let parsed: Probes = serde_json::from_str(&maximum).unwrap();
        assert_eq!(parsed.items.len(), 32);
        assert_eq!(DECODED.with(Cell::get), 32);
        DECODED.with(|n| n.set(0));
        let excess = format!(
            r#"{{"items":[{},{{"deliberately":"not a bool"}}]}}"#,
            std::iter::repeat_n("true", 32)
                .collect::<Vec<_>>()
                .join(",")
        );
        let error = serde_json::from_str::<Probes>(&excess)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("exceeds 32"), "{error}");
        assert_eq!(DECODED.with(Cell::get), 32);
    }
}

#[cfg(test)]
mod revision8_writer_acceptance {
    use super::*;
    const ACTUAL7: &[u8] =
        include_bytes!("../tests/fixtures/shell-v7-synthetic-writer-progress.json");
    fn empty() -> Progress {
        Progress {
            schema_version: 1,
            ..Default::default()
        }
    }
    #[test]
    fn actual7_exact_copy_race_and_failed_persist_do_not_stamp_or_replace_target() {
        assert_eq!(crate::shell::EVALUATOR_REVISION, 8);
        for same in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("snapshot.json");
            let occupied = if same {
                ACTUAL7.to_vec()
            } else {
                serde_json::to_vec(&empty()).unwrap()
            };
            let result = preserve_copy_with_race(&path, ACTUAL7, || {
                std::fs::write(&path, &occupied).map_err(|e| e.to_string())
            });
            assert_eq!(result.is_ok(), same);
            assert_eq!(std::fs::read(path).unwrap(), occupied);
        }
        for callback_fails in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("progress.json");
            std::fs::write(&path, ACTUAL7).unwrap();
            let candidate = empty();
            let result = save_with_writer(
                &path,
                &candidate,
                WriteMode::Normal,
                |raw| {
                    assert_eq!(raw, Some(ACTUAL7));
                    if callback_fails {
                        Err("controlled callback failure".into())
                    } else {
                        Ok(())
                    }
                },
                |_, _| Err("controlled main persistence failure".into()),
            );
            assert!(result.is_err());
            assert_eq!(std::fs::read(&path).unwrap(), ACTUAL7);
            assert_eq!(
                std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
                ACTUAL7
            );
            assert!(candidate.migration_receipts.is_none());
            save(&path, &candidate).unwrap();
            assert_eq!(
                std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
                ACTUAL7
            );
        }
    }
    #[test]
    fn actual7_postreplacement_sync_warning_commits8_receipt_and_retains_exact7_snapshot() {
        assert_eq!(crate::shell::EVALUATOR_REVISION, 8);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        std::fs::write(&path, ACTUAL7).unwrap();
        let candidate = empty();
        let saved = save_with_writer(
            &path,
            &candidate,
            WriteMode::Normal,
            |_| Ok(()),
            |p, b| {
                write_atomic_with_sync(p, b, |_| {
                    Err(std::io::Error::other(
                        "controlled post-replacement sync warning",
                    ))
                })
            },
        )
        .unwrap();
        assert!(saved.durability_warning.is_some());
        assert_eq!(
            std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
            ACTUAL7
        );
        assert!(candidate.migration_receipts.is_none());
        let disk: Progress = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(disk.migration_receipts.as_ref().unwrap().shell, vec![7, 8]);
        save(&path, &disk).unwrap();
        assert_eq!(
            std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
            ACTUAL7
        );
    }
}
