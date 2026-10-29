//! Complete-attempt typed physical workspace. No host I/O or learner shell.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    physical_link as core, physical_link_assignment as assignment,
    physical_link_teaching as teaching,
    progress::Attempt,
    script_teaching::View,
    workspace::WorkspaceData,
};
use serde::Serialize;
pub const MAX_ATTEMPT: usize = 2 * 1024 * 1024;
pub const MAX_SCENARIO: usize = 512 * 1024;
pub const PREFIX: &str = "@physical ";
pub const HELP: &str = "Use native interface/cable configuration and bounded simulated samples. No IOS/Linux command or host network is available. Internal author actions are not equipment commands.";
pub fn size<T: Serialize + ?Sized>(value: &T, cap: usize) -> Option<usize> {
    assignment::bytes(value, cap)
}
pub fn within<T: Serialize + ?Sized>(value: &T, cap: usize) -> bool {
    size(value, cap).is_some()
}
pub fn has_metadata(a: &Attempt) -> bool {
    a.workspace
        .as_ref()
        .is_some_and(|w| w.physical_link_draft.is_some() || w.physical_link_learning.is_some())
}
pub fn is_workspace(a: &Attempt) -> bool {
    matches!(a.state, State::PhysicalLink { .. }) || has_metadata(a)
}
pub fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        one_off_session,
        one_off_learning,
        etherchannel_draft,
        etherchannel_learning,
        physical_link_draft: _,
        physical_link_learning: _,
        file_backup_learning,
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
    one_off_session.is_some()
        || one_off_learning.is_some()
        || file_backup_learning.is_some()
        || !w.command.is_empty()
        || cron_session.is_some()
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

pub fn source_shape(s: &crate::pack::Scenario) -> bool {
    let crate::pack::Scenario {
        one_off_teaching,
        physical_link_teaching,
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
    one_off_teaching.is_none()
        && physical_link_teaching.is_some()
        && file_backup_teaching.is_none()
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
            (State::PhysicalLink { .. }, Goal::PhysicalLink { .. })
        )
}

