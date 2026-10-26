//! Distinct steady-state assignments; no protocol, packet or real switch execution.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    spanning_tree::{Action, PortConfig},
    spanning_tree_teaching::{self as teaching, Learning},
    spanning_tree_workspace as work, workspace,
};
fn labs() -> Vec<Scenario> {
    pack::load(include_str!("../scenarios/spanning-tree-v1.json")).unwrap()
}
fn apply(s: &Scenario, state: &mut State, action: &Action) {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!("spanning-tree {}", serde_json::to_string(action).unwrap()),
    )
    .unwrap();
}
fn priority(bridge: &str, priority: u16) -> Action {
    Action::BridgePriority {
        bridge: bridge.into(),
        priority,
    }
}
fn port(bridge: &str, port: &str, cost: u32, priority: u8) -> Action {
    Action::Port {
        bridge: bridge.into(),
        port: port.into(),
        settings: PortConfig {
            priority,
            cost,
            up: true,
        },
    }
}
fn rows(s: &Scenario, state: &State) -> Vec<(bool, String)> {
    let State::SpanningTree { simulation } = state else {
        panic!()
    };
    let Goal::SpanningTree {
        initial,
        requirements,
    } = &s.goal
    else {
        panic!()
    };
    work::checks(initial, simulation, requirements)
}
fn finished(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for command in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{} {:?}",
        s.id,
        rows(s, &state)
    );
    state
}

#[test]
fn distinct_root_local_cost_parallel_and_handover_assignments_have_reachable_references() {
    let labs = labs();
    assert_eq!(labs.len(), 4);
    for (i, s) in labs.iter().enumerate() {
        assert!(
            !engine::solved(&s.initial, &s.goal),
            "{} initially solved",
            s.id
        );
        let checks = rows(s, &s.initial);
        assert!(checks.iter().any(|r| r.0) && checks.iter().any(|r| !r.0));
        assert_eq!(
            s.prerequisites,
            if i == 0 {
                vec![]
            } else {
                vec![labs[i - 1].id.clone()]
            }
        );
        assert!(
            s.guide.iter().all(|g| g.commands.is_empty()),
            "internal actions must not be taught as switch commands"
        );
        finished(s);
    }
}

#[test]
fn alternate_legal_priorities_costs_and_repair_order_are_accepted_by_result() {
    let alternatives = [
        vec![priority("core", 4096)],
        vec![port("west", "core", 9, 128)],
        vec![port("west", "a", 10, 128)],
        vec![
            port("leaf", "lower", 100, 128),
            priority("leaf", 32768),
            priority("core", 4096),
        ],
    ];
    for (s, actions) in labs().iter().zip(alternatives) {
        let mut state = s.initial.clone();
        for a in actions {
            apply(s, &mut state, &a);
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            rows(s, &state)
        );
    }
}

#[test]
fn sender_and_receiver_priority_are_not_confused_and_a_root_only_capstone_fix_is_partial() {
    let labs = labs();
    let s = &labs[2];
    let mut state = s.initial.clone();
    apply(s, &mut state, &port("west", "b", 4, 0));
    let r = rows(s, &state);
    assert!(r[2].0, "root is already correct");
    assert!(
        !r[3].0,
        "receiver port priority cannot beat the earlier sender-port comparison"
    );
    assert!(!engine::solved(&state, &s.goal));
    let s = &labs[3];
    let mut state = s.initial.clone();
    apply(s, &mut state, &priority("core", 0));
    let r = rows(s, &state);
    assert!(r[2].0, "root was fixed");
    assert!(!r[3].0 && !r[5].0, "local path fault remains");
    assert!(!engine::solved(&state, &s.goal));
}

#[test]
fn physical_isolation_never_passes_and_current_success_breaks_then_repairs() {
    let breaks = [
        priority("core", 61440),
        port("west", "core", 1, 128),
        port("core", "b", 4, 128),
        port("leaf", "upper", 40, 128),
    ];
    for (s, break_action) in labs().iter().zip(breaks) {
        let good = finished(s);
        let State::SpanningTree { simulation } = &good else {
            panic!()
        };
        for link in simulation.topology().links.keys() {
            let mut isolated = good.clone();
            apply(
                s,
                &mut isolated,
                &Action::LinkUp {
                    link: link.clone(),
                    up: false,
                },
            );
            assert!(!rows(s, &isolated)[0].0);
            assert!(!engine::solved(&isolated, &s.goal));
        }
        let mut state = good;
        apply(s, &mut state, &break_action);
        assert!(!engine::solved(&state, &s.goal), "{} stale success", s.id);
        for command in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
    }
}

#[test]
fn separate_graphs_fill_only_then_apply_and_exact_experiment_expectations_hold() {
    for s in labs() {
        let lesson = s.spanning_tree_teaching.as_ref().unwrap();
        let mut p = Learning::new(lesson).unwrap();
        let before = p.simulation.clone();
        p.fill_example(lesson).unwrap();
        assert_eq!(p.simulation, before, "Fill must not execute");
        p.apply(lesson).unwrap();
        assert!(p.checks(lesson).iter().all(|r| r.0));
        for a in &lesson.try_change.actions {
            p.execute(
                lesson,
                &format!("spanning-tree {}", serde_json::to_string(a).unwrap()),
            )
            .unwrap();
        }
        let actual = p.checks(lesson);
        assert!(actual[0].0, "experiment actions leave no unapplied draft");
        assert_eq!(
            actual[1..].iter().map(|r| r.0).collect::<Vec<_>>(),
            lesson.try_change.expected,
            "{} authored misconception",
            s.id
        );
        for a in &lesson.repair {
            p.execute(
                lesson,
                &format!("spanning-tree {}", serde_json::to_string(a).unwrap()),
            )
            .unwrap();
        }
        assert!(p.checks(lesson).iter().all(|r| r.0));
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}

#[test]
fn unfinished_drafts_resume_honestly_and_selected_reset_preserves_other_attempts() {
    let labs = labs();
    let catalog = pack::bundled().unwrap();
    assert!(labs.iter().all(|s| catalog.iter().any(|c| c.id == s.id)));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.spanning_tree_teaching.as_ref().unwrap()).unwrap();
        a.state = finished(s);
        let State::SpanningTree { simulation } = &a.state else {
            panic!()
        };
        a.workspace.as_mut().unwrap().spanning_tree_draft =
            Some(work::Draft::new(simulation).unwrap());
        assert!(progress::current_solved(&a, s));
        a.workspace
            .as_mut()
            .unwrap()
            .spanning_tree_draft
            .as_mut()
            .unwrap()
            .bridges
            .get_mut("core")
            .unwrap()
            .priority = "not a number".into();
        assert!(!progress::current_solved(&a, s));
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .spanning_tree_learning
            .as_mut()
            .unwrap();
        learning
            .draft
            .bridges
            .get_mut("sample-core")
            .unwrap()
            .priority = "different unfinished practice".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut q = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let other = serde_json::to_value(&q.attempts[&labs[1].id]).unwrap();
    q.attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 9));
    assert_eq!(
        serde_json::to_value(&q.attempts[&labs[1].id]).unwrap(),
        other
    );
    assert_eq!(q.favorites, p.favorites);
}
