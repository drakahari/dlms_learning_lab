use dlms_learning_lab::samba_share_workspace::{Definition, Session};
fn drafts() -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(include_str!(
        "../scenarios/samba-standalone-lessons-v1.json"
    ))
    .unwrap()["scenarios"]
        .as_array()
        .unwrap()
        .clone()
}
#[test]
fn authored_definitions_are_reachable_and_initial_sessions_match() {
    for s in drafts() {
        let d: Definition = serde_json::from_value(s["goal"]["definition"].clone()).unwrap();
        d.validate().unwrap_or_else(|e| panic!("{}: {e}", s["id"]));
        let a: Session = serde_json::from_value(s["initial"]["simulation"].clone()).unwrap();
        assert_eq!(a, d.new_session().unwrap());
        assert!(!a.assignment.solved(&d.assignment));
        for t in [&d.assignment, &d.example] {
            let failures: Vec<_> = t
                .checks(&t.reference)
                .into_iter()
                .filter(|c| !c.passed)
                .collect();
            assert!(failures.is_empty(), "{}: {failures:?}", t.id);
            assert!(t.checks(&t.initial).iter().any(|c| c.passed));
            assert!(t.checks(&t.initial).iter().any(|c| !c.passed));
        }
    }
}
#[test]
fn integrated_unit_preserves_every_455_scenario_and_181_unit_in_order() {
    use dlms_learning_lab::{course, pack};
    let before = pack::accepted455().unwrap();
    let now = pack::accepted458().unwrap();
    assert_eq!((before.len(), now.len()), (455, 458));
    let old: Vec<_> = now
        .iter()
        .filter(|s| !s.id.starts_with("lpic2-smb-"))
        .collect();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let oldunits = course::accepted455(&before).unwrap();
    let units = course::accepted458(&now).unwrap();
    assert_eq!((oldunits.len(), units.len()), (181, 182));
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "samba-standalone-shares")
        .collect();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(oldunits).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "samba-standalone-shares")
        .unwrap();
    assert_eq!(units[at - 1].id, "nfs-network-file-sharing");
    assert_eq!(now.iter().filter(|s| s.track == "LPIC-2").count(), 117);
}
#[test]
fn each_taught_guide_repair_passes_without_awarding_assignment_credit() {
    use dlms_learning_lab::{pack, samba_share_source as binding};
    for s in pack::load(include_str!(
        "../scenarios/samba-standalone-lessons-v1.json"
    ))
    .unwrap()
    {
        let d = binding::source(&s).unwrap();
        let mut a = d.new_session().unwrap();
        let p = &mut a.practice;
        assert!(
            p.command(&d.example, "id -Gn riley")
                .unwrap()
                .contains("project")
        );
        assert!(
            p.command(&d.example, "ls -ld /practice/notes")
                .unwrap()
                .contains("/practice/notes")
        );
        for user in &d.example.editable_accounts {
            p.account(&d.example, &user.id, true).unwrap();
        }
        p.command(&d.example, "chmod g+rx /practice/notes").unwrap();
        p.edit(
            &d.example,
            d.example.reference.saved_config().unwrap().into(),
        )
        .unwrap();
        p.save(&d.example).unwrap();
        p.publish(&d.example).unwrap();
        p.check(&d.example).unwrap();
        assert!(p.solved(&d.example), "{} {:?}", s.id, p.checks(&d.example));
        assert!(!a.assignment.solved(&d.assignment));
    }
}
