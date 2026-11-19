use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    local_mail_workspace as work, pack, progress,
};

fn scenarios() -> Vec<pack::Scenario> {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v11.json")).unwrap();
    pack["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|lab| lab["initial"]["kind"] == "LocalMail")
        .map(|lab| serde_json::from_value(lab.clone()).unwrap())
        .collect()
}

fn repair(draft: &mut work::Draft, task: &work::Task, files: &work::Files) {
    draft.selected = work::Document::Aliases;
    draft.edit(files.aliases.clone()).unwrap();
    draft.save_file(task).unwrap();
    for (user, text) in &files.forwards {
        draft.selected = work::Document::Forward(user.clone());
        draft.edit(text.clone()).unwrap();
        draft.save_file(task).unwrap();
    }
    draft.rebuild_aliases(task).unwrap();
    draft.test(task).unwrap();
}

#[test]
fn all_three_assignments_have_separate_worked_cases_and_behavioral_repairs() {
    let scenarios = scenarios();
    assert_eq!(scenarios.len(), 3);
    for scenario in scenarios {
        let definition = work::source(&scenario).unwrap();
        let mut session = work::Session::new(definition);
        session.assignment.test(&definition.initial).unwrap();
        let checks = session.assignment.checks(&definition.initial);
        assert!(checks.iter().any(|row| row.0));
        assert!(checks.iter().any(|row| !row.0));
        repair(
            &mut session.practice,
            &definition.example,
            &definition.example.reference,
        );
        assert!(
            session
                .practice
                .checks(&definition.example)
                .iter()
                .all(|row| row.0)
        );
        assert!(!definition.checks(&session).iter().all(|row| row.0));
        repair(
            &mut session.assignment,
            &definition.initial,
            &definition.initial.reference,
        );
        assert!(
            definition.checks(&session).iter().all(|row| row.0),
            "{}",
            scenario.id
        );
        let mut state = scenario.initial.clone();
        engine::execute_for_goal(&mut state, &scenario.goal, &scenario.solution[0]).unwrap();
        assert!(
            engine::checks(&state, &scenario.goal)
                .iter()
                .all(|row| row.0)
        );
    }
}

#[test]
fn supported_alias_paths_case_and_forward_order_are_equivalent() {
    let scenarios = scenarios();
    for scenario in scenarios {
        let definition = work::source(&scenario).unwrap();
        let task = &definition.initial;
        let mut alternative = task.reference.clone();
        if scenario.id.ends_with("team-alias") {
            alternative.aliases =
                "# another route\nTeam: helpers\nhelpers: JULES, ALEX, alex\npostmaster: sam\n"
                    .into();
        } else if scenario.id.ends_with("personal-forward") {
            alternative
                .forwards
                .insert("alex".into(), "# keep both\njules, alex, jules\n".into());
        } else {
            alternative.aliases = "postmaster: sam\nteam: helper\nhelper: ALEX\n".into();
        }
        let mut draft = work::Draft::new(task);
        repair(&mut draft, task, &alternative);
        assert!(
            draft.checks(task).iter().all(|row| row.0),
            "{}",
            scenario.id
        );
    }
}

#[test]
fn saved_policy_and_current_index_are_both_required_and_wrong_branches_are_visible() {
    let scenario = scenarios().pop().unwrap();
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut draft = work::Draft::new(task);
    draft.test(task).unwrap();
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(_, text)| text.contains("retired"))
    );
    draft.rebuild_aliases(task).unwrap();
    draft.test(task).unwrap();
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(_, text)| text.contains("loop"))
    );

    repair(&mut draft, task, &task.reference);
    draft.selected = work::Document::Aliases;
    draft.edit("team: sam\npostmaster: sam\n".into()).unwrap();
    assert!(draft.tested.is_none());
    draft.save_file(task).unwrap();
    // The current lookup remains correct, but the next rebuild would break team mail.
    draft.test(task).unwrap();
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(ok, text)| !ok && text.contains("would fail after rebuilding"))
    );
    assert!(!draft.checks(task).iter().all(|row| row.0));
}

