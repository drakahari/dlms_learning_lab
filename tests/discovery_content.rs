use dlms_learning_lab::{
    discovery::{self, Check, Requirement},
    discovery_teaching::Learning,
    discovery_workspace::{self as editor, Action, Draft},
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/discovery-v1.json")).unwrap()
}
fn old_catalog() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit != "lldp-discovery")
        .collect()
}
fn catalog() -> Vec<pack::Scenario> {
    let mut all = old_catalog();
    all.extend(labs());
    all
}
fn simulation(state: &State) -> &discovery::State {
    let State::Discovery { simulation } = state else {
        panic!("LLDP state expected")
    };
    simulation
}
fn simulation_mut(state: &mut State) -> &mut discovery::State {
    let State::Discovery { simulation } = state else {
        panic!("LLDP state expected")
    };
    simulation
}
fn goal(lab: &pack::Scenario) -> (&discovery::Topology, &[Requirement]) {
    let Goal::Discovery {
        initial,
        requirements,
    } = &lab.goal
    else {
        panic!("LLDP goal expected")
    };
    (initial, requirements)
}
fn expected(lab: &pack::Scenario) -> &discovery::Topology {
    goal(lab)
        .1
        .iter()
        .find_map(|r| match r {
            Requirement::Configuration { expected } => Some(expected),
            _ => None,
        })
        .unwrap()
}
fn run(lab: &pack::Scenario, commands: &[String]) -> State {
    let mut state = lab.initial.clone();
    for command in commands {
        engine::execute(&mut state, command).unwrap();
    }
    state
}
fn reference(lab: &pack::Scenario) -> State {
    run(lab, &lab.solution)
}
fn command(action: &Action) -> String {
    format!("discovery {}", serde_json::to_string(action).unwrap())
}
fn e(device: &str, port: &str) -> discovery::Endpoint {
    discovery::Endpoint {
        device: device.into(),
        port: port.into(),
    }
}
fn send(state: &mut State, device: &str, port: &str) {
    engine::execute(
        state,
        &command(&Action::Advertise {
            from: e(device, port),
        }),
    )
    .unwrap();
}
fn apply_expected(lab: &pack::Scenario) -> State {
    let mut state = lab.initial.clone();
    simulation_mut(&mut state)
        .reconfigure(expected(lab).clone())
        .unwrap();
    state
}
fn attempt(lab: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(lab, 1);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().discovery_learning =
        Some(Learning::new(lab.discovery_teaching.as_ref().unwrap()));
    a
}

#[test]
fn four_substantive_assignments_teach_native_actions_and_keep_reference_json_out_of_learner_text() {
    let all = labs();
    assert_eq!(all.len(), 4);
    let shapes: Vec<_> = all
        .iter()
        .map(|s| simulation(&s.initial).topology().devices.len())
        .collect();
    assert_eq!(shapes, [2, 3, 3, 4]);
    for s in &all {
        assert_eq!(s.unit, "lldp-discovery");
        assert!(s.objective.len() > 250 && s.explanation.len() > 200);
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(s.solution.iter().all(|c| c.starts_with("discovery {")));
        let t = s.discovery_teaching.as_ref().unwrap();
        t.validate().unwrap();
        let initial = simulation(&s.initial).topology();
        assert!(
            initial
                .devices
                .keys()
                .all(|id| !t.fixture.topology().devices.contains_key(id))
        );
        assert!(!initial.same_identity(t.fixture.topology()));
        for prose in [
            &s.objective,
            &s.explanation,
            &s.summary,
            &t.example_description,
            &t.try_change.description,
        ] {
            assert!(!prose.contains("\"kind\"") && !prose.contains("discovery {"));
        }
        assert!(
            s.guide
                .iter()
                .all(|g| g.commands.is_empty() && !g.action.contains("\"kind\""))
        );
        assert!(s.sources.iter().all(|url| url.starts_with("https://")));
        assert!(
            s.sources
                .iter()
                .any(|url| url.contains("200-301-CCNA-v1.1.pdf"))
        );
        assert!(s.simplifications.contains("no real packets") && s.simplifications.contains("CDP"));
        assert!(
            s.solution
                .iter()
                .map(|s| editor::parse_action(s).unwrap())
                .all(|a| !matches!(a, Action::Withdraw { .. }))
        );
        assert!(
            t.example_solution
                .iter()
                .chain(&t.try_change.actions)
                .all(|a| !matches!(a, Action::Withdraw { .. }))
        );
        assert_eq!(
            goal(s)
                .1
                .iter()
                .filter(|r| matches!(r, Requirement::Configuration { .. }))
                .count(),
            1
        );
        assert!(expected(s).links.iter().all(|l| l.up));
    }
}

