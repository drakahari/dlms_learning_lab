use dlms_learning_lab::{
    engine::{self, Goal, State},
    network_transfer_source as source, network_transfer_workspace as work, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
#[path = "support/network_transfer_fixture.rs"]
mod fixture;
fn lab() -> pack::Scenario {
    let d = fixture::definition();
    let actions = [
        source::Event::Edit {
            draft: d.assignment.new_draft().unwrap(),
        },
        source::Event::Transfer {},
        source::Event::Check {},
    ];
    let solution: Vec<_> = actions
        .iter()
        .map(|e| format!("{}{}", source::PREFIX, serde_json::to_string(e).unwrap()))
        .collect();
    let value = serde_json::json!({"id":d.id,"version":1,"track":"CCNA","unit":"transfer-fixture","topic":"File transfer fixture","title":"Transfer a supplied virtual configuration","summary":"A controlled source-binding test, not a registered lesson.","objective":"Copy the supplied original server file to staged.cfg and preserve startup.cfg; keep the transfer path ready.","difficulty":"Beginner","minutes":15,"steps":["Understand the transfer","Try separate practice","Complete the independent copy"],"hints":["Compare the direction and source file.","Keep the original device file.","A successful copy does not activate the file."],"sources":["https://www.rfc-editor.org/rfc/rfc1350.html"],"simplifications":"Bounded developer fixture only, no host transfer or catalog coverage.","initial":{"kind":"NetworkTransfer","simulation":d.new_session().unwrap()},"goal":{"kind":"NetworkTransfer","definition":d},"solution":solution});
    let text = serde_json::json!({"schema_version":1,"scenarios":[value]}).to_string();
    pack::load(&text).unwrap().remove(0)
}
fn state_mut(a: &mut Attempt) -> &mut work::Session {
    let State::NetworkTransfer { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn strict_pack_source_and_real_authoring_events_grade_outcomes_without_a_generic_terminal() {
    let s = lab();
    source::source(&s).unwrap();
    let mut state = s.initial.clone();
    assert!(!engine::solved(&state, &s.goal));
    assert!(engine::execute(&mut state, "file-transfer {\"kind\":\"Transfer\"}").is_err());
    assert!(!engine::solved(&state, &s.goal));
    for action in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        engine::checks(&state, &s.goal)
    );
    assert!(
        engine::execute_for_goal(
            &mut state,
            &Goal::EditorHandover {
                definition: dlms_learning_lab::editor_handover_source::source(
                    &pack::load(include_str!("../scenarios/editor-handover-lessons-v1.json"))
                        .unwrap()[0]
                )
                .unwrap()
                .clone()
                .into()
            },
            "file-transfer {\"kind\":\"Check\"}"
        )
        .is_err()
    );
}
#[test]
fn unknown_commands_fields_and_oversized_actions_leave_actual_state_unchanged() {
    let s = lab();
    let mut state = s.initial.clone();
    let before = serde_json::to_value(&state).unwrap();
    for action in [
        "cp /etc/passwd remote",
        "file-transfer {\"kind\":\"Transfer\",\"exec\":\"anything\"}",
        "file-transfer {\"kind\":\"Spawn\"}",
        "file-transfer invalid JSON",
    ] {
        assert!(engine::execute_for_goal(&mut state, &s.goal, action).is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    assert!(
        engine::execute_for_goal(&mut state, &s.goal, &"x".repeat(source::MAX_ACTION + 1)).is_err()
    );
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}
#[test]
fn ordinary_progress_round_trip_replays_saved_operations_and_keeps_independent_drafts() {
    let s = lab();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    for action in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let Goal::NetworkTransfer { definition } = &s.goal else {
        panic!()
    };
    let mut draft = state_mut(&mut a).practice.draft.clone();
    draft.request.local_path = "unfinished/".into();
    state_mut(&mut a)
        .practice
        .edit(&definition.example, draft.clone())
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    let State::NetworkTransfer { simulation } = &loaded.attempts[&s.id].state else {
        panic!()
    };
    assert_eq!(simulation.practice.draft, draft);
    assert!(simulation.practice.history.is_empty());
    assert!(!simulation.practice.checked);
    let raw = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw);
}
#[test]
fn source_identity_initial_state_and_progress_metadata_must_match_before_credit() {
    let s = lab();
    let mut a = Attempt::new(&s, 1);
    for action in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let mut foreign = s.clone();
    foreign.track = "LPIC-1".into();
    assert!(source::source(&foreign).is_err());
    assert!(!progress::current_solved(&a, &foreign));
    let mut foreign = s.clone();
    let State::NetworkTransfer { simulation } = &mut foreign.initial else {
        panic!()
    };
    simulation.assignment.checked = true;
    assert!(source::source(&foreign).is_err());
    a.scenario_version = 2;
    assert!(source::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    a.scenario_version = 1;
    state_mut(&mut a).id = "other-source".into();
    assert!(source::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
}
