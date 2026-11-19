use dlms_learning_lab::{
    smtp_config::Config,
    smtp_equivalence,
    smtp_routing::{Outcome, Trace},
    smtp_workspace::{Action, Expected, Session},
};
use std::collections::BTreeSet;
#[path = "support/smtp_tasks.rs"]
mod setup;
fn solved() -> (dlms_learning_lab::smtp_workspace::Definition, Session) {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    for a in &d.reference {
        s.action(&d, a, false).unwrap();
    }
    (d, s)
}
#[test]
fn separate_taught_methods_reach_all_required_results_and_only_assignment_check_earns_current_success()
 {
    let d = setup::definition();
    d.validate_reachability().unwrap();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    for a in &d.example_steps {
        s.action(&d, a, true).unwrap();
    }
    assert_eq!(s.assignment, original);
    assert!(!d.current(&s));
    assert!(s.practice.checks(&d.example).iter().all(|r| r.passed));
    for a in &d.reference {
        s.action(&d, a, false).unwrap();
    }
    assert!(d.current(&s));
    assert_eq!(s.assignment.selected, "relay");
}
#[test]
fn partial_unsaved_unloaded_and_not_retried_work_are_distinct_failures_and_old_output_never_grades()
{
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    s.assignment.output = "Assignment complete: everything passed".into();
    s.assignment.checked = true;
    assert!(!d.current(&s));
    s.action(&d, &d.reference[0], false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.current(&s));
    assert!(!s.assignment.checks(&d.assignment)[0].passed);
    s.action(&d, &d.reference[1], false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.current(&s));
    s.action(&d, &d.reference[2], false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.current(&s));
    assert!(
        s.assignment
            .checks(&d.assignment)
            .iter()
            .filter(|r| r.label.starts_with("Queued sample"))
            .all(|r| !r.passed)
    );
    s.action(&d, &d.reference[3], false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.current(&s));
}
#[test]
fn later_edits_and_reloads_invalidate_success_despite_old_delivery_records_and_a_stored_check_flag()
{
    let (d, mut s) = solved();
    assert!(d.current(&s));
    let wrong = d.assignment.facts.initial_files["relay"].clone();
    s.action(
        &d,
        &Action::Edit {
            node: "relay".into(),
            text: wrong,
        },
        false,
    )
    .unwrap();
    assert!(!d.current(&s));
    s.action(
        &d,
        &Action::Save {
            node: "relay".into(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.current(&s));
    s.action(
        &d,
        &Action::Reload {
            node: "relay".into(),
        },
        false,
    )
    .unwrap();
    s.assignment.checked = true;
    assert!(!d.current(&s));
}
#[test]
fn supported_alternate_endpoint_and_equivalent_protected_settings_pass_by_behavior() {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    let (_, working) = solved();
    let relay =
        working.assignment.runtime.files["relay"].replace("[delivery.lab.test]", "[192.0.2.30]");
    let gate = working.assignment.runtime.files["gate"]
        .replace("192.0.2.0/24", "192.0.2.0/25, 192.0.2.128/25")
        .replace(
            "permit_mynetworks, reject_unauth_destination",
            "permit_mynetworks, permit_mynetworks, reject_unauth_destination",
        );
    for (node, text) in [("relay", relay), ("gate", gate)] {
        s.action(
            &d,
            &Action::Edit {
                node: node.into(),
                text,
            },
            false,
        )
        .unwrap();
        s.action(&d, &Action::Save { node: node.into() }, false)
            .unwrap();
        s.action(&d, &Action::Reload { node: node.into() }, false)
            .unwrap();
    }
    s.action(
        &d,
        &Action::Flush {
            node: "gate".into(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.current(&s), "{:?}", s.assignment.checks(&d.assignment));
}
#[test]
fn refusal_is_an_expected_behavior_not_any_failure_and_a_correct_negative_case_cannot_hide_failed_positive_cases()
 {
    let wanted = Expected::Reject {
        node: "gate".into(),
    };
    assert!(!wanted.matches(&Trace {
        hops: vec![],
        outcome: Outcome::NoRoute {
            node: "gate".into()
        }
    }));
    assert!(!wanted.matches(&Trace {
        hops: vec![],
        outcome: Outcome::Unavailable {
            target: "unknown.test".into(),
            port: 25
        }
    }));
    assert!(wanted.matches(&Trace {
        hops: vec![],
        outcome: Outcome::Rejected {
            node: "gate".into()
        }
    }));
    let d = setup::definition();
    let s = Session::new(&d).unwrap();
    let checks = s.assignment.checks(&d.assignment);
    assert!(
        checks
            .iter()
            .any(|r| r.label == "Outside client cannot relay outward" && r.passed)
    );
    assert!(
        checks
            .iter()
            .any(|r| r.label == "Outside client reaches the staff mailbox" && !r.passed)
    );
    assert!(!checks.iter().all(|r| r.passed));
}
#[test]
fn resume_preserves_two_drafts_selections_and_histories_and_reset_affects_only_selected_scope() {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    s.action(
        &d,
        &Action::Edit {
            node: "relay".into(),
            text: "unfinished assignment text".into(),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Edit {
            node: "guide-relay".into(),
            text: "unfinished practice text".into(),
        },
        true,
    )
    .unwrap();
    s.assignment.input = "postfix re".into();
    s.practice.input = "postqueue ".into();
    s.assignment.output = "Actual earlier assignment history".into();
    s.practice.output = "Actual different practice history".into();
    let raw = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&raw).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(!d.current(&restored));
    let assignment = restored.assignment.clone();
    restored.reset(&d, true).unwrap();
    assert_eq!(restored.assignment, assignment);
    let example = restored.practice.clone();
    restored.reset(&d, false).unwrap();
    assert_eq!(restored.practice, example);
    assert_eq!(
        restored.assignment.runtime,
        Session::new(&d).unwrap().assignment.runtime
    );
}
#[test]
fn invalid_source_unreachable_methods_and_foreign_work_are_rejected_without_vacuous_credit() {
    let d = setup::definition();
    let mut bad = d.clone();
    bad.assignment.probes.clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.probes.push(bad.assignment.probes[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.probes[0].expected = Expected::Delivery {
        node: "delivery".into(),
        mailboxes: BTreeSet::from(["not_a_user".into()]),
    };
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example.facts = bad.assignment.facts.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.reference = vec![Action::Check];
    assert!(bad.validate_reachability().is_err());
    let mut s = Session::new(&d).unwrap();
    s.source_id = "another-source".into();
    assert!(d.bound(&s).is_err());
    assert!(!d.current(&s));
    let mut s = Session::new(&d).unwrap();
    s.assignment.drafts.insert("../../host".into(), "x".into());
    assert!(d.bound(&s).is_err());
}
#[test]
fn network_union_equivalence_covers_every_prefix_and_policy_equivalence_keeps_reject_distinct_from_defer()
 {
    let d = setup::definition();
    let template = &d.assignment.facts.initial_active["gate"];
    for prefix in 0..32u8 {
        let span = 1u64 << (32 - prefix);
        let at = u64::from(u32::from(
            "198.51.100.217".parse::<std::net::Ipv4Addr>().unwrap(),
        ));
        let base = (at / span) * span;
        let parent = format!("{}/{prefix}", std::net::Ipv4Addr::from(base as u32));
        let children = format!(
            "{}/{}, {}/{}",
            std::net::Ipv4Addr::from(base as u32),
            prefix + 1,
            std::net::Ipv4Addr::from((base + span / 2) as u32),
            prefix + 1
        );
        let a = Config::parse(&template.replace("192.0.2.0/24", &parent)).unwrap();
        let b = Config::parse(&template.replace("192.0.2.0/24", &children)).unwrap();
        assert!(smtp_equivalence::field(&a, &b, "mynetworks"));
    }
    let a = Config::parse(template).unwrap();
    let b =
        Config::parse(&template.replace("reject_unauth_destination", "defer_unauth_destination"))
            .unwrap();
    assert!(!smtp_equivalence::field(&a, &b, "smtpd_relay_restrictions"));
    assert!(!smtp_equivalence::field(&a, &b, "foreign_parameter"));
}
