//! Full reviewed Main160 prefix plus exactly four substantive Cron assignments.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack, progress,
};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PIN: &str = "980ac05b935f9cbcc76bf3f1dff351de2e11dbad326ccf349f195f70c4f13072";
const IDS: [&str; 4] = [
    "lpic1-cron-weekday-report",
    "lpic1-cron-environment-output",
    "lpic1-cron-access-retirement",
    "lpic1-cron-calendar-handover",
];
fn recipe() -> Value {
    let raw = include_bytes!("fixtures/catalog164-registration.json");
    assert_eq!(digest::hash(raw), PIN);
    serde_json::from_slice(raw).unwrap()
}
fn old_catalog() -> Vec<pack::Scenario> {
    let r = recipe();
    let mut texts = Vec::new();
    for pin in r["packs"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        assert!(
            name.starts_with("scenarios/") && std::path::Path::new(name).components().count() == 2
        );
        let raw =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
        assert_eq!(raw.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(digest::hash(&raw), pin["sha256"].as_str().unwrap());
        texts.push(String::from_utf8(raw).unwrap());
    }
    assert_eq!(texts.len(), 39);
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
#[test]
fn every_reviewed160_definition_and_order_precedes_the_exact_four_authored_cron_tasks() {
    let old = old_catalog();
    let all = pack::bundled().unwrap();
    assert_eq!((old.len(), one_off_boundary::before_len(&all)), (160, 176));
    assert_eq!(value(&all[..160]), value(&old));
    assert_eq!(
        value(
            &old.iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        recipe()["before_ids"]
    );
    assert_eq!(
        all[160..164]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let raw = include_bytes!("../scenarios/linux-cron-v1.json");
    assert_eq!(
        digest::hash(raw),
        recipe()["canonical_cron_sha256"].as_str().unwrap()
    );
    let authored: pack::Pack = serde_json::from_slice(raw).unwrap();
    assert_eq!(value(&all[160..164]), value(&authored.scenarios));
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|track| {
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all)
                .iter()
                .filter(|s| s.track == track)
                .count()
        }),
        [53, 40, 32, 57]
    );
    assert_eq!(["CCNA","LPIC-1","LPIC-2","Bash/Shell"].map(|track|one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&all).iter().filter(|s|s.track==track).count()),[53,40,36,57]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&all).iter().filter(|s| s.track == track).count()),
        [57, 40, 36, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all.iter().filter(|s| s.track == track).count()),
        [57, 40, 36, 60]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all[..176].iter().filter(|s| s.track == track).count()),
        [53, 36, 32, 55]
    );
    for s in &all[160..164] {
        assert!(matches!(s.initial, State::Cron { .. }) && matches!(s.goal, Goal::Cron { .. }));
        assert!(
            s.cron_teaching.is_some()
                && s.etherchannel_teaching.is_none()
                && s.file_backup_teaching.is_none()
        );
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        let mut a = progress::Attempt::new(s, 1);
        dlms_learning_lab::cron_workspace::budget::prepare(&mut a, s).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert_eq!(a.state, s.initial);
    }
}
#[test]
fn exact_new42_inverse_retains_the_complete_reviewed41_then_every_prior_closed_layer() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let previous = combined_catalog::before_cron_units(raw).unwrap();
    assert_eq!(
        digest::hash(&previous),
        recipe()["before_units41_sha256"].as_str().unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&previous).unwrap(),
        recipe()["before_units41"]
    );
    let ether40 = combined_catalog::before_optional_units(&previous).unwrap();
    let old39 = combined_catalog::before_lacp_units(&ether40).unwrap();
    let old37 = combined_catalog::old_unit_bytes(&old39).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&old37).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!(one_off_boundary::before_len(&units), 46);
    let old = course::load(std::str::from_utf8(&previous).unwrap(), &old_catalog()).unwrap();
    assert_eq!(old.len(), 41);
    assert_eq!(value(&units[..41]), value(&old));
    assert_eq!(value(&units[41]), recipe()["cron_unit"]);
    assert_eq!(
        units[41].prerequisites,
        [
            "linux-file-foundations",
            "shell-foundations",
            "local-account-lifecycle"
        ]
    );
}
#[test]
fn new_inverse_refuses_missing_extra_changed_and_reordered_bytes_without_weakening_old_recipes() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let previous = combined_catalog::before_cron_units(raw).unwrap();
    assert!(combined_catalog::before_cron_units(&previous).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(combined_catalog::before_cron_units(&extra).is_err());
    let mut changed: Value = serde_json::from_slice(raw).unwrap();
    changed["units"][0]["title"] = "Changed old unit".into();
    assert!(
        combined_catalog::before_cron_units(&serde_json::to_vec_pretty(&changed).unwrap()).is_err()
    );
    let mut reordered: Value = serde_json::from_slice(raw).unwrap();
    reordered["units"].as_array_mut().unwrap().swap(40, 41);
    assert!(
        combined_catalog::before_cron_units(&serde_json::to_vec_pretty(&reordered).unwrap())
            .is_err()
    );
}
#[test]
fn genuine_old_profile_does_not_gain_any_new_cron_answers_attempts_favorites_or_credit() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let before: Value = serde_json::from_slice(raw).unwrap();
    let loaded = progress::decode(raw, &all).unwrap();
    assert_eq!(value(&loaded.attempts), before["attempts"]);
    assert_eq!(value(&loaded.favorites), before["favorites"]);
    assert_eq!(
        value(&loaded.migration_receipts),
        before["migration_receipts"]
    );
    for id in IDS {
        assert!(!loaded.attempts.contains_key(id) && !loaded.favorites.contains(id));
    }
    let portable = serde_json::to_vec(&loaded).unwrap();
    let reopened = progress::decode(&portable, &all).unwrap();
    assert_eq!(value(&reopened.attempts), value(&loaded.attempts));
    for id in IDS {
        assert!(!reopened.attempts.contains_key(id) && !reopened.favorites.contains(id));
    }
}
