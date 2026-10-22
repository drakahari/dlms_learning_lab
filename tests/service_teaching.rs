use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    service_teaching::{self, Learning},
    workspace,
};
fn lesson() -> service_teaching::Lesson {
    pack::bundled().unwrap()[6]
        .service_teaching
        .clone()
        .unwrap()
}
fn command(l: &mut Learning, command: &str) {
    l.command = command.into();
    l.submit().unwrap();
}
#[test]
fn taught_service_workflow_preserves_diagnosis_saved_runtime_and_boot_distinctions() {
    let lesson = lesson();
    lesson.validate().unwrap();
    let mut l = Learning::new(&lesson);
    let initial = l.state.clone();
    for c in [
        "systemctl status atlas",
        "journalctl -u atlas",
        "cat /etc/atlas.conf",
    ] {
        command(&mut l, c);
    }
    assert_eq!(l.state, initial);
    assert!(l.transcript.iter().any(|s| s.contains("ninety")));
    l.command = "systemctl restart atlas".into();
    assert!(l.submit().is_err());
    l.check();
    assert!(!l.solved(&lesson));
    l.port_draft = "9090".into();
    assert!(l.has_unsaved_port());
    assert!(!l.solved(&lesson));
    l.save_port().unwrap();
    l.check();
    assert_eq!(
        l.checks(&lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
        [true, false, false]
    );
    command(&mut l, "systemctl restart atlas");
    l.check();
    assert_eq!(
        l.checks(&lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
        [true, true, false]
    );
    command(&mut l, "systemctl enable atlas");
    l.check();
    assert!(l.solved(&lesson));
    assert!(l.valid());
    for c in ["systemctl is-active atlas", "systemctl is-enabled atlas"] {
        command(&mut l, c);
    }
    assert!(l.solved(&lesson));
}
#[test]
fn exact_suggested_experiment_and_alternative_order_grade_current_behavior() {
    let lesson = lesson();
    let mut l = Learning::new(&lesson);
    l.port_draft = "9090".into();
    l.save_port().unwrap();
    command(&mut l, "systemctl enable atlas");
    l.check();
    assert!(!l.solved(&lesson));
    command(&mut l, "systemctl start atlas");
    l.check();
    assert!(l.solved(&lesson));
    l.port_draft = "9091".into();
    assert!(!l.solved(&lesson));
    l.save_port().unwrap();
    command(&mut l, "systemctl status atlas");
    assert!(matches!(
        l.state,
        State::Service {
            loaded_port: Some(9090),
            ..
        }
    ));
    l.check();
    assert!(!l.solved(&lesson));
    command(&mut l, "systemctl start atlas");
    assert!(matches!(
        l.state,
        State::Service {
            loaded_port: Some(9090),
            ..
        }
    ));
    command(&mut l, "systemctl restart atlas");
    assert!(matches!(
        l.state,
        State::Service {
            loaded_port: Some(9091),
            ..
        }
    ));
    l.check();
    assert!(!l.solved(&lesson));
    l.port_draft = "9090".into();
    l.save_port().unwrap();
    command(&mut l, "systemctl restart atlas");
    l.check();
    assert!(l.solved(&lesson));
    command(&mut l, "systemctl disable atlas");
    assert!(matches!(
        l.state,
        State::Service {
            running: true,
            enabled: false,
            ..
        }
    ));
    l.check();
    assert!(!l.solved(&lesson));
    command(&mut l, "systemctl enable atlas");
    l.check();
    assert!(l.solved(&lesson));
    command(&mut l, "systemctl stop atlas");
    assert!(!l.solved(&lesson));
}
#[test]
fn saved_service_example_drafts_and_credit_are_separate_and_legacy_history_is_not_invented() {
    let labs = pack::bundled().unwrap();
    let lesson = labs[6].service_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&labs[6], 1);
    workspace::prepare(&mut a);
    let original = a.state.clone();
    service_teaching::prepare(&mut a, lesson, true);
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .service_learning
        .as_mut()
        .unwrap();
    assert_eq!(l.view, View::Introduction);
    l.port_draft = "9090".into();
    l.save_port().unwrap();
    command(l, "systemctl restart atlas");
    command(l, "systemctl enable atlas");
    l.check();
    assert!(l.solved(lesson));
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, &labs[6]));
    a.workspace.as_mut().unwrap().service_port = Some("original unfinished draft".into());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(labs[6].id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let mut old = serde_json::to_value(&p).unwrap();
    old["attempts"][&labs[6].id]["workspace"]
        .as_object_mut()
        .unwrap()
        .remove("service_learning");
    old["attempts"][&labs[6].id]["state"]
        .as_object_mut()
        .unwrap()
        .remove("journal");
    let mut legacy = progress::decode(&serde_json::to_vec(&old).unwrap(), &labs).unwrap();
    let a = legacy.attempts.get_mut(&labs[6].id).unwrap();
    service_teaching::prepare(a, lesson, false);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    assert!(matches!(&a.state,State::Service{journal,..}if journal.is_empty()));
    assert_eq!(
        a.workspace.as_ref().unwrap().service_port.as_deref(),
        Some("original unfinished draft")
    );
}
#[test]
fn service_metadata_bounds_foreign_imports_and_false_real_commands_are_rejected() {
    let labs = pack::bundled().unwrap();
    let lesson = lesson();
    let mut bad = lesson.clone();
    bad.commands[0].command = "config set port 9090".into();
    assert!(bad.validate().is_err());
    bad = lesson.clone();
    bad.required_port = 0;
    assert!(bad.validate().is_err());
    let mut a = Attempt::new(&labs[0], 1);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().service_learning = Some(Learning::new(&lesson));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(labs[0].id.clone(), a);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_err());
    let mut l = Learning::new(&lesson);
    for command in [
        "sh -c echo",
        "sudo systemctl start atlas",
        "systemctl start atlas; rm -r /",
        "cat /home/personal-file",
        "echo $(id)",
    ] {
        let before = l.state.clone();
        l.command = command.into();
        assert!(l.submit().is_err());
        assert_eq!(l.state, before);
    }
    l.command = "x".repeat(4097);
    assert!(l.submit().is_err());
    assert!(!l.valid());
    l.command.clear();
    for _ in 0..150 {
        command(&mut l, "systemctl status atlas");
    }
    assert!(l.valid());
    assert_eq!(l.transcript.len(), 80);
    if let State::Service { port, .. } = &mut l.state {
        *port = "x".repeat(257);
    }
    assert!(!l.valid());
    let mut state = lesson.fixture.clone();
    if let State::Service { journal, .. } = &mut state {
        journal.push(engine::ServiceEvent::Rejected {
            port: "x".repeat(257),
        });
    }
    assert!(pack::validate_state(&state).is_err());
}
#[test]
fn unsaved_original_service_drafts_invalidate_completion_without_changing_saved_work() {
    let labs = pack::bundled().unwrap();
    let mut a = Attempt::new(&labs[6], 1);
    workspace::prepare(&mut a);
    for c in &labs[6].solution {
        engine::execute(&mut a.state, c).unwrap();
    }
    assert!(progress::current_solved(&a, &labs[6]));
    let original = a.state.clone();
    a.workspace.as_mut().unwrap().service_port = Some("broken draft".into());
    assert!(!progress::current_solved(&a, &labs[6]));
    assert_eq!(a.state, original);
    a.workspace.as_mut().unwrap().service_port = Some("8080".into());
    assert!(progress::current_solved(&a, &labs[6]));
}
#[test]
fn bounded_example_journal_keeps_latest_diagnostic_and_discloses_omitted_output() {
    let lesson = lesson();
    let mut l = Learning::new(&lesson);
    let mut latest = String::new();
    for n in 0..70 {
        latest = format!("{n}-{}", "é".repeat(100));
        l.port_draft = latest.clone();
        l.save_port().unwrap();
        l.command = "systemctl restart atlas".into();
        assert!(l.submit().is_err());
    }
    let full = engine::execute(&mut l.state, "journalctl -u atlas").unwrap();
    assert!(full.len() > 8192);
    assert!(full.contains(&latest));
    command(&mut l, "journalctl -u atlas");
    let output = l.transcript.last().unwrap();
    assert!(output.starts_with("[Earlier output omitted"));
    assert!(output.contains(&latest));
    assert!(output.len() <= 8192);
    assert!(l.valid());
}
#[test]
fn supported_compatibility_helper_refreshes_clean_example_editor_but_preserves_unsaved_work() {
    let lesson = lesson();
    let mut l = Learning::new(&lesson);
    command(&mut l, "config set port 9090");
    assert_eq!(l.port_draft, "9090");
    command(&mut l, "systemctl start atlas");
    command(&mut l, "systemctl enable atlas");
    l.check();
    assert!(l.solved(&lesson));
    l.port_draft = "unsaved personal experiment".into();
    command(&mut l, "config set port 9091");
    assert_eq!(l.port_draft, "unsaved personal experiment");
    assert!(matches!(&l.state,State::Service{port,..}if port=="9091"));
    assert!(!l.solved(&lesson));
}
#[test]
fn service_feedback_distinguishes_invalid_wrong_saved_failed_stopped_stale_and_disabled() {
    let lesson = lesson();
    let mut l = Learning::new(&lesson);
    let issues = workspace::issues(&l.state, &lesson.goal());
    assert!(
        issues
            .iter()
            .any(|s| s.contains("ninety") && s.contains("9090"))
    );
    assert!(issues.iter().any(|s| s.contains("failed to start")));
    assert!(issues.iter().any(|s| s.contains("disabled at boot")));
    command(&mut l, "systemctl stop atlas");
    assert!(
        workspace::issues(&l.state, &lesson.goal())
            .iter()
            .any(|s| s.contains("is stopped"))
    );
    l.port_draft = "9091".into();
    l.save_port().unwrap();
    command(&mut l, "systemctl start atlas");
    l.port_draft = "9090".into();
    l.save_port().unwrap();
    assert!(
        workspace::issues(&l.state, &lesson.goal())
            .iter()
            .any(|s| s.contains("running on port 9091") && s.contains("requires 9090"))
    );
    l.port_draft = "9092".into();
    l.save_port().unwrap();
    assert!(
        workspace::issues(&l.state, &lesson.goal())
            .iter()
            .any(|s| s.contains("Saved port is 9092") && s.contains("requires 9090"))
    );
}
