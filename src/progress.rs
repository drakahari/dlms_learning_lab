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
/// Fresh assignment results shared by DHCP feedback and current completion.
/// No result survives a state/draft change or becomes a persisted credential.
/// Reject context and pending draft first; do not run new-client probes for them.
pub fn current_dhcpv4_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    if s.cron_teaching.is_some() {
        return vec![(
            false,
            "Scheduling teaching belongs to its scheduling activity".into(),
        )];
    }

    if let Err(e) = validate_nfs_workspace(a) {
        return vec![(false, crate::nfs_workspace::notice(e))];
    }
    if let Err(e) = validate_rpm_workspace(a) {
        return vec![(false, crate::rpm_packages_workspace::notice(&e))];
    }
    if let Err(e) = validate_nat_workspace(a) {
        return vec![(false, crate::nat_workspace::notice(&e))];
    }
    let fail = |text: &str| vec![(false, text.to_owned())];
    if let Err(error) = validate_dhcpv4_workspace(a) {
        return fail(&error);
    }
    let (
        State::DhcpV4 { simulation },
        State::DhcpV4 {
            simulation: authored,
        },
        crate::engine::Goal::DhcpV4 {
            initial,
            requirements,
            probes,
        },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return fail("DHCP assignment context differs; previous work retained");
    };
    if authored != initial || simulation.validate_context(initial.topology()).is_err() {
        return fail("DHCP assignment context differs; previous work retained");
    }
    if a.workspace.as_ref().is_some_and(|w| {
        w.dhcpv4_learning.as_ref().is_some_and(|l| {
            s.dhcpv4_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        })
    }) {
        return fail("DHCP practice context differs; previous work retained");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.dhcpv4_draft.as_ref())
        .is_some_and(|d| !d.matches(simulation))
    {
        return fail("Apply the pending policy draft before checking the assignment");
    }
    let mut rows = vec![(true, "Current policy draft is applied".to_owned())];
    rows.extend(crate::dhcpv4_workspace::checks(
        initial,
        simulation,
        requirements,
        probes,
    ));
    rows
}
/// Current applied draft/context first; snapshot predicates are derived once per batch.
pub fn current_spanning_tree_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    if s.cron_teaching.is_some() {
        return vec![(
            false,
            "Scheduling teaching belongs to its scheduling activity".into(),
        )];
    }

    if let Err(e) = validate_nfs_workspace(a) {
        return vec![(false, crate::nfs_workspace::notice(e))];
    }
    if let Err(e) = validate_rpm_workspace(a) {
        return vec![(false, crate::rpm_packages_workspace::notice(&e))];
    }
    if let Err(e) = validate_nat_workspace(a) {
        return vec![(false, crate::nat_workspace::notice(&e))];
    }
    let fail = |m: &str| vec![(false, crate::spanning_tree_workspace::notice(m))];
    if let Err(e) = validate_spanning_tree_workspace(a) {
        return fail(&e);
    }
    let (
        State::SpanningTree { simulation },
        State::SpanningTree {
            simulation: authored,
        },
        crate::engine::Goal::SpanningTree {
            initial,
            requirements,
        },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return fail("Spanning-tree assignment kind/context differs");
    };
    if authored != initial || !crate::spanning_tree_workspace::same_context(initial, simulation) {
        return fail("Spanning-tree fixed topology differs from the authored assignment");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.spanning_tree_learning.as_ref())
        .is_some_and(|l| {
            s.spanning_tree_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        })
    {
        return fail("Spanning-tree practice differs from its authored fixture");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.spanning_tree_draft.as_ref())
        .is_some_and(|d| !d.matches(simulation))
    {
        return fail("Apply the pending spanning-tree configuration before checking work");
    }
    let mut rows = vec![(
        true,
        "Current spanning-tree configuration is applied".into(),
    )];
    rows.extend(crate::spanning_tree_workspace::checks(
        initial,
        simulation,
        requirements,
    ));
    rows
}
/// Backup metadata only: no grading recursion and no inferred result.
pub(crate) fn validate_file_backup_workspace(a: &Attempt) -> Result<(), String> {
    let State::FileBackup { simulation } = &a.state else {
        return if crate::file_backup_workspace::has_metadata(a) {
            Err("Backup practice belongs to another activity; previous work retained".into())
        } else {
            Ok(())
        };
    };
    if a.ipv4.is_some()
        || !simulation.valid()
        || a.workspace
            .as_ref()
            .is_some_and(|w| crate::file_backup_workspace::foreign(w) || !w.valid())
        || !crate::file_backup_workspace::within(a, crate::file_backup_workspace::MAX_ATTEMPT)
    {
        return Err("Saved backup work is too large or contains another activity's fields; previous work retained".into());
    }
    Ok(())
}
pub fn current_physical_link_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    crate::physical_link_workspace::checks(a, s)
}
pub fn current_file_backup_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    match crate::file_backup_workspace::context(a, s) {
        Ok(d) => {
            let State::FileBackup { simulation } = &a.state else {
                unreachable!()
            };
            d.checks(simulation)
        }
        Err(e) => vec![(false, e)],
    }
}
/// Current LACP outcomes use applied typed configuration and bounded independent fault snapshots.
pub fn current_etherchannel_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    let d = match crate::etherchannel_workspace::context(a, s) {
        Ok(d) => d,
        Err(e) => return vec![(false, crate::etherchannel_workspace::notice(e))],
    };
    let State::EtherChannel { simulation } = &a.state else {
        return vec![(false, "LACP assignment kind differs".into())];
    };
    let mut rows = d.checks(simulation);
    if a.workspace
        .as_ref()
        .and_then(|w| w.etherchannel_draft.as_ref())
        .is_some_and(|draft| !draft.matches(simulation))
    {
        rows[0] = (
            false,
            "Configuration changes are pending; Apply the current draft before checking".into(),
        );
    }
    rows
}
/// Scheduling metadata admission only; no grading, parser execution or inferred credit.
pub(crate) fn validate_cron_metadata(a: &Attempt) -> Result<(), String> {
    crate::one_off_workspace::metadata(a)?;
    let present = crate::workspace::has_cron_metadata(a);
    let State::Cron { simulation } = &a.state else {
        return if present {
            Err("Scheduling drafts/practice on another activity; previous work retained".into())
        } else {
            Ok(())
        };
    };
    simulation.validate().map_err(|e| e.to_string())?;
    if a.ipv4.is_some()
        || a.workspace
            .as_ref()
            .is_some_and(|w| crate::cron_workspace::foreign(w) || !w.valid())
        || !crate::cron_workspace::within(a, crate::cron_workspace::MAX_ATTEMPT)
    {
        return Err(
            "Scheduling metadata or whole Attempt bound differs; previous work retained".into(),
        );
    }
    Ok(())
}
pub fn current_cron_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    let d = match crate::cron_workspace::context(a, s) {
        Ok(d) => d,
        Err(e) => return vec![(false, e)],
    };
    let State::Cron { simulation } = &a.state else {
        return vec![(false, "Scheduling assignment kind differs".into())];
    };
    if let Some(t) = a.workspace.as_ref().and_then(|w| w.cron_session.as_ref()) {
        crate::cron_workspace::checks_with_session(d, simulation, t)
    } else {
        d.checks(simulation)
    }
}
/// NFS metadata admission only: no grader recursion and no inferred completion.
pub(crate) fn validate_nfs_metadata(a: &Attempt) -> Result<(), String> {
    validate_file_backup_workspace(a)?;
    crate::etherchannel_workspace::metadata(a)?;
    let present = crate::workspace::has_nfs_metadata(a);
    let State::Nfs { simulation } = &a.state else {
        return if present {
            Err("NFS editor/practice metadata on another activity; previous work retained".into())
        } else {
            Ok(())
        };
    };
    if a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            crate::nfs_workspace::foreign(w)
                || !w.valid()
                || w.nfs_session
                    .as_ref()
                    .is_some_and(|t| !t.valid_for(simulation))
                || w.nfs_learning
                    .as_ref()
                    .is_some_and(|l| !crate::nfs_teaching::separate(&l.context.initial, simulation))
        })
        || !crate::nfs_workspace::within(a, crate::nfs_workspace::MAX_ATTEMPT)
    {
        return Err("NFS bounds or foreign metadata differ; previous work retained".into());
    }
    Ok(())
}
pub(crate) fn validate_nfs_workspace(a: &Attempt) -> Result<(), String> {
    validate_cron_metadata(a)?;
    validate_nfs_metadata(a)?;
    if let State::Nfs { simulation } = &a.state {
        simulation
            .validate()
            .map_err(|e| crate::nfs_workspace::notice(e.message))?;
    }
    Ok(())
}
/// Snapshot grading uses actual current files/mounts plus independent next-mount trials.
pub fn current_nfs_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    let d = match crate::nfs_workspace::context(a, s) {
        Ok(d) => d,
        Err(e) => return vec![(false, crate::nfs_workspace::notice(e))],
    };
    let State::Nfs { simulation } = &a.state else {
        return vec![(false, "NFS assignment kind differs".into())];
    };
    d.checks(simulation)
}
/// Fresh RPM current outcomes. A transcript, raw command or checked flag never grants credit.
pub fn current_rpm_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    if s.cron_teaching.is_some() {
        return vec![(
            false,
            "Scheduling teaching belongs to its scheduling activity".into(),
        )];
    }

    if let Err(e) = validate_nfs_workspace(a) {
        return vec![(false, crate::nfs_workspace::notice(e))];
    }
    let fail = |e: &str| vec![(false, crate::rpm_packages_workspace::notice(e))];
    if let Err(e) = validate_rpm_metadata(a) {
        return fail(&e);
    }
    let (
        State::RpmPackages { simulation },
        State::RpmPackages {
            simulation: authored,
        },
        crate::engine::Goal::RpmPackages { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return fail("RPM assignment kind differs");
    };
    if authored != &definition.initial || !definition.same_assignment(simulation) {
        return fail("RPM immutable catalog differs from authored assignment");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.rpm_learning.as_ref())
        .is_some_and(|l| {
            s.rpm_packages_teaching.as_ref().is_none_or(|source| {
                !source.separate_from_definition(definition) || !l.valid_for(source)
            })
        })
    {
        return fail("RPM practice differs from its authored lesson");
    }
    // Definition.checks owns the one runtime State validation/view for this batch.
    definition.checks(simulation)
}
fn validate_rpm_metadata(a: &Attempt) -> Result<(), String> {
    let w = a.workspace.as_ref();
    let metadata = w.is_some_and(|w| w.rpm_session.is_some() || w.rpm_learning.is_some());
    let State::RpmPackages { simulation } = &a.state else {
        return if metadata {
            Err("RPM console/practice on another activity".into())
        } else {
            Ok(())
        };
    };
    if w.is_some_and(|w| {
        crate::rpm_packages_workspace::foreign(w)
            || !w.valid()
            || w.rpm_session
                .as_ref()
                .is_some_and(|t| !t.valid_for(simulation))
            || w.rpm_learning.as_ref().is_some_and(|l| {
                !crate::rpm_packages_teaching::separate(&l.context.initial, simulation)
            })
    }) || !crate::rpm_packages_workspace::within(a, crate::rpm_packages_workspace::MAX_ATTEMPT)
    {
        return Err("RPM bounds or foreign metadata differ; previous work retained".into());
    }
    Ok(())
}
fn validate_rpm_workspace(a: &Attempt) -> Result<(), String> {
    validate_rpm_metadata(a)?;
    if let State::RpmPackages { simulation } = &a.state
        && !crate::rpm_packages_workspace::admitted(simulation)
    {
        return Err("RPM State exceeds app admission".into());
    }
    Ok(())
}
/// Fresh package outcomes with context/session guards; display history is never credit.
pub fn current_debian_packages_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    if s.cron_teaching.is_some() {
        return vec![(
            false,
            "Scheduling teaching belongs to its scheduling activity".into(),
        )];
    }

    if let Err(e) = validate_nfs_workspace(a) {
        return vec![(false, crate::nfs_workspace::notice(e))];
    }
    if let Err(e) = validate_rpm_workspace(a) {
        return vec![(false, crate::rpm_packages_workspace::notice(&e))];
    }
    if let Err(e) = validate_nat_workspace(a) {
        return vec![(false, crate::nat_workspace::notice(&e))];
    }
    let fail = |m: &str| vec![(false, crate::debian_packages_workspace::notice(m))];
    if let Err(e) = validate_apache_static_workspace(a) {
        return fail(&e);
    }
    if let Err(e) = validate_debian_packages_workspace(a) {
        return fail(&e);
    }
    let (
        State::DebianPackages { simulation },
        State::DebianPackages {
            simulation: authored,
        },
        crate::engine::Goal::DebianPackages { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return fail("Package assignment kind/context differs");
    };
    if authored != &definition.initial {
        return fail("Package initial context differs from the task");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.debian_packages_learning.as_ref())
        .is_some_and(|l| {
            s.debian_packages_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        })
    {
        return fail("Package practice differs from its authored fixture");
    }
    definition.checks(simulation)
}
pub fn current_apache_static_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    if s.cron_teaching.is_some() {
        return vec![(
            false,
            "Scheduling teaching belongs to its scheduling activity".into(),
        )];
    }

    if let Err(e) = validate_nfs_workspace(a) {
        return vec![(false, crate::nfs_workspace::notice(e))];
    }
    if let Err(e) = validate_rpm_workspace(a) {
        return vec![(false, crate::rpm_packages_workspace::notice(&e))];
    }
    if let Err(e) = validate_nat_workspace(a) {
        return vec![(false, crate::nat_workspace::notice(&e))];
    }
    let fail = |m: &str| vec![(false, crate::apache_static_workspace::notice(m))];
    if let Err(e) = validate_debian_packages_workspace(a) {
        return fail(&e);
    }
    if let Err(e) = validate_apache_static_workspace(a) {
        return fail(&e);
    }
    let (
        State::ApacheStatic { simulation },
        State::ApacheStatic {
            simulation: authored,
        },
        crate::engine::Goal::ApacheStatic { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return fail("Apache assignment kind/context differs");
    };
    if authored != &definition.initial || !definition.same_assignment(simulation) {
        return fail("Apache fixed initial inventory differs from this assignment");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.apache_static_learning.as_ref())
        .is_some_and(|l| {
            s.apache_static_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        })
    {
        return fail("Apache example differs from its authored fixture");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.apache_static_draft.as_ref())
        .is_some_and(|d| !d.matches(simulation))
    {
        return fail(
            "Save pending configuration/file text before checking; request fields are read-only",
        );
    }
    let mut rows = vec![(true, "Current configuration/file drafts are saved".into())];
    rows.extend(definition.checks(simulation));
    rows
}
pub(crate) fn validate_nat_metadata(a: &Attempt) -> Result<(), String> {
    crate::physical_link_workspace::metadata(a)?;
    validate_nfs_workspace(a)?;
    validate_nat_workspace(a)?;
    // Existing family validators are admission only; none calls a grader.
    for validate in [
        validate_rpm_workspace,
        validate_apache_static_workspace,
        validate_debian_packages_workspace,
        validate_spanning_tree_workspace,
        validate_dhcpv4_workspace,
        validate_unit_manager_workspace,
        validate_ospf_workspace,
        validate_acl_workspace,
        validate_accounts_workspace,
        validate_storage_workspace,
        validate_discovery_workspace,
        validate_process_workspace,
        validate_dns_workspace,
        validate_legacy_teaching_workspace,
        validate_seed_workspace,
        validate_client_workspace,
        validate_service_workspace,
        validate_address_workspace,
        validate_routing_workspace,
        validate_mac_workspace,
    ] {
        validate(a)?;
    }
    Ok(())
}
pub fn current_nat_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    if let Err(e) = crate::one_off_workspace::metadata(a)
        .and_then(|_| crate::one_off_workspace::reject_foreign_source(s))
    {
        return vec![(false, e)];
    }
    if let Err(e) = crate::etherchannel_workspace::reject_foreign_source(s) {
        return vec![(false, crate::etherchannel_workspace::notice(e))];
    }
    if s.cron_teaching.is_some() {
        return vec![(
            false,
            "Scheduling teaching belongs to its scheduling activity".into(),
        )];
    }

    let fail = |m: &str| vec![(false, crate::nat_workspace::notice(m))];
    if let Err(e) = validate_nat_metadata(a) {
        return fail(&e);
    }
    let (
        State::BasicNat { simulation },
        State::BasicNat {
            simulation: authored,
        },
        crate::engine::Goal::BasicNat { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return fail("NAT assignment kind/context differs");
    };
    if authored != &definition.initial || !definition.same_assignment(simulation) {
        return fail("NAT immutable origin differs from authored assignment");
    }
    if s.nat_teaching
        .as_ref()
        .is_some_and(|l| l.validate_for(authored).is_err())
        || a.workspace
            .as_ref()
            .and_then(|w| w.nat_learning.as_ref())
            .is_some_and(|l| {
                s.nat_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            })
    {
        return fail("NAT practice metadata differs from the authored separate example");
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.nat_draft.as_ref())
        .is_some_and(|d| !d.matches(simulation))
    {
        return fail("Apply or repair pending NAT policy; request fields are read-only");
    }
    let mut rows = vec![(
        true,
        "Current NAT policy edits match the applied configuration".into(),
    )];
    rows.extend(definition.checks(simulation));
    rows
}
pub fn current_solved(a: &Attempt, s: &Scenario) -> bool {
    if crate::wireless_channel_workspace::relevant(a,s){return crate::wireless_channel_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::wlan_workspace::relevant(a,s){return crate::wlan_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::proxy_workspace::relevant(a,s){return crate::proxy_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::snmp_workspace::relevant(a,s){return crate::snmp_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::wlan_paths_workspace::relevant(a,s){return crate::wlan_paths_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::file_index_source::relevant(a,s){return crate::file_index_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal)}
    if crate::command_location_workspace::relevant(a,s)
        && !crate::command_location_workspace::ready(a,s){return false;}
    if crate::accounts_template_workspace::relevant(a,s)
        && !crate::accounts_template_workspace::ready(a,s){return false;}

    if matches!(a.state,State::SharedLibraries{..})||matches!(s.initial,State::SharedLibraries{..})||matches!(s.goal,crate::engine::Goal::SharedLibraries{..}){
        return crate::shared_libraries_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);
    }

    if matches!(a.state,State::Raid1{..})||matches!(s.initial,State::Raid1{..})||matches!(s.goal,crate::engine::Goal::Raid1{..}){
        return crate::raid1_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);
    }

    if matches!(a.state, State::StartupFiles { .. })
        || matches!(s.initial, State::StartupFiles { .. })
        || matches!(s.goal, crate::engine::Goal::StartupFiles { .. })
    {
        return crate::startup_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::FsIntegrity { .. })
        || matches!(s.initial, State::FsIntegrity { .. })
        || matches!(s.goal, crate::engine::Goal::FsIntegrity { .. })
    {
        return crate::fs_integrity_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::UserMounts { .. })
        || matches!(s.initial, State::UserMounts { .. })
        || matches!(s.goal, crate::engine::Goal::UserMounts { .. })
    {
        return crate::user_mounts_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Environment { .. })
        || matches!(s.initial, State::Environment { .. })
        || matches!(s.goal, crate::engine::Goal::Environment { .. })
    {
        return crate::environment_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::DebianSources { .. })
        || matches!(s.initial, State::DebianSources { .. })
        || matches!(s.goal, crate::engine::Goal::DebianSources { .. })
    {
        return crate::debian_sources_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::ClientNames { .. })
        || matches!(s.initial, State::ClientNames { .. })
        || matches!(s.goal, crate::engine::Goal::ClientNames { .. })
    {
        return crate::client_names_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::RemovableMounts { .. })
        || matches!(s.initial, State::RemovableMounts { .. })
        || matches!(s.goal, crate::engine::Goal::RemovableMounts { .. })
    {
        return crate::removable_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::NmProfiles { .. })
        || matches!(s.initial, State::NmProfiles { .. })
        || matches!(s.goal, crate::engine::Goal::NmProfiles { .. })
    {
        return crate::nm_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::PamStack { .. })
        || matches!(s.initial, State::PamStack { .. })
        || matches!(s.goal, crate::engine::Goal::PamStack { .. })
    {
        return crate::pam_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Aging { .. })
        || matches!(s.initial, State::Aging { .. })
        || matches!(s.goal, crate::engine::Goal::Aging { .. })
    {
        return crate::aging_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::PortSecurity { .. })
        || matches!(s.initial, State::PortSecurity { .. })
        || matches!(s.goal, crate::engine::Goal::PortSecurity { .. })
    {
        return crate::port_security_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::DhcpSnooping { .. })
        || matches!(s.initial, State::DhcpSnooping { .. })
        || matches!(s.goal, crate::engine::Goal::DhcpSnooping { .. })
    {
        return crate::dhcp_snooping_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::ArpInspection { .. })
        || matches!(s.initial, State::ArpInspection { .. })
        || matches!(s.goal, crate::engine::Goal::ArpInspection { .. })
    {
        return crate::arp_inspection_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::ManagementPolicy { .. })
        || matches!(s.initial, State::ManagementPolicy { .. })
        || matches!(s.goal, crate::engine::Goal::ManagementPolicy { .. })
    {
        return crate::management_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LocalAccess { .. })
        || matches!(s.initial, State::LocalAccess { .. })
        || matches!(s.goal, crate::engine::Goal::LocalAccess { .. })
    {
        return crate::local_access_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::AaaAccess { .. })
        || matches!(s.initial, State::AaaAccess { .. })
        || matches!(s.goal, crate::engine::Goal::AaaAccess { .. })
    {
        return crate::aaa_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LdapClient { .. })
        || matches!(s.initial, State::LdapClient { .. })
        || matches!(s.goal, crate::engine::Goal::LdapClient { .. })
    {
        return crate::ldap_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LdapAccess { .. })
        || matches!(s.initial, State::LdapAccess { .. })
        || matches!(s.goal, crate::engine::Goal::LdapAccess { .. })
    {
        return crate::ldap_access_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LdapSchema { .. })
        || matches!(s.initial, State::LdapSchema { .. })
        || matches!(s.goal, crate::engine::Goal::LdapSchema { .. })
    {
        return crate::ldap_schema_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state,State::CopiedJournal{..}) || matches!(s.initial,State::CopiedJournal{..}) || matches!(s.goal,crate::engine::Goal::CopiedJournal{..}) {return crate::journal_archive_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::JsonInventory{..}) || matches!(s.initial,State::JsonInventory{..}) || matches!(s.goal,crate::engine::Goal::JsonInventory{..}) {return crate::json_inventory_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::RestController{..}) || matches!(s.initial,State::RestController{..}) || matches!(s.goal,crate::engine::Goal::RestController{..}) {return crate::rest_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::DevicePath{..}) || matches!(s.initial,State::DevicePath{..}) || matches!(s.goal,crate::engine::Goal::DevicePath{..}) {return crate::device_path_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LocalMail{..}) || matches!(s.initial,State::LocalMail{..}) || matches!(s.goal,crate::engine::Goal::LocalMail{..}) {return crate::local_mail_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Chrony{..}) || matches!(s.initial,State::Chrony{..}) || matches!(s.goal,crate::engine::Goal::Chrony{..}) {return crate::chrony_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::GrubOnce{..}) || matches!(s.initial,State::GrubOnce{..}) || matches!(s.goal,crate::engine::Goal::GrubOnce{..}) {return crate::grub_once_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::EfiLoader{..}) || matches!(s.initial,State::EfiLoader{..}) || matches!(s.goal,crate::engine::Goal::EfiLoader{..}) {return crate::efi_loader_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::BiosLoader{..}) || matches!(s.initial,State::BiosLoader{..}) || matches!(s.goal,crate::engine::Goal::BiosLoader{..}) {return crate::bios_loader_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::BootEntry{..}) || matches!(s.initial,State::BootEntry{..}) || matches!(s.goal,crate::engine::Goal::BootEntry{..}) {return crate::boot_entry_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state, State::LdapSchemaQuery { .. })
        || matches!(s.initial, State::LdapSchemaQuery { .. })
        || matches!(s.goal, crate::engine::Goal::LdapSchemaQuery { .. })
    {
        return crate::ldap_schema_query_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LdapPassword { .. })
        || matches!(s.initial, State::LdapPassword { .. })
        || matches!(s.goal, crate::engine::Goal::LdapPassword { .. })
    {
        return crate::ldap_password_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LdapChanges { .. })
        || matches!(s.initial, State::LdapChanges { .. })
        || matches!(s.goal, crate::engine::Goal::LdapChanges { .. })
    {
        return crate::ldap_change_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::DhcpConfig { .. })
        || matches!(s.initial, State::DhcpConfig { .. })
        || matches!(s.goal, crate::engine::Goal::DhcpConfig { .. })
    {
        return crate::dhcp_server_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Qos { .. })
        || matches!(s.initial, State::Qos { .. })
        || matches!(s.goal, crate::engine::Goal::Qos { .. })
    {
        return crate::qos_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::LinuxLogging { .. })
        || matches!(s.initial, State::LinuxLogging { .. })
        || matches!(s.goal, crate::engine::Goal::LinuxLogging { .. })
    {
        return crate::linux_logging_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state,State::VirtualIsolation{..})||matches!(s.initial,State::VirtualIsolation{..})||matches!(s.goal,crate::engine::Goal::VirtualIsolation{..}){return crate::virtual_isolation_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::SieveFiltering{..})||matches!(s.initial,State::SieveFiltering{..})||matches!(s.goal,crate::engine::Goal::SieveFiltering{..}){return crate::sieve_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LinuxRouter{..})||matches!(s.initial,State::LinuxRouter{..})||matches!(s.goal,crate::engine::Goal::LinuxRouter{..}){return crate::linux_router_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LinuxV6{..})||matches!(s.initial,State::LinuxV6{..})||matches!(s.goal,crate::engine::Goal::LinuxV6{..}){return crate::linux_v6_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LinuxServiceRouter{..})||matches!(s.initial,State::LinuxServiceRouter{..})||matches!(s.goal,crate::engine::Goal::LinuxServiceRouter{..}){return crate::linux_service_router_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LinuxV6Administration{..})||matches!(s.initial,State::LinuxV6Administration{..})||matches!(s.goal,crate::engine::Goal::LinuxV6Administration{..}){return crate::linux_v6_admin_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LinuxV6Startup{..})||matches!(s.initial,State::LinuxV6Startup{..})||matches!(s.goal,crate::engine::Goal::LinuxV6Startup{..}){return crate::linux_v6_startup_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::IdsRule{..})||matches!(s.initial,State::IdsRule{..})||matches!(s.goal,crate::engine::Goal::IdsRule{..}){return crate::ids_rule_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::AuthFilter{..})||matches!(s.initial,State::AuthFilter{..})||matches!(s.goal,crate::engine::Goal::AuthFilter{..}){return crate::auth_filter_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::VpnPoint{..})||matches!(s.initial,State::VpnPoint{..})||matches!(s.goal,crate::engine::Goal::VpnPoint{..}){return crate::vpn_point_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Poe{..})||matches!(s.initial,State::Poe{..})||matches!(s.goal,crate::engine::Goal::Poe{..}){return crate::poe_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::KernelBuild{..})||matches!(s.initial,State::KernelBuild{..})||matches!(s.goal,crate::engine::Goal::KernelBuild{..}){return crate::kernel_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Dkms{..})||matches!(s.initial,State::Dkms{..})||matches!(s.goal,crate::engine::Goal::Dkms{..}){return crate::dkms_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Initramfs{..})||matches!(s.initial,State::Initramfs{..})||matches!(s.goal,crate::engine::Goal::Initramfs{..}){return crate::initramfs_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Pxe{..})||matches!(s.initial,State::Pxe{..})||matches!(s.goal,crate::engine::Goal::Pxe{..}){return crate::pxe::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Iscsi{..})||matches!(s.initial,State::Iscsi{..})||matches!(s.goal,crate::engine::Goal::Iscsi{..}){return crate::iscsi::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::AtaSmart{..})||matches!(s.initial,State::AtaSmart{..})||matches!(s.goal,crate::engine::Goal::AtaSmart{..}){return crate::ata_smart::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::ApacheHttps{..})||matches!(s.initial,State::ApacheHttps{..})||matches!(s.goal,crate::engine::Goal::ApacheHttps{..}){return crate::apache_https::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Desktop{..})||matches!(s.initial,State::Desktop{..})||matches!(s.goal,crate::engine::Goal::Desktop{..}){return crate::desktop::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::SecurityInvestigation{..})||matches!(s.initial,State::SecurityInvestigation{..})||matches!(s.goal,crate::engine::Goal::SecurityInvestigation{..}){return crate::security_investigation::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::AutomationWorkshop{..})||matches!(s.initial,State::AutomationWorkshop{..})||matches!(s.goal,crate::engine::Goal::AutomationWorkshop{..}){return crate::automation_workshop::source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::ServiceExposure{..})||matches!(s.initial,State::ServiceExposure{..})||matches!(s.goal,crate::engine::Goal::ServiceExposure{..}){return crate::service_exposure_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::FtpIdentity{..})||matches!(s.initial,State::FtpIdentity{..})||matches!(s.goal,crate::engine::Goal::FtpIdentity{..}){return crate::ftp_identity_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::LinuxFlow{..})||matches!(s.initial,State::LinuxFlow{..})||matches!(s.goal,crate::engine::Goal::LinuxFlow{..}){return crate::linux_flow_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::MailboxAccess{..}|State::MailboxAdministration{..})||matches!(s.initial,State::MailboxAccess{..}|State::MailboxAdministration{..})||matches!(s.goal,crate::engine::Goal::MailboxAccess{..}|crate::engine::Goal::MailboxAdministration{..}){return crate::mailbox_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::SmtpRouting{..})||matches!(s.initial,State::SmtpRouting{..})||matches!(s.goal,crate::engine::Goal::SmtpRouting{..}){return crate::smtp_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::GuestDrivers{..})||matches!(s.initial,State::GuestDrivers{..})||matches!(s.goal,crate::engine::Goal::GuestDrivers{..}){return crate::guest_driver_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::SambaShare{..})||matches!(s.initial,State::SambaShare{..})||matches!(s.goal,crate::engine::Goal::SambaShare{..}){return crate::samba_share_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::CampusArchitecture{..})||matches!(s.initial,State::CampusArchitecture{..})||matches!(s.goal,crate::engine::Goal::CampusArchitecture{..}){return crate::network_architecture_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::NetworkTransfer{..})||matches!(s.initial,State::NetworkTransfer{..})||matches!(s.goal,crate::engine::Goal::NetworkTransfer{..}){return crate::network_transfer_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::EditorHandover{..})||matches!(s.initial,State::EditorHandover{..})||matches!(s.goal,crate::engine::Goal::EditorHandover{..}){return crate::editor_handover_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::ModalEditing{..})||matches!(s.initial,State::ModalEditing{..})||matches!(s.goal,crate::engine::Goal::ModalEditing{..}){return crate::modal_source::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::SshAccess{..})||matches!(s.initial,State::SshAccess{..})||matches!(s.goal,crate::engine::Goal::SshAccess{..}){return crate::ssh_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state, State::HostNetwork { .. })
        || matches!(s.initial, State::HostNetwork { .. })
        || matches!(s.goal, crate::engine::Goal::HostNetwork { .. })
    {
        return crate::host_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Repositories { .. })
        || matches!(s.initial, State::Repositories { .. })
        || matches!(s.goal, crate::engine::Goal::Repositories { .. })
    {
        return crate::repository_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if crate::maintenance_notice_workspace::relevant(a,s) {return crate::maintenance_notice_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::dns_access_workspace::relevant(a,s) {return crate::dns_access_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::print_queue_workspace::relevant(a,s) {return crate::print_queue_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::localization_workspace::relevant(a,s) {return crate::localization_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::source_install_workspace::relevant(a,s) {return crate::source_install_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::capacity_planning_workspace::relevant(a,s) {return crate::capacity_planning_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if crate::resource_observation_workspace::relevant(a,s) {return crate::resource_observation_workspace::context(a,s).is_ok()&&crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state, State::Capacity { .. })
        || matches!(s.initial, State::Capacity { .. })
        || matches!(s.goal, crate::engine::Goal::Capacity { .. })
    {
        return crate::capacity_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state,State::KernelModules{..}) || matches!(s.initial,State::KernelModules{..}) || matches!(s.goal,crate::engine::Goal::KernelModules{..}) {return crate::kernel_module_source::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::RecursiveTree{..}) || matches!(s.initial,State::RecursiveTree{..}) || matches!(s.goal,crate::engine::Goal::RecursiveTree{..}) {
        return crate::recursive_tree_source::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);
    }
    if matches!(a.state,State::Transport{..}) || matches!(s.initial,State::Transport{..}) || matches!(s.goal,crate::engine::Goal::Transport{..}) {
        return crate::transport_workspace::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);
    }
    if matches!(a.state,State::RaidAssembly{..}) || matches!(s.initial,State::RaidAssembly{..}) || matches!(s.goal,crate::engine::Goal::RaidAssembly{..}) {return crate::raid_assembly_source::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state,State::Autofs{..}) || matches!(s.initial,State::Autofs{..}) || matches!(s.goal,crate::engine::Goal::Autofs{..}) {return crate::autofs_source::context(a,s).is_ok() && crate::engine::solved(&a.state,&s.goal);}
    if matches!(a.state, State::DirectoryPolicy { .. })
        || matches!(s.initial, State::DirectoryPolicy { .. })
        || matches!(s.goal, crate::engine::Goal::DirectoryPolicy { .. })
    {
        return crate::directory_policy_source::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Links { .. })
        || matches!(s.initial, State::Links { .. })
        || matches!(s.goal, crate::engine::Goal::Links { .. })
    {
        return crate::links_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if let State::MagicSubnet { session } = &a.state {
        return session.checked.as_ref() == Some(&session.answers)
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::PartitionFs { .. })
        || matches!(s.initial, State::PartitionFs { .. })
        || matches!(s.goal, crate::engine::Goal::PartitionFs { .. })
    {
        return crate::partition_fs_workspace::binding::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Hsrp { .. }) || matches!(s.initial, State::Hsrp { .. }) {
        return crate::hsrp_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Syslog { .. })
        || matches!(s.initial, State::Syslog { .. })
        || matches!(s.goal, crate::engine::Goal::Syslog { .. })
    {
        return crate::syslog_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Ntp { .. }) || matches!(s.initial, State::Ntp { .. }) {
        return crate::ntp_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if matches!(a.state, State::Lvm { .. }) || matches!(s.initial, State::Lvm { .. }) {
        return crate::lvm_workspace::context(a, s).is_ok()
            && crate::engine::solved(&a.state, &s.goal);
    }
    if crate::physical_link_workspace::metadata(a).is_err() {
        return false;
    }
    if matches!(a.state, State::PhysicalLink { .. }) {
        let rows = crate::physical_link_workspace::checks(a, s);
        return !rows.is_empty() && rows.iter().all(|r| r.0);
    }
    if s.physical_link_teaching.is_some() {
        return false;
    }

    if matches!(a.state, State::OneOff { .. }) {
        let rows = current_one_off_checks(a, s);
        return !rows.is_empty() && rows.iter().all(|r| r.0);
    }
    if crate::workspace::has_one_off_metadata(a)
        || crate::one_off_workspace::reject_foreign_source(s).is_err()
    {
        return false;
    }
    if crate::etherchannel_workspace::reject_foreign_source(s).is_err()
        || crate::etherchannel_workspace::metadata(a).is_err()
    {
        return false;
    }
    if matches!(a.state, State::EtherChannel { .. }) {
        let rows = current_etherchannel_checks(a, s);
        return !rows.is_empty() && rows.iter().all(|r| r.0);
    }

    if validate_file_backup_workspace(a).is_err() {
        return false;
    }
    if matches!(a.state, State::FileBackup { .. }) {
        let rows = current_file_backup_checks(a, s);
        return !rows.is_empty() && rows.iter().all(|r| r.0);
    }
    if matches!(a.state, State::Cron { .. }) {
        let rows = current_cron_checks(a, s);
        return !rows.is_empty() && rows.iter().all(|r| r.0);
    }
    if validate_cron_metadata(a).is_err() || s.cron_teaching.is_some() {
        return false;
    }
    if matches!(a.state, State::Nfs { .. }) {
        let rows = current_nfs_checks(a, s);
        return !rows.is_empty() && rows.iter().all(|r| r.0);
    }
    if validate_nfs_workspace(a).is_err() {
        return false;
    }

    if matches!(a.state, State::RpmPackages { .. }) {
        return current_rpm_checks(a, s).iter().all(|r| r.0);
    }
    if validate_rpm_workspace(a).is_err() {
        return false;
    }
    if validate_nat_workspace(a).is_err() {
        return false;
    }
    if matches!(a.state, State::BasicNat { .. }) {
        return current_nat_checks(a, s).iter().all(|r| r.0);
    }
    if matches!(a.state, State::ApacheStatic { .. }) {
        return current_apache_static_checks(a, s).iter().all(|r| r.0);
    }
    if validate_apache_static_workspace(a).is_err() {
        return false;
    }
    if matches!(a.state, State::DebianPackages { .. }) {
        return current_debian_packages_checks(a, s).iter().all(|r| r.0);
    }
    if validate_debian_packages_workspace(a).is_err() {
        return false;
    }
    if matches!(a.state, State::SpanningTree { .. }) {
        return current_spanning_tree_checks(a, s).iter().all(|r| r.0);
    }
    if validate_spanning_tree_workspace(a).is_err() {
        return false;
    }
    if matches!(a.state, State::DhcpV4 { .. }) {
        return current_dhcpv4_checks(a, s).iter().all(|row| row.0);
    }
    if validate_dhcpv4_workspace(a).is_err() {
        return false;
    }
    if validate_legacy_teaching_workspace(a).is_err() {
        return false;
    }
    if a.workspace
        .as_ref()
        .and_then(|w| w.legacy_access_learning.as_ref())
        .is_some_and(|l| {
            s.legacy_access_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        })
        || a.workspace
            .as_ref()
            .and_then(|w| w.legacy_logs_learning.as_ref())
            .is_some_and(|l| {
                s.legacy_logs_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            })
    {
        return false;
    }
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
        if crate::accounts_template_workspace::relevant(a,s){crate::accounts_template_workspace::context(a,s)?;}
        if crate::command_location_workspace::relevant(a,s){crate::command_location_workspace::context(a,s)?;}

        if matches!(a.state, State::Syslog { .. })
            || matches!(s.initial, State::Syslog { .. })
            || matches!(s.goal, crate::engine::Goal::Syslog { .. })
        {
            crate::syslog_workspace::context(a, s)?;
        }
        if crate::maintenance_notice_workspace::relevant(a,s){crate::maintenance_notice_workspace::context(a,s)?;}
        if crate::dns_access_workspace::relevant(a,s){crate::dns_access_workspace::context(a,s)?;}
        if crate::print_queue_workspace::relevant(a,s){crate::print_queue_workspace::context(a,s)?;}
        if crate::localization_workspace::relevant(a,s){crate::localization_workspace::context(a,s)?;}
        if crate::source_install_workspace::relevant(a,s){crate::source_install_workspace::context(a,s)?;}
        if crate::capacity_planning_workspace::relevant(a,s){crate::capacity_planning_workspace::context(a,s)?;}
        if crate::resource_observation_workspace::relevant(a,s){crate::resource_observation_workspace::context(a,s)?;}
        if matches!(a.state, State::Capacity { .. })
            || matches!(s.initial, State::Capacity { .. })
            || matches!(s.goal, crate::engine::Goal::Capacity { .. })
        {
            crate::capacity_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Repositories { .. })
            || matches!(s.initial, State::Repositories { .. })
            || matches!(s.goal, crate::engine::Goal::Repositories { .. })
        {
            crate::repository_workspace::context(a, s)?;
        }
        if matches!(a.state, State::StartupFiles { .. })
            || matches!(s.initial, State::StartupFiles { .. })
            || matches!(s.goal, crate::engine::Goal::StartupFiles { .. })
        {
            crate::startup_workspace::context(a, s)?;
        }
        if matches!(a.state,State::Raid1{..})||matches!(s.initial,State::Raid1{..})||matches!(s.goal,crate::engine::Goal::Raid1{..}){crate::raid1_workspace::context(a,s)?;}
        if crate::file_index_source::relevant(a,s){crate::file_index_source::context(a,s)?;}
        if matches!(a.state,State::SharedLibraries{..})||matches!(s.initial,State::SharedLibraries{..})||matches!(s.goal,crate::engine::Goal::SharedLibraries{..}){crate::shared_libraries_workspace::context(a,s)?;}
        if matches!(a.state, State::FsIntegrity { .. })
            || matches!(s.initial, State::FsIntegrity { .. })
            || matches!(s.goal, crate::engine::Goal::FsIntegrity { .. })
        {
            crate::fs_integrity_workspace::context(a, s)?;
        }
        if matches!(a.state, State::UserMounts { .. })
            || matches!(s.initial, State::UserMounts { .. })
            || matches!(s.goal, crate::engine::Goal::UserMounts { .. })
        {
            crate::user_mounts_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Environment { .. })
            || matches!(s.initial, State::Environment { .. })
            || matches!(s.goal, crate::engine::Goal::Environment { .. })
        {
            crate::environment_workspace::context(a, s)?;
        }
        if matches!(a.state, State::DebianSources { .. })
            || matches!(s.initial, State::DebianSources { .. })
            || matches!(s.goal, crate::engine::Goal::DebianSources { .. })
        {
            crate::debian_sources_workspace::context(a, s)?;
        }
        if matches!(a.state, State::ClientNames { .. })
            || matches!(s.initial, State::ClientNames { .. })
            || matches!(s.goal, crate::engine::Goal::ClientNames { .. })
        {
            crate::client_names_workspace::context(a, s)?;
        }
        if matches!(a.state, State::RemovableMounts { .. })
            || matches!(s.initial, State::RemovableMounts { .. })
            || matches!(s.goal, crate::engine::Goal::RemovableMounts { .. })
        {
            crate::removable_workspace::context(a, s)?;
        }
        if matches!(a.state, State::NmProfiles { .. })
            || matches!(s.initial, State::NmProfiles { .. })
            || matches!(s.goal, crate::engine::Goal::NmProfiles { .. })
        {
            crate::nm_workspace::context(a, s)?;
        }
        if matches!(a.state, State::PamStack { .. })
            || matches!(s.initial, State::PamStack { .. })
            || matches!(s.goal, crate::engine::Goal::PamStack { .. })
        {
            crate::pam_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Aging { .. })
            || matches!(s.initial, State::Aging { .. })
            || matches!(s.goal, crate::engine::Goal::Aging { .. })
        {
            crate::aging_workspace::context(a, s)?;
        }
        if matches!(a.state, State::PortSecurity { .. })
            || matches!(s.initial, State::PortSecurity { .. })
            || matches!(s.goal, crate::engine::Goal::PortSecurity { .. })
        {
            crate::port_security_workspace::context(a, s)?;
        }
        if matches!(a.state, State::DhcpSnooping { .. })
            || matches!(s.initial, State::DhcpSnooping { .. })
            || matches!(s.goal, crate::engine::Goal::DhcpSnooping { .. })
        {
            crate::dhcp_snooping_workspace::context(a, s)?;
        }
        if matches!(a.state, State::ArpInspection { .. })
            || matches!(s.initial, State::ArpInspection { .. })
            || matches!(s.goal, crate::engine::Goal::ArpInspection { .. })
        {
            crate::arp_inspection_workspace::context(a, s)?;
        }
        if matches!(a.state, State::ManagementPolicy { .. })
            || matches!(s.initial, State::ManagementPolicy { .. })
            || matches!(s.goal, crate::engine::Goal::ManagementPolicy { .. })
        {
            crate::management_workspace::context(a, s)?;
        }
        if matches!(a.state, State::LocalAccess { .. })
            || matches!(s.initial, State::LocalAccess { .. })
            || matches!(s.goal, crate::engine::Goal::LocalAccess { .. })
        {
            crate::local_access_workspace::context(a, s)?;
        }
        if matches!(a.state, State::AaaAccess { .. })
            || matches!(s.initial, State::AaaAccess { .. })
            || matches!(s.goal, crate::engine::Goal::AaaAccess { .. })
        {
            crate::aaa_workspace::context(a, s)?;
        }
        if matches!(a.state, State::LdapClient { .. })
            || matches!(s.initial, State::LdapClient { .. })
            || matches!(s.goal, crate::engine::Goal::LdapClient { .. })
        {
            crate::ldap_workspace::context(a, s)?;
        }
        if matches!(a.state, State::LdapAccess { .. })
            || matches!(s.initial, State::LdapAccess { .. })
            || matches!(s.goal, crate::engine::Goal::LdapAccess { .. })
        {
            crate::ldap_access_workspace::context(a, s)?;
        }
        if matches!(a.state, State::LdapPassword { .. })
            || matches!(s.initial, State::LdapPassword { .. })
            || matches!(s.goal, crate::engine::Goal::LdapPassword { .. })
        {
            crate::ldap_password_workspace::context(a, s)?;
        }
        if matches!(a.state, State::LdapChanges { .. })
            || matches!(s.initial, State::LdapChanges { .. })
            || matches!(s.goal, crate::engine::Goal::LdapChanges { .. })
        {
            crate::ldap_change_workspace::context(a, s)?;
        }
        if matches!(a.state, State::DhcpConfig { .. })
            || matches!(s.initial, State::DhcpConfig { .. })
            || matches!(s.goal, crate::engine::Goal::DhcpConfig { .. })
        {
            crate::dhcp_server_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Qos { .. })
            || matches!(s.initial, State::Qos { .. })
            || matches!(s.goal, crate::engine::Goal::Qos { .. })
        {
            crate::qos_workspace::context(a, s)?;
        }
        if matches!(a.state, State::LinuxLogging { .. })
            || matches!(s.initial, State::LinuxLogging { .. })
            || matches!(s.goal, crate::engine::Goal::LinuxLogging { .. })
        {
            crate::linux_logging_workspace::context(a, s)?;
        }
        if matches!(a.state, State::HostNetwork { .. })
            || matches!(s.initial, State::HostNetwork { .. })
            || matches!(s.goal, crate::engine::Goal::HostNetwork { .. })
        {
            crate::host_workspace::context(a, s)?;
        }
        if matches!(a.state,State::VirtualIsolation{..})||matches!(s.initial,State::VirtualIsolation{..})||matches!(s.goal,crate::engine::Goal::VirtualIsolation{..}){crate::virtual_isolation_source::context(a,s)?;}
        if matches!(a.state,State::SieveFiltering{..})||matches!(s.initial,State::SieveFiltering{..})||matches!(s.goal,crate::engine::Goal::SieveFiltering{..}){crate::sieve_source::context(a,s)?;}
        if matches!(a.state,State::LinuxRouter{..})||matches!(s.initial,State::LinuxRouter{..})||matches!(s.goal,crate::engine::Goal::LinuxRouter{..}){crate::linux_router_source::context(a,s)?;}
        if matches!(a.state,State::LinuxV6{..})||matches!(s.initial,State::LinuxV6{..})||matches!(s.goal,crate::engine::Goal::LinuxV6{..}){crate::linux_v6_source::context(a,s)?;}
        if matches!(a.state,State::LinuxServiceRouter{..})||matches!(s.initial,State::LinuxServiceRouter{..})||matches!(s.goal,crate::engine::Goal::LinuxServiceRouter{..}){crate::linux_service_router_source::context(a,s)?;}
        if matches!(a.state,State::LinuxV6Administration{..})||matches!(s.initial,State::LinuxV6Administration{..})||matches!(s.goal,crate::engine::Goal::LinuxV6Administration{..}){crate::linux_v6_admin_source::context(a,s)?;}
        if matches!(a.state,State::LinuxV6Startup{..})||matches!(s.initial,State::LinuxV6Startup{..})||matches!(s.goal,crate::engine::Goal::LinuxV6Startup{..}){crate::linux_v6_startup_source::context(a,s)?;}
        if matches!(a.state,State::IdsRule{..})||matches!(s.initial,State::IdsRule{..})||matches!(s.goal,crate::engine::Goal::IdsRule{..}){crate::ids_rule_source::context(a,s)?;}
        if matches!(a.state,State::AuthFilter{..})||matches!(s.initial,State::AuthFilter{..})||matches!(s.goal,crate::engine::Goal::AuthFilter{..}){crate::auth_filter_source::context(a,s)?;}
        if matches!(a.state,State::VpnPoint{..})||matches!(s.initial,State::VpnPoint{..})||matches!(s.goal,crate::engine::Goal::VpnPoint{..}){crate::vpn_point_source::context(a,s)?;}
        if matches!(a.state,State::Poe{..})||matches!(s.initial,State::Poe{..})||matches!(s.goal,crate::engine::Goal::Poe{..}){crate::poe_source::context(a,s)?;}
        if matches!(a.state,State::KernelBuild{..})||matches!(s.initial,State::KernelBuild{..})||matches!(s.goal,crate::engine::Goal::KernelBuild{..}){crate::kernel_source::context(a,s)?;}
        if matches!(a.state,State::Dkms{..})||matches!(s.initial,State::Dkms{..})||matches!(s.goal,crate::engine::Goal::Dkms{..}){crate::dkms_source::context(a,s)?;}
        if matches!(a.state,State::Initramfs{..})||matches!(s.initial,State::Initramfs{..})||matches!(s.goal,crate::engine::Goal::Initramfs{..}){crate::initramfs_source::context(a,s)?;}
        if matches!(a.state,State::Pxe{..})||matches!(s.initial,State::Pxe{..})||matches!(s.goal,crate::engine::Goal::Pxe{..}){crate::pxe::source::context(a,s)?;}
        if matches!(a.state,State::Iscsi{..})||matches!(s.initial,State::Iscsi{..})||matches!(s.goal,crate::engine::Goal::Iscsi{..}){crate::iscsi::source::context(a,s)?;}
        if matches!(a.state,State::AtaSmart{..})||matches!(s.initial,State::AtaSmart{..})||matches!(s.goal,crate::engine::Goal::AtaSmart{..}){crate::ata_smart::source::context(a,s)?;}
        if matches!(a.state,State::ApacheHttps{..})||matches!(s.initial,State::ApacheHttps{..})||matches!(s.goal,crate::engine::Goal::ApacheHttps{..}){crate::apache_https::source::context(a,s)?;}
        if matches!(a.state,State::Desktop{..})||matches!(s.initial,State::Desktop{..})||matches!(s.goal,crate::engine::Goal::Desktop{..}){crate::desktop::source::context(a,s)?;}
        if matches!(a.state,State::SecurityInvestigation{..})||matches!(s.initial,State::SecurityInvestigation{..})||matches!(s.goal,crate::engine::Goal::SecurityInvestigation{..}){crate::security_investigation::source::context(a,s)?;}
        if matches!(a.state,State::AutomationWorkshop{..})||matches!(s.initial,State::AutomationWorkshop{..})||matches!(s.goal,crate::engine::Goal::AutomationWorkshop{..}){crate::automation_workshop::source::context(a,s)?;}
        if matches!(a.state,State::ServiceExposure{..})||matches!(s.initial,State::ServiceExposure{..})||matches!(s.goal,crate::engine::Goal::ServiceExposure{..}){crate::service_exposure_source::context(a,s)?;}
        if matches!(a.state,State::FtpIdentity{..})||matches!(s.initial,State::FtpIdentity{..})||matches!(s.goal,crate::engine::Goal::FtpIdentity{..}){crate::ftp_identity_source::context(a,s)?;}
        if matches!(a.state,State::LinuxFlow{..})||matches!(s.initial,State::LinuxFlow{..})||matches!(s.goal,crate::engine::Goal::LinuxFlow{..}){crate::linux_flow_source::context(a,s)?;}
        if matches!(a.state,State::MailboxAccess{..}|State::MailboxAdministration{..})||matches!(s.initial,State::MailboxAccess{..}|State::MailboxAdministration{..})||matches!(s.goal,crate::engine::Goal::MailboxAccess{..}|crate::engine::Goal::MailboxAdministration{..}){crate::mailbox_source::context(a,s)?;}
        if matches!(a.state,State::SmtpRouting{..})||matches!(s.initial,State::SmtpRouting{..})||matches!(s.goal,crate::engine::Goal::SmtpRouting{..}){crate::smtp_source::context(a,s)?;}
        if matches!(a.state,State::GuestDrivers{..})||matches!(s.initial,State::GuestDrivers{..})||matches!(s.goal,crate::engine::Goal::GuestDrivers{..}){crate::guest_driver_source::context(a,s)?;}
        if matches!(a.state,State::SambaShare{..})||matches!(s.initial,State::SambaShare{..})||matches!(s.goal,crate::engine::Goal::SambaShare{..}){crate::samba_share_source::context(a,s)?;}
        if matches!(a.state,State::CampusArchitecture{..})||matches!(s.initial,State::CampusArchitecture{..})||matches!(s.goal,crate::engine::Goal::CampusArchitecture{..}){crate::network_architecture_source::context(a,s)?;}
        if matches!(a.state,State::NetworkTransfer{..})||matches!(s.initial,State::NetworkTransfer{..})||matches!(s.goal,crate::engine::Goal::NetworkTransfer{..}){crate::network_transfer_source::context(a,s)?;}
        if matches!(a.state,State::EditorHandover{..})||matches!(s.initial,State::EditorHandover{..})||matches!(s.goal,crate::engine::Goal::EditorHandover{..}){crate::editor_handover_source::context(a,s)?;}
        if matches!(a.state,State::ModalEditing{..})||matches!(s.initial,State::ModalEditing{..})||matches!(s.goal,crate::engine::Goal::ModalEditing{..}){crate::modal_source::context(a,s)?;}
        if matches!(a.state,State::SshAccess{..})||matches!(s.initial,State::SshAccess{..})||matches!(s.goal,crate::engine::Goal::SshAccess{..}){
            crate::ssh_workspace::context(a,s)?;
        }
        crate::physical_link_workspace::reject_foreign_source(s)?;
        if matches!(a.state,State::KernelModules{..}) || matches!(s.initial,State::KernelModules{..}) || matches!(s.goal,crate::engine::Goal::KernelModules{..}) {crate::kernel_module_source::context(a,s)?;}
        if matches!(a.state,State::RecursiveTree{..}) || matches!(s.initial,State::RecursiveTree{..}) || matches!(s.goal,crate::engine::Goal::RecursiveTree{..}) {crate::recursive_tree_source::context(a,s)?;}
        if matches!(a.state,State::Transport{..}) || matches!(s.initial,State::Transport{..}) || matches!(s.goal,crate::engine::Goal::Transport{..}) {crate::transport_workspace::context(a,s)?;}
        if matches!(a.state,State::RaidAssembly{..}) || matches!(s.initial,State::RaidAssembly{..}) || matches!(s.goal,crate::engine::Goal::RaidAssembly{..}) {crate::raid_assembly_source::context(a,s)?;}
        if matches!(a.state,State::CopiedJournal{..}) || matches!(s.initial,State::CopiedJournal{..}) || matches!(s.goal,crate::engine::Goal::CopiedJournal{..}) {crate::journal_archive_workspace::context(a,s)?;}
        if matches!(a.state,State::JsonInventory{..}) || matches!(s.initial,State::JsonInventory{..}) || matches!(s.goal,crate::engine::Goal::JsonInventory{..}) {crate::json_inventory_workspace::context(a,s)?;}
        if matches!(a.state,State::RestController{..}) || matches!(s.initial,State::RestController{..}) || matches!(s.goal,crate::engine::Goal::RestController{..}) {crate::rest_workspace::context(a,s)?;}
        if matches!(a.state,State::DevicePath{..}) || matches!(s.initial,State::DevicePath{..}) || matches!(s.goal,crate::engine::Goal::DevicePath{..}) {crate::device_path_workspace::context(a,s)?;}
        if matches!(a.state,State::LocalMail{..}) || matches!(s.initial,State::LocalMail{..}) || matches!(s.goal,crate::engine::Goal::LocalMail{..}) {crate::local_mail_workspace::context(a,s)?;}
        if matches!(a.state,State::Chrony{..}) || matches!(s.initial,State::Chrony{..}) || matches!(s.goal,crate::engine::Goal::Chrony{..}) {crate::chrony_workspace::context(a,s)?;}
        if matches!(a.state,State::GrubOnce{..}) || matches!(s.initial,State::GrubOnce{..}) || matches!(s.goal,crate::engine::Goal::GrubOnce{..}) {crate::grub_once_workspace::context(a,s)?;}
        if matches!(a.state,State::EfiLoader{..}) || matches!(s.initial,State::EfiLoader{..}) || matches!(s.goal,crate::engine::Goal::EfiLoader{..}) {crate::efi_loader_workspace::context(a,s)?;}
        if matches!(a.state,State::BiosLoader{..}) || matches!(s.initial,State::BiosLoader{..}) || matches!(s.goal,crate::engine::Goal::BiosLoader{..}) {crate::bios_loader_workspace::context(a,s)?;}
        if matches!(a.state,State::BootEntry{..}) || matches!(s.initial,State::BootEntry{..}) || matches!(s.goal,crate::engine::Goal::BootEntry{..}) {crate::boot_entry_workspace::context(a,s)?;}
        if matches!(a.state,State::Autofs{..}) || matches!(s.initial,State::Autofs{..}) || matches!(s.goal,crate::engine::Goal::Autofs{..}) {crate::autofs_source::context(a,s)?;}
        if matches!(a.state, State::DirectoryPolicy { .. })
            || matches!(s.initial, State::DirectoryPolicy { .. })
            || matches!(s.goal, crate::engine::Goal::DirectoryPolicy { .. })
        {
            crate::directory_policy_source::context(a, s)?;
        }
        if matches!(a.state, State::Links { .. })
            || matches!(s.initial, State::Links { .. })
            || matches!(s.goal, crate::engine::Goal::Links { .. })
        {
            crate::links_workspace::context(a, s)?;
        }
        if a.scenario_version != s.version
            || std::mem::discriminant(&a.state) != std::mem::discriminant(&s.initial)
            || a.step >= s.steps.len()
            || a.hints > s.hints.len()
            || a.transcript.len() > 120
        {
            return Err("Incompatible saved attempt; original retained".into());
        }
        if let State::MagicSubnet { session } = &a.state {
            let crate::engine::Goal::MagicSubnet { problem, stage } = &s.goal else {
                return Err(
                    "Subnet worksheet belongs to another assignment; original retained".into(),
                );
            };
            if session.problem != *problem || session.stage != *stage {
                return Err(
                    "Saved subnet problem differs from the assignment; original retained".into(),
                );
            }
        }
        crate::partition_fs_workspace::binding::metadata(a)?;
        if matches!(a.state, State::PartitionFs { .. })
            || matches!(s.initial, State::PartitionFs { .. })
            || matches!(s.goal, crate::engine::Goal::PartitionFs { .. })
        {
            crate::partition_fs_workspace::binding::context(a, s)?;
        }
        crate::hsrp_workspace::metadata(a)?;
        crate::syslog_workspace::metadata(a)?;
        crate::ntp_workspace::metadata(a)?;
        crate::lvm_workspace::metadata(a)?;
        crate::physical_link_workspace::metadata(a)?;
        crate::one_off_workspace::metadata(a)?;
        if matches!(a.state, State::OneOff { .. }) {
            crate::one_off_workspace::context(a, s)?;
        }
        validate_nfs_workspace(a)?;
        validate_rpm_workspace(a)?;
        validate_nat_workspace(a)?;
        validate_apache_static_workspace(a)?;
        validate_debian_packages_workspace(a)?;
        validate_dhcpv4_workspace(a)?;
        validate_accounts_workspace(a)?;
        validate_legacy_teaching_workspace(a)?;
        validate_ospf_workspace(a)?;
        validate_spanning_tree_workspace(a)?;
        validate_unit_manager_workspace(a)?;
        if a.workspace
            .as_ref()
            .and_then(|w| w.legacy_access_learning.as_ref())
            .is_some_and(|l| {
                s.legacy_access_teaching
                    .as_ref()
                    .is_none_or(|lesson| !l.valid_for(lesson))
            })
            || a.workspace
                .as_ref()
                .and_then(|w| w.legacy_logs_learning.as_ref())
                .is_some_and(|l| {
                    s.legacy_logs_teaching
                        .as_ref()
                        .is_none_or(|lesson| !l.valid_for(lesson))
                })
        {
            return Err(
                "Saved access/log practice differs from its lesson; original retained".into(),
            );
        }
        if matches!(a.state, State::Hsrp { .. }) {
            crate::hsrp_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Ntp { .. }) {
            crate::ntp_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Lvm { .. }) {
            crate::lvm_workspace::context(a, s)?;
        }
        if matches!(a.state, State::PhysicalLink { .. }) {
            crate::physical_link_workspace::context(a, s)?;
        }
        if matches!(a.state, State::FileBackup { .. }) {
            crate::file_backup_workspace::context(a, s)?;
        }
        if matches!(a.state, State::EtherChannel { .. }) {
            crate::etherchannel_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Cron { .. }) {
            crate::cron_workspace::context(a, s)?;
        }
        if matches!(a.state, State::Nfs { .. }) {
            crate::nfs_workspace::context(a, s)?;
        }
        if let State::RpmPackages { simulation } = &a.state {
            let (
                State::RpmPackages {
                    simulation: authored,
                },
                crate::engine::Goal::RpmPackages { definition },
            ) = (&s.initial, &s.goal)
            else {
                return Err("RPM state belongs to another activity".into());
            };
            if authored != &definition.initial
                || !definition.same_assignment(simulation)
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.rpm_learning.as_ref())
                    .is_some_and(|l| {
                        s.rpm_packages_teaching.as_ref().is_none_or(|source| {
                            !source.separate_from_definition(definition) || !l.valid_for(source)
                        })
                    })
            {
                return Err(
                    "Saved RPM authored catalog/practice context differs; original retained".into(),
                );
            }
        }
        if let State::DebianPackages { simulation } = &a.state {
            let (
                State::DebianPackages {
                    simulation: authored,
                },
                crate::engine::Goal::DebianPackages { definition },
            ) = (&s.initial, &s.goal)
            else {
                return Err("Package state belongs to another activity".into());
            };
            if authored != &definition.initial || !definition.same_assignment(simulation) {
                return Err(
                    "Saved package catalog/initial assignment differs; original retained".into(),
                );
            }
            if a.workspace
                .as_ref()
                .and_then(|w| w.debian_packages_learning.as_ref())
                .is_some_and(|l| {
                    s.debian_packages_teaching
                        .as_ref()
                        .is_none_or(|lesson| !l.valid_for(lesson))
                })
            {
                return Err("Saved package practice fixture differs; original retained".into());
            }
        }
        if let State::DhcpV4 { simulation } = &a.state {
            let (
                State::DhcpV4 {
                    simulation: authored,
                },
                crate::engine::Goal::DhcpV4 { initial, .. },
            ) = (&s.initial, &s.goal)
            else {
                return Err("DHCP state belongs to another activity; original retained".into());
            };
            if authored != initial || simulation.validate_context(initial.topology()).is_err() {
                return Err(
                    "Saved DHCP immutable topology differs from this assignment; original retained"
                        .into(),
                );
            }
            if a.workspace
                .as_ref()
                .and_then(|w| w.dhcpv4_learning.as_ref())
                .is_some_and(|l| {
                    s.dhcpv4_teaching
                        .as_ref()
                        .is_none_or(|lesson| !l.valid_for(lesson))
                })
            {
                return Err(
                    "Saved DHCP practice differs from its authored fixture; original retained"
                        .into(),
                );
            }
        }
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

        if let State::BasicNat { simulation } = &a.state {
            let (
                State::BasicNat {
                    simulation: authored,
                },
                crate::engine::Goal::BasicNat { definition },
            ) = (&s.initial, &s.goal)
            else {
                return Err("Saved NAT state on another activity".into());
            };
            if authored != &definition.initial || !definition.same_assignment(simulation) {
                return Err("Saved NAT origin differs from authored inventory".into());
            }
            if s.nat_teaching
                .as_ref()
                .is_some_and(|l| l.validate_for(authored).is_err())
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.nat_learning.as_ref())
                    .is_some_and(|l| {
                        s.nat_teaching
                            .as_ref()
                            .is_none_or(|lesson| !l.valid_for(lesson))
                    })
            {
                return Err("Saved NAT practice differs from authored independent fixture".into());
            }
        }
        if let State::ApacheStatic { simulation } = &a.state {
            let (
                State::ApacheStatic {
                    simulation: authored,
                },
                crate::engine::Goal::ApacheStatic { definition },
            ) = (&s.initial, &s.goal)
            else {
                return Err("Apache state on another activity".into());
            };
            if authored != &definition.initial
                || !definition.same_assignment(simulation)
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.apache_static_learning.as_ref())
                    .is_some_and(|l| {
                        s.apache_static_teaching
                            .as_ref()
                            .is_none_or(|lesson| !l.valid_for(lesson))
                    })
            {
                return Err("Saved Apache inventory/example differs from authored context; original retained".into());
            }
        }
        if let State::SpanningTree { simulation } = &a.state {
            let (
                State::SpanningTree {
                    simulation: authored,
                },
                crate::engine::Goal::SpanningTree { initial, .. },
            ) = (&s.initial, &s.goal)
            else {
                return Err(
                    "Spanning-tree state belongs to a different activity; original retained".into(),
                );
            };
            if authored != initial
                || !crate::spanning_tree_workspace::same_context(initial, simulation)
            {
                return Err(
                    "Saved spanning-tree immutable graph differs; original retained".into(),
                );
            }
            if a.workspace
                .as_ref()
                .and_then(|w| w.spanning_tree_learning.as_ref())
                .is_some_and(|l| {
                    s.spanning_tree_teaching
                        .as_ref()
                        .is_none_or(|lesson| !l.valid_for(lesson))
                })
            {
                return Err(
                    "Saved spanning-tree practice fixture differs; original retained".into(),
                );
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
        crate::one_off_workspace::metadata(a)?;
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
fn validate_nat_workspace(a: &Attempt) -> Result<(), String> {
    let draft = a.workspace.as_ref().and_then(|w| w.nat_draft.as_ref());
    let State::BasicNat { simulation } = &a.state else {
        return if draft.is_some()
            || a.workspace
                .as_ref()
                .is_some_and(|w| w.nat_learning.is_some())
        {
            Err("NAT editor metadata on another activity".into())
        } else {
            Ok(())
        };
    };
    // These older families lack a standalone wrong-kind validator. Reject only
    // their typed editors/practice on the new NAT kind; shared command/scalars,
    // historical records and provenance remain unchanged and never grade NAT.
    if a.workspace.as_ref().is_some_and(|w| {
        w.script_learning.is_some()
            || w.terminal_learning.is_some()
            || w.switching_draft.is_some()
            || w.switching_learning.is_some()
            || w.ipv6_draft.is_some()
            || w.ipv6_learning.is_some()
    }) {
        return Err("Another activity's typed editor/practice metadata is attached to NAT".into());
    }
    if a.workspace.as_ref().is_some_and(|w| !w.valid())
        || !crate::nat_workspace::admitted(simulation)
        || draft.is_some_and(|d| !d.valid_for(simulation))
        || a.workspace
            .as_ref()
            .and_then(|w| w.nat_learning.as_ref())
            .is_some_and(|l| !l.valid() || !l.separate_from(simulation))
    {
        return Err("NAT state/editor bounds or immutable inventory differ".into());
    }
    Ok(())
}
fn validate_apache_static_workspace(a: &Attempt) -> Result<(), String> {
    let metadata = a
        .workspace
        .as_ref()
        .is_some_and(|w| w.apache_static_draft.is_some() || w.apache_static_learning.is_some());
    let State::ApacheStatic { simulation } = &a.state else {
        return if metadata {
            Err("Apache metadata on another activity".into())
        } else {
            Ok(())
        };
    };
    if !crate::apache_static_workspace::admitted(simulation) {
        return Err("Apache state exceeds app admission".into());
    }
    if a.workspace.as_ref().is_some_and(|w| {
        w.apache_static_draft
            .as_ref()
            .is_some_and(|d| !d.valid_for(simulation))
            || w.apache_static_learning.as_ref().is_some_and(|l| {
                !l.valid()
                    || l.fixture.inventory.fixture == simulation.inventory.fixture
                    || l.fixture.inventory.server == simulation.inventory.server
                    || l.fixture.inventory.config_path == simulation.inventory.config_path
                    || l.fixture
                        .inventory
                        .entries
                        .iter()
                        .filter(|(_, v)| **v == crate::apache_static::EntryKind::File)
                        .any(|(p, _)| {
                            simulation.inventory.entries.get(p)
                                == Some(&crate::apache_static::EntryKind::File)
                        })
            })
    }) {
        return Err("Foreign Apache draft/example inventory; original retained".into());
    }
    Ok(())
}
fn validate_spanning_tree_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else {
        return Ok(());
    };
    match &a.state {
        State::SpanningTree { simulation } => {
            if w.spanning_tree_draft
                .as_ref()
                .is_some_and(|d| !d.valid_for(simulation))
                || w.spanning_tree_learning.as_ref().is_some_and(|l| {
                    !l.valid()
                        || !l
                            .fixture
                            .topology()
                            .bridges
                            .keys()
                            .all(|k| !simulation.topology().bridges.contains_key(k))
                        || !l
                            .fixture
                            .topology()
                            .links
                            .keys()
                            .all(|k| !simulation.topology().links.contains_key(k))
                        || l.fixture.topology().bridges.values().any(|b| {
                            simulation
                                .topology()
                                .bridges
                                .values()
                                .any(|o| b.mac == o.mac)
                        })
                })
            {
                return Err("Invalid spanning-tree draft or separate fixed practice context; original retained".into());
            }
        }
        _ if w.spanning_tree_draft.is_some() || w.spanning_tree_learning.is_some() => {
            return Err("Spanning-tree metadata on another activity; original retained".into());
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
    if a.workspace.as_ref().is_some_and(|w|w.accounts_template_drafts.is_some())
        && !matches!(a.state,State::Accounts{..}){
        return Err("Native template metadata on another activity; original retained".into());
    }

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
fn validate_legacy_teaching_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else { return Ok(()) };
    if w.legacy_access_learning
        .as_ref()
        .is_some_and(|l| !matches!(a.state, State::Unix { .. }) || !l.valid())
        || w.legacy_logs_learning
            .as_ref()
            .is_some_and(|l| !matches!(a.state, State::Logs { .. }) || !l.valid())
    {
        return Err("Invalid or wrong-kind access/log practice; original retained".into());
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
fn validate_debian_packages_workspace(a: &Attempt) -> Result<(), String> {
    let metadata = a.workspace.as_ref().is_some_and(|w| {
        w.debian_packages_draft.is_some()
            || w.debian_packages_learning.is_some()
            || w.debian_packages_terminal.is_some()
    });
    let State::DebianPackages { simulation } = &a.state else {
        return if metadata {
            Err("Package metadata on another activity; original retained".into())
        } else {
            Ok(())
        };
    };
    if !crate::debian_packages_workspace::admitted(simulation) {
        return Err("Saved package state exceeds app admission".into());
    }
    if let Some(w) = &a.workspace
        && (w
            .debian_packages_draft
            .as_ref()
            .is_some_and(|d| !d.valid_for(simulation))
            || w.debian_packages_terminal
                .as_ref()
                .is_some_and(|t| !t.valid_for(simulation))
            || w.debian_packages_learning.as_ref().is_some_and(|l| {
                !l.valid()
                    || !crate::debian_packages_teaching::separate_contexts(&l.context, simulation)
            }))
    {
        return Err("Invalid package draft/practice context; original retained".into());
    }
    Ok(())
}
fn validate_dhcpv4_workspace(a: &Attempt) -> Result<(), String> {
    let metadata = a
        .workspace
        .as_ref()
        .is_some_and(|w| w.dhcpv4_draft.is_some() || w.dhcpv4_learning.is_some());
    let State::DhcpV4 { simulation } = &a.state else {
        return if metadata {
            Err("DHCP editing metadata on another activity; original retained".into())
        } else {
            Ok(())
        };
    };
    simulation
        .validate()
        .map_err(|_| "Invalid saved DHCP state; original retained".to_owned())?;
    if let Some(w) = &a.workspace
        && (w
            .dhcpv4_draft
            .as_ref()
            .is_some_and(|d| !d.same_shape(simulation))
            || w.dhcpv4_learning.as_ref().is_some_and(|l| {
                !l.valid()
                    || l.context.topology().server == simulation.topology().server
                    || l.context
                        .topology()
                        .clients
                        .keys()
                        .any(|k| simulation.topology().clients.contains_key(k))
                    || l.context
                        .topology()
                        .segments
                        .keys()
                        .any(|k| simulation.topology().segments.contains_key(k))
            }))
    {
        return Err("Invalid DHCP editing/practice immutable context; original retained".into());
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
        crate::partition_fs_workspace::binding::metadata(a)?;
        crate::hsrp_workspace::metadata(a)?;
        crate::syslog_workspace::metadata(a)?;
        crate::ntp_workspace::metadata(a)?;
        crate::lvm_workspace::metadata(a)?;
        crate::physical_link_workspace::metadata(a)?;
        crate::pack::validate_state(&a.state)?;
        crate::one_off_workspace::metadata(a)?;
        validate_mac_workspace(a)?;
        validate_acl_workspace(a)?;
        validate_nfs_workspace(a)?;
        validate_rpm_workspace(a)?;
        validate_nat_workspace(a)?;
        validate_apache_static_workspace(a)?;
        validate_debian_packages_workspace(a)?;
        validate_dhcpv4_workspace(a)?;
        validate_accounts_workspace(a)?;
        validate_legacy_teaching_workspace(a)?;
        validate_ospf_workspace(a)?;
        validate_spanning_tree_workspace(a)?;
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
    if p.attempts.values().any(|a|matches!(a.state,State::KernelModules{..})){return Err("Module progress needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::RecursiveTree{..})){return Err("Recursive file progress needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Transport{..})){return Err("Transport progress needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(crate::command_location_workspace::has_metadata){
        return Err("Command-location work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts.values().any(crate::accounts_template_workspace::has_metadata){
        return Err("Template work needs its authored catalog; use save_with_catalog. Original retained".into());
    }

    if p.attempts.values().any(|a| {
        matches!(&a.state, State::Processes { table } if !table.listeners.is_empty())
            || a.workspace.as_ref().and_then(|w| w.process_learning.as_ref())
                .is_some_and(|learning| !learning.table.listeners.is_empty())
    }) {
        return Err("Listener work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts.values().any(|a|matches!(a.state,State::FileIndex{..})){return Err("Private-index work needs its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SharedLibraries{..})){return Err("Library work needs its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Raid1{..})){return Err("Mirror work needs its authored catalog; use save_with_catalog. Original retained".into());}

    if p.attempts.values().any(|a| matches!(a.state, State::StartupFiles { .. })) {
        return Err("Integrity work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts.values().any(|a| matches!(a.state, State::FsIntegrity { .. })) {
        return Err("Integrity work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::UserMounts { .. }))
    {
        return Err("Caller-aware mount work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::Environment { .. }))
    {
        return Err("Environment work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::DebianSources { .. }))
    {
        return Err("Source-selection work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::ClientNames { .. }))
    {
        return Err("Name-source work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::RemovableMounts { .. }))
    {
        return Err("Mount-policy work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::NmProfiles { .. }))
    {
        return Err("Saved-profile work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::PamStack { .. }))
    {
        return Err("Authentication-stack work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::Aging { .. }))
    {
        return Err("Aging-record work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::PortSecurity { .. }))
    {
        return Err("Secure-port work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::DhcpSnooping { .. }))
    {
        return Err("DHCP observer work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::ArpInspection { .. }))
    {
        return Err("ARP inspection work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::ManagementPolicy { .. }))
    {
        return Err("Management policy work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::LocalAccess { .. }))
    {
        return Err("Local account access work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::AaaAccess { .. }))
    {
        return Err("AAA access work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::LdapClient { .. }))
    {
        return Err("Directory client work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::LdapAccess { .. }))
    {
        return Err("ACL work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::LdapPassword { .. }))
    {
        return Err("Password work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::LdapChanges { .. }))
    {
        return Err("Directory-change work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::DhcpConfig { .. }))
    {
        return Err("DHCP configuration work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::Qos { .. }))
    {
        return Err("Traffic policy work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::LinuxLogging { .. }))
    {
        return Err("Linux logging work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts.values().any(|a|matches!(a.state,State::VirtualIsolation{..})){return Err("Represented-network progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SieveFiltering{..})){return Err("Mail-filter progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LinuxRouter{..})){return Err("Router progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LinuxV6{..})){return Err("Flow progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LinuxServiceRouter{..})){return Err("Flow progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LinuxV6Administration{..})){return Err("IPv6 configuration progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LinuxV6Startup{..})){return Err("IPv6 firewall startup progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::IdsRule{..})){return Err("Offline rule progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::AuthFilter{..})){return Err("Offline filter progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::VpnPoint{..})){return Err("VPN peer progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Poe{..})){return Err("Power-plan progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::KernelBuild{..})){return Err("Kernel-build progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Dkms{..})){return Err("DKMS progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Initramfs{..})){return Err("Early-userspace progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Pxe{..})){return Err("network boot progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Iscsi{..})){return Err("iSCSI progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::AtaSmart{..})){return Err("ATA SMART progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ApacheHttps{..})){return Err("Apache HTTPS progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Desktop{..})){return Err("virtual desktop progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SecurityInvestigation{..})){return Err("virtual security investigation progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::AutomationWorkshop{..})){return Err("virtual staged network changes progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ServiceExposure{..})){return Err("Service exposure progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::FtpIdentity{..})){return Err("FTP identity progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LinuxFlow{..})){return Err("Flow progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::MailboxAccess{..}|State::MailboxAdministration{..})){return Err("Mail-access progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SmtpRouting{..})){return Err("Mail-routing progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::GuestDrivers{..})){return Err("Guest-image progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SambaShare{..})){return Err("Sharing progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::CampusArchitecture{..})){return Err("Campus-layout progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::NetworkTransfer{..})){return Err("File-transfer progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::EditorHandover{..})){return Err("Editor-handover progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ModalEditing{..})){return Err("Modal progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SshAccess{..})){return Err("SSH progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::HostNetwork { .. }))
    {
        return Err("Host-network work needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::Repositories { .. }))
    {
        return Err("Repository evidence progress needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts.values().any(|a|matches!(a.state,State::MaintenanceNotice{..})){return Err("Notice progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::DnsAccess{..})){return Err("DNS-policy progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::PrintQueues{..})){return Err("Print-queue progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Localization{..})){return Err("Locale/encoding progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::WirelessChannels{..})){return Err("Channel-plan progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::WlanProfiles{..})){return Err("WLAN profile progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::HttpCache{..})){return Err("Cache progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ServiceClients{..})){return Err("Service-client progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ReverseProxy{..})){return Err("Routing progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ProxyPolicy{..})){return Err("Proxy progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SnmpMonitor{..})){return Err("Monitoring progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::WlanPaths{..})){return Err("AP-path progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Timezone{..})){return Err("Time-zone progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::SourceInstall{..})){return Err("Source-install progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::CapacityPlanning{..})){return Err("Capacity-planning progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::ResourceObservations{..})){return Err("Observation progress requires its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::Capacity { .. }))
    {
        return Err("Capacity progress needs its authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::PartitionFs { .. }))
    {
        return Err("Disk preparation progress needs its held authored catalog; use save_with_catalog. Original retained".into());
    }
    if p.attempts
        .values()
        .any(|a| matches!(a.state, State::Syslog { .. }))
    {
        return Err(
            "Syslog progress needs its authored catalog; use save_with_catalog. Original retained"
                .into(),
        );
    }
    if p.attempts.values().any(|a|matches!(a.state,State::RaidAssembly{..})){return Err("RAID assembly progress needs its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::CopiedJournal{..})){return Err("Copied-journal work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::JsonInventory{..})){return Err("JSON-inventory work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::RestController{..})){return Err("REST-controller work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::DevicePath{..})){return Err("Device-path work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::LocalMail{..})){return Err("Local-mail work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Chrony{..})){return Err("Time-source work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::GrubOnce{..})){return Err("Temporary GRUB work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::EfiLoader{..})){return Err("EFI-loader work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::BiosLoader{..})){return Err("BIOS-loader work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::BootEntry{..})){return Err("Boot-entry work needs its authored catalog; use save_with_catalog. Original retained.".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::Autofs{..})){return Err("AutoFS progress needs its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a|matches!(a.state,State::DirectoryPolicy{..})) {return Err("Directory-policy progress needs its authored catalog; use save_with_catalog. Original retained".into());}
    if p.attempts.values().any(|a| {
        matches!(
            a.state,
            State::Links { .. } | State::Lvm { .. } | State::Ntp { .. } | State::Hsrp { .. }
        )
    }) {
        return Err(
            "LVM/NTP/HSRP progress needs its authored catalog; use save_with_catalog. Original retained"
                .into(),
        );
    }
    save_with_writer(path, p, WriteMode::Normal, |_| Ok(()), write_atomic)
}
/// Source-aware normal saves for scripts, storage, time and HSRP work. The writer,
/// provenance and family behavior stays in the same existing transaction.
pub fn save_with_catalog(
    path: &Path,
    p: &Progress,
    scenarios: &[Scenario],
) -> Result<Saved, String> {
    for (id, a) in &p.attempts {
        let source = scenarios.iter().find(|s| &s.id == id);
        if crate::command_location_workspace::has_metadata(a)
            || source.is_some_and(crate::command_location_workspace::has_source){
            crate::command_location_workspace::context(a,source.ok_or("Unknown command-location source; original retained")?)?;
        }
        if crate::accounts_template_workspace::has_metadata(a)
            || source.is_some_and(crate::accounts_template_workspace::has_source){
            crate::accounts_template_workspace::context(a,source.ok_or("Unknown authored template source; original retained")?)?;
        }

        // Old process tables retain their existing save path. Endpoint-bearing
        // work needs its immutable authored roster before any bytes are replaced,
        // including a forged attempt which erased every current endpoint.
        let listener_source = source.is_some_and(|s| {
            matches!(&s.initial, State::Processes { table } if !table.listeners.is_empty())
                || s.process_teaching.as_ref().is_some_and(|l| !l.fixture.listeners.is_empty())
        });
        let listener_attempt = matches!(&a.state, State::Processes { table } if !table.listeners.is_empty())
            || a.workspace.as_ref().and_then(|w| w.process_learning.as_ref())
                .is_some_and(|l| !l.table.listeners.is_empty());
        if listener_source || listener_attempt {
            let s = source.ok_or("Unknown authored listener source; original retained")?;
            let (State::Processes { table }, State::Processes { table: initial }, crate::engine::Goal::Processes { .. }) =
                (&a.state, &s.initial, &s.goal) else {
                    return Err("Listener work belongs to another activity; original retained".into());
                };
            if a.scenario_version != s.version || !table.same_problem(initial) {
                return Err("Listener process identities or endpoint roster differ; original retained".into());
            }
            if a.workspace.as_ref().and_then(|w| w.process_learning.as_ref()).is_some_and(|l| {
                s.process_teaching.as_ref().is_none_or(|lesson| {
                    !l.table.same_problem(&lesson.fixture) || l.example_command >= lesson.example.len()
                })
            }) {
                return Err("Listener practice differs from its authored example; original retained".into());
            }
        }
        if matches!(a.state,State::KernelModules{..}) || source.is_some_and(|s|matches!(s.initial,State::KernelModules{..}) || matches!(s.goal,crate::engine::Goal::KernelModules{..})) {crate::kernel_module_source::context(a,source.ok_or("Module progress needs its authored source; original retained.")?)?;}
        if matches!(a.state,State::RecursiveTree{..}) || source.is_some_and(|s|matches!(s.initial,State::RecursiveTree{..}) || matches!(s.goal,crate::engine::Goal::RecursiveTree{..})) {
            crate::recursive_tree_source::context(a, source.ok_or("Recursive progress needs its authored source; original retained.")?)?;
        }
        if matches!(a.state,State::Transport{..}) || source.is_some_and(|s|matches!(s.initial,State::Transport{..}) || matches!(s.goal,crate::engine::Goal::Transport{..})) {
            crate::transport_workspace::context(a,source.ok_or("Unknown authored transport source; original retained")?)?;
        }
        if matches!(a.state,State::RaidAssembly{..}) || source.is_some_and(|s|matches!(s.initial,State::RaidAssembly{..}) || matches!(s.goal,crate::engine::Goal::RaidAssembly{..})) {crate::raid_assembly_source::context(a,source.ok_or("Unknown authored RAID assembly source; original retained")?)?;}
        if matches!(a.state,State::CopiedJournal{..}) || source.is_some_and(|s|matches!(s.initial,State::CopiedJournal{..}) || matches!(s.goal,crate::engine::Goal::CopiedJournal{..})) {crate::journal_archive_workspace::context(a,source.ok_or("Unknown Copied-journal source; original retained.")?)?;}
        if matches!(a.state,State::JsonInventory{..}) || source.is_some_and(|s|matches!(s.initial,State::JsonInventory{..}) || matches!(s.goal,crate::engine::Goal::JsonInventory{..})) {crate::json_inventory_workspace::context(a,source.ok_or("Unknown JSON-inventory source; original retained.")?)?;}
        if matches!(a.state,State::RestController{..}) || source.is_some_and(|s|matches!(s.initial,State::RestController{..}) || matches!(s.goal,crate::engine::Goal::RestController{..})) {crate::rest_workspace::context(a,source.ok_or("Unknown REST-controller source; original retained.")?)?;}
        if matches!(a.state,State::DevicePath{..}) || source.is_some_and(|s|matches!(s.initial,State::DevicePath{..}) || matches!(s.goal,crate::engine::Goal::DevicePath{..})) {crate::device_path_workspace::context(a,source.ok_or("Unknown Device-path source; original retained.")?)?;}
        if matches!(a.state,State::LocalMail{..}) || source.is_some_and(|s|matches!(s.initial,State::LocalMail{..}) || matches!(s.goal,crate::engine::Goal::LocalMail{..})) {crate::local_mail_workspace::context(a,source.ok_or("Unknown Local-mail source; original retained.")?)?;}
        if matches!(a.state,State::Chrony{..}) || source.is_some_and(|s|matches!(s.initial,State::Chrony{..}) || matches!(s.goal,crate::engine::Goal::Chrony{..})) {crate::chrony_workspace::context(a,source.ok_or("Unknown Time-source source; original retained.")?)?;}
        if matches!(a.state,State::GrubOnce{..}) || source.is_some_and(|s|matches!(s.initial,State::GrubOnce{..}) || matches!(s.goal,crate::engine::Goal::GrubOnce{..})) {crate::grub_once_workspace::context(a,source.ok_or("Unknown Temporary GRUB source; original retained.")?)?;}
        if matches!(a.state,State::EfiLoader{..}) || source.is_some_and(|s|matches!(s.initial,State::EfiLoader{..}) || matches!(s.goal,crate::engine::Goal::EfiLoader{..})) {crate::efi_loader_workspace::context(a,source.ok_or("Unknown EFI-loader source; original retained.")?)?;}
        if matches!(a.state,State::BiosLoader{..}) || source.is_some_and(|s|matches!(s.initial,State::BiosLoader{..}) || matches!(s.goal,crate::engine::Goal::BiosLoader{..})) {crate::bios_loader_workspace::context(a,source.ok_or("Unknown BIOS-loader source; original retained.")?)?;}
        if matches!(a.state,State::BootEntry{..}) || source.is_some_and(|s|matches!(s.initial,State::BootEntry{..}) || matches!(s.goal,crate::engine::Goal::BootEntry{..})) {crate::boot_entry_workspace::context(a,source.ok_or("Unknown boot-entry source; original retained.")?)?;}
        if matches!(a.state,State::Autofs{..}) || source.is_some_and(|s|matches!(s.initial,State::Autofs{..}) || matches!(s.goal,crate::engine::Goal::Autofs{..})) {crate::autofs_source::context(a,source.ok_or("Unknown authored AutoFS source; original retained")?)?;}
        if matches!(a.state, State::DirectoryPolicy { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::DirectoryPolicy { .. })
                    || matches!(s.goal, crate::engine::Goal::DirectoryPolicy { .. })
            })
        {
            crate::directory_policy_source::context(
                a,
                source.ok_or("Unknown authored directory-policy source; original retained")?,
            )?;
        }
        if matches!(a.state, State::Links { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::Links { .. })
                    || matches!(s.goal, crate::engine::Goal::Links { .. })
            })
        {
            crate::links_workspace::context(
                a,
                source.ok_or("Unknown authored link source; original retained")?,
            )?;
        }
        if matches!(a.state,State::DnsAccess{..})||source.is_some_and(|s|matches!(s.initial,State::DnsAccess{..})||matches!(s.goal,crate::engine::Goal::DnsAccess{..})){
          crate::dns_access_workspace::context(a,source.ok_or("Unknown authored DNS-policy task; original retained")?)?;
        }
        if matches!(a.state,State::PrintQueues{..})||source.is_some_and(|s|matches!(s.initial,State::PrintQueues{..})||matches!(s.goal,crate::engine::Goal::PrintQueues{..})){
          crate::print_queue_workspace::context(a,source.ok_or("Unknown authored print-queue task; original retained")?)?;
        }
        if matches!(a.state,State::Localization{..})||source.is_some_and(|s|matches!(s.initial,State::Localization{..})||matches!(s.goal,crate::engine::Goal::Localization{..})){
          crate::localization_workspace::context(a,source.ok_or("Unknown authored locale/encoding task; original retained")?)?;
        }
        if matches!(a.state,State::WirelessChannels{..})||source.is_some_and(|s|matches!(s.initial,State::WirelessChannels{..})||matches!(s.goal,crate::engine::Goal::WirelessChannels{..})){crate::wireless_channel_workspace::context(a,source.ok_or("Unknown authored channel-plan task; original retained")?)?;}
        if matches!(a.state,State::WlanProfiles{..})||source.is_some_and(|s|matches!(s.initial,State::WlanProfiles{..})||matches!(s.goal,crate::engine::Goal::WlanProfiles{..})){crate::wlan_workspace::context(a,source.ok_or("Unknown authored WLAN task; original retained")?)?;}
        if matches!(a.state,State::HttpCache{..})||source.is_some_and(|s|matches!(s.initial,State::HttpCache{..})||matches!(s.goal,crate::engine::Goal::HttpCache{..})){crate::http_cache_workspace::context(a,source.ok_or("Unknown authored cache task; original retained")?)?;}
        if matches!(a.state,State::ServiceClients{..})||source.is_some_and(|s|matches!(s.initial,State::ServiceClients{..})||matches!(s.goal,crate::engine::Goal::ServiceClients{..})){crate::service_workspace::context(a,source.ok_or("Unknown authored service-client task; original retained")?)?;}
        if matches!(a.state,State::ReverseProxy{..})||source.is_some_and(|s|matches!(s.initial,State::ReverseProxy{..})||matches!(s.goal,crate::engine::Goal::ReverseProxy{..})){crate::reverse_proxy_workspace::context(a,source.ok_or("Unknown authored routing task; original retained")?)?;}
        if matches!(a.state,State::ProxyPolicy{..})||source.is_some_and(|s|matches!(s.initial,State::ProxyPolicy{..})||matches!(s.goal,crate::engine::Goal::ProxyPolicy{..})){crate::proxy_workspace::context(a,source.ok_or("Unknown authored proxy task; original retained")?)?;}
        if matches!(a.state,State::SnmpMonitor{..})||source.is_some_and(|s|matches!(s.initial,State::SnmpMonitor{..})||matches!(s.goal,crate::engine::Goal::SnmpMonitor{..})){crate::snmp_workspace::context(a,source.ok_or("Unknown authored monitoring task; original retained")?)?;}
        if matches!(a.state,State::WlanPaths{..})||source.is_some_and(|s|matches!(s.initial,State::WlanPaths{..})||matches!(s.goal,crate::engine::Goal::WlanPaths{..})){crate::wlan_paths_workspace::context(a,source.ok_or("Unknown authored AP-path task; original retained")?)?;}
        if matches!(a.state,State::Timezone{..})||source.is_some_and(|s|matches!(s.initial,State::Timezone{..})||matches!(s.goal,crate::engine::Goal::Timezone{..})){
          crate::timezone_workspace::context(a,source.ok_or("Unknown authored time-zone task; original retained")?)?;
        }
        if matches!(a.state,State::SourceInstall{..})||source.is_some_and(|s|matches!(s.initial,State::SourceInstall{..})||matches!(s.goal,crate::engine::Goal::SourceInstall{..})){
          crate::source_install_workspace::context(a,source.ok_or("Unknown authored source-install task; original retained")?)?;
        }
        if matches!(a.state,State::MaintenanceNotice{..})||source.is_some_and(|s|matches!(s.initial,State::MaintenanceNotice{..})||matches!(s.goal,crate::engine::Goal::MaintenanceNotice{..})){
          crate::maintenance_notice_workspace::context(a,source.ok_or("Unknown authored notice task; original retained")?)?;
        }
        if matches!(a.state,State::CapacityPlanning{..})||source.is_some_and(|s|matches!(s.initial,State::CapacityPlanning{..})||matches!(s.goal,crate::engine::Goal::CapacityPlanning{..})){
          crate::capacity_planning_workspace::context(a,source.ok_or("Unknown authored plan; original retained")?)?;
        }
        if matches!(a.state,State::ResourceObservations{..})||source.is_some_and(|s|matches!(s.initial,State::ResourceObservations{..})||matches!(s.goal,crate::engine::Goal::ResourceObservations{..})){
          crate::resource_observation_workspace::context(a,source.ok_or("Unknown authored investigation; original retained")?)?;
        }
        if matches!(a.state, State::Capacity { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::Capacity { .. })
                    || matches!(s.goal, crate::engine::Goal::Capacity { .. })
            })
        {
            crate::capacity_workspace::context(
                a,
                source.ok_or("Unknown authored capacity source; original retained")?,
            )?;
        }
        if matches!(a.state, State::Repositories { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::Repositories { .. })
                    || matches!(s.goal, crate::engine::Goal::Repositories { .. })
            })
        {
            crate::repository_workspace::context(
                a,
                source.ok_or("Unknown authored repository source; original retained")?,
            )?;
        }
        if matches!(a.state, State::StartupFiles { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::StartupFiles { .. })
                    || matches!(s.goal, crate::engine::Goal::StartupFiles { .. })
            })
        {
            crate::startup_workspace::context(
                a,
                source.ok_or("Unknown authored Bash startup source; original retained")?,
            )?;
        }
        if matches!(a.state,State::FileIndex{..})||source.is_some_and(|s|matches!(s.initial,State::FileIndex{..})||matches!(s.goal,crate::engine::Goal::FileIndex{..})){crate::file_index_source::context(a,source.ok_or("Unknown private-index source; original retained")?)?;}
        if matches!(a.state,State::SharedLibraries{..})||source.is_some_and(|s|matches!(s.initial,State::SharedLibraries{..})||matches!(s.goal,crate::engine::Goal::SharedLibraries{..})){
            crate::shared_libraries_workspace::context(a,source.ok_or("Unknown authored library source; original retained")?)?;
        }
        if matches!(a.state,State::Raid1{..})||source.is_some_and(|s|matches!(s.initial,State::Raid1{..})||matches!(s.goal,crate::engine::Goal::Raid1{..})){
            crate::raid1_workspace::context(a,source.ok_or("Unknown authored mirror source; original retained")?)?;
        }
        if matches!(a.state, State::FsIntegrity { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::FsIntegrity { .. })
                    || matches!(s.goal, crate::engine::Goal::FsIntegrity { .. })
            })
        {
            crate::fs_integrity_workspace::context(
                a,
                source.ok_or("Unknown authored filesystem integrity source; original retained")?,
            )?;
        }
        if matches!(a.state, State::UserMounts { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::UserMounts { .. })
                    || matches!(s.goal, crate::engine::Goal::UserMounts { .. })
            })
        {
            crate::user_mounts_workspace::context(
                a,
                source.ok_or("Unknown authored caller-aware mount source; original retained")?,
            )?;
        }
        if matches!(a.state, State::Environment { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::Environment { .. })
                    || matches!(s.goal, crate::engine::Goal::Environment { .. })
            })
        {
            crate::environment_workspace::context(
                a,
                source.ok_or("Unknown authored environment source; original retained")?,
            )?;
        }
        if matches!(a.state, State::DebianSources { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::DebianSources { .. })
                    || matches!(s.goal, crate::engine::Goal::DebianSources { .. })
            })
        {
            crate::debian_sources_workspace::context(
                a,
                source.ok_or("Unknown authored source-selection source; original retained")?,
            )?;
        }
        if matches!(a.state, State::ClientNames { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::ClientNames { .. })
                    || matches!(s.goal, crate::engine::Goal::ClientNames { .. })
            })
        {
            crate::client_names_workspace::context(
                a,
                source.ok_or("Unknown authored name-source source; original retained")?,
            )?;
        }
        if matches!(a.state, State::RemovableMounts { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::RemovableMounts { .. })
                    || matches!(s.goal, crate::engine::Goal::RemovableMounts { .. })
            })
        {
            crate::removable_workspace::context(
                a,
                source.ok_or("Unknown authored mount-policy source; original retained")?,
            )?;
        }
        if matches!(a.state, State::NmProfiles { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::NmProfiles { .. })
                    || matches!(s.goal, crate::engine::Goal::NmProfiles { .. })
            })
        {
            crate::nm_workspace::context(
                a,
                source.ok_or("Unknown authored saved-profile source; original retained")?,
            )?;
        }
        if matches!(a.state, State::PamStack { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::PamStack { .. })
                    || matches!(s.goal, crate::engine::Goal::PamStack { .. })
            })
        {
            crate::pam_workspace::context(
                a,
                source.ok_or("Unknown authored authentication-stack source; original retained")?,
            )?;
        }
        if matches!(a.state, State::Aging { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::Aging { .. })
                    || matches!(s.goal, crate::engine::Goal::Aging { .. })
            })
        {
            crate::aging_workspace::context(
                a,
                source.ok_or("Unknown authored aging-record source; original retained")?,
            )?;
        }
        if matches!(a.state, State::PortSecurity { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::PortSecurity { .. })
                    || matches!(s.goal, crate::engine::Goal::PortSecurity { .. })
            })
        {
            crate::port_security_workspace::context(
                a,
                source.ok_or("Unknown authored secure-port source; original retained")?,
            )?;
        }
        if matches!(a.state, State::DhcpSnooping { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::DhcpSnooping { .. })
                    || matches!(s.goal, crate::engine::Goal::DhcpSnooping { .. })
            })
        {
            crate::dhcp_snooping_workspace::context(
                a,
                source.ok_or("Unknown authored DHCP observer source; original retained")?,
            )?;
        }
        if matches!(a.state, State::ArpInspection { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::ArpInspection { .. })
                    || matches!(s.goal, crate::engine::Goal::ArpInspection { .. })
            })
        {
            crate::arp_inspection_workspace::context(
                a,
                source.ok_or("Unknown authored ARP inspection source; original retained")?,
            )?;
        }
        if matches!(a.state, State::ManagementPolicy { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::ManagementPolicy { .. })
                    || matches!(s.goal, crate::engine::Goal::ManagementPolicy { .. })
            })
        {
            crate::management_workspace::context(
                a,
                source.ok_or("Unknown authored Management policy source; original retained")?,
            )?;
        }
        if matches!(a.state, State::LocalAccess { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::LocalAccess { .. })
                    || matches!(s.goal, crate::engine::Goal::LocalAccess { .. })
            })
        {
            crate::local_access_workspace::context(
                a,
                source.ok_or("Unknown authored Local account access source; original retained")?,
            )?;
        }
        if matches!(a.state, State::AaaAccess { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::AaaAccess { .. })
                    || matches!(s.goal, crate::engine::Goal::AaaAccess { .. })
            })
        {
            crate::aaa_workspace::context(
                a,
                source.ok_or("Unknown authored AAA access source; original retained")?,
            )?;
        }
        if matches!(a.state, State::LdapClient { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::LdapClient { .. })
                    || matches!(s.goal, crate::engine::Goal::LdapClient { .. })
            })
        {
            crate::ldap_workspace::context(
                a,
                source.ok_or("Unknown authored Directory client source; original retained")?,
            )?;
        }
        if matches!(a.state, State::LdapAccess { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::LdapAccess { .. })
                    || matches!(s.goal, crate::engine::Goal::LdapAccess { .. })
            })
        {
            crate::ldap_access_workspace::context(
                a,
                source.ok_or("Unknown authored ACL source; original retained")?,
            )?;
        }
        if matches!(a.state, State::LdapPassword { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::LdapPassword { .. })
                    || matches!(s.goal, crate::engine::Goal::LdapPassword { .. })
            })
        {
            crate::ldap_password_workspace::context(
                a,
                source.ok_or("Unknown authored Password source; original retained")?,
            )?;
        }
        if matches!(a.state, State::LdapChanges { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::LdapChanges { .. })
                    || matches!(s.goal, crate::engine::Goal::LdapChanges { .. })
            })
        {
            crate::ldap_change_workspace::context(
                a,
                source.ok_or("Unknown authored Directory-change source; original retained")?,
            )?;
        }
        if matches!(a.state, State::DhcpConfig { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::DhcpConfig { .. })
                    || matches!(s.goal, crate::engine::Goal::DhcpConfig { .. })
            })
        {
            crate::dhcp_server_workspace::context(
                a,
                source.ok_or("Unknown authored DHCP configuration source; original retained")?,
            )?;
        }
        if matches!(a.state, State::Qos { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::Qos { .. })
                    || matches!(s.goal, crate::engine::Goal::Qos { .. })
            })
        {
            crate::qos_workspace::context(
                a,
                source.ok_or("Unknown authored traffic-policy source; original retained")?,
            )?;
        }
        if matches!(a.state, State::LinuxLogging { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::LinuxLogging { .. })
                    || matches!(s.goal, crate::engine::Goal::LinuxLogging { .. })
            })
        {
            crate::linux_logging_workspace::context(
                a,
                source.ok_or("Unknown authored logging source; original retained")?,
            )?;
        }
        if matches!(a.state, State::HostNetwork { .. })
            || source.is_some_and(|s| {
                matches!(s.initial, State::HostNetwork { .. })
                    || matches!(s.goal, crate::engine::Goal::HostNetwork { .. })
            })
        {
            crate::host_workspace::context(
                a,
                source.ok_or("Unknown authored host source; original retained")?,
            )?;
        }
        if matches!(a.state,State::VirtualIsolation{..})||source.is_some_and(|s|matches!(s.initial,State::VirtualIsolation{..})||matches!(s.goal,crate::engine::Goal::VirtualIsolation{..})){crate::virtual_isolation_source::context(a,source.ok_or("Unknown authored represented-network source; original retained")?)?;}
        if matches!(a.state,State::SieveFiltering{..})||source.is_some_and(|s|matches!(s.initial,State::SieveFiltering{..})||matches!(s.goal,crate::engine::Goal::SieveFiltering{..})){crate::sieve_source::context(a,source.ok_or("Unknown authored mail-filter source; original retained")?)?;}
        if matches!(a.state,State::LinuxRouter{..})||source.is_some_and(|s|matches!(s.initial,State::LinuxRouter{..})||matches!(s.goal,crate::engine::Goal::LinuxRouter{..})){crate::linux_router_source::context(a,source.ok_or("Unknown authored router source; original retained")?)?;}
        if matches!(a.state,State::LinuxV6{..})||source.is_some_and(|s|matches!(s.initial,State::LinuxV6{..})||matches!(s.goal,crate::engine::Goal::LinuxV6{..})){crate::linux_v6_source::context(a,source.ok_or("Unknown authored flow source; original retained")?)?;}
        if matches!(a.state,State::LinuxServiceRouter{..})||source.is_some_and(|s|matches!(s.initial,State::LinuxServiceRouter{..})||matches!(s.goal,crate::engine::Goal::LinuxServiceRouter{..})){crate::linux_service_router_source::context(a,source.ok_or("Unknown authored flow source; original retained")?)?;}
        if matches!(a.state,State::LinuxV6Administration{..})||source.is_some_and(|s|matches!(s.initial,State::LinuxV6Administration{..})||matches!(s.goal,crate::engine::Goal::LinuxV6Administration{..})){crate::linux_v6_admin_source::context(a,source.ok_or("Unknown authored IPv6 configuration source; original retained")?)?;}
        if matches!(a.state,State::LinuxV6Startup{..})||source.is_some_and(|s|matches!(s.initial,State::LinuxV6Startup{..})||matches!(s.goal,crate::engine::Goal::LinuxV6Startup{..})){crate::linux_v6_startup_source::context(a,source.ok_or("Unknown authored IPv6 firewall startup source; original retained")?)?;}
        if matches!(a.state,State::IdsRule{..})||source.is_some_and(|s|matches!(s.initial,State::IdsRule{..})||matches!(s.goal,crate::engine::Goal::IdsRule{..})){crate::ids_rule_source::context(a,source.ok_or("Unknown authored IDS-rule source; original retained")?)?;}
        if matches!(a.state,State::AuthFilter{..})||source.is_some_and(|s|matches!(s.initial,State::AuthFilter{..})||matches!(s.goal,crate::engine::Goal::AuthFilter{..})){crate::auth_filter_source::context(a,source.ok_or("Unknown authored authentication-filter source; original retained")?)?;}
        if matches!(a.state,State::VpnPoint{..})||source.is_some_and(|s|matches!(s.initial,State::VpnPoint{..})||matches!(s.goal,crate::engine::Goal::VpnPoint{..})){crate::vpn_point_source::context(a,source.ok_or("Unknown authored VPN peer source; original retained")?)?;}
        if matches!(a.state,State::Poe{..})||source.is_some_and(|s|matches!(s.initial,State::Poe{..})||matches!(s.goal,crate::engine::Goal::Poe{..})){crate::poe_source::context(a,source.ok_or("Unknown authored Power-plan source; original retained")?)?;}
        if matches!(a.state,State::KernelBuild{..})||source.is_some_and(|s|matches!(s.initial,State::KernelBuild{..})||matches!(s.goal,crate::engine::Goal::KernelBuild{..})){crate::kernel_source::context(a,source.ok_or("Unknown authored Kernel-build source; original retained")?)?;}
        if matches!(a.state,State::Dkms{..})||source.is_some_and(|s|matches!(s.initial,State::Dkms{..})||matches!(s.goal,crate::engine::Goal::Dkms{..})){crate::dkms_source::context(a,source.ok_or("Unknown authored DKMS source; original retained")?)?;}
        if matches!(a.state,State::Initramfs{..})||source.is_some_and(|s|matches!(s.initial,State::Initramfs{..})||matches!(s.goal,crate::engine::Goal::Initramfs{..})){crate::initramfs_source::context(a,source.ok_or("Unknown authored Early-userspace source; original retained")?)?;}
        if matches!(a.state,State::Pxe{..})||source.is_some_and(|s|matches!(s.initial,State::Pxe{..})||matches!(s.goal,crate::engine::Goal::Pxe{..})){crate::pxe::source::context(a,source.ok_or("Unknown authored network boot source; original retained")?)?;}
        if matches!(a.state,State::Iscsi{..})||source.is_some_and(|s|matches!(s.initial,State::Iscsi{..})||matches!(s.goal,crate::engine::Goal::Iscsi{..})){crate::iscsi::source::context(a,source.ok_or("Unknown authored iSCSI source; original retained")?)?;}
        if matches!(a.state,State::AtaSmart{..})||source.is_some_and(|s|matches!(s.initial,State::AtaSmart{..})||matches!(s.goal,crate::engine::Goal::AtaSmart{..})){crate::ata_smart::source::context(a,source.ok_or("Unknown authored ATA SMART source; original retained")?)?;}
        if matches!(a.state,State::ApacheHttps{..})||source.is_some_and(|s|matches!(s.initial,State::ApacheHttps{..})||matches!(s.goal,crate::engine::Goal::ApacheHttps{..})){crate::apache_https::source::context(a,source.ok_or("Unknown authored Apache HTTPS source; original retained")?)?;}
        if matches!(a.state,State::Desktop{..})||source.is_some_and(|s|matches!(s.initial,State::Desktop{..})||matches!(s.goal,crate::engine::Goal::Desktop{..})){crate::desktop::source::context(a,source.ok_or("Unknown authored virtual desktop source; original retained")?)?;}
        if matches!(a.state,State::SecurityInvestigation{..})||source.is_some_and(|s|matches!(s.initial,State::SecurityInvestigation{..})||matches!(s.goal,crate::engine::Goal::SecurityInvestigation{..})){crate::security_investigation::source::context(a,source.ok_or("Unknown authored virtual security investigation source; original retained")?)?;}
        if matches!(a.state,State::AutomationWorkshop{..})||source.is_some_and(|s|matches!(s.initial,State::AutomationWorkshop{..})||matches!(s.goal,crate::engine::Goal::AutomationWorkshop{..})){crate::automation_workshop::source::context(a,source.ok_or("Unknown authored virtual staged network changes source; original retained")?)?;}
        if matches!(a.state,State::ServiceExposure{..})||source.is_some_and(|s|matches!(s.initial,State::ServiceExposure{..})||matches!(s.goal,crate::engine::Goal::ServiceExposure{..})){crate::service_exposure_source::context(a,source.ok_or("Unknown authored Service exposure source; original retained")?)?;}
        if matches!(a.state,State::FtpIdentity{..})||source.is_some_and(|s|matches!(s.initial,State::FtpIdentity{..})||matches!(s.goal,crate::engine::Goal::FtpIdentity{..})){crate::ftp_identity_source::context(a,source.ok_or("Unknown authored FTP identity source; original retained")?)?;}
        if matches!(a.state,State::LinuxFlow{..})||source.is_some_and(|s|matches!(s.initial,State::LinuxFlow{..})||matches!(s.goal,crate::engine::Goal::LinuxFlow{..})){crate::linux_flow_source::context(a,source.ok_or("Unknown authored flow source; original retained")?)?;}
        if matches!(a.state,State::MailboxAccess{..}|State::MailboxAdministration{..})||source.is_some_and(|s|matches!(s.initial,State::MailboxAccess{..}|State::MailboxAdministration{..})||matches!(s.goal,crate::engine::Goal::MailboxAccess{..}|crate::engine::Goal::MailboxAdministration{..})){crate::mailbox_source::context(a,source.ok_or("Unknown authored mail-access source; original retained")?)?;}
        if matches!(a.state,State::SmtpRouting{..})||source.is_some_and(|s|matches!(s.initial,State::SmtpRouting{..})||matches!(s.goal,crate::engine::Goal::SmtpRouting{..})){crate::smtp_source::context(a,source.ok_or("Unknown authored mail-routing source; original retained")?)?;}
        if matches!(a.state,State::GuestDrivers{..})||source.is_some_and(|s|matches!(s.initial,State::GuestDrivers{..})||matches!(s.goal,crate::engine::Goal::GuestDrivers{..})){crate::guest_driver_source::context(a,source.ok_or("Unknown authored guest-image source; original retained")?)?;}
        if matches!(a.state,State::SambaShare{..})||source.is_some_and(|s|matches!(s.initial,State::SambaShare{..})||matches!(s.goal,crate::engine::Goal::SambaShare{..})){crate::samba_share_source::context(a,source.ok_or("Unknown authored sharing source; original retained")?)?;}
        if matches!(a.state,State::CampusArchitecture{..})||source.is_some_and(|s|matches!(s.initial,State::CampusArchitecture{..})||matches!(s.goal,crate::engine::Goal::CampusArchitecture{..})){crate::network_architecture_source::context(a,source.ok_or("Unknown authored campus-layout source; original retained")?)?;}
        if matches!(a.state,State::NetworkTransfer{..})||source.is_some_and(|s|matches!(s.initial,State::NetworkTransfer{..})||matches!(s.goal,crate::engine::Goal::NetworkTransfer{..})){crate::network_transfer_source::context(a,source.ok_or("Unknown authored file-transfer source; original retained")?)?;}
        if matches!(a.state,State::EditorHandover{..})||source.is_some_and(|s|matches!(s.initial,State::EditorHandover{..})||matches!(s.goal,crate::engine::Goal::EditorHandover{..})){crate::editor_handover_source::context(a,source.ok_or("Unknown authored editor-handover source; original retained")?)?;}
        if matches!(a.state,State::ModalEditing{..})||source.is_some_and(|s|matches!(s.initial,State::ModalEditing{..})||matches!(s.goal,crate::engine::Goal::ModalEditing{..})){crate::modal_source::context(a,source.ok_or("Unknown authored editing source; original retained")?)?;}
        if matches!(a.state,State::SshAccess{..})||source.is_some_and(|s|matches!(s.initial,State::SshAccess{..})||matches!(s.goal,crate::engine::Goal::SshAccess{..})){crate::ssh_workspace::context(a,source.ok_or("Unknown authored SSH source; original retained")?)?;}
        let script_source = source.is_some_and(|s| {
            matches!(s.initial, State::Script { .. })
                || matches!(s.goal, crate::engine::Goal::Script { .. })
        });
        if matches!(a.state, State::Script { .. }) || script_source {
            let source = source.ok_or("Unknown authored script source; original retained")?;
            if a.scenario_version != source.version
                || !matches!(
                    (&a.state, &source.initial, &source.goal),
                    (
                        State::Script { .. },
                        State::Script { .. },
                        crate::engine::Goal::Script { .. }
                    )
                )
            {
                return Err("Script work does not match this activity; original retained".into());
            }
        }
        let disk_source = source.is_some_and(|s| {
            matches!(s.initial, State::PartitionFs { .. })
                || matches!(s.goal, crate::engine::Goal::PartitionFs { .. })
        });
        if matches!(a.state, State::PartitionFs { .. }) || disk_source {
            crate::partition_fs_workspace::binding::context(
                a,
                source.ok_or("Unknown authored disk source; original retained")?,
            )?;
        }
        let hsrp_source = source.is_some_and(|s| {
            matches!(s.initial, State::Hsrp { .. })
                || matches!(s.goal, crate::engine::Goal::Hsrp { .. })
        });
        if matches!(a.state, State::Hsrp { .. }) || hsrp_source {
            let source = source.ok_or("Unknown authored HSRP source; original retained")?;
            crate::hsrp_workspace::context(a, source)?;
        }
        let syslog_source = source.is_some_and(|s| {
            matches!(s.initial, State::Syslog { .. })
                || matches!(s.goal, crate::engine::Goal::Syslog { .. })
        });
        if matches!(a.state, State::Syslog { .. }) || syslog_source {
            let source = source.ok_or("Unknown authored Syslog source; original retained")?;
            crate::syslog_workspace::context(a, source)?;
        }
        let lvm_source = source.is_some_and(|s| {
            matches!(s.initial, State::Lvm { .. })
                || matches!(s.goal, crate::engine::Goal::Lvm { .. })
        });
        let ntp_source = source.is_some_and(|s| {
            matches!(s.initial, State::Ntp { .. })
                || matches!(s.goal, crate::engine::Goal::Ntp { .. })
        });
        if matches!(a.state, State::Ntp { .. }) || ntp_source {
            let source = source.ok_or("Unknown authored NTP source; original retained")?;
            crate::ntp_workspace::context(a, source)?;
        }
        if matches!(a.state, State::Lvm { .. }) || lvm_source {
            let source = source.ok_or("Unknown authored LVM source; original retained")?;
            crate::lvm_workspace::context(a, source)?;
        }
    }
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
mod current_revision_writer_acceptance {
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
        const {
            assert!(crate::shell::EVALUATOR_REVISION >= 8);
        }
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
                std::fs::read(path.with_extension(format!(
                    "before-shell-v{}.json",
                    crate::shell::EVALUATOR_REVISION
                )))
                .unwrap(),
                ACTUAL7
            );
            assert!(candidate.migration_receipts.is_none());
            save(&path, &candidate).unwrap();
            assert_eq!(
                std::fs::read(path.with_extension(format!(
                    "before-shell-v{}.json",
                    crate::shell::EVALUATOR_REVISION
                )))
                .unwrap(),
                ACTUAL7
            );
        }
    }
    #[test]
    fn actual7_postreplacement_sync_warning_commits_current_receipt_and_retains_exact7_snapshot() {
        const {
            assert!(crate::shell::EVALUATOR_REVISION >= 8);
        }
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
            std::fs::read(path.with_extension(format!(
                "before-shell-v{}.json",
                crate::shell::EVALUATOR_REVISION
            )))
            .unwrap(),
            ACTUAL7
        );
        assert!(candidate.migration_receipts.is_none());
        let disk: Progress = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            disk.migration_receipts.as_ref().unwrap().shell,
            vec![7, crate::shell::EVALUATOR_REVISION]
        );
        save(&path, &disk).unwrap();
        assert_eq!(
            std::fs::read(path.with_extension(format!(
                "before-shell-v{}.json",
                crate::shell::EVALUATOR_REVISION
            )))
            .unwrap(),
            ACTUAL7
        );
    }
}

#[cfg(test)]
mod genuine8_to_current_writer_acceptance {
    use super::*;
    const ACTUAL8: &[u8] =
        include_bytes!("../tests/fixtures/shell-v8-synthetic-writer-progress.json");
    fn empty() -> Progress {
        Progress {
            schema_version: 1,
            ..Default::default()
        }
    }
    #[test]
    fn genuine8_exact_copy_race_and_failed_main_persist_keep_target_and_caller_unchanged() {
        const {
            assert!(crate::shell::EVALUATOR_REVISION >= 9);
        }
        for same in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("copy.json");
            let competing = if same {
                ACTUAL8.to_vec()
            } else {
                serde_json::to_vec(&empty()).unwrap()
            };
            let result = preserve_copy_with_race(&path, ACTUAL8, || {
                std::fs::write(&path, &competing).map_err(|e| e.to_string())
            });
            assert_eq!(result.is_ok(), same);
            assert_eq!(std::fs::read(&path).unwrap(), competing);
        }
        for callback_fails in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("progress.json");
            std::fs::write(&path, ACTUAL8).unwrap();
            let candidate = empty();
            let result = save_with_writer(
                &path,
                &candidate,
                WriteMode::Normal,
                |raw| {
                    assert_eq!(raw, Some(ACTUAL8));
                    if callback_fails {
                        Err("controlled callback failure".into())
                    } else {
                        Ok(())
                    }
                },
                |_, _| Err("controlled persistence failure".into()),
            );
            assert!(result.is_err());
            assert_eq!(std::fs::read(&path).unwrap(), ACTUAL8);
            assert_eq!(
                std::fs::read(path.with_extension(format!(
                    "before-shell-v{}.json",
                    crate::shell::EVALUATOR_REVISION
                )))
                .unwrap(),
                ACTUAL8
            );
            assert!(candidate.migration_receipts.is_none());
        }
    }
    #[test]
    fn genuine8_replacement_sync_warning_commits_current_without_claiming_rollback() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        std::fs::write(&path, ACTUAL8).unwrap();
        let candidate = empty();
        let saved = save_with_writer(
            &path,
            &candidate,
            WriteMode::Normal,
            |_| Ok(()),
            |p, b| {
                write_atomic_with_sync(p, b, |_| {
                    Err(std::io::Error::other("controlled postcommit sync warning"))
                })
            },
        )
        .unwrap();
        assert!(saved.durability_warning.is_some());
        let disk: Progress = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            disk.migration_receipts.as_ref().unwrap().shell,
            vec![7, 8, crate::shell::EVALUATOR_REVISION]
        );
        assert!(candidate.migration_receipts.is_none());
        save(&path, &disk).unwrap();
        assert_eq!(
            std::fs::read(path.with_extension(format!(
                "before-shell-v{}.json",
                crate::shell::EVALUATOR_REVISION
            )))
            .unwrap(),
            ACTUAL8
        );
    }
}

#[cfg(test)]
mod actual9_before10_writer_acceptance {
    use super::*;
    use crate::actual9_test_evidence as evidence;
    fn empty() -> Progress {
        Progress {
            schema_version: 1,
            ..Default::default()
        }
    }
    #[test]
    #[ignore = "Requires fixed private actual9 COMPLETE; explicit guarded replay only"]
    fn actual9_copy_races_do_not_overwrite_and_compare_exact_raw_bytes() {
        assert_eq!(crate::shell::EVALUATOR_REVISION, 19);
        let raw = evidence::original9();
        for same in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("copy.json");
            let other = if same {
                raw.clone()
            } else {
                serde_json::to_vec(&empty()).unwrap()
            };
            let result = preserve_copy_with_race(&path, &raw, || {
                std::fs::write(&path, &other).map_err(|e| e.to_string())
            });
            assert_eq!(result.is_ok(), same);
            assert_eq!(std::fs::read(path).unwrap(), other);
        }
    }
    #[test]
    #[ignore = "Requires fixed private actual9 COMPLETE; explicit guarded replay only"]
    fn actual9_callback_and_persist_failure_leave_exact_target_caller_and_safety_copy() {
        let raw = evidence::original9();
        for callback_fails in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("progress.json");
            std::fs::write(&path, &raw).unwrap();
            let candidate = empty();
            let before = serde_json::to_value(&candidate).unwrap();
            let result = save_with_writer(
                &path,
                &candidate,
                WriteMode::Normal,
                |original| {
                    assert_eq!(original, Some(raw.as_slice()));
                    if callback_fails {
                        Err("controlled before10 callback failure".into())
                    } else {
                        Ok(())
                    }
                },
                |_, _| Err("controlled before10 persistence failure".into()),
            );
            assert!(result.is_err());
            assert_eq!(std::fs::read(&path).unwrap(), raw);
            assert_eq!(
                std::fs::read(path.with_extension(format!(
                    "before-shell-v{}.json",
                    crate::shell::EVALUATOR_REVISION
                )))
                .unwrap(),
                raw
            );
            evidence::exact(&candidate, &before);
            assert!(candidate.migration_receipts.is_none());
        }
    }
    #[test]
    #[ignore = "Requires fixed private actual9 COMPLETE; explicit guarded replay only"]
    fn actual9_postcommit_sync_warning_records_only_the_real_transition_to_current() {
        let raw = evidence::original9();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        std::fs::write(&path, &raw).unwrap();
        let candidate = empty();
        let saved = save_with_writer(
            &path,
            &candidate,
            WriteMode::Normal,
            |_| Ok(()),
            |p, b| {
                write_atomic_with_sync(p, b, |_| {
                    Err(std::io::Error::other("controlled before10 sync warning"))
                })
            },
        )
        .unwrap();
        assert!(saved.durability_warning.is_some());
        assert_eq!(
            std::fs::read(path.with_extension(format!(
                "before-shell-v{}.json",
                crate::shell::EVALUATOR_REVISION
            )))
            .unwrap(),
            raw
        );
        let disk: Progress = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(
            disk.migration_receipts.unwrap().shell,
            vec![7, 8, 9, crate::shell::EVALUATOR_REVISION]
        );
        assert!(candidate.migration_receipts.is_none());
    }
}

pub fn current_one_off_checks(a: &Attempt, s: &Scenario) -> Vec<(bool, String)> {
    let d = match crate::one_off_workspace::context(a, s) {
        Ok(d) => d,
        Err(e) => return vec![(false, e)],
    };
    let State::OneOff { simulation } = &a.state else {
        return vec![(false, "One-off state differs".into())];
    };
    let Some(session) = a
        .workspace
        .as_ref()
        .and_then(|w| w.one_off_session.as_ref())
    else {
        return vec![(
            false,
            "Prepare the one-off editing workspace before checking".into(),
        )];
    };
    crate::one_off_workspace::checks_with_session(d, simulation, session)
}

/// A one-time private copy for the review candidate. Never writes the installed profile.
pub fn pilot_profile(original: &std::path::Path) -> Result<std::path::PathBuf, String> {
    use std::{fs, io::Write};
    let parent = original.parent().ok_or("Missing profile directory")?;
    let dir = parent.join("ccna-presentation-pilot");
    if dir.is_symlink() {
        return Err("Pilot profile directory cannot be a symbolic link".into());
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    }
    for name in [
        original.file_name().ok_or("Missing profile name")?,
        std::ffi::OsStr::new("preferences-v1.json"),
    ] {
        let source = parent.join(name);
        let destination = dir.join(name);
        if destination.is_symlink() {
            return Err("Pilot profile files cannot be symbolic links".into());
        }
        if let Ok(meta) = destination.symlink_metadata() {
            if !meta.is_file() {
                return Err("Pilot profile must be an ordinary file".into());
            }
            continue;
        }
        let Ok(meta) = source.symlink_metadata() else {
            continue;
        };
        if !meta.is_file() || meta.len() > 16_000_000 {
            return Err("Existing profile is not a bounded ordinary file".into());
        }
        let bytes = crate::local_file::read_bounded(&source, 16_000_000)?;
        let mut pending = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
        pending
            .write_all(&bytes)
            .and_then(|_| pending.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        pending
            .persist_noclobber(destination)
            .map_err(|e| e.to_string())?;
        #[cfg(unix)]
        fs::File::open(&dir)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
    }
    Ok(dir.join(original.file_name().unwrap()))
}
