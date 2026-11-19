use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    removable_workspace as w, workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const RAW: &str = include_str!("../scenarios/removable-mount-lessons-v1.json");
const IDS: [&str; 1] = ["lpic1-removable-startup-policy"];
fn catalog() -> Vec<pack::Scenario> {
    pack::load(RAW).unwrap()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::RemovableMounts { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::RemovableMounts { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 5);
    workspace::prepare(&mut a);
    a
}
fn reference(s: &pack::Scenario) -> String {
    s.solution[0].strip_prefix(w::PREFIX).unwrap().into()
}
fn apply(s: &pack::Scenario, text: &str) -> State {
    let mut state = s.initial.clone();
    engine::execute_for_goal(&mut state, &s.goal, &format!("{}{text}", w::PREFIX)).unwrap();
    state
}
#[test]
fn drafts_current_credit_backup_restore_and_selected_reset_survive_safe_writes() {
    let mut all = catalog();
    all.push(
        pack::load(include_str!("../scenarios/starter-v1.json"))
            .unwrap()
            .remove(0),
    );
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = fresh(&all[0]);
    a.state = apply(&all[0], &reference(&all[0]));
    let d = definition(&all[0]);
    session(&mut a).practice.replace(&d.example_worked).unwrap();
    session(&mut a).view = dlms_learning_lab::script_teaching::View::Example;
    p.attempts.insert(all[0].id.clone(), a);
    p.favorites.insert(all[0].id.clone());
    p.attempts.insert(all[1].id.clone(), fresh(&all[1]));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&got.attempts[&all[0].id], &all[0]));
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &all).unwrap();
    let restore =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let receipt = backup::restore(&path, &restore, &all).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    let kept = serde_json::to_value(&p.attempts[&all[1].id]).unwrap();
    p.attempts.insert(IDS[0].into(), fresh(&all[0]));
    assert_eq!(serde_json::to_value(&p.attempts[&all[1].id]).unwrap(), kept);
    assert!(p.favorites.contains(IDS[0]));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], &all[0]));
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let decoded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
#[test]
fn preserve247_labs68_units_and_add_one_state_based_removable_capstone() {
    let prior = pack::accepted247().unwrap();
    let now = pack::accepted248().unwrap();
    assert_eq!((prior.len(), now.len()), (247, 248));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..247]).unwrap()
    );
    let before = course::accepted247(&prior).unwrap();
    let units = course::accepted248(&now).unwrap();
    assert_eq!((before.len(), units.len()), (68, 69));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(&units[..68]).unwrap()
    );
    assert_eq!(units.last().unwrap().labs, IDS);
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["linux-disk-preparation"]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| now.iter().filter(|s| s.track == track).count()),
        [72, 67, 43, 66]
    );
}
#[test]
fn authored_taught_and_alternate_policy_check_actual_mounts_and_expected_required_failure() {
    for s in catalog() {
        let d = w::source(&s).unwrap();
        let raw = reference(&s);
        assert!(!engine::solved(&s.initial, &s.goal));
        for text in [
            raw.clone(),
            raw.replace("rw,auto", "defaults")
                .replace("ro,auto,nofail", "nofail,ro"),
            format!("# Equivalent policy\n{}", raw.replace(' ', "\t")),
        ] {
            let state = apply(&s, &text);
            assert!(
                engine::solved(&state, &s.goal),
                "{:?}",
                engine::checks(&state, &s.goal)
            );
        }
        let rows = d.initial.run(&raw).unwrap();
        assert!(rows.iter().all(w::CaseResult::passed));
        let r = rows[2].projection.as_ref().unwrap();
        assert!(!r.required_attempts_satisfied());
        assert!(r.state.live().contains_key("/media/audit"));
        assert!(!r.state.live().contains_key("/project"));
        for challenge in [false, true] {
            let mut a = fresh(&s);
            a.challenge = challenge;
            a.state = apply(&s, &raw);
            assert!(progress::current_solved(&a, &s));
            session(&mut a)
                .assignment
                .replace(&d.initial.starter)
                .unwrap();
            assert!(!progress::current_solved(&a, &s));
        }
    }
}
#[test]
fn wrong_uuid_partial_case_empty_noauto_nofail_and_syntax_cannot_claim_current_success() {
    let s = &catalog()[0];
    let d = definition(s);
    let raw = reference(s);
    for wrong in [
        d.initial.starter.clone(),
        String::new(),
        raw.replace("000000000091", "000000000099"),
        raw.replace("rw,auto", "rw,auto,nofail"),
        raw.replace("ro,auto,nofail", "rw,noauto"),
        raw.replace("/media/audit ext4", "/media/audit xfs"),
        "$(cat /etc/shadow)".into(),
    ] {
        let state = apply(s, &wrong);
        assert!(!engine::solved(&state, &s.goal), "{wrong}");
        assert!(engine::checks(&state, &s.goal).iter().any(|r| !r.0));
    }
    let rows = d
        .initial
        .run(&raw.replace("000000000091", "000000000099"))
        .unwrap();
    assert_eq!(
        rows.iter().map(w::CaseResult::passed).collect::<Vec<_>>(),
        [false, true, false]
    );
}
#[test]
fn exact_guided_experiment_reports_the_required_policy_mistake_then_repairs_without_credit() {
    let s = &catalog()[0];
    let d = definition(s);
    let mut sim = w::Session::new(d).unwrap();
    let initial = sim.assignment.clone();
    sim.practice.replace(&d.example_worked).unwrap();
    assert!(
        sim.practice
            .run(&d.example)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    assert!(!d.solved(&sim));
    let wrong = d.example_worked.replace("rw,auto", "rw,auto,nofail");
    sim.practice.replace(&wrong).unwrap();
    let rows = sim.practice.run(&d.example).unwrap();
    assert!(rows.iter().all(|r| !r.passed()));
    assert!(
        rows[2]
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("Required-storage"))
    );
    assert!(
        rows[2]
            .projection
            .as_ref()
            .unwrap()
            .required_attempts_satisfied()
    );
    sim.practice.replace(&d.example_worked).unwrap();
    assert!(
        sim.practice
            .run(&d.example)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    assert_eq!(sim.assignment, initial);
}
