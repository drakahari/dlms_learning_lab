//! Normal catalog registration; private capability/content fixtures do not earn credit.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{course, pack};
use serde_json::{Value, json};
const ID: &str = "ccna-private-office-plan";
fn before() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/shell-unit-v1.json"),
        include_str!("../scenarios/shell-composition-v1.json"),
        include_str!("../scenarios/shell-reusable-v1.json"),
        include_str!("../scenarios/linux-files-v1.json"),
        include_str!("../scenarios/ipv6-static-v1.json"),
        include_str!("../scenarios/shell-scope-v1.json"),
        include_str!("../scenarios/shell-text-v1.json"),
        include_str!("../scenarios/shell-counts-v1.json"),
        include_str!("../scenarios/linux-access-v1.json"),
        include_str!("../scenarios/vlan-data-v1.json"),
        include_str!("../scenarios/shell-file-batches-v1.json"),
        include_str!("../scenarios/ipv4-static-v1.json"),
        include_str!("../scenarios/dns-forward-v1.json"),
        include_str!("../scenarios/shell-arrays-v1.json"),
        include_str!("../scenarios/linux-processes-v1.json"),
        include_str!("../scenarios/discovery-v1.json"),
        include_str!("../scenarios/storage-v1.json"),
        include_str!("../scenarios/shell-capture-v1.json"),
        include_str!("../scenarios/acl-policy-v1.json"),
        include_str!("../scenarios/accounts-lifecycle-v1.json"),
        include_str!("../scenarios/ospf-p2p-v1.json"),
        include_str!("../scenarios/unit-manager-v1.json"),
        include_str!("../scenarios/shell-case-v1.json"),
        include_str!("../scenarios/mac-learning-v1.json"),
        include_str!("../scenarios/linux-find-v1.json"),
        include_str!("../scenarios/dns-reverse-v1.json"),
        include_str!("../scenarios/shell-getopts-v1.json"),
        include_str!("../scenarios/dhcpv4-client-v1.json"),
        include_str!("../scenarios/debian-packages-v1.json"),
        include_str!("../scenarios/spanning-tree-v1.json"),
        include_str!("../scenarios/shell-safe-automation-v1.json"),
        include_str!("../scenarios/apache-static-v1.json"),
        include_str!("../scenarios/nat-basic-v1.json"),
        include_str!("../scenarios/rpm-packages-v1.json"),
        include_str!("../scenarios/shell-command-vectors-v1.json"),
        include_str!("../scenarios/nfs-v1.json"),
        include_str!("../scenarios/etherchannel-v1.json"),
        include_str!("../scenarios/shell-optional-inputs-v1.json"),
        include_str!("../scenarios/linux-cron-v1.json"),
        include_str!("../scenarios/file-backup-v1.json"),
        include_str!("../scenarios/shell-numeric-validation-v1.json"),
        include_str!("../scenarios/physical-links-v1.json"),
    ])
    .unwrap()
}
#[test]
fn normal_catalog_preserves_all_175_full_scenarios_and_appends_one_project() {
    let old = before();
    let all = pack::bundled().unwrap();
    assert_eq!(old.len(), 175);
    assert_eq!(one_off_boundary::before_len(&all), 176);
    assert_eq!(
        serde_json::to_value(&all[..175]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let authored: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/ipv4-private-planning-v1.json")).unwrap();
    assert_eq!(authored.scenarios.len(), 1);
    assert_eq!(
        serde_json::to_value(&all[175]).unwrap(),
        serde_json::to_value(&authored.scenarios[0]).unwrap()
    );
    assert_eq!(all[175].id, ID);
    for (track, count) in [
        ("CCNA", 53),
        ("LPIC-1", 40),
        ("LPIC-2", 32),
        ("Bash/Shell", 57),
    ] {
        assert_eq!(
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all)
                .iter()
                .filter(|s| s.track == track)
                .count(),
            count
        );
        let current = if track == "LPIC-2" { count + 4 } else { count };
        assert_eq!(one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&all).iter().filter(|s|s.track==track).count(),current);
        let current190 = if track == "CCNA" {
            current + 4
        } else {
            current
        };
        assert_eq!(all.iter().filter(|s| s.track == track).count(), current190);
    }
}
#[test]
fn normal_unit_index_preserves_45_units_and_appends_real_prerequisite_guided_project() {
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let old_raw: Value =
        serde_json::from_str(include_str!("fixtures/private-office-units-before175.json")).unwrap();
    let raw: Value = serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    assert_eq!(old_raw["units"].as_array().unwrap().len(), 45);
    assert_eq!(
        json!(&raw["units"].as_array().unwrap()[..45]),
        old_raw["units"]
    );
    assert_eq!(one_off_boundary::before_len(&units), 46);
    let unit = units
        .iter()
        .find(|u| u.id == "private-office-planning")
        .unwrap();
    assert_eq!(unit.track, "CCNA");
    assert_eq!(unit.prerequisites, vec!["ipv4-foundations"]);
    assert_eq!(unit.labs, vec![ID]);
    let scenario = all.iter().find(|s| s.id == ID).unwrap();
    assert_eq!(
        scenario.prerequisites,
        vec!["ccna-ipv4", "ccna-vlsm", "ccna-gateway"]
    );
    assert!(scenario.ipv4_routing_teaching.is_some());
}
