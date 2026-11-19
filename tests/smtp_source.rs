#[path = "support/smtp_source_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    smtp_source as binding,
    smtp_workspace::{self as work, Action},
};
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::SmtpRouting { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn source_actions_reach_delivery_and_policy_results_and_old_output_cannot_override_broken_saved_settings()
 {
    let s = fixture::scenario();
    let mut a = Attempt::new(&s, 1);
    binding::context(&a, &s).unwrap();
    assert!(!progress::current_solved(&a, &s));
    for line in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let d = binding::source(&s).unwrap();
    session(&mut a).assignment.runtime.files.insert(
        "relay".into(),
        d.assignment.facts.initial_files["relay"].clone(),
    );
    assert!(session(&mut a).assignment.checked);
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[s]}).to_string())
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn real_progress_file_roundtrip_retains_separate_drafts_histories_favorites_and_uncredited_practice()
 {
    let s = fixture::scenario();
    let d = binding::source(&s).unwrap();
    let mut a = Attempt::new(&s, 1);
    for action in &d.example_steps {
        session(&mut a).action(d, action, true).unwrap();
    }
    session(&mut a)
        .action(
            d,
            &Action::Edit {
                node: "relay".into(),
                text: "unfinished assignment text".into(),
            },
            false,
        )
        .unwrap();
    session(&mut a).assignment.input = "postfix re".into();
    session(&mut a).practice.input = "postqueue ".into();
    let wanted = session(&mut a).clone();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let mut r = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert!(r.favorites.contains(&s.id));
    assert_eq!(*session(r.attempts.get_mut(&s.id).unwrap()), wanted);
    assert!(!progress::current_solved(&r.attempts[&s.id], &s));
    let bytes = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &r).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    session(r.attempts.get_mut(&s.id).unwrap()).source_id = "foreign-source".into();
    assert!(progress::save_with_catalog(&path, &r, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn malformed_foreign_and_goal_free_actions_fail_before_other_interpreters() {
    let s = fixture::scenario();
    for input in [
        "sh -c whoami",
        "curl real.invalid",
        "smtp-workspace {bad}",
        "smtp-workspace {\"kind\":\"run\",\"input\":\"postconf -n\",\"host\":true}",
    ] {
        let mut state = s.initial.clone();
        let before = state.clone();
        assert!(engine::execute_for_goal(&mut state, &s.goal, input).is_err());
        assert_eq!(state, before);
    }
    let mut state = s.initial.clone();
    let before = state.clone();
    assert!(engine::execute(&mut state, &s.solution[0]).is_err());
    assert_eq!(state, before);
    let mut a = Attempt::new(&s, 1);
    a.scenario_version += 1;
    assert!(binding::context(&a, &s).is_err());
    let mut a = Attempt::new(&s, 1);
    session(&mut a).source_id = "other-task".into();
    assert!(binding::context(&a, &s).is_err());
}
#[test]
fn pack_refuses_mismatched_initial_work_wrong_track_and_unreachable_references() {
    let s = fixture::scenario();
    let mut bad = s.clone();
    if let State::SmtpRouting { simulation } = &mut bad.initial {
        simulation.assignment.checked = true;
    }
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    bad.track = "LPIC-1".into();
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    if let Goal::SmtpRouting { definition } = &mut bad.goal {
        definition.reference = vec![Action::Check];
    }
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[bad]}).to_string()).is_err()
    );
    let mut bad = s;
    bad.guide[0].commands[0].command = "smtp-workspace {}".into();
    assert!(binding::source(&bad).is_err());
}
