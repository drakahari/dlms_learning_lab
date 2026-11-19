use dlms_learning_lab::{
    backup, course,
    engine::{self, State},
    pack, pam_workspace as w,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const ID: &str = "lpic2-pam-auth-bypass";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted343().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::PamStack { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 9);
    workspace::prepare(&mut a);
    w::context(&a, lab()).unwrap();
    a
}
#[test]
fn all342_previous_scenarios_and120_units_are_exact_with_one_distinct_auth_task() {
    let old = pack::accepted342().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (342, 343));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(now.iter().filter(|s| s.id != ID).collect::<Vec<_>>()).unwrap()
    );
    let before = course::accepted342(&old).unwrap();
    let units = course::accepted343(now).unwrap();
    assert_eq!((before.len(), units.len()), (120, 121));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-authentication-stacks")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let index = units
        .iter()
        .position(|u| u.id == "linux-authentication-stacks")
        .unwrap();
    assert_eq!(units[index - 1].id, "local-account-lifecycle");
    assert_eq!(units[index].labs, [ID]);
    assert_eq!(lab().prerequisites, ["lpic1-account-team-handover"]);
    for (track, count) in [
        ("CCNA", 74),
        ("LPIC-1", 114),
        ("LPIC-2", 78),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn taught_and_alternate_stacks_pass_while_wrong_partial_and_unsupported_fail() {
    let d = w::source(lab()).unwrap();
    let starter = d.initial.run(&d.initial.starter).unwrap();
    assert_eq!(starter.iter().filter(|r| r.passed()).count(), 1);
    assert!(starter[1].feedback().contains("before checking"));
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    assert!(progress::current_solved(&a, lab()));
    for valid in [
        "auth required pam_unix.so",
        "auth sufficient pam_unix.so\nauth required pam_deny.so",
        "# another approach\nAUTH ReQuIsItE pam_unix.so\nauth optional pam_deny.so",
    ] {
        sim(&mut a).assignment.edit(valid).unwrap();
        assert!(!progress::current_solved(&a, lab()));
        sim(&mut a).assignment.run(&d.initial).unwrap();
        assert!(progress::current_solved(&a, lab()));
    }
    for wrong in [
        "auth required pam_permit.so",
        "auth required pam_deny.so",
        "auth optional pam_unix.so\nauth required pam_permit.so",
        "auth",
        "auth required pam_unix.so nullok",
        "auth required ../../pam_unix.so",
    ] {
        sim(&mut a).assignment.edit(wrong).unwrap();
        let rows = sim(&mut a).assignment.run(&d.initial).unwrap();
        assert!(rows.iter().any(|r| !r.passed()));
        assert!(!progress::current_solved(&a, lab()));
    }
}
#[test]
fn practice_reset_and_retry_leave_independent_work_ungraded_and_separate() {
    let d = w::source(lab()).unwrap();
    let mut a = fresh();
    let old = sim(&mut a).assignment.clone();
    sim(&mut a).practice.edit(&d.example_worked).unwrap();
    sim(&mut a).practice.run(&d.example).unwrap();
    assert_eq!(sim(&mut a).assignment, old);
    assert!(!progress::current_solved(&a, lab()));
    sim(&mut a).practice = w::Session::new(d).unwrap().practice;
    assert_eq!(sim(&mut a).assignment, old);
    assert_eq!(sim(&mut a).practice.tested, None);
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    assert!(progress::current_solved(&a, lab()));
    sim(&mut a)
        .assignment
        .edit("# unfinished changed draft")
        .unwrap();
    assert!(!progress::current_solved(&a, lab()));
    assert!(!d.checks(sim(&mut a))[0].0);
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    assert!(progress::current_solved(&a, lab()));
}
#[test]
fn real_storage_export_restore_and_selected_reset_preserve_an_old_bridge_draft() {
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    sim(&mut a)
        .practice
        .edit("# unfinished practice\nauth optional pam_unix.so")
        .unwrap();
    let old_source = catalog()
        .iter()
        .find(|s| s.id == "lpic2-bridge-lan-commission")
        .unwrap();
    let mut old = Attempt::new(old_source, 4);
    workspace::prepare(&mut old);
    let State::NmProfiles { simulation } = &mut old.state else {
        panic!()
    };
    simulation
        .assignment
        .edit("bridge.nmconnection", "# retained unfinished bridge")
        .unwrap();
    assert!(!progress::current_solved(&old, old_source));
    let old_bytes = serde_json::to_value(&old).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(ID.into(), a);
    p.attempts.insert(old_source.id.clone(), old);
    p.favorites.extend([ID.into(), old_source.id.clone()]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[ID], lab()));
    let exported = dir.path().join("backup.json");
    backup::export(&exported, &p, catalog()).unwrap();
    let checked = backup::inspect(
        &exported,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let receipt = backup::restore(&path, &checked, catalog()).unwrap();
    assert_eq!(std::fs::read(receipt.safety_copy.unwrap()).unwrap(), bytes);
    p.attempts.insert(ID.into(), fresh());
    assert!(!progress::current_solved(&p.attempts[ID], lab()));
    assert_eq!(
        serde_json::to_value(&p.attempts[&old_source.id]).unwrap(),
        old_bytes
    );
    assert!(p.favorites.contains(ID) && p.favorites.contains(&old_source.id));
}
