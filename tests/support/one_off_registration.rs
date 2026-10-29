//! Closed OneOff47→46 test-only inverse; no runtime loader or history changes.
use super::digest;
use dlms_learning_lab::{engine::State, pack};
use serde_json::Value;
pub const IDS: [&str; 4] = [
    "lpic1-at-report",
    "lpic1-at-capture-repair",
    "lpic1-at-owner-queue",
    "lpic1-at-handover",
];
fn recipe() -> Value {
    let raw = include_bytes!("../fixtures/catalog180-registration.json");
    assert_eq!(
        digest::hash(raw),
        "c472de759a336ae226a9349da605b569e08b9ab65c8b86d162276b78d01cda0d"
    );
    serde_json::from_slice(raw).unwrap()
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let r = recipe();
    if digest::hash(raw) != r["after_units47_sha256"].as_str().ok_or("47pin")?
        || raw.len() as u64 != r["after_units47_bytes"].as_u64().ok_or("47bytes")?
    {
        return Err("Exact OneOff47 raw units differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let suffix = r["append_suffix_utf8"].as_str().ok_or("47suffix")?;
    let prefix = text
        .strip_suffix(suffix)
        .ok_or("Exact OneOff append differs")?;
    let before = format!("{prefix}{}", r["old_tail_utf8"].as_str().ok_or("46tail")?).into_bytes();
    if digest::hash(&before) != r["before_units46_sha256"].as_str().ok_or("46pin")?
        || before.len() as u64 != r["before_units46_bytes"].as_u64().ok_or("46bytes")?
        || before.as_slice() != include_bytes!("../fixtures/one-off-units-before176.json")
    {
        return Err("Original46 raw units differ".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != r["before_units46"] {
        return Err("Original46 typed objects differ".into());
    }
    let units = expected["units"].as_array_mut().ok_or("46units")?;
    if units.len() != 46 {
        return Err("46count".into());
    }
    units.push(r["one_off_unit"].clone());
    if expected != serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? {
        return Err("Change outside OneOff append".into());
    }
    Ok(before)
}
pub fn original_catalog(all: &[pack::Scenario]) -> Vec<pack::Scenario> {
    let r = recipe();
    assert_eq!(all.len(), 180);
    assert_eq!(
        all[176..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let raw = include_bytes!("../../scenarios/linux-one-off-v1.json");
    assert_eq!(digest::hash(raw), r["canonical_pack_sha256"]);
    let p: pack::Pack = serde_json::from_slice(raw).unwrap();
    assert_eq!(
        serde_json::to_value(&all[176..]).unwrap(),
        serde_json::to_value(&p.scenarios).unwrap()
    );
    assert!(
        all[176..]
            .iter()
            .all(|s| matches!(s.initial, State::OneOff { .. }) && s.one_off_teaching.is_some())
    );
    let packs: [&str; 44] = [
        include_str!("../../scenarios/starter-v1.json"),
        include_str!("../../scenarios/shell-unit-v1.json"),
        include_str!("../../scenarios/shell-composition-v1.json"),
        include_str!("../../scenarios/shell-reusable-v1.json"),
        include_str!("../../scenarios/linux-files-v1.json"),
        include_str!("../../scenarios/ipv6-static-v1.json"),
        include_str!("../../scenarios/shell-scope-v1.json"),
        include_str!("../../scenarios/shell-text-v1.json"),
        include_str!("../../scenarios/shell-counts-v1.json"),
        include_str!("../../scenarios/linux-access-v1.json"),
        include_str!("../../scenarios/vlan-data-v1.json"),
        include_str!("../../scenarios/shell-file-batches-v1.json"),
        include_str!("../../scenarios/ipv4-static-v1.json"),
        include_str!("../../scenarios/dns-forward-v1.json"),
        include_str!("../../scenarios/shell-arrays-v1.json"),
        include_str!("../../scenarios/linux-processes-v1.json"),
        include_str!("../../scenarios/discovery-v1.json"),
        include_str!("../../scenarios/storage-v1.json"),
        include_str!("../../scenarios/shell-capture-v1.json"),
        include_str!("../../scenarios/acl-policy-v1.json"),
        include_str!("../../scenarios/accounts-lifecycle-v1.json"),
        include_str!("../../scenarios/ospf-p2p-v1.json"),
        include_str!("../../scenarios/unit-manager-v1.json"),
        include_str!("../../scenarios/shell-case-v1.json"),
        include_str!("../../scenarios/mac-learning-v1.json"),
        include_str!("../../scenarios/linux-find-v1.json"),
        include_str!("../../scenarios/dns-reverse-v1.json"),
        include_str!("../../scenarios/shell-getopts-v1.json"),
        include_str!("../../scenarios/dhcpv4-client-v1.json"),
        include_str!("../../scenarios/debian-packages-v1.json"),
        include_str!("../../scenarios/spanning-tree-v1.json"),
        include_str!("../../scenarios/shell-safe-automation-v1.json"),
        include_str!("../../scenarios/apache-static-v1.json"),
        include_str!("../../scenarios/nat-basic-v1.json"),
        include_str!("../../scenarios/rpm-packages-v1.json"),
        include_str!("../../scenarios/shell-command-vectors-v1.json"),
        include_str!("../../scenarios/nfs-v1.json"),
        include_str!("../../scenarios/etherchannel-v1.json"),
        include_str!("../../scenarios/shell-optional-inputs-v1.json"),
        include_str!("../../scenarios/linux-cron-v1.json"),
        include_str!("../../scenarios/file-backup-v1.json"),
        include_str!("../../scenarios/shell-numeric-validation-v1.json"),
        include_str!("../../scenarios/physical-links-v1.json"),
        include_str!("../../scenarios/ipv4-private-planning-v1.json"),
    ];
    for (raw, pin) in packs.iter().zip(r["before_packs"].as_array().unwrap()) {
        assert_eq!(raw.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(
            digest::hash(raw.as_bytes()),
            pin["sha256"].as_str().unwrap()
        );
    }
    let old = pack::load_catalog(&packs).unwrap();
    assert_eq!(old.len(), 176);
    assert_eq!(
        serde_json::to_value(&all[..176]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        serde_json::to_value(
            old.iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        )
        .unwrap(),
        r["before_ids"]
    );
    let raw = include_bytes!("../../scenarios/units-v1.json");
    before_units(raw).unwrap();
    old
}
