#[path = "support/guest_driver_source_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::{self, Goal, State},
    guest_driver_source as binding, guest_driver_workspace as work, pack,
    progress::{self, Attempt, Progress},
};

fn session(a: &mut Attempt) -> &mut work::Session {
    let State::GuestDrivers { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn typed_source_actions_grade_outcomes_and_source_flags_cannot_hide_broken_image() {
    let s = fixture::scenario();
    let mut a = Attempt::new(&s, 1);
    binding::context(&a, &s).unwrap();
    assert!(!progress::current_solved(&a, &s));
    for line in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let d = binding::source(&s).unwrap();
    session(&mut a)
        .assignment
        .runtime
        .remove_object(&d.assignment.facts, "lab_sensor")
        .unwrap();
    assert!(session(&mut a).assignment.checked);
    assert!(!progress::current_solved(&a, &s));
    let encoded = serde_json::json!({"schema_version":1,"scenarios":[s]});
    assert_eq!(pack::load(&encoded.to_string()).unwrap().len(), 1);
}
#[test]
fn ordinary_source_save_load_preserves_independent_drafts_favorites_and_uncredited_example() {
    let s = fixture::scenario();
    let d = binding::source(&s).unwrap();
    let mut a = Attempt::new(&s, 1);
    for action in &d.example_steps {
        session(&mut a).action(d, action, true).unwrap();
    }
    session(&mut a).assignment.input = "modinfo lab_sensor".into();
    session(&mut a).practice.input = "uname -r".into();
    let expected = session(&mut a).clone();
    assert!(!progress::current_solved(&a, &s));
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("progress-v1.json");
    progress::save_with_catalog(&file, &p, std::slice::from_ref(&s)).unwrap();
    let mut restored = progress::load(&file, std::slice::from_ref(&s)).unwrap();
    assert!(restored.favorites.contains(&s.id));
    let a = restored.attempts.get_mut(&s.id).unwrap();
    binding::context(a, &s).unwrap();
    assert_eq!(*session(a), expected);
    assert!(!progress::current_solved(a, &s));
    let raw = std::fs::read(&file).unwrap();
    assert!(progress::save(&file, &restored).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), raw);
}
#[test]
fn malformed_foreign_and_goal_free_requests_do_not_reach_any_other_interpreter() {
    let s = fixture::scenario();
    for input in [
        "sh -c anything",
        "curl host.invalid",
        "guest-image {bad}",
        "guest-image {\"kind\":\"run\",\"input\":\"uname -r\",\"host\":true}",
    ] {
        let mut state = s.initial.clone();
        let before = state.clone();
        assert!(engine::execute_for_goal(&mut state, &s.goal, input).is_err());
        assert_eq!(
            serde_json::to_value(state).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
    let mut state = s.initial.clone();
    let before = serde_json::to_value(&state).unwrap();
    assert!(engine::execute(&mut state, &s.solution[0]).is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    let mut a = Attempt::new(&s, 1);
    a.scenario_version += 1;
    assert!(binding::context(&a, &s).is_err());
    let mut a = Attempt::new(&s, 1);
    session(&mut a).source_id = "foreign-lesson".into();
    assert!(binding::context(&a, &s).is_err());
}
#[test]
fn invalid_initial_source_pair_and_unreachable_reference_pack_are_refused() {
    let s = fixture::scenario();
    let mut bad = s.clone();
    if let State::GuestDrivers { simulation } = &mut bad.initial {
        simulation.assignment.checked = true;
    }
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    if let Goal::GuestDrivers { definition } = &mut bad.goal {
        definition.reference = vec![work::Action::Check];
    }
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[bad]}).to_string()).is_err()
    );
    let mut bad = s.clone();
    bad.track = "CCNA".into();
    assert!(binding::source(&bad).is_err());
    let mut bad = s;
    bad.guide[0].commands[0].command = format!("{}{{}}", binding::PREFIX);
    bad.guide[0].commands[0].simulator_only = false;
    assert!(binding::source(&bad).is_err());
}
