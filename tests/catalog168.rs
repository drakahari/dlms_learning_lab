//! Exact complete Main164 prefix plus four distinct backup/recovery assignments.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    file_backup_teaching, pack, progress, workspace,
};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PIN: &str = "8e8b382e4546f3e48a7441281f4db9448b68bf9333b19c7acf20e882623d4c23";
const IDS: [&str; 4] = [
    "lpic2-backup-selective-recovery",
    "lpic2-backup-incremental-chain",
    "lpic2-backup-snapshot-branch",
    "lpic2-backup-recovery-handover",
];
fn recipe() -> Value {
    let raw = include_bytes!("fixtures/catalog168-registration.json");
    assert_eq!(digest::hash(raw), PIN);
    serde_json::from_slice(raw).unwrap()
}
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
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
    assert_eq!(texts.len(), 40);
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
#[test]
fn all_reviewed164_complete_definitions_precede_only_the_four_authored_backups() {
    let old = old_catalog();
    let all = pack::bundled().unwrap();
    assert_eq!((old.len(), one_off_boundary::before_len(&all)), (164, 176));
    assert_eq!(value(&all[..164]), value(&old));
    assert_eq!(
        value(
            &old.iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        recipe()["before_ids"]
    );
    assert_eq!(
        all[164..168]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let raw = include_bytes!("../scenarios/file-backup-v1.json");
    assert_eq!(
        digest::hash(raw),
        recipe()["canonical_backup_sha256"].as_str().unwrap()
    );
    assert_eq!(
        raw.len() as u64,
        recipe()["canonical_backup_bytes"].as_u64().unwrap()
    );
    let canonical: pack::Pack = serde_json::from_slice(raw).unwrap();
    assert_eq!(value(&all[164..168]), value(&canonical.scenarios));
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
    for s in &all[164..168] {
        assert!(
            matches!(s.initial, State::FileBackup { .. })
                && matches!(s.goal, Goal::FileBackup { .. })
        );
        assert!(
            s.file_backup_teaching.is_some()
                && s.cron_teaching.is_none()
                && s.etherchannel_teaching.is_none()
                && s.nfs_teaching.is_none()
        );
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        let mut attempt = progress::Attempt::new(s, 1);
        workspace::prepare(&mut attempt);
        file_backup_teaching::prepare(&mut attempt, s.file_backup_teaching.as_ref().unwrap(), true)
            .unwrap();
        assert!(!progress::current_solved(&attempt, s));
        assert_eq!(attempt.state, s.initial);
        assert_eq!(
            attempt
                .workspace
                .as_ref()
                .unwrap()
                .file_backup_learning
                .as_ref()
                .unwrap()
                .view,
            dlms_learning_lab::script_teaching::View::Introduction
        );
    }
}
#[test]
fn exact43_append_recovers_all42_bytes_objects_and_every_earlier_closed_inverse() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let previous = combined_catalog::before_backup_units(raw).unwrap();
    let r = recipe();
    assert_eq!(
        digest::hash(&previous),
        r["before_units42_sha256"].as_str().unwrap()
    );
    assert_eq!(
        previous.len() as u64,
        r["before_units42_bytes"].as_u64().unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&previous).unwrap(),
        r["before_units42"]
    );
    let before41 = combined_catalog::before_cron_units(&previous).unwrap();
    assert_eq!(combined_catalog::before_cron_units(raw).unwrap(), before41);
    let before40 = combined_catalog::before_optional_units(&before41).unwrap();
    let before39 = combined_catalog::before_lacp_units(&before40).unwrap();
    let before37 = combined_catalog::old_unit_bytes(&before39).unwrap();
    assert_eq!(combined_catalog::old_unit_bytes(raw).unwrap(), before37);
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let old = course::load(std::str::from_utf8(&previous).unwrap(), &old_catalog()).unwrap();
    assert_eq!((old.len(), one_off_boundary::before_len(&units)), (42, 46));
    assert_eq!(value(&units[..42]), value(&old));
    assert_eq!(value(&units[42]), r["backup_unit"]);
    assert_eq!(units[42].id, "backup-and-recovery");
    assert_eq!(units[42].prerequisites, ["linux-file-foundations"]);
}
#[test]
fn current43_inverse_refuses_missing_extra_old_change_new_change_and_reordering() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let previous = combined_catalog::before_backup_units(raw).unwrap();
    assert!(combined_catalog::before_backup_units(&previous).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(combined_catalog::before_backup_units(&extra).is_err());
    for mutation in 0..4 {
        let mut changed: Value = serde_json::from_slice(raw).unwrap();
        match mutation {
            0 => changed["units"][0]["title"] = "Changed original unit".into(),
            1 => changed["units"][42]["summary"] = "Changed backup scope".into(),
            2 => changed["units"].as_array_mut().unwrap().swap(41, 42),
            _ => {
                let units = changed["units"].as_array_mut().unwrap();
                let at = units
                    .iter()
                    .position(|u| u["id"] == "backup-and-recovery")
                    .unwrap();
                assert_eq!(at, 42);
                units.remove(at);
            }
        }
        let modified = serde_json::to_vec_pretty(&changed).unwrap();
        assert!(combined_catalog::before_backup_units(&modified).is_err());
        assert!(combined_catalog::before_cron_units(&modified).is_err());
    }
}
#[test]
fn new_registration_preserves_the_full_real_legacy_profile_and_creates_no_new_work() {
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    assert_eq!(
        digest::hash(raw),
        "88c07f20276ba1555eaad886ec2a58eff3d3b3cb3cef2bf3108be64a63bacd26"
    );
    let all = pack::bundled().unwrap();
    let before = progress::decode(raw, &old_catalog()).unwrap();
    let after = progress::decode(raw, &all).unwrap();
    assert_eq!(before.attempts.len(), 116);
    assert_eq!(value(&after), value(&before));
    for id in IDS {
        assert!(!after.attempts.contains_key(id));
        assert!(!after.favorites.contains(id));
    }
}
