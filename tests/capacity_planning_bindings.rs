//! Technical model fixtures, not authored/catalog lessons or prerequisite evidence.
use dlms_learning_lab::{
    capacity_planning_workspace as work,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
};
fn model() -> pack::Scenario {
    let d: work::Definition =
        serde_json::from_str(include_str!("fixtures/capacity-planning-workspace-v1.json")).unwrap();
    let ops: Vec<_> = d
        .reference
        .iter()
        .map(|a| format!("plan-action {}", serde_json::to_string(a).unwrap()))
        .collect();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[{
     "id":"capacity-planning-model-fixture","version":1,"track":"LPIC-2","topic":"Capacity planning","unit":"capacity-model-fixture","title":"Plan a supplied virtual volume","summary":"Technical integration fixture, not authored curriculum.","objective":"Meet the supplied evidence window, budget and reserve policy in every workload case.","difficulty":"Intermediate","minutes":20,"steps":["Review supplied observations and quotes.","Plan and test every workload case."],"hints":["Check arrival time as well as capacity."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Technical integration fixture; distinct authored lessons follow capability validation.","simplifications":"Supplied occupancy stocks and finite simulated upgrades, without host monitoring or storage changes.","initial":{"kind":"CapacityPlanning","simulation":work::Session::new(&d)},"goal":{"kind":"CapacityPlanning","definition":d},"solution":ops,"prerequisites":[]
    }]});
    pack::load(&serde_json::to_string(&raw).unwrap())
        .unwrap()
        .remove(0)
}
fn state(a: &mut Attempt) -> &mut work::Session {
    let State::CapacityPlanning { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn engine_grades_all_outcomes_and_edits_clear_success_without_changing_peer_state() {
    let s = model();
    let mut a = Attempt::new(&s, 1);
    assert!(!progress::current_solved(&a, &s));
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let Goal::CapacityPlanning { definition } = &s.goal else {
        panic!()
    };
    state(&mut a)
        .assignment
        .apply(
            &definition.initial,
            &work::Action::Offer {
                id: "costly".into(),
            },
        )
        .unwrap();
    assert!(!progress::current_solved(&a, &s));
    state(&mut a)
        .assignment
        .apply(&definition.initial, &work::Action::Test)
        .unwrap();
    let checks = engine::checks(&a.state, &s.goal);
    assert!(checks.iter().any(|r| !r.0 && r.1.contains("Budget:")));
    assert!(checks.iter().any(|r| r.0 && r.1.starts_with("Case ")));
    assert!(!engine::solved(&a.state, &s.goal));
}
#[test]
fn wrong_source_unsupported_actions_and_malformed_pack_never_claim_completion() {
    let s = model();
    let mut a = Attempt::new(&s, 1);
    for op in [
        "sh -c anything",
        "plan-action {}",
        "plan-action {\"kind\":\"shell\",\"input\":\"id\"}",
    ] {
        let before = serde_json::to_value(&a).unwrap();
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, op).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
    let mut bad = s.clone();
    let Goal::CapacityPlanning { definition } = &mut bad.goal else {
        panic!()
    };
    definition.reference.pop();
    assert!(work::source(&bad).is_err());
    let pack = pack::Pack {
        schema_version: 1,
        scenarios: vec![bad],
    };
    assert!(pack::load(&serde_json::to_string(&pack).unwrap()).is_err());
    a.scenario_version = 99;
    assert!(work::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn catalog_bound_save_resume_and_selected_reset_protect_practice_favorites_and_existing_attempts() {
    let s = model();
    let peer = pack::load(include_str!("../scenarios/starter-v1.json"))
        .unwrap()
        .remove(0);
    let mut a = Attempt::new(&s, 2);
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    state(&mut a).practice.inputs.reserve_kib = "still learning".into();
    state(&mut a).assignment.selected_case = 2;
    let peer_attempt = Attempt::new(&peer, 1);
    let peer_bytes = serde_json::to_value(&peer_attempt).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.attempts.insert(peer.id.clone(), peer_attempt);
    p.favorites.extend([s.id.clone(), peer.id.clone()]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let catalog = vec![peer.clone(), s.clone()];
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut restored = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[&s.id], &s));
    restored.clock = 4;
    restored.attempts.insert(s.id.clone(), Attempt::new(&s, 4));
    progress::save_with_catalog(&path, &restored, &catalog).unwrap();
    let reset = progress::load(&path, &catalog).unwrap();
    assert_eq!(reset.attempts[&s.id].state, s.initial);
    assert_eq!(
        serde_json::to_value(&reset.attempts[&peer.id]).unwrap(),
        peer_bytes
    );
    assert!(reset.favorites.contains(&s.id));
}
#[test]
fn foreign_namespace_and_forged_saved_quote_or_case_are_rejected() {
    let s = model();
    let mut a = Attempt::new(&s, 1);
    a.ipv4 = Some(dlms_learning_lab::ipv4::Lesson::from_legacy(
        "192.0.2.1".parse().unwrap(),
        24,
        0,
    ));
    assert!(work::context(&a, &s).is_err());
    a = Attempt::new(&s, 1);
    state(&mut a).assignment.inputs.offer = Some("unavailable".into());
    assert!(work::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    a = Attempt::new(&s, 1);
    state(&mut a).assignment.selected_case = 7;
    assert!(work::context(&a, &s).is_err());
}
