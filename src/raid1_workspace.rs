//! Source-bound mirrored-storage exercises. No host devices or subprocesses.
use crate::{
    cron_workspace as bounded, raid1 as core,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
pub const MAX_MODEL: usize = 64 * 1024;
pub const MAX_DEFINITION: usize = 192 * 1024;
pub const MAX_SESSION: usize = 512 * 1024;
pub const MAX_CONTEXT: usize = 1024 * 1024;
pub const PREFIX: &str = "raid1 ";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
    #[serde(default, deserialize_with = "optional_command")]
    pub command: Option<String>,
    pub action: Option<core::Action>,
}
fn optional_command<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,2048>")] String);
    Option::<Text>::deserialize(d).map(|v| v.map(|v| v.0))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub array: String,
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
    pub initial: core::State,
    pub plan: Plan,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,12>")]
    pub worked: Vec<Step>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: core::State,
    pub plan: Plan,
    pub lesson: Lesson,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
    pub status: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub initial: core::State,
    pub current: core::State,
    pub example_initial: core::State,
    pub example: core::State,
    pub view: View,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub example_command: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,32>")]
    pub transcript: Vec<Row>,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,32>")]
    pub example_transcript: Vec<Row>,
    pub checked: bool,
    pub example_checked: bool,
    pub selected_example: usize,
}
fn model(s: &core::State) -> Result<(), String> {
    if !bounded::within(s, MAX_MODEL) {
        return Err(
            "The simulated mirror exceeds its saved-data limit; previous work retained".into(),
        );
    }
    s.validate().map_err(|e| e.to_string())
}
fn plan(initial: &core::State, p: &Plan) -> bool {
    initial.arrays.contains_key(&p.array) && initial.replica_content(&p.array).is_ok()
}
fn checks(initial: &core::State, current: &core::State, p: &Plan) -> Vec<(bool, String)> {
    if model(initial).is_err()
        || model(current).is_err()
        || !current.environment_matches(initial)
        || !plan(initial, p)
    {
        return vec![(false,"The supplied array/device identities or capacities differ; restore this task's saved progress or reset the selected attempt.".into())];
    }
    let health = current.health(&p.array);
    let healthy = matches!(health, Ok(core::Health::Healthy));
    let same = current.replica_content(&p.array).ok() == initial.replica_content(&p.array).ok();
    let mut protected = true;
    for (name, a) in &initial.arrays {
        if name != &p.array {
            protected &= current.arrays.get(name) == Some(a);
            for member in &a.members {
                protected &=
                    current.devices.get(&member.device) == initial.devices.get(&member.device);
            }
        }
    }
    for (path, device) in &initial.devices {
        if device.protected || matches!(device.media, core::Media::Foreign) {
            protected &= current.devices.get(path) == Some(device);
        }
    }
    let retired = initial.arrays[&p.array]
        .members
        .iter()
        .filter(|m| m.role == core::Role::Faulty)
        .all(|m| {
            !current.arrays[&p.array]
                .members
                .iter()
                .any(|n| n.device == m.device)
                && current.devices.get(&m.device) == initial.devices.get(&m.device)
        });
    let mut rows = vec![
        (
            healthy,
            format!(
                "{}: restore two fully synchronized copies. Actual: {}. Adding a replacement alone does not finish recovery.",
                p.array,
                health
                    .map(|h| h.label().to_owned())
                    .unwrap_or_else(|e| e.to_string())
            ),
        ),
        (
            same,
            format!(
                "{}: retain every supplied data segment; redundancy does not replace a backup or prove that original data was correct.",
                p.array
            ),
        ),
    ];
    if initial.arrays.len() > 1
        || initial
            .devices
            .values()
            .any(|d| d.protected || matches!(d.media, core::Media::Foreign))
    {
        rows.push((
            protected,
            "Keep the unrelated mirror, protected devices and their data unchanged.".into(),
        ));
    }
    if initial.arrays[&p.array]
        .members
        .iter()
        .any(|m| m.role == core::Role::Faulty)
    {
        rows.insert(0,(retired,"Detach each originally listed failed member from the target array; keep its old metadata and bytes rather than pretending it was erased.".into()));
    }
    rows
}
// Authoring-only finite witness search. Never called by shape/bound/current grading.
fn reachable(initial: &core::State, p: &Plan) -> bool {
    let mut prepared = initial.clone();
    let failed = prepared.arrays[&p.array]
        .members
        .iter()
        .filter(|m| m.role == core::Role::Faulty)
        .map(|m| m.device.clone())
        .collect::<Vec<_>>();
    for device in failed {
        if prepared
            .apply(&core::Action::Remove {
                array: p.array.clone(),
                device,
            })
            .is_err()
        {
            return false;
        }
    }
    if checks(initial, &prepared, p).iter().all(|r| r.0) {
        return true;
    }
    let mut finish = prepared.clone();
    if finish
        .apply(&core::Action::Advance {
            array: p.array.clone(),
            units: 128,
        })
        .is_ok()
        && checks(initial, &finish, p).iter().all(|r| r.0)
    {
        return true;
    }
    for device in initial.devices.keys() {
        let mut probe = prepared.clone();
        if probe
            .apply(&core::Action::Add {
                array: p.array.clone(),
                device: device.clone(),
            })
            .is_ok()
            && probe
                .apply(&core::Action::Advance {
                    array: p.array.clone(),
                    units: 128,
                })
                .is_ok()
            && checks(initial, &probe, p).iter().all(|r| r.0)
        {
            return true;
        }
    }
    false
}
impl Definition {
    pub fn shape(&self) -> Result<(), String> {
        model(&self.initial)?;
        model(&self.lesson.initial)?;
        if !bounded::within(self, MAX_DEFINITION)
            || !plan(&self.initial, &self.plan)
            || !plan(&self.lesson.initial, &self.lesson.plan)
            || self
                .initial
                .devices
                .keys()
                .any(|p| self.lesson.initial.devices.contains_key(p))
            || self
                .initial
                .arrays
                .keys()
                .any(|p| self.lesson.initial.arrays.contains_key(p))
            || self.lesson.concepts.is_empty()
            || self.lesson.concepts.len() > 6
            || self.lesson.worked.is_empty()
            || self.lesson.worked.len() > 12
            || [&self.lesson.prerequisite_note, &self.lesson.description]
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 1600)
            || self.lesson.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.len() > 100
                    || c.text.trim().is_empty()
                    || c.text.len() > 1200
            })
            || self.lesson.worked.iter().any(|s| {
                s.title.trim().is_empty()
                    || s.title.len() > 120
                    || s.explanation.trim().is_empty()
                    || s.explanation.len() > 800
                    || s.command.is_some() == s.action.is_some()
            })
        {
            return Err("Invalid mirror task, separate example or essential teaching".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if checks(&self.initial, &self.initial, &self.plan)
            .iter()
            .all(|r| r.0)
            || checks(
                &self.lesson.initial,
                &self.lesson.initial,
                &self.lesson.plan,
            )
            .iter()
            .all(|r| r.0)
        {
            return Err("Keep meaningful unfinished work in both mirror tasks".into());
        }
        let mut example = self.lesson.initial.clone();
        for step in &self.lesson.worked {
            if let Some(command) = &step.command {
                if command.len() > 2048 || command.contains(['\0', '\n', '\r']) {
                    return Err("Invalid example command".into());
                }
                let result = example.execute(command).map_err(|e| e.to_string())?;
                if result.status != 0 {
                    return Err(format!("Worked mirror action failed: {}", result.output));
                }
            } else {
                example
                    .apply(step.action.as_ref().unwrap())
                    .map_err(|e| e.to_string())?;
            }
        }
        if !checks(&self.lesson.initial, &example, &self.lesson.plan)
            .iter()
            .all(|r| r.0)
        {
            return Err("Worked mirror sequence does not reach its stated outcomes".into());
        }
        if !reachable(&self.initial, &self.plan) {
            return Err(
                "No eligible replacement/recovery path satisfies the independent mirror task"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.shape()?;
        s.validate()?;
        if s.initial != self.initial
            || s.example_initial != self.lesson.initial
            || s.selected_example >= self.lesson.worked.len()
        {
            return Err("Saved mirror work belongs to a different assignment or example".into());
        }
        if !bounded::within(&(self, s), MAX_CONTEXT) {
            return Err("Mirror task and progress exceed the shared limit".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        checks(&self.initial, &s.current, &self.plan)
    }
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.shape()?;
        let s = Self {
            version: 1,
            initial: d.initial.clone(),
            current: d.initial.clone(),
            example_initial: d.lesson.initial.clone(),
            example: d.lesson.initial.clone(),
            view: View::Introduction,
            command: String::new(),
            example_command: String::new(),
            transcript: vec![],
            example_transcript: vec![],
            checked: false,
            example_checked: false,
            selected_example: 0,
        };
        d.bound(&s)?;
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
            || !self.current.environment_matches(&self.initial)
            || !self.example.environment_matches(&self.example_initial)
            || !bounded::within(self, MAX_SESSION)
            || self.selected_example >= 12
            || [&self.command, &self.example_command]
                .iter()
                .any(|s| s.len() > 2048 || s.contains(['\0', '\n', '\r']))
            || self.transcript.len() > 32
            || self.example_transcript.len() > 32
            || self
                .transcript
                .iter()
                .chain(&self.example_transcript)
                .any(|r| {
                    r.input.len() > 2048
                        || r.output.len() > 8192
                        || r.input.contains('\0')
                        || r.output.contains('\0')
                })
        {
            return Err("Invalid or oversized saved mirror work; original retained".into());
        }
        Ok(())
    }
    pub fn set_command(
        &mut self,
        d: &Definition,
        text: String,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        if text.len() > 2048 || text.contains(['\0', '\n', '\r']) {
            return Err("Use one supported command; previous draft retained".into());
        }
        let mut next = self.clone();
        if practice {
            next.example_command = text;
            next.example_checked = false
        } else {
            next.command = text;
            next.checked = false
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn set_view(&mut self, d: &Definition, view: View) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.view = view;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    fn record(
        &mut self,
        d: &Definition,
        practice: bool,
        input: String,
        output: core::Output,
    ) -> Result<core::Output, String> {
        let rows = if practice {
            &mut self.example_transcript
        } else {
            &mut self.transcript
        };
        if rows.len() == 32 {
            rows.remove(0);
        }
        rows.push(Row {
            input,
            output: output.output.clone(),
            status: output.status,
        });
        if practice {
            self.example_checked = false
        } else {
            self.checked = false
        }
        d.bound(self)?;
        Ok(output)
    }
    pub fn execute(
        &mut self,
        d: &Definition,
        input: &str,
        practice: bool,
    ) -> Result<core::Output, String> {
        d.bound(self)?;
        if input.len() > 2048 || input.contains(['\0', '\n', '\r']) {
            return Err("Use one supported mirror command; nothing ran".into());
        }
        let mut next = self.clone();
        let result = if practice {
            next.example.execute(input)
        } else {
            next.current.execute(input)
        }
        .map_err(|e| e.to_string())?;
        let result = next.record(d, practice, input.into(), result)?;
        *self = next;
        Ok(result)
    }
    pub fn action(
        &mut self,
        d: &Definition,
        a: &core::Action,
        practice: bool,
    ) -> Result<core::Output, String> {
        d.bound(self)?;
        if !matches!(a, core::Action::Advance { .. }) {
            return Err("Use the supported mdadm terminal for member management; the native control only advances simulated recovery".into());
        }
        if !bounded::within(a, 2048) {
            return Err("Oversized mirror action".into());
        }
        let mut next = self.clone();
        let result = (if practice {
            &mut next.example
        } else {
            &mut next.current
        })
        .apply(a)
        .map_err(|e| e.to_string())?;
        let result = next.record(
            d,
            practice,
            "Advance simulated recovery (native action)".into(),
            result,
        )?;
        *self = next;
        Ok(result)
    }
    pub fn example_checks(&self, d: &Definition) -> Vec<(bool, String)> {
        if let Err(e) = d.bound(self) {
            return vec![(false, e)];
        }
        checks(&d.lesson.initial, &self.example, &d.lesson.plan)
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.example = d.lesson.initial.clone();
        next.example_command.clear();
        next.example_transcript.clear();
        next.example_checked = false;
        next.selected_example = 0;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
}

/// Reject old-family metadata rather than treating it as portable RAID1 work.
/// Exact borrowed field checks are generated from WorkspaceData at proposal freeze.
pub fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    if !matches!(a.state, crate::engine::State::Raid1 { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() {
        return Err("Addressing metadata on RAID1 work".into());
    }
    if let Some(w) = &a.workspace
        && !foreign_empty(w)
    {
        return Err("Foreign workspace metadata on RAID1 work; original retained".into());
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
        accounts_template_drafts: _,
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
        && w.accounts_template_drafts.is_none()
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
    let (crate::engine::State::Raid1 { simulation }, crate::engine::Goal::Raid1 { definition }) =
        (&a.state, &s.goal)
    else {
        return Err("RAID1 work requires its authored Goal".into());
    };
    let crate::engine::State::Raid1 {
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
        return Err("RAID1 complete source/Attempt exceeds its bound".into());
    }
    if initial.initial != definition.initial {
        return Err("Source initial differs".into());
    }
    definition.bound(simulation)
}
pub fn source(s: &crate::pack::Scenario) -> Result<(), String> {
    let (crate::engine::State::Raid1 { simulation }, crate::engine::Goal::Raid1 { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("RAID1 source/Goal kinds differ".into());
    };
    definition.shape()?;
    simulation.validate()?;
    if simulation.initial != definition.initial
        || simulation.current != definition.initial
        || simulation.example_initial != definition.lesson.initial
        || simulation.example != definition.lesson.initial
        || simulation.view != View::Introduction
        || !simulation.command.is_empty()
        || !simulation.example_command.is_empty()
        || !simulation.transcript.is_empty()
        || !simulation.example_transcript.is_empty()
        || simulation.checked
        || simulation.example_checked
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
        return Err("Foreign teaching metadata on RAID1 source".into());
    }
    Ok(())
}