/// Scalar admission/foreign rejection only; no implicit normalization or grade.
pub fn metadata(a: &Attempt) -> Result<(), String> {
    let State::PhysicalLink { simulation } = &a.state else {
        return if has_metadata(a) {
            Err("Physical-link work belongs to another activity; previous work retained".into())
        } else {
            Ok(())
        };
    };
    if a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(foreign)
        || !within(a, MAX_ATTEMPT)
        || simulation.validate().is_err()
        || a.workspace.as_ref().is_some_and(|w| !w.valid())
    {
        return Err("Saved physical-link work exceeds its bound or contains another activity's fields; previous work retained".into());
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.physical_link_draft.as_ref())
        .is_some_and(|d| !d.valid_shape(simulation.topology()))
    {
        return Err("Physical-link draft does not match its supplied equipment".into());
    }
    Ok(())
}
pub fn context<'a>(a: &Attempt, s: &'a Scenario) -> Result<&'a assignment::Definition, String> {
    if !source_shape(s) {
        return Err(
            "Physical-link source contains another activity's fields; previous work retained"
                .into(),
        );
    }
    metadata(a)?;
    if !within(s, MAX_SCENARIO)
        || a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
    {
        return Err(
            "Physical-link source/version/instruction bound differs; previous work retained".into(),
        );
    }
    let (
        State::PhysicalLink { simulation },
        State::PhysicalLink {
            simulation: initial,
        },
        Goal::PhysicalLink { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return Err("Physical-link State/Goal kind differs".into());
    };
    if initial != &definition.initial {
        return Err("Physical-link initial state differs from its authored definition".into());
    }
    definition.context(simulation)?;
    let lesson = s
        .physical_link_teaching
        .as_ref()
        .ok_or("Separate physical example is missing")?;
    lesson.validate(definition)?;
    if a.workspace
        .as_ref()
        .and_then(|w| w.physical_link_learning.as_ref())
        .is_some_and(|l| !l.valid_for(lesson))
    {
        return Err(
            "Saved physical practice differs from its full separate source; previous work retained"
                .into(),
        );
    }
    Ok(definition)
}
pub fn checks(a: &Attempt, s: &Scenario) -> assignment::Rows {
    let d = match context(a, s) {
        Ok(d) => d,
        Err(e) => return vec![(false, e)],
    };
    let State::PhysicalLink { simulation } = &a.state else {
        unreachable!("context proves kind")
    };
    if let Some(draft) = a
        .workspace
        .as_ref()
        .and_then(|w| w.physical_link_draft.as_ref())
    {
        assignment::current_checks(d, simulation, draft)
    } else {
        d.checks(simulation)
    }
}
/// Internal authoring route used for declarative reachability, never taught CLI.
pub fn execute(
    d: &assignment::Definition,
    current: &mut core::State,
    input: &str,
) -> Result<String, String> {
    if input.len() > PREFIX.len() + core::MAX_ACTION {
        return Err("Physical action exceeds raw bound before trimming".into());
    }
    d.context(current)?;
    let raw = input.strip_prefix(PREFIX).ok_or(
        "Use native configuration tools; only bounded internal author actions are accepted here",
    )?;
    let action = core::Action::from_json(raw.as_bytes()).map_err(|e| e.to_string())?;
    let report = current.perform(&action).map_err(|e| e.to_string())?;
    Ok(match report {
        Some(r) => format!("Simulated sample: {:?}", r.link.state),
        None => "Simulated physical configuration updated".into(),
    })
}
#[derive(Serialize)]
#[serde(tag = "kind")]
enum EngineRef<'a, T: Serialize + ?Sized> {
    PhysicalLink { simulation: &'a T },
}
fn field_bytes(name: &str, n: Option<usize>) -> Option<usize> {
    match n {
        None => Some(0),
        Some(n) => n.checked_add(name.len() + 4),
    }
}
fn option_size<T: Serialize>(v: Option<&T>, cap: usize) -> Option<Option<usize>> {
    match v {
        None => Some(None),
        Some(v) => Some(Some(size(v, cap)?)),
    }
}
fn retained(a: &Attempt) -> Result<(Option<usize>, Option<usize>), String> {
    let w = a.workspace.as_ref();
    Ok((
        option_size(
            w.and_then(|w| w.physical_link_draft.as_ref()),
            assignment::MAX_DRAFT,
        )
        .ok_or("Retained physical draft bound")?,
        option_size(
            w.and_then(|w| w.physical_link_learning.as_ref()),
            teaching::MAX_LEARNING,
        )
        .ok_or("Retained physical practice bound")?,
    ))
}
fn projection<T: Serialize + ?Sized>(
    a: &Attempt,
    current: &T,
    draft: Option<usize>,
    learning: Option<usize>,
    checked: Option<bool>,
    challenge: Option<bool>,
    visited: Option<u64>,
) -> Option<usize> {
    let mut n = size(a, MAX_ATTEMPT)?
        .checked_sub(size(&a.state, MAX_ATTEMPT)?)?
        .checked_add(size(
            &EngineRef::PhysicalLink {
                simulation: current,
            },
            MAX_ATTEMPT,
        )?)?;
    let blank = WorkspaceData::new();
    let w = a.workspace.as_ref().unwrap_or(&blank);
    let old_w = size(w, MAX_ATTEMPT)?;
    let (old_d, old_l) = retained(a).ok()?;
    let mut next_w = old_w
        .checked_sub(field_bytes("physical_link_draft", old_d)?)?
        .checked_sub(field_bytes("physical_link_learning", old_l)?)?
        .checked_add(field_bytes("physical_link_draft", draft)?)?
        .checked_add(field_bytes("physical_link_learning", learning)?)?;
    if let Some(value) = checked {
        next_w = next_w
            .checked_sub(size(&w.checked, 16)?)?
            .checked_add(size(&value, 16)?)?;
    }
    n = if a.workspace.is_some() {
        n.checked_sub(old_w)?.checked_add(next_w)?
    } else if draft.is_some() || learning.is_some() || checked.is_some() {
        n.checked_add("workspace".len() + 4)?.checked_add(next_w)?
    } else {
        n
    };
    if let Some(value) = challenge {
        n = n
            .checked_sub(size(&a.challenge, 16)?)?
            .checked_add(size(&value, 16)?)?;
    }
    if let Some(value) = visited {
        n = n
            .checked_sub(size(&a.visited, 32)?)?
            .checked_add(size(&value, 32)?)?;
    }
    Some(n)
}
fn admitted(n: Option<usize>) -> Result<(), String> {
    if n.is_some_and(|n| n <= MAX_ATTEMPT) {
        Ok(())
    } else {
        Err("Physical complete saved Attempt exceeds2MiB; previous work retained".into())
    }
}
fn current(a: &Attempt) -> Result<&core::State, String> {
    match &a.state {
        State::PhysicalLink { simulation } => Ok(simulation),
        _ => Err("Not a physical-link workspace".into()),
    }
}
pub fn prepare_attempt(a: &mut Attempt, s: &Scenario, new: bool) -> Result<(), String> {
    let d = context(a, s)?;
    let state = current(a)?;
    let lesson = s.physical_link_teaching.as_ref().unwrap();
    let (old_d, old_l) = retained(a)?;
    let view = if new {
        View::Introduction
    } else {
        View::Assignment
    };
    let preferred_link = &d.requirements[0].link;
    let draft_n = old_d
        .or_else(|| {
            size(
                &assignment::fresh_draft_for(state, preferred_link),
                assignment::MAX_DRAFT,
            )
        })
        .ok_or("Fresh physical draft bound")?;
    let learning_n = old_l
        .or_else(|| teaching::fresh_size(lesson, view))
        .ok_or("Fresh physical example bound")?;
    admitted(projection(
        a,
        state,
        Some(draft_n),
        Some(learning_n),
        None,
        None,
        None,
    ))?;
    let draft = if old_d.is_none() {
        let mut value = assignment::Draft::from_state(state)?;
        value.selected_link = preferred_link.clone();
        value.selected_port = state.topology().links[preferred_link].a.clone();
        Some(value)
    } else {
        None
    };
    let learning = if old_l.is_none() {
        let mut value = teaching::Learning::new(lesson, d)?;
        value.install_view(view);
        Some(value)
    } else {
        None
    };
    let w = a.workspace.get_or_insert_with(WorkspaceData::new);
    if let Some(v) = draft {
        w.physical_link_draft = Some(v);
    }
    if let Some(v) = learning {
        w.physical_link_learning = Some(v);
    }
    Ok(())
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Assignment,
    Practice,
}
fn lesson(s: &Scenario) -> Result<&teaching::Lesson, String> {
    s.physical_link_teaching
        .as_ref()
        .ok_or_else(|| "Separate example is missing".into())
}
fn learning(a: &Attempt) -> Result<&teaching::Learning, String> {
    a.workspace
        .as_ref()
        .and_then(|w| w.physical_link_learning.as_ref())
        .ok_or_else(|| "Open the separate example first".into())
}
pub fn set_draft(
    a: &mut Attempt,
    s: &Scenario,
    scope: Scope,
    draft: &assignment::Draft,
) -> Result<(), String> {
    context(a, s)?;
    let state = current(a)?;
    let (old_d, old_l) = retained(a)?;
    match scope {
        Scope::Assignment => {
            if !draft.valid_shape(state.topology()) {
                return Err("Draft belongs to different supplied interfaces/links".into());
            }
            let checked = a.workspace.as_ref().is_some_and(|w| {
                w.checked
                    && w.physical_link_draft
                        .as_ref()
                        .is_some_and(|old| old.configuration == draft.configuration)
            });
            let bytes = size(draft, assignment::MAX_DRAFT).ok_or("Physical draft bound")?;
            admitted(projection(
                a,
                state,
                Some(bytes),
                old_l,
                Some(checked),
                None,
                None,
            ))?;
            a.workspace
                .get_or_insert_with(WorkspaceData::new)
                .physical_link_draft = Some(draft.clone());
            a.workspace.as_mut().unwrap().checked = checked;
        }
        Scope::Practice => {
            let l = learning(a)?;
            if !draft.valid_shape(l.current.topology()) {
                return Err("Example draft belongs to different equipment".into());
            }
            let checked = l.checked && l.draft.configuration == draft.configuration;
            let projected = l.projection(&l.current, draft, l.view, checked);
            let bytes =
                size(&projected, teaching::MAX_LEARNING).ok_or("Physical example draft bound")?;
            admitted(projection(a, state, old_d, Some(bytes), None, None, None))?;
            a.workspace
                .as_mut()
                .unwrap()
                .physical_link_learning
                .as_mut()
                .unwrap()
                .set_draft(lesson(s)?, draft)?;
        }
    }
    Ok(())
}
pub fn apply(a: &mut Attempt, s: &Scenario, scope: Scope) -> Result<(), String> {
    context(a, s)?;
    let (old_d, old_l) = retained(a)?;
    match scope {
        Scope::Assignment => {
            let state = current(a)?;
            let draft = a
                .workspace
                .as_ref()
                .and_then(|w| w.physical_link_draft.as_ref())
                .ok_or("Prepare a configuration draft first")?;
            let action = core::Action::Configure {
                configuration: draft.configuration.clone(),
            };
            let next = state.project(&action).map_err(|e| e.to_string())?;
            admitted(projection(a, &next, old_d, old_l, Some(false), None, None))?;
            drop(next);
            let State::PhysicalLink { simulation } = &mut a.state else {
                unreachable!()
            };
            simulation.perform(&action).map_err(|e| e.to_string())?;
            a.workspace.as_mut().unwrap().checked = false;
        }
        Scope::Practice => {
            let state = current(a)?;
            let l = learning(a)?;
            let action = core::Action::Configure {
                configuration: l.draft.configuration.clone(),
            };
            let next = l.current.project(&action).map_err(|e| e.to_string())?;
            let p = l.projection(&next, &l.draft, l.view, false);
            admitted(projection(
                a,
                state,
                old_d,
                Some(size(&p, teaching::MAX_LEARNING).ok_or("Projected example apply bound")?),
                None,
                None,
                None,
            ))?;
            drop(next);
            a.workspace
                .as_mut()
                .unwrap()
                .physical_link_learning
                .as_mut()
                .unwrap()
                .apply(lesson(s)?)?;
        }
    }
    Ok(())
}
pub fn perform(
    a: &mut Attempt,
    s: &Scenario,
    scope: Scope,
    action: &core::Action,
) -> Result<Option<core::ProbeReport>, String> {
    if !matches!(
        action,
        core::Action::Probe { .. } | core::Action::ClearCounters { .. }
    ) {
        return Err("Configuration changes belong in the native draft; Apply them together".into());
    }
    context(a, s)?;
    let (old_d, old_l) = retained(a)?;
    let state = current(a)?;
    match scope {
        Scope::Assignment => {
            let next = state.project(action).map_err(|e| e.to_string())?;
            admitted(projection(a, &next, old_d, old_l, Some(false), None, None))?;
            drop(next);
            let State::PhysicalLink { simulation } = &mut a.state else {
                unreachable!()
            };
            let report = simulation.perform(action).map_err(|e| e.to_string())?;
            a.workspace.get_or_insert_with(WorkspaceData::new).checked = false;
            Ok(report)
        }
        Scope::Practice => {
            let l = learning(a)?;
            let next = l.current.project(action).map_err(|e| e.to_string())?;
            let p = l.projection(&next, &l.draft, l.view, false);
            admitted(projection(
                a,
                state,
                old_d,
                Some(size(&p, teaching::MAX_LEARNING).ok_or("Projected example sample bound")?),
                None,
                None,
                None,
            ))?;
            drop(next);
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .physical_link_learning
                .as_mut()
                .unwrap();
            let result = l.current.perform(action).map_err(|e| e.to_string())?;
            l.install_checked(false);
            Ok(result)
        }
    }
}
pub fn fill_example(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    context(a, s)?;
    let l = learning(a)?;
    let mut draft = assignment::Draft::from_state(&l.current)?;
    draft.configuration = lesson(s)?.reference.clone();
    set_draft(a, s, Scope::Practice, &draft)
}
pub fn set_view(a: &mut Attempt, s: &Scenario, view: View) -> Result<(), String> {
    context(a, s)?;
    let state = current(a)?;
    let l = learning(a)?;
    let (d, _) = retained(a)?;
    let p = l.projection(&l.current, &l.draft, view, l.checked);
    admitted(projection(
        a,
        state,
        d,
        Some(size(&p, teaching::MAX_LEARNING).ok_or("Example view bound")?),
        None,
        None,
        None,
    ))?;
    a.workspace
        .as_mut()
        .unwrap()
        .physical_link_learning
        .as_mut()
        .unwrap()
        .install_view(view);
    Ok(())
}
pub fn record_mode(a: &mut Attempt, s: &Scenario, challenge: bool) -> Result<(), String> {
    context(a, s)?;
    let (d, l) = retained(a)?;
    admitted(projection(
        a,
        current(a)?,
        d,
        l,
        None,
        Some(challenge),
        None,
    ))?;
    a.challenge = challenge;
    Ok(())
}
pub fn record_visit(a: &mut Attempt, s: &Scenario, visited: u64) -> Result<(), String> {
    context(a, s)?;
    if visited < a.visited {
        return Err("Saved visit order differs; previous work retained".into());
    }
    let (d, l) = retained(a)?;
    admitted(projection(a, current(a)?, d, l, None, None, Some(visited)))?;
    a.visited = visited;
    Ok(())
}
pub fn set_checked(
    a: &mut Attempt,
    s: &Scenario,
    scope: Scope,
    checked: bool,
) -> Result<(), String> {
    context(a, s)?;
    let state = current(a)?;
    let (d, l) = retained(a)?;
    match scope {
        Scope::Assignment => {
            admitted(projection(a, state, d, l, Some(checked), None, None))?;
            a.workspace.get_or_insert_with(WorkspaceData::new).checked = checked;
        }
        Scope::Practice => {
            let l = learning(a)?;
            let p = l.projection(&l.current, &l.draft, l.view, checked);
            admitted(projection(
                a,
                state,
                d,
                Some(size(&p, teaching::MAX_LEARNING).ok_or("Example check flag bound")?),
                None,
                None,
                None,
            ))?;
            a.workspace
                .as_mut()
                .unwrap()
                .physical_link_learning
                .as_mut()
                .unwrap()
                .install_checked(checked);
        }
    }
    Ok(())
}

