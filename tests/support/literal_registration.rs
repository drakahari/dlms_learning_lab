//! Exact199→197/53→52 developer predecessor. Never learner grading or mastery.
#![allow(dead_code)] // Each test/native binary imports different closed subsets.
use dlms_learning_lab::developer_digest as digest;
use dlms_learning_lab::{
    course::Unit,
    engine::State,
    pack::{Pack, Scenario},
    progress::Attempt,
};
use serde_json::Value;
use std::{collections::BTreeMap, sync::OnceLock};
pub const IDS: [&str; 2] = ["shell-trim-options", "shell-trim-manifest"];
fn recipe() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        let raw = include_bytes!("../fixtures/catalog199-registration.json");
        assert_eq!(
            digest::hash(raw),
            "c04649d4a8c0e3b25f41d68af3e5e5f5ae3a7a287190cbf0417ee2c60b7c15e0"
        );
        serde_json::from_slice(raw).unwrap()
    })
}
pub fn original197() -> Vec<Scenario> {
    let mut old = super::original193();
    let r: Value =
        serde_json::from_slice(include_bytes!("../fixtures/catalog197-registration.json")).unwrap();
    let raw = include_bytes!("../../scenarios/disk-preparation-v1.json");
    assert_eq!(
        digest::hash(raw),
        r["canonical_disk_pack_sha256"].as_str().unwrap()
    );
    let pack: Pack = serde_json::from_slice(raw).unwrap();
    assert_eq!(pack.scenarios.len(), 4);
    old.extend(pack.scenarios);
    assert_eq!(old.len(), 197);
    old
}
fn old_values() -> &'static Vec<Value> {
    static V: OnceLock<Vec<Value>> = OnceLock::new();
    V.get_or_init(|| {
        original197()
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect()
    })
}
fn new_sources() -> &'static Vec<Scenario> {
    static S: OnceLock<Vec<Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        let raw = include_bytes!("../../scenarios/shell-literal-normalization-draft.json");
        assert_eq!(
            raw.len() as u64,
            recipe()["canonical_literal_pack_bytes"].as_u64().unwrap()
        );
        assert_eq!(
            digest::hash(raw),
            recipe()["canonical_literal_pack_sha256"].as_str().unwrap()
        );
        let p: Pack = serde_json::from_slice(raw).unwrap();
        assert_eq!(
            p.scenarios
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        p.scenarios
    })
}
fn new_values() -> &'static Vec<Value> {
    static V: OnceLock<Vec<Value>> = OnceLock::new();
    V.get_or_init(|| {
        new_sources()
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect()
    })
}
pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    assert_eq!(all.len(), 199);
    for (actual, expected) in all.iter().zip(old_values().iter().chain(new_values())) {
        assert!(
            super::serialized_equal(actual, expected),
            "Complete source differs: {}",
            actual.id
        );
    }
    &all[..197]
}
pub fn prefix_scenarios(all: &[Scenario]) -> Option<&[Scenario]> {
    (all.len() == 199).then(|| before_scenarios(all))
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let r = recipe();
    if raw.len() as u64 != r["after_units53_bytes"].as_u64().unwrap()
        || digest::hash(raw) != r["after_units53_sha256"].as_str().unwrap()
    {
        return Err("Exact53 raw unit bytes differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().unwrap())
        .ok_or("Exact literal unit append differs")?;
    let old = format!("{head}{}", r["old_tail_utf8"].as_str().unwrap()).into_bytes();
    if old != include_bytes!("../fixtures/literal-units-before197.json")
        || old.len() as u64 != r["before_units52_bytes"].as_u64().unwrap()
        || digest::hash(&old) != r["before_units52_sha256"].as_str().unwrap()
    {
        return Err("Complete previous52 raw bytes differ".into());
    }
    let mut expected = r["before_units52"].clone();
    expected["units"]
        .as_array_mut()
        .unwrap()
        .push(r["literal_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact appended unit".into());
    }
    Ok(old)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    if raw.len() as u64 == recipe()["after_units53_bytes"].as_u64().unwrap()
        && digest::hash(raw) == recipe()["after_units53_sha256"].as_str().unwrap()
    {
        before_units(raw).map(Some)
    } else {
        Ok(None)
    }
}
pub fn before_typed_units(all: &[Unit]) -> &[Unit] {
    assert_eq!(all.len(), 53);
    for (actual, expected) in all.iter().zip(
        recipe()["before_units52"]["units"]
            .as_array()
            .unwrap()
            .iter()
            .chain(std::iter::once(&recipe()["literal_unit"])),
    ) {
        assert!(
            super::serialized_equal(actual, expected),
            "Complete unit differs: {}",
            actual.id
        );
    }
    &all[..52]
}
pub fn prefix_typed_units(all: &[Unit]) -> Option<&[Unit]> {
    (all.len() == 53).then(|| before_typed_units(all))
}
pub fn before_values(all: &[Value]) -> &[Value] {
    assert_eq!(all.len(), 53);
    assert_eq!(
        &all[..52],
        recipe()["before_units52"]["units"].as_array().unwrap()
    );
    assert_eq!(&all[52], &recipe()["literal_unit"]);
    &all[..52]
}
pub fn prefix_values(all: &[Value]) -> Option<&[Value]> {
    (all.len() == 53).then(|| before_values(all))
}
/// Count only; validates every key/version and both bounded Script contexts.
pub fn before_attempts_len(all: &BTreeMap<String, Attempt>) -> usize {
    assert_eq!(all.len(), 199);
    for v in old_values().iter().chain(new_values()) {
        assert_eq!(
            all[v["id"].as_str().unwrap()].scenario_version as u64,
            v["version"].as_u64().unwrap()
        );
    }
    for s in new_sources() {
        let a = &all[&s.id];
        assert!(matches!(a.state, State::Script { .. }));
        assert!(a.ipv4.is_none());
        assert!(a.step < s.steps.len() && a.hints <= s.hints.len());
        dlms_learning_lab::pack::validate_state(&a.state).unwrap();
        assert!(a.workspace.as_ref().is_none_or(|w| w.valid()));
    }
    197
}
