//! Source-only proposed OSPF content checks. No host protocol/packet execution.
//! Intended paths apply only after reviewed content is brought into the project.
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    ipv4_routing::Prefix,
    ospf::{Action, Endpoint, State as Simulation},
    ospf_teaching::{self, Learning},
    ospf_workspace::{self as model, Draft, Requirement},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, AtomicUsize},
};
const IDS: [&str; 4] = [
    "ccna-ospf-router-identity",
    "ccna-ospf-p2p-parameters",
    "ccna-ospf-directed-ecmp",
    "ccna-ospf-passive-handover",
];
#[derive(Deserialize)]
struct Plans {
    cases: Vec<Plan>,
}
#[derive(Deserialize)]
struct Plan {
    id: String,
    reference_actions: Vec<Action>,
    alternates: Vec<Variant>,
    negatives: Vec<Variant>,
}
#[derive(Deserialize)]
struct Variant {
    name: String,
    actions: Vec<Action>,
    explanation: String,
}
fn labs() -> Vec<Scenario> {
    // Resolve the real cross-unit prerequisite before selecting the proposed labs.
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/ipv4-static-v1.json"),
        include_str!("../scenarios/ospf-p2p-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| IDS.contains(&s.id.as_str()))
    .collect()
}
fn plans() -> Vec<Plan> {
    serde_json::from_str::<Plans>(include_str!("fixtures/ospf-content-review.json"))
        .unwrap()
        .cases
}
fn catalog_with_labs() -> Vec<Scenario> {
    let mut all = pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| !IDS.contains(&s.id.as_str()))
        .collect::<Vec<_>>();
    all.extend(labs());
    all
}
fn sim(state: &State) -> &Simulation {
    let State::Ospf { simulation } = state else {
        panic!()
    };
    simulation
}
fn sim_mut(state: &mut State) -> &mut Simulation {
    let State::Ospf { simulation } = state else {
        panic!()
    };
    simulation
}
fn goal(s: &Scenario) -> (&Simulation, &[Requirement]) {
    let Goal::Ospf {
        initial,
        requirements,
    } = &s.goal
    else {
        panic!()
    };
    (initial, requirements)
}
fn replay(s: &Scenario, state: &mut State, actions: &[Action]) {
    for action in actions {
        let input = format!("ospf {}", serde_json::to_string(action).unwrap());
        engine::execute_for_goal(state, &s.goal, &input)
            .unwrap_or_else(|e| panic!("{}: {e}", s.id));
    }
}
fn reference(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    state
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    ospf_teaching::prepare(&mut a, s.ospf_teaching.as_ref().unwrap(), true).unwrap();
    a
}
fn learning(a: &mut Attempt) -> &mut Learning {
    a.workspace
        .as_mut()
        .unwrap()
        .ospf_learning
        .as_mut()
        .unwrap()
}
fn sync_draft(a: &mut Attempt) {
    a.workspace.as_mut().unwrap().ospf_draft = Some(Draft::new(sim(&a.state)).unwrap());
}

