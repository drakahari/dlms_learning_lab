//! Normal LACP registration: exact public154/39 prefixes plus three authored tasks.
use dlms_learning_lab::{course, engine::State, pack, progress};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PIN: &str = "0d2035c4da961607c6b40066a944040f2ee133a1270dceefe5c69598768f9969";
const IDS: [&str; 3] = [
    "ccna-lacp-initiation",
    "ccna-lacp-member-service",
    "ccna-lacp-resilient-handover",
];
fn boundary() -> Value {
    let raw = include_str!("fixtures/catalog157-original154-boundary.json");
    assert_eq!(digest::hash(raw.as_bytes()), PIN);
    serde_json::from_str(raw).unwrap()
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}
#[test]
fn original154_full_pack_bytes_typed_objects_and39_unit_objects_remain_exact() {
    let pin = boundary();
    assert_eq!(
        pin["public_source_commit"],
        "a6aca2c31967d29c5d48ea3406b3ecf8a4bab268"
    );
    let mut texts = Vec::new();
    for item in pin["packs"].as_array().unwrap() {
        let name = item["path"].as_str().unwrap();
        assert!(name.starts_with("scenarios/") && !name.contains(".."));
        let raw =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
        assert_eq!(raw.len() as u64, item["bytes"].as_u64().unwrap());
        assert_eq!(digest::hash(&raw), item["sha256"].as_str().unwrap());
        texts.push(String::from_utf8(raw).unwrap());
    }
    assert_eq!(texts.len(), 37);
    let old = pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(old.len(), 154);
    exact(
        &old.iter()
            .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
            .collect::<Vec<_>>(),
        &pin["catalog"],
    );
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 168);
    exact(&all[..154].to_vec(), &old);
    let declared: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/etherchannel-v1.json")).unwrap();
    exact(&all[154..157].to_vec(), &declared.scenarios);
    assert_eq!(
        all[154..157]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all.iter().filter(|s| s.track == track).count()),
        [48, 36, 32, 52]
    );
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let reviewed41 = combined_catalog::before_cron_units(current_raw).unwrap();
    let raw = reviewed41.as_slice();
    let ether40 = combined_catalog::before_optional_units(raw).unwrap();
    let old_raw = combined_catalog::before_lacp_units(&ether40).unwrap();
    assert_eq!(
        old_raw,
        pin["original_units_utf8"].as_str().unwrap().as_bytes()
    );
    let old_units = course::load(std::str::from_utf8(&old_raw).unwrap(), &old).unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!((old_units.len(), units.len()), (39, 43));
    exact(&units[..39].to_vec(), &old_units);
    exact(&units[39], &pin["append"]["new_unit"]);
    assert_eq!(units[39].prerequisites, ["vlan-data-foundations"]);
    // Also retain the prior exact37 raw-byte inverse and both prior append pins.
    let original37 = combined_catalog::old_unit_bytes(raw).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&original37).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
}
#[test]
fn closed_lacp_inverse_rejects_missing_extra_reordered_or_changed_unit_bytes() {
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let reviewed41 = combined_catalog::before_cron_units(current_raw).unwrap();
    let current = reviewed41.as_slice();
    let ether40 = combined_catalog::before_optional_units(current).unwrap();
    let raw = ether40.as_slice();
    let old = combined_catalog::before_lacp_units(raw).unwrap();
    assert!(combined_catalog::before_lacp_units(&old).is_err());
    let mut whitespace = raw.to_vec();
    whitespace.push(b' ');
    assert!(combined_catalog::before_lacp_units(&whitespace).is_err());
    let mut edited: Value = serde_json::from_slice(raw).unwrap();
    edited["units"][39]["title"] = "Unreviewed LACP unit".into();
    assert!(
        combined_catalog::before_lacp_units(&serde_json::to_vec_pretty(&edited).unwrap()).is_err()
    );
    let mut reordered: Value = serde_json::from_slice(raw).unwrap();
    reordered["units"].as_array_mut().unwrap().swap(38, 39);
    assert!(
        combined_catalog::before_lacp_units(&serde_json::to_vec_pretty(&reordered).unwrap())
            .is_err()
    );
    let mut original_changed = raw.to_vec();
    let byte = original_changed.iter().position(|b| *b == b'I').unwrap();
    original_changed[byte] = b'X';
    assert!(combined_catalog::before_lacp_units(&original_changed).is_err());
}
#[test]
fn unchanged_actual_legacy_writer_gains_no_lacp_answers_or_credit() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let before: progress::Progress = serde_json::from_slice(raw).unwrap();
    let loaded = progress::decode(raw, &all).unwrap();
    exact(&loaded.attempts, &before.attempts);
    exact(&loaded.favorites, &before.favorites);
    for (n, id) in IDS.iter().enumerate() {
        assert!(!loaded.attempts.contains_key(*id) && !loaded.favorites.contains(*id));
        let source = &all[154 + n];
        assert_eq!(source.id, *id);
        assert!(matches!(source.initial, State::EtherChannel { .. }));
        let attempt = progress::Attempt::new(source, 1);
        assert!(!progress::current_solved(&attempt, source));
    }
}
