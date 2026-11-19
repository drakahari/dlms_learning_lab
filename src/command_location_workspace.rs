//! Source-bound assignment/practice filesystem contexts; no host execution.
use crate::{
    engine::{Goal, State},
    filesystem::{Tree, discovery_policy as policy},
    pack::Scenario,
    progress::Attempt,
    terminal_teaching,
    workspace::WorkspaceData,
};
pub struct Source<'a> {
    pub initial: &'a Tree,
    pub definition: policy::Reference<'a>,
    pub lesson: &'a terminal_teaching::Lesson,
}
fn opted(t: &Tree) -> bool {
    t.command_location.is_some()
}
fn reference(g: &Goal) -> bool {
    matches!(g,Goal::Filesystem{requirements}if policy::has_reference(requirements))
}
pub fn has_source(s: &Scenario) -> bool {
    matches!(&s.initial,State::Filesystem{tree}if opted(tree))
        || reference(&s.goal)
        || s.terminal_teaching.as_ref().is_some_and(|l| {
            opted(&l.fixture)
                || policy::has_reference(&l.outcomes)
        })
}
pub fn has_metadata(a: &Attempt) -> bool {
    matches!(&a.state,State::Filesystem{tree}if opted(tree))
        || a.workspace
            .as_ref()
            .is_some_and(|w| w.terminal_learning.as_ref().is_some_and(|l| opted(&l.tree)))
}
pub fn relevant(a: &Attempt, s: &Scenario) -> bool {
    has_source(s) || has_metadata(a)
}
/// Pure source pairing only; never runs authored example or solution commands.
pub fn source(s: &Scenario) -> Result<Option<Source<'_>>, String> {
    if !has_source(s) {
        return Ok(None);
    }
    let (State::Filesystem { tree: initial }, Goal::Filesystem { requirements }) =
        (&s.initial, &s.goal)
    else {
        return Err("Command-location work requires its supplied filesystem task".into());
    };
    let definition =
        policy::unique(requirements)?.ok_or("Command-location source reference is missing")?;
    if !definition.valid_for(initial) || requirements.iter().any(|r| !r.valid_for(initial)) {
        return Err("Command-location reference differs from its original source".into());
    }
    let lesson = s
        .terminal_teaching
        .as_ref()
        .ok_or("Command-location work needs separate teaching and practice")?;
    let example =
        policy::unique(&lesson.outcomes)?.ok_or("Separate practice source reference is missing")?;
    if !example.valid_for(&lesson.fixture)
        || lesson
            .outcomes
            .iter()
            .any(|r| !r.valid_for(&lesson.fixture))
    {
        return Err("Practice reference differs from its own supplied source".into());
    }
    if !definition.same_family(example) {
        return Err("Assignment and practice must use the same discovery family".into());
    }
    if let (policy::Reference::Deployment(a),policy::Reference::Deployment(e))=(definition,example)
        && a.context.directories.iter().any(|d|e.context.directories.iter().any(|p|p.path==d.path)) {
        return Err("Separate practice needs different category search folders".into());
    }
    let assigned = initial
        .command_location
        .as_ref()
        .ok_or("Assignment command locations are missing")?;
    let practice = lesson
        .fixture
        .command_location
        .as_ref()
        .ok_or("Practice command locations are missing")?;
    if assigned.source_id != format!("{}-assignment", s.id)
        || practice.source_id != format!("{}-example", s.id)
        || assigned.source_id == practice.source_id
        || assigned.path == practice.path
        || assigned.names.iter().any(|n| practice.names.contains(n))
    {
        return Err(
            "Worked practice must use different command names, paths and source identity".into(),
        );
    }
    Ok(Some(Source {
        initial,
        definition,
        lesson,
    }))
}
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
        terminal_learning: _,
        process_learning,
        accounts_learning,
        accounts_template_drafts,
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
    accounts_learning.is_some()
        || accounts_template_drafts.is_some()
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
    let State::Filesystem { tree } = &a.state else {
        return Err("Saved command-location work has a different activity kind".into());
    };
    if a.scenario_version != s.version || a.ipv4.is_some() || !w.valid() || foreign(w) {
        return Err(
            "Command-location workspace differs from its supplied context; original retained"
                .into(),
        );
    }
    source.definition.bound(tree)?;
    let l = w
        .terminal_learning
        .as_ref()
        .ok_or("Separate command-location practice is missing; original retained")?;
    if !l.valid_for_source(source.lesson) {
        return Err("Saved practice differs from its own source; original retained".into());
    }
    Ok(())
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let Some(source) = source(s)? else {
        return if has_metadata(a) {
            Err("Command-location metadata belongs to another activity".into())
        } else {
            Ok(())
        };
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Saved command-location workspace is missing; original retained")?;
    bound(a, s, &source, w)
}
pub fn prepare(a: &mut Attempt, s: &Scenario, new_attempt: bool) -> Result<(), String> {
    let Some(source) = source(s)? else {
        return if has_metadata(a) {
            Err("Command-location metadata belongs to another activity".into())
        } else {
            Ok(())
        };
    };
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.terminal_learning.is_some())
    {
        return context(a, s);
    }
    let State::Filesystem { tree } = &a.state else {
        return Err("Saved filesystem kind differs; work retained".into());
    };
    if !new_attempt || tree != source.initial {
        return Err("Saved practice is missing; no answers or drafts were recreated".into());
    }
    if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
        return Err("Invalid prior workspace; work retained".into());
    }
    let mut candidate = a
        .workspace
        .as_ref()
        .map_or_else(WorkspaceData::new, Clone::clone);
    candidate.terminal_learning = Some(terminal_teaching::Learning::new(source.lesson));
    bound(a, s, &source, &candidate)?;
    a.workspace = Some(candidate);
    Ok(())
}
pub fn ready(a: &Attempt, s: &Scenario) -> bool {
    context(a, s).is_ok()
}
