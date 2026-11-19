//! Source-bound config drafts, separate trials and behavior checks; no host execution.
use dlms_learning_lab::linux_flow_workspace::{Action, Definition, Expected, Scope, Session};
fn fixtures() -> Vec<(Definition, Vec<Action>)> {
    let v: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/linux-flow-workspace-v1.json")).unwrap();
    v.into_iter()
        .map(|v| {
            (
                serde_json::from_value(v["definition"].clone()).unwrap(),
                serde_json::from_value(v["solution"].clone()).unwrap(),
            )
        })
        .collect()
}
fn repair(session: &mut Session, d: &Definition, actions: &[Action]) {
    for a in actions {
        session.action(d, a, false).unwrap();
    }
}
#[test]
fn taught_methods_reach_all_actual_outcomes_in_both_substantive_profiles() {
    for (d, solution) in fixtures() {
        d.validate().unwrap();
        let mut session = Session::new(&d).unwrap();
        assert!(!d.solved(&session));
        assert!(
            d.assignment
                .checks(&session.assignment)
                .iter()
                .any(|c| !c.passed)
        );
        repair(&mut session, &d, &solution);
        assert!(
            d.solved(&session),
            "{}: {:?}",
            d.source_id,
            d.assignment.checks(&session.assignment)
        );
        for (index, c) in d.assignment.cases.iter().enumerate() {
            session
                .action(&d, &Action::SelectCase { index }, false)
                .unwrap();
            session.action(&d, &Action::RunTraffic, false).unwrap();
            let trial = session.assignment.last_trial.as_ref().unwrap();
            assert_eq!(trial.case_id, c.id);
            assert!(
                trial.steps.iter().all(|s| s.check.passed),
                "{}: {:?}",
                c.label,
                trial
            );
        }
    }
}
#[test]
fn example_repairs_and_traffic_never_change_assignment_or_award_its_credit() {
    for (d, _) in fixtures() {
        let mut session = Session::new(&d).unwrap();
        let before = session.assignment.clone();
        for a in &d.worked {
            session.action(&d, a, true).unwrap();
        }
        assert_eq!(session.assignment, before);
        assert!(!d.solved(&session));
        assert!(d.example.checks(&session.practice).iter().all(|c| c.passed));
        session.action(&d, &Action::RunTraffic, true).unwrap();
        assert_eq!(session.assignment, before);
        assert!(!d.solved(&session));
    }
}
#[test]
fn partial_work_wrong_pool_broad_policy_and_broken_state_cannot_remain_successful() {
    for (d, solution) in fixtures() {
        let mut session = Session::new(&d).unwrap();
        session.action(&d, &solution[0], false).unwrap();
        session.action(&d, &Action::Check, false).unwrap();
        assert!(!d.solved(&session));
        let rows = d.assignment.checks(&session.assignment);
        assert!(rows.iter().any(|c| c.passed));
        assert!(rows.iter().any(|c| !c.passed));
        repair(&mut session, &d, &solution);
        assert!(d.solved(&session));
        session
            .action(
                &d,
                &Action::Command {
                    line: "iptables -P FORWARD ACCEPT".into(),
                },
                false,
            )
            .unwrap();
        assert!(!d.solved(&session));
        session.action(&d, &Action::Check, false).unwrap();
        assert!(!d.solved(&session));
        assert!(
            d.assignment
                .checks(&session.assignment)
                .iter()
                .any(|c| !c.passed && c.label.contains("FORWARD"))
        );
    }
}
#[test]
fn any_supported_equivalent_source_translation_solution_is_accepted() {
    let (d, solution) = fixtures().remove(0);
    for alternative in [
        "iptables -t nat -A POSTROUTING -s 10.20.30.0/24 -o wan0 -j MASQUERADE",
        "iptables -t nat -A POSTROUTING -s 10.20.30.0/24 -o wan0 -p tcp -j SNAT --to-source 198.51.100.1:50000-50001",
    ] {
        let mut session = Session::new(&d).unwrap();
        repair(&mut session, &d, &solution);
        session
            .action(
                &d,
                &Action::Command {
                    line: "iptables -t nat -F POSTROUTING".into(),
                },
                false,
            )
            .unwrap();
        session
            .action(
                &d,
                &Action::Command {
                    line: alternative.into(),
                },
                false,
            )
            .unwrap();
        session.action(&d, &Action::Check, false).unwrap();
        assert!(
            d.solved(&session),
            "{alternative}: {:?}",
            d.assignment.checks(&session.assignment)
        );
    }
}
#[test]
fn hardcoded_clients_wrong_state_unions_and_unwanted_redirection_fail() {
    for (d, solution) in fixtures() {
        let mut session = Session::new(&d).unwrap();
        repair(&mut session, &d, &solution);
        let line = if d.assignment.preserve_prerouting {
            "iptables -t nat -I POSTROUTING 1 -s 10.20.30.0/24 -p tcp -j SNAT --to-source 198.51.100.1:40000"
        } else {
            "iptables -I FORWARD 1 -i wan0 -o lan0 -m conntrack --ctstate NEW,DNAT -j ACCEPT"
        };
        session
            .action(&d, &Action::Command { line: line.into() }, false)
            .unwrap();
        session.action(&d, &Action::Check, false).unwrap();
        assert!(!d.solved(&session));
        assert!(
            d.assignment
                .checks(&session.assignment)
                .iter()
                .any(|c| !c.passed)
        );
    }
    let (d, solution) = fixtures().remove(0);
    let mut session = Session::new(&d).unwrap();
    repair(&mut session, &d, &solution);
    for line in [
        "iptables -t nat -F POSTROUTING",
        "iptables -t nat -A POSTROUTING -s 10.20.30.55 -j SNAT --to-source 198.51.100.1",
        "iptables -t nat -A POSTROUTING -s 10.20.30.56 -j SNAT --to-source 198.51.100.1",
    ] {
        session
            .action(&d, &Action::Command { line: line.into() }, false)
            .unwrap();
    }
    session.action(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&session));
    assert!(
        d.assignment
            .checks(&session.assignment)
            .iter()
            .any(|c| !c.passed && c.label.contains("public source"))
    );
    let (d, solution) = fixtures().remove(1);
    let mut session = Session::new(&d).unwrap();
    repair(&mut session, &d, &solution);
    session.action(&d,&Action::Command{line:"iptables -t nat -A PREROUTING -p tcp --dport 9443 -j DNAT --to-destination 10.20.30.99:443".into()},false).unwrap();
    session.action(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&session));
    assert!(
        d.assignment
            .checks(&session.assignment)
            .iter()
            .any(|c| !c.passed && c.label.contains("destinations"))
    );
}
#[test]
fn reset_resume_and_independent_drafts_preserve_actual_traces() {
    for (d, solution) in fixtures() {
        let mut session = Session::new(&d).unwrap();
        repair(&mut session, &d, &solution);
        session.action(&d, &Action::RunTraffic, false).unwrap();
        session
            .action(
                &d,
                &Action::Input {
                    text: "iptables -t nat -S".into(),
                },
                true,
            )
            .unwrap();
        let practice = session.practice.clone();
        let bytes = serde_json::to_vec(&session).unwrap();
        let restored: Session = serde_json::from_slice(&bytes).unwrap();
        d.bound(&restored).unwrap();
        assert_eq!(restored, session);
        let assignment = session.assignment.clone();
        session.reset(&d, true).unwrap();
        assert_eq!(session.assignment, assignment);
        session.practice = practice.clone();
        session.reset(&d, false).unwrap();
        assert_eq!(session.practice, practice);
        assert_eq!(session.assignment, Scope::new(&d.assignment).unwrap());
    }
}
#[test]
fn unsupported_commands_and_readonly_boundary_errors_never_mutate_rules() {
    let (d, _) = fixtures().remove(0);
    let mut session = Session::new(&d).unwrap();
    let before = session.assignment.state.clone();
    for line in [
        "sudo iptables -F",
        "iptables -P OUTPUT DROP",
        "iptables -t nat -A OUTPUT -j DNAT --to-destination 10.20.30.42",
        "iptables -A FORWARD -m conntrack --ctstate RELATED -j ACCEPT",
        "sh -c 'touch /tmp/private'",
        "iptables -S; rm -rf private",
    ] {
        session
            .action(&d, &Action::Command { line: line.into() }, false)
            .unwrap();
        assert_eq!(session.assignment.state, before);
        assert!(!session.assignment.checked);
        assert_eq!(session.assignment.input, line);
        assert!(!session.assignment.output.is_empty());
    }
    let mut state = before.clone();
    for line in [
        "iptables -S",
        "iptables -t nat -S",
        "ip -4 route show",
        "sysctl net.ipv4.ip_forward",
    ] {
        let result = d.assignment.command(&mut state, line).unwrap();
        assert!(!result.changed);
        assert_eq!(state, before);
        assert!(!result.text.is_empty());
    }
}
#[test]
fn malformed_context_forged_traffic_and_unsupported_events_are_not_accepted() {
    let (d, _) = fixtures().remove(0);
    let mut session = Session::new(&d).unwrap();
    session.action(&d, &Action::RunTraffic, false).unwrap();
    let failing = session
        .assignment
        .last_trial
        .as_mut()
        .unwrap()
        .steps
        .iter_mut()
        .find(|s| !s.check.passed)
        .expect("The intentionally broken initial trial has a genuine failure");
    failing.check.passed = true; // Forge success for an actual failure, not an already-passing event.
    assert!(d.bound(&session).is_err());
    let mut wrong = d.clone();
    wrong.example = wrong.assignment.clone();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.assignment.policies.pop();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.assignment.cases[0].events[0].packet =
        dlms_learning_lab::linux_flow_workspace::PacketInput::Reply { event: 0 };
    assert!(wrong.validate().is_err());
    let mut state = Session::new(&d).unwrap();
    let before = state.clone();
    assert!(
        state
            .action(&d, &Action::SelectCase { index: usize::MAX }, false)
            .is_err()
    );
    assert_eq!(state, before);
    assert!(
        state
            .action(
                &d,
                &Action::Input {
                    text: "x".repeat(2049)
                },
                false
            )
            .is_err()
    );
    assert_eq!(state, before);
}
#[test]
fn a_missing_reply_or_error_cannot_count_as_an_expected_filter_refusal() {
    let (d, _) = fixtures().remove(0);
    let mut case = d.assignment.cases[0].clone();
    case.events[2].expected = Expected::Filtered;
    let mut state = d.assignment.initial.clone();
    state.filter.command("iptables -F FORWARD").unwrap();
    let trial = d.assignment.trial(&state, &case).unwrap();
    assert!(trial.steps[2].trace.is_none());
    assert!(!trial.steps[2].check.passed);
    assert!(trial.steps[2].check.detail.contains("earlier request"));
}

