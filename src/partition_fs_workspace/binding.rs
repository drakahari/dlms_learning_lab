//! Bind every disk action to the held authored Scenario; saved witnesses are not authority.
use super::*;
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};

pub fn metadata(a: &Attempt) -> Result<(), String> {
    if !matches!(a.state, State::PartitionFs { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() || a.workspace.as_ref().is_some_and(|w| !foreign_empty(w)) {
        return Err("Foreign historical teaching/draft/check metadata on disk preparation work; original retained".into());
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
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::PartitionFs { simulation }, Goal::PartitionFs { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Disk preparation source and Goal kinds differ".into());
    };
    definition.shape()?;
    if !definition.fresh(simulation)
        || !bounded::within(s, MAX_CONTEXT)
        || !fresh_context(s, simulation)
    {
        return Err(
            "Authored disk initial must be fresh, bounded assignment and independent practice"
                .into(),
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
        return Err("Foreign teaching metadata on partition/filesystem source".into());
    }
    Ok(definition)
}
pub fn context<'a>(a: &Attempt, s: &'a Scenario) -> Result<&'a Definition, String> {
    metadata(a)?;
    let d = source(s)?;
    let State::PartitionFs { simulation } = &a.state else {
        return Err("Disk source requires its typed disk session".into());
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.transcript
            .iter()
            .any(|v| v.len() > 8192 || v.contains('\0'))
        || !bounded::within(&(s, a), MAX_CONTEXT)
    {
        return Err(
            "Complete disk source/Attempt or saved metadata exceeds its authored bounds".into(),
        );
    }
    d.bound(simulation)?;
    Ok(d)
}
#[derive(Serialize)]
#[serde(tag = "kind")]
enum StateRef<'a> {
    PartitionFs { simulation: &'a Session },
}
#[derive(Serialize)]
struct AttemptRef<'a> {
    scenario_version: u32,
    state: StateRef<'a>,
    step: usize,
    challenge: bool,
    hints: usize,
    transcript: &'a [String],
    visited: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv4: Option<&'a crate::ipv4::Lesson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace: Option<&'a crate::workspace::WorkspaceData>,
}
/// Fresh launch with the widest recency and default generic workspace must fit
/// already at source admission, before any new Attempt/State clone.
fn fresh_context(s: &Scenario, simulation: &Session) -> bool {
    let blank = crate::workspace::WorkspaceData::new();
    let p = AttemptRef {
        scenario_version: s.version,
        state: StateRef::PartitionFs { simulation },
        step: 0,
        challenge: false,
        hints: 0,
        transcript: &[],
        visited: u64::MAX,
        ipv4: None,
        workspace: Some(&blank),
    };
    bounded::within(&(s, &p), MAX_CONTEXT)
}
fn projected<'a>(
    a: &'a Attempt,
    state: &'a Session,
    hints: usize,
    workspace: Option<&'a crate::workspace::WorkspaceData>,
) -> AttemptRef<'a> {
    AttemptRef {
        scenario_version: a.scenario_version,
        state: StateRef::PartitionFs { simulation: state },
        step: a.step,
        challenge: a.challenge,
        hints,
        transcript: &a.transcript,
        visited: a.visited,
        ipv4: a.ipv4.as_ref(),
        workspace,
    }
}
fn admit(
    a: &Attempt,
    s: &Scenario,
    state: &Session,
    hints: usize,
    workspace: Option<&crate::workspace::WorkspaceData>,
) -> Result<(), String> {
    if !bounded::within(&(s, &projected(a, state, hints, workspace)), MAX_CONTEXT) {
        return Err("Saved disk work has no room for this edit; every old value retained and nothing applied".into());
    }
    Ok(())
}
/// The complete old source/Attempt is admitted before a bounded candidate copy.
/// Candidate projection includes every inactive draft and all caller metadata.
fn transaction(
    a: &mut Attempt,
    s: &Scenario,
    update: impl FnOnce(&mut Session, &Definition) -> Result<(), String>,
) -> Result<(), String> {
    let d = context(a, s)?;
    let State::PartitionFs { simulation } = &a.state else {
        unreachable!()
    };
    let mut candidate = simulation.as_ref().clone();
    update(&mut candidate, d)?;
    d.bound(&candidate)?;
    admit(a, s, &candidate, a.hints, a.workspace.as_ref())?;
    a.state = State::PartitionFs {
        simulation: Box::new(candidate),
    };
    Ok(())
}
pub fn prepare(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    context(a, s)?;
    if a.workspace.is_some() {
        return Ok(());
    }
    let State::PartitionFs { simulation } = &a.state else {
        unreachable!()
    };
    let blank = crate::workspace::WorkspaceData::new();
    admit(a, s, simulation, a.hints, Some(&blank))?;
    a.workspace = Some(blank);
    Ok(())
}
pub fn set_view(a: &mut Attempt, s: &Scenario, view: View) -> Result<(), String> {
    transaction(a, s, |w, d| w.set_view(d, view))
}
pub fn select_disk(
    a: &mut Attempt,
    s: &Scenario,
    path: &str,
    practice: bool,
) -> Result<(), String> {
    if !disk_ok(path) {
        return Err("Choose an exact supplied disk".into());
    }
    transaction(a, s, |w, d| w.select_disk(d, path, practice))
}
pub fn set_draft(
    a: &mut Attempt,
    s: &Scenario,
    path: &str,
    draft: Draft,
    practice: bool,
) -> Result<(), String> {
    if !disk_ok(path) {
        return Err("Choose an exact supplied disk".into());
    }
    draft.validate()?;
    transaction(a, s, move |w, d| w.set_draft(d, path, draft, practice))
}
pub fn apply(
    a: &mut Attempt,
    s: &Scenario,
    operation: &Operation,
    practice: bool,
) -> Result<(), String> {
    operation.shape()?;
    transaction(a, s, |w, d| w.apply(d, operation, practice).map(|_| ()))
}
pub fn check(a: &mut Attempt, s: &Scenario, practice: bool) -> Result<(), String> {
    transaction(a, s, |w, d| w.check(d, practice))
}
pub fn select_example(a: &mut Attempt, s: &Scenario, index: usize) -> Result<(), String> {
    transaction(a, s, |w, d| w.select_example(d, index))
}
pub fn insert(a: &mut Attempt, s: &Scenario, index: usize) -> Result<(), String> {
    transaction(a, s, |w, d| w.insert(d, index))
}
pub fn reset_example(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    transaction(a, s, |w, d| w.reset_example(d))
}
pub fn request_hint(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    context(a, s)?;
    let next = a
        .hints
        .checked_add(1)
        .filter(|n| *n <= s.hints.len())
        .ok_or("No further hint is available")?;
    let State::PartitionFs { simulation } = &a.state else {
        unreachable!()
    };
    admit(a, s, simulation, next, a.workspace.as_ref())?;
    a.hints = next;
    Ok(())
}
/// Preflight launch/recency, including a previously absent generic workspace,
/// before the application clones the fresh source or changes its clock.
pub fn admit_launch(previous: Option<&Attempt>, s: &Scenario, visited: u64) -> Result<(), String> {
    source(s)?;
    let blank = crate::workspace::WorkspaceData::new();
    let empty: &[String] = &[];
    let mut p = if let Some(a) = previous {
        context(a, s)?;
        let State::PartitionFs { simulation } = &a.state else {
            unreachable!()
        };
        projected(
            a,
            simulation,
            a.hints,
            Some(a.workspace.as_ref().unwrap_or(&blank)),
        )
    } else {
        let State::PartitionFs { simulation } = &s.initial else {
            unreachable!()
        };
        AttemptRef {
            scenario_version: s.version,
            state: StateRef::PartitionFs { simulation },
            step: 0,
            challenge: false,
            hints: 0,
            transcript: empty,
            visited,
            ipv4: None,
            workspace: Some(&blank),
        }
    };
    p.visited = visited;
    if !bounded::within(&(s, &p), MAX_CONTEXT) {
        return Err(
            "The complete disk task has no room for this visit; prior profile and recency retained"
                .into(),
        );
    }
    Ok(())
}
pub fn set_challenge(a: &mut Attempt, s: &Scenario, value: bool) -> Result<(), String> {
    context(a, s)?;
    let State::PartitionFs { simulation } = &a.state else {
        unreachable!()
    };
    let mut p = projected(a, simulation, a.hints, a.workspace.as_ref());
    p.challenge = value;
    if !bounded::within(&(s, &p), MAX_CONTEXT) {
        return Err("No room for this mode change; previous mode retained".into());
    }
    a.challenge = value;
    Ok(())
}
