//! Authored power plans: real outcomes, prerequisite closure and saved work.
use dlms_learning_lab::{
    engine::{self, State},
    pack,
    poe::Plan,
    poe_source as binding,
    poe_workspace::Action,
    progress::{self, Attempt, Progress},
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/poe-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn act(state: &mut State, s: &pack::Scenario, action: Action, practice: bool) {
    let text = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":action})
    );
    engine::execute_for_goal(state, &s.goal, &text).unwrap();
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
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
fn both_authored_tasks_and_worked_plans_are_reachable_without_credit_from_practice() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        assert!(!engine::solved(&state, &s.goal));
        let State::Poe { simulation } = &state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        act(
            &mut state,
            &s,
            Action::Edit {
                plan: d.example.reference.clone(),
            },
            true,
        );
        act(&mut state, &s, Action::Check, true);
        let State::Poe { simulation } = &state else {
            panic!()
        };
        assert!(simulation.example.solved(&d.example));
        assert_eq!(simulation.assignment, original);
        assert!(!engine::solved(&state, &s.goal));
        solve(&s);
        assert!(
            s.guide
                .iter()
                .flat_map(|g| &g.commands)
                .all(|c| !c.command.starts_with(binding::PREFIX))
        );
    }
}
#[test]
fn different_supported_port_choices_pass_both_assignments() {
    let all = sources();
    let alternatives = [
        Plan {
            attachments: vec![Some(3), Some(0), Some(1)],
            enabled: vec![true; 4],
        },
        Plan {
            attachments: vec![Some(0), Some(2), None],
            enabled: vec![true, true, true, false],
        },
    ];
    for (s, alt) in all.iter().zip(alternatives) {
        let d = binding::source(s).unwrap();
        assert_ne!(alt, d.assignment.reference);
        let mut state = s.initial.clone();
        act(&mut state, s, Action::Edit { plan: alt }, false);
        assert!(!engine::solved(&state, &s.goal));
        act(&mut state, s, Action::Check, false);
        assert!(
            engine::solved(&state, &s.goal),
            "{:?}",
            engine::checks(&state, &s.goal)
        );
    }
}
#[test]
fn partial_repairs_budget_starvation_and_success_breakage_fail_helpfully() {
    for s in sources() {
        let mut state = s.initial.clone();
        act(&mut state, &s, Action::Check, false);
        let checks = engine::checks(&state, &s.goal);
        assert!(checks.iter().any(|r| r.0) && checks.iter().any(|r| !r.0));
        assert!(!engine::solved(&state, &s.goal));
        let mut good = solve(&s);
        let State::Poe { simulation } = &good else {
            panic!()
        };
        let mut broken = simulation.assignment.plan.clone();
        let port = broken.attachments[0].unwrap();
        broken.enabled[port] = false;
        act(&mut good, &s, Action::Edit { plan: broken }, false);
        assert!(!engine::solved(&good, &s.goal));
    }
    let s = sources().remove(1);
    let mut state = s.initial.clone();
    let mut p = Plan {
        attachments: vec![Some(3), Some(1), Some(0)],
        enabled: vec![true; 4],
    };
    act(&mut state, &s, Action::Edit { plan: p.clone() }, false);
    act(&mut state, &s, Action::Check, false);
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("shared source budget"))
    );
    p.attachments = vec![Some(0), Some(1), Some(3)];
    act(&mut state, &s, Action::Edit { plan: p }, false);
    act(&mut state, &s, Action::Check, false);
    assert!(!engine::solved(&state, &s.goal));
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("not retirement"))
    );
}
#[test]
fn source_context_progress_and_both_unfinished_plans_roundtrip_without_inventing_completion() {
    let s = sources().remove(0);
    let mut a = Attempt::new(&s, 1);
    let d = binding::source(&s).unwrap();
    act(
        &mut a.state,
        &s,
        Action::Edit {
            plan: d.example.reference.clone(),
        },
        true,
    );
    a.hints = 1;
    a.challenge = true;
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let bytes = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&bytes, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    let mut bad = s.clone();
    bad.version = 2;
    assert!(binding::context(&restored.attempts[&s.id], &bad).is_err());
    let mut bad = p;
    let State::Poe { simulation } = &mut bad.attempts.get_mut(&s.id).unwrap().state else {
        panic!()
    };
    simulation.assignment.source_task = "forged".into();
    assert!(
        progress::decode(&serde_json::to_vec(&bad).unwrap(), std::slice::from_ref(&s)).is_err()
    );
}
#[test]
fn authored_prerequisites_are_real_and_loaded_through_bounded_fragments() {
    let added = sources();
    assert_eq!(added.len(), 2);
    assert!(
        pack::load(include_str!("../scenarios/poe-lessons-v1.json"))
            .unwrap_err()
            .contains("invalid prerequisites")
    );
    let old = pack::current().unwrap();
    let mut closure = added.clone();
    let mut i = 0;
    while i < closure.len() {
        for id in closure[i].prerequisites.clone() {
            if !closure.iter().any(|s| s.id == id) {
                closure.push(
                    old.iter()
                        .find(|s| s.id == id)
                        .expect("real existing prerequisite")
                        .clone(),
                );
            }
        }
        i += 1;
    }
    let owned = closure
        .into_iter()
        .map(|s| {
            serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![s],
            })
            .unwrap()
        })
        .collect::<Vec<_>>();
    pack::load_catalog(&owned.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
}
#[test]
fn additive_aggregate_preserves_all_four_prior_records_and_the_pack_byte_ceiling() {
    let old: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/security-detection-lessons-v2.json"
    ))
    .unwrap();
    let new: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/curriculum-continuation-v1.json")).unwrap();
    let added: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/poe-lessons-v1.json")).unwrap();
    assert_eq!(old["scenarios"].as_array().unwrap().len(), 4);
    assert_eq!(
        &new["scenarios"].as_array().unwrap()[..4],
        old["scenarios"].as_array().unwrap()
    );
    assert_eq!(
        &new["scenarios"].as_array().unwrap()[4..],
        added["scenarios"].as_array().unwrap()
    );
    assert!(
        include_bytes!("../scenarios/curriculum-continuation-v1.json").len() < pack::MAX_PACK_BYTES
    );
}

#[test]
fn registered_catalog_retains_every507_prior_scenario_and204_units_in_order() {
    let old = pack::accepted507().unwrap();
    let current = pack::accepted509().unwrap();
    assert_eq!(old.len(), 507);
    assert_eq!(current.len(), 509);
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..507]).unwrap()
    );
    assert_eq!(
        current[507..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        vec!["ccna-poe-compatible-ports", "ccna-poe-budget-recovery"]
    );
    for s in &current[507..] {
        binding::source(s).unwrap();
    }
    let before = dlms_learning_lab::course::accepted507(&old).unwrap();
    let units = dlms_learning_lab::course::accepted509(&current).unwrap();
    assert_eq!(before.len(), 204);
    assert_eq!(units.len(), 205);
    let retained = units
        .iter()
        .filter(|u| u.id != "ccna-poe-power-plans")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ccna-poe-power-plans")
        .unwrap();
    for prereq in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == prereq).unwrap() < at);
    }
}