#[test]
fn resource_and_route_failures_do_not_count_as_correct_access_policy_refusals() {
    let (d, _) = fixtures().remove(0);
    let mut case = d.assignment.cases[0].clone();
    case.events[1].expected = Expected::NotForwarded;
    let trial = d.assignment.trial(&d.assignment.initial, &case).unwrap();
    assert_eq!(
        trial.steps[1].trace.as_ref().unwrap().gate,
        dlms_learning_lab::linux_connection_flow::Gate::MappingRefused
    );
    assert!(!trial.steps[1].check.passed);
    let mut task = d.assignment.clone();
    task.facts.routes.clear();
    case.events[0].expected = Expected::NotForwarded;
    let trial = task.trial(&task.initial, &case).unwrap();
    assert_eq!(
        trial.steps[0].trace.as_ref().unwrap().gate,
        dlms_learning_lab::linux_connection_flow::Gate::NoRoute
    );
    assert!(!trial.steps[0].check.passed);
}

#[test]
fn readonly_traffic_trials_keep_verified_configuration_but_never_award_initial_credit() {
    for (d, solution) in fixtures() {
        let mut session = Session::new(&d).unwrap();
        session.action(&d, &Action::RunTraffic, false).unwrap();
        assert!(!d.solved(&session));
        repair(&mut session, &d, &solution);
        let state = session.assignment.state.clone();
        assert!(d.solved(&session));
        session.action(&d, &Action::RunTraffic, false).unwrap();
        assert_eq!(session.assignment.state, state);
        assert!(d.solved(&session));
    }
}
#[test]
fn authoring_bounds_are_enforced_before_serialization_too() {
    let (d, _) = fixtures().remove(0);
    let mut oversized = d.clone();
    oversized.assignment.cases[0].events = vec![d.assignment.cases[0].events[0].clone(); 13];
    assert!(oversized.validate().is_err());
    let mut oversized = d.clone();
    oversized.worked = vec![Action::Check; 33];
    assert!(oversized.validate().is_err());
    let mut oversized = d;
    oversized.assignment.cases = vec![oversized.assignment.cases[0].clone(); 9];
    assert!(oversized.validate().is_err());
}

