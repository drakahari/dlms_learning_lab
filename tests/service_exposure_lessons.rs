//! Two registered authored investigations; preservation, alternatives and negative grading.
use dlms_learning_lab::{
    engine::{self, State},
    pack, service_exposure_source as binding,
    service_exposure_workspace::{Action, Event},
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/service-exposure-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
}
fn act(state: &mut State, s: &pack::Scenario, a: Action, practice: bool) {
    let text = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    );
    engine::execute_for_goal(state, &s.goal, &text).unwrap();
}
fn command(state: &mut State, s: &pack::Scenario, text: &str) {
    act(
        state,
        s,
        Action::Event {
            event: Event::Admin { input: text.into() },
        },
        false,
    );
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{}: {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
#[test]
fn both_authored_lessons_and_practice_methods_load_with_real_prerequisites_and_are_reachable() {
    let new = sources();
    assert_eq!(new.len(), 2);
    assert!(
        pack::load(include_str!(
            "../scenarios/service-exposure-lessons-v1.json"
        ))
        .unwrap_err()
        .contains("invalid prerequisites")
    );
    let old = pack::current().unwrap();
    let mut all = new.clone();
    let mut at = 0;
    while at < all.len() {
        for id in all[at].prerequisites.clone() {
            if !all.iter().any(|s| s.id == id) {
                all.push(
                    old.iter()
                        .find(|s| s.id == id)
                        .expect("existing real prerequisite")
                        .clone(),
                );
            }
        }
        at += 1;
    }
    // Preserve the per-fragment size bound; the catalog loader validates the same
    // complete prerequisite closure over separately bounded declarative packs.
    let owned = all
        .into_iter()
        .map(|s| {
            serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![s],
            })
            .unwrap()
        })
        .collect::<Vec<_>>();
    let texts = owned.iter().map(String::as_str).collect::<Vec<_>>();
    pack::load_catalog(&texts).unwrap();
    for s in new {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let State::ServiceExposure { simulation } = &state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        for a in &d.worked {
            act(&mut state, &s, a.clone(), true);
        }
        let State::ServiceExposure { simulation } = &state else {
            panic!()
        };
        assert_eq!(simulation.assignment, original);
        assert!(simulation.example.solved(&d.example));
        assert!(!engine::solved(&state, &s.goal));
        solve(&s);
    }
}
#[test]
fn assignment_policy_accepts_an_earlier_correct_admin_allow_and_split_public_prefixes() {
    let s = sources().remove(0);
    let mut state = s.initial.clone();
    command(
        &mut state,
        &s,
        "iptables -I INPUT -s 10.20.0.0/24 -p tcp --dport 22 -j ACCEPT",
    );
    for net in ["10.30.0.0/25", "10.30.0.128/25"] {
        for port in [22, 9000] {
            command(
                &mut state,
                &s,
                &format!("iptables -A INPUT -s {net} -p tcp --dport {port} -j DROP"),
            );
        }
    }
    act(
        &mut state,
        &s,
        Action::Event {
            event: Event::Check,
        },
        false,
    );
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn retirement_accepts_supported_ending_signals_and_extra_filtering_but_not_pausing_or_hiding() {
    let s = sources().remove(1);
    for signal in ["TERM", "INT", "KILL"] {
        let mut state = s.initial.clone();
        command(&mut state, &s, &format!("kill -{signal} 410"));
        command(
            &mut state,
            &s,
            "iptables -A INPUT -p tcp --dport 9000 -j DROP",
        );
        act(
            &mut state,
            &s,
            Action::Event {
                event: Event::Check,
            },
            false,
        );
        assert!(engine::solved(&state, &s.goal), "{signal}");
    }
    for commands in [
        vec!["kill -STOP 410"],
        vec!["iptables -A INPUT -p tcp --dport 9000 -j DROP"],
        vec!["kill -TERM 411"],
    ] {
        let mut state = s.initial.clone();
        for c in commands {
            command(&mut state, &s, c);
        }
        act(
            &mut state,
            &s,
            Action::Event {
                event: Event::Check,
            },
            false,
        );
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn wrong_blanket_policy_partial_repairs_and_broken_current_state_cannot_pass() {
    for s in sources() {
        let mut partial = s.initial.clone();
        act(
            &mut partial,
            &s,
            Action::Event {
                event: Event::Check,
            },
            false,
        );
        assert!(!engine::solved(&partial, &s.goal));
        command(&mut partial, &s, "iptables -P INPUT DROP");
        act(
            &mut partial,
            &s,
            Action::Event {
                event: Event::Check,
            },
            false,
        );
        assert!(!engine::solved(&partial, &s.goal));
        let mut state = solve(&s);
        command(&mut state, &s, "kill -TERM 500");
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn malformed_authored_sources_and_forged_work_are_refused_without_losing_the_original() {
    for mut s in sources() {
        let good = s.clone();
        let mut state = solve(&s);
        let State::ServiceExposure { simulation } = &mut state else {
            panic!()
        };
        simulation.assignment.scan_output = "Forged successful scan".into();
        assert!(!engine::solved(&state, &s.goal));
        s.guide[0].commands[0].command = "nmap --script all real.example".into();
        assert!(binding::source(&s).is_err());
        assert!(binding::source(&good).is_ok());
    }
}

#[test]
fn registration_preserves_all497_labs199_units_and_every_aggregation_value() {
    let before = pack::accepted497().unwrap();
    let now = pack::accepted499().unwrap();
    assert_eq!((before.len(), now.len()), (497, 499));
    for old in &before {
        let current = now.iter().find(|s| s.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(current).unwrap()
        );
    }
    let old = dlms_learning_lab::course::accepted497(&before).unwrap();
    let current = dlms_learning_lab::course::accepted499(&now).unwrap();
    assert_eq!((old.len(), current.len()), (199, 200));
    for u in &old {
        assert_eq!(
            serde_json::to_value(u).unwrap(),
            serde_json::to_value(current.iter().find(|v| v.id == u.id).unwrap()).unwrap()
        );
    }
    let unit = current
        .iter()
        .find(|u| u.id == "linux-service-exposure")
        .unwrap();
    assert_eq!(
        unit.labs,
        [
            "lpic2-exposure-client-policy",
            "lpic2-exposure-listener-retirement"
        ]
    );
    for predecessor in ["linux-listener-audit", "linux-router-policies"] {
        assert!(
            current.iter().position(|u| u.id == predecessor).unwrap()
                < current.iter().position(|u| u.id == unit.id).unwrap()
        );
    }
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v60.json")).unwrap();
    let new: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v61.json")).unwrap();
    let a = old["scenarios"].as_array().unwrap();
    let b = new["scenarios"].as_array().unwrap();
    assert_eq!((a.len(), b.len()), (124, 126));
    assert_eq!(a.as_slice(), &b[..124]);
}
