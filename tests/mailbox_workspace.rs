use dlms_learning_lab::{
    mailbox_access::Flag,
    mailbox_config::Protocol,
    mailbox_workspace::{Action, Definition, Requirement, Scope, Session, Task},
};
#[path = "support/mailbox_fixtures.rs"]
mod fixture;
use fixture::*;

fn task() -> Task {
    let mut pop = client(true);
    pop.protocol = Protocol::Pop3;
    pop.port = 995;
    Task {
        editable_config: true,
        facts: facts(),
        saved: CONFIG.into(),
        active: Some(CONFIG.into()),
        enabled: false,
        clients: [
            ("imap-tls".into(), client(true)),
            ("imap-plain".into(), client(false)),
            ("pop-tls".into(), pop),
        ]
        .into(),
        requirements: vec![
            Requirement::Enabled,
            Requirement::Access {
                profile: "imap-tls".into(),
                user: "alex".into(),
                upgrade: false,
                allow: true,
            },
            Requirement::Access {
                profile: "imap-tls".into(),
                user: "sam".into(),
                upgrade: false,
                allow: true,
            },
            Requirement::Access {
                profile: "imap-plain".into(),
                user: "alex".into(),
                upgrade: false,
                allow: false,
            },
        ],
    }
}
fn definition() -> Definition {
    let assignment = task();
    let mut example = task();
    example.facts.day = 101;
    Definition {
        schema: 1,
        source_id: "mail-access-test".into(),
        assignment,
        example,
        worked_config: None,
    }
}
fn send(scope: &mut Scope, task: &Task, line: &str) {
    scope
        .apply(task, &Action::Command { line: line.into() })
        .unwrap();
}
fn all(scope: &Scope, task: &Task) -> bool {
    scope.checks(task).iter().all(|c| c.passed)
}

