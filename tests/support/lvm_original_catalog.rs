//! Cold independent source reconstruction used only by explicit preservation tests.
use dlms_learning_lab::developer_digest as digest;
use dlms_learning_lab::pack::Scenario;
use serde_json::Value;
fn recipe() -> Value {
    let raw = include_bytes!("../fixtures/catalog186-registration.json");
    assert_eq!(
        digest::hash(raw),
        "72befcc13bf6fb0f8d78b301f699de98b105430d5b4240babcc3b7b409cab861"
    );
    serde_json::from_slice(raw).unwrap()
}
/// Cold independent182 catalog for historical full-object preservation tests.
/// No actual learner state, reference result or source is taken from the input.
pub fn load() -> Vec<Scenario> {
    let packs = [
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
        include_str!("../../scenarios/linux-one-off-v1.json"),
        include_str!("../../scenarios/shell-loop-recovery-v1.json"),
    ];
    for (raw, pin) in packs
        .iter()
        .zip(recipe()["before_packs"].as_array().unwrap())
    {
        assert_eq!(raw.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(
            digest::hash(raw.as_bytes()),
            pin["sha256"].as_str().unwrap()
        );
    }
    let all = dlms_learning_lab::pack::load_catalog(&packs).unwrap();
    assert_eq!(all.len(), 182);
    assert_eq!(
        all.iter()
            .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
            .collect::<Vec<_>>(),
        *recipe()["before_ids"].as_array().unwrap()
    );
    all
}
