use dlms_learning_lab::{
    dns::QueryType,
    dns_access_workspace::{
        Action, Config, Name,
        session::{Definition, Draft, Session},
    },
};
#[path = "support/dns_access_task.rs"]
mod fixture;
fn solve(draft: &mut Draft, task: &dlms_learning_lab::dns_access_workspace::Task) {
    for action in &task.reference {
        if let Action::Save { config } = action {
            draft.set_editor(config.0.clone()).unwrap();
        }
        draft.run(task, action.clone()).unwrap().unwrap();
    }
    assert!(draft.check(task).unwrap().iter().all(|r| r.correct));
}
#[test]
fn worked_runs_do_not_complete_or_overwrite_the_independent_assignment() {
    let def = fixture::definition();
    def.validate().unwrap();
    let mut session = Session::new(&def).unwrap();
    session
        .assignment
        .set_editor("# unfinished assignment\noptions {".into())
        .unwrap();
    let before = session.assignment.clone();
    solve(&mut session.practice, &def.example);
    assert!(session.practice.checked);
    assert_eq!(session.assignment, before);
    assert!(!session.solved(&def));
    assert_eq!(
        session.practice.world(&def.example).unwrap().zones(),
        fixture::reached(&def.example).zones()
    );
}
#[test]
fn separate_drafts_real_output_and_current_grades_survive_round_trip() {
    let def = fixture::definition();
    let mut session = Session::new(&def).unwrap();
    solve(&mut session.assignment, &def.initial);
    assert!(session.solved(&def));
    session
        .practice
        .set_editor("# kept practice draft\noptions {".into())
        .unwrap();
    session.assignment.query_name = Name("unfinished.example".into());
    session.assignment.query_client.0 = "192.".into();
    let raw = serde_json::to_vec(&session).unwrap();
    let restored: Session = serde_json::from_slice(&raw).unwrap();
    restored.validate(&def).unwrap();
    assert_eq!(restored, session);
    assert!(restored.solved(&def));
    assert!(
        restored
            .assignment
            .last_output(&def.initial)
            .unwrap()
            .unwrap()
            .unwrap()
            .contains("current serial")
    );
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert!(value["assignment"].get("world").is_none());
    assert!(value["assignment"].get("output").is_none());
}
#[test]
fn changed_editor_and_reloaded_bad_policy_clear_success_but_read_queries_keep_it() {
    let def = fixture::definition();
    let mut session = Session::new(&def).unwrap();
    solve(&mut session.assignment, &def.initial);
    let query = Action::Query {
        client: "192.0.2.7".parse().unwrap(),
        name: Name("api.external.example".into()),
        record_type: QueryType::A,
        recursive: true,
    };
    session
        .assignment
        .run(&def.initial, query)
        .unwrap()
        .unwrap();
    assert!(session.solved(&def));
    session
        .assignment
        .set_editor(def.initial.initial_config.0.clone())
        .unwrap();
    assert!(!session.solved(&def));
    assert!(
        session
            .assignment
            .check(&def.initial)
            .unwrap()
            .iter()
            .any(|r| !r.correct && r.label == "Editor saved")
    );
    for action in [
        Action::Save {
            config: def.initial.initial_config.clone(),
        },
        Action::Reload {},
    ] {
        session
            .assignment
            .run(&def.initial, action)
            .unwrap()
            .unwrap();
    }
    assert!(
        session
            .assignment
            .check(&def.initial)
            .unwrap()
            .iter()
            .any(|r| !r.correct)
    );
    assert!(!session.solved(&def));
}
#[test]
fn failed_save_or_unsupported_configuration_is_rederived_without_changing_live_state() {
    let def = fixture::definition();
    let mut draft = Draft::new(&def.initial).unwrap();
    let before = draft.world(&def.initial).unwrap();
    let action = Action::Save {
        config: Config("include \"/etc/passwd\";".into()),
    };
    assert!(
        draft
            .run(&def.initial, action)
            .unwrap()
            .unwrap_err()
            .contains("Unsupported")
    );
    assert_eq!(draft.world(&def.initial).unwrap(), before);
    assert_eq!(draft.history, def.initial.starter);
    let restored: Draft = serde_json::from_str(&serde_json::to_string(&draft).unwrap()).unwrap();
    restored.validate(&def.initial).unwrap();
    assert_eq!(
        restored.last_output(&def.initial).unwrap(),
        draft.last_output(&def.initial).unwrap()
    );
    let mut forged = restored.clone();
    forged.last.as_mut().unwrap().appended = true;
    assert!(forged.validate(&def.initial).is_err());
}
#[test]
fn validation_refuses_foreign_draft_versions_forged_success_and_output_provenance() {
    let def = fixture::definition();
    let session = Session::new(&def).unwrap();
    let mut bad = session.clone();
    bad.version = 2;
    assert!(bad.validate(&def).is_err());
    let mut bad = session.clone();
    bad.assignment.checked = true;
    bad.assignment.attempted = true;
    assert!(bad.validate(&def).is_err());
    let mut value = serde_json::to_value(&session).unwrap();
    value["assignment"]["output"] = serde_json::json!("all tests passed");
    assert!(serde_json::from_value::<Session>(value).is_err());
    let mut solved = session.clone();
    solve(&mut solved.assignment, &def.initial);
    let mut bad = solved.clone();
    bad.assignment.last.as_mut().unwrap().before = 0;
    assert!(bad.validate(&def).is_err());
    let mut bad = solved.clone();
    bad.assignment.history.push(Action::Reload {});
    assert!(bad.validate(&def).is_err());
    let mut bad = solved;
    bad.assignment = bad.practice.clone();
    assert!(bad.validate(&def).is_err());
}
#[test]
fn selected_reset_preserves_the_other_draft_and_alternate_effective_solutions() {
    let def = fixture::definition();
    let mut session = Session::new(&def).unwrap();
    let alternate = fixture::alternate_config();
    session.assignment.set_editor(alternate.clone()).unwrap();
    for action in [
        Action::Save {
            config: Config(alternate),
        },
        Action::Reload {},
        def.initial.reference[2].clone(),
    ] {
        session
            .assignment
            .run(&def.initial, action)
            .unwrap()
            .unwrap();
    }
    assert!(
        session
            .assignment
            .check(&def.initial)
            .unwrap()
            .iter()
            .all(|r| r.correct)
    );
    assert!(session.solved(&def));
    session
        .practice
        .set_editor("# retain this practice".into())
        .unwrap();
    let practice = session.practice.clone();
    session.reset_assignment(&def).unwrap();
    assert_eq!(session.practice, practice);
    assert!(!session.solved(&def));
    let assignment = session.assignment.clone();
    session.reset_practice(&def).unwrap();
    assert_eq!(session.assignment, assignment);
    assert_eq!(def.initial, fixture::task());
}
#[test]
fn bounded_history_and_inherited_starter_cannot_be_erased_or_forged() {
    let mut def: Definition = fixture::definition();
    def.initial.starter = vec![Action::Reload {}];
    def.validate().unwrap();
    let mut draft = Draft::new(&def.initial).unwrap();
    draft.history.clear();
    assert!(draft.validate(&def.initial).is_err());
    let mut draft = Draft::new(&def.initial).unwrap();
    while draft.history.len() < 64 {
        draft
            .run(&def.initial, Action::ClearCache {})
            .unwrap()
            .unwrap();
    }
    let before = draft.world(&def.initial).unwrap();
    assert!(draft.run(&def.initial, Action::Reload {}).unwrap().is_err());
    assert_eq!(draft.history.len(), 64);
    assert_eq!(draft.world(&def.initial).unwrap(), before);
    draft.validate(&def.initial).unwrap();
    let mut value = serde_json::to_value(&draft).unwrap();
    value["history"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"kind":"reload"}));
    assert!(serde_json::from_value::<Draft>(value).is_err());
}
