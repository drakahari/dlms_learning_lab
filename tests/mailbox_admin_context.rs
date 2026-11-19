//! Synthetic source-bound capability fixtures, not registered/authored lessons.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    mailbox_source as binding,
    mailbox_workspace::{Action, Session},
    pack,
    progress::{self, Attempt, Progress},
};
use std::sync::OnceLock;
fn original() -> pack::Scenario {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::current().unwrap())
        .iter()
        .find(|s| s.id == "lpic2-dovecot-secure-access")
        .unwrap()
        .clone()
}
fn source() -> pack::Scenario {
    let mut s = original();
    s.id = "synthetic-admin-context".into();
    s.prerequisites.clear();
    s.guide.clear();
    let Goal::MailboxAccess { mut definition } = s.goal else {
        panic!()
    };
    definition.source_id = s.id.clone();
    s.initial = State::MailboxAdministration {
        simulation: Box::new(Session::new(&definition).unwrap()),
    };
    s.goal = Goal::MailboxAdministration { definition };
    binding::source(&s).unwrap();
    s
}
fn line(a: Action) -> String {
    format!("{}{}", binding::PREFIX, serde_json::to_string(&a).unwrap())
}
#[test]
fn administration_reference_runs_real_queries_and_requires_fresh_checks() {
    let s = source();
    let mut state = s.initial.clone();
    for a in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, a).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    let out = engine::execute_for_goal(
        &mut state,
        &s.goal,
        &line(Action::Admin {
            line: "doveconf -h mail_path".into(),
        }),
    )
    .unwrap();
    assert!(out.ends_with("/srv/mail/%{user}/Maildir\n"));
    assert!(!engine::solved(&state, &s.goal));
    engine::execute_for_goal(&mut state, &s.goal, &line(Action::Check)).unwrap();
    assert!(engine::solved(&state, &s.goal));
    assert!(engine::help(&state).contains("doveadm"));
    assert!(engine::execute(&mut state, "doveconf ssl").is_err());
}
#[test]
fn protocol_references_reject_admin_actions_and_progress_rejects_swapped_activity_kinds() {
    let old = original();
    let mut state = old.initial.clone();
    let before = state.clone();
    let e = engine::execute_for_goal(
        &mut state,
        &old.goal,
        &line(Action::Admin {
            line: "doveconf ssl".into(),
        }),
    )
    .unwrap_err();
    assert!(e.contains("separate administration"));
    assert_eq!(state, before);
    let s = source();
    let mut a = Attempt::new(&s, 1);
    binding::context(&a, &s).unwrap();
    let State::MailboxAdministration { simulation } = &a.state else {
        panic!()
    };
    a.state = State::MailboxAccess {
        simulation: simulation.clone(),
    };
    assert!(binding::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn separate_admin_drafts_inputs_and_history_survive_real_file_resume_then_selected_reset() {
    let s = source();
    let d = binding::source(&s).unwrap();
    let mut a = Attempt::new(&s, 1);
    let State::MailboxAdministration { simulation } = &mut a.state else {
        panic!()
    };
    for (practice, prefix) in [(false, "Assignment"), (true, "Example")] {
        simulation
            .action(
                d,
                &Action::Admin {
                    line: "doveconf -h ssl".into(),
                },
                practice,
            )
            .unwrap();
        let task = if practice { &d.example } else { &d.assignment };
        simulation
            .action(
                d,
                &Action::Edit {
                    text: format!("# {prefix} pending\n{}", task.saved),
                },
                practice,
            )
            .unwrap();
        simulation
            .action(
                d,
                &Action::Input {
                    text: format!("doveconf -h {prefix}"),
                },
                practice,
            )
            .unwrap();
    }
    let expected = a.state.clone();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let read = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    let a = &read.attempts[&s.id];
    assert_eq!(a.state, expected);
    assert!(read.favorites.contains(&s.id));
    assert!(!progress::current_solved(a, &s));
    binding::context(a, &s).unwrap();
    let State::MailboxAdministration { mut simulation } = a.state.clone() else {
        panic!()
    };
    let example = simulation.practice.clone();
    simulation.action(d, &Action::Reset, false).unwrap();
    assert_eq!(simulation.practice, example);
    assert_eq!(simulation.assignment, Session::new(d).unwrap().assignment);
}
#[test]
fn administrative_mutation_drops_paused_protocol_connection_and_does_not_invent_pop_credit() {
    let catalog = pack::current().unwrap();
    let s = catalog
        .iter()
        .find(|s| s.id == "lpic2-imap-mailbox-recovery")
        .unwrap();
    let d = binding::source(s).unwrap();
    let mut scope = dlms_learning_lab::mailbox_workspace::Scope::new(&d.assignment).unwrap();
    scope
        .apply(
            &d.assignment,
            &Action::Client {
                id: "imap-tls".into(),
            },
        )
        .unwrap();
    scope.apply(&d.assignment, &Action::Connect).unwrap();
    assert!(scope.connection.is_some());
    scope
        .apply(
            &d.assignment,
            &Action::Admin {
                line: "doveadm expunge -u lee mailbox INBOX uid 107".into(),
            },
        )
        .unwrap();
    assert!(scope.connection.is_none());
    assert!(
        !scope.runtime.store["lee"]["INBOX"]
            .messages
            .iter()
            .any(|m| m.uid == 107)
    );
    assert!(scope.runtime.pop_committed.is_empty());
    assert!(!scope.checked);
    scope.bound(&d.assignment).unwrap();
}
