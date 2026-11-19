use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    smtp_source as binding,
    smtp_workspace::{Action, Session},
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/smtp-routing-lessons-v1.json")).unwrap()
}
fn solved(s: &pack::Scenario) -> Session {
    let d = binding::source(s).unwrap();
    let mut state = Session::new(d).unwrap();
    for a in &d.reference {
        state.action(d, a, false).unwrap();
    }
    assert!(d.current(&state), "{}", s.id);
    state
}
#[test]
fn all_three_distinct_assignments_and_separate_taught_methods_reach_every_required_outcome() {
    let labs = labs();
    assert_eq!(labs.len(), 3);
    let fields = labs
        .iter()
        .map(|s| {
            binding::source(s)
                .unwrap()
                .assignment
                .editable
                .values()
                .next()
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .as_str()
        })
        .collect::<Vec<_>>();
    assert_eq!(fields, vec!["mydestination", "mynetworks", "relayhost"]);
    for s in labs {
        let d = binding::source(&s).unwrap();
        assert_eq!(s.steps.len(), 3);
        assert_eq!(s.guide.len(), 3);
        assert!(d.assignment.probes.len() >= 9);
        assert_eq!(d.assignment.queued.len(), 2);
        let mut state = Session::new(d).unwrap();
        assert!(!d.current(&state));
        let independent = state.assignment.clone();
        for a in &d.example_steps {
            state.action(d, a, true).unwrap();
        }
        assert_eq!(state.assignment, independent);
        assert!(!d.current(&state));
        assert!(state.practice.checks(&d.example).iter().all(|r| r.passed));
        let mut engine_state = State::SmtpRouting {
            simulation: Box::new(state),
        };
        assert!(!engine::solved(&engine_state, &s.goal));
        for line in &s.solution {
            engine::execute_for_goal(&mut engine_state, &s.goal, line).unwrap();
        }
        assert!(engine::solved(&engine_state, &s.goal));
        let completed = solved(&s);
        assert!(
            completed
                .assignment
                .checks(&d.assignment)
                .iter()
                .all(|r| r.passed)
        );
        assert_eq!(
            d.assignment.facts.samples,
            independent
                .runtime
                .messages
                .keys()
                .map(|k| (k.clone(), d.assignment.facts.samples[k].clone()))
                .collect()
        );
    }
}
#[test]
fn alternate_domain_order_network_decomposition_and_literal_relay_endpoint_are_accepted() {
    for s in labs() {
        let d = binding::source(&s).unwrap();
        let mut state = Session::new(d).unwrap();
        for action in &d.reference {
            let mut a = action.clone();
            if let Action::Edit { text, .. } = &mut a {
                if s.id.contains("destination") {
                    *text = text.replace(
                        "mydestination = edge.dispatch.test, localhost",
                        "mydestination = localhost, EDGE.DISPATCH.TEST",
                    );
                } else if s.id.contains("boundary") {
                    *text = text.replace(
                        "192.0.2.0/24, 10.44.8.0/24",
                        "10.44.8.128/25, 192.0.2.128/25, 10.44.8.0/25, 192.0.2.0/25",
                    );
                } else {
                    *text = text.replace("[mailbox.dispatch.test]", "[192.0.2.30]:25");
                }
            }
            state.action(d, &a, false).unwrap();
        }
        assert!(
            d.current(&state),
            "{} {:?}",
            s.id,
            state.assignment.checks(&d.assignment)
        );
    }
}
#[test]
fn partial_edits_no_reload_and_no_retry_fail_and_breaking_a_successful_state_clears_current_credit()
{
    for s in labs() {
        let d = binding::source(&s).unwrap();
        let mut state = Session::new(d).unwrap();
        for (at, a) in d.reference.iter().enumerate() {
            state.action(d, a, false).unwrap();
            if at < 3 {
                state.action(d, &Action::Check, false).unwrap();
                assert!(!d.current(&state), "{} partial {at}", s.id);
            }
        }
        assert!(d.current(&state));
        let node = d.assignment.editable.keys().next().unwrap();
        let wrong = d.assignment.facts.initial_files[node].clone();
        state
            .action(
                d,
                &Action::Edit {
                    node: node.clone(),
                    text: wrong,
                },
                false,
            )
            .unwrap();
        assert!(!state.assignment.checked);
        assert!(!d.current(&state));
        state
            .action(d, &Action::Save { node: node.clone() }, false)
            .unwrap();
        state.assignment.checked = true;
        assert!(!d.current(&state));
        state
            .action(d, &Action::Reload { node: node.clone() }, false)
            .unwrap();
        state.assignment.checked = true;
        assert!(!d.current(&state));
    }
}
#[test]
fn good_negative_cases_cannot_hide_failed_delivery_and_broad_trust_fails_multiple_outsider_cases() {
    let labs = labs();
    for s in &labs {
        let d = binding::source(s).unwrap();
        let state = Session::new(d).unwrap();
        let rows = state.assignment.checks(&d.assignment);
        assert!(rows.iter().any(|r| r.passed));
        assert!(rows.iter().any(|r| !r.passed));
        assert!(!d.current(&state));
    }
    let s = &labs[1];
    let d = binding::source(s).unwrap();
    let initial = Session::new(d).unwrap();
    assert_eq!(
        initial
            .assignment
            .checks(&d.assignment)
            .iter()
            .filter(|r| r.label.starts_with("Reject external relay") && !r.passed)
            .count(),
        3
    );
    let mut state = solved(s);
    state
        .assignment
        .runtime
        .messages
        .values_mut()
        .for_each(|m| {
            m.status = dlms_learning_lab::smtp_queue::Status::Failed {
                reason: "Empty is not delivered".into(),
            }
        });
    assert!(
        state
            .assignment
            .runtime
            .pending("edge", &d.assignment.facts)
            .unwrap()
            .is_empty()
    );
    assert!(!d.current(&state));
}
#[test]
fn ordinary_progress_retains_two_partial_mail_drafts_and_reset_does_not_touch_other_labs() {
    let labs = labs();
    let s = &labs[2];
    let d = binding::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    let State::SmtpRouting { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.input = "postqueue -".into();
    simulation.practice.input = "postfix re".into();
    simulation
        .action(
            d,
            &Action::Edit {
                node: "transit".into(),
                text: "unfinished assignment".into(),
            },
            false,
        )
        .unwrap();
    simulation
        .action(
            d,
            &Action::Edit {
                node: "guide-transit".into(),
                text: "unfinished guide".into(),
            },
            true,
        )
        .unwrap();
    let expected = simulation.clone();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 2));
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    let other = serde_json::to_value(&restored.attempts[&labs[0].id]).unwrap();
    let State::SmtpRouting { simulation } = &mut restored.attempts.get_mut(&s.id).unwrap().state
    else {
        panic!()
    };
    assert_eq!(*simulation, expected);
    let practice = simulation.practice.clone();
    simulation.reset(d, false).unwrap();
    assert_eq!(simulation.practice, practice);
    assert!(!d.current(simulation));
    assert!(restored.favorites.contains(&s.id));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&labs[0].id]).unwrap(),
        other
    );
}
