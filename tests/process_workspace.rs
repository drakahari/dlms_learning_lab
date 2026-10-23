use dlms_learning_lab::{
    engine::{self, State},
    pack, process_teaching as teaching,
    processes::Phase,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/process-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn catalog() -> Vec<pack::Scenario> {
    let mut all = pack::bundled().unwrap();
    all.push(scenario());
    all
}
#[test]
fn reference_alternatives_partial_errors_and_current_state_grade_without_command_matching() {
    let s = scenario();
    for command in ["kill -STOP 101", "kill -19 101", "kill -s SIGSTOP -- 101"] {
        let mut a = Attempt::new(&s, 1);
        assert!(!progress::current_solved(&a, &s));
        engine::execute(&mut a.state, command).unwrap();
        assert!(progress::current_solved(&a, &s));
        engine::execute(&mut a.state, "kill -CONT 101").unwrap();
        assert!(!progress::current_solved(&a, &s));
    }
    let mut a = Attempt::new(&s, 1);
    let message = engine::execute(&mut a.state, "kill -STOP 101 201").unwrap_err();
    assert!(message.contains("Operation not permitted") && message.contains("exit status 1"));
    assert!(
        progress::current_solved(&a, &s),
        "valid own action persists despite another denied target"
    );
    let mut state = s.initial.clone();
    engine::execute(&mut state, "kill -TERM 101").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    assert!(engine::checks(&state, &s.goal)[0].1.contains("Reset"));
}
#[test]
fn practice_commands_and_changed_experiment_remain_separate_and_ungraded() {
    let s = scenario();
    let t = s.process_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, t, true);
    let initial = a.state.clone();
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .process_learning
        .as_mut()
        .unwrap();
    for command in &t.example {
        learning.command = command.clone();
        learning.execute();
    }
    assert!(t.outcomes.iter().all(|r| r.check(&learning.table).0));
    learning.command = "kill -CONT 610".into();
    learning.execute();
    assert!(matches!(
        learning.table.processes[&610].phase,
        Phase::Active { .. }
    ));
    learning.command = "kill -STOP 101".into();
    learning.execute();
    assert!(
        learning
            .transcript
            .last()
            .unwrap()
            .contains("No such process")
    );
    assert_eq!(a.state, initial);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn separate_drafts_outcomes_old_attempts_and_selected_reset_resume_exactly() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for old in all.iter().take(74) {
        p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    }
    let original = serde_json::to_vec(&p).unwrap();
    let mut a = Attempt::new(s, 2);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.process_teaching.as_ref().unwrap(), true);
    engine::execute(&mut a.state, "kill -STOP 101").unwrap();
    let w = a.workspace.as_mut().unwrap();
    w.command = "ps -p 101 -o pid,stat".into();
    let l = w.process_learning.as_mut().unwrap();
    l.command = "ps -p 610 -o pid,stat".into();
    l.view = View::Assignment;
    p.attempts.insert(s.id.clone(), a);
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded = progress::decode(&bytes, &all).unwrap();
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    loaded.attempts.insert(s.id.clone(), Attempt::new(s, 3));
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    loaded.attempts.remove(&s.id);
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), original);
}
#[test]
fn decoded_assignment_and_practice_metadata_cannot_change_identities_or_cross_activities() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.process_teaching.as_ref().unwrap(), true);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let encoded = serde_json::to_value(&p).unwrap();
    for pointer in [
        format!("/attempts/{}/state/table/processes/101/comm", s.id),
        format!(
            "/attempts/{}/workspace/process_learning/table/processes/610/comm",
            s.id
        ),
    ] {
        let mut bad = encoded.clone();
        *bad.pointer_mut(&pointer).unwrap() = serde_json::json!("foreign");
        assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
    }
    let mut bad = encoded.clone();
    bad["attempts"][&s.id]["workspace"]["process_learning"]["example_command"] =
        serde_json::json!(7);
    assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
    let old = &all[0];
    let mut wrong = Attempt::new(old, 1);
    workspace::prepare(&mut wrong);
    wrong.workspace.as_mut().unwrap().process_learning = p.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .process_learning
        .clone();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(old.id.clone(), wrong);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}
#[test]
fn invalid_objectives_examples_bounds_and_unsolved_reference_are_rejected() {
    let s = scenario();
    let json = serde_json::to_value(&s).unwrap();
    for pointer in [
        "/goal/requirements/0/pid",
        "/process_teaching/outcomes/0/pid",
    ] {
        let mut invalid = json.clone();
        *invalid.pointer_mut(pointer).unwrap() = serde_json::json!(999);
        let data = serde_json::json!({"schema_version":1,"scenarios":[invalid]});
        assert!(pack::load(&data.to_string()).is_err());
    }
    let mut invalid = json.clone();
    invalid["solution"] = serde_json::json!(["kill -TERM 101"]);
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[invalid]}).to_string())
            .is_err()
    );
    let mut invalid = json.clone();
    invalid["process_teaching"]["fixture"] = invalid["initial"]["table"].clone();
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[invalid]}).to_string())
            .is_err()
    );
    let mut learning = teaching::Learning::new(s.process_teaching.as_ref().unwrap());
    learning.command = "é".repeat(3000);
    learning.execute();
    assert!(learning.valid());
    assert!(learning.transcript[0].len() < 40_000);
    let huge = vec![b' '; 16_000_001];
    assert!(progress::decode(&huge, &catalog()).is_err());
}
#[test]
fn foreign_process_shapes_cannot_award_current_success_and_helpers_are_disclosed() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    engine::execute(&mut a.state, "kill -STOP 101").unwrap();
    assert!(progress::current_solved(&a, &s));
    let State::Processes { table } = &mut a.state else {
        panic!()
    };
    table.processes.get_mut(&101).unwrap().comm = "foreign".into();
    assert!(!progress::current_solved(&a, &s));
    assert!(engine::help(&s.initial).contains("simulator-only"));
    assert!(
        !pack::bundled()
            .unwrap()
            .iter()
            .any(|s| s.id == "test-process-workspace")
    );
}

#[test]
fn invalid_process_metadata_cannot_replace_existing_progress_bytes_on_save() {
    let all = catalog();
    let original = &all[0];
    let mut a = Attempt::new(original, 1);
    workspace::prepare(&mut a);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(original.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    p.attempts
        .get_mut(&original.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .process_learning = Some(teaching::Learning::new(
        scenario().process_teaching.as_ref().unwrap(),
    ));
    assert!(
        progress::save(&path, &p)
            .unwrap_err()
            .contains("Process example metadata")
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(progress::decode(&before, &all).is_ok());
}
