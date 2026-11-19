//! Replay/resume tests use the actual bounded model, not injected success strings.
use dlms_learning_lab::{
    script_teaching::View,
    service_exposure_workspace::{Action, Event, Scope, Session},
};
#[path = "support/service_exposure_fixture.rs"]
mod server_fixture;
#[path = "support/service_exposure_task_fixture.rs"]
mod task_fixture;
#[path = "support/service_exposure_workspace_fixture.rs"]
mod workspace_fixture;
use workspace_fixture::{action, definition, repair};
#[test]
fn example_repair_does_not_overwrite_or_complete_the_assignment() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    for a in &d.worked {
        s.action(&d, a, true).unwrap();
    }
    assert!(s.example.solved(&d.example));
    assert_eq!(s.assignment, original);
    assert!(!s.assignment.solved(&d.assignment));
    for a in repair() {
        s.action(&d, &a, false).unwrap();
    }
    assert!(s.assignment.solved(&d.assignment));
    assert_eq!(s.assignment.world, task_fixture::solve(&d.assignment));
}
#[test]
fn separate_drafts_reports_observers_and_exact_actions_resume_and_selected_reset_isolated() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.action(
        &d,
        &Action::AdminInput {
            text: "ss -lntp".into(),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::ScanInput {
            text: "unfinished example".into(),
        },
        true,
    )
    .unwrap();
    s.action(&d, &action(Event::Select { viewpoint: 1 }), false)
        .unwrap();
    s.action(
        &d,
        &action(Event::Scan {
            input: "nmap -n -Pn -sT -p 22,80,443 203.0.113.10".into(),
        }),
        false,
    )
    .unwrap();
    s.view = View::Assignment;
    let wire = serde_json::to_string(&s).unwrap();
    let mut restored: Session = serde_json::from_str(&wire).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(restored.assignment.last_scan.is_some());
    let example = restored.example.clone();
    restored.reset(&d, false).unwrap();
    assert_eq!(restored.example, example);
    assert_eq!(restored.assignment, Scope::new(&d.assignment).unwrap());
    assert_eq!(restored.view, View::Assignment);
}
#[test]
fn mutations_and_observer_changes_clear_stale_observations_and_broken_success() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    for a in repair() {
        s.apply(&d.assignment, &a).unwrap();
    }
    assert!(s.solved(&d.assignment));
    let scan = action(Event::Scan {
        input: "nmap -n -Pn -sT -p 22,443 203.0.113.10".into(),
    });
    s.apply(&d.assignment, &scan).unwrap();
    assert!(s.last_scan.is_some());
    assert!(s.solved(&d.assignment));
    s.apply(&d.assignment, &action(Event::Select { viewpoint: 1 }))
        .unwrap();
    assert!(s.last_scan.is_none());
    assert!(s.scan_output.is_empty());
    assert!(s.solved(&d.assignment));
    s.apply(&d.assignment, &scan).unwrap();
    s.apply(
        &d.assignment,
        &action(Event::Admin {
            input: "iptables -F INPUT".into(),
        }),
    )
    .unwrap();
    assert!(!s.checked);
    assert!(!s.solved(&d.assignment));
    assert!(s.last_scan.is_none());
    assert!(s.scan_output.is_empty());
}
#[test]
fn unsupported_inputs_and_partial_failed_commands_replay_with_honest_status() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    let original = s.world.clone();
    s.apply(
        &d.assignment,
        &action(Event::Scan {
            input: "nmap --script all 203.0.113.10".into(),
        }),
    )
    .unwrap();
    assert_eq!(s.scan_status, Some(1));
    assert!(s.scan_output.contains("Unsupported"));
    assert!(s.last_scan.is_none());
    assert_eq!(s.world, original);
    s.apply(
        &d.assignment,
        &action(Event::Admin {
            input: "kill -TERM 101 999".into(),
        }),
    )
    .unwrap();
    assert_eq!(s.admin_status, Some(1));
    assert_ne!(s.world, original);
    s.bound(&d.assignment).unwrap();
    let restored: Scope = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    restored.bound(&d.assignment).unwrap();
}
#[test]
fn source_changes_and_forged_state_outputs_credit_or_probe_reports_are_rejected_without_loss() {
    let d = definition();
    let mut original = Scope::new(&d.assignment).unwrap();
    for a in repair() {
        original.apply(&d.assignment, &a).unwrap();
    }
    assert!(original.solved(&d.assignment));
    let mut changed = d.assignment.clone();
    changed.cases[0].label = "New requirement wording".into();
    assert!(original.bound(&changed).is_err());
    assert!(original.solved(&d.assignment));
    for kind in 0..5 {
        let mut forged = original.clone();
        match kind {
            0 => forged.world.processes.processes.get_mut(&201).unwrap().nice = 5,
            1 => forged.admin_output.push_str("Forged result"),
            2 => forged.checked = false,
            3 => forged.admin_status = Some(99),
            _ => forged.source_task = "{}".into(),
        }
        let before = forged.clone();
        assert!(forged.bound(&d.assignment).is_err());
        assert!(forged.apply(&d.assignment, &action(Event::Check)).is_err());
        assert_eq!(forged, before);
    }
    let mut forged = Scope::new(&d.assignment).unwrap();
    forged.last_scan = Some(dlms_learning_lab::service_exposure_workspace::Scan {
        viewpoint: 0,
        rows: vec![],
    });
    assert!(forged.bound(&d.assignment).is_err());
}
#[test]
fn replay_history_and_input_limits_refuse_atomically_without_host_execution() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    let before = s.clone();
    assert!(
        s.apply(
            &d.assignment,
            &Action::AdminInput {
                text: "x".repeat(4097)
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
    let a = action(Event::Scan {
        input: "nmap".into(),
    });
    for _ in 0..96 {
        s.apply(&d.assignment, &a).unwrap();
    }
    s.bound(&d.assignment).unwrap();
    let before = s.clone();
    assert!(s.apply(&d.assignment, &a).is_err());
    assert_eq!(s, before);
}
#[test]
fn malformed_unreachable_and_cross_lesson_definitions_cannot_award_credit() {
    let d = definition();
    let mut bad = d.clone();
    bad.worked.clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.worked = vec![action(Event::Check)];
    assert!(bad.validate().is_err());
    let mut s = Session::new(&d).unwrap();
    s.source_id = "foreign-lesson".into();
    let original = s.clone();
    assert!(s.action(&d, &action(Event::Check), false).is_err());
    assert!(s.reset(&d, true).is_err());
    assert_eq!(s, original);
}

#[test]
fn old_unsupported_nc_history_stays_unsupported_but_new_probes_resume_exactly() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    let world = s.world.clone();
    let command = "nc -n -v -z 203.0.113.10 443";
    s.apply(
        &d.assignment,
        &action(Event::Scan {
            input: command.into(),
        }),
    )
    .unwrap();
    assert_eq!(s.scan_status, Some(1));
    assert!(s.scan_output.contains("Unsupported"));
    assert!(s.last_scan.is_none());
    let old_output = s.scan_output.clone();
    assert_eq!(
        old_output,
        "observer $ nc -n -v -z 203.0.113.10 443\nUnsupported command or syntax: Scanner subset: nmap -n -Pn -sT -p PORT[,PORT...] SUPPLIED_IP. Other scanner syntax is not implemented.\n"
    );
    let restored: Scope = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    restored.bound(&d.assignment).unwrap();
    assert_eq!(restored, s);
    s.apply(
        &d.assignment,
        &action(Event::ProbeV2 {
            input: command.into(),
        }),
    )
    .unwrap();
    assert!(s.scan_output.starts_with(&old_output));
    assert!(s.scan_output.contains("succeeded!"));
    assert_eq!(s.scan_status, Some(0));
    assert!(s.last_scan.is_some());
    assert_eq!(s.world, world);
    assert!(!s.checked && !s.solved(&d.assignment));
    assert!(matches!(
        &s.history[..],
        [Event::Scan { .. }, Event::ProbeV2 { .. }]
    ));
    let restored: Scope = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    restored.bound(&d.assignment).unwrap();
    assert_eq!(restored, s);
}

#[test]
fn negative_probe_observation_differs_from_invalid_command_and_neither_grants_credit() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    for (command, diagnostic, observation) in [
        ("nc -nvz 203.0.113.10 80", "Connection refused", true),
        ("nc -nvz 203.0.113.10 9001", "Connection refused", true),
        ("nc -ul 203.0.113.10 443", "Unsupported", false),
        ("nc -nvz 198.51.100.99 443", "Workspace requirement", false),
        (
            "nc -nvz 203.0.113.10 443; touch /home/test",
            "Unsupported",
            false,
        ),
    ] {
        let world = s.world.clone();
        s.apply(
            &d.assignment,
            &action(Event::ProbeV2 {
                input: command.into(),
            }),
        )
        .unwrap();
        assert_eq!(s.scan_status, Some(1), "{command}");
        assert!(s.scan_output.contains(diagnostic), "{command}");
        assert_eq!(s.last_scan.is_some(), observation, "{command}");
        assert_eq!(s.world, world);
        assert!(!s.solved(&d.assignment));
        s.bound(&d.assignment).unwrap();
    }
    for a in repair() {
        s.apply(&d.assignment, &a).unwrap();
    }
    assert!(s.solved(&d.assignment));
    s.apply(&d.assignment, &action(Event::Select { viewpoint: 1 }))
        .unwrap();
    s.apply(
        &d.assignment,
        &action(Event::ProbeV2 {
            input: "nc -nvz 203.0.113.10 22".into(),
        }),
    )
    .unwrap();
    assert_eq!(s.scan_status, Some(1));
    assert!(s.last_scan.is_some());
    assert!(s.scan_output.contains("modeled drop"));
    assert!(
        s.solved(&d.assignment),
        "A valid negative connection is useful evidence, not failed grading"
    );
    s.apply(
        &d.assignment,
        &action(Event::Admin {
            input: "iptables -F INPUT".into(),
        }),
    )
    .unwrap();
    assert!(!s.checked && !s.solved(&d.assignment));
    assert!(s.last_scan.is_none() && s.scan_status.is_none());
}

#[test]
fn new_nmap_keeps_legacy_report_and_probe_overflow_refuses_atomically() {
    let d = definition();
    let mut old = Scope::new(&d.assignment).unwrap();
    let mut new = old.clone();
    let input = "nmap -n -Pn -sT -p 22,80,443 203.0.113.10";
    old.apply(
        &d.assignment,
        &action(Event::Scan {
            input: input.into(),
        }),
    )
    .unwrap();
    new.apply(
        &d.assignment,
        &action(Event::ProbeV2 {
            input: input.into(),
        }),
    )
    .unwrap();
    assert_eq!(old.scan_output, new.scan_output);
    assert_eq!(old.scan_status, new.scan_status);
    assert_eq!(old.last_scan, new.last_scan);
    let before = new.clone();
    assert!(
        new.apply(
            &d.assignment,
            &action(Event::ProbeV2 {
                input: "x".repeat(1025)
            })
        )
        .is_err()
    );
    assert_eq!(new, before);
    new.bound(&d.assignment).unwrap();
    let mut forged = new.clone();
    forged.scan_status = Some(1);
    assert!(forged.bound(&d.assignment).is_err());
}
