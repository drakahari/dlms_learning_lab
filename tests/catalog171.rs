//! Exact complete currentMain168 prefix plus three numeric validation assignments.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack, progress, script_teaching, workspace,
};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PIN: &str = "573d8d85745ef923b515673ebce518107dbdc19e6558906aae13f6b9f3b4519c";
const IDS: [&str; 3] = [
    "shell-numeric-retry",
    "shell-numeric-intake",
    "shell-numeric-publication",
];
fn recipe() -> Value {
    let raw = include_bytes!("fixtures/catalog171-registration.json");
    assert_eq!(digest::hash(raw), PIN);
    serde_json::from_slice(raw).unwrap()
}
fn value<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn old_catalog() -> Vec<pack::Scenario> {
    let r = recipe();
    let mut texts = Vec::new();
    for p in r["packs"].as_array().unwrap() {
        let n = p["path"].as_str().unwrap();
        assert!(n.starts_with("scenarios/") && std::path::Path::new(n).components().count() == 2);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(n);
        let meta = std::fs::symlink_metadata(&path).unwrap();
        assert!(
            meta.is_file()
                && !meta.file_type().is_symlink()
                && meta.len() <= pack::MAX_PACK_BYTES as u64
        );
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(bytes.len() as u64, p["bytes"].as_u64().unwrap());
        assert_eq!(digest::hash(&bytes), p["sha256"].as_str().unwrap());
        texts.push(String::from_utf8(bytes).unwrap());
    }
    assert_eq!(texts.len(), 41);
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
#[test]
fn exact168_full_definitions_order52_scripts384_cases_and52_examples_stay_unchanged() {
    let old = old_catalog();
    let all = pack::bundled().unwrap();
    assert_eq!((old.len(), one_off_boundary::before_len(&all)), (168, 176));
    assert_eq!(value(&all[..168]), value(&old));
    assert_eq!(
        value(
            &old.iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        recipe()["before_ids"]
    );
    let scripts = old
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 52);
    assert_eq!(
        scripts
            .iter()
            .map(|s| if let Goal::Script { cases } = &s.goal {
                cases.len()
            } else {
                0
            })
            .sum::<usize>(),
        384
    );
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 52);
    assert_eq!(
        all[168..171]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let raw = include_bytes!("../scenarios/shell-numeric-validation-v1.json");
    assert_eq!(digest::hash(raw), recipe()["canonical_numeric_sha256"]);
    assert_eq!(
        raw.len() as u64,
        recipe()["canonical_numeric_bytes"].as_u64().unwrap()
    );
    let added: pack::Pack = serde_json::from_slice(raw).unwrap();
    assert_eq!(value(&all[168..171]), value(&added.scenarios));
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|t| {
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all)
                .iter()
                .filter(|s| s.track == t)
                .count()
        }),
        [53, 40, 32, 57]
    );
    assert_eq!(["CCNA","LPIC-1","LPIC-2","Bash/Shell"].map(|t|one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&all).iter().filter(|s|s.track==t).count()),[53,40,36,57]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 40, 36, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all[..176].iter().filter(|s| s.track == t).count()),
        [53, 36, 32, 55]
    );
    for s in &all[168..171] {
        assert!(matches!(s.initial, State::Script { .. }) && s.teaching.is_some());
        let mut a = progress::Attempt::new(s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        assert!(!progress::current_solved(&a, s));
        assert_eq!(a.state, s.initial);
        assert_eq!(
            a.workspace.unwrap().script_learning.unwrap().view,
            script_teaching::View::Introduction
        );
    }
}
#[test]
fn exact44_append_recovers_all43_raw_objects_and_every_old_closed_inverse() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let r = recipe();
    let before = combined_catalog::before_numeric_units(raw).unwrap();
    assert_eq!(digest::hash(&before), r["before_units43_sha256"]);
    assert_eq!(
        before.len() as u64,
        r["before_units43_bytes"].as_u64().unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&before).unwrap(),
        r["before_units43"]
    );
    let b42 = combined_catalog::before_backup_units(&before).unwrap();
    assert_eq!(combined_catalog::before_backup_units(raw).unwrap(), b42);
    let b41 = combined_catalog::before_cron_units(&b42).unwrap();
    assert_eq!(combined_catalog::before_cron_units(raw).unwrap(), b41);
    let b40 = combined_catalog::before_optional_units(&b41).unwrap();
    let b39 = combined_catalog::before_lacp_units(&b40).unwrap();
    let b37 = combined_catalog::old_unit_bytes(&b39).unwrap();
    assert_eq!(combined_catalog::old_unit_bytes(raw).unwrap(), b37);
    assert_eq!(
        digest::hash(&b37),
        "e87bd9685cc69cef5a3c4929497323cee15fb44cc1518e7c16fec09c24bcc574"
    );
    let old = old_catalog();
    let all = pack::bundled().unwrap();
    let old_units = course::load(std::str::from_utf8(&before).unwrap(), &old).unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!(
        (old_units.len(), one_off_boundary::before_len(&units)),
        (43, 46)
    );
    assert_eq!(value(&units[..43]), value(&old_units));
    assert_eq!(value(&units[43]), r["numeric_unit"]);
    assert_eq!(units[43].labs, IDS.map(str::to_string));
}
#[test]
fn new_inverse_rejects_partial_extra_changed_old_new_reordered_and_missing_unit() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = combined_catalog::before_numeric_units(raw).unwrap();
    assert!(combined_catalog::before_numeric_units(&before).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(combined_catalog::before_numeric_units(&extra).is_err());
    for change in 0..4 {
        let mut v: Value = serde_json::from_slice(raw).unwrap();
        match change {
            0 => v["units"][0]["summary"] = "Changed old".into(),
            1 => v["units"][43]["summary"] = "Changed new".into(),
            2 => v["units"].as_array_mut().unwrap().swap(42, 43),
            _ => {
                let units = v["units"].as_array_mut().unwrap();
                let at = units
                    .iter()
                    .position(|u| u["id"] == "shell-numeric-validation")
                    .unwrap();
                assert_eq!(at, 43);
                units.remove(at);
            }
        }
        let b = serde_json::to_vec_pretty(&v).unwrap();
        assert!(combined_catalog::before_numeric_units(&b).is_err());
        assert!(combined_catalog::before_backup_units(&b).is_err());
    }
}
#[test]
fn old_explicit_profile_decode_preserves_every_answer_and_never_creates_numeric_credit() {
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    assert_eq!(
        digest::hash(raw),
        "88c07f20276ba1555eaad886ec2a58eff3d3b3cb3cef2bf3108be64a63bacd26"
    );
    let old = old_catalog();
    let all = pack::bundled().unwrap();
    let before = progress::decode(raw, &old).unwrap();
    let after = progress::decode(raw, &all).unwrap();
    assert_eq!(before.attempts.len(), 116);
    assert_eq!(value(&before), value(&after));
    for id in IDS {
        assert!(!after.attempts.contains_key(id));
        assert!(!after.favorites.contains(id));
    }
}
