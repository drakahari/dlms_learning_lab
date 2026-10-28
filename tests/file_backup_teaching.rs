use dlms_learning_lab::{
    engine::Node,
    file_backup::{Entry, Tree},
    file_backup_teaching::{Command, Concept, Experiment, Learning, Lesson},
    file_backup_terminal::Store,
    file_backup_workspace::{Definition, Requirement},
    script_teaching::View,
};
use std::collections::BTreeMap;
fn sample() -> Lesson {
    let metadata = Node {
        directory: true,
        mode: 0o750,
        owner: "sam".into(),
        group: "docs".into(),
    };
    let empty = Tree {
        clock: 0,
        entries: BTreeMap::from([(
            "".into(),
            Entry {
                node: metadata.clone(),
                text: None,
                modified: 0,
                changed: 0,
            },
        )]),
    };
    let mut source = empty.clone();
    let mut file = metadata;
    file.directory = false;
    file.mode = 0o640;
    source
        .add("guide.txt", file, Some("sample reference\n"))
        .unwrap();
    let store = Store {
        trees: BTreeMap::from([
            ("/demo/projects".into(), source.clone()),
            ("/example/projects".into(), empty),
        ]),
        storage: "/example-backups".into(),
        archives: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        cwd: "/demo".into(),
    };
    let extract = Command {
        command: "tar -xf /example-backups/projects.tar -C /example".into(),
        explanation:
            "Restore the saved folder into the example destination; source files stay unchanged."
                .into(),
    };
    Lesson{definition:Definition{version:1,initial:store,protected:vec!["/demo/projects".into()],requirements:vec![Requirement::Tree{name:"Restore the example".into(),path:"/example/projects".into(),expected:source}]},prerequisite_note:"Use the terminal field for one command at a time. These files belong only to this example.".into(),concepts:vec![Concept{title:"A backup is a separate saved copy".into(),text:"An archive keeps the saved file contents after the original changes. Creating an archive and restoring it are different operations; list its members before choosing what to restore.".into()}],example_description:"Save the sample projects folder, then restore its guide into the separate example destination.".into(),example_commands:vec![Command{command:"tar -cf /example-backups/projects.tar -C /demo projects".into(),explanation:"Create (-c) the archive named after -f. Look in the directory after -C for the projects folder.".into()},extract.clone()],try_change:Experiment{description:"Make the restored guide world-writable. Check why a file with the right text can still be an incorrect restore.".into(),commands:vec![Command{command:"chmod 777 /example/projects/guide.txt".into(),explanation:"This changes virtual access permissions, not the saved archive. The restore goal also checks metadata.".into()}],expected:vec![true,false]},repair:vec![extract]}
}
fn run_example(learning: &mut Learning, lesson: &Lesson) {
    for step in &lesson.example_commands {
        learning.command = step.command.clone();
        learning.execute(lesson).unwrap();
    }
}
#[test]
fn worked_example_experiment_and_repair_all_execute_in_the_simulator() {
    let lesson = sample();
    lesson.validate().unwrap();
    let mut learning = Learning::new(&lesson).unwrap();
    assert!(learning.valid_for(&lesson));
    assert!(
        lesson
            .definition
            .checks(&learning.state)
            .iter()
            .any(|r| !r.0)
    );
    run_example(&mut learning, &lesson);
    assert!(
        lesson
            .definition
            .checks(&learning.state)
            .iter()
            .all(|r| r.0)
    );
    for c in &lesson.try_change.commands {
        learning.command = c.command.clone();
        learning.execute(&lesson).unwrap();
    }
    assert_eq!(
        lesson
            .definition
            .checks(&learning.state)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        lesson.try_change.expected
    );
    for c in &lesson.repair {
        learning.command = c.command.clone();
        learning.execute(&lesson).unwrap();
    }
    assert!(
        lesson
            .definition
            .checks(&learning.state)
            .iter()
            .all(|r| r.0)
    );
}
#[test]
fn separate_example_namespaces_prevent_assignment_collisions() {
    let lesson = sample();
    let mut assignment = lesson.definition.initial.clone();
    assignment.storage = "/assignment-backups".into();
    assignment.trees = assignment
        .trees
        .into_iter()
        .map(|(p, t)| {
            (
                p.replace("/demo/", "/work/")
                    .replace("/example/", "/restore/"),
                t,
            )
        })
        .collect();
    assignment.cwd = "/work".into();
    assert!(assignment.valid());
    assert!(lesson.separate_from(&assignment));
    let original = assignment.clone();
    let mut learning = Learning::new(&lesson).unwrap();
    run_example(&mut learning, &lesson);
    assert_eq!(assignment, original);
    let same = lesson.definition.initial.clone();
    assert!(!lesson.separate_from(&same));
    let mut nested = assignment.clone();
    nested.storage = "/demo/projects/backups".into();
    assert!(!lesson.separate_from(&nested));
}
#[test]
fn introduction_example_assignment_navigation_and_unexecuted_draft_survive_roundtrip() {
    let lesson = sample();
    let mut learning = Learning::new(&lesson).unwrap();
    learning.command = "tar -tf /example-backups/projects.tar".into();
    learning.example_command = 1;
    for view in [View::Example, View::Assignment, View::Introduction] {
        learning.view = view;
        let bytes = serde_json::to_vec(&learning).unwrap();
        let restored: Learning = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, learning);
        assert_eq!(restored.command, "tar -tf /example-backups/projects.tar");
        assert!(restored.valid_for(&lesson));
    }
    assert!(learning.state.archives.is_empty());
    assert!(!learning.checked);
}
#[test]
fn bad_commands_keep_retry_draft_and_previous_state_with_helpful_diagnostics() {
    let lesson = sample();
    let mut learning = Learning::new(&lesson).unwrap();
    let initial = learning.state.clone();
    for command in [
        "tar -cf /example-backups/bad.tar ../../personal",
        "tar -tvf /example-backups/missing.tar",
        "bash /host/script",
        "tar -cf /example-backups/o.tar -C /demo projects; rm -rf /",
    ] {
        learning.command = command.into();
        learning.checked = true;
        let error = learning.execute(&lesson).unwrap_err();
        assert!(!error.is_empty());
        assert_eq!(learning.command, command);
        assert_eq!(learning.state, initial);
        assert!(!learning.checked);
        assert!(learning.transcript.last().unwrap().contains(&error));
    }
    run_example(&mut learning, &lesson);
    assert!(
        lesson
            .definition
            .checks(&learning.state)
            .iter()
            .all(|r| r.0)
    );
}
#[test]
fn transcript_and_draft_limits_are_bounded_without_discarding_the_current_draft() {
    let lesson = sample();
    let mut learning = Learning::new(&lesson).unwrap();
    for _ in 0..40 {
        learning.command = "pwd".into();
        learning.execute(&lesson).unwrap();
    }
    assert_eq!(learning.transcript.len(), 16);
    assert!(learning.valid());
    let mut bad = learning.clone();
    bad.command = "x".repeat(4097);
    assert!(!bad.valid());
    assert!(bad.execute(&lesson).is_err());
    let mut raw = serde_json::to_value(&learning).unwrap();
    raw["transcript"] = serde_json::Value::Array(vec![serde_json::json!("x"); 17]);
    assert!(serde_json::from_value::<Learning>(raw).is_err());
    let mut raw = serde_json::to_value(&learning).unwrap();
    raw["command"] = serde_json::json!("x".repeat(4097));
    assert!(serde_json::from_value::<Learning>(raw).is_err());
}
#[test]
fn lesson_loading_rejects_unreachable_examples_wrong_experiments_and_untaught_actions() {
    let valid = sample();
    assert_eq!(
        Lesson::from_json(&serde_json::to_vec(&valid).unwrap()).unwrap(),
        valid
    );
    let mut wrong = valid.clone();
    wrong.example_commands.pop();
    assert!(wrong.validate().is_err());
    let mut wrong = valid.clone();
    wrong.try_change.expected = vec![true, true];
    assert!(wrong.validate().is_err());
    let mut wrong = valid.clone();
    wrong.repair[0].command = "pwd".into();
    assert!(wrong.validate().is_err());
    let mut wrong = valid.clone();
    wrong.example_commands[0].explanation.clear();
    assert!(wrong.validate().is_err());
    let mut wrong = valid.clone();
    wrong.concepts.clear();
    assert!(wrong.validate().is_err());
    let mut wrong = valid.clone();
    wrong.example_commands[0].command = "tar -czf /example-backups/x.tar projects".into();
    assert!(wrong.validate().is_err());
    assert!(Lesson::from_json(&vec![b' '; 2 * 1024 * 1024 + 1]).is_err());
}
#[test]
fn altered_example_context_is_rejected_and_reset_is_selected_practice_only() {
    let lesson = sample();
    let mut learning = Learning::new(&lesson).unwrap();
    run_example(&mut learning, &lesson);
    let worked = learning.clone();
    learning.state.storage = "/foreign-backups".into();
    assert!(!learning.valid_for(&lesson));
    let original = learning.clone();
    assert!(learning.execute(&lesson).is_err());
    assert_eq!(learning, original);
    learning = Learning::new(&lesson).unwrap();
    assert!(learning.state.archives.is_empty());
    assert_eq!(learning.view, View::Introduction);
    assert!(
        worked
            .state
            .archives
            .contains_key("/example-backups/projects.tar")
    );
}
#[test]
fn practice_retains_its_complete_origin_instead_of_relabeling_same_path_work() {
    let lesson = sample();
    let mut learning = Learning::new(&lesson).unwrap();
    run_example(&mut learning, &lesson);
    let before = learning.clone();
    let mut different = lesson.clone();
    different
        .definition
        .initial
        .trees
        .get_mut("/demo/projects")
        .unwrap()
        .write("guide.txt", "different supplied example\n")
        .unwrap();
    assert!(different.bounded());
    assert!(!learning.valid_for(&different));
    assert!(learning.execute(&different).is_err());
    assert_eq!(learning, before);
    let raw = serde_json::to_vec(&learning).unwrap();
    let restored: Learning = serde_json::from_slice(&raw).unwrap();
    assert_eq!(restored.context, lesson.definition);
    assert!(restored.valid_for(&lesson));
}
