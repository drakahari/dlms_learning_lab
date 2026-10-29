//! Preserve the independently pinned standalone Cron158/40 proof inside combined164.
//! Not a claim of runtime acceptance until this source is composed and gated.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack, progress,
};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "support/cron_registration.rs"]
mod cron_registration;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const IDS: [&str; 4] = [
    "lpic1-cron-weekday-report",
    "lpic1-cron-environment-output",
    "lpic1-cron-access-retirement",
    "lpic1-cron-calendar-handover",
];
fn private_catalog() -> Vec<pack::Scenario> {
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 171);
    assert_eq!(
        all[154..160]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "ccna-lacp-initiation",
            "ccna-lacp-member-service",
            "ccna-lacp-resilient-handover",
            "shell-optional-policy",
            "shell-optional-report",
            "shell-optional-handover"
        ]
    );
    all[..154]
        .iter()
        .chain(all[160..164].iter())
        .cloned()
        .collect()
}
fn old_catalog() -> Vec<pack::Scenario> {
    let i = cron_registration::index();
    let mut data = Vec::new();
    for pin in i["packs"].as_array().unwrap() {
        let n = pin["path"].as_str().unwrap();
        assert!(n.starts_with("scenarios/") && !n.contains(".."));
        let b = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(n)).unwrap();
        assert_eq!(b.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(digest::hash(&b), pin["sha256"].as_str().unwrap());
        data.push(String::from_utf8(b).unwrap());
    }
    assert_eq!(data.len(), 37);
    pack::load_catalog(&data.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    )
}
#[test]
fn complete_original154_specs_and_order_are_exact_before_four_distinct_cron_tasks() {
    let old = old_catalog();
    let all = private_catalog();
    assert_eq!((old.len(), all.len()), (154, 158));
    exact(&all[..154].to_vec(), &old);
    assert_eq!(
        all[154..158]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let raw = include_bytes!("../scenarios/linux-cron-v1.json");
    assert_eq!(
        digest::hash(raw),
        cron_registration::recipe()["canonical_pack_sha256"]
            .as_str()
            .unwrap()
    );
    let p: pack::Pack = serde_json::from_slice(raw).unwrap();
    exact(&all[154..158].to_vec(), &p.scenarios);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [45, 36, 28, 49]
    );
    assert!(
        all[154..158]
            .iter()
            .all(|s| matches!(s.initial, State::Cron { .. })
                && matches!(s.goal, Goal::Cron { .. })
                && s.cron_teaching.is_some())
    );
}
#[test]
fn raw40_inverse_recovers_full_raw39_then_existing_immutable_raw37_recipe() {
    let all = private_catalog();
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let private_raw = cron_registration::private_cron_units(current_raw).unwrap();
    let raw = private_raw.as_slice();
    let old = cron_registration::previous_units(raw).unwrap();
    let prior: Value = serde_json::from_slice(&old).unwrap();
    assert_eq!(prior, cron_registration::index()["units"]);
    let original37 = combined_catalog::old_unit_bytes(&old).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&original37).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
    let units = course::load(std::str::from_utf8(raw).unwrap(), &all).unwrap();
    let old_units = course::load(std::str::from_utf8(&old).unwrap(), &old_catalog()).unwrap();
    assert_eq!((old_units.len(), units.len()), (39, 40));
    exact(&units[..39].to_vec(), &old_units);
    exact(&units[39], &cron_registration::recipe()["unit"]);
    assert_eq!(
        units[39].prerequisites,
        [
            "linux-file-foundations",
            "shell-foundations",
            "local-account-lifecycle"
        ]
    );
}
#[test]
fn inverse_refuses_missing_append_changed_prefix_reordered_units_and_trailing_bytes() {
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let private_raw = cron_registration::private_cron_units(current_raw).unwrap();
    let raw = private_raw.as_slice();
    let prior = cron_registration::previous_units(raw).unwrap();
    assert!(cron_registration::previous_units(&prior).is_err());
    let mut bad = raw.to_vec();
    bad.push(b' ');
    assert!(cron_registration::previous_units(&bad).is_err());
    let text = std::str::from_utf8(raw)
        .unwrap()
        .replacen("IPv4", "IPvX", 1);
    assert!(cron_registration::previous_units(text.as_bytes()).is_err());
    let mut changed: Value = serde_json::from_slice(raw).unwrap();
    changed["units"].as_array_mut().unwrap().swap(38, 39);
    assert!(
        cron_registration::previous_units(&serde_json::to_vec_pretty(&changed).unwrap()).is_err()
    );
}
#[test]
fn actual8_decode_never_invents_new_cron_answers_favorites_or_credit() {
    let all = private_catalog();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let p = progress::decode(raw, &all).unwrap();
    for id in IDS {
        assert!(!p.attempts.contains_key(id) && !p.favorites.contains(id));
    }
    let old = serde_json::to_value(&p.attempts).unwrap();
    let q = progress::decode(&serde_json::to_vec(&p).unwrap(), &all).unwrap();
    exact(&q.attempts, &old);
    for id in IDS {
        assert!(!q.attempts.contains_key(id) && !q.favorites.contains(id));
    }
}
