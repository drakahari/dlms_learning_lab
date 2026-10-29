//! Exact optional-input append. Historical154/39/49/350 are independent pins.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn original() -> Vec<Scenario> {
    let review: Value =
        serde_json::from_str(include_str!("fixtures/shell-optional-inputs-review.json")).unwrap();
    let mut texts = Vec::new();
    let mut paths = BTreeSet::new();
    for pin in review["original_source_pins"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        assert!(
            name.starts_with("scenarios/") && std::path::Path::new(name).components().count() == 2
        );
        assert!(paths.insert(name));
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let meta = std::fs::symlink_metadata(&path).unwrap();
        assert!(meta.is_file() && !meta.file_type().is_symlink() && meta.len() <= 2 * 1024 * 1024);
        let raw = std::fs::read(path).unwrap();
        assert_eq!(digest::hash(&raw), pin["sha256"].as_str().unwrap());
        texts.push(String::from_utf8(raw).unwrap());
    }
    assert_eq!(texts.len(), 37);
    let all = pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(all.len(), 154);
    all
}
#[test]
fn original154_ether_three_and_optional_three_have_exact_normal_order_and_fresh_credit() {
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    assert_eq!(value(&all[..154]), value(&original()));
    let ether: Pack =
        serde_json::from_str(include_str!("../scenarios/etherchannel-v1.json")).unwrap();
    assert_eq!(ether.scenarios.len(), 3);
    assert_eq!(value(&all[154..157]), value(&ether.scenarios));
    let added: Pack =
        serde_json::from_str(include_str!("../scenarios/shell-optional-inputs-v1.json")).unwrap();
    assert_eq!(added.scenarios.len(), 3);
    assert_eq!(value(&all[157..160]), value(&added.scenarios));
    assert_eq!(
        all[157..160]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-optional-policy",
            "shell-optional-report",
            "shell-optional-handover"
        ]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [53, 40, 32, 55]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all[..176].iter().filter(|s| s.track == t).count()),
        [53, 36, 32, 55]
    );
    let scripts = all
        .iter()
        .filter_map(|s| {
            if let Goal::Script { cases } = &s.goal {
                Some(cases.len())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!((scripts.len(), scripts.iter().sum::<usize>()), (55, 420));
    for s in &all[157..160] {
        assert!(matches!(s.initial, State::Script { .. }));
        let a = Attempt::new(s, 1);
        assert!(
            !progress::current_solved(&a, s),
            "New content never invents credit"
        );
    }
}
fn remove_exact(raw: &[u8], pos: usize, insert: &str) -> Vec<u8> {
    let end = pos.checked_add(insert.len()).unwrap();
    assert_eq!(raw.get(pos..end), Some(insert.as_bytes()));
    let mut old = raw[..pos].to_vec();
    old.extend_from_slice(&raw[end..]);
    old
}
#[test]
fn exact_two_unit_appends_preserve_original39_and_both_fixed_intermediates() {
    let optional: Value = serde_json::from_str(include_str!(
        "fixtures/shell-optional-inputs-registration.json"
    ))
    .unwrap();
    let combined: Value =
        serde_json::from_str(include_str!("fixtures/catalog160-registration.json")).unwrap();
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let reviewed41 = combined_catalog::before_cron_units(current_raw).unwrap();
    let raw = reviewed41.as_slice();
    assert_eq!(
        digest::hash(raw),
        combined["combined41_sha256"].as_str().unwrap()
    );
    let ether40 = remove_exact(
        raw,
        combined["optional_insert_byte_offset"].as_u64().unwrap() as usize,
        combined["optional_insert_utf8"].as_str().unwrap(),
    );
    assert_eq!(
        digest::hash(&ether40),
        combined["ether40_sha256"].as_str().unwrap()
    );
    let original39 = remove_exact(
        &ether40,
        combined["ether_insert_byte_offset"].as_u64().unwrap() as usize,
        combined["ether_insert_utf8"].as_str().unwrap(),
    );
    assert_eq!(
        digest::hash(&original39),
        combined["original39_sha256"].as_str().unwrap()
    );
    assert_eq!(
        digest::hash(&original39),
        optional["old_sha256"].as_str().unwrap()
    );
    // The independently reviewed Bash-only40 source remains byte-identical:
    // remove only the exact Ether insertion from the combined source.
    let canonical_bash40 = remove_exact(
        raw,
        combined["ether_insert_byte_offset"].as_u64().unwrap() as usize,
        combined["ether_insert_utf8"].as_str().unwrap(),
    );
    assert_eq!(
        digest::hash(&canonical_bash40),
        optional["new_sha256"].as_str().unwrap()
    );
    assert_eq!(
        remove_exact(
            &canonical_bash40,
            optional["insert_byte_offset"].as_u64().unwrap() as usize,
            optional["insert_utf8"].as_str().unwrap()
        ),
        original39
    );
    let old: Value = serde_json::from_slice(&original39).unwrap();
    assert_eq!(old["units"].as_array().unwrap().len(), 39);
    let mut expected = old.clone();
    expected["units"]
        .as_array_mut()
        .unwrap()
        .push(combined["ether_unit"].clone());
    expected["units"]
        .as_array_mut()
        .unwrap()
        .push(optional["unit"].clone());
    assert_eq!(serde_json::from_slice::<Value>(raw).unwrap(), expected);
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!(one_off_boundary::before_len(&units), 46);
    assert_eq!(value(&units[..39]), old["units"]);
    // Exercise the complete41→40→39→37 inverse against the immutable old
    // source evidence as well as the newer two-append source checks above.
    let original37 = combined_catalog::old_unit_bytes(raw).unwrap();
    let nfs: Value =
        serde_json::from_str(include_str!("fixtures/nfs-original147-source-index.json")).unwrap();
    assert_eq!(
        digest::hash(&original37),
        nfs["unit_original_sha256"].as_str().unwrap()
    );
    assert_eq!(
        original37.len() as u64,
        nfs["unit_original_bytes"].as_u64().unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&original37).unwrap(),
        nfs["units"]
    );
    assert_eq!(value(&units[39]), combined["ether_unit"]);
    assert_eq!(value(&units[40]), optional["unit"]);
    assert_eq!(
        json!(units[40].prerequisites),
        json!(["shell-scope", "shell-case-dispatch"])
    );
}

#[test]
fn closed_optional_inverse_rejects_missing_extra_reordered_or_changed_unit_bytes() {
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let reviewed41 = combined_catalog::before_cron_units(current_raw).unwrap();
    let raw = reviewed41.as_slice();
    let ether40 = combined_catalog::before_optional_units(raw).unwrap();
    assert!(combined_catalog::before_optional_units(&ether40).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(combined_catalog::before_optional_units(&extra).is_err());
    let mut edited: Value = serde_json::from_slice(raw).unwrap();
    edited["units"][40]["title"] = "Unreviewed optional unit".into();
    assert!(
        combined_catalog::before_optional_units(&serde_json::to_vec_pretty(&edited).unwrap())
            .is_err()
    );
    let mut reordered: Value = serde_json::from_slice(raw).unwrap();
    reordered["units"].as_array_mut().unwrap().swap(39, 40);
    assert!(
        combined_catalog::before_optional_units(&serde_json::to_vec_pretty(&reordered).unwrap())
            .is_err()
    );
    let mut changed_original = raw.to_vec();
    let pos = changed_original.iter().position(|b| *b == b'I').unwrap();
    changed_original[pos] = b'X';
    assert!(combined_catalog::before_optional_units(&changed_original).is_err());
}
