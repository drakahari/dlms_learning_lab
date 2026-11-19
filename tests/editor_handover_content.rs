use dlms_learning_lab::{
    editor_handover_source as binding,
    engine::{self, State},
    environment_workspace as env,
    modal_editing::Key,
    modal_workspace::Event,
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn lab() -> pack::Scenario {
    pack::load(include_str!("../scenarios/editor-handover-lessons-v1.json"))
        .unwrap()
        .remove(0)
}
fn simulation(s: &mut State) -> &mut dlms_learning_lab::editor_handover::Session {
    let State::EditorHandover { simulation } = s else {
        panic!()
    };
    simulation
}
#[test]
fn complete_taught_assignment_and_separate_worked_example_reach_their_own_outcomes() {
    let s = lab();
    let d = binding::source(&s).unwrap();
    let mut state = s.initial.clone();
    assert!(!engine::solved(&state, &s.goal));
    let a = simulation(&mut state);
    a.edit_setup(d, true, d.example_setup.clone()).unwrap();
    let r = env::run(&d.setup.example, &d.example_setup).unwrap();
    a.store_setup(d, true, r).unwrap();
    a.open_editor(d, true).unwrap();
    a.files.show_worked(&d.editing).unwrap();
    a.apply_file(d, true, &Event::Check {}).unwrap();
    assert!(a.checks(d, true).iter().all(|r| r.0));
    assert!(!engine::solved(&state, &s.goal));
    for e in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, e).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        engine::checks(&state, &s.goal)
    );
    simulation(&mut state)
        .apply_file(
            d,
            false,
            &Event::Key {
                key: Key::Character('x'),
            },
        )
        .unwrap();
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn different_supported_setup_and_record_order_pass_but_partial_and_duplicate_records_fail() {
    let s = lab();
    let d = binding::source(&s).unwrap();
    let mut state = s.initial.clone();
    let a = simulation(&mut state);
    let source = "EDITOR=$PREFERRED_EDITOR\nexport EDITOR";
    a.edit_setup(d, false, source.into()).unwrap();
    a.store_setup(d, false, env::run(&d.setup.assignment, source).unwrap())
        .unwrap();
    a.open_editor(d, false).unwrap();
    let keys = |a: &mut dlms_learning_lab::editor_handover::Session, text: &str| {
        for c in text.chars() {
            a.apply_file(
                d,
                false,
                &Event::Key {
                    key: match c {
                        '\r' => Key::Enter,
                        '\x1b' => Key::Escape,
                        _ => Key::Character(c),
                    },
                },
            )
            .unwrap();
        }
    };
    keys(a, "/OWNER\rddGoREVIEW ready\rOWNER platform\x1b");
    a.apply_file(d, false, &Event::Check {}).unwrap();
    assert!(!a.solved(d), "Unsaved work cannot pass");
    keys(a, ":w\r");
    a.apply_file(d, false, &Event::Check {}).unwrap();
    assert!(a.solved(d), "{:?}", a.checks(d, false));
    keys(a, "/REVIEW\ryyp:w\r");
    a.apply_file(d, false, &Event::Check {}).unwrap();
    assert!(!a.solved(d));
    assert!(
        a.checks(d, false)
            .iter()
            .any(|r| !r.0 && r.1.contains("saved 2"))
    );
}
#[test]
fn actual_progress_round_trip_preserves_assignment_practice_favorites_and_unverified_credit() {
    let s = lab();
    let catalog = vec![s.clone()];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    let d = binding::source(&s).unwrap();
    for e in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, e).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    a.hints = 1;
    let simulation = simulation(&mut a.state);
    simulation
        .edit_setup(d, true, "# keep practice draft\n".into())
        .unwrap();
    simulation
        .apply_file(
            d,
            false,
            &Event::Key {
                key: Key::Character('i'),
            },
        )
        .unwrap();
    simulation
        .apply_file(d, false, &Event::Insert { text: "X".into() })
        .unwrap();
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    let saved = &loaded.attempts[&s.id];
    binding::context(saved, &s).unwrap();
    assert_eq!(saved.visited, 7);
    assert_eq!(saved.hints, 1);
    assert!(loaded.favorites.contains(&s.id));
    assert!(!progress::current_solved(saved, &s));
    let State::EditorHandover { simulation } = &saved.state else {
        panic!()
    };
    assert!(
        simulation
            .files
            .assignment
            .editor()
            .contents()
            .contains('X')
    );
    assert_eq!(simulation.setup.example.source, "# keep practice draft\n");
    let mut resumed = (**simulation).clone();
    let source = resumed.setup.assignment.source.clone();
    resumed
        .store_setup(d, false, env::run(&d.setup.assignment, &source).unwrap())
        .unwrap();
    resumed.open_editor(d, false).unwrap();
    resumed
        .apply_file(d, false, &Event::Key { key: Key::Escape })
        .unwrap();
    resumed
        .apply_file(
            d,
            false,
            &Event::Key {
                key: Key::Character('u'),
            },
        )
        .unwrap();
    resumed.apply_file(d, false, &Event::Check {}).unwrap();
    assert!(resumed.solved(d));
    let practice = resumed.setup.example.clone();
    resumed.reset(d, false).unwrap();
    assert_eq!(resumed.setup.example, practice);
    assert!(!resumed.solved(d));
}
#[test]
fn mismatched_sources_and_metadata_do_not_admit_saved_work() {
    let s = lab();
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    binding::context(&a, &s).unwrap();
    a.scenario_version += 1;
    assert!(binding::context(&a, &s).is_err());
    a.scenario_version -= 1;
    simulation(&mut a.state).id = "other-task".into();
    assert!(binding::context(&a, &s).is_err());
    let mut wrong = s.clone();
    wrong.track = "CCNA".into();
    assert!(binding::source(&wrong).is_err());
    let mut wrong = s;
    let engine::Goal::EditorHandover { definition } = &mut wrong.goal else {
        panic!()
    };
    definition.example_setup = "export EDITOR=nano".into();
    assert!(binding::source(&wrong).is_err());
}
#[test]
fn normal_catalog_preserves_all_448_lab_and_177_unit_values_and_prerequisite_order() {
    let old = pack::accepted448().unwrap();
    let current = pack::current().unwrap();
    assert_eq!(old.len(), 448);
    assert_eq!(current.len(), 452);
    for s in &old {
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(current.iter().find(|n| n.id == s.id).unwrap()).unwrap(),
            "{}",
            s.id
        );
    }
    let old_units = dlms_learning_lab::course::accepted448(&old).unwrap();
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!(old_units.len(), 177);
    assert_eq!(units.len(), 179);
    for u in &old_units {
        assert_eq!(
            serde_json::to_value(u).unwrap(),
            serde_json::to_value(units.iter().find(|n| n.id == u.id).unwrap()).unwrap(),
            "{}",
            u.id
        );
    }
    let unit = units
        .iter()
        .find(|u| u.id == "standard-editor-handover")
        .unwrap();
    assert_eq!(
        unit.prerequisites,
        ["modal-file-editing", "linux-shell-environment"]
    );
    let position = units.iter().position(|u| u.id == unit.id).unwrap();
    for id in &unit.prerequisites {
        assert!(units.iter().position(|u| &u.id == id).unwrap() < position);
    }
    assert_eq!(unit.labs, ["lpic1-editor-handover"]);
    for (track, count) in [
        ("CCNA", 107),
        ("LPIC-1", 151),
        ("LPIC-2", 114),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(
            current.iter().filter(|s| s.track == track).count(),
            count,
            "{track}"
        );
    }
}
