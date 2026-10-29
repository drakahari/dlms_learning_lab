//! Real existing-family prepare matrices. Source-only until Root gates.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    engine::Goal, pack::Scenario, physical_link_workspace as work, progress::Attempt, workspace,
};
#[path = "support/physical_scenario.rs"]
mod fixture;
use std::collections::BTreeSet;
const FIELDS: [&str; 30] = [
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
    "file_backup_teaching",
];
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
        29 => lab::file_backup_teaching::prepare(a, s.file_backup_teaching.as_ref().unwrap(), new),
        _ => unreachable!(),
    }
}
fn wire(a: &Attempt) -> Vec<u8> {
    serde_json::to_vec(a).unwrap()
}
#[test]
fn all_thirty_real_old_prepares_reject_new_kind_before_metadata_and_valid_optional_payloads() {
    let source = fixture::scenario();
    let mut prepared = Attempt::new(&source, 9);
    work::prepare_attempt(&mut prepared, &source, true).unwrap();
    let baseline = prepared.workspace.as_ref().unwrap();
    let draft = baseline.physical_link_draft.clone().unwrap();
    let learning = baseline.physical_link_learning.clone().unwrap();
    assert!(learning.valid());
    let all = dlms_learning_lab::pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    let mut seen = BTreeSet::new();
    for old in &all {
        let value = serde_json::to_value(old).unwrap();
        for (site, field) in FIELDS
            .iter()
            .enumerate()
            .filter(|(_, field)| value.get(**field).is_some())
        {
            seen.insert(*field);
            // Real own-family positive; never construct an invalid lesson to make
            // the foreign rejection appear to work.
            let mut own = Attempt::new(old, 3);
            workspace::prepare(&mut own);
            invoke(site, &mut own, old, true)
                .unwrap_or_else(|e| panic!("positive {field}/{}: {e}", old.id));
            assert!(own.workspace.as_ref().unwrap().valid());
            for new in [false, true] {
                for with_metadata in [false, true] {
                    let mut incoming = if with_metadata {
                        prepared.clone()
                    } else {
                        Attempt::new(&source, 9)
                    };
                    let before = wire(&incoming);
                    let _ = invoke(site, &mut incoming, old, new);
                    assert_eq!(wire(&incoming), before, "physical kind {field}/{}", old.id);
                }
            }
            for mask in 1..4 {
                let mut foreign = own.clone();
                let w = foreign.workspace.as_mut().unwrap();
                if mask & 1 != 0 {
                    w.physical_link_draft = Some(draft.clone());
                }
                if mask & 2 != 0 {
                    w.physical_link_learning = Some(learning.clone());
                }
                assert!(w.valid());
                let before = wire(&foreign);
                for new in [false, true] {
                    let _ = invoke(site, &mut foreign, old, new);
                    assert_eq!(
                        wire(&foreign),
                        before,
                        "valid foreign physical {field}/{} mask{mask}",
                        old.id
                    );
                }
                let w = foreign.workspace.as_mut().unwrap();
                w.physical_link_draft = None;
                w.physical_link_learning = None;
                invoke(site, &mut foreign, old, false)
                    .unwrap_or_else(|e| panic!("restored {field}: {e}"));
            }
        }
    }
    assert_eq!(seen, FIELDS.into_iter().collect());
}
#[test]
fn every_old_source_refuses_the_new_teaching_field_and_every_physical_source_refuses_old_payloads()
{
    let source = fixture::scenario();
    let all = dlms_learning_lab::pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    let lesson = source.physical_link_teaching.clone();
    assert_eq!(
        all[171..175]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "ccna-physical-media-span",
            "ccna-physical-duplex-diagnosis",
            "ccna-physical-counter-baseline",
            "ccna-physical-uplink-handover"
        ]
    );
    for own in &all[171..175] {
        assert!(own.physical_link_teaching.is_some());
    }
    let canonical = dlms_learning_lab::pack::Pack {
        schema_version: 1,
        scenarios: all[171..175].to_vec(),
    };
    let positive =
        dlms_learning_lab::pack::load(&serde_json::to_string(&canonical).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&positive).unwrap(),
        serde_json::to_value(&all[171..175]).unwrap(),
        "All four positive sources load with every genuine prerequisite intact"
    );
    let contexts = |rows: &[Scenario]| {
        let chunks = rows
            .chunks(16)
            .map(|chunk| {
                serde_json::to_string(&dlms_learning_lab::pack::Pack {
                    schema_version: 1,
                    scenarios: chunk.to_vec(),
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        dlms_learning_lab::pack::load_catalog(
            &chunks.iter().map(String::as_str).collect::<Vec<_>>(),
        )
    };
    assert_eq!(
        serde_json::to_value(contexts(&all).unwrap()).unwrap(),
        serde_json::to_value(&all).unwrap(),
        "Complete independent baseline is valid"
    );
    let old_indices = all
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            !matches!(
                s.initial,
                dlms_learning_lab::engine::State::PhysicalLink { .. }
            )
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(
        old_indices[..172].len(),
        172,
        "Every previous171 and the new private-office kind is tested"
    );
    let prior180 = one_off_boundary::loop_registration::before_scenarios(&all);
    let previous_indices = prior180
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            !matches!(
                s.initial,
                dlms_learning_lab::engine::State::PhysicalLink { .. }
            )
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(previous_indices.len(), 176);
    assert_eq!(
        &old_indices[..previous_indices.len()],
        previous_indices.as_slice()
    );
    assert_eq!(old_indices.len(), 182);
    let prior182 = one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all);
    let prior178 = prior182
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            !matches!(
                s.initial,
                dlms_learning_lab::engine::State::PhysicalLink { .. }
            )
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(prior178.len(), 178);
    assert_eq!(&old_indices[..178], prior178.as_slice());
    assert_eq!(&old_indices[178..], &[182, 183, 184, 185]);
    assert_eq!(
        old_indices[178..]
            .iter()
            .map(|i| all[*i].id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic2-lvm-free-space",
            "lpic2-lvm-add-pv",
            "lpic2-lvm-filesystem-gap",
            "lpic2-lvm-handover"
        ]
    );
    assert!(old_indices[178..].iter().all(|i| matches!(
        all[*i].initial,
        dlms_learning_lab::engine::State::Lvm { .. }
    ) && matches!(
        all[*i].goal,
        dlms_learning_lab::engine::Goal::Lvm { .. }
    )));
    assert_eq!(
        &old_indices[previous_indices.len()..previous_indices.len() + 2],
        &[180, 181]
    );
    assert_eq!(
        old_indices[previous_indices.len()..previous_indices.len() + 2]
            .iter()
            .map(|i| all[*i].id.as_str())
            .collect::<Vec<_>>(),
        ["shell-loop-intake", "shell-loop-fallback"]
    );
    assert!(
        old_indices[previous_indices.len()..previous_indices.len() + 2]
            .iter()
            .all(|i| matches!(
                all[*i].initial,
                dlms_learning_lab::engine::State::Script { .. }
            ) && matches!(all[*i].goal, dlms_learning_lab::engine::Goal::Script { .. }))
    );
    assert_eq!(&previous_indices[172..], &[176, 177, 178, 179]);
    assert!(previous_indices[172..].iter().all(|i| matches!(
        &all[*i].initial,
        dlms_learning_lab::engine::State::OneOff { .. }
    )));

    for index in old_indices {
        let mut foreign = all.clone();
        foreign[index].physical_link_teaching = lesson.clone();
        let error = contexts(&foreign).unwrap_err();
        let expected = if matches!(
            all[index].initial,
            dlms_learning_lab::engine::State::Lvm { .. }
        ) {
            "Foreign teaching metadata on LVM source"
        } else {
            "Physical-link teaching/Goal belongs to another activity"
        };
        assert_eq!(
            error, expected,
            "{} must fail for the injected metadata, not missing prerequisites or a previously mutated neighbor",
            all[index].id
        );
    }
    let own = Attempt::new(&source, 9);
    for old in dlms_learning_lab::pack::bundled().unwrap() {
        let value = serde_json::to_value(old).unwrap();
        for field in FIELDS {
            if let Some(payload) = value.get(field) {
                let mut value = serde_json::to_value(&source).unwrap();
                value[field] = payload.clone();
                let wrong: Scenario = serde_json::from_value(value).unwrap();
                assert!(!work::source_shape(&wrong));
                assert!(work::context(&own, &wrong).is_err());
            }
        }
    }
}

#[test]
fn backup_direct_context_and_commands_reject_each_valid_physical_optional_field_atomically() {
    use dlms_learning_lab::{file_backup_teaching, file_backup_workspace as backup};
    let physical = fixture::scenario();
    let mut sample = Attempt::new(&physical, 9);
    work::prepare_attempt(&mut sample, &physical, true).unwrap();
    let payload = sample.workspace.as_ref().unwrap();
    let source = dlms_learning_lab::pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| {
            matches!(
                s.initial,
                dlms_learning_lab::engine::State::FileBackup { .. }
            )
        })
        .unwrap();
    let mut clean = Attempt::new(&source, 3);
    workspace::prepare(&mut clean);
    file_backup_teaching::prepare(
        &mut clean,
        source.file_backup_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    assert!(backup::context(&clean, &source).is_ok());
    assert!(backup::source_shape(&source));
    for mask in 1..4 {
        let mut foreign = clean.clone();
        let w = foreign.workspace.as_mut().unwrap();
        if mask & 1 != 0 {
            w.physical_link_draft = payload.physical_link_draft.clone();
        }
        if mask & 2 != 0 {
            w.physical_link_learning = payload.physical_link_learning.clone();
        }
        assert!(w.valid());
        let before = wire(&foreign);
        assert!(backup::context(&foreign, &source).is_err());
        for scope in [backup::Scope::Assignment, backup::Scope::Practice] {
            assert!(backup::perform(&mut foreign, &source, scope, "pwd").is_err());
            assert_eq!(wire(&foreign), before, "foreign physical mask {mask}");
        }
    }
    let mut foreign_source = source.clone();
    foreign_source.physical_link_teaching = physical.physical_link_teaching.clone();
    assert!(!backup::source_shape(&foreign_source));
    let before = wire(&clean);
    assert!(backup::context(&clean, &foreign_source).is_err());
    for scope in [backup::Scope::Assignment, backup::Scope::Practice] {
        assert!(backup::perform(&mut clean, &foreign_source, scope, "pwd").is_err());
        assert_eq!(wire(&clean), before);
    }
    assert!(backup::context(&clean, &source).is_ok());
}