#[test]
fn source_validation_preserves_meaningful_separate_work_and_known_cases() {
    let d = definition();
    d.validate().unwrap();
    let mut bad = d.clone();
    bad.example.facts = bad.assignment.facts.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.enabled = true;
    assert!(bad.validate().unwrap_err().contains("meaningful"));
    let mut bad = d.clone();
    bad.assignment.requirements.push(Requirement::Access {
        profile: "not-supplied".into(),
        user: "alex".into(),
        upgrade: false,
        allow: true,
    });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.requirements.push(Requirement::Access {
        profile: "imap-tls".into(),
        user: "alex".into(),
        upgrade: false,
        allow: false,
    });
    assert!(bad.validate().is_err());
}
#[test]
fn practice_cannot_award_assignment_credit_or_overwrite_its_draft() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    s.action(&d, &Action::Enable, true).unwrap();
    s.action(&d, &Action::Check, true).unwrap();
    assert!(all(&s.practice, &d.example));
    assert_eq!(s.assignment, assignment);
    assert!(d.checks(&s).iter().any(|c| !c.passed));
    s.action(
        &d,
        &Action::Edit {
            text: "unfinished configuration".into(),
        },
        false,
    )
    .unwrap();
    let draft = s.assignment.draft.clone();
    s.action(&d, &Action::Reset, true).unwrap();
    assert_eq!(s.assignment.draft, draft);
}
#[test]
fn behavior_checks_accept_equivalent_tls_policy_and_path_but_reject_hardcoded_user() {
    let task = task();
    let mut scope = Scope::new(&task).unwrap();
    scope.apply(&task, &Action::Enable).unwrap();
    assert!(all(&scope, &task));
    let before = scope.runtime.store.clone();
    for text in [
        CONFIG.replace("ssl = required", "ssl = yes"),
        CONFIG.replace("/srv/mail/%{user}/Maildir", "~/Maildir"),
    ] {
        scope.apply(&task, &Action::Edit { text }).unwrap();
        scope.apply(&task, &Action::Save).unwrap();
        scope.apply(&task, &Action::Restart).unwrap();
        assert!(all(&scope, &task));
    }
    assert_eq!(scope.runtime.store, before);
    scope
        .apply(
            &task,
            &Action::Edit {
                text: CONFIG.replace("/srv/mail/%{user}/Maildir", "/srv/mail/alex/Maildir"),
            },
        )
        .unwrap();
    scope.apply(&task, &Action::Save).unwrap();
    scope.apply(&task, &Action::Restart).unwrap();
    let checks = scope.checks(&task);
    assert!(checks.iter().any(|c| !c.passed
        && c.label.contains("sam")
        && c.detail.contains("resolved virtual Maildir")));
}
#[test]
fn edits_clear_success_and_save_restart_failures_remain_useful_and_retryable() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.action(&d, &Action::Enable, false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.checks(&s).iter().all(|c| c.passed));
    s.action(
        &d,
        &Action::Edit {
            text: "bad_setting = yes".into(),
        },
        false,
    )
    .unwrap();
    assert!(!s.assignment.checked);
    assert!(d.checks(&s).iter().any(|c| !c.passed));
    s.action(&d, &Action::Save, false).unwrap();
    assert!(s.assignment.runtime.running);
    s.action(&d, &Action::Restart, false).unwrap();
    assert!(!s.assignment.runtime.running);
    assert!(s.assignment.output.contains("Restart failed"));
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.checks(&s).iter().any(|c| !c.passed));
    s.action(
        &d,
        &Action::Edit {
            text: CONFIG.into(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Save, false).unwrap();
    s.action(&d, &Action::Restart, false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.checks(&s).iter().all(|c| c.passed));
}
#[test]
fn denied_access_cannot_pass_because_a_listener_is_disabled_or_a_certificate_is_broken() {
    let task = task();
    let mut scope = Scope::new(&task).unwrap();
    scope.apply(&task, &Action::Enable).unwrap();
    let disabled =
        format!("{CONFIG}service imap-login {{\n inet_listener imap {{\n port = 0\n }}\n}}\n");
    scope
        .apply(&task, &Action::Edit { text: disabled })
        .unwrap();
    scope.apply(&task, &Action::Save).unwrap();
    scope.apply(&task, &Action::Restart).unwrap();
    assert!(scope.checks(&task).iter().any(|c| !c.passed
        && c.label.contains("reject unprotected")
        && c.detail.contains("Connection")));
    let mut task = task.clone();
    task.clients
        .get_mut("imap-tls")
        .unwrap()
        .trusted_issuers
        .clear();
    let mut scope = Scope::new(&task).unwrap();
    scope.apply(&task, &Action::Enable).unwrap();
    assert!(
        scope
            .checks(&task)
            .iter()
            .any(|c| !c.passed && c.detail.contains("does not trust"))
    );
}
#[test]
fn pop_pending_delete_resumes_without_committing_and_feedback_explains_the_difference() {
    let mut task = task();
    task.requirements.push(Requirement::Message {
        user: "alex".into(),
        mailbox: "INBOX".into(),
        uid: 101,
        present: false,
        seen: None,
        deleted: None,
    });
    let mut scope = Scope::new(&task).unwrap();
    scope.apply(&task, &Action::Enable).unwrap();
    scope
        .apply(
            &task,
            &Action::Client {
                id: "pop-tls".into(),
            },
        )
        .unwrap();
    scope.apply(&task, &Action::Connect).unwrap();
    for line in ["USER alex", "PASS sample-alex", "DELE 1"] {
        send(&mut scope, &task, line);
    }
    assert!(
        scope
            .checks(&task)
            .iter()
            .any(|c| !c.passed && c.detail.contains("DELE only marked"))
    );
    let bytes = serde_json::to_vec(&scope).unwrap();
    let mut restored: Scope = serde_json::from_slice(&bytes).unwrap();
    restored.bound(&task).unwrap();
    assert_eq!(scope, restored);
    send(&mut restored, &task, "QUIT");
    assert!(all(&restored, &task));
    scope.apply(&task, &Action::Disconnect).unwrap();
    assert!(!all(&scope, &task));
    assert_eq!(scope.runtime.store["alex"]["INBOX"].messages.len(), 2);
}
#[test]
fn imap_state_targets_accept_supported_alternatives_and_preserve_unrelated_mail() {
    let mut task = task();
    task.requirements.push(Requirement::Message {
        user: "alex".into(),
        mailbox: "INBOX".into(),
        uid: 101,
        present: true,
        seen: Some(true),
        deleted: Some(false),
    });
    for solution in [r"a3 STORE 1 +FLAGS \Seen", "a3 FETCH 1 BODY[]"] {
        let mut scope = Scope::new(&task).unwrap();
        scope.apply(&task, &Action::Enable).unwrap();
        scope
            .apply(
                &task,
                &Action::Client {
                    id: "imap-tls".into(),
                },
            )
            .unwrap();
        scope.apply(&task, &Action::Connect).unwrap();
        for line in ["a1 LOGIN alex sample-alex", "a2 SELECT INBOX", solution] {
            send(&mut scope, &task, line);
        }
        assert!(all(&scope, &task), "{solution}");
        let before = scope.runtime.store.clone();
        scope.checks(&task);
        assert_eq!(scope.runtime.store, before);
        send(&mut scope, &task, r"a4 STORE 2 +FLAGS \Deleted");
        send(&mut scope, &task, "a5 EXPUNGE");
        assert!(
            scope
                .checks(&task)
                .iter()
                .any(|c| !c.passed && c.label == "Preserve other supplied mail")
        );
    }
}
#[test]
fn restore_rejects_invented_content_namespaces_and_uid_metadata() {
    let task = task();
    let scope = Scope::new(&task).unwrap();
    let mut changed = scope.clone();
    changed
        .runtime
        .store
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap()
        .messages[0]
        .body = "fabricated".into();
    assert!(changed.bound(&task).is_err());
    let mut changed = scope.clone();
    changed
        .runtime
        .store
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap()
        .uid_next = 200;
    assert!(changed.bound(&task).is_err());
    let mut changed = scope.clone();
    changed
        .runtime
        .store
        .insert("intruder".into(), Default::default());
    assert!(changed.bound(&task).is_err());
    let mut changed = scope.clone();
    changed.runtime.active = None;
    assert!(changed.bound(&task).is_err());
    let mut changed = scope.clone();
    changed
        .runtime
        .store
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap()
        .messages[0]
        .flags
        .insert(Flag::Seen);
    changed.bound(&task).unwrap();
}
#[test]
fn partial_practice_and_assignment_roundtrip_and_selected_reset_are_independent() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "practice draft".into(),
        },
        true,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "assignment draft".into(),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Input {
            text: "a1 LOG".into(),
        },
        false,
    )
    .unwrap();
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&bytes).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(s, restored);
    let practice = restored.practice.clone();
    restored.action(&d, &Action::Reset, false).unwrap();
    assert_eq!(restored.practice, practice);
    assert_eq!(restored.assignment, Scope::new(&d.assignment).unwrap());
}
#[test]
fn protocol_only_lessons_keep_server_settings_readonly_without_limiting_message_work() {
    let mut task = task();
    task.editable_config = false;
    task.enabled = true;
    let mut scope = Scope::new(&task).unwrap();
    let before = scope.clone();
    assert!(
        scope
            .apply(
                &task,
                &Action::Edit {
                    text: CONFIG.replace("ssl = required", "ssl = no")
                }
            )
            .is_err()
    );
    assert_eq!(scope, before);
    let mut forged = scope.clone();
    forged.runtime.saved = CONFIG.replace("ssl = required", "ssl = yes");
    assert!(forged.bound(&task).is_err());
    scope
        .apply(
            &task,
            &Action::Client {
                id: "imap-tls".into(),
            },
        )
        .unwrap();
    scope.apply(&task, &Action::Connect).unwrap();
    send(&mut scope, &task, "a1 LOGIN alex sample-alex");
    send(&mut scope, &task, "a2 SELECT INBOX");
    send(&mut scope, &task, r"a3 STORE 1 +FLAGS \Seen");
    scope.bound(&task).unwrap();
}
#[test]
fn worked_configuration_is_verified_against_typed_server_cases_not_case_labels() {
    let mut d = definition();
    d.worked_config = Some(CONFIG.into());
    d.validate().unwrap();
    d.worked_config = Some(CONFIG.replace("/srv/mail/%{user}/Maildir", "/srv/mail/alex/Maildir"));
    assert!(
        d.validate()
            .unwrap_err()
            .contains("worked example configuration")
    );
    d.worked_config = Some(CONFIG.into());
    d.example.editable_config = false;
    assert!(d.validate().is_err());
}
#[test]
fn malformed_actions_fail_atomically_and_no_path_or_host_command_is_executed() {
    let task = task();
    let mut scope = Scope::new(&task).unwrap();
    let original = scope.clone();
    assert!(
        scope
            .apply(
                &task,
                &Action::Client {
                    id: "../../etc/passwd".into()
                }
            )
            .is_err()
    );
    assert_eq!(scope, original);
    assert!(
        scope
            .apply(
                &task,
                &Action::Edit {
                    text: "x".repeat(8193)
                }
            )
            .is_err()
    );
    assert_eq!(scope, original);
    scope
        .apply(
            &task,
            &Action::Client {
                id: "imap-tls".into(),
            },
        )
        .unwrap();
    scope.apply(&task, &Action::Connect).unwrap();
    let before = scope.runtime.store.clone();
    send(&mut scope, &task, "a1 bash -c id");
    assert!(scope.output.contains("outside the documented IMAP subset"));
    assert_eq!(scope.runtime.store, before);
}
