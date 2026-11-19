//! Actual virtual actions and persistence; no synthetic command-to-success responses.
use dlms_learning_lab::{
    script_teaching::View,
    vpn_point_workspace::{Action, Query, Scope, Session},
};
#[path = "support/vpn_point_fixture.rs"]
mod fixture;
#[path = "support/vpn_point_task_fixture.rs"]
mod task_fixture;
use fixture::peer;
#[path = "support/vpn_point_workspace_fixture.rs"]
mod work_fixture;
use work_fixture::{definition, repair};
#[test]
fn separate_practice_actions_never_overwrite_or_complete_assignment() {
    let d = definition();
    d.validate().unwrap();
    let mut s = Session::new(&d).unwrap();
    let before = s.assignment.clone();
    for a in &d.worked {
        s.action(&d, a, true).unwrap();
    }
    assert!(s.example.solved(&d.example));
    assert_eq!(s.assignment, before);
    assert!(!s.assignment.solved(&d.assignment));
    for a in repair() {
        s.action(&d, &a, false).unwrap();
    }
    assert!(s.assignment.solved(&d.assignment));
    assert_ne!(s.assignment.world.applied, s.example.world.applied);
    assert_eq!(s.assignment.world.applied, Some(fixture::configs()));
}
#[test]
fn editor_changes_clear_success_and_queries_never_show_stale_results() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    for a in repair() {
        s.apply(&d.assignment, &a).unwrap();
    }
    s.apply(&d.assignment, &Action::Probe).unwrap();
    assert!(s.visible_probe().is_some());
    assert!(s.solved(&d.assignment));
    let changed = s.editors[0].clone() + "# Unfinished edit\n";
    s.apply(
        &d.assignment,
        &Action::Edit {
            peer: 0,
            text: changed,
        },
    )
    .unwrap();
    assert!(!s.checked);
    assert!(!s.solved(&d.assignment));
    assert!(s.visible_probe().is_none());
    s.bound(&d.assignment).unwrap();
    s.apply(&d.assignment, &Action::Save { peer: 0 }).unwrap();
    s.apply(&d.assignment, &Action::Apply).unwrap();
    s.apply(&d.assignment, &Action::Check).unwrap();
    assert!(s.solved(&d.assignment));
    s.apply(&d.assignment, &Action::Probe).unwrap();
    let mut query = s.query.clone();
    query.destination = "198.51.100.7".into();
    s.apply(&d.assignment, &Action::Query { query }).unwrap();
    assert!(s.visible_probe().is_none());
    s.apply(&d.assignment, &Action::Probe).unwrap();
    assert_eq!(
        s.visible_probe().unwrap().outcome,
        dlms_learning_lab::vpn_point_task::Outcome::OutsideTunnel
    );
    assert!(s.solved(&d.assignment));
}
#[test]
fn syntax_and_unsupported_feedback_is_specific_and_disappears_after_input_changes() {
    let d = definition();
    for (bad, kind) in [
        (
            "dev tun\nproto udp\nifconfig bad 10.8.0.1\n",
            "Syntax error",
        ),
        ("up /etc/passwd\n", "Unsupported setting"),
    ] {
        let mut s = Scope::new(&d.assignment).unwrap();
        s.apply(
            &d.assignment,
            &Action::Edit {
                peer: 1,
                text: bad.into(),
            },
        )
        .unwrap();
        s.apply(&d.assignment, &Action::Save { peer: 1 }).unwrap();
        let applied = s.world.applied.clone();
        s.apply(&d.assignment, &Action::Apply).unwrap();
        assert_eq!(s.world.applied, applied);
        assert_eq!(s.visible_notice().unwrap().kind, kind);
        assert!(
            s.visible_notice()
                .unwrap()
                .message
                .contains("branch profile")
        );
        assert!(s.visible_notice().unwrap().message.contains("Line"));
        assert!(!s.solved(&d.assignment));
        s.apply(
            &d.assignment,
            &Action::Edit {
                peer: 1,
                text: peer(false),
            },
        )
        .unwrap();
        assert!(s.visible_notice().is_none());
        for a in [Action::Save { peer: 1 }, Action::Apply, Action::Check] {
            s.apply(&d.assignment, &a).unwrap();
        }
        assert!(s.solved(&d.assignment));
    }
}
#[test]
fn roundtrip_keeps_independent_unsaved_drafts_and_reset_changes_only_selected_scope() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    for a in &d.worked {
        s.action(&d, a, true).unwrap();
    }
    for a in repair() {
        s.action(&d, &a, false).unwrap();
    }
    assert_eq!(s.assignment.world, task_fixture::repaired(&d.assignment));
    s.action(&d, &Action::Probe, false).unwrap();
    s.action(
        &d,
        &Action::Edit {
            peer: 0,
            text: peer(true) + "# assignment draft\n",
        },
        false,
    )
    .unwrap();
    let example_text = d.example.initial.saved[0].clone() + "# independent example draft\n";
    s.action(
        &d,
        &Action::Edit {
            peer: 0,
            text: example_text,
        },
        true,
    )
    .unwrap();
    s.view = View::Assignment;
    let raw = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&raw).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert_ne!(restored.assignment.editors, restored.example.editors);
    assert!(restored.assignment.last_probe.is_some());
    assert!(restored.assignment.visible_probe().is_none());
    let example = restored.example.clone();
    restored.reset(&d, false).unwrap();
    assert_eq!(restored.example, example);
    assert_eq!(restored.view, View::Assignment);
    assert_eq!(restored.assignment, Scope::new(&d.assignment).unwrap());
}
#[test]
fn forged_reports_changed_source_and_malformed_inputs_do_not_create_completion() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    for a in repair() {
        s.action(&d, &a, false).unwrap();
    }
    s.action(&d, &Action::Probe, false).unwrap();
    let mut forged = s.clone();
    forged.assignment.last_probe.as_mut().unwrap().detail = "Invented successful tunnel".into();
    assert!(d.bound(&forged).is_err());
    assert!(!forged.assignment.solved(&d.assignment));
    let mut changed = d.clone();
    changed.assignment.cases[0].label = "New required case wording".into();
    changed.validate().unwrap();
    assert!(changed.bound(&s).is_err());
    let before = s.clone();
    assert!(
        s.action(
            &d,
            &Action::Edit {
                peer: 2,
                text: "invalid editor".into()
            },
            false
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert!(
        s.action(
            &d,
            &Action::Edit {
                peer: 0,
                text: "x".repeat(8193)
            },
            false
        )
        .is_err()
    );
    assert_eq!(s, before);
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["editors"][0] = serde_json::json!("x".repeat(8193));
    assert!(serde_json::from_value::<Session>(value).is_err());
}
#[test]
fn blank_test_input_is_an_honest_diagnostic_and_does_not_change_correct_configuration() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    for a in repair() {
        s.apply(&d.assignment, &a).unwrap();
    }
    let world = s.world.clone();
    s.apply(
        &d.assignment,
        &Action::Query {
            query: Query {
                from: 0,
                source: String::new(),
                destination: "10.40.0.41".into(),
            },
        },
    )
    .unwrap();
    s.apply(&d.assignment, &Action::Probe).unwrap();
    assert!(s.visible_probe().is_none());
    let n = s.visible_notice().unwrap();
    assert_eq!(n.kind, "Syntax error");
    assert!(n.message.contains("both source and destination"));
    assert_eq!(s.world, world);
    assert!(s.solved(&d.assignment));
    s.apply(
        &d.assignment,
        &Action::Query {
            query: Query {
                from: 0,
                source: "10.20.0.21".into(),
                destination: "10.40.0.41".into(),
            },
        },
    )
    .unwrap();
    assert!(s.visible_notice().is_none());
}
#[test]
fn history_limits_refuse_atomically_and_practice_work_must_be_reachable() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    for a in repair() {
        s.apply(&d.assignment, &a).unwrap();
    }
    while s.history.len() < 64 {
        s.apply(&d.assignment, &Action::Probe).unwrap();
    }
    let before = s.clone();
    assert!(s.apply(&d.assignment, &Action::Probe).is_err());
    assert_eq!(s, before);
    let mut bad = d.clone();
    bad.worked = vec![Action::Check];
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    let extra = value["history"][0].clone();
    value["history"].as_array_mut().unwrap().push(extra);
    assert!(serde_json::from_value::<Scope>(value).is_err());
}
