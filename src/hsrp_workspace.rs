//! Focused settled-HSRP learner state. Typed configurations, no IOS/host execution.
use crate::{cron_workspace as bounded, hsrp as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
pub mod goal;
pub use goal::{Concept, Definition, Event, Lesson, Policy, Probe, Worked};
pub mod binding;

pub const MAX_ACTIONS: usize = 128;
pub const MAX_SESSION: usize = 256 * 1024;
pub const MAX_DEFINITION: usize = 128 * 1024;
pub const MAX_CONTEXT: usize = 1024 * 1024;
pub const PREFIX: &str = "SIM HSRP ";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Action {
    Router {
        router: core::RouterId,
        config: core::RouterConfig,
    },
    Gateway {
        client: u8,
        gateway: Ipv4Addr,
    },
}
fn changed(current: &core::State, a: &Action) -> bool {
    match a {
        Action::Router { router, config } => config != &current.router(*router).config,
        Action::Gateway { client, gateway } => current
            .clients()
            .iter()
            .find(|c| c.id == *client)
            .is_none_or(|c| c.gateway != *gateway),
    }
}
fn apply(s: &mut core::State, a: &Action) -> Result<(), String> {
    match a {
        Action::Router { router, config } => s.configure_router(*router, config.clone()),
        Action::Gateway { client, gateway } => s.configure_gateway(*client, *gateway),
    }
    .map_err(|e| e.to_string())
}
fn model(s: &core::State) -> Result<(), String> {
    s.validate().map_err(|e| e.to_string())?;
    if !bounded::within(s, core::MAX_STATE_BYTES) {
        return Err("HSRP model exceeds its16KiB bound".into());
    }
    Ok(())
}
/// Core identity checks intentionally allow faults; the learner boundary does not.
pub fn same_fixed_facts(source: &core::State, current: &core::State) -> Result<bool, String> {
    model(source)?;
    model(current)?;
    Ok(current
        .environment_matches(source)
        .map_err(|e| e.to_string())?
        && core::RouterId::ALL
            .iter()
            .all(|id| source.router(*id).facts == current.router(*id).facts)
        && source
            .clients()
            .iter()
            .zip(current.clients())
            .all(|(a, b)| a.link_up == b.link_up))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterDraft {
    pub participating: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub priority: String,
    pub preempt: bool,
    pub track: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub decrement: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientDraft {
    pub id: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub gateway: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub routers: [RouterDraft; 2],
    #[serde(deserialize_with = "bounded::sequence::<_,ClientDraft,8>")]
    pub clients: Vec<ClientDraft>,
}
impl RouterDraft {
    fn from(c: &core::RouterConfig) -> Self {
        Self {
            participating: c.participating,
            priority: c.priority.to_string(),
            preempt: c.preempt,
            track: c.track_wan_decrement.is_some(),
            decrement: c
                .track_wan_decrement
                .map(|n| n.to_string())
                .unwrap_or_default(),
        }
    }
    fn parse(&self) -> Result<core::RouterConfig, String> {
        let c = core::RouterConfig {
            participating: self.participating,
            priority: self
                .priority
                .trim()
                .parse::<u8>()
                .map_err(|_| "Priority needs a whole number1–255")?,
            preempt: self.preempt,
            track_wan_decrement: if self.track {
                Some(self.decrement.trim().parse::<u8>().map_err(
                    |_| "Tracking decrement needs a positive whole number smaller than priority",
                )?)
            } else {
                None
            },
        };
        c.validate().map_err(|e| e.to_string())?;
        Ok(c)
    }
}
impl Draft {
    pub fn from(s: &core::State) -> Self {
        Self {
            routers: core::RouterId::ALL.map(|id| RouterDraft::from(&s.router(id).config)),
            clients: s
                .clients()
                .iter()
                .map(|c| ClientDraft {
                    id: c.id,
                    gateway: c.gateway.to_string(),
                })
                .collect(),
        }
    }
    pub fn shape(&self, source: &core::State) -> Result<(), String> {
        let text = |s: &str| {
            s.len() <= 16
                && !s
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
        };
        if self.clients.len() != source.clients().len()
            || self
                .clients
                .iter()
                .zip(source.clients())
                .any(|(a, b)| a.id != b.id || !text(&a.gateway))
            || self
                .routers
                .iter()
                .any(|r| !text(&r.priority) || !text(&r.decrement))
        {
            return Err("Foreign or oversized HSRP configuration draft".into());
        }
        Ok(())
    }
    pub fn changes(&self, policy: &Policy, current: &core::State) -> Result<Vec<Action>, String> {
        self.shape(&policy.initial)?;
        let mut result = Vec::new();
        for (index, id) in core::RouterId::ALL.into_iter().enumerate() {
            let config = self.routers[index].parse()?;
            if config != current.router(id).config {
                let a = Action::Router { router: id, config };
                policy.authorize(&a)?;
                result.push(a);
            }
        }
        for c in &self.clients {
            let gateway = c
                .gateway
                .trim()
                .parse::<Ipv4Addr>()
                .map_err(|_| format!("Client{} needs a complete IPv4 gateway", c.id))?;
            let before = current
                .clients()
                .iter()
                .find(|b| b.id == c.id)
                .ok_or("Unknown client")?;
            if gateway != before.gateway {
                let a = Action::Gateway {
                    client: c.id,
                    gateway,
                };
                policy.authorize(&a)?;
                result.push(a);
            }
        }
        Ok(result)
    }
    pub fn insert(&mut self, a: &Action) -> Result<(), String> {
        match a {
            Action::Router { router, config } => {
                self.routers[if *router == core::RouterId::A { 0 } else { 1 }] =
                    RouterDraft::from(config)
            }
            Action::Gateway { client, gateway } => {
                self.clients
                    .iter_mut()
                    .find(|c| c.id == *client)
                    .ok_or("Unknown example client")?
                    .gateway = gateway.to_string()
            }
        };
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub initial: core::State,
    pub current: core::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Action,128>")]
    pub history: Vec<Action>,
    pub example_initial: core::State,
    pub example: core::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Action,128>")]
    pub example_history: Vec<Action>,
    pub draft: Draft,
    pub example_draft: Draft,
    pub view: View,
    pub checked: bool,
    pub example_checked: bool,
    pub selected_example: usize,
}
impl Session {
    pub fn from_json(raw: &[u8], d: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("HSRP raw session exceeds256KiB".into());
        }
        let s: Self = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
        d.bound(&s)?;
        Ok(s)
    }
    pub fn to_json(&self, d: &Definition) -> Result<Vec<u8>, String> {
        d.bound(self)?;
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.shape()?;
        let s = Self {
            version: 1,
            initial: d.policy.initial.clone(),
            current: d.policy.initial.clone(),
            history: vec![],
            example_initial: d.lesson.policy.initial.clone(),
            example: d.lesson.policy.initial.clone(),
            example_history: vec![],
            draft: Draft::from(&d.policy.initial),
            example_draft: Draft::from(&d.lesson.policy.initial),
            view: View::Introduction,
            checked: false,
            example_checked: false,
            selected_example: 0,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<(), String> {
        for s in [
            &self.initial,
            &self.current,
            &self.example_initial,
            &self.example,
        ] {
            model(s)?;
        }
        if self.version != 1
            || self.history.len() > MAX_ACTIONS
            || self.example_history.len() > MAX_ACTIONS
            || self.selected_example >= 12
            || !bounded::within(self, MAX_SESSION)
        {
            return Err("Invalid bounded HSRP learner session".into());
        }
        self.draft.shape(&self.initial)?;
        self.example_draft.shape(&self.example_initial)?;
        // Intrinsic admission is deliberately not authored-source authority.
        if !same_fixed_facts(&self.initial, &self.current)?
            || !same_fixed_facts(&self.example_initial, &self.example)?
        {
            return Err("Persistent HSRP fault facts are forbidden; use a temporary probe".into());
        }
        Ok(())
    }
    pub fn set_draft(
        &mut self,
        d: &Definition,
        draft: Draft,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        draft.shape(if practice {
            &d.lesson.policy.initial
        } else {
            &d.policy.initial
        })?;
        let mut n = self.clone();
        if practice {
            n.example_draft = draft
        } else {
            n.draft = draft
        }
        d.bound(&n)?;
        *self = n;
        Ok(())
    }
    pub fn apply_draft(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        let (policy, current, draft) = if practice {
            (&d.lesson.policy, &self.example, &self.example_draft)
        } else {
            (&d.policy, &self.current, &self.draft)
        };
        let actions = draft.changes(policy, current)?;
        self.apply_actions(d, &actions, practice)
    }
    pub fn apply_action(
        &mut self,
        d: &Definition,
        a: &Action,
        practice: bool,
    ) -> Result<(), String> {
        self.apply_actions(d, std::slice::from_ref(a), practice)
    }
    fn apply_actions(
        &mut self,
        d: &Definition,
        actions: &[Action],
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        if actions.len() > 10 {
            return Err("Too many HSRP configuration changes".into());
        }
        let mut next = self.clone();
        let (policy, current, history, draft) = if practice {
            (
                &d.lesson.policy,
                &mut next.example,
                &mut next.example_history,
                &mut next.example_draft,
            )
        } else {
            (
                &d.policy,
                &mut next.current,
                &mut next.history,
                &mut next.draft,
            )
        };
        for a in actions {
            policy.authorize(a)?;
            if changed(current, a) {
                if history.len() == MAX_ACTIONS {
                    return Err("128 applied configuration changes reached. Selected reset starts a fresh attempt; previous work retained".into());
                }
                apply(current, a)?;
                history.push(a.clone());
            }
        }
        *draft = Draft::from(current);
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn execute(
        &mut self,
        d: &Definition,
        text: &str,
        practice: bool,
    ) -> Result<String, String> {
        if text.len() > 1024 {
            return Err("Typed simulator action exceeds1024bytes".into());
        }
        let raw = text
            .strip_prefix(PREFIX)
            .ok_or("Use native HSRP controls; this is not an IOS command parser")?;
        let action: Action =
            serde_json::from_str(raw).map_err(|e| format!("Invalid typed HSRP action: {e}"))?;
        self.apply_action(d, &action, practice)?;
        Ok("Settled simulator configuration applied; no host network or IOS command ran".into())
    }
    pub fn insert_example(&mut self, d: &Definition, index: usize) -> Result<(), String> {
        d.bound(self)?;
        let w = d.lesson.worked.get(index).ok_or("Unknown worked action")?;
        let mut n = self.clone();
        n.example_draft.insert(&w.action)?;
        n.selected_example = index;
        d.bound(&n)?;
        *self = n;
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        d.bound(self)?;
        let fresh = Self::new(d)?;
        self.example = fresh.example;
        self.example_initial = fresh.example_initial;
        self.example_history.clear();
        self.example_draft = fresh.example_draft;
        self.example_checked = false;
        self.selected_example = 0;
        Ok(())
    }
    /// An inspection snapshot is never written back to either persistent world.
    pub fn inspect(
        &self,
        d: &Definition,
        events: &[core::FaultEvent],
        practice: bool,
    ) -> Result<core::State, String> {
        d.bound(self)?;
        if events.len() > 8 {
            return Err("Inspection is limited to eight settled events".into());
        }
        let mut state = if practice {
            self.example.clone()
        } else {
            self.current.clone()
        };
        for e in events {
            state.fault(*e).map_err(|e| e.to_string())?;
        }
        Ok(state)
    }
}

pub fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    if !matches!(a.state, crate::engine::State::Hsrp { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() {
        return Err("Addressing metadata on HSRP work".into());
    }
    if let Some(w) = &a.workspace
        && !foreign_empty(w)
    {
        return Err("Foreign workspace metadata on HSRP work; original retained".into());
    }
    Ok(())
}
fn foreign_empty(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        one_off_session: _,
        one_off_learning: _,
        physical_link_draft: _,
        physical_link_learning: _,
        file_backup_learning: _,
        etherchannel_draft: _,
        etherchannel_learning: _,
        cron_session: _,
        cron_learning: _,
        nfs_session: _,
        nfs_learning: _,
        nat_learning: _,
        rpm_session: _,
        rpm_learning: _,
        nat_draft: _,
        debian_packages_terminal: _,
        apache_static_draft: _,
        apache_static_learning: _,
        debian_packages_draft: _,
        debian_packages_learning: _,
        spanning_tree_draft: _,
        spanning_tree_learning: _,
        dhcpv4_draft: _,
        dhcpv4_learning: _,
        version: _,
        unit_manager_draft: _,
        unit_manager_learning: _,
        unit_manager_terminal: _,
        ospf_draft: _,
        ospf_learning: _,
        acl_draft: _,
        acl_learning: _,
        script_learning: _,
        script_history: _,
        script_history_more: _,
        terminal_learning: _,
        process_learning: _,
        accounts_learning: _,
        discovery_draft: _,
        discovery_learning: _,
        storage_draft: _,
        storage_learning: _,
        ipv6_draft: _,
        ipv6_learning: _,
        routing_v4_draft: _,
        routing_v4_learning: _,
        address_learning: _,
        service_learning: _,
        client_learning: _,
        seed_dns_learning: _,
        seed_mount_learning: _,
        legacy_access_learning: _,
        legacy_logs_learning: _,
        client_draft: _,
        dns_learning: _,
        dns_draft: _,
        switching_draft: _,
        switching_learning: _,
        mac_draft: _,
        mac_learning: _,
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
    w.one_off_session.is_none()
        && w.one_off_learning.is_none()
        && w.physical_link_draft.is_none()
        && w.physical_link_learning.is_none()
        && w.file_backup_learning.is_none()
        && w.etherchannel_draft.is_none()
        && w.etherchannel_learning.is_none()
        && w.cron_session.is_none()
        && w.cron_learning.is_none()
        && w.nfs_session.is_none()
        && w.nfs_learning.is_none()
        && w.nat_learning.is_none()
        && w.rpm_session.is_none()
        && w.rpm_learning.is_none()
        && w.nat_draft.is_none()
        && w.debian_packages_terminal.is_none()
        && w.apache_static_draft.is_none()
        && w.apache_static_learning.is_none()
        && w.debian_packages_draft.is_none()
        && w.debian_packages_learning.is_none()
        && w.spanning_tree_draft.is_none()
        && w.spanning_tree_learning.is_none()
        && w.dhcpv4_draft.is_none()
        && w.dhcpv4_learning.is_none()
        && w.version == 1
        && w.unit_manager_draft.is_none()
        && w.unit_manager_learning.is_none()
        && w.unit_manager_terminal.is_none()
        && w.ospf_draft.is_none()
        && w.ospf_learning.is_none()
        && w.acl_draft.is_none()
        && w.acl_learning.is_none()
        && w.script_learning.is_none()
        && w.script_history.is_none()
        && w.script_history_more.is_empty()
        && w.terminal_learning.is_none()
        && w.process_learning.is_none()
        && w.accounts_learning.is_none()
        && w.discovery_draft.is_none()
        && w.discovery_learning.is_none()
        && w.storage_draft.is_none()
        && w.storage_learning.is_none()
        && w.ipv6_draft.is_none()
        && w.ipv6_learning.is_none()
        && w.routing_v4_draft.is_none()
        && w.routing_v4_learning.is_none()
        && w.address_learning.is_none()
        && w.service_learning.is_none()
        && w.client_learning.is_none()
        && w.seed_dns_learning.is_none()
        && w.seed_mount_learning.is_none()
        && w.legacy_access_learning.is_none()
        && w.legacy_logs_learning.is_none()
        && w.client_draft.is_none()
        && w.dns_learning.is_none()
        && w.dns_draft.is_none()
        && w.switching_draft.is_none()
        && w.switching_learning.is_none()
        && w.mac_draft.is_none()
        && w.mac_learning.is_none()
        && w.command.is_empty()
        && w.service_port.is_none()
        && w.dns_address.is_none()
        && w.dns_serial.is_none()
        && w.mount_device.is_none()
        && !w.checked
        && w.ipv4_check.is_none()
        && w.practice_check.is_none()
        && !w.migrated_host
        && !w.legacy_host_reviewed
        && !w.legacy_practice_reviewed
        && !w.migrated_practice_host
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    metadata(a)?;
    let (crate::engine::State::Hsrp { simulation }, crate::engine::Goal::Hsrp { definition }) =
        (&a.state, &s.goal)
    else {
        return Err("HSRP work requires its authored Goal".into());
    };
    source(s)?;
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.transcript
            .iter()
            .any(|t| t.len() > 8192 || t.contains('\0'))
        || !bounded::within(&(s, a), MAX_CONTEXT)
    {
        return Err("HSRP complete source/Attempt exceeds its context bound".into());
    }
    definition.bound(simulation)
}
pub fn source(s: &crate::pack::Scenario) -> Result<(), String> {
    let (crate::engine::State::Hsrp { simulation }, crate::engine::Goal::Hsrp { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("HSRP source/Goal kinds differ".into());
    };
    definition.shape()?;
    simulation.validate()?;
    if simulation.version != 1
        || simulation.initial != definition.policy.initial
        || simulation.current != definition.policy.initial
        || simulation.example_initial != definition.lesson.policy.initial
        || simulation.example != definition.lesson.policy.initial
        || !simulation.history.is_empty()
        || !simulation.example_history.is_empty()
        || simulation.draft != Draft::from(&definition.policy.initial)
        || simulation.example_draft != Draft::from(&definition.lesson.policy.initial)
        || simulation.view != View::Introduction
        || simulation.checked
        || simulation.example_checked
        || simulation.selected_example != 0
        || !bounded::within(s, MAX_CONTEXT)
        || !binding::fresh_context(s, simulation)
    {
        return Err(
            "Authored HSRP initial must be the fresh assignment and independent example".into(),
        );
    }
    let crate::pack::Scenario {
        one_off_teaching: _,
        physical_link_teaching: _,
        file_backup_teaching: _,
        etherchannel_teaching: _,
        cron_teaching: _,
        nfs_teaching: _,
        nat_teaching: _,
        rpm_packages_teaching: _,
        apache_static_teaching: _,
        spanning_tree_teaching: _,
        dhcpv4_teaching: _,
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
        teaching: _,
        terminal_teaching: _,
        process_teaching: _,
        accounts_teaching: _,
        unit_manager_teaching: _,
        ospf_teaching: _,
        discovery_teaching: _,
        storage_teaching: _,
        acl_teaching: _,
        ipv6_teaching: _,
        ipv4_routing_teaching: _,
        address_teaching: _,
        service_teaching: _,
        client_teaching: _,
        seed_dns_teaching: _,
        seed_mount_teaching: _,
        legacy_access_teaching: _,
        legacy_logs_teaching: _,
        dns_teaching: _,
        switching_teaching: _,
        mac_teaching: _,
        debian_packages_teaching: _,
    } = s;
    if s.one_off_teaching.is_some()
        || s.physical_link_teaching.is_some()
        || s.file_backup_teaching.is_some()
        || s.etherchannel_teaching.is_some()
        || s.cron_teaching.is_some()
        || s.nfs_teaching.is_some()
        || s.nat_teaching.is_some()
        || s.rpm_packages_teaching.is_some()
        || s.apache_static_teaching.is_some()
        || s.spanning_tree_teaching.is_some()
        || s.dhcpv4_teaching.is_some()
        || s.teaching.is_some()
        || s.terminal_teaching.is_some()
        || s.process_teaching.is_some()
        || s.accounts_teaching.is_some()
        || s.unit_manager_teaching.is_some()
        || s.ospf_teaching.is_some()
        || s.discovery_teaching.is_some()
        || s.storage_teaching.is_some()
        || s.acl_teaching.is_some()
        || s.ipv6_teaching.is_some()
        || s.ipv4_routing_teaching.is_some()
        || s.address_teaching.is_some()
        || s.service_teaching.is_some()
        || s.client_teaching.is_some()
        || s.seed_dns_teaching.is_some()
        || s.seed_mount_teaching.is_some()
        || s.legacy_access_teaching.is_some()
        || s.legacy_logs_teaching.is_some()
        || s.dns_teaching.is_some()
        || s.switching_teaching.is_some()
        || s.mac_teaching.is_some()
        || s.debian_packages_teaching.is_some()
    {
        return Err("Foreign teaching metadata on HSRP source".into());
    }
    Ok(())
}
