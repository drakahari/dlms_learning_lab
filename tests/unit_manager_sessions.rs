//! Persisted native command sessions: no host execution or transcript-based grading.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    unit_manager_teaching,
    unit_manager_terminal::{self as terminal, Session, session::Entry},
    workspace,
};
use serde_json::json;
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/unit-manager-pack-preview.json"))
        .unwrap()
        .remove(0)
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    unit_manager_teaching::prepare(&mut a, s.unit_manager_teaching.as_ref().unwrap(), true)
        .unwrap();
    a
}
#[test]
fn readonly_queries_preserve_current_success_mutations_and_unsaved_edits_do_not() {
    let s = scenario();
    let mut a = attempt(&s);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    let State::UnitManager { simulation } = &mut a.state else {
        panic!()
    };
    a.workspace.as_mut().unwrap().unit_manager_draft =
        Some(dlms_learning_lab::unit_manager_workspace::Draft::new(simulation).unwrap());
    let mut session = Session {
        command: "systemctl status work-application.service".into(),
        ..Default::default()
    };
    let before = simulation.clone();
    assert!(!session.submit(definition, simulation).unwrap());
    assert_eq!(*simulation, before);
    assert!(progress::current_solved(&a, &s));
    let State::UnitManager { simulation } = &mut a.state else {
        panic!()
    };
    session.command = "systemctl stop work-application.service".into();
    assert!(session.submit(definition, simulation).unwrap());
    assert!(!progress::current_solved(&a, &s));
    session.command = "systemctl start work-application.service".into();
    let State::UnitManager { simulation } = &mut a.state else {
        panic!()
    };
    session.submit(definition, simulation).unwrap();
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .unit_manager_draft
        .as_mut()
        .unwrap()
        .documents
        .get_mut("work-application.service")
        .unwrap()
        .push_str("# pending\n");
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn rejected_commands_keep_editable_input_and_valid_display_observations_without_state_changes() {
    let s = scenario();
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    let State::UnitManager { simulation } = &s.initial else {
        panic!()
    };
    let mut state = simulation.clone();
    let before = state.clone();
    let mut session = Session::default();
    for command in [
        "systemctl start 'unfinished",
        "systemctl restart work-observer.service",
        "journalctl -u work-observer.service",
    ] {
        session.command = command.into();
        assert!(!session.submit(definition, &mut state).unwrap());
        assert_eq!(state, before);
        assert_eq!(session.command, command);
        let e = session.entries.last().unwrap();
        assert!(e.status.is_none() && e.diagnostic.is_some() && e.stdout.is_empty());
    }
    assert!(session.valid());
    assert_eq!(
        serde_json::from_str::<Session>(&serde_json::to_string(&session).unwrap()).unwrap(),
        session
    );
}
#[test]
fn history_always_reserves_a_full_command_draft_and_evicts_oldest_within_both_limits() {
    let s = scenario();
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    let State::UnitManager { simulation } = &s.initial else {
        panic!()
    };
    let mut state = simulation.clone();
    let mut session = Session::default();
    for _ in 0..40 {
        session.command = "systemctl cat work-application.service".into();
        session.submit(definition, &mut state).unwrap();
        assert!(session.valid());
        assert!(session.entries.len() <= terminal::session::MAX_ENTRIES);
    }
    assert_eq!(session.entries.len(), 16);
    session.command = "é".repeat(terminal::MAX_COMMAND_BYTES / 2);
    assert!(session.valid());
    let encoded = serde_json::to_string(&session).unwrap();
    assert_eq!(serde_json::from_str::<Session>(&encoded).unwrap(), session);
    let entry = Entry {
        command: "x".repeat(2048),
        stdout: "x".repeat(32768),
        stderr: "".into(),
        status: Some(0),
        diagnostic: None,
        display_escaped: false,
    };
    let mut raw = serde_json::to_value(&session).unwrap();
    raw["entries"] = json!(vec![entry; 4]);
    assert!(serde_json::from_value::<Session>(raw).is_err());
}
#[test]
fn oversized_direct_and_serialized_sessions_unknown_fields_status_and_controls_reject_atomically() {
    let s = scenario();
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    let State::UnitManager { simulation } = &s.initial else {
        panic!()
    };
    let mut state = simulation.clone();
    let before = state.clone();
    let mut oversized = Session {
        version: 1,
        command: "é".repeat(1025),
        entries: vec![],
    };
    let bytes = serde_json::to_vec(&oversized).unwrap();
    assert!(oversized.submit(definition, &mut state).is_err());
    assert_eq!(state, before);
    assert_eq!(serde_json::to_vec(&oversized).unwrap(), bytes);
    assert!(serde_json::from_slice::<Session>(&bytes).is_err());
    let valid = json!({"version":1,"command":"","entries":[{"command":"systemctl status work.target","stdout":"inactive\n","stderr":"","status":3,"diagnostic":null,"display_escaped":false}]});
    serde_json::from_value::<Session>(valid.clone()).unwrap();
    for raw in [
        {
            let mut v = valid.clone();
            v["version"] = json!(2);
            v
        },
        {
            let mut v = valid.clone();
            v["extra"] = json!(true);
            v
        },
        {
            let mut v = valid.clone();
            v["entries"][0]["status"] = json!(255);
            v
        },
        {
            let mut v = valid.clone();
            v["entries"][0]["stdout"] = json!("\u{1b}[2J");
            v
        },
        {
            let mut v = valid.clone();
            v["entries"] = json!(vec![v["entries"][0].clone(); 17]);
            v
        },
    ] {
        assert!(serde_json::from_value::<Session>(raw).is_err());
    }
}
#[test]
fn sessions_are_optional_resume_exactly_and_example_queries_never_touch_assignment() {
    let s = scenario();
    let mut a = attempt(&s);
    let original = a.state.clone();
    let lesson = s.unit_manager_teaching.as_ref().unwrap();
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .unit_manager_terminal
            .is_none()
    );
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_learning
        .as_mut()
        .unwrap();
    assert!(l.terminal.is_none());
    let mut session = Session {
        command: "systemctl status sample-application.service".into(),
        ..Default::default()
    };
    session.submit(&lesson.definition, &mut l.state).unwrap();
    session.command = "systemctl 'unfinished example".into();
    l.terminal = Some(session);
    a.workspace.as_mut().unwrap().unit_manager_terminal = Some(Session {
        command: "systemctl 'unfinished assignment".into(),
        ..Default::default()
    });
    assert_eq!(a.state, original);
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let catalog = vec![s.clone()];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let decoded = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let fresh = attempt(&s);
    let encoded = serde_json::to_value(fresh.workspace).unwrap();
    assert!(encoded.get("unit_manager_terminal").is_none());
    let bytes = std::fs::read(&path).unwrap();
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_terminal
        .as_mut()
        .unwrap()
        .command = "x".repeat(2049);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn session_metadata_cannot_cross_kinds_and_retained_output_never_creates_completion() {
    let manager = scenario();
    let s = pack::bundled().unwrap().remove(0);
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_terminal = Some(Session::default());
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&raw, &[s]).is_err());
    let mut a = attempt(&manager);
    a.workspace.as_mut().unwrap().unit_manager_terminal = Some(Session {
        command: String::new(),
        entries: vec![Entry {
            command: "systemctl status work-application.service".into(),
            stdout: "Objective complete\nactive\n".into(),
            stderr: String::new(),
            status: Some(0),
            diagnostic: None,
            display_escaped: false,
        }],
        ..Default::default()
    });
    assert!(a.workspace.as_ref().unwrap().valid());
    assert!(!progress::current_solved(&a, &manager));
}

