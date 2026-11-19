use dlms_learning_lab::{
    ftp_identity_workspace::{Action, Definition, Event, Scope, Session},
    script_teaching::View,
};
#[path = "support/ftp_identity_fixture.rs"]
mod fixture;
#[path = "support/ftp_identity_task_fixture.rs"]
mod task_fixture;
use std::collections::BTreeMap;
fn definition() -> Definition {
    let mut assignment = task_fixture::task();
    assignment.initial = fixture::world();
    assignment.initial.saved.push_str("local_enable=NO\n");
    assignment.initial.restart().unwrap();
    let mut example = assignment.clone();
    example.id = "separate-example".into();
    example.initial.server = "example-ftp".into();
    Definition {
        source_id: "ftp-test-source".into(),
        assignment,
        example,
        assignment_inputs: BTreeMap::from([("report.txt".into(), "assignment upload\n".into())]),
        example_inputs: BTreeMap::from([("report.txt".into(), "example upload\n".into())]),
        worked: vec![
            Action::Edit {
                text: fixture::settings(),
            },
            Action::Save,
            Action::Event {
                event: Event::Admin {
                    input: "systemctl restart vsftpd".into(),
                },
            },
            Action::Check,
        ],
    }
}
fn event(s: &mut Scope, d: &Definition, ev: Event) {
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::Event { event: ev },
    )
    .unwrap();
}
fn repair(s: &mut Scope, d: &Definition) {
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::Edit {
            text: fixture::settings(),
        },
    )
    .unwrap();
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Save)
        .unwrap();
    event(
        s,
        d,
        Event::Admin {
            input: "systemctl restart vsftpd".into(),
        },
    );
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Check)
        .unwrap();
}
#[test]
fn worked_example_is_separate_and_cannot_complete_or_overwrite_assignment() {
    let d = definition();
    d.validate().unwrap();
    let mut session = Session::new(&d).unwrap();
    let assignment = session.assignment.clone();
    for a in &d.worked {
        session
            .example
            .apply(&d.example, &d.example_inputs, a)
            .unwrap();
    }
    assert!(session
        .example
        .checks(&d.example, &d.example_inputs)
        .iter()
        .all(|c| c.passed));
    assert_eq!(session.assignment, assignment);
    assert!(!session.assignment.checked);
    session.view = View::Assignment;
    session.bound(&d).unwrap();
}
#[test]
fn drafts_cwd_pending_commands_and_client_files_resume_through_replay() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    repair(&mut s.assignment, &d);
    event(&mut s.assignment, &d, Event::Connect { client: 0 });
    event(
        &mut s.assignment,
        &d,
        Event::Client {
            input: "cd inbox".into(),
        },
    );
    event(
        &mut s.assignment,
        &d,
        Event::Client {
            input: "put report.txt submitted.txt".into(),
        },
    );
    s.assignment
        .apply(
            &d.assignment,
            &d.assignment_inputs,
            &Action::Edit {
                text: fixture::settings() + "# unfinished assignment",
            },
        )
        .unwrap();
    s.assignment
        .apply(
            &d.assignment,
            &d.assignment_inputs,
            &Action::ClientInput {
                text: "get submitted.txt copy.txt".into(),
            },
        )
        .unwrap();
    s.example
        .apply(
            &d.example,
            &d.example_inputs,
            &Action::Edit {
                text: "# separate unfinished example\n".into(),
            },
        )
        .unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    assert!(!raw.contains("\"connection\""));
    let mut restored: Session = serde_json::from_str(&raw).unwrap();
    restored.bound(&d).unwrap();
    assert_eq!(s, restored);
    let pending = restored.assignment.client_input.clone();
    event(
        &mut restored.assignment,
        &d,
        Event::Client { input: pending },
    );
    assert_eq!(
        restored.assignment.local_files["copy.txt"],
        "assignment upload\n"
    );
    assert!(restored
        .assignment
        .client_output
        .contains("simulated client file copy.txt"));
    assert_eq!(restored.example, s.example);
}
#[test]
fn edits_invalidate_success_and_read_only_commands_preserve_valid_current_success() {
    let d = definition();
    let mut s = Scope::new(&d.assignment, &d.assignment_inputs);
    repair(&mut s, &d);
    assert!(s
        .checks(&d.assignment, &d.assignment_inputs)
        .iter()
        .all(|c| c.passed));
    event(
        &mut s,
        &d,
        Event::Admin {
            input: "systemctl status vsftpd".into(),
        },
    );
    assert!(s.checked);
    assert!(s
        .checks(&d.assignment, &d.assignment_inputs)
        .iter()
        .all(|c| c.passed));
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::Edit {
            text: "local_enable=NO\n".into(),
        },
    )
    .unwrap();
    assert!(!s.checked);
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Check)
        .unwrap();
    assert!(s
        .checks(&d.assignment, &d.assignment_inputs)
        .iter()
        .any(|c| !c.passed));
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Save)
        .unwrap();
    event(
        &mut s,
        &d,
        Event::Admin {
            input: "systemctl restart vsftpd".into(),
        },
    );
    assert!(s
        .checks(&d.assignment, &d.assignment_inputs)
        .iter()
        .any(|c| !c.passed));
}
#[test]
fn failed_restart_is_replayed_and_stale_client_does_not_remain_connected() {
    let d = definition();
    let mut s = Scope::new(&d.assignment, &d.assignment_inputs);
    repair(&mut s, &d);
    event(&mut s, &d, Event::Connect { client: 0 });
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::Edit {
            text: "unimplemented_option=YES".into(),
        },
    )
    .unwrap();
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Save)
        .unwrap();
    event(
        &mut s,
        &d,
        Event::Admin {
            input: "systemctl restart vsftpd".into(),
        },
    );
    assert!(s.world.published.is_none());
    assert!(s.admin_output.contains("outside this simulator"));
    event(
        &mut s,
        &d,
        Event::Client {
            input: "pwd".into(),
        },
    );
    assert!(s
        .client_output
        .contains("Connect using a supplied client identity first"));
    s.bound(&d.assignment, &d.assignment_inputs).unwrap();
    let restored: Scope = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    restored.bound(&d.assignment, &d.assignment_inputs).unwrap();
}
#[test]
fn configuration_file_switch_does_not_discard_an_unsaved_draft() {
    let d = definition();
    let mut s = Scope::new(&d.assignment, &d.assignment_inputs);
    let path = "/etc/vsftpd.chroot_list".to_string();
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::Edit {
            text: "# unfinished".into(),
        },
    )
    .unwrap();
    let before = s.clone();
    assert!(s
        .apply(
            &d.assignment,
            &d.assignment_inputs,
            &Action::ChooseFile { path: path.clone() }
        )
        .is_err());
    assert_eq!(s, before);
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Reload)
        .unwrap();
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::ChooseFile { path: path.clone() },
    )
    .unwrap();
    s.apply(
        &d.assignment,
        &d.assignment_inputs,
        &Action::Edit {
            text: "alex\n".into(),
        },
    )
    .unwrap();
    s.apply(&d.assignment, &d.assignment_inputs, &Action::Save)
        .unwrap();
    assert_eq!(s.world.files.files[&path], "alex\n");
    s.bound(&d.assignment, &d.assignment_inputs).unwrap();
}
#[test]
fn forged_state_output_trial_client_buffers_and_oversized_actions_are_refused_atomically() {
    let d = definition();
    let s = Scope::new(&d.assignment, &d.assignment_inputs);
    for n in 0..4 {
        let mut bad = s.clone();
        match n {
            0 => bad.world.saved = fixture::settings(),
            1 => bad.client_output = "fake result".into(),
            2 => {
                bad.local_files
                    .insert("secret".into(), "fake download".into());
            }
            _ => bad.last_trial = Some(vec![]),
        };
        assert!(bad.bound(&d.assignment, &d.assignment_inputs).is_err());
    }
    let mut s = s;
    let before = s.clone();
    assert!(s
        .apply(
            &d.assignment,
            &d.assignment_inputs,
            &Action::Edit {
                text: "x".repeat(16385)
            }
        )
        .is_err());
    assert_eq!(s, before);
    assert!(s
        .apply(
            &d.assignment,
            &d.assignment_inputs,
            &Action::Event {
                event: Event::SaveFile {
                    path: "/etc/secret".into(),
                    text: "wrong".into()
                }
            }
        )
        .is_err());
    assert_eq!(s, before);
}
#[test]
fn reset_only_replaces_the_selected_scope_and_definition_requires_reachable_work() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    repair(&mut s.assignment, &d);
    let assignment = s.assignment.clone();
    s.example = Scope::new(&d.example, &d.example_inputs);
    assert_eq!(s.assignment, assignment);
    let mut bad = d.clone();
    bad.worked = vec![Action::Check];
    assert!(bad.validate().is_err());
    let mut bad = d;
    bad.example.initial.server = bad.assignment.initial.server.clone();
    assert!(bad.validate().is_err());
}
