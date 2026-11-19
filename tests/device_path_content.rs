use dlms_learning_lab::{
    course, device_path_workspace as work,
    engine::{self, State},
    pack, progress,
    switching::Jack,
};

fn scenarios() -> Vec<pack::Scenario> {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v12.json")).unwrap();
    data["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|scenario| scenario["initial"]["kind"] == "DevicePath")
        .map(|scenario| serde_json::from_value(scenario.clone()).unwrap())
        .collect()
}
fn passed(draft: &work::Draft, task: &work::Task) -> bool {
    draft.checks(task).iter().all(|check| check.0)
}

#[test]
fn both_distinct_assignments_and_separate_practice_solutions_work() {
    for scenario in scenarios() {
        let definition = work::source(&scenario).unwrap();
        let mut session = work::Session::new(definition);
        session.assignment.test(&definition.initial).unwrap();
        assert!(!passed(&session.assignment, &definition.initial));
        assert!(
            session
                .assignment
                .checks(&definition.initial)
                .iter()
                .any(|check| check.0)
        );
        session
            .practice
            .edit(&definition.example, definition.example.reference.clone())
            .unwrap();
        session.practice.test(&definition.example).unwrap();
        assert!(passed(&session.practice, &definition.example));
        assert!(!definition.checks(&session).iter().all(|check| check.0));
        let mut state = scenario.initial.clone();
        engine::execute_for_goal(&mut state, &scenario.goal, &scenario.solution[0]).unwrap();
        assert!(engine::solved(&state, &scenario.goal), "{}", scenario.id);
    }
}

#[test]
fn alternate_ports_pass_and_edits_invalidate_success() {
    let scenario = scenarios().remove(0);
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut draft = work::Draft::new(task);
    let mut alternative = task.reference.clone();
    alternative[0].b = Jack::Switch {
        switch: "office".into(),
        port: "p4".into(),
    };
    draft.edit(task, alternative.clone()).unwrap();
    draft.test(task).unwrap();
    assert!(passed(&draft, task));
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|row| row.1.contains("client -> edge -> server")
                && row.1.contains("server -> edge -> client"))
    );
    alternative[1].up = false;
    draft.edit(task, alternative).unwrap();
    assert!(!draft.checked && !passed(&draft, task));
    draft.test(task).unwrap();
    assert!(draft.checks(task).iter().any(|check| check.0));
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|check| !check.0 && check.1.contains("monitor"))
    );
}

#[test]
fn working_ipv4_does_not_hide_broadcast_leaks_or_disconnected_hosts() {
    let scenario = scenarios().remove(1);
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut draft = work::Draft::new(task);
    draft.test(task).unwrap();
    let checks = draft.checks(task);
    assert_eq!(
        checks.iter().map(|row| row.0).collect::<Vec<_>>(),
        [true, false, true, true]
    );
    assert!(checks[1].1.contains("Layer 2"));
    let mut disconnected = task.reference.clone();
    for cable in &mut disconnected {
        cable.up = false;
    }
    draft.edit(task, disconnected).unwrap();
    draft.test(task).unwrap();
    assert_eq!(
        draft
            .checks(task)
            .iter()
            .map(|row| row.0)
            .collect::<Vec<_>>(),
        [false, true, false, false]
    );
    assert!(!passed(&draft, task));
}

#[test]
fn duplicate_jacks_are_editable_retryable_and_never_award_success() {
    let scenario = scenarios().remove(0);
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut draft = work::Draft::new(task);
    let mut cables = task.reference.clone();
    cables[0].b = cables[1].b.clone();
    draft.edit(task, cables).unwrap();
    draft.test(task).unwrap();
    assert!(!passed(&draft, task));
    assert!(draft.checks(task)[0].1.contains("jack"));
    draft.edit(task, task.reference.clone()).unwrap();
    draft.test(task).unwrap();
    assert!(passed(&draft, task));
    let before = draft.clone();
    let mut malicious = task.reference.clone();
    malicious[0].b = Jack::Switch {
        switch: "../../host".into(),
        port: "p1".into(),
    };
    assert!(draft.edit(task, malicious).is_err());
    assert_eq!(draft, before);
}

#[test]
fn source_reachability_foreign_progress_and_allocation_bounds_are_enforced() {
    let scenarios = scenarios();
    let scenario = &scenarios[0];
    let definition = work::source(scenario).unwrap();
    let mut bad = definition.clone();
    bad.initial.reference = bad.initial.initial.clone();
    assert!(bad.validate().is_err());
    let mut bad = definition.clone();
    bad.initial.probes.pop();
    bad.initial.probes.pop();
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(definition).unwrap();
    value["initial"]["facts"]["attachments"][0]["endpoint"] = "x".repeat(65).into();
    assert!(serde_json::from_value::<work::Definition>(value).is_err());
    let mut value = serde_json::to_value(work::Session::new(definition)).unwrap();
    value["assignment"]["cables"] =
        serde_json::Value::Array(vec![value["assignment"]["cables"][0].clone(); 9]);
    assert!(serde_json::from_value::<work::Session>(value).is_err());
    let attempt = progress::Attempt::new(&scenarios[1], 10);
    assert!(work::context(&attempt, scenario).is_err());
    let mut bad = scenario.clone();
    bad.track = "LPIC-1".into();
    assert!(work::source(&bad).is_err());
}

#[test]
fn save_resume_preserves_independent_drafts_and_reset_only_changes_selected_attempt() {
    let catalog = scenarios();
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for scenario in &catalog {
        let mut attempt = progress::Attempt::new(scenario, 10);
        engine::execute_for_goal(&mut attempt.state, &scenario.goal, &scenario.solution[0])
            .unwrap();
        let State::DevicePath { simulation } = &mut attempt.state else {
            panic!()
        };
        simulation.practice.cables[0].up = false;
        assert!(progress::current_solved(&attempt, scenario));
        profile.attempts.insert(scenario.id.clone(), attempt);
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save_with_catalog(&path, &profile, &catalog).unwrap();
    let mut restored = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&profile).unwrap()
    );
    let retained = serde_json::to_value(&restored.attempts[&catalog[1].id]).unwrap();
    restored.attempts.insert(
        catalog[0].id.clone(),
        progress::Attempt::new(&catalog[0], 11),
    );
    assert!(!progress::current_solved(
        &restored.attempts[&catalog[0].id],
        &catalog[0]
    ));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&catalog[1].id]).unwrap(),
        retained
    );
}

#[test]
fn all_390_existing_scenarios_and_149_units_are_preserved_exactly() {
    let old = pack::accepted390().unwrap();
    let current = pack::accepted392().unwrap();
    assert_eq!((old.len(), current.len()), (390, 392));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..390]).unwrap()
    );
    let old_units = course::accepted390(&old).unwrap();
    let units = course::accepted392(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (149, 150));
    let retained: Vec<_> = units
        .iter()
        .filter(|unit| unit.id != "ccna-wired-device-paths")
        .collect();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}