#[test]
fn every_reference_and_alternate_order_or_refresh_count_is_graded_by_current_state() {
    for s in labs() {
        assert!(
            engine::solved(&reference(&s), &s.goal),
            "{} reference",
            s.id
        );
        let mut configuration: Vec<_> = s
            .solution
            .iter()
            .map(|c| editor::parse_action(c).unwrap())
            .filter(Action::is_configuration)
            .collect();
        configuration.reverse();
        let mut alternate = s.initial.clone();
        for action in &configuration {
            engine::execute(&mut alternate, &command(action)).unwrap();
        }
        let endpoints: Vec<_> = expected(&s)
            .links
            .iter()
            .flat_map(|l| [l.a.clone(), l.b.clone()])
            .collect();
        for _ in 0..3 {
            for endpoint in endpoints.iter().rev() {
                engine::execute(
                    &mut alternate,
                    &command(&Action::Advertise {
                        from: endpoint.clone(),
                    }),
                )
                .unwrap();
            }
        }
        assert!(
            engine::solved(&alternate, &s.goal),
            "{} alternate order and refreshes",
            s.id
        );
        let mut native = attempt(&s);
        let final_state = discovery::State::new(expected(&s).clone()).unwrap();
        let draft = Draft::new(&final_state);
        draft.apply(simulation_mut(&mut native.state)).unwrap();
        native.workspace.as_mut().unwrap().discovery_draft = Some(draft);
        for endpoint in endpoints {
            engine::execute(
                &mut native.state,
                &command(&Action::Advertise { from: endpoint }),
            )
            .unwrap();
        }
        assert!(
            progress::current_solved(&native, &s),
            "{} native draft path",
            s.id
        );
    }
}

#[test]
fn distinct_plausible_partial_repairs_and_read_only_inspection_never_complete_a_task() {
    for s in labs() {
        let before = s.initial.clone();
        for device in simulation(&before).topology().devices.keys() {
            simulation(&before).neighbors(device).unwrap();
        }
        assert_eq!(before, s.initial);
        assert!(!engine::solved(&before, &s.goal));
        let mut partial = apply_expected(&s);
        if s.id == "ccna-lldp-handover" {
            for (device, port) in [
                ("finance", "uplink"),
                ("relay", "finance"),
                ("lab", "uplink"),
                ("relay", "lab"),
            ] {
                send(&mut partial, device, port);
            }
        }
        assert!(!engine::solved(&partial, &s.goal), "{} partial", s.id);
        let rows = engine::checks(&partial, &s.goal);
        assert!(
            rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0),
            "{} meaningful partial feedback",
            s.id
        );
        assert!(
            rows.last().unwrap().0,
            "partial has exactly the requested configuration"
        );
        if s.id == "ccna-lldp-inventory" || s.id == "ccna-lldp-handover" {
            assert!(
                rows.iter().any(|(_, text)| text.contains("historical")),
                "{} stale inventory is distinct from a missing row",
                s.id
            );
        }
    }
}

#[test]
fn stale_cache_missing_peer_and_exact_expiry_invalidate_otherwise_correct_work() {
    for s in labs() {
        let good = reference(&s);
        let (_, requirements) = goal(&s);
        let local = requirements
            .iter()
            .find_map(|r| match r {
                Requirement::CurrentNeighbor { local, .. } => Some(local),
                _ => None,
            })
            .unwrap();
        let mut historical = serde_json::to_value(simulation(&good)).unwrap();
        let n = historical["cache"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["local"] == serde_json::to_value(local).unwrap())
            .unwrap();
        n["advertisement"]["system_name"] = serde_json::json!("Stale inventory");
        let state = State::Discovery {
            simulation: discovery::State::from_json(&historical.to_string()).unwrap(),
        };
        assert!(
            !engine::solved(&state, &s.goal),
            "{} stale false positive",
            s.id
        );
        assert!(engine::checks(&state, &s.goal).last().unwrap().0);
        let mut missing = serde_json::to_value(simulation(&good)).unwrap();
        missing["cache"]
            .as_array_mut()
            .unwrap()
            .retain(|n| n["local"] != serde_json::to_value(local).unwrap());
        let state = State::Discovery {
            simulation: discovery::State::from_json(&missing.to_string()).unwrap(),
        };
        assert!(!engine::solved(&state, &s.goal), "{} missing record", s.id);
        let minimum = requirements
            .iter()
            .filter_map(|r| match r {
                Requirement::CurrentNeighbor { local, .. } => Some(
                    simulation(&good)
                        .neighbor(local)
                        .unwrap()
                        .unwrap()
                        .remaining_seconds,
                ),
                _ => None,
            })
            .min()
            .unwrap();
        let mut aged = good.clone();
        engine::execute(
            &mut aged,
            &command(&Action::Advance {
                seconds: u64::from(minimum) - 1,
            }),
        )
        .unwrap();
        assert!(
            engine::solved(&aged, &s.goal),
            "{} before exact expiry",
            s.id
        );
        engine::execute(&mut aged, &command(&Action::Advance { seconds: 1 })).unwrap();
        assert!(!engine::solved(&aged, &s.goal), "{} at exact expiry", s.id);
    }
}

