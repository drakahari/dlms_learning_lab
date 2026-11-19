use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    qos_workspace as w, workspace,
};
const IDS: [&str; 3] = [
    "ccna-qos-mark-boundary",
    "ccna-qos-queue-bursts",
    "ccna-qos-trusted-handover",
];
const BASE: &str = include_str!("../scenarios/starter-v1.json");
const ROUTES: &str = include_str!("../scenarios/ipv4-static-v1.json");
const RAW: &str = include_str!("../scenarios/qos-lessons-v1.json");
fn catalog() -> Vec<pack::Scenario> {
    pack::load_catalog(&[BASE, ROUTES, RAW]).unwrap()
}
fn def(s: &pack::Scenario) -> &w::Definition {
    let Goal::Qos { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn reference(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for action in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    state
}
#[test]
fn registered_three_follow_prerequisites_and_preserve_every238_lab_and64_unit() {
    let old = pack::accepted238().unwrap();
    let now = pack::accepted241().unwrap();
    assert_eq!((old.len(), now.len()), (238, 241));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..238]).unwrap()
    );
    let before = course::accepted238(&old).unwrap();
    let units = course::accepted241(&now).unwrap();
    assert_eq!((before.len(), units.len()), (64, 65));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(&units[..64]).unwrap()
    );
    let unit = units.iter().find(|u| u.id == "ccna-qos-hop").unwrap();
    assert_eq!(unit.labs, IDS);
}
#[test]
fn real_pack_loader_admits_matching_source_reference_and_challenge_goals() {
    for s in catalog()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
    {
        w::source(&s).unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let state = reference(&s);
        for challenge in [false, true] {
            let mut a = Attempt::new(&s, 1);
            a.challenge = challenge;
            workspace::prepare(&mut a);
            a.state = state.clone();
            assert!(progress::current_solved(&a, &s));
        }
        let mut wrong = state;
        let State::Qos { simulation } = &mut wrong else {
            panic!()
        };
        simulation
            .assignment
            .replace(&def(&s).initial, def(&s).initial.starter.clone())
            .unwrap();
        assert!(!engine::solved(&wrong, &s.goal));
    }
}
#[test]
fn actual_progress_save_resume_keeps_both_drafts_and_invalid_budget_then_selected_reset() {
    let all = catalog();
    let labs = all
        .iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect::<Vec<_>>();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        a.state = reference(s);
        let State::Qos { simulation } = &mut a.state else {
            panic!()
        };
        simulation
            .practice
            .replace(&def(s).example, def(s).example_worked.clone())
            .unwrap();
        simulation.practice.run(&def(s).example).unwrap();
        let mut partial = simulation.assignment.draft.clone();
        partial.high_buffer = 64;
        partial.standard_buffer = 64;
        simulation
            .assignment
            .replace(&def(s).initial, partial)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let mut got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::save(&path, &p).is_err());
    let kept = serde_json::to_value(&got.attempts[IDS[1]]).unwrap();
    got.attempts.insert(IDS[0].into(), Attempt::new(labs[0], 2));
    assert_eq!(serde_json::to_value(&got.attempts[IDS[1]]).unwrap(), kept);
    assert!(got.favorites.contains(IDS[0]));
    progress::save_with_catalog(&path, &got, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap(),
        serde_json::to_value(got).unwrap()
    );
}
#[test]
fn foreign_workspace_import_and_failed_save_leave_previous_bytes() {
    let all = catalog();
    let s = all.iter().find(|s| s.id == IDS[0]).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let before = std::fs::read(&path).unwrap();
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = "foreign terminal draft".into();
    assert!(progress::save_with_catalog(&path, &p, &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}
#[test]
fn malformed_sources_reject_without_weakening_initial_worked_or_context_validation() {
    assert!(pack::load_catalog(&[BASE, ROUTES, RAW]).is_ok());
    for n in 0..5 {
        let mut v: serde_json::Value = serde_json::from_str(RAW).unwrap();
        let s = &mut v["scenarios"][0];
        match n {
            0 => s["initial"]["simulation"]["origin"] = serde_json::json!("wrong-source"),
            1 => {
                s["goal"]["definition"]["initial"]["task"] = serde_json::json!("lost requirements")
            }
            2 => {
                s["goal"]["definition"]["example_worked"] =
                    s["goal"]["definition"]["example"]["starter"].clone()
            }
            3 => {
                s["initial"]["simulation"]["assignment"]["diagnostic"] =
                    serde_json::json!("x".repeat(513))
            }
            _ => {
                s["goal"]["definition"]["initial"]["requirements"][0]["kind"] =
                    serde_json::json!("HostCommand")
            }
        }
        assert!(
            pack::load_catalog(&[BASE, ROUTES, &v.to_string()]).is_err(),
            "{n}"
        );
    }
}