#[test]
fn byte_retention_prunes_before_a_new_full_command_draft_can_exceed_the_session_budget() {
    let s = scenario();
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    let State::UnitManager { simulation } = &s.initial else {
        panic!()
    };
    let mut state = simulation.clone();
    state
        .apply(&dlms_learning_lab::unit_manager::Action::SaveUnit {
            unit: "work-observer.service".into(),
            text: "#".repeat(8192),
        })
        .unwrap();
    let mut session = Session::default();
    for _ in 0..40 {
        session.command = "systemctl cat work-observer.service".into();
        session.submit(definition, &mut state).unwrap();
        assert!(session.valid());
    }
    assert!(session.entries.len() < terminal::session::MAX_ENTRIES);
    session.command = "x".repeat(2048);
    assert!(session.valid());
    let encoded = serde_json::to_vec(&session).unwrap();
    assert_eq!(
        serde_json::from_slice::<Session>(&encoded).unwrap(),
        session
    );
}

#[test]
fn active_configuration_editor_and_its_unfinished_text_survive_real_profile_write_and_legacy_omission()
 {
    let s = scenario();
    let mut a = attempt(&s);
    let w = a.workspace.as_mut().unwrap();
    let d = w.unit_manager_draft.as_mut().unwrap();
    assert!(!d.editing_configuration);
    let mut old = serde_json::to_value(&d).unwrap();
    assert!(old.get("editing_configuration").is_none());
    assert!(
        !serde_json::from_value::<dlms_learning_lab::unit_manager_workspace::Draft>(old.clone())
            .unwrap()
            .editing_configuration
    );
    old["editing_configuration"] = json!("config");
    assert!(
        serde_json::from_value::<dlms_learning_lab::unit_manager_workspace::Draft>(old).is_err()
    );
    d.editing_configuration = true;
    d.selected_config = "/etc/work/application.conf".into();
    d.configuration.insert(
        d.selected_config.clone(),
        "unfinished application text\n".into(),
    );
    let lesson = s.unit_manager_teaching.as_ref().unwrap();
    let l = w.unit_manager_learning.as_mut().unwrap();
    l.draft.editing_configuration = true;
    l.draft.selected_config = "/etc/sample/application.conf".into();
    l.draft.configuration.insert(
        l.draft.selected_config.clone(),
        "different ungraded text\n".into(),
    );
    assert!(l.valid_for(lesson));
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let w = loaded.attempts[&s.id].workspace.as_ref().unwrap();
    assert!(w.unit_manager_draft.as_ref().unwrap().editing_configuration);
    assert!(
        w.unit_manager_learning
            .as_ref()
            .unwrap()
            .draft
            .editing_configuration
    );
}