#[test]
fn whole_configuration_rejects_physical_shortcuts_unrelated_damage_and_wrong_receive_policy() {
    for s in labs() {
        for damage in 0..2 {
            let mut state = reference(&s);
            let mut configuration = simulation(&state).topology().clone();
            if damage == 0 {
                configuration.links[0].up = false;
            } else {
                let device = configuration
                    .devices
                    .values_mut()
                    .find(|d| d.ports.contains_key("spare"))
                    .unwrap();
                device.ports.get_mut("spare").unwrap().transmit = true;
            }
            simulation_mut(&mut state)
                .reconfigure(configuration)
                .unwrap();
            assert!(
                !engine::solved(&state, &s.goal),
                "{} preserved setting",
                s.id
            );
            assert!(!engine::checks(&state, &s.goal).last().unwrap().0);
        }
    }
    let s = labs()
        .into_iter()
        .find(|s| s.id == "ccna-lldp-transmit-policy")
        .unwrap();
    let mut wrong = reference(&s);
    let mut configuration = simulation(&wrong).topology().clone();
    configuration
        .devices
        .get_mut("visitor")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .receive = false;
    simulation_mut(&mut wrong)
        .reconfigure(configuration)
        .unwrap();
    let rows = engine::checks(&wrong, &s.goal);
    assert!(
        rows[..rows.len() - 1].iter().all(|row| row.0),
        "every neighbor outcome alone still passes"
    );
    assert!(
        !rows.last().unwrap().0 && !engine::solved(&wrong, &s.goal),
        "changing the wrong receiver is not the requested preserved policy"
    );
    let mut broken = reference(&s);
    let mut configuration = simulation(&broken).topology().clone();
    configuration
        .links
        .iter_mut()
        .find(|l| l.id == "visitor-wire")
        .unwrap()
        .up = false;
    simulation_mut(&mut broken)
        .reconfigure(configuration)
        .unwrap();
    assert!(
        engine::checks(&broken, &s.goal)[0].0,
        "absence alone can be manufactured by breaking the link"
    );
    assert!(!engine::solved(&broken, &s.goal));
}

#[test]
fn each_small_example_and_typed_try_change_is_isolated_and_fill_never_sends_or_advances() {
    for s in labs() {
        let lesson = s.discovery_teaching.as_ref().unwrap();
        let mut a = attempt(&s);
        let assignment = a.state.clone();
        let assignment_draft = a.workspace.as_ref().unwrap().discovery_draft.clone();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .discovery_learning
            .as_mut()
            .unwrap();
        learning.view = View::Example;
        let first = learning.draft.devices.keys().next().unwrap().clone();
        learning.draft.devices.get_mut(&first).unwrap().ttl = "unfinished".into();
        let before = learning.simulation.clone();
        learning.fill_example(lesson).unwrap();
        assert_eq!(learning.simulation, before);
        assert!(!learning.checked);
        learning.apply().unwrap();
        assert_eq!(learning.simulation.now(), before.now());
        assert!(
            learning.check(lesson).unwrap().iter().any(|r| !r.passed()),
            "{} config alone is partial",
            s.id
        );
        for action in lesson
            .example_solution
            .iter()
            .filter(|a| !a.is_configuration())
        {
            learning.execute(action).unwrap();
        }
        assert!(
            learning.check(lesson).unwrap().iter().all(|r| r.passed()),
            "{} small example",
            s.id
        );
        let worked = learning.simulation.clone();
        for action in &lesson.try_change.actions {
            learning.execute(action).unwrap();
        }
        assert_ne!(learning.simulation, worked);
        assert!(
            lesson.try_change.outcomes.iter().all(|r| r
                .check(lesson.fixture.topology(), &learning.simulation)
                .is_ok_and(Check::passed)),
            "{} try change",
            s.id
        );
        assert_eq!(a.state, assignment);
        assert_eq!(
            a.workspace.as_ref().unwrap().discovery_draft,
            assignment_draft
        );
        assert!(!progress::current_solved(&a, &s));
    }
}

