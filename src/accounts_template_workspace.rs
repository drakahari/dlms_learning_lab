//! Authored-source pairing for native account-template drafts. No host I/O.
use crate::{
    accounts, accounts_teaching, accounts_template,
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    workspace::WorkspaceData,
};
pub struct Source<'a> {
    pub initial: &'a accounts::State,
    pub lesson: &'a accounts_teaching::Lesson,
    pub teaching: &'a accounts_template::Teaching,
}
fn opted(state: &State) -> bool {
    matches!(state,State::Accounts{simulation} if !simulation.policy().editable_skeleton_leaves.is_empty())
}
pub fn has_source(s: &Scenario) -> bool {
    opted(&s.initial)
        || matches!(&s.goal,Goal::Accounts{initial,..} if !initial.policy().editable_skeleton_leaves.is_empty())
        || s.accounts_teaching.as_ref().is_some_and(|l| {
            l.template_teaching.is_some() || !l.fixture.policy().editable_skeleton_leaves.is_empty()
        })
}
pub fn has_metadata(a: &Attempt) -> bool {
    opted(&a.state)
        || a.workspace.as_ref().is_some_and(|w| {
            w.accounts_template_drafts.is_some()
                || w.accounts_learning.as_ref().is_some_and(|l| {
                    l.template_drafts.is_some()
                        || !l.simulation.policy().editable_skeleton_leaves.is_empty()
                })
        })
}
pub fn relevant(a: &Attempt, s: &Scenario) -> bool {
    has_metadata(a) || has_source(s)
}
/// Checks source declarations only; never applies native reference steps or runs commands.
pub fn source(s: &Scenario) -> Result<Option<Source<'_>>, String> {
    if !has_source(s) {
        return Ok(None);
    }
    let (
        State::Accounts {
            simulation: initial,
        },
        Goal::Accounts {
            initial: goal,
            requirements,
        },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Template work needs its authored account activity".into());
    };
    let lesson = s
        .accounts_teaching
        .as_ref()
        .ok_or("Template work needs its supplied teaching context")?;
    let teaching = lesson
        .template_teaching
        .as_ref()
        .ok_or("Template activity is missing its native teaching metadata")?;
    if initial != goal
        || !initial.valid_initial()
        || initial.policy().editable_skeleton_leaves.is_empty()
        || !accounts::valid_requirements(initial, requirements)
        || !lesson.separate_from(initial)
    {
        return Err(
            "Template initial state, outcomes or separate example do not match their source".into(),
        );
    }
    teaching.validate_assignment(initial)?;
    teaching.validate_example(&lesson.fixture)?;
    Ok(Some(Source {
        initial,
        lesson,
        teaching,
    }))
}
/// New work starts from source values only. Missing resumed drafts are refused,
/// never silently recreated or filled from a worked assignment reference.
pub fn prepare(a: &mut Attempt, s: &Scenario, new_attempt: bool) -> Result<(), String> {
    let Some(source) = source(s)? else {
        return if has_metadata(a) {
            Err("Template metadata belongs to a different activity".into())
        } else {
            Ok(())
        };
    };
    let State::Accounts { simulation } = &a.state else {
        return Err("Template work has a different saved activity kind".into());
    };
    if a.scenario_version != s.version || !simulation.valid_against(source.initial) {
        return Err("Saved template account context differs; previous work retained".into());
    }
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.accounts_template_drafts.is_some() || w.accounts_learning.is_some())
    {
        return context(a, s);
    }
    if !new_attempt || simulation != source.initial {
        return Err("Saved native template drafts are missing; previous work retained".into());
    }
    let drafts = accounts_template::Drafts::new(source.initial, simulation)?;
    let learning = accounts_teaching::Learning::new(source.lesson);
    if !learning.valid_for(source.lesson) {
        return Err("Separate template example cannot be initialized from its source".into());
    }
    if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
        return Err("Invalid existing workspace; previous work retained".into());
    }
    // Default has version 0; a new native workspace must use version 1.
    let mut candidate = a
        .workspace
        .as_ref()
        .map_or_else(WorkspaceData::new, Clone::clone);
    candidate.accounts_template_drafts = Some(drafts);
    candidate.accounts_learning = Some(learning);
    bound(a, s, &source, &candidate)?;
    a.workspace = Some(candidate);
    Ok(())
}
/// Closed native account workspace projection; unrelated drafts cannot grant context.
pub fn foreign(w: &WorkspaceData) -> bool {
    let WorkspaceData {
        physical_link_draft,
        physical_link_learning,
        one_off_session,
        one_off_learning,
        file_backup_learning,
        etherchannel_draft,
        etherchannel_learning,
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
        accounts_learning: _,
        accounts_template_drafts: _,
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
    false
        || physical_link_draft.is_some()
        || physical_link_learning.is_some()
        || one_off_session.is_some()
        || one_off_learning.is_some()
        || file_backup_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || cron_session.is_some()
        || cron_learning.is_some()
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

fn bound(a: &Attempt, s: &Scenario, source: &Source<'_>, w: &WorkspaceData) -> Result<(), String> {
    let State::Accounts { simulation } = &a.state else {
        return Err("Native template work has the wrong activity kind; original retained".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || !simulation.valid_against(source.initial)
    {
        return Err("Native template account/source context differs; original retained".into());
    }
    let drafts = w
        .accounts_template_drafts
        .as_ref()
        .ok_or("Assignment template drafts are missing; original retained")?;
    let learning = w
        .accounts_learning
        .as_ref()
        .ok_or("Separate template practice is missing; original retained")?;
    if !w.valid()
        || foreign(w)
        || !drafts.valid_for(source.initial, simulation)
        || !learning.valid_for(source.lesson)
    {
        return Err("Template drafts, practice or identities differ from their authored context; original retained".into());
    }
    Ok(())
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let Some(source) = source(s)? else {
        return if has_metadata(a) {
            Err("Native template metadata on a different activity; original retained".into())
        } else {
            Ok(())
        };
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Native template workspace is missing; original retained")?;
    bound(a, s, &source, w)
}

pub fn ready(a: &Attempt, s: &Scenario) -> bool {
    if context(a, s).is_err() {
        return false;
    }
    let Ok(Some(source)) = source(s) else {
        return !has_metadata(a);
    };
    let State::Accounts { simulation } = &a.state else {
        return false;
    };
    a.workspace
        .as_ref()
        .and_then(|w| w.accounts_template_drafts.as_ref())
        .is_some_and(|d| d.matches_state(source.initial, simulation))
}
/// Only authoring/reachability code calls this, before real solution commands.
/// Learner launch, Run, Save, reset and resume never call it.
pub fn reference_state(s: &Scenario) -> Result<Option<accounts::State>, String> {
    let Some(source) = source(s)? else {
        return Ok(None);
    };
    source
        .teaching
        .assignment_reference(source.initial)
        .map(Some)
}
