use dlms_learning_lab::{
    automation_workshop::{
        authenticated_transfer::{Client, Policy, Task as DnsTask, TransferReply},
        socket_retirement::{Operation, Request, Target, Task as SocketTask},
        transfer_editor::Edit,
        workspace::{Action, Task, Work},
    },
    dns::Transfer,
    dns_access::config::parse_transfer_fragment,
};
use std::collections::BTreeMap;

fn sockets() -> BTreeMap<String, SocketTask> {
    serde_json::from_str(include_str!("fixtures/socket_retirement_tasks_v1.json")).unwrap()
}

fn transfers() -> BTreeMap<String, DnsTask> {
    serde_json::from_str(include_str!(
        "fixtures/authenticated_transfer_tasks_v1.json"
    ))
    .unwrap()
}

#[test]
fn stopping_or_disabling_in_isolation_cannot_entirely_retire_a_listener() {
    for task in sockets().into_values() {
        task.validate().unwrap();
        let mut state = task.initial.clone();
        task.apply(&mut state, Target::ObsoleteService, Operation::Stop)
            .unwrap();
        assert_eq!(
            task.request(&mut state, Target::ObsoleteService).unwrap(),
            Request::SocketActivated
        );
        task.apply(&mut state, Target::ObsoleteSocket, Operation::Disable)
            .unwrap();
        assert!(state.activation.listening);
        task.apply(&mut state, Target::ObsoleteService, Operation::Disable)
            .unwrap();
        let midway = task.checks(&state).unwrap();
        assert_eq!(midway, [true, false, true, false, true, true]);
        task.apply(&mut state, Target::ObsoleteSocket, Operation::Stop)
            .unwrap();
        task.apply(&mut state, Target::ObsoleteService, Operation::Stop)
            .unwrap();
        assert_eq!(task.checks(&state).unwrap(), [true; 6]);
        task.apply(&mut state, Target::RequiredService, Operation::Disable)
            .unwrap();
        assert!(!task.checks(&state).unwrap()[5]);
    }
}

#[test]
fn supported_systemctl_form_verifies_both_policies_without_any_extra_commands() {
    for task in sockets().into_values() {
        let mut state = task.initial.clone();
        for name in [&task.obsolete_service, &task.activating_socket] {
            assert_eq!(
                task.terminal(&mut state, &format!("systemctl disable --now {name}"))
                    .unwrap()
                    .status,
                0
            );
            assert_eq!(
                task.terminal(&mut state, &format!("systemctl is-enabled {name}"))
                    .unwrap()
                    .status,
                1
            );
            assert_eq!(
                task.terminal(&mut state, &format!("systemctl is-active {name}"))
                    .unwrap()
                    .status,
                3
            );
        }
        assert_eq!(task.checks(&state).unwrap(), [true; 6]);
        for rejected in [
            "systemctl disable --now /etc/ssh/sshd_config",
            "systemctl status nonexistent.service",
            "systemctl enable *",
            "systemctl stop ../user.socket",
            "systemctl try-restart foo.service",
            "systemctl status legacy-import.service # continue",
        ] {
            assert!(task.terminal(&mut state, rejected).is_err());
        }
        assert_eq!(task.checks(&state).unwrap(), [true; 6]);
    }
}

#[test]
fn address_admission_does_not_authenticate_and_failed_proofs_do_not_update_a_replica() {
    for task in transfers().into_values() {
        task.validate().unwrap();
        let mut state = task.start();
        assert!(matches!(
            task.transfer(&mut state, Client::UnsignedAtSecondaryAddress, false)
                .unwrap(),
            TransferReply::UnauthorizedRead(_)
        ));
        assert_eq!(
            task.transfer(&mut state, Client::OldProofAtSecondaryAddress, false)
                .unwrap(),
            TransferReply::AuthenticationRejected
        );
        assert_eq!(
            task.transfer(&mut state, Client::ConfiguredSecondary, false)
                .unwrap(),
            TransferReply::AuthenticationRejected
        );
        assert_eq!(state.copies.secondary, task.served.secondary);
        assert!(!task.checks(&state).unwrap()[2]);
        assert!(!task.checks(&state).unwrap()[3]);
    }
}