#[test]
fn proposed_tasks_and_prerequisites_are_distinct_bounded_and_reachable() {
    let authored = labs();
    let all = catalog_with_labs();
    assert_eq!(
        authored.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert!(
        pack::bundled()
            .unwrap()
            .iter()
            .any(|s| s.id == "ccna-ipv4-branch-handover")
    );
    assert_eq!(authored[0].prerequisites, ["ccna-ipv4-branch-handover"]);
    for (i, s) in authored.iter().enumerate() {
        assert_eq!(s.unit, "ospf-p2p-foundations");
        assert_eq!(s.track, "CCNA");
        if i > 0 {
            assert_eq!(s.prerequisites, [authored[i - 1].id.clone()]);
        }
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|s| &s.id == id))
        );
        assert!(!engine::solved(&s.initial, &s.goal));
        assert_eq!(goal(s).0, sim(&s.initial));
        assert!(model::valid_requirements(goal(s).0, goal(s).1));
        assert!(
            goal(s)
                .1
                .iter()
                .any(|r| matches!(r, Requirement::Physical { .. }))
        );
        assert!(
            !goal(s)
                .1
                .iter()
                .any(|r| matches!(r, Requirement::WholeState { .. })),
            "no hidden whole-answer comparator"
        );
        assert!(engine::solved(&reference(s), &s.goal));
        let lesson = s.ospf_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.separate_from(sim(&s.initial)));
        assert!(
            lesson
                .example_actions
                .iter()
                .all(|a| !a.explanation.is_empty())
        );
        assert!(lesson.concepts.len() >= 3);
    }
    assert!(
        goal(&authored[0])
            .1
            .iter()
            .any(|r| matches!(r, Requirement::Process { .. }))
    );
    assert!(goal(&authored[2]).1.iter().any(
        |r| matches!(r,Requirement::RemoteRoute{cost:None,next_hops:Some(h),..} if h.len()==2)
    ));
    assert!(goal(&authored[3]).1.iter().any(|r|matches!(r,Requirement::Port{endpoint,expected} if endpoint.router=="Hub" && endpoint.interface=="users" && expected.enabled && expected.passive)));
}
#[test]
fn actual_references_alternates_partials_and_wrong_policies_use_fresh_outcomes() {
    let authored = labs();
    for p in plans() {
        let s = authored.iter().find(|s| s.id == p.id).unwrap();
        let mut current = s.initial.clone();
        replay(s, &mut current, &p.reference_actions);
        assert!(engine::solved(&current, &s.goal));
        assert!(p.alternates.len() >= 2);
        assert!(p.negatives.len() >= 4);
        for v in p.alternates {
            let mut current = s.initial.clone();
            replay(s, &mut current, &v.actions);
            assert!(
                engine::solved(&current, &s.goal),
                "{} / {}: {}",
                s.id,
                v.name,
                v.explanation
            );
        }
        for v in p.negatives {
            let mut current = s.initial.clone();
            replay(s, &mut current, &v.actions);
            let rows = model::checks(goal(s).0, sim(&current), goal(s).1);
            assert!(!engine::solved(&current, &s.goal), "{} / {}", s.id, v.name);
            assert!(
                rows.iter()
                    .any(|(pass, message)| !pass && !message.trim().is_empty()),
                "{} / {}: meaningful current failure",
                s.id,
                v.name
            );
        }
    }
}
#[test]
fn manual_route_costs_and_complete_hop_sets_match_independent_assignment_equations() {
    let authored = labs();
    let facts = [
        (
            "ccna-ospf-router-identity",
            "Branch",
            "192.0.2.0/24",
            12usize,
            1usize,
        ),
        (
            "ccna-ospf-p2p-parameters",
            "Office",
            "198.51.100.0/24",
            15,
            1,
        ),
        (
            "ccna-ospf-directed-ecmp",
            "Ingress",
            "198.51.100.0/24",
            19,
            2,
        ),
        ("ccna-ospf-directed-ecmp", "Storage", "192.0.2.0/24", 12, 1),
        (
            "ccna-ospf-passive-handover",
            "Site",
            "172.28.50.0/24",
            11,
            1,
        ),
        (
            "ccna-ospf-passive-handover",
            "Archive",
            "172.28.50.0/24",
            12,
            1,
        ),
        (
            "ccna-ospf-passive-handover",
            "Site",
            "198.51.100.0/24",
            17,
            1,
        ),
        (
            "ccna-ospf-passive-handover",
            "Archive",
            "192.0.2.0/24",
            23,
            1,
        ),
    ];
    for (id, router, prefix, cost, hops) in facts {
        let s = authored.iter().find(|s| s.id == id).unwrap();
        let state = reference(s);
        let snapshot = sim(&state).snapshot().unwrap();
        let route = snapshot.routers[router]
            .route(Prefix::parse(prefix).unwrap())
            .unwrap();
        assert_eq!(route.cost as usize, cost, "{id}/{router}/{prefix}");
        assert_eq!(route.next_hops.len(), hops);
        if id == "ccna-ospf-directed-ecmp" && router == "Ingress" {
            assert_eq!(
                route
                    .next_hops
                    .iter()
                    .map(|h| h.interface.as_str())
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from(["east", "west"])
            );
        }
    }
}
#[test]
fn current_success_rejects_pending_malformed_and_applied_breakage_then_accepts_repair() {
    for s in labs() {
        let mut a = prepared(&s);
        a.state = reference(&s);
        sync_draft(&mut a);
        a.workspace.as_mut().unwrap().checked = true;
        assert!(progress::current_solved(&a, &s));
        let applied = a.state.clone();
        a.workspace
            .as_mut()
            .unwrap()
            .ospf_draft
            .as_mut()
            .unwrap()
            .routers
            .values_mut()
            .next()
            .unwrap()
            .enabled = false;
        assert!(!progress::current_solved(&a, &s), "valid pending change");
        assert_eq!(a.state, applied);
        sync_draft(&mut a);
        a.workspace
            .as_mut()
            .unwrap()
            .ospf_draft
            .as_mut()
            .unwrap()
            .routers
            .values_mut()
            .next()
            .unwrap()
            .process_id
            .clear();
        assert!(
            !progress::current_solved(&a, &s),
            "malformed pending numeric text"
        );
        let mut draft = a
            .workspace
            .as_ref()
            .unwrap()
            .ospf_draft
            .as_ref()
            .unwrap()
            .clone();
        assert!(draft.apply(goal(&s).0, sim_mut(&mut a.state)).is_err());
        assert_eq!(a.state, applied);
        sync_draft(&mut a);
        let Requirement::Compatibility { local, .. } = goal(&s)
            .1
            .iter()
            .find(|r| matches!(r, Requirement::Compatibility { .. }))
            .unwrap()
        else {
            panic!()
        };
        let original = sim(&a.state).configuration().routers[&local.router].interfaces
            [&local.interface]
            .clone();
        let mut broken = original.clone();
        broken.enabled = false;
        replay(
            &s,
            &mut a.state,
            &[Action::SetPort {
                endpoint: local.clone(),
                configuration: broken,
            }],
        );
        sync_draft(&mut a);
        assert!(
            !progress::current_solved(&a, &s),
            "fresh applied behavior, not old checked flag"
        );
        replay(
            &s,
            &mut a.state,
            &[Action::SetPort {
                endpoint: local.clone(),
                configuration: original,
            }],
        );
        sync_draft(&mut a);
        assert!(progress::current_solved(&a, &s));
    }
}
#[test]
fn separate_worked_examples_fill_apply_try_and_repair_never_complete_assignment() {
    for s in labs() {
        let mut a = prepared(&s);
        let assignment = a.state.clone();
        let lesson = s.ospf_teaching.as_ref().unwrap();
        let l = learning(&mut a);
        l.view = View::Example;
        let initial = l.simulation.clone();
        l.fill_example(lesson).unwrap();
        assert_eq!(l.simulation, initial);
        assert!(!l.checked);
        assert!(!l.draft.matches(&l.simulation));
        assert!(l.apply_check(lesson).unwrap().iter().all(|r| r.0));
        let worked = l.simulation.clone();
        let mut changed = worked.clone();
        for action in &lesson.try_change.actions {
            changed.apply_against(&lesson.fixture, action).unwrap();
        }
        l.draft = Draft::new(&changed).unwrap();
        assert_eq!(
            l.apply_check(lesson)
                .unwrap()
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            lesson.try_change.expected
        );
        for action in &lesson.repair {
            changed.apply_against(&lesson.fixture, action).unwrap();
        }
        l.draft = Draft::new(&changed).unwrap();
        assert!(l.apply_check(lesson).unwrap().iter().all(|r| r.0));
        assert_eq!(l.simulation, worked);
        assert_eq!(a.state, assignment);
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn partial_policy_does_not_pass_by_suppressing_peers_or_advertising_as_active_p2p() {
    let authored = labs();
    let s = authored
        .iter()
        .find(|s| s.id == "ccna-ospf-passive-handover")
        .unwrap();
    let mut current = reference(s);
    let mut policy = sim(&current).configuration().routers["Hub"].interfaces["users"].clone();
    policy.passive = false;
    policy.network_type = dlms_learning_lab::ospf::NetworkType::PointToPoint;
    replay(
        s,
        &mut current,
        &[Action::SetPort {
            endpoint: Endpoint::new("Hub", "users"),
            configuration: policy,
        }],
    );
    let rows = model::checks(goal(s).0, sim(&current), goal(s).1);
    assert!(
        rows[2].0 && rows[3].0,
        "a route-only rubric would wrongly accept this active singleton advertisement"
    );
    assert!(
        !rows.iter().all(|r| r.0),
        "explicit required passive/Broadcast LAN policy prevents that shortcut"
    );
}
#[test]
fn profiles_preserve_all_four_unfinished_practice_and_assignment_drafts_through_restore_reset() {
    let all = catalog_with_labs();
    let authored = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 501,
        ..Default::default()
    };
    // Rich current7/frozen6 record preservation is independently exercised by ospf_workspace.
    for (i, s) in all
        .iter()
        .filter(|s| !IDS.contains(&s.id.as_str()))
        .enumerate()
    {
        p.attempts
            .insert(s.id.clone(), Attempt::new(s, i as u64 + 1));
    }
    let prior = serde_json::to_value(&p.attempts).unwrap();
    for s in &authored {
        let mut a = prepared(s);
        a.state = reference(s);
        sync_draft(&mut a);
        let lesson = s.ospf_teaching.as_ref().unwrap();
        learning(&mut a).fill_example(lesson).unwrap();
        learning(&mut a).apply_check(lesson).unwrap();
        learning(&mut a).view = View::Example;
        learning(&mut a)
            .draft
            .routers
            .values_mut()
            .next()
            .unwrap()
            .area = "unfinished practice".into();
        a.workspace
            .as_mut()
            .unwrap()
            .ospf_draft
            .as_mut()
            .unwrap()
            .routers
            .values_mut()
            .next()
            .unwrap()
            .process_id
            .clear();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let expected = serde_json::to_value(&p.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-current.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let before = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), expected);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let export = dir.path().join("synthetic-export.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(candidate.rechecked, 0);
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts).unwrap(),
        expected
    );
    let target = dir.path().join("synthetic-target.json");
    let _lock = progress::lock_profile(&target).unwrap();
    progress::save(
        &target,
        &Progress {
            schema_version: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let previous = std::fs::read(&target).unwrap();
    let restored = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        previous
    );
    let mut loaded = progress::load(&target, &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), expected);
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(loaded.clock, p.clock);
    loaded
        .attempts
        .insert(IDS[0].into(), prepared(&authored[0]));
    progress::save_with_catalog(&target, &loaded, &all).unwrap();
    let reset = progress::load(&target, &all).unwrap();
    for s in &all {
        if !IDS.contains(&s.id.as_str()) {
            assert_eq!(
                serde_json::to_value(&reset.attempts[&s.id]).unwrap(),
                prior[&s.id]
            );
        } else if s.id != IDS[0] {
            assert_eq!(
                serde_json::to_value(&reset.attempts[&s.id]).unwrap(),
                expected[&s.id]
            );
        }
    }
    assert_eq!(reset.attempts[IDS[0]].state, authored[0].initial);
    assert_eq!(reset.favorites, p.favorites);
}
#[test]
fn learner_text_explains_ui_work_without_exposing_pack_only_json_as_a_command() {
    for s in labs() {
        let mut visible = vec![s.objective, s.summary, s.explanation];
        visible.extend(s.hints);
        let lesson = s.ospf_teaching.unwrap();
        visible.push(lesson.example_description);
        visible.push(lesson.try_change.description);
        for a in lesson.example_actions {
            visible.push(a.explanation);
        }
        for text in visible {
            assert!(!text.contains("ospf {"));
            assert!(!text.contains("\"kind\""));
        }
    }
}
