//! Bounded native NTP teaching sessions. All facts are simulated and supplied.
//! Configuration, practice, saved drafts and current outcomes remain distinct.
use crate::{
    cron_workspace as bounded, ntp,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
pub const MAX_SESSION: usize = 512 * 1024;
pub const MAX_DEFINITION: usize = 128 * 1024;
pub const MAX_CONTEXT: usize = 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub client: Ipv4Addr,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,16>")]
    pub allowed_references: Vec<Ipv4Addr>,
    pub must_serve: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub device: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub text: String,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub description: String,
    pub fixture: ntp::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,16>")]
    pub targets: Vec<Target>,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,16>")]
    pub protected: Vec<Ipv4Addr>,
    #[serde(deserialize_with = "bounded::sequence::<_,Command,12>")]
    pub worked: Vec<Command>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub try_change: String,
    pub experiment: Command,
    pub repair: Command,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: ntp::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,16>")]
    pub targets: Vec<Target>,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,16>")]
    pub protected: Vec<Ipv4Addr>,
    pub lesson: Lesson,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub device: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub initial: ntp::State,
    pub current: ntp::State,
    pub example_initial: ntp::State,
    pub example: ntp::State,
    pub selected: Ipv4Addr,
    pub example_selected: Ipv4Addr,
    pub view: View,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub example_command: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,32>")]
    pub transcript: Vec<Row>,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,32>")]
    pub example_transcript: Vec<Row>,
    pub checked: bool,
    pub example_checked: bool,
    pub selected_example: usize,
}
fn model(s: &ntp::State) -> Result<(), String> {
    s.validate().map_err(|e| e.to_string())?;
    if !bounded::within(s, ntp::MAX_STATE_BYTES) {
        return Err("NTP supplied model exceeds32KiB".into());
    }
    Ok(())
}
fn device(s: &ntp::State, ip: Ipv4Addr) -> bool {
    s.nodes
        .get(&ip)
        .is_some_and(|n| matches!(n.role, ntp::Role::Device { .. }))
}
fn first_device(s: &ntp::State) -> Result<Ipv4Addr, String> {
    s.nodes
        .keys()
        .copied()
        .find(|ip| device(s, *ip))
        .ok_or_else(|| "NTP needs a configurable device".into())
}
fn valid_targets(s: &ntp::State, targets: &[Target], protected: &[Ipv4Addr]) -> bool {
    if targets.is_empty() || targets.len() > 16 || protected.len() > 16 {
        return false;
    }
    let mut ids = BTreeSet::new();
    if !targets.iter().all(|r| {
        device(s, r.client)
            && ids.insert(r.client)
            && !r.allowed_references.is_empty()
            && r.allowed_references.len() <= 16
            && r.allowed_references
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                == r.allowed_references.len()
            && r.allowed_references
                .iter()
                .all(|ip| *ip != r.client && s.nodes.contains_key(ip))
    }) {
        return false;
    }
    let mut pinned = BTreeSet::new();
    if !protected
        .iter()
        .all(|ip| device(s, *ip) && pinned.insert(*ip))
    {
        return false;
    }
    ids.extend(pinned);
    // Every supplied configurable device is graded or explicitly protected.
    s.nodes
        .keys()
        .filter(|ip| device(s, **ip))
        .all(|ip| ids.contains(ip))
}
fn requirements(targets: &[Target]) -> Vec<ntp::Requirement> {
    targets
        .iter()
        .map(|r| ntp::Requirement {
            client: r.client,
            allowed_references: r.allowed_references.iter().copied().collect(),
            must_serve: r.must_serve,
        })
        .collect()
}
pub fn outcomes(
    initial: &ntp::State,
    current: &ntp::State,
    targets: &[Target],
    protected: &[Ipv4Addr],
) -> Vec<(bool, String)> {
    if model(initial).is_err()
        || model(current).is_err()
        || !valid_targets(initial, targets, protected)
    {
        return vec![(false, "Invalid supplied NTP context".into())];
    }
    let checks = match ntp::check_with_environment(current, initial, &requirements(targets)) {
        Ok(c) => c,
        Err(e) => return vec![(false, e.to_string())],
    };
    let mut rows = checks
        .into_iter()
        .map(|c| {
            let status = current.status(c.client).unwrap();
            let chain = status
                .chain
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" → ");
            (
                c.passed,
                format!(
                    "{} ({}): {} Current dependency chain: {chain}; derived stratum {}.",
                    current.nodes[&c.client].label, c.client, c.message, status.stratum
                ),
            )
        })
        .collect::<Vec<_>>();
    for ip in protected {
        let same = current.nodes[ip].role == initial.nodes[ip].role
            && current.status(*ip).is_ok_and(|v| v.synchronized);
        let upstream = |role| match role {
            ntp::Role::Device { upstream } => upstream
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into()),
            _ => "read-only reference".into(),
        };
        rows.push((same,format!("{} ({ip}): protected upstream supplied {}; current {}; currently {}. Keep the supplied association and restore its dependency.",initial.nodes[ip].label,upstream(initial.nodes[ip].role),upstream(current.nodes[ip].role),if current.status(*ip).is_ok_and(|v|v.synchronized){"synchronized"}else{"unsynchronized"})));
    }
    rows
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
fn command_shape(c: &Command, s: &ntp::State) -> bool {
    if !device(s, c.device)
        || c.text.is_empty()
        || c.text.len() > 512
        || c.text.chars().any(|c| c.is_control() && c != '\t')
        || c.explanation.is_empty()
        || c.explanation.len() > 800
    {
        return false;
    }
    let w = c.text.split_whitespace().collect::<Vec<_>>();
    match w.as_slice() {
        ["show", "ntp", "status" | "associations"] | ["show", "running-config"] => true,
        ["ntp", "server", ip] | ["no", "ntp", "server", ip] => ip.parse::<Ipv4Addr>().is_ok(),
        _ => false,
    }
}
impl Definition {
    /// Borrowed structural/source admission; no worked-answer execution here.
    pub fn shape(&self) -> Result<(), String> {
        model(&self.initial)?;
        model(&self.lesson.fixture)?;
        let l = &self.lesson;
        if !valid_targets(&self.initial, &self.targets, &self.protected)
            || !valid_targets(&l.fixture, &l.targets, &l.protected)
            || !bounded::within(self, MAX_DEFINITION)
            || l.concepts.is_empty()
            || l.concepts.len() > 6
            || l.concepts.iter().any(|c| {
                c.title.is_empty()
                    || c.title.len() > 100
                    || c.text.is_empty()
                    || c.text.len() > 1200
            })
            || [&l.prerequisite_note, &l.description, &l.try_change]
                .iter()
                .any(|s| s.is_empty() || s.len() > 1600)
            || l.worked.is_empty()
            || l.worked.len() > 12
            || l.worked.iter().any(|c| !command_shape(c, &l.fixture))
            || !command_shape(&l.experiment, &l.fixture)
            || !command_shape(&l.repair, &l.fixture)
            || self
                .initial
                .nodes
                .keys()
                .any(|ip| l.fixture.nodes.contains_key(ip))
        {
            return Err("Invalid or nonseparate bounded NTP lesson".into());
        }
        Ok(())
    }
    /// Authoring gate: practice remains inert until the learner explicitly acts.
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let l = &self.lesson;
        let mut e = l.fixture.clone();
        if all(&outcomes(&l.fixture, &e, &l.targets, &l.protected)) {
            return Err("Practice must require changes".into());
        }
        for c in &l.worked {
            e.execute(c.device, &c.text).map_err(|e| e.to_string())?;
        }
        if !all(&outcomes(&l.fixture, &e, &l.targets, &l.protected)) {
            return Err("Worked example misses outcomes".into());
        }
        let before = e.clone();
        e.execute(l.experiment.device, &l.experiment.text)
            .map_err(|e| e.to_string())?;
        if e == before || all(&outcomes(&l.fixture, &e, &l.targets, &l.protected)) {
            return Err("Practice change must expose failure".into());
        }
        e.execute(l.repair.device, &l.repair.text)
            .map_err(|e| e.to_string())?;
        if !all(&outcomes(&l.fixture, &e, &l.targets, &l.protected)) {
            return Err("Practice repair misses outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.shape()?;
        s.validate()?;
        if s.initial != self.initial
            || s.example_initial != self.lesson.fixture
            || s.selected_example >= self.lesson.worked.len()
        {
            return Err("NTP saved session belongs to another authored world".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        outcomes(&self.initial, &s.current, &self.targets, &self.protected)
    }
    pub fn practice_checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        outcomes(
            &self.lesson.fixture,
            &s.example,
            &self.lesson.targets,
            &self.lesson.protected,
        )
    }
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.shape()?;
        Ok(Self {
            version: 1,
            initial: d.initial.clone(),
            current: d.initial.clone(),
            example_initial: d.lesson.fixture.clone(),
            example: d.lesson.fixture.clone(),
            selected: first_device(&d.initial)?,
            example_selected: first_device(&d.lesson.fixture)?,
            view: View::Introduction,
            command: String::new(),
            example_command: String::new(),
            transcript: vec![],
            example_transcript: vec![],
            checked: false,
            example_checked: false,
            selected_example: 0,
        })
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
            || !ntp::environment_matches(&self.current, &self.initial).map_err(|e| e.to_string())?
            || !ntp::environment_matches(&self.example, &self.example_initial)
                .map_err(|e| e.to_string())?
            || !device(&self.current, self.selected)
            || !device(&self.example, self.example_selected)
            || self.selected_example >= 12
            || self.transcript.len() > 32
            || self.example_transcript.len() > 32
            || [&self.command, &self.example_command]
                .iter()
                .any(|s| s.len() > 512 || s.chars().any(|c| c.is_control() && c != '\t'))
            || self
                .transcript
                .iter()
                .any(|r| !device(&self.current, r.device))
            || self
                .example_transcript
                .iter()
                .any(|r| !device(&self.example, r.device))
            || self
                .transcript
                .iter()
                .chain(&self.example_transcript)
                .any(|r| {
                    r.input.len() > 512
                        || r.output.len() > 8192
                        || r.input.contains('\0')
                        || r.output.contains('\0')
                })
            || !bounded::within(self, MAX_SESSION)
        {
            return Err("Invalid NTP session, environment or byte limit; original retained".into());
        }
        Ok(())
    }
    pub fn set_view(&mut self, d: &Definition, v: View) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.view = v;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn select_device(
        &mut self,
        d: &Definition,
        ip: Ipv4Addr,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        if !device(
            if practice {
                &self.example
            } else {
                &self.current
            },
            ip,
        ) {
            return Err("Select a supplied configurable device".into());
        }
        let mut next = self.clone();
        if practice {
            next.example_selected = ip
        } else {
            next.selected = ip
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn set_command(
        &mut self,
        d: &Definition,
        text: String,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        if text.len() > 512 || text.chars().any(|c| c.is_control() && c != '\t') {
            return Err("Use one NTP command up to512bytes".into());
        }
        let mut next = self.clone();
        if practice {
            next.example_command = text
        } else {
            next.command = text
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn execute(
        &mut self,
        d: &Definition,
        input: &str,
        practice: bool,
    ) -> Result<String, String> {
        d.bound(self)?;
        // SIM select is an explicit app/test navigation helper, never an IOS command.
        if let Some(ip) = input.strip_prefix("SIM select ") {
            let ip = ip
                .parse()
                .map_err(|_| "Select a supplied IPv4 device".to_string())?;
            self.select_device(d, ip, practice)?;
            return Ok("Selected simulated device; no NTP configuration changed".into());
        }
        let mut next = self.clone();
        let selected = if practice {
            next.example_selected
        } else {
            next.selected
        };
        let output = (if practice {
            &mut next.example
        } else {
            &mut next.current
        })
        .execute(selected, input)
        .map_err(|e| e.to_string())?;
        let rows = if practice {
            &mut next.example_transcript
        } else {
            &mut next.transcript
        };
        if rows.len() == 32 {
            rows.remove(0);
        }
        rows.push(Row {
            device: selected,
            input: input.into(),
            output: output.clone(),
        });
        if practice {
            next.example_checked = false
        } else {
            next.checked = false
        }
        d.bound(&next)?;
        *self = next;
        Ok(output)
    }
    pub fn select_example(&mut self, d: &Definition, index: usize) -> Result<(), String> {
        d.bound(self)?;
        if index >= d.lesson.worked.len() {
            return Err("No such separate example command".into());
        }
        let mut next = self.clone();
        next.selected_example = index;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn insert(&mut self, d: &Definition, index: usize) -> Result<(), String> {
        d.bound(self)?;
        let c = d
            .lesson
            .worked
            .get(index)
            .ok_or("No such example command")?;
        let mut next = self.clone();
        next.example_selected = c.device;
        next.example_command = c.text.clone();
        next.selected_example = index;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.example = d.lesson.fixture.clone();
        next.example_selected = first_device(&next.example)?;
        next.example_command.clear();
        next.example_transcript.clear();
        next.example_checked = false;
        next.selected_example = 0;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
}
/// Reject old-family metadata rather than treating it as portable NTP work.
/// Exact borrowed field checks are generated from WorkspaceData at proposal freeze.
pub fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    if !matches!(a.state, crate::engine::State::Ntp { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() {
        return Err("Addressing metadata on NTP work".into());
    }
    if let Some(w) = &a.workspace
        && !foreign_empty(w)
    {
        return Err("Foreign workspace metadata on NTP work; original retained".into());
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
    let (crate::engine::State::Ntp { simulation }, crate::engine::Goal::Ntp { definition }) =
        (&a.state, &s.goal)
    else {
        return Err("NTP work requires its authored Goal".into());
    };
    let crate::engine::State::Ntp {
        simulation: initial,
    } = &s.initial
    else {
        return Err("Wrong source kind".into());
    };
    source(s)?;
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.transcript.iter().any(|t| t.len() > 8192)
        || !bounded::within(&(s, a), MAX_CONTEXT)
    {
        return Err("NTP complete source/Attempt exceeds its bound".into());
    }
    if initial.initial != definition.initial {
        return Err("Source initial differs".into());
    }
    definition.bound(simulation)
}
pub fn source(s: &crate::pack::Scenario) -> Result<(), String> {
    let (crate::engine::State::Ntp { simulation }, crate::engine::Goal::Ntp { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("NTP source/Goal kinds differ".into());
    };
    definition.shape()?;
    simulation.validate()?;
    if simulation.initial != definition.initial
        || simulation.current != definition.initial
        || simulation.example_initial != definition.lesson.fixture
        || simulation.example != definition.lesson.fixture
        || simulation.view != View::Introduction
        || !simulation.command.is_empty()
        || !simulation.example_command.is_empty()
        || !simulation.transcript.is_empty()
        || !simulation.example_transcript.is_empty()
        || simulation.checked
        || simulation.example_checked
        || simulation.selected != first_device(&definition.initial)?
        || simulation.example_selected != first_device(&definition.lesson.fixture)?
        || simulation.selected_example != 0
    {
        return Err("Authored initial must be the fresh assignment and independent example".into());
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
        return Err("Foreign teaching metadata on NTP source".into());
    }
    Ok(())
}