pub fn reset(a: &mut Attempt, s: &Scenario, scope: Scope) -> Result<(), String> {
    let definition = context(a, s)?;
    let (d, l) = retained(a)?;
    match scope {
        Scope::Assignment => {
            let preferred_link = &definition.requirements[0].link;
            let fresh = assignment::fresh_draft_for(&definition.initial, preferred_link);
            admitted(projection(
                a,
                &definition.initial,
                Some(size(&fresh, assignment::MAX_DRAFT).ok_or("Reset draft bound")?),
                l,
                Some(false),
                None,
                None,
            ))?;
            let mut draft = assignment::Draft::from_state(&definition.initial)?;
            draft.selected_link = preferred_link.clone();
            draft.selected_port = definition.initial.topology().links[preferred_link]
                .a
                .clone();
            let state = definition.initial.clone();
            a.state = State::PhysicalLink { simulation: state };
            let w = a.workspace.get_or_insert_with(WorkspaceData::new);
            w.physical_link_draft = Some(draft);
            w.checked = false;
        }
        Scope::Practice => {
            let state = current(a)?;
            let view = learning(a)?.view;
            let lesson = lesson(s)?;
            let n = teaching::fresh_size(lesson, view).ok_or("Reset practice bound")?;
            admitted(projection(a, state, d, Some(n), None, None, None))?;
            let mut fresh = teaching::Learning::new(lesson, definition)?;
            fresh.install_view(view);
            a.workspace.as_mut().unwrap().physical_link_learning = Some(fresh);
        }
    }
    Ok(())
}
fn scalar_replace<T: Serialize>(a: &Attempt, old: &T, new: &T) -> Result<(), String> {
    admitted(
        size(a, MAX_ATTEMPT)
            .and_then(|n| n.checked_sub(size(old, 32)?))
            .and_then(|n| n.checked_add(size(new, 32)?)),
    )
}
pub fn record_step(a: &mut Attempt, s: &Scenario, step: usize) -> Result<(), String> {
    context(a, s)?;
    if step >= s.steps.len() {
        return Err("Choose an available supporting section; earlier work retained".into());
    }
    scalar_replace(a, &a.step, &step)?;
    a.step = step;
    Ok(())
}
pub fn record_hints(a: &mut Attempt, s: &Scenario, hints: usize) -> Result<(), String> {
    context(a, s)?;
    if hints > s.hints.len() {
        return Err("Choose an available hint; earlier work retained".into());
    }
    scalar_replace(a, &a.hints, &hints)?;
    a.hints = hints;
    Ok(())
}

pub fn reject_foreign_source(s: &Scenario) -> Result<(), String> {
    if s.physical_link_teaching.is_some()
        && !matches!(
            (&s.initial, &s.goal),
            (State::PhysicalLink { .. }, Goal::PhysicalLink { .. })
        )
    {
        Err("Physical-link teaching belongs to its physical-link activity".into())
    } else {
        Ok(())
    }
}

pub fn practice_checks(a: &Attempt, s: &Scenario) -> assignment::Rows {
    if let Err(e) = context(a, s) {
        return vec![(false, e)];
    }
    match (learning(a), lesson(s)) {
        (Ok(l), Ok(source)) => l.checks(source),
        (Err(e), _) | (_, Err(e)) => vec![(false, e)],
    }
}
