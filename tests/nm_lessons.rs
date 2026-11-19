use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    nm_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const RAW: &str = include_str!("../scenarios/linux-nm-lessons-v1.json");
const IDS: [&str; 3] = [
    "lpic1-nm-persistent-profile",
    "lpic1-nm-autoconnect-choice",
    "lpic1-nm-uplink-handover",
];
fn catalog() -> Vec<pack::Scenario> {
    pack::load(RAW).unwrap()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::NmProfiles { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::NmProfiles { simulation } = &mut a.state else {
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
fn solution(s: &pack::Scenario) -> w::Files {
    serde_json::from_str(&reference(s)).unwrap()
}
fn apply(s: &pack::Scenario, text: &str) -> State {
    let mut state = s.initial.clone();
    engine::execute_for_goal(&mut state, &s.goal, &format!("{}{text}", w::PREFIX)).unwrap();
    state
}
#[test]
fn drafts_current_credit_backup_restore_and_selected_reset_survive_safe_writes() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = fresh(s);
        a.state = apply(s, &reference(s));
        let d = definition(s);
        session(&mut a).practice.replace(&d.example_worked).unwrap();
        session(&mut a).view = dlms_learning_lab::script_teaching::View::Example;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
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
    for s in &all {
        assert!(progress::current_solved(&got.attempts[&s.id], s));
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &all).unwrap();
    let restore =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let receipt = backup::restore(&path, &restore, &all).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    let kept = serde_json::to_value(&p.attempts[IDS[1]]).unwrap();
    p.attempts.insert(IDS[0].into(), fresh(&all[0]));
    assert_eq!(serde_json::to_value(&p.attempts[IDS[1]]).unwrap(), kept);
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
fn preserve244_previous_labs_67_units_and_register_three_distinct_persistent_tasks() {
    let before = pack::accepted244().unwrap();
    let now = pack::accepted247().unwrap();
    assert_eq!((before.len(), now.len()), (244, 247));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&now[..244]).unwrap()
    );
    let prior = course::accepted244(&before).unwrap();
    let units = course::accepted247(&now).unwrap();
    assert_eq!((prior.len(), units.len()), (67, 68));
    assert_eq!(
        serde_json::to_value(prior).unwrap(),
        serde_json::to_value(&units[..67]).unwrap()
    );
    assert_eq!(units.last().unwrap().labs, IDS);
    assert_eq!(now[245].prerequisites, [IDS[0]]);
    assert_eq!(now[246].prerequisites, [IDS[1]]);
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["linux-host-networking"]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [72, 66, 43, 66]
    );
}
#[test]
fn authored_references_alternatives_and_both_modes_pass_state_outcomes() {
    for s in catalog() {
        let d = w::source(&s).unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let correct = solution(&s);
        let mut alternate = correct.clone();
        if s.id == IDS[1] {
            alternate = d.initial.starter.clone();
            let old = alternate.get_mut("app.nmconnection").unwrap();
            *old = old.replace("autoconnect=true", "autoconnect=false");
        } else {
            if s.id == IDS[2] {
                let management = alternate.get_mut("management.nmconnection").unwrap();
                *management = management
                    .replace("never-default=true\n", "")
                    .replace("route-metric=100", "route-metric=200");
            }
            let app = alternate.get_mut("app.nmconnection").unwrap();
            *app = app
                .replace("route-metric=100", "route-metric=40")
                .replace("route-metric=50", "route-metric=40");
        }
        for files in [&correct, &alternate] {
            let state = apply(&s, &serde_json::to_string(files).unwrap());
            assert!(
                engine::solved(&state, &s.goal),
                "{} {:?}",
                s.id,
                engine::checks(&state, &s.goal)
            );
        }
        for challenge in [false, true] {
            let mut a = fresh(&s);
            a.challenge = challenge;
            a.state = apply(&s, &reference(&s));
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
fn common_wrong_states_fail_and_repair_while_examples_never_award_assignment_credit() {
    for s in catalog() {
        let d = definition(&s);
        let mut sim = w::Session::new(d).unwrap();
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert!(!d.solved(&sim));
        let correct = solution(&s);
        let mut wrong = correct.clone();
        let p = wrong.get_mut("app.nmconnection").unwrap();
        *p = p.replace("autoconnect=true", "autoconnect=false");
        if s.id == IDS[1] {
            let p = wrong.get_mut("replacement.nmconnection").unwrap();
            *p = p.replace("autoconnect=true", "autoconnect=false");
        }
        for files in [&d.initial.starter, &wrong] {
            let state = apply(&s, &serde_json::to_string(files).unwrap());
            assert!(!engine::solved(&state, &s.goal), "{}", s.id);
            assert!(engine::checks(&state, &s.goal).iter().any(|r| !r.0));
        }
        let mut bad = correct.clone();
        bad.insert("app.nmconnection".into(), "$(cat /etc/shadow)".into());
        let state = apply(&s, &serde_json::to_string(&bad).unwrap());
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn exact_taught_experiments_fail_for_their_stated_reason_and_can_be_repaired() {
    for (n, s) in catalog().into_iter().enumerate() {
        let d = definition(&s);
        let mut sim = w::Session::new(d).unwrap();
        sim.practice.replace(&d.example_worked).unwrap();
        sim.practice.run(&d.example).unwrap();
        let mut changed = d.example_worked.clone();
        let (name, from, to) = match n {
            0 => ("app.nmconnection", "autoconnect=true", "autoconnect=false"),
            1 => (
                "replacement.nmconnection",
                "autoconnect-priority=30",
                "autoconnect-priority=20",
            ),
            _ => (
                "management.nmconnection",
                "route1=10.40.0.0/24,10.85.0.1,100\n",
                "",
            ),
        };
        let text = changed.get_mut(name).unwrap();
        assert!(text.contains(from));
        *text = text.replace(from, to);
        sim.practice.replace(&changed).unwrap();
        let rows = sim.practice.run(&d.example).unwrap();
        assert!(rows.iter().all(|r| !r.passed()));
        if n == 1 {
            assert!(rows.iter().all(|r| {
                r.diagnostic
                    .as_ref()
                    .is_some_and(|s| s.contains("timestamp/history"))
            }));
        }
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert_eq!(sim.assignment.draft, d.initial.starter);
    }
}