#[test]
fn forwarding_is_read_after_save_without_rebuilding_and_malformed_files_are_atomic() {
    let scenario = scenarios().remove(1);
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut draft = work::Draft::new(task);
    let old_index = draft.index.clone();
    assert_eq!(draft.selected, work::Document::Forward("alex".into()));
    draft.edit("alex, jules\n".into()).unwrap();
    draft.test(task).unwrap();
    assert!(!draft.checks(task).iter().all(|row| row.0));
    draft.save_file(task).unwrap();
    draft.test(task).unwrap();
    assert_eq!(draft.index, old_index);
    assert!(draft.checks(task).iter().all(|row| row.0));
    let saved = draft.saved.clone();
    draft.edit("|touch /tmp/example\n".into()).unwrap();
    assert!(draft.save_file(task).is_err());
    assert_eq!(draft.saved, saved);
    assert!(draft.tested.is_none());
}

#[test]
fn real_progress_resume_and_selected_reset_preserve_other_attempts_and_drafts() {
    let scenarios = scenarios();
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for scenario in &scenarios {
        let mut attempt = progress::Attempt::new(scenario, 10);
        engine::execute_for_goal(&mut attempt.state, &scenario.goal, &scenario.solution[0])
            .unwrap();
        let State::LocalMail { simulation } = &mut attempt.state else {
            panic!()
        };
        simulation.practice.selected = work::Document::Aliases;
        simulation
            .practice
            .edit("# a separate unfinished example".into())
            .unwrap();
        assert!(progress::current_solved(&attempt, scenario));
        profile.attempts.insert(scenario.id.clone(), attempt);
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save_with_catalog(&path, &profile, &scenarios).unwrap();
    let mut restored = progress::load(&path, &scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&profile).unwrap()
    );
    let original = serde_json::to_value(&restored.attempts[&scenarios[1].id]).unwrap();
    restored.attempts.insert(
        scenarios[0].id.clone(),
        progress::Attempt::new(&scenarios[0], 11),
    );
    assert!(!progress::current_solved(
        &restored.attempts[&scenarios[0].id],
        &scenarios[0]
    ));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&scenarios[1].id]).unwrap(),
        original
    );
}

#[test]
fn every_preceding_387_scenario_and_148_unit_value_remains_exact() {
    let old = pack::accepted387().unwrap();
    let current = pack::accepted390().unwrap();
    assert_eq!((old.len(), current.len()), (387, 390));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..387]).unwrap()
    );
    let old_units = course::accepted387(&old).unwrap();
    let units = course::accepted390(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (148, 149));
    let retained = units
        .iter()
        .filter(|unit| unit.id != "linux-local-mail-routing")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    for scenario in scenarios() {
        let Goal::LocalMail { definition } = &scenario.goal else {
            panic!()
        };
        assert!(definition.validate().is_ok());
    }
}

#[test]
fn invalid_task_metadata_oversized_names_and_foreign_progress_are_refused() {
    let scenario = scenarios().remove(1);
    let definition = work::source(&scenario).unwrap();
    let mut bad = definition.clone();
    bad.example.users[0] = bad.initial.users[0].clone();
    assert!(bad.validate().is_err());
    let mut bad = definition.clone();
    bad.initial.probes[0].label = "Broken\nlabel".into();
    assert!(bad.validate().is_err());
    let mut encoded = serde_json::to_value(definition).unwrap();
    encoded["initial"]["probes"][0]["expected"][0] = "x".repeat(33).into();
    assert!(serde_json::from_value::<work::Definition>(encoded).is_err());
    let encoded = serde_json::json!({"Forward":"x".repeat(33)});
    assert!(serde_json::from_value::<work::Document>(encoded).is_err());

    let mut attempt = progress::Attempt::new(&scenario, 12);
    let State::LocalMail { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation.context = "another-task".into();
    assert!(work::context(&attempt, &scenario).is_err());
    let mut attempt = progress::Attempt::new(&scenario, 13);
    let State::LocalMail { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .edit("|unsupported program".into())
        .unwrap();
    assert!(work::context(&attempt, &scenario).is_ok());
    let State::LocalMail { simulation } = &mut attempt.state else {
        panic!()
    };
    assert!(
        simulation
            .assignment
            .save_file(&definition.initial)
            .is_err()
    );
}
