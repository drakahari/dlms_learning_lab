use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    journal_archive_workspace as work, pack, progress,
};

const ID: &str = "lpic1-copied-journal-incident";

fn scenario() -> pack::Scenario {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v10.json")).unwrap();
    serde_json::from_value(
        value["scenarios"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone(),
    )
    .unwrap()
}

fn query(task: &work::Task) -> String {
    format!(
        "journalctl --unit={} --directory={} --priority={} --boot={} --output=cat",
        task.expected_unit, task.archive.directory, task.expected_priority, task.expected_boot
    )
}

fn list(draft: &mut work::Draft, task: &work::Task) {
    draft
        .edit(format!(
            "journalctl -D {} --list-boots",
            task.archive.directory
        ))
        .unwrap();
    draft.run(task).unwrap();
    assert!(draft.showing_boot_list());
    assert!(
        draft
            .output(task)
            .unwrap()
            .unwrap()
            .contains(&task.expected_boot)
    );
}

#[test]
fn practice_retrieves_real_messages_with_no_assignment_credit() {
    let scenario = scenario();
    let definition = work::source(&scenario).unwrap();
    let mut session = work::Session::new(definition);
    list(&mut session.practice, &definition.example);
    session.practice.edit(query(&definition.example)).unwrap();
    session.practice.run(&definition.example).unwrap();
    let output = session
        .practice
        .output(&definition.example)
        .unwrap()
        .unwrap();
    assert!(output.contains("Input stream stopped responding"));
    assert!(output.contains("Reader gave up"));
    assert!(!output.contains("unrelated"));
    session.practice.check(&definition.example).unwrap();
    assert!(
        session
            .practice
            .checks(&definition.example)
            .iter()
            .all(|row| row.0)
    );
    assert_eq!(session.assignment, work::Draft::default());
    assert!(!definition.checks(&session).iter().all(|row| row.0));
}

#[test]
fn alternate_flags_pass_and_wrong_boot_unit_and_threshold_fail_with_feedback() {
    let scenario = scenario();
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut draft = work::Draft::default();
    list(&mut draft, task);
    draft.edit(query(task)).unwrap();
    assert!(draft.check(task).is_err());
    draft.run(task).unwrap();
    draft.check(task).unwrap();
    assert!(draft.checks(task).iter().all(|row| row.0));
    assert_eq!(
        draft.output(task).unwrap().unwrap(),
        "Backup target is read-only\nBackup aborted before writing"
    );
    for (command, feedback) in [
        (
            query(task).replace(&task.expected_boot, &task.archive.boots[2].id),
            "boot",
        ),
        (
            query(task).replace("backup.service", "files.service"),
            "backup.service",
        ),
        (query(task).replace("--priority=3", "--priority=4"), "err"),
        (query(task).replace("--priority=3", "--priority=2"), "err"),
    ] {
        draft.edit(command).unwrap();
        assert!(!draft.checked && !draft.ran_current);
        assert!(draft.output(task).unwrap().is_none());
        draft.run(task).unwrap();
        draft.check(task).unwrap();
        assert!(
            draft
                .checks(task)
                .iter()
                .any(|(ok, text)| !ok && text.contains(feedback))
        );
    }
}

#[test]
fn malformed_queries_and_progress_tampering_do_not_produce_success() {
    let scenario = scenario();
    let definition = work::source(&scenario).unwrap();
    let task = &definition.initial;
    let mut session = work::Session::new(definition);
    for bad in [
        "journalctl --list-boots",
        "journalctl --directory=/var/log/journal --list-boots",
        "journalctl --directory=/recovery/field-journal --rotate",
        "journalctl --directory=/recovery/field-journal --list-boots | cat",
    ] {
        session.assignment.edit(bad.into()).unwrap();
        assert!(session.assignment.run(task).is_err());
        assert!(session.assignment.check(task).is_err());
        assert!(session.assignment.output(task).unwrap().is_none());
        assert!(!definition.checks(&session).iter().all(|row| row.0));
    }
    session.assignment.ran_current = true;
    assert!(session.bound(definition).is_err());
}

#[test]
fn actual_progress_save_restore_retains_independent_drafts_and_invalidates_success() {
    let scenario = scenario();
    let mut attempt = progress::Attempt::new(&scenario, 4);
    engine::execute_for_goal(&mut attempt.state, &scenario.goal, &scenario.solution[0]).unwrap();
    let State::CopiedJournal { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .practice
        .edit("journalctl -D /recovery/workshop-journal".into())
        .unwrap();
    assert!(progress::current_solved(&attempt, &scenario));
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    profile.attempts.insert(ID.into(), attempt);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save_with_catalog(&path, &profile, std::slice::from_ref(&scenario)).unwrap();
    let mut restored = progress::load(&path, std::slice::from_ref(&scenario)).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&profile).unwrap()
    );
    let attempt = restored.attempts.get_mut(ID).unwrap();
    let State::CopiedJournal { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation.assignment.edit("journalctl".into()).unwrap();
    assert!(!progress::current_solved(attempt, &scenario));
    let fresh = progress::Attempt::new(&scenario, 5);
    let State::CopiedJournal { simulation } = fresh.state else {
        panic!()
    };
    assert_eq!(simulation.assignment, work::Draft::default());
}

#[test]
fn all_386_predecessor_scenarios_and_147_units_are_exact() {
    let old = pack::accepted386().unwrap();
    let current = pack::accepted387().unwrap();
    assert_eq!((old.len(), current.len()), (386, 387));
    for scenario in &old {
        let retained = current.iter().find(|lab| lab.id == scenario.id).unwrap();
        assert_eq!(
            serde_json::to_value(scenario).unwrap(),
            serde_json::to_value(retained).unwrap()
        );
    }
    let old_units = course::accepted386(&old).unwrap();
    let units = course::accepted387(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (147, 148));
    for unit in old_units {
        let retained = units.iter().find(|item| item.id == unit.id).unwrap();
        assert_eq!(
            serde_json::to_value(&unit).unwrap(),
            serde_json::to_value(retained).unwrap()
        );
    }
    let own = scenario();
    let Goal::CopiedJournal { definition } = &own.goal else {
        panic!()
    };
    assert!(definition.validate().is_ok());
}
