//! A bounded growth session over the pure LVM model. No host devices or processes.
//! Assignment and practice use separate state; goals never trust saved success.
use crate::{
    cron_workspace as bounded, lvm,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_MODEL: usize = 64 * 1024;
pub const MAX_SESSION: usize = 512 * 1024;
pub const MAX_DEFINITION: usize = 192 * 1024;
pub const MAX_CONTEXT: usize = 1024 * 1024;
pub const MAX_MAPPED_EXTENTS: usize = 128;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub volume: String,
    pub minimum_lv: u64,
    pub minimum_filesystem: u64,
    pub fill_lv: bool,
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
    pub fixture: lvm::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,4>")]
    pub targets: Vec<Target>,
    #[serde(deserialize_with = "bounded::sequence::<_,Command,12>")]
    pub worked: Vec<Command>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub try_change: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub experiment: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub repair: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub text: String,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: lvm::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,4>")]
    pub targets: Vec<Target>,
    pub lesson: Lesson,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub initial: lvm::State,
    pub current: lvm::State,
    pub example_initial: lvm::State,
    pub example: lvm::State,
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
fn model(s: &lvm::State) -> Result<(), String> {
    if s.groups
        .values()
        .flat_map(|g| g.volumes.values())
        .map(|v| v.extents.len())
        .sum::<usize>()
        > MAX_MAPPED_EXTENTS
        || !bounded::within(s, MAX_MODEL)
    {
        return Err("This teaching workspace admits at most128 mapped extents and64KiB per model; previous work retained".into());
    }
    s.validate().map_err(|e| e.to_string())
}
/// Immutable inventory and allocation-prefix context, not a success predicate.
/// Blank devices may legitimately join a wrong supplied VG; feedback then exposes
/// the changed protected context. No wrong answer is silently normalized.
pub fn same_problem(initial: &lvm::State, current: &lvm::State) -> bool {
    if model(initial).is_err()
        || model(current).is_err()
        || initial.devices.keys().ne(current.devices.keys())
        || initial.groups.keys().ne(current.groups.keys())
    {
        return false;
    }
    for (path, before) in &initial.devices {
        let after = &current.devices[path];
        if before.bytes != after.bytes
            || match before.role {
                lvm::DeviceRole::Foreign => before != after,
                lvm::DeviceRole::PhysicalVolume => {
                    after.role != before.role
                        || (!before.group.is_empty() && after.group != before.group)
                }
                lvm::DeviceRole::Blank => after.role == lvm::DeviceRole::Foreign,
            }
        {
            return false;
        }
    }
    for (name, before) in &initial.groups {
        let after = &current.groups[name];
        if before.extent_bytes != after.extent_bytes
            || before.volumes.keys().ne(after.volumes.keys())
        {
            return false;
        }
        for (name, v) in &before.volumes {
            let a = &after.volumes[name];
            if !a.extents.starts_with(&v.extents)
                || a.filesystem.is_some() != v.filesystem.is_some()
                || v.filesystem
                    .as_ref()
                    .zip(a.filesystem.as_ref())
                    .is_some_and(|(v, a)| v.files.keys().ne(a.files.keys()))
            {
                return false;
            }
        }
    }
    true
}
fn valid_targets(initial: &lvm::State, targets: &[Target]) -> bool {
    if targets.is_empty() || targets.len() > 4 {
        return false;
    }
    let mut ids = BTreeSet::new();
    targets.iter().all(|t| {
        ids.insert(t.volume.strip_prefix("/dev/").unwrap_or(&t.volume))
            && t.minimum_lv > 0
            && t.minimum_filesystem > 0
            && t.minimum_lv >= t.minimum_filesystem
            && t.minimum_lv <= 128 * 64 * lvm::MIB
            && initial
                .volume(&t.volume)
                .is_ok_and(|(_, v)| v.filesystem.is_some())
    })
}
fn outcomes(initial: &lvm::State, current: &lvm::State, targets: &[Target]) -> Vec<(bool, String)> {
    if !same_problem(initial, current) || !valid_targets(initial, targets) {
        return vec![(
            false,
            "The supplied device, group, volume or allocation-prefix context differs".into(),
        )];
    }
    let mut rows = Vec::new();
    for t in targets {
        let before = initial
            .volume(&t.volume)
            .unwrap()
            .1
            .filesystem
            .as_ref()
            .unwrap();
        rows.extend(
            lvm::growth_checks(
                current,
                &t.volume,
                t.minimum_lv,
                t.minimum_filesystem,
                &before.files,
            )
            .into_iter()
            .enumerate()
            .map(|(index, (b, text))| {
                let detail = match index {
                    0 => format!(
                        "Current LV {} bytes; required at least {} bytes",
                        current.lv_bytes(&t.volume).unwrap(),
                        t.minimum_lv
                    ),
                    1 => format!(
                        "Current filesystem {} bytes; required at least {} bytes",
                        current.filesystem_bytes(&t.volume).unwrap(),
                        t.minimum_filesystem
                    ),
                    _ => text,
                };
                (b, format!("{}: {detail}", t.volume))
            }),
        );
        if t.fill_lv {
            rows.push((
                current.lv_bytes(&t.volume) == current.filesystem_bytes(&t.volume),
                format!("{}: filesystem uses the full current LV", t.volume),
            ));
        }
    }
    let mut protected = true;
    for (vg, g) in &initial.groups {
        for (lv, v) in &g.volumes {
            if !targets
                .iter()
                .any(|t| t.volume == format!("{vg}/{lv}") || t.volume == format!("/dev/{vg}/{lv}"))
            {
                protected &= current.groups[vg].volumes[lv] == *v;
            }
        }
    }
    let target_groups = targets
        .iter()
        .filter_map(|t| {
            t.volume
                .strip_prefix("/dev/")
                .unwrap_or(&t.volume)
                .split_once('/')
                .map(|v| v.0)
        })
        .collect::<BTreeSet<_>>();
    for (path, before) in &initial.devices {
        let after = &current.devices[path];
        if before.group.is_empty()
            && before != after
            && !after.group.is_empty()
            && !target_groups.contains(after.group.as_str())
        {
            protected = false;
        }
    }
    rows.push((
        protected,
        "Sibling volumes, peer data and unrelated device membership remain unchanged".into(),
    ));
    rows
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|v| v.0)
}
impl Definition {
    /// Structural admission only: no reference execution on idle/current grading.
    pub fn shape(&self) -> Result<(), String> {
        model(&self.initial)?;
        model(&self.lesson.fixture)?;
        if !valid_targets(&self.initial, &self.targets)
            || !valid_targets(&self.lesson.fixture, &self.lesson.targets)
            || !bounded::within(self, MAX_DEFINITION)
            || self.lesson.concepts.is_empty()
            || self.lesson.concepts.len() > 6
            || [
                &self.lesson.prerequisite_note,
                &self.lesson.description,
                &self.lesson.try_change,
            ]
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 1600)
            || self.lesson.concepts.iter().any(|v| {
                v.title.trim().is_empty()
                    || v.title.len() > 100
                    || v.text.trim().is_empty()
                    || v.text.len() > 1200
            })
            || self.lesson.worked.is_empty()
            || self.lesson.worked.len() > 12
            || self.lesson.worked.iter().any(|v| {
                v.text.trim().is_empty()
                    || v.text.len() > 2048
                    || v.explanation.trim().is_empty()
                    || v.explanation.len() > 800
            })
            || [&self.lesson.experiment, &self.lesson.repair]
                .iter()
                .any(|v| v.trim().is_empty() || v.len() > 2048)
            || self
                .initial
                .devices
                .keys()
                .any(|p| self.lesson.fixture.devices.contains_key(p))
            || self
                .initial
                .groups
                .keys()
                .any(|p| self.lesson.fixture.groups.contains_key(p))
        {
            return Err("Invalid or nonseparate bounded LVM assignment/practice definition".into());
        }
        for text in self
            .lesson
            .worked
            .iter()
            .map(|c| c.text.as_str())
            .chain([self.lesson.experiment.as_str(), self.lesson.repair.as_str()])
        {
            crate::lvm::validate_command(text).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    /// Authoring gate only; the native learner must explicitly run each command.
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let l = &self.lesson;
        let mut example = l.fixture.clone();
        if all(&outcomes(&l.fixture, &example, &l.targets)) {
            return Err("Practice must require real changes".into());
        }
        for c in &l.worked {
            example.execute(&c.text).map_err(|e| e.to_string())?;
        }
        if !all(&outcomes(&l.fixture, &example, &l.targets)) {
            return Err("Worked example misses its outcomes".into());
        }
        let before = example.clone();
        example.execute(&l.experiment).map_err(|e| e.to_string())?;
        if example == before || all(&outcomes(&l.fixture, &example, &l.targets)) {
            return Err(
                "Practice experiment must change state and expose a real incomplete outcome".into(),
            );
        }
        example.execute(&l.repair).map_err(|e| e.to_string())?;
        if !all(&outcomes(&l.fixture, &example, &l.targets)) {
            return Err("Practice repair misses its outcomes".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if self.bound(s).is_err() {
            return vec![(
                false,
                "LVM source/session context is invalid; previous work retained".into(),
            )];
        }
        outcomes(&self.initial, &s.current, &self.targets)
    }
    pub fn practice_checks(&self, s: &Session) -> Vec<(bool, String)> {
        if self.bound(s).is_err() {
            return vec![(false, "Separate practice context differs".into())];
        }
        outcomes(&self.lesson.fixture, &s.example, &self.lesson.targets)
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.shape()?;
        s.validate()?;
        if s.initial != self.initial
            || s.example_initial != self.lesson.fixture
            || s.selected_example >= self.lesson.worked.len()
        {
            return Err("LVM session belongs to another authored assignment/practice".into());
        }
        Ok(())
    }
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.shape()?;
        let s = Self {
            version: 1,
            initial: d.initial.clone(),
            current: d.initial.clone(),
            example_initial: d.lesson.fixture.clone(),
            example: d.lesson.fixture.clone(),
            view: View::Introduction,
            command: String::new(),
            example_command: String::new(),
            transcript: vec![],
            example_transcript: vec![],
            checked: false,
            example_checked: false,
            selected_example: 0,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !same_problem(&self.initial, &self.current)
            || !same_problem(&self.example_initial, &self.example)
            || self.command.len() > 2048
            || self.example_command.len() > 2048
            || self.command.contains(['\0', '\n', '\r'])
            || self.example_command.contains(['\0', '\n', '\r'])
            || self.transcript.len() > 32
            || self.example_transcript.len() > 32
            || self.selected_example >= 12
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
            || !bounded::within(self, MAX_SESSION)
        {
            return Err("Invalid or oversized LVM session; previous work retained".into());
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
            return Err("Command draft is too large or contains a line break; nothing ran".into());
        }
        let mut next = self.clone();
        if practice {
            next.example_command = text;
        } else {
            next.command = text;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
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
    pub fn set_view(&mut self, d: &Definition, view: View) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.view = view;
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
        if input.len() > 2048 || input.contains(['\0', '\n', '\r']) {
            return Err("Use one LVM command up to2048bytes; previous work retained".into());
        }
        let mut next = self.clone();
        let output = (if practice {
            &mut next.example
        } else {
            &mut next.current
        })
        .execute(input)
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
            input: input.into(),
            output: output.clone(),
        });
        if practice {
            next.example_checked = false;
        } else {
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        Ok(output)
    }
    pub fn insert(&mut self, d: &Definition, index: usize) -> Result<(), String> {
        d.bound(self)?;
        let c = d
            .lesson
            .worked
            .get(index)
            .ok_or("No such separate example command")?;
        let mut next = self.clone();
        next.selected_example = index;
        next.example_command = c.text.clone();
        next.example_checked = false;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.example = d.lesson.fixture.clone();
        next.example_command.clear();
        next.example_transcript.clear();
        next.example_checked = false;
        next.selected_example = 0;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
}
/// Reject old-family metadata rather than treating it as portable LVM work.
/// Exact borrowed field checks are generated from WorkspaceData at proposal freeze.
pub fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    if !matches!(a.state, crate::engine::State::Lvm { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() {
        return Err("Addressing metadata on LVM work".into());
    }
    if let Some(w) = &a.workspace
        && !foreign_empty(w)
    {
        return Err("Foreign workspace metadata on LVM work; original retained".into());
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
    let (crate::engine::State::Lvm { simulation }, crate::engine::Goal::Lvm { definition }) =
        (&a.state, &s.goal)
    else {
        return Err("LVM work requires its authored Goal".into());
    };
    let crate::engine::State::Lvm {
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
        return Err("LVM complete source/Attempt exceeds its bound".into());
    }
    if initial.initial != definition.initial {
        return Err("Source initial differs".into());
    }
    definition.bound(simulation)
}
pub fn source(s: &crate::pack::Scenario) -> Result<(), String> {
    let (crate::engine::State::Lvm { simulation }, crate::engine::Goal::Lvm { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("LVM source/Goal kinds differ".into());
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
        return Err("Foreign teaching metadata on LVM source".into());
    }
    Ok(())
}
