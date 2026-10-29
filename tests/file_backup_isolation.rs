//! Real valid cross-family preparation inputs; no installed learner State/Progress.
use dlms_learning_lab::{
    engine::Goal, file_backup_teaching as teach, pack::Scenario, progress::Attempt, workspace,
};
use std::collections::BTreeSet;
#[path = "support/file_backup_authored.rs"]
mod authored;
const FIELDS: [&str; 29] = [
    "teaching",
    "terminal_teaching",
    "process_teaching",
    "accounts_teaching",
    "unit_manager_teaching",
    "ospf_teaching",
    "storage_teaching",
    "acl_teaching",
    "ipv6_teaching",
    "ipv4_routing_teaching",
    "address_teaching",
    "service_teaching",
    "client_teaching",
    "seed_dns_teaching",
    "seed_mount_teaching",
    "legacy_access_teaching",
    "legacy_logs_teaching",
    "dns_teaching",
    "switching_teaching",
    "mac_teaching",
    "debian_packages_teaching",
    "apache_static_teaching",
    "nat_teaching",
    "spanning_tree_teaching",
    "dhcpv4_teaching",
    "nfs_teaching",
    "rpm_packages_teaching",
    "etherchannel_teaching",
    "cron_teaching",
];
fn wire(a: &Attempt) -> Vec<u8> {
    serde_json::to_vec(a).unwrap()
}
fn invoke(site: usize, a: &mut Attempt, s: &Scenario, new: bool) -> Result<(), String> {
    use dlms_learning_lab as lab;
    match site {
        0 => {
            lab::script_teaching::prepare(a, s.teaching.as_ref().unwrap(), new);
            Ok(())
        }
        1 => {
            lab::terminal_teaching::prepare(a, s.terminal_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        2 => {
            lab::process_teaching::prepare(a, s.process_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        3 => {
            lab::accounts_teaching::prepare(a, s.accounts_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        4 => lab::unit_manager_teaching::prepare(a, s.unit_manager_teaching.as_ref().unwrap(), new),
        5 => lab::ospf_teaching::prepare(a, s.ospf_teaching.as_ref().unwrap(), new),
        6 => {
            lab::storage_teaching::prepare(a, s.storage_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        7 => lab::acl_teaching::prepare(a, s.acl_teaching.as_ref().unwrap(), new),
        8 => {
            lab::ipv6_teaching::prepare(a, s.ipv6_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        9 => {
            lab::ipv4_routing_teaching::prepare(a, s.ipv4_routing_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        10 => {
            lab::address_teaching::prepare(a, s.address_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        11 => {
            lab::service_teaching::prepare(a, s.service_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        12 => lab::client_teaching::prepare(a, s.client_teaching.as_ref().unwrap(), new),
        13 => lab::seed_dns_teaching::prepare(a, s.seed_dns_teaching.as_ref().unwrap(), new),
        14 => lab::seed_mount_teaching::prepare(a, s.seed_mount_teaching.as_ref().unwrap(), new),
        15 => {
            lab::legacy_access_teaching::prepare(a, s.legacy_access_teaching.as_ref().unwrap(), new)
        }
        16 => lab::legacy_logs_teaching::prepare(a, s.legacy_logs_teaching.as_ref().unwrap(), new),
        17 => {
            lab::dns_teaching::prepare(a, s.dns_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        18 => {
            lab::switching_teaching::prepare(a, s.switching_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        19 => {
            lab::mac_learning_teaching::prepare(a, s.mac_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        20 => lab::debian_packages_teaching::prepare(
            a,
            s.debian_packages_teaching.as_ref().unwrap(),
            new,
        ),
        21 => lab::apache_static_teaching::prepare(a, s.apache_static_teaching.as_ref().unwrap()),
        22 => lab::nat_teaching::prepare(a, s.nat_teaching.as_ref().unwrap(), new),
        23 => lab::spanning_tree_teaching::prepare(a, s.spanning_tree_teaching.as_ref().unwrap()),
        24 => lab::dhcpv4_teaching::prepare(a, s.dhcpv4_teaching.as_ref().unwrap(), new),
        25 => lab::nfs_teaching::prepare(a, s, new),
        26 => {
            let Goal::RpmPackages { definition } = &s.goal else {
                panic!("real RPM source")
            };
            lab::rpm_packages_teaching::prepare(
                a,
                definition,
                s.rpm_packages_teaching.as_ref().unwrap(),
                new,
            )
        }
        27 => lab::etherchannel_teaching::prepare(a, s, new),
        28 => lab::cron_workspace::budget::prepare(a, s),
        _ => unreachable!(),
    }
}
#[test]
fn every_real_old_prepare_retains_backup_kind_with_and_without_its_optional_learning() {
    let all = authored::catalog();
    let reviewed = authored::review();
    assert_eq!(
        reviewed["labs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "lpic2-backup-selective-recovery",
            "lpic2-backup-incremental-chain",
            "lpic2-backup-snapshot-branch",
            "lpic2-backup-recovery-handover",
        ]
    );
    let source = authored::lab("lpic2-backup-selective-recovery");
    let mut prepared = Attempt::new(source, 27);
    workspace::prepare(&mut prepared);
    teach::prepare(
        &mut prepared,
        source.file_backup_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    assert!(
        prepared
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .valid()
    );
    let mut seen = BTreeSet::new();
    for old in all {
        let raw = serde_json::to_value(old).unwrap();
        for (site, field) in FIELDS
            .iter()
            .enumerate()
            .filter(|(_, field)| raw.get(**field).is_some())
        {
            seen.insert(*field);
            for with_learning in [false, true] {
                for new in [false, true] {
                    let mut a = if with_learning {
                        prepared.clone()
                    } else {
                        Attempt::new(source, 27)
                    };
                    let before = wire(&a);
                    let _ = invoke(site, &mut a, old, new);
                    assert_eq!(
                        wire(&a),
                        before,
                        "{field}/{} new={new} learning={with_learning}",
                        old.id
                    );
                    assert!(workspace::is_file_backup_workspace(&a));
                }
            }
        }
    }
    assert_eq!(seen, FIELDS.into_iter().collect());
}
#[test]
fn valid_backup_learning_on_an_old_kind_refuses_before_mutation_but_own_preparation_still_works() {
    let all = authored::catalog();
    let backup = authored::lab("lpic2-backup-selective-recovery");
    let learning = teach::Learning::new(backup.file_backup_teaching.as_ref().unwrap()).unwrap();
    assert!(learning.valid());
    let mut seen = BTreeSet::new();
    for source in all {
        let raw = serde_json::to_value(source).unwrap();
        for (site, field) in FIELDS
            .iter()
            .enumerate()
            .filter(|(_, field)| raw.get(**field).is_some())
        {
            seen.insert(*field);
            let mut own = Attempt::new(source, 19);
            workspace::prepare(&mut own);
            invoke(site, &mut own, source, true)
                .unwrap_or_else(|e| panic!("positive {field}/{}: {e}", source.id));
            assert!(own.workspace.as_ref().unwrap().valid());
            assert!(!workspace::is_file_backup_workspace(&own));
            let state = own.state.clone();
            own.workspace.as_mut().unwrap().file_backup_learning = Some(learning.clone());
            assert!(
                own.workspace.as_ref().unwrap().valid(),
                "The injected learning is structurally valid"
            );
            assert!(workspace::is_file_backup_workspace(&own));
            let before = wire(&own);
            for new in [false, true] {
                let _ = invoke(site, &mut own, source, new);
                assert_eq!(wire(&own), before, "foreign learning {field}/{}", source.id);
            }
            assert_eq!(own.state, state);
            own.workspace.as_mut().unwrap().file_backup_learning = None;
            invoke(site, &mut own, source, false)
                .unwrap_or_else(|e| panic!("restored positive {field}/{}: {e}", source.id));
            assert!(own.workspace.as_ref().unwrap().valid());
        }
    }
    assert_eq!(seen, FIELDS.into_iter().collect());
}