#[test]
fn state_policy_feedback_uses_live_rule_numbers_after_inactive_masks() {
    use dlms_learning_lab::{linux_connections::Class, linux_filter::Chain, linux_filter_policy};
    for (d, solution) in fixtures() {
        let mut session = Session::new(&d).unwrap();
        repair(&mut session, &d, &solution);
        assert!(d.solved(&session));
        for line in [
            "iptables -I FORWARD 1 -m conntrack --ctstate ESTABLISHED -j DROP",
            "iptables -A FORWARD -j ACCEPT",
        ] {
            session
                .action(&d, &Action::Command { line: line.into() }, false)
                .unwrap();
        }
        session.action(&d, &Action::Check, false).unwrap();
        assert!(!d.solved(&session));
        let p = d
            .assignment
            .policies
            .iter()
            .find(|p| p.class == Class::New && !p.dnat)
            .unwrap();
        let table = &session.assignment.state.filter;
        let specialized = table.specialize_context(p.class, p.dnat).unwrap();
        let counter = linux_filter_policy::compare_filter(
            &specialized,
            Chain::Forward,
            &d.assignment.facts,
            &p.domains,
            &p.allowed,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            counter.rule,
            Some(2),
            "The state-only view really has fewer rows."
        );
        let live = table
            .evaluate_context(Chain::Forward, &counter.packet, p.class, p.dnat)
            .unwrap();
        let original = table.base.forward.rules.len();
        assert_eq!(live.rule, Some(original));
        assert_eq!(live.verdict, counter.actual);
        assert!(original > 2);
        let checks = d.assignment.checks(&session.assignment);
        let failed = checks
            .iter()
            .find(|c| c.label == "FORWARD · NEW · destination unchanged")
            .unwrap();
        assert!(!failed.passed);
        assert!(
            failed
                .detail
                .contains(&format!("Inspect FORWARD rule {original},")),
            "Feedback must name the actual displayed rule: {}",
            failed.detail
        );
    }
}
