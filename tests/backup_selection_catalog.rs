use dlms_learning_lab::{course, pack};
#[test]
fn selection_appends_one_authored_capstone_and_preserves_all241_labs_and65_units() {
    let before = pack::accepted241().unwrap();
    let after = pack::accepted242().unwrap();
    assert_eq!((before.len(), after.len()), (241, 242));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&after[..241]).unwrap()
    );
    let old_units = course::accepted241(&before).unwrap();
    let units = course::accepted242(&after).unwrap();
    assert_eq!((old_units.len(), units.len()), (65, 66));
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(&units[..65]).unwrap()
    );
    let s = after.last().unwrap();
    assert_eq!(s.id, "lpic2-backup-directory-selection");
    assert_eq!(s.prerequisites, ["lpic2-backup-recovery-handover"]);
    let u = units.last().unwrap();
    assert_eq!(u.id, "backup-selection");
    assert_eq!(u.prerequisites, ["backup-and-recovery"]);
    assert_eq!(u.labs.as_slice(), std::slice::from_ref(&s.id));
    assert_eq!(
        after
            .iter()
            .filter(|s| s.file_backup_teaching.is_some())
            .count(),
        5
    );
}
