use dlms_learning_lab::{
    course, engine, mailbox_source as binding,
    mailbox_workspace::{Action, Definition, Session},
    pack,
    progress::{self, Attempt, Progress},
};
const IDS: [&str; 3] = [
    "lpic2-dovecot-secure-access",
    "lpic2-imap-mailbox-recovery",
    "lpic2-pop-transaction-recovery",
];
fn sources() -> Vec<pack::Scenario> {
    pack::current()
        .unwrap()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
}
fn perform(s: &mut Session, d: &Definition, a: Action, practice: bool) {
    s.action(d, &a, practice).unwrap();
}
fn command(s: &mut Session, d: &Definition, line: &str, practice: bool) {
    perform(s, d, Action::Command { line: line.into() }, practice);
}
fn configuration(source: &pack::Scenario) -> String {
    source
        .solution
        .iter()
        .filter_map(|line| line.strip_prefix(binding::PREFIX))
        .filter_map(|line| serde_json::from_str::<Action>(line).ok())
        .find_map(|a| {
            if let Action::Edit { text } = a {
                Some(text)
            } else {
                None
            }
        })
        .unwrap()
}
fn taught(source: &pack::Scenario) -> engine::State {
    let mut state = source.initial.clone();
    for line in &source.solution {
        engine::execute_for_goal(&mut state, &source.goal, line).unwrap();
    }
    assert!(
        engine::solved(&state, &source.goal),
        "{} {:?}",
        source.id,
        engine::checks(&state, &source.goal)
    );
    state
}

