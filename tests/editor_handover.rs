use dlms_learning_lab::{
    editor_handover as handover, environment_workspace as env, modal_editing::Key,
    modal_workspace as files, pack, shell,
};
use std::collections::BTreeMap;
fn task(id: &str, team: &str) -> env::Task {
    let cases = [("vi", None), ("vim", Some("nano")), ("vi", Some(""))]
        .into_iter()
        .enumerate()
        .map(|(i, (preferred, old))| {
            let mut supplied = BTreeMap::from([
                ("PREFERRED_EDITOR".into(), preferred.into()),
                ("TEAM".into(), team.into()),
            ]);
            if let Some(old) = old {
                supplied.insert("EDITOR".into(), old.into());
            }
            let input = shell::Case {
                name: format!("Sample {}", i + 1),
                args: vec![],
                stdin: String::new(),
                env: supplied.clone(),
                files: BTreeMap::new(),
                after: BTreeMap::new(),
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
            };
            let mut child = input.clone();
            child.env.insert("EDITOR".into(), preferred.into());
            child.stdout = format!("editor={preferred}\nteam={team}\n");
            env::Case {
                input,
                parent: BTreeMap::from([(
                    "EDITOR".into(),
                    shell::EnvironmentVariable {
                        value: Some(preferred.into()),
                        exported: true,
                    },
                )]),
                child,
            }
        })
        .collect();
    env::Task { id: id.into(), instructions: "Use the supplied PREFERRED_EDITOR value for exported EDITOR. Preserve the other supplied settings and do not print setup output.".into(), starter: "# Configure the simulated child editor.\n".into(), child_source: "printf 'editor=%s\\nteam=%s\\n' \"$EDITOR\" \"$TEAM\"".into(), cases, path_lookup: None }
}
fn definition() -> handover::Definition {
    let labs = pack::load(include_str!("../scenarios/modal-editing-lessons-v1.json")).unwrap();
    let dlms_learning_lab::engine::Goal::ModalEditing { definition } = &labs[0].goal else {
        panic!()
    };
    let mut editing = (**definition).clone();
    editing.id = "default-editor-test".into();
    handover::Definition {
        id: editing.id.clone(),
        version: 1,
        setup: env::Definition {
            id: "default-editor-test-setup".into(),
            assignment: task("editor-test-assignment", "operations"),
            example: task("editor-test-example", "training"),
        },
        editing,
        example_setup: "export EDITOR=\"$PREFERRED_EDITOR\"".into(),
    }
}
fn run(s: &mut handover::Session, d: &handover::Definition, practice: bool, source: &str) {
    s.edit_setup(d, practice, source.into()).unwrap();
    let result = env::run(s.task(d, practice), source).unwrap();
    s.store_setup(d, practice, result).unwrap();
}
fn key(s: &mut handover::Session, d: &handover::Definition, text: &str) {
    for c in text.chars() {
        s.apply_file(
            d,
            false,
            &files::Event::Key {
                key: match c {
                    '\r' => Key::Enter,
                    '\x1b' => Key::Escape,
                    _ => Key::Character(c),
                },
            },
        )
        .unwrap();
    }
}
fn repair(s: &mut handover::Session, d: &handover::Definition) {
    key(s, d, "ggddddddi");
    s.apply_file(
        d,
        false,
        &files::Event::Insert {
            text: "name=edge-a\nport=8080\nworkers=4".into(),
        },
    )
    .unwrap();
    key(s, d, "\x1b:w\r");
    s.apply_file(d, false, &files::Event::Check {}).unwrap();
}
#[test]
fn taught_method_and_equivalent_setup_pass_only_with_actual_saved_repairs() {
    let d = definition();
    d.validate_example().unwrap();
    for source in [
        "export EDITOR=\"$PREFERRED_EDITOR\"",
        "EDITOR=$PREFERRED_EDITOR\nexport EDITOR",
        "export -n EDITOR\nEDITOR=\"${PREFERRED_EDITOR}\"\nexport EDITOR",
    ] {
        let mut s = d.new_session().unwrap();
        assert!(!s.solved(&d));
        run(&mut s, &d, false, source);
        assert!(s.ready(&d, false), "{source}: {:?}", s.checks(&d, false));
        assert!(!s.opened(&d, false));
        assert!(!s.solved(&d));
        s.open_editor(&d, false).unwrap();
        assert_eq!(s.assignment_launch.as_ref().unwrap().editor, "vi");
        repair(&mut s, &d);
        assert!(s.solved(&d), "{:?}", s.checks(&d, false));
        key(&mut s, &d, "x");
        assert!(!s.solved(&d));
        assert!(!s.files.assignment.checked);
    }
}
#[test]
fn missing_export_hardcoding_unsupported_names_and_faked_output_cannot_open_editor() {
    let d = definition();
    for source in [
        "EDITOR=$PREFERRED_EDITOR",
        "export EDITOR=vi",
        "export EDITOR=nano",
        "printf 'editor=vi\\nteam=operations\\n'",
        "export EDITOR='/usr/bin/vi -u NONE'",
        "export EDITOR=\"$(curl example.invalid)\"",
        "unset EDITOR\nEDITOR=$PREFERRED_EDITOR\nexport EDITOR",
    ] {
        let mut s = d.new_session().unwrap();
        run(&mut s, &d, false, source);
        assert!(!s.ready(&d, false), "{source}");
        assert!(s.open_editor(&d, false).is_err());
        assert!(
            s.apply_file(
                &d,
                false,
                &files::Event::Key {
                    key: Key::Character('x')
                }
            )
            .is_err()
        );
        assert!(!s.solved(&d));
    }
    let mut s = d.new_session().unwrap();
    run(&mut s, &d, false, "EDITOR=$PREFERRED_EDITOR");
    let rows = s.checks(&d, false);
    assert!(
        rows.iter().any(|r| !r.0 && r.1.contains("exported")),
        "{rows:?}"
    );
}
#[test]
fn setup_edits_invalidate_launch_and_success_without_destroying_file_drafts() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    run(&mut s, &d, false, &d.example_setup);
    s.open_editor(&d, false).unwrap();
    repair(&mut s, &d);
    assert!(s.solved(&d));
    let saved = s.files.assignment.clone();
    s.edit_setup(&d, false, "EDITOR=wrong".into()).unwrap();
    assert!(!s.opened(&d, false));
    assert!(!s.solved(&d));
    assert_eq!(s.files.assignment.editors, saved.editors);
    assert!(!s.files.assignment.checked);
    run(&mut s, &d, false, &d.example_setup);
    s.open_editor(&d, false).unwrap();
    s.apply_file(&d, false, &files::Event::Check {}).unwrap();
    assert!(s.solved(&d));
}
#[test]
fn practice_reset_resume_and_unverified_records_keep_credit_honest() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    let assignment = s.files.assignment.clone();
    run(&mut s, &d, true, &d.example_setup);
    s.open_editor(&d, true).unwrap();
    s.files.show_worked(&d.editing).unwrap();
    s.apply_file(&d, true, &files::Event::Check {}).unwrap();
    assert!(s.checks(&d, true).iter().all(|r| r.0));
    assert_eq!(s.files.assignment, assignment);
    assert!(!s.solved(&d));
    run(&mut s, &d, false, &d.example_setup);
    s.open_editor(&d, false).unwrap();
    key(&mut s, &d, "iX");
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut resumed: handover::Session = serde_json::from_slice(&bytes).unwrap();
    d.context(&resumed).unwrap();
    assert_eq!(resumed.files.assignment, s.files.assignment);
    assert_eq!(resumed.setup.assignment.source, s.setup.assignment.source);
    assert!(!resumed.ready(&d, false));
    assert!(!resumed.solved(&d));
    let record = env::run(resumed.task(&d, false), &resumed.draft(false).source).unwrap();
    resumed.store_setup(&d, false, record).unwrap();
    resumed.open_editor(&d, false).unwrap();
    assert!(
        resumed
            .files
            .assignment
            .editor()
            .contents()
            .starts_with('X')
    );
    let practice = resumed.files.practice.clone();
    resumed.reset(&d, false).unwrap();
    assert_eq!(
        resumed.files.assignment,
        d.new_session().unwrap().files.assignment
    );
    assert_eq!(resumed.files.practice, practice);
}
#[test]
fn bounded_setup_can_stop_and_bad_source_or_forged_launch_is_rejected() {
    let d = definition();
    let s = d.new_session().unwrap();
    let mut job = env::Job::new(s.task(&d, false), "while :; do :; done").unwrap();
    job.advance(16);
    job.stop();
    assert!(job.done());
    assert!(job.record().is_none());
    let mut job = env::Job::new(s.task(&d, false), "while :; do :; done").unwrap();
    for _ in 0..100_000 {
        if job.done() {
            break;
        }
        job.advance(64);
    }
    assert!(job.done());
    let record = job.record().expect("The per-sample instruction cap should produce bounded failure observations before the whole-job cap");
    assert_eq!(record.results.len(), 3);
    assert!(record.results.iter().all(|r| {
        r.parent
            .error
            .as_deref()
            .is_some_and(|e| e.contains("2,000 instructions"))
            && r.child.is_none()
            && r.bindings.is_none()
    }));
    let mut limited = s.clone();
    limited
        .edit_setup(&d, false, "while :; do :; done".into())
        .unwrap();
    limited.store_setup(&d, false, record).unwrap();
    assert!(!limited.ready(&d, false));
    assert!(limited.open_editor(&d, false).is_err());
    assert!(!limited.solved(&d));
    let mut invalid = s.clone();
    assert!(invalid.edit_setup(&d, false, "x".repeat(8193)).is_err());
    assert_eq!(invalid, s);
    invalid.assignment_launch = Some(handover::Launch {
        sample: "Sample 1".into(),
        editor: "vi".into(),
    });
    assert!(d.context(&invalid).is_err());
    let mut definition = d.clone();
    definition.setup.assignment.cases[0]
        .parent
        .get_mut("EDITOR")
        .unwrap()
        .value = Some("/bin/vi".into());
    assert!(definition.validate().is_err());
    let mut definition = d;
    definition.example_setup = "x".repeat(8193);
    assert!(definition.validate().is_err());
}
#[test]
fn beginning_a_new_run_invalidates_previous_success_before_any_execution() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    run(&mut s, &d, false, &d.example_setup);
    s.open_editor(&d, false).unwrap();
    repair(&mut s, &d);
    assert!(s.solved(&d));
    let saved = s.files.assignment.editors.clone();
    s.begin_run(&d, false).unwrap();
    assert!(s.setup.assignment.result.is_none());
    assert!(s.assignment_launch.is_none());
    assert!(!s.files.assignment.checked);
    assert!(!s.solved(&d));
    assert_eq!(s.files.assignment.editors, saved);
}