#[test]
fn protected_zone_is_checked_with_independent_current_proof() {
    for task in transfers().into_values() {
        let mut state = task.start();
        assert!(!task.checks(&state).unwrap()[3]);
        assert!(task.checks(&state).unwrap()[4]);
        let original = state.clone();
        assert_eq!(
            task.transfer(&mut state, Client::ConfiguredSecondary, true)
                .unwrap(),
            TransferReply::AuthenticationRejected
        );
        assert_eq!(state, original);
        state.draft.unrelated = Policy::SourceAddress;
        task.apply(&mut state).unwrap();
        assert!(!task.checks(&state).unwrap()[4]);
    }
}

#[test]
fn either_approved_key_works_and_no_unrelated_policy_can_ride_along() {
    for task in transfers().into_values() {
        for reference in &task.approved {
            let mut state = task.start();
            state.draft.primary = Policy::VerifiedKey {
                reference: reference.clone(),
            };
            state.draft.secondary = Some(reference.clone());
            assert!(!task.checks(&state).unwrap()[0]);
            task.apply(&mut state).unwrap();
            assert!(!task.checks(&state).unwrap()[2]);
            assert_eq!(
                task.transfer(&mut state, Client::ConfiguredSecondary, false)
                    .unwrap(),
                TransferReply::Secondary(Transfer::Updated)
            );
            assert_eq!(task.checks(&state).unwrap(), [true; 6]);
            state.draft.unrelated = Policy::VerifiedKey {
                reference: reference.clone(),
            };
            task.apply(&mut state).unwrap();
            assert!(!task.checks(&state).unwrap()[4]);
            assert!(task.public_lookup(&state).unwrap().authoritative);
        }
    }
}

#[test]
fn ordinary_terminal_inspection_retains_current_grading_and_state_changes_clear_it() {
    let task = sockets().remove("assignment").unwrap();
    let definitions = Task::SocketRetirement {
        supplied: Box::new(task.clone()),
    };
    let mut work = Work::new(&definitions);
    let command = |verb: &str, name: &str| Action::RetirementCommand {
        text: format!("systemctl {verb} {name}"),
    };
    for name in [&task.obsolete_service, &task.activating_socket] {
        work.apply(&definitions, command("disable --now", name))
            .unwrap();
    }
    work.apply(&definitions, Action::Check).unwrap();
    assert!(
        work.checked
            && work
                .outcomes(&definitions)
                .into_iter()
                .all(|finding| finding.passed)
    );
    work.apply(&definitions, command("status", &task.activating_socket))
        .unwrap();
    work.apply(
        &definitions,
        Action::RetirementDraft {
            text: "systemctl ".into(),
        },
    )
    .unwrap();
    assert!(work.checked);
    work.apply(&definitions, command("start", &task.activating_socket))
        .unwrap();
    assert!(!work.checked);
    assert!(
        work.outcomes(&definitions)
            .into_iter()
            .any(|finding| !finding.passed)
    );
}

#[test]
fn fragments_accept_real_syntax_and_invalid_edits_preserve_the_applied_copy() {
    let task = transfers().remove("example").unwrap();
    let key = task.approved[1].clone();
    let accepted = format!(
        "/* use the approved spare */ allow-transfer {{ key \"{}\"; }};",
        key.name
    );
    assert_eq!(
        parse_transfer_fragment(&accepted, &task).unwrap(),
        Policy::VerifiedKey {
            reference: key.clone()
        }
    );
    for text in [
        "include \"/etc/named.conf\";",
        "allow-transfer { any; };",
        "allow-transfer { none; key elsewhere; };",
        "allow-transfer { key unknown.example.; };",
        "allow-transfer { none; }; zone invalid",
    ] {
        assert!(parse_transfer_fragment(text, &task).is_err(), "{text}");
    }
    let mut edit = Edit::new(&task);
    edit.change(false, accepted).unwrap();
    edit.selected_key = Some(key);
    edit.apply(&task).unwrap();
    edit.trial(&task, Client::ConfiguredSecondary, false)
        .unwrap();
    assert!(
        edit.outcomes(&task)
            .unwrap()
            .into_iter()
            .all(|finding| finding.0)
    );
    let live = edit.actual.clone();
    edit.change(true, "allow-transfer { incomplete".into())
        .unwrap();
    assert!(edit.apply(&task).is_err());
    assert_eq!(edit.actual, live);
    assert!(!edit.outcomes(&task).unwrap()[0].0);
}
