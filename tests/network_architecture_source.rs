use dlms_learning_lab::{
    engine::{self, State},
    network_architecture_source as source, network_architecture_workspace as work, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!(
        "../scenarios/campus-architecture-lessons-v1.json"
    ))
    .unwrap()
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::CampusArchitecture { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn authored_reference_events_and_alternate_ports_grade_current_state_without_host_commands() {
    for s in labs() {
        let d = source::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        assert!(!progress::current_solved(&a, &s));
        assert!(engine::execute(&mut a.state, "campus-layout {\"kind\":\"Check\"}").is_err());
        for event in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, event).unwrap();
        }
        assert!(
            progress::current_solved(&a, &s),
            "{:?}",
            engine::checks(&a.state, &s.goal)
        );
        let mut broken = d.assignment.reference.clone();
        let wire = &d.assignment.editable[0].id;
        broken
            .connections
            .iter_mut()
            .find(|c| &c.wire == wire)
            .unwrap()
            .b = None;
        session(&mut a)
            .assignment
            .edit(&d.assignment, broken)
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
        session(&mut a).assignment.checked = true;
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn malformed_unknown_and_oversized_declarative_actions_cannot_mutate_state() {
    for s in labs() {
        let mut a = Attempt::new(&s, 1);
        let before = serde_json::to_value(&a.state).unwrap();
        for bad in [
            "ssh personal-host",
            "campus-layout {\"kind\":\"Spawn\"}",
            "campus-layout {\"kind\":\"Check\",\"passed\":true}",
            "campus-layout invalid JSON",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, bad).is_err());
            assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
        }
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, &"x".repeat(source::MAX_ACTION + 1))
                .is_err()
        );
        assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
    }
}
#[test]
fn catalog_bound_disk_resume_preserves_practice_favorites_and_checked_assignment() {
    for s in labs() {
        let d = source::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        for e in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, e).unwrap();
        }
        let mut draft = d.example.initial.clone();
        draft
            .connections
            .iter_mut()
            .find(|c| c.wire == d.example.editable[0].id)
            .unwrap()
            .b = None;
        session(&mut a)
            .practice
            .edit(&d.example, draft.clone())
            .unwrap();
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress-v1.json");
        progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
        let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
        let State::CampusArchitecture { simulation } = &loaded.attempts[&s.id].state else {
            panic!()
        };
        assert_eq!(simulation.practice.draft, draft);
        assert!(!simulation.practice.checked);
        let raw = std::fs::read(&path).unwrap();
        assert!(progress::save(&path, &p).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
    }
}
#[test]
fn foreign_source_versions_initial_states_and_metadata_are_rejected_before_credit() {
    let s = labs().remove(0);
    let mut a = Attempt::new(&s, 1);
    for e in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, e).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let mut foreign = s.clone();
    foreign.track = "LPIC-1".into();
    assert!(source::source(&foreign).is_err());
    assert!(!progress::current_solved(&a, &foreign));
    let mut foreign = s.clone();
    let State::CampusArchitecture { simulation } = &mut foreign.initial else {
        panic!()
    };
    simulation.assignment.checked = true;
    assert!(source::source(&foreign).is_err());
    a.scenario_version += 1;
    assert!(source::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    a.scenario_version = s.version;
    session(&mut a).id = "foreign".into();
    assert!(!progress::current_solved(&a, &s));
}
