//! Real authored capstone behavior; registration/visual acceptance are separate gates.
use dlms_learning_lab::{
    engine::{self, State},
    mailbox_source as binding,
    mailbox_workspace::{Action, Definition, Requirement, Session},
    pack,
    progress::{self, Attempt, Progress},
};
fn source() -> pack::Scenario {
    serde_json::from_value(
        serde_json::from_str::<serde_json::Value>(include_str!(
            "../scenarios/mailbox-administration-lessons-v1.json"
        ))
        .unwrap()["scenarios"][0]
            .clone(),
    )
    .unwrap()
}
fn session(state: &mut State) -> &mut Session {
    let State::MailboxAdministration { simulation } = state else {
        panic!()
    };
    simulation
}
fn config(s: &pack::Scenario) -> String {
    s.solution
        .iter()
        .find_map(|l| {
            match serde_json::from_str::<Action>(l.strip_prefix(binding::PREFIX)?).ok()? {
                Action::Edit { text } => Some(text),
                _ => None,
            }
        })
        .unwrap()
}
fn action(st: &mut State, d: &Definition, a: Action, practice: bool) {
    session(st).action(d, &a, practice).unwrap();
}
fn query(st: &mut State, d: &Definition, line: &str, practice: bool) {
    action(st, d, Action::Admin { line: line.into() }, practice);
}
fn prepared(st: &mut State, s: &pack::Scenario, alternate: bool) {
    let d = binding::source(s).unwrap();
    let mut cfg = config(s);
    if alternate {
        cfg = cfg.replace("/srv/mail/%{user}/Maildir", "~/Maildir");
    }
    for a in [
        Action::Edit { text: cfg },
        Action::Save,
        Action::Restart,
        Action::Enable,
    ] {
        action(st, d, a, false);
    }
}
#[test]
fn taught_method_and_separate_worked_values_reach_all_current_outcomes() {
    let s = source();
    let d = binding::source(&s).unwrap();
    let mut st = s.initial.clone();
    assert!(!engine::solved(&st, &s.goal));
    for l in &s.solution {
        engine::execute_for_goal(&mut st, &s.goal, l).unwrap();
    }
    assert!(
        engine::solved(&st, &s.goal),
        "{:?}",
        engine::checks(&st, &s.goal)
    );
    let mut practice = s.initial.clone();
    let actions: Vec<Action> = serde_json::from_str(include_str!(
        "fixtures/mailbox-administration-worked-v1.json"
    ))
    .unwrap();
    for a in actions {
        action(&mut practice, d, a, true);
    }
    assert!(
        session(&mut practice)
            .practice
            .checks(&d.example)
            .iter()
            .all(|r| r.passed)
    );
    assert!(!engine::solved(&practice, &s.goal));
    assert_eq!(
        session(&mut practice).assignment,
        Session::new(d).unwrap().assignment
    );
    assert_ne!(d.assignment.facts.accounts, d.example.facts.accounts);
    assert_ne!(config(&s), *d.worked_config.as_ref().unwrap());
}
#[test]
fn equivalent_home_path_and_individual_uid_queries_pass_without_exact_command_matching() {
    let s = source();
    let d = binding::source(&s).unwrap();
    for alternate in [false, true] {
        let mut st = s.initial.clone();
        prepared(&mut st, &s, alternate);
        for r in &d.assignment.requirements {
            if let Requirement::Message {
                user,
                mailbox,
                uid,
                present: false,
                ..
            } = r
            {
                query(
                    &mut st,
                    d,
                    &format!("doveadm expunge -u {user} uid {uid} mailbox '{mailbox}'"),
                    false,
                );
            }
        }
        action(&mut st, d, Action::Check, false);
        assert!(
            engine::solved(&st, &s.goal),
            "{:?}",
            engine::checks(&st, &s.goal)
        );
    }
}
#[test]
fn missing_predicates_wrong_account_and_copied_worked_config_do_not_pass() {
    let s = source();
    let d = binding::source(&s).unwrap();
    for (user, predicate) in [
        ("rowan", "subject '[EXPIRED]'"),
        ("tess", "seen subject '[EXPIRED]'"),
        ("rowan", "uid 1:4294967295"),
    ] {
        let mut st = s.initial.clone();
        prepared(&mut st, &s, false);
        query(
            &mut st,
            d,
            &format!("doveadm expunge -u {user} mailbox INBOX {predicate}"),
            false,
        );
        action(&mut st, d, Action::Check, false);
        assert!(!engine::solved(&st, &s.goal));
        let checks = engine::checks(&st, &s.goal);
        assert!(
            checks
                .iter()
                .any(|r| !r.0 && r.1.contains("Preserve other supplied mail"))
                || checks.iter().any(|r| !r.0 && r.1.contains("317")),
            "{checks:?}"
        );
    }
    let mut st = s.initial.clone();
    for a in [
        Action::Edit {
            text: d.worked_config.clone().unwrap(),
        },
        Action::Save,
        Action::Restart,
        Action::Enable,
        Action::Check,
    ] {
        action(&mut st, d, a, false);
    }
    assert!(!engine::solved(&st, &s.goal));
}
#[test]
fn partial_progress_feedback_reset_and_real_file_resume_are_honest() {
    let s = source();
    let d = binding::source(&s).unwrap();
    let mut st = s.initial.clone();
    prepared(&mut st, &s, false);
    query(
        &mut st,
        d,
        "doveadm expunge -u rowan mailbox INBOX uid 311",
        false,
    );
    action(&mut st, d, Action::Check, false);
    let rows = engine::checks(&st, &s.goal);
    assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
    assert!(!engine::solved(&st, &s.goal));
    action(
        &mut st,
        d,
        Action::Input {
            text: "doveadm search -u rowan mailbox Archive ".into(),
        },
        false,
    );
    action(
        &mut st,
        d,
        Action::Input {
            text: "doveconf -h mail_".into(),
        },
        true,
    );
    let expected = st.clone();
    let mut a = Attempt::new(&s, 1);
    a.state = st;
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let read = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    let mut st = read.attempts[&s.id].state.clone();
    assert_eq!(st, expected);
    assert!(!progress::current_solved(&read.attempts[&s.id], &s));
    binding::context(&read.attempts[&s.id], &s).unwrap();
    let example = session(&mut st).practice.clone();
    action(&mut st, d, Action::Reset, false);
    assert_eq!(session(&mut st).practice, example);
    assert_eq!(
        session(&mut st).assignment,
        Session::new(d).unwrap().assignment
    );
}
#[test]
fn edits_and_administrative_mail_loss_cannot_leave_stale_success() {
    let s = source();
    let d = binding::source(&s).unwrap();
    let mut st = s.initial.clone();
    for l in &s.solution {
        engine::execute_for_goal(&mut st, &s.goal, l).unwrap();
    }
    assert!(engine::solved(&st, &s.goal));
    action(
        &mut st,
        d,
        Action::Edit {
            text: "unknown_setting = yes".into(),
        },
        false,
    );
    assert!(!engine::solved(&st, &s.goal));
    action(&mut st, d, Action::Check, false);
    assert!(!engine::solved(&st, &s.goal));
    action(&mut st, d, Action::Edit { text: config(&s) }, false);
    action(&mut st, d, Action::Check, false);
    assert!(engine::solved(&st, &s.goal));
    query(
        &mut st,
        d,
        "doveadm expunge -u tess mailbox INBOX all",
        false,
    );
    action(&mut st, d, Action::Check, false);
    assert!(!engine::solved(&st, &s.goal));
}
#[test]
fn guide_commands_are_runnable_and_the_counterexample_reveals_protected_unread_mail() {
    let s = source();
    let d = binding::source(&s).unwrap();
    let mut st = s.initial.clone();
    for a in [
        Action::Edit {
            text: d.worked_config.clone().unwrap(),
        },
        Action::Save,
        Action::Restart,
        Action::Enable,
    ] {
        action(&mut st, d, a, true);
    }
    for c in s.guide.iter().flat_map(|g| &g.commands) {
        query(&mut st, d, &c.command, true);
        assert!(!session(&mut st).practice.output.contains("[Unsupported]"));
        assert!(!session(&mut st).practice.output.contains("[Syntax]"));
    }
    assert!(session(&mut st).practice.output.contains(" 44\n"));
    action(&mut st, d, Action::Check, true);
    assert!(
        session(&mut st)
            .practice
            .checks(&d.example)
            .iter()
            .all(|r| r.passed)
    );
    assert!(!engine::solved(&st, &s.goal));
}

#[test]
fn main_catalog_registers_one_real_capstone_and_retains_every_existing_scenario_and_unit() {
    let before = pack::accepted481().unwrap();
    let now = pack::current().unwrap();
    assert_eq!((before.len(), now.len()), (481, 487));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&now[..481]).unwrap()
    );
    assert_eq!(now[481].id, "lpic2-mailbox-administration-recovery");
    let old_units = dlms_learning_lab::course::accepted481(&before).unwrap();
    let units = dlms_learning_lab::course::current(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (191, 194));
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-mailbox-administration"
                    && u.id != "linux-router-policies"
                    && u.id != "linux-connection-nat")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let u = units
        .iter()
        .find(|u| u.id == "linux-mailbox-administration")
        .unwrap();
    assert_eq!(u.prerequisites, ["linux-mailbox-access"]);
    assert_eq!(u.labs, [now[481].id.clone()]);
}
