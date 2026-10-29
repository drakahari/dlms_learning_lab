//! Private backend-only fixture, prospective tests; no authored/native acceptance.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    shell,
    spanning_tree::{self as core, Action, Endpoint, State as Simulation},
    spanning_tree_teaching::{self as teaching, Learning},
    spanning_tree_workspace::{self as model, Availability, Draft, Requirement, StableRole},
    workspace,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn scenario() -> Scenario {
    static FIXTURE: std::sync::OnceLock<Scenario> = std::sync::OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            pack::load(include_str!(
                "fixtures/spanning-tree-workspace-preview.json"
            ))
            .unwrap()
            .remove(0)
        })
        .clone()
}
fn catalog() -> Vec<Scenario> {
    static CATALOG: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| pack::bundled().unwrap()).clone()
}

fn sim(s: &State) -> &Simulation {
    let State::SpanningTree { simulation } = s else {
        panic!()
    };
    simulation
}
fn sim_mut(s: &mut State) -> &mut Simulation {
    let State::SpanningTree { simulation } = s else {
        panic!()
    };
    simulation
}
fn goal(s: &Scenario) -> (&Simulation, &[Requirement]) {
    let Goal::SpanningTree {
        initial,
        requirements,
    } = &s.goal
    else {
        panic!()
    };
    (initial, requirements)
}
fn endpoint(b: &str, p: &str) -> Endpoint {
    Endpoint {
        bridge: b.into(),
        port: p.into(),
    }
}
fn command(a: &Action) -> String {
    format!("{}{}", model::PREFIX, serde_json::to_string(a).unwrap())
}
fn priority(b: &str, n: u16) -> Action {
    Action::BridgePriority {
        bridge: b.into(),
        priority: n,
    }
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.spanning_tree_teaching.as_ref().unwrap()).unwrap();
    a
}
fn sync(a: &mut Attempt) {
    a.workspace.as_mut().unwrap().spanning_tree_draft = Some(Draft::new(sim(&a.state)).unwrap());
}
fn repaired(s: &Scenario) -> Attempt {
    let mut a = prepared(s);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    sync(&mut a);
    assert!(progress::current_solved(&a, s));
    a
}
fn learning(a: &mut Attempt) -> &mut Learning {
    a.workspace
        .as_mut()
        .unwrap()
        .spanning_tree_learning
        .as_mut()
        .unwrap()
}
#[test]
fn pack_reference_and_distinct_valid_priority_or_cost_strategies_reach_current_tree() {
    let s = scenario();
    assert!(!catalog().iter().any(|x| x.id == s.id));
    assert!(!engine::solved(&s.initial, &s.goal));
    for strategy in [0, 1, 2] {
        let mut a = prepared(&s);
        let actions = match strategy {
            0 => vec![priority("core", 0)],
            1 => vec![priority("west", 8192), priority("core", 4096)],
            _ => {
                let mut p = sim(&a.state).configuration().bridges["west"].ports["core"];
                p.cost = 3;
                let mut q = sim(&a.state).configuration().bridges["east"].ports["core"];
                q.cost = 5;
                vec![
                    Action::Port {
                        bridge: "west".into(),
                        port: "core".into(),
                        settings: p,
                    },
                    priority("core", 0),
                    Action::Port {
                        bridge: "east".into(),
                        port: "core".into(),
                        settings: q,
                    },
                ]
            }
        };
        for action in actions {
            engine::execute_for_goal(&mut a.state, &s.goal, &command(&action)).unwrap();
        }
        sync(&mut a);
        assert!(progress::current_solved(&a, &s));
        let snap = sim(&a.state).snapshot().unwrap();
        assert_eq!(
            snap.forwarding_path("west", "east").unwrap(),
            Some(vec!["core-west".into(), "core-east".into()])
        );
    }
}
#[test]
fn disconnection_offline_port_and_link_shortcuts_fail_specific_physical_and_positive_rows() {
    let s = scenario();
    let (initial, rs) = goal(&s);
    let good = repaired(&s);
    for action in [
        Action::BridgeOnline {
            bridge: "core".into(),
            online: false,
        },
        Action::LinkUp {
            link: "core-west".into(),
            up: false,
        },
        Action::Port {
            bridge: "west".into(),
            port: "core".into(),
            settings: core::PortConfig {
                priority: 128,
                cost: 4,
                up: false,
            },
        },
    ] {
        let mut state = sim(&good.state).clone();
        state.apply(&action).unwrap();
        let rows = model::checks(initial, &state, rs);
        assert!(!rows[0].0, "physical guard must fail itself");
        assert!(rows.iter().any(|r| !r.0));
        let mut a = good.clone();
        a.state = State::SpanningTree { simulation: state };
        sync(&mut a);
        assert!(!progress::current_solved(&a, &s));
    }
    let mut state = sim(&good.state).clone();
    for l in ["core-west", "core-east"] {
        state
            .apply(&Action::LinkUp {
                link: l.into(),
                up: false,
            })
            .unwrap();
    }
    let rows = model::checks(initial, &state, rs);
    assert!(
        !rows[1].0,
        "positive connected group cannot accept an isolated root"
    );
}
#[test]
fn role_rootport_rootcost_and_path_checks_use_current_snapshot_not_success_history() {
    let s = scenario();
    let (initial, rs) = goal(&s);
    let mut a = repaired(&s);
    engine::execute_for_goal(&mut a.state, &s.goal, &command(&priority("core", 61440))).unwrap();
    sync(&mut a);
    let rows = progress::current_spanning_tree_checks(&a, &s);
    assert!(rows[1].0 && rows[2].0);
    assert!(!rows[3].0 && !rows[4].0 && !rows[5].0 && !rows[6].0);
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, &command(&priority("core", 0))).unwrap();
    sync(&mut a);
    assert!(progress::current_solved(&a, &s));
    let mut with_cost = rs.to_vec();
    with_cost.push(Requirement::RootCost {
        bridge: "east".into(),
        value: 4,
    });
    assert!(model::valid_requirements(initial, &with_cost));
    assert!(
        model::checks(initial, sim(&a.state), &with_cost)
            .iter()
            .all(|r| r.0)
    );
    let mut port = sim(&a.state).configuration().bridges["east"].ports["core"];
    port.cost = 5;
    sim_mut(&mut a.state)
        .apply(&Action::Port {
            bridge: "east".into(),
            port: "core".into(),
            settings: port,
        })
        .unwrap();
    let rows = model::checks(initial, sim(&a.state), &with_cost);
    assert!(rows[..rs.len()].iter().all(|r| r.0));
    assert!(!rows.last().unwrap().0, "cost row itself must fail");
}
#[test]
fn exact_preservation_is_explicit_and_keeps_present_and_absent_physical_settings() {
    let s = scenario();
    let (initial, rs) = goal(&s);
    let good = repaired(&s);
    let mut r = rs.to_vec();
    r.push(Requirement::PortSettings {
        endpoint: endpoint("core", "west"),
        expected: sim(&good.state).configuration().bridges["core"].ports["west"],
    });
    let mut changed = sim(&good.state).clone();
    let mut p = changed.configuration().bridges["core"].ports["west"];
    p.priority = 64;
    changed
        .apply(&Action::Port {
            bridge: "core".into(),
            port: "west".into(),
            settings: p,
        })
        .unwrap();
    let rows = model::checks(initial, &changed, &r);
    assert!(rows[..rs.len()].iter().all(|r| r.0));
    assert!(!rows.last().unwrap().0);
    let mut c = sim(&good.state).configuration().clone();
    c.links.insert("east-west".into(), false);
    let down = Simulation::new(initial.topology().clone(), c).unwrap();
    let mut expected = Availability::new(&down).unwrap();
    assert!(!expected.links["east-west"]);
    let mut r = rs.to_vec();
    r[0] = Requirement::Availability {
        expected: expected.clone(),
    };
    assert!(model::checks(initial, &down, &r)[0].0);
    expected.links.insert("east-west".into(), true);
    let up = Simulation::new(initial.topology().clone(), {
        let mut c = down.configuration().clone();
        c.links.insert("east-west".into(), true);
        c
    })
    .unwrap();
    assert!(
        !model::checks(initial, &up, &r)[0].0,
        "preserved-down link cannot be enabled unnoticed"
    );
}
#[test]
fn pending_invalid_numeric_text_is_saveable_atomic_and_current_credit_invalidates_then_repairs() {
    let s = scenario();
    let mut a = repaired(&s);
    let old = sim(&a.state).clone();
    let draft = a
        .workspace
        .as_mut()
        .unwrap()
        .spanning_tree_draft
        .as_mut()
        .unwrap();
    draft
        .bridges
        .get_mut("core")
        .unwrap()
        .ports
        .get_mut("west")
        .unwrap()
        .cost = "not-finished".into();
    assert!(draft.valid());
    assert!(!progress::current_solved(&a, &s));
    let (initial, _) = goal(&s);
    let State::SpanningTree { simulation } = &mut a.state else {
        panic!()
    };
    let d = a
        .workspace
        .as_mut()
        .unwrap()
        .spanning_tree_draft
        .as_mut()
        .unwrap();
    assert!(d.apply(initial, simulation).is_err());
    assert_eq!(*simulation, old);
    assert_eq!(d.bridges["core"].ports["west"].cost, "not-finished");
    for value in ["0", "200000001", "4294967296", "-1", "1.5"] {
        d.bridges
            .get_mut("core")
            .unwrap()
            .ports
            .get_mut("west")
            .unwrap()
            .cost = value.into();
        assert!(d.apply(initial, simulation).is_err());
        assert_eq!(*simulation, old);
    }
    d.bridges
        .get_mut("core")
        .unwrap()
        .ports
        .get_mut("west")
        .unwrap()
        .cost = "4".into();
    d.apply(initial, simulation).unwrap();
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn standalone_and_goal_engine_admission_checks_raw_length_before_trimming_and_context() {
    let s = scenario();
    let mut state = s.initial.clone();
    let before = state.clone();
    assert!(engine::execute(&mut state, &s.solution[0]).is_err());
    assert_eq!(state, before);
    assert!(
        engine::execute(&mut state, "help")
            .unwrap()
            .contains("steady-state")
    );
    let raw = " ".repeat(model::PREFIX.len() + core::MAX_ACTION_BYTES + 1);
    for goal in [true, false] {
        let e = if goal {
            engine::execute_for_goal(&mut state, &s.goal, &raw)
        } else {
            engine::execute(&mut state, &raw)
        }
        .unwrap_err();
        assert!(e.contains("exceeds"));
        assert_eq!(state, before);
    }
    let mut foreign_topology = sim(&state).topology().clone();
    foreign_topology.bridges.get_mut("core").unwrap().mac =
        dlms_learning_lab::mac_learning::Mac::parse("02:00:00:00:00:77").unwrap();
    let mut foreign = State::SpanningTree {
        simulation: Simulation::new(foreign_topology, sim(&state).configuration().clone()).unwrap(),
    };
    let exact = foreign.clone();
    assert!(engine::execute_for_goal(&mut foreign, &s.goal, &s.solution[0]).is_err());
    assert_eq!(foreign, exact);
}
#[test]
fn requirement_validation_rejects_missing_guards_duplicates_bad_paths_and_direct_oversized_values()
{
    let s = scenario();
    let (i, r) = goal(&s);
    for n in [0, 1] {
        let mut bad = r.to_vec();
        bad.remove(n);
        assert!(!model::valid_requirements(i, &bad));
    }
    let mut duplicate = r.to_vec();
    duplicate.push(r[2].clone());
    assert!(!model::valid_requirements(i, &duplicate));
    for path in [
        vec![],
        vec!["core-east".into()],
        vec!["core-west".into(), "core-west".into()],
    ] {
        let mut bad = r.to_vec();
        bad.push(Requirement::ForwardingPath {
            from: "west".into(),
            to: "east".into(),
            expected_links: Some(path),
        });
        assert!(!model::valid_requirements(i, &bad));
    }
    let mut large = r.to_vec();
    large.push(Requirement::Configuration {
        expected: core::Configuration {
            bridges: (0..13)
                .map(|n| {
                    (
                        format!("extra{n}"),
                        core::BridgeConfig {
                            priority: 32768,
                            online: true,
                            ports: BTreeMap::new(),
                        },
                    )
                })
                .collect(),
            links: BTreeMap::new(),
        },
    });
    assert!(!model::valid_requirements(i, &large));
    let mut unrelated = r.to_vec();
    unrelated[1] = Requirement::Connected {
        bridges: vec!["core".into(), "east".into()],
    };
    assert!(!model::valid_requirements(i, &unrelated));
    let mut absent = r.to_vec();
    absent.retain(|r| {
        matches!(
            r,
            Requirement::Availability { .. } | Requirement::Connected { .. }
        )
    });
    absent.push(Requirement::PortRole {
        endpoint: endpoint("east", "west"),
        role: StableRole::Disabled,
    });
    assert!(!model::valid_requirements(i, &absent));
}
#[test]
fn raw_collection_and_duplicate_bounds_preserve_wire_and_limit_typed_excess_before_body() {
    let s = scenario();
    let (i, r) = goal(&s);
    let json = serde_json::to_string(r).unwrap();
    let decoded: Vec<Requirement> = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, r);
    assert!(model::valid_requirements(i, &decoded));
    let mut entries = (0..12)
        .map(|n| format!("\"bridge{n}\":{{\"online\":true,\"ports\":{{}}}}"))
        .collect::<Vec<_>>();
    entries.push("\"excess\":42".into());
    let raw = format!("{{\"bridges\":{{{}}},\"links\":{{}}}}", entries.join(","));
    let e = serde_json::from_str::<Availability>(&raw)
        .unwrap_err()
        .to_string();
    assert!(e.contains("count"), "{e}");
    let mut v = serde_json::to_value(Draft::new(i).unwrap()).unwrap();
    v["bridges"]["core"]["priority"] = "x".repeat(65).into();
    assert!(serde_json::from_value::<Draft>(v).is_err());
    let d = serde_json::to_string(&Draft::new(i).unwrap()).unwrap();
    assert!(
        serde_json::from_str::<Draft>(&d.replacen(
            "\"cost\":\"4\"",
            "\"cost\":\"4\",\"cost\":\"5\"",
            1
        ))
        .is_err()
    );
}
#[test]
fn separate_example_fill_apply_experiment_repair_never_changes_assignment_or_borrows_credit() {
    let s = scenario();
    let mut a = prepared(&s);
    let assignment = a.state.clone();
    let lesson = s.spanning_tree_teaching.as_ref().unwrap();
    {
        let l = learning(&mut a);
        let initial = l.simulation.clone();
        l.fill_example(lesson).unwrap();
        assert_eq!(l.simulation, initial);
        assert!(!l.draft.matches(&l.simulation));
        assert!(!l.checks(lesson)[0].0);
        l.apply(lesson).unwrap();
        assert!(l.checks(lesson).iter().all(|r| r.0));
        for action in &lesson.try_change.actions {
            l.execute(lesson, &command(action)).unwrap();
        }
        assert_eq!(
            l.checks(lesson)[1..]
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            lesson.try_change.expected
        );
        for action in &lesson.repair {
            l.execute(lesson, &command(action)).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
    }
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn practice_immutable_initial_and_fixed_graph_pins_allow_legitimate_coincident_configs() {
    let s = scenario();
    let lesson = s.spanning_tree_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let before = serde_json::to_vec(&l).unwrap();
    let mut other = lesson.clone();
    let mut c = other.fixture.configuration().clone();
    c.bridges.get_mut("sample-core").unwrap().priority = 49152;
    other.fixture = Simulation::new(other.fixture.topology().clone(), c).unwrap();
    other.validate().unwrap();
    assert!(model::same_context(&other.fixture, &lesson.fixture));
    assert!(!l.valid_for(&other));
    assert!(
        l.execute(&other, &command(&priority("sample-core", 0)))
            .is_err()
    );
    assert_eq!(serde_json::to_vec(&l).unwrap(), before);
    l.fill_example(lesson).unwrap();
    l.apply(lesson).unwrap();
    let mut coincidence = Learning::new(lesson).unwrap();
    coincidence.simulation = l.simulation.clone();
    coincidence.draft = Draft::new(&coincidence.simulation).unwrap();
    assert!(coincidence.valid_for(lesson));
    assert!(coincidence.checks(lesson).iter().all(|r| r.0));
}
#[test]
fn prepare_wrong_kind_or_foreign_existing_practice_is_atomic() {
    let s = scenario();
    let lesson = s.spanning_tree_teaching.as_ref().unwrap();
    let old = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic1-permissions")
        .unwrap();
    let mut a = Attempt::new(&old, 1);
    let before = serde_json::to_vec(&a).unwrap();
    assert!(teaching::prepare(&mut a, lesson).is_err());
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    let mut a = prepared(&s);
    let mut other = lesson.clone();
    let mut t = other.fixture.topology().clone();
    t.bridges.get_mut("sample-core").unwrap().mac =
        dlms_learning_lab::mac_learning::Mac::parse("02:00:00:00:00:88").unwrap();
    other.fixture = Simulation::new(t, other.fixture.configuration().clone()).unwrap();
    let foreign = Learning::new(&other).unwrap();
    assert!(foreign.valid());
    a.workspace.as_mut().unwrap().spanning_tree_learning = Some(foreign);
    a.workspace.as_mut().unwrap().spanning_tree_draft = None;
    let before = serde_json::to_vec(&a).unwrap();
    assert!(teaching::prepare(&mut a, lesson).is_err());
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
}
#[test]
fn pack_rejects_kind_context_teaching_duplicates_and_unreachable_reference_without_editing_old_specs()
 {
    let s = scenario();
    let original = serde_json::to_value(&s).unwrap();
    for change in 0..5 {
        let mut v = original.clone();
        match change {
            0 => v["goal"]["initial"]["topology"]["system_id_extension"] = 2.into(),
            1 => {
                let d = v["goal"]["requirements"][2].clone();
                v["goal"]["requirements"].as_array_mut().unwrap().push(d);
            }
            2 => v["solution"] = serde_json::json!([command(&priority("core", 61440))]),
            3 => v["spanning_tree_teaching"]["fixture"] = v["initial"]["simulation"].clone(),
            _ => {
                v["initial"] = serde_json::json!({"kind":"Logs","files":{"x":"line"},"result":""});
            }
        }
        let raw = serde_json::json!({"schema_version":1,"scenarios":[v]}).to_string();
        assert!(pack::load(&raw).is_err(), "invalid group{change}");
    }
    assert_eq!(serde_json::to_value(&s).unwrap(), original);
}
fn maximum_world(flavor: u8) -> Simulation {
    let key = |kind: &str, n: usize| {
        format!(
            "{}{kind}-{n:02}-{}",
            char::from(flavor),
            "x".repeat(32 - (1 + kind.len() + 4))
        )
    };
    let mut topology = core::Topology {
        system_id_extension: 4095,
        bridges: BTreeMap::new(),
        links: BTreeMap::new(),
    };
    let mut configuration = core::Configuration {
        bridges: BTreeMap::new(),
        links: BTreeMap::new(),
    };
    let mut next = [0usize; 12];
    for i in 0..12 {
        let mut ports = BTreeMap::new();
        let mut settings = BTreeMap::new();
        for p in 0..8 {
            let name = key("port", p);
            assert_eq!(name.len(), 32);
            ports.insert(name.clone(), p as u16 + 1);
            settings.insert(
                name,
                core::PortConfig {
                    priority: 240,
                    cost: core::MAX_COST,
                    up: true,
                },
            );
        }
        let b = key("bridge", i);
        assert_eq!(b.len(), 32);
        topology.bridges.insert(
            b.clone(),
            core::Bridge {
                mac: dlms_learning_lab::mac_learning::Mac([2, 0, 0, 0, flavor, i as u8 + 1]),
                ports,
            },
        );
        configuration.bridges.insert(
            b,
            core::BridgeConfig {
                priority: if i == 1 { 0 } else { 32768 },
                online: true,
                ports: settings,
            },
        );
    }
    for i in 0..12 {
        for delta in 1..=4 {
            let j = (i + delta) % 12;
            let id = key("link", topology.links.len());
            let a = Endpoint {
                bridge: key("bridge", i),
                port: key("port", next[i]),
            };
            next[i] += 1;
            let b = Endpoint {
                bridge: key("bridge", j),
                port: key("port", next[j]),
            };
            next[j] += 1;
            topology.links.insert(id.clone(), core::Link { a, b });
            configuration.links.insert(id, true);
        }
    }
    assert!(next.iter().all(|n| *n == 8));
    Simulation::new(topology, configuration).unwrap()
}
fn max_scenario() -> Scenario {
    let mut s = scenario();
    let initial = maximum_world(b'a');
    let sample = maximum_world(b'z');
    let goal_for = |s: &Simulation| {
        let keys = s.topology().bridges.keys().cloned().collect::<Vec<_>>();
        vec![
            Requirement::Availability {
                expected: Availability::new(s).unwrap(),
            },
            Requirement::Connected {
                bridges: keys.clone(),
            },
            Requirement::Root {
                bridge: keys[1].clone(),
                root: keys[0].clone(),
            },
        ]
    };
    let requirements = goal_for(&initial);
    let sr = goal_for(&sample);
    let sample_first = sample.topology().bridges.keys().next().unwrap().clone();
    let lesson = s.spanning_tree_teaching.as_mut().unwrap();
    lesson.fixture = sample;
    lesson.requirements = sr;
    lesson.example_actions[0].action = priority(&sample_first, 0);
    lesson.try_change.actions = vec![priority(&sample_first, 32768)];
    lesson.try_change.expected = vec![true, true, false];
    lesson.repair = vec![priority(&sample_first, 0)];
    let first = initial.topology().bridges.keys().next().unwrap().clone();
    s.initial = State::SpanningTree {
        simulation: initial.clone(),
    };
    s.goal = Goal::SpanningTree {
        initial,
        requirements,
    };
    s.solution = vec![command(&priority(&first, 0))];
    s
}
fn maximal_raw(d: &mut Draft) {
    for b in d.bridges.values_mut() {
        b.priority = "\\\"".repeat(32);
        for p in b.ports.values_mut() {
            p.priority = "\\\"".repeat(32);
            p.cost = "\\\"".repeat(32);
        }
    }
    assert!(d.valid());
}
#[test]
fn worst_shape_escaped_unfinished_drafts_and_learning_compact_pretty_profile_roundtrip() {
    let s = max_scenario();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[s]}).to_string();
    let s = pack::load(&raw).unwrap().remove(0);
    let mut a = prepared(&s);
    maximal_raw(
        a.workspace
            .as_mut()
            .unwrap()
            .spanning_tree_draft
            .as_mut()
            .unwrap(),
    );
    maximal_raw(&mut learning(&mut a).draft);
    assert!(learning(&mut a).valid());
    for pretty in [false, true] {
        let encoded = if pretty {
            serde_json::to_vec_pretty(&a).unwrap()
        } else {
            serde_json::to_vec(&a).unwrap()
        };
        let copy: Attempt = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            serde_json::to_value(copy).unwrap(),
            serde_json::to_value(&a).unwrap()
        );
        let l = learning(&mut a);
        let bytes = if pretty {
            serde_json::to_vec_pretty(l).unwrap()
        } else {
            serde_json::to_vec(l).unwrap()
        };
        assert!(bytes.len() < teaching::MAX_LEARNING_BYTES);
        assert!(
            serde_json::from_slice::<Learning>(&bytes)
                .unwrap()
                .valid_for(s.spanning_tree_teaching.as_ref().unwrap())
        );
    }
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let bytes = serde_json::to_vec_pretty(&p).unwrap();
    assert!(bytes.len() < 16_000_000);
    let decoded = progress::decode(&bytes, &[s]).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        serde_json::to_value(p).unwrap()
    );
}
#[test]
fn original125_source_metadata_unit_order_and_omitted_optional_wire_are_exact() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/spanning-tree-original125.json")).unwrap();
    let expected: Vec<Scenario> = serde_json::from_value(v["scenarios"].clone()).unwrap();
    let all = catalog();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    assert_eq!(
        all[..125].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        expected.iter().map(|s| s.id.as_str()).collect::<Vec<_>>()
    );
    for (a, b) in all.iter().zip(expected) {
        let encoded = teaching_amendment::original_scenario(a).unwrap();
        assert_eq!(encoded, serde_json::to_value(b).unwrap(), "{} spec", a.id);
        assert!(encoded.get("spanning_tree_teaching").is_none());
        let mut attempt = Attempt::new(a, 1);
        workspace::prepare(&mut attempt);
        let w = serde_json::to_value(attempt.workspace.unwrap()).unwrap();
        assert!(
            w.get("spanning_tree_draft").is_none() && w.get("spanning_tree_learning").is_none()
        );
    }
    assert_eq!(
        &serde_json::from_str::<serde_json::Value>(include_str!("../scenarios/units-v1.json"))
            .unwrap()["units"]
            .as_array()
            .unwrap()[..31],
        v["units"]["units"].as_array().unwrap().as_slice()
    );
}
fn retained_profile(all: &[Scenario]) -> Progress {
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        all,
    )
    .unwrap();
    for s in all {
        let a = p
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
        workspace::prepare(a);
        if let (State::Script { source, run }, Goal::Script { cases }) = (&mut a.state, &s.goal) {
            let authored = if s.id == "shell-case-publication" {
                "# retained unfinished assignment\nprintf '".to_owned()
            } else {
                s.solution.last().unwrap().clone()
            };
            *source = authored.clone();
            if s.id == "shell-case-publication" {
                *run = None;
            } else {
                *run = Some(shell::Record {
                    engine_revision: shell::EVALUATOR_REVISION,
                    source: authored.clone(),
                    results: cases.iter().map(|c| shell::run(&authored, c)).collect(),
                });
            }
            if let Some(l) = a
                .workspace
                .as_mut()
                .and_then(|w| w.script_learning.as_mut())
            {
                if s.id == "shell-case-publication" {
                    l.example_source = "# separate retained unfinished example\nprintf '".into();
                }
                if let Some(teaching) = &s.teaching {
                    l.example_result = Some(shell::run(&l.example_source, &teaching.fixture));
                    l.example_revision = shell::EVALUATOR_REVISION;
                }
            }
        }
    }
    p.favorites.insert("shell-case-publication".into());
    p.favorites.insert("ccna-dhcp-held-options".into());
    p
}
fn prior(p: &Progress, all: &[Scenario]) -> BTreeMap<String, serde_json::Value> {
    all.iter()
        .map(|s| {
            (
                s.id.clone(),
                serde_json::to_value(&p.attempts[&s.id]).unwrap(),
            )
        })
        .collect()
}
#[test]
fn real_current_records_genuine678_history_both_drafts_export_restore_and_selected_reset_preserve125()
 {
    let mut all = catalog();
    let original = all.clone();
    let s = scenario();
    let mut p = retained_profile(&all);
    assert_eq!(one_off_boundary::before_len(&p.attempts), 176);
    let histories = p
        .attempts
        .values()
        .filter_map(|a| a.workspace.as_ref())
        .flat_map(|w| {
            w.script_history
                .as_ref()
                .into_iter()
                .chain(&w.script_history_more)
        })
        .map(|r| r.engine_revision)
        .collect::<BTreeSet<_>>();
    assert!(histories.is_superset(&BTreeSet::from([6, 7, 8])));
    let current=p.attempts.values().filter(|a|matches!(&a.state,State::Script{run:Some(r),..}if r.engine_revision==shell::EVALUATOR_REVISION)).count();
    // Closed literal IDs from the accepted public49-script catalog, independent
    // of the newly extended current catalog. Its original48 current executions
    // and one deliberately unfinished assignment must still be preserved.
    let original_script_ids = BTreeSet::from([
        "shell-handover",
        "shell-arguments",
        "shell-input",
        "shell-validation",
        "shell-report",
        "shell-loop",
        "shell-capstone",
        "shell-streams",
        "shell-pipelines",
        "shell-eof",
        "shell-batch-audit",
        "shell-functions",
        "shell-return",
        "shell-shift",
        "shell-bundle",
        "shell-local-heading",
        "shell-scope-review",
        "shell-scope-capstone",
        "shell-cut-inventory",
        "shell-sort-manifest",
        "shell-uniq-counts",
        "shell-text-capstone",
        "shell-number-records",
        "shell-order-total",
        "shell-pack-items",
        "shell-count-capstone",
        "shell-file-manifest",
        "shell-collect-errors",
        "shell-file-batch-capstone",
        "shell-array-intake",
        "shell-array-priority",
        "shell-array-package",
        "shell-array-handover",
        "shell-capture-preview",
        "shell-capture-evidence",
        "shell-capture-decision",
        "shell-capture-handover",
        "shell-case-interface",
        "shell-case-routing",
        "shell-case-publication",
        "shell-options-report",
        "shell-options-jobs",
        "shell-options-publication",
        "shell-preserve-lines",
        "shell-pipeline-status",
        "shell-safe-handover",
        "shell-command-options",
        "shell-command-dispatch",
        "shell-command-plan",
    ]);
    assert_eq!(original_script_ids.len(), 49);
    let original_current = p.attempts.iter().filter(|(id, _)| original_script_ids.contains(id.as_str()))
        .filter(|(_, a)| matches!(&a.state, State::Script { run: Some(record), .. } if record.engine_revision == shell::EVALUATOR_REVISION)).count();
    assert_eq!(
        original_current, 48,
        "Original49 scripts retain48 genuine current runs and the original unfinished draft"
    );
    let new_ids = [
        "shell-optional-policy",
        "shell-optional-report",
        "shell-optional-handover",
    ];
    let numeric_ids = [
        "shell-numeric-retry",
        "shell-numeric-intake",
        "shell-numeric-publication",
    ];
    let expected_scripts = original_script_ids
        .iter()
        .copied()
        .chain(new_ids)
        .chain(numeric_ids)
        .chain(["shell-loop-intake", "shell-loop-fallback"])
        .collect::<BTreeSet<_>>();
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.goal, Goal::Script { .. }))
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>(),
        expected_scripts
    );
    for id in new_ids
        .into_iter()
        .chain(numeric_ids)
        .chain(["shell-loop-intake", "shell-loop-fallback"])
    {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let a = &p.attempts[id];
        let (
            State::Script {
                source,
                run: Some(record),
            },
            Goal::Script { cases },
        ) = (&a.state, &s.goal)
        else {
            panic!("Expected explicitly executed new script {id}")
        };
        assert_eq!(source, s.solution.last().unwrap());
        assert_eq!(record.engine_revision, shell::EVALUATOR_REVISION);
        assert_eq!(&record.source, source);
        assert_eq!(record.results.len(), cases.len());
        assert!(
            record.results.iter().zip(cases).all(|(r, c)| c.accepts(r)),
            "Every genuinely run new case passes its whole outcome"
        );
        assert!(progress::current_solved(a, s));
    }
    let original52_current=p.attempts.iter().filter(|(id,_)|original_script_ids.contains(id.as_str())||new_ids.contains(&id.as_str()))
        .filter(|(_,a)|matches!(&a.state,State::Script{run:Some(r),..}if r.engine_revision==shell::EVALUATOR_REVISION)).count();
    assert_eq!(
        original52_current, 51,
        "Original52 retain51 actual current runs and the same unfinished assignment"
    );
    let original55_current=p.attempts.iter().filter(|(id,_)|expected_scripts.contains(id.as_str()) && !["shell-loop-intake","shell-loop-fallback"].contains(&id.as_str()))
        .filter(|(_,a)|matches!(&a.state,State::Script{run:Some(r),..}if r.engine_revision==shell::EVALUATOR_REVISION)).count();
    assert_eq!(
        original55_current, 54,
        "Original55 keep54 actual current runs and same unfinished draft"
    );
    assert_eq!(
        current, 56,
        "57 current scripts with the same one retained unfinished assignment"
    );
    let prior = prior(&p, &original);
    let mut a = prepared(&s);
    let lesson = s.spanning_tree_teaching.as_ref().unwrap();
    learning(&mut a).fill_example(lesson).unwrap();
    learning(&mut a).apply(lesson).unwrap();
    learning(&mut a).view = View::Example;
    learning(&mut a)
        .draft
        .bridges
        .get_mut("sample-core")
        .unwrap()
        .priority = "unfinished-sample".into();
    a.workspace
        .as_mut()
        .unwrap()
        .spanning_tree_draft
        .as_mut()
        .unwrap()
        .bridges
        .get_mut("core")
        .unwrap()
        .priority = "unfinished-main".into();
    let selected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    all.push(s.clone());
    let favorites = p.favorites.clone();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let old8 = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    std::fs::write(&path, old8).unwrap();
    let old7 = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    std::fs::write(path.with_extension("before-shell-v8.json"), old7).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension(format!(
            "before-shell-v{}.json",
            dlms_learning_lab::shell::EVALUATOR_REVISION
        )))
        .unwrap(),
        old8
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        old7
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, expected) in &prior {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[id]).unwrap(),
            *expected,
            "{id}"
        );
    }
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        selected
    );
    let export = dir.path().join("explicit-backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    for (id, expected) in &prior {
        assert_eq!(
            serde_json::to_value(&candidate.progress.attempts[id]).unwrap(),
            *expected,
            "inspect {id}"
        );
    }
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts[&s.id]).unwrap(),
        selected
    );
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(restored.favorites, favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        selected
    );
    for (id, expected) in &prior {
        assert_eq!(
            serde_json::to_value(&restored.attempts[id]).unwrap(),
            *expected,
            "restored {id}"
        );
    }
    restored.attempts.insert(s.id.clone(), prepared(&s));
    for (id, expected) in &prior {
        assert_eq!(
            serde_json::to_value(&restored.attempts[id]).unwrap(),
            *expected,
            "reset {id}"
        );
    }
    assert_eq!(restored.favorites, favorites);
    assert_eq!(restored.attempts[&s.id].state, s.initial);
}
#[test]
fn import_and_save_reject_wrong_kind_fixed_identity_and_foreign_practice_without_overwrite() {
    let mut all = catalog();
    let s = scenario();
    all.push(s.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), prepared(&s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut raw = serde_json::to_value(&p).unwrap();
    raw["attempts"][&s.id]["state"]["simulation"]["topology"]["system_id_extension"] = 2.into();
    assert!(progress::decode(&serde_json::to_vec(&raw).unwrap(), &all).is_err());
    let old = all.iter().find(|s| s.id == "lpic1-permissions").unwrap();
    let metadata = p.attempts[&s.id].workspace.clone();
    let mut wrong = Attempt::new(old, 1);
    wrong.workspace = metadata;
    p.attempts.insert(old.id.clone(), wrong);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}

#[test]
fn exact_configuration_and_bridge_priority_guards_reject_only_explicitly_stated_differences() {
    let s = scenario();
    let (initial, requirements) = goal(&s);
    let good = repaired(&s);
    let mut rs = requirements.to_vec();
    rs.push(Requirement::BridgePriority {
        bridge: "core".into(),
        value: 0,
    });
    rs.push(Requirement::Configuration {
        expected: sim(&good.state).configuration().clone(),
    });
    assert!(
        model::checks(initial, sim(&good.state), &rs)
            .iter()
            .all(|r| r.0)
    );
    let mut changed = sim(&good.state).clone();
    changed.apply(&priority("core", 4096)).unwrap();
    let rows = model::checks(initial, &changed, &rs);
    assert!(rows[..requirements.len()].iter().all(|r| r.0));
    assert!(!rows[requirements.len()].0 && !rows[requirements.len() + 1].0);
}

#[test]
fn directly_constructed_oversized_action_names_and_maps_reject_before_example_publication() {
    let s = scenario();
    let lesson = s.spanning_tree_teaching.as_ref().unwrap();
    let oversized = Action::BridgePriority {
        bridge: "\\".repeat(core::MAX_ACTION_BYTES * 2),
        priority: 0,
    };
    let too_many = Action::Reconfigure {
        configuration: core::Configuration {
            bridges: (0..13)
                .map(|n| {
                    (
                        format!("bridge{n}"),
                        core::BridgeConfig {
                            priority: 32768,
                            online: true,
                            ports: BTreeMap::new(),
                        },
                    )
                })
                .collect(),
            links: BTreeMap::new(),
        },
    };
    for action in [oversized, too_many] {
        for location in 0..3 {
            let mut bad = lesson.clone();
            match location {
                0 => bad.example_actions[0].action = action.clone(),
                1 => bad.try_change.actions = vec![action.clone()],
                _ => bad.repair = vec![action.clone()],
            }
            let fixture = serde_json::to_vec(&bad.fixture).unwrap();
            let error = bad.validate().unwrap_err();
            assert!(error.len() <= 512);
            assert!(!error.contains("\\"));
            assert!(Learning::new(&bad).is_err());
            let mut a = Attempt::new(&s, 1);
            let before = serde_json::to_vec(&a).unwrap();
            assert!(teaching::prepare(&mut a, &bad).is_err());
            assert_eq!(serde_json::to_vec(&a).unwrap(), before);
            assert_eq!(serde_json::to_vec(&bad.fixture).unwrap(), fixture);
        }
    }
}