#[test]
fn exact478_catalog190_units_and_retained_factory_precede_three_distinct_tasks() {
    let before = pack::accepted478().unwrap();
    let now = pack::accepted481().unwrap();
    assert_eq!((before.len(), now.len()), (478, 481));
    assert_eq!(
        serde_json::to_value(&now[..478]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        now[478..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let old = course::accepted478(&before).unwrap();
    let current = course::accepted481(&now).unwrap();
    assert_eq!((old.len(), current.len()), (190, 191));
    assert_eq!(
        serde_json::to_value(
            current
                .iter()
                .filter(|u| u.id != "linux-mailbox-access")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let unit = current
        .iter()
        .find(|u| u.id == "linux-mailbox-access")
        .unwrap();
    assert_eq!(unit.labs, IDS);
    assert_eq!(
        unit.prerequisites,
        ["linux-smtp-routing", "linux-host-networking"]
    );
    for (track, n) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 134),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
    }
    let a = pack::accepted475().unwrap();
    let b = pack::accepted477().unwrap();
    assert_eq!(
        serde_json::to_value(&b[..475]).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
}
#[test]
fn every_taught_method_and_different_worked_task_reaches_the_required_state() {
    let worked: std::collections::BTreeMap<String, Vec<Action>> =
        serde_json::from_str(include_str!("fixtures/mailbox-access-worked-v1.json")).unwrap();
    for source in sources() {
        let d = binding::source(&source).unwrap();
        let mut s = Session::new(d).unwrap();
        let initial = s.assignment.clone();
        for action in &worked[&source.id] {
            s.action(d, action, true).unwrap();
        }
        assert!(
            s.practice.checks(&d.example).iter().all(|c| c.passed),
            "{} {:?}",
            source.id,
            s.practice.checks(&d.example)
        );
        assert_eq!(s.assignment, initial);
        assert!(d.checks(&s).iter().any(|c| !c.passed));
        taught(&source);
    }
}
#[test]
fn equivalent_server_config_and_imap_operations_pass_current_behavior() {
    for source in sources() {
        let mut state = taught(&source);
        let engine::State::MailboxAccess { simulation } = &mut state else {
            panic!()
        };
        let d = binding::source(&source).unwrap();
        if d.assignment.editable_config {
            let text = simulation
                .assignment
                .draft
                .replace("ssl = required", "ssl = yes")
                .replace("/srv/mail/%{user}/Maildir", "~/Maildir");
            for a in [
                Action::Edit { text },
                Action::Save,
                Action::Restart,
                Action::Check,
            ] {
                perform(simulation, d, a, false);
            }
            assert!(
                engine::solved(&state, &source.goal),
                "{} {:?}",
                source.id,
                engine::checks(&state, &source.goal)
            );
        } else {
            let mut s = Session::new(d).unwrap();
            perform(
                &mut s,
                d,
                Action::Client {
                    id: "imap-tls".into(),
                },
                false,
            );
            perform(&mut s, d, Action::Connect, false);
            for line in [
                "a1 LOGIN lee sample-lee",
                "a2 SELECT INBOX",
                "a3 UID FETCH 101 BODY[]",
                r"a4 UID STORE 107 FLAGS (\Deleted)",
                "a5 CLOSE",
            ] {
                command(&mut s, d, line, false);
            }
            perform(&mut s, d, Action::Check, false);
            assert!(d.checks(&s).iter().all(|c| c.passed), "{:?}", d.checks(&s));
        }
    }
}
#[test]
fn wrong_or_partial_server_repairs_never_pass_and_errors_identify_the_failed_layer() {
    let source = &sources()[0];
    let d = binding::source(source).unwrap();
    let mut s = Session::new(d).unwrap();
    perform(&mut s, d, Action::Check, false);
    assert!(d.checks(&s).iter().any(|c| !c.passed));
    assert!(d.checks(&s).iter().any(|c| c.passed));
    let good = configuration(source);
    for text in [
        good.replace("/srv/mail/%{user}/Maildir", "/srv/mail/alex/Maildir"),
        good.replace("ssl = required", "ssl = yes")
            .replace("auth_allow_cleartext = no", "auth_allow_cleartext = yes"),
        good.replace("protocols = imap pop3", "protocols = pop3"),
        good.replace("/site.pem", "/old.pem")
            .replace("/site.key", "/old.key"),
    ] {
        let mut s = Session::new(d).unwrap();
        for a in [
            Action::Edit { text },
            Action::Save,
            Action::Restart,
            Action::Enable,
            Action::Check,
        ] {
            perform(&mut s, d, a, false);
        }
        assert!(
            d.checks(&s).iter().any(|c| !c.passed),
            "Wrong server passed"
        );
        assert!(
            d.checks(&s)
                .iter()
                .any(|c| c.detail.contains("Observed") || c.detail.contains("unprotected"))
        );
    }
}
#[test]
fn independent_protocol_tasks_reject_pending_removal_and_unrelated_message_loss() {
    for source in sources().into_iter().skip(1) {
        let d = binding::source(&source).unwrap();
        let mut s = Session::new(d).unwrap();
        if source.id == IDS[1] {
            perform(
                &mut s,
                d,
                Action::Client {
                    id: "imap-tls".into(),
                },
                false,
            );
            perform(&mut s, d, Action::Connect, false);
            for line in [
                "a1 LOGIN lee sample-lee",
                "a2 SELECT INBOX",
                r"a3 UID STORE 101 +FLAGS \Seen",
                r"a4 UID STORE 107 +FLAGS \Deleted",
            ] {
                command(&mut s, d, line, false);
            }
        } else {
            for a in [
                Action::Edit {
                    text: configuration(&source),
                },
                Action::Save,
                Action::Restart,
                Action::Enable,
                Action::Client {
                    id: "pop-tls".into(),
                },
                Action::Connect,
            ] {
                perform(&mut s, d, a, false);
            }
            for line in ["USER kai", "PASS sample-kai", "DELE 2"] {
                command(&mut s, d, line, false);
            }
        }
        perform(&mut s, d, Action::Check, false);
        assert!(
            d.checks(&s)
                .iter()
                .any(|c| !c.passed && c.label.contains("UID 107"))
        );
        let mut state = taught(&source);
        let engine::State::MailboxAccess { simulation } = &mut state else {
            panic!()
        };
        let user = if source.id == IDS[1] { "lee" } else { "kai" };
        perform(
            simulation,
            d,
            Action::Client {
                id: "imap-tls".into(),
            },
            false,
        );
        perform(simulation, d, Action::Connect, false);
        for line in [
            format!("a1 LOGIN {user} sample-{user}"),
            "a2 SELECT INBOX".into(),
            r"a3 UID STORE 109 +FLAGS \Deleted".into(),
            "a4 EXPUNGE".into(),
        ] {
            command(simulation, d, &line, false);
        }
        perform(simulation, d, Action::Check, false);
        assert!(
            d.checks(simulation)
                .iter()
                .any(|c| !c.passed && c.label == "Preserve other supplied mail")
        );
        assert!(!engine::solved(&state, &source.goal));
    }
}
#[test]
fn copying_the_worked_server_file_is_not_an_independent_assignment_solution() {
    for source in sources() {
        let d = binding::source(&source).unwrap();
        if let Some(text) = &d.worked_config {
            let mut s = Session::new(d).unwrap();
            for a in [
                Action::Edit { text: text.clone() },
                Action::Save,
                Action::Restart,
                Action::Enable,
                Action::Check,
            ] {
                perform(&mut s, d, a, false);
            }
            assert!(
                d.checks(&s).iter().any(|c| !c.passed),
                "{} accepted worked fixture paths",
                source.id
            );
            assert_ne!(d.assignment.facts.accounts, d.example.facts.accounts);
            assert_ne!(configuration(&source), *text);
        }
    }
}
#[test]
fn worked_protocol_identifiers_and_positions_do_not_reveal_independent_operations() {
    let source = &sources()[1];
    let d = binding::source(source).unwrap();
    let assignment: Vec<_> = d.assignment.facts.mailboxes["lee"]["INBOX"]
        .messages
        .iter()
        .map(|m| m.uid)
        .collect();
    let example: Vec<_> = d.example.facts.mailboxes["jules"]["INBOX"]
        .messages
        .iter()
        .map(|m| m.uid)
        .collect();
    assert!(assignment.iter().all(|u| !example.contains(u)));
    let mut state = taught(source);
    let engine::State::MailboxAccess { simulation } = &mut state else {
        panic!()
    };
    perform(simulation, d, Action::Reset, false);
    perform(
        simulation,
        d,
        Action::Client {
            id: "imap-tls".into(),
        },
        false,
    );
    perform(simulation, d, Action::Connect, false);
    for line in [
        "a1 LOGIN lee sample-lee",
        "a2 SELECT INBOX",
        r"a3 UID STORE 201 +FLAGS \Seen",
        r"a4 UID STORE 209 +FLAGS \Deleted",
        "a5 EXPUNGE",
    ] {
        command(simulation, d, line, false);
    }
    perform(simulation, d, Action::Check, false);
    assert!(d.checks(simulation).iter().any(|c| !c.passed));
    let source = &sources()[2];
    let d = binding::source(source).unwrap();
    let target = |task: &dlms_learning_lab::mailbox_workspace::Task, user: &str| {
        task.facts.mailboxes[user]["INBOX"]
            .messages
            .iter()
            .position(|m| {
                m.headers
                    .iter()
                    .any(|(n, v)| n == "Subject" && v == "Expired test probe")
            })
            .unwrap()
    };
    assert_ne!(target(&d.assignment, "kai"), target(&d.example, "ada"));
}
#[test]
fn imap_removal_does_not_falsely_demonstrate_a_pop_transaction_and_old_credit_is_not_invented() {
    let source = &sources()[2];
    let d = binding::source(source).unwrap();
    let mut s = Session::new(d).unwrap();
    for a in [
        Action::Edit {
            text: configuration(source),
        },
        Action::Save,
        Action::Restart,
        Action::Enable,
        Action::Client {
            id: "imap-tls".into(),
        },
        Action::Connect,
    ] {
        perform(&mut s, d, a, false);
    }
    for line in [
        "a1 LOGIN kai sample-kai",
        "a2 SELECT INBOX",
        r"a3 UID STORE 107 +FLAGS \Deleted",
        "a4 EXPUNGE",
    ] {
        command(&mut s, d, line, false);
    }
    perform(&mut s, d, Action::Check, false);
    let checks = d.checks(&s);
    assert!(
        checks
            .iter()
            .any(|c| c.label.contains("UID 107") && c.passed)
    );
    assert!(
        checks
            .iter()
            .any(|c| c.label.contains("committed POP cleanup") && !c.passed)
    );
    assert!(
        checks
            .iter()
            .any(|c| c.label == "Preserve other supplied mail" && c.passed)
    );
    let state = taught(source);
    let engine::State::MailboxAccess { simulation } = &state else {
        panic!()
    };
    let mut old = serde_json::to_value(simulation).unwrap();
    old["assignment"]["runtime"]
        .as_object_mut()
        .unwrap()
        .remove("pop_committed");
    let old: Session = serde_json::from_value(old).unwrap();
    d.bound(&old).unwrap();
    assert!(
        d.checks(&old)
            .iter()
            .any(|c| c.label.contains("committed POP cleanup") && !c.passed)
    );
}
#[test]
fn normal_real_file_progress_retains_distinct_drafts_and_invalidates_changed_success() {
    let catalog = pack::current().unwrap();
    let source = catalog.iter().find(|s| s.id == IDS[2]).unwrap();
    let d = binding::source(source).unwrap();
    let mut state = taught(source);
    let engine::State::MailboxAccess { simulation } = &mut state else {
        panic!()
    };
    perform(
        simulation,
        d,
        Action::Edit {
            text: "unfinished assignment file".into(),
        },
        false,
    );
    perform(
        simulation,
        d,
        Action::Edit {
            text: "separate worked draft".into(),
        },
        true,
    );
    perform(
        simulation,
        d,
        Action::Input {
            text: "USER ka".into(),
        },
        false,
    );
    assert!(!engine::solved(&state, &source.goal));
    let expected = serde_json::to_value(&state).unwrap();
    let mut attempt = Attempt::new(source, 1);
    attempt.state = state;
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.favorites.insert(source.id.clone());
    p.attempts.insert(source.id.clone(), attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let read = progress::load(&path, &catalog).unwrap();
    let a = &read.attempts[&source.id];
    assert_eq!(serde_json::to_value(&a.state).unwrap(), expected);
    binding::context(a, source).unwrap();
    assert!(!progress::current_solved(a, source));
    assert!(read.favorites.contains(&source.id));
    let engine::State::MailboxAccess { simulation } = &a.state else {
        panic!()
    };
    let mut restored = (**simulation).clone();
    let practice = restored.practice.clone();
    perform(&mut restored, d, Action::Reset, false);
    assert_eq!(restored.practice, practice);
    assert!(d.checks(&restored).iter().any(|c| !c.passed));
}