#[test]
fn current_unsaved_edits_and_saved_invalid_ttl_clock_and_practice_drafts_survive_selected_reset() {
    let old = old_catalog();
    assert_eq!(old.len(), 82);
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &old {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let original = serde_json::to_vec(&p).unwrap();
    for s in labs() {
        let mut a = Attempt::new(&s, 2);
        a.state = reference(&s);
        workspace::prepare(&mut a);
        assert!(progress::current_solved(&a, &s));
        let draft = a
            .workspace
            .as_mut()
            .unwrap()
            .discovery_draft
            .as_mut()
            .unwrap();
        let first = draft.devices.keys().next().unwrap().clone();
        draft.devices.get_mut(&first).unwrap().ttl = "invalid saved TTL".into();
        draft.advance_seconds = "unfinished clock request".into();
        let mut learning = Learning::new(s.discovery_teaching.as_ref().unwrap());
        learning.view = View::Assignment;
        let first = learning.draft.devices.keys().next().unwrap().clone();
        learning.draft.devices.get_mut(&first).unwrap().ttl = "incomplete practice".into();
        a.workspace.as_mut().unwrap().discovery_learning = Some(learning);
        assert!(!progress::current_solved(&a, &s));
        p.attempts.insert(s.id.clone(), a);
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress-v1.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    // Main-file provenance is intentionally absent from portable learning data.
    assert_eq!(
        serde_json::to_vec(&loaded).unwrap(),
        serde_json::to_vec(&p).unwrap()
    );
    let receipts = loaded.migration_receipts.as_ref().unwrap();
    assert!(receipts.workspace);
    assert!(
        receipts
            .shell
            .contains(&dlms_learning_lab::shell::EVALUATOR_REVISION)
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        bytes,
        "loading does not rewrite"
    );
    let selected = &labs()[0];
    let others: Vec<_> = loaded
        .attempts
        .iter()
        .filter(|(id, _)| *id != &selected.id)
        .map(|(id, a)| (id.clone(), serde_json::to_vec(a).unwrap()))
        .collect();
    loaded
        .attempts
        .insert(selected.id.clone(), attempt(selected));
    for (id, bytes) in others {
        assert_eq!(serde_json::to_vec(&loaded.attempts[&id]).unwrap(), bytes);
    }
    for s in labs() {
        loaded.attempts.remove(&s.id);
    }
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), original);
}

#[test]
fn authored_graph_and_practice_identity_pins_reject_coordinated_rewrites_and_cross_lab_drafts() {
    let all = catalog();
    let new = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &new {
        p.attempts.insert(s.id.clone(), attempt(s));
    }
    let encoded = serde_json::to_value(&p).unwrap();
    for s in &new {
        let mut bad = encoded.clone();
        let first = simulation(&s.initial)
            .topology()
            .devices
            .keys()
            .next()
            .unwrap();
        for base in [
            "state/simulation/topology",
            "workspace/discovery_draft/identity",
        ] {
            let path = format!("/attempts/{}/{base}/devices/{first}/chassis_id", s.id);
            *bad.pointer_mut(&path).unwrap() = serde_json::json!("foreign-chassis");
        }
        bad["attempts"][&s.id]["state"]["simulation"]["cache"] = serde_json::json!([]);
        assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
        let mut bad = encoded.clone();
        let first = s
            .discovery_teaching
            .as_ref()
            .unwrap()
            .fixture
            .topology()
            .devices
            .keys()
            .next()
            .unwrap();
        for base in ["simulation/topology", "draft/identity"] {
            let path = format!(
                "/attempts/{}/workspace/discovery_learning/{base}/devices/{first}/chassis_id",
                s.id
            );
            *bad.pointer_mut(&path).unwrap() = serde_json::json!("foreign-practice-chassis");
        }
        bad["attempts"][&s.id]["workspace"]["discovery_learning"]["simulation"]["cache"] =
            serde_json::json!([]);
        assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
    }
    let mut bad = encoded.clone();
    bad["attempts"][&new[0].id]["workspace"]["discovery_draft"] =
        encoded["attempts"][&new[1].id]["workspace"]["discovery_draft"].clone();
    assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
}

#[test]
fn wrong_peer_goal_identity_unsupported_actions_and_unreachable_examples_fail_actual_pack_validation()
 {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/discovery-v1.json")).unwrap();
    for kind in 0..4 {
        let mut bad = source.clone();
        match kind {
            0 => {
                bad["scenarios"][0]["goal"]["requirements"][0]["peer"] =
                    bad["scenarios"][0]["goal"]["requirements"][0]["local"].clone()
            }
            1 => {
                bad["scenarios"][0]["goal"]["initial"]["devices"]["access"]["chassis_id"] =
                    serde_json::json!("foreign-chassis")
            }
            2 => bad["scenarios"][0]["solution"] = serde_json::json!(["show lldp neighbors"]),
            _ => {
                bad["scenarios"][0]["discovery_teaching"]["example_solution"] =
                    serde_json::json!([{"kind":"advance","seconds":1}])
            }
        }
        assert!(pack::load(&bad.to_string()).is_err(), "case {kind}");
    }
}
