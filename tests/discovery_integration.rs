//! Private native-integration fixture tests; this fixture is not bundled curriculum.
use dlms_learning_lab::{
    discovery::{self, Check},
    discovery_teaching::Learning,
    discovery_workspace::{Action, Draft},
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};

fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/discovery-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn catalog() -> Vec<pack::Scenario> {
    let mut all: Vec<_> = pack::bundled().unwrap().into_iter().take(78).collect();
    all.push(scenario());
    all
}
fn simulation(state: &State) -> &discovery::State {
    let State::Discovery { simulation } = state else {
        panic!("Discovery fixture expected")
    };
    simulation
}
fn simulation_mut(state: &mut State) -> &mut discovery::State {
    let State::Discovery { simulation } = state else {
        panic!("Discovery fixture expected")
    };
    simulation
}
fn command(action: &Action) -> String {
    format!("discovery {}", serde_json::to_string(action).unwrap())
}
fn endpoint(device: &str) -> discovery::Endpoint {
    discovery::Endpoint {
        device: device.into(),
        port: "uplink".into(),
    }
}
fn advertise(device: &str) -> Action {
    Action::Advertise {
        from: endpoint(device),
    }
}
fn attempt(lab: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(lab, 1);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().discovery_learning =
        Some(Learning::new(lab.discovery_teaching.as_ref().unwrap()));
    a
}
fn apply_draft(a: &mut Attempt) -> Result<(), String> {
    let draft = a
        .workspace
        .as_ref()
        .unwrap()
        .discovery_draft
        .as_ref()
        .unwrap()
        .clone();
    draft.apply(simulation_mut(&mut a.state))
}
fn native_repair(a: &mut Attempt) {
    a.workspace
        .as_mut()
        .unwrap()
        .discovery_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("birch")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .receive = true;
    apply_draft(a).unwrap();
}
fn native_solved(lab: &pack::Scenario) -> Attempt {
    let mut a = attempt(lab);
    native_repair(&mut a);
    for device in ["cedar", "birch"] {
        engine::execute(&mut a.state, &command(&advertise(device))).unwrap();
    }
    assert!(progress::current_solved(&a, lab));
    a
}
fn reference(lab: &pack::Scenario) -> State {
    let mut state = lab.initial.clone();
    for input in &lab.solution {
        engine::execute(&mut state, input).unwrap();
    }
    state
}

#[test]
fn private_pack_reference_alternate_orders_and_refreshes_reach_both_current_neighbors() {
    let lab = scenario();
    assert!(!engine::solved(&lab.initial, &lab.goal));
    let good = reference(&lab);
    assert!(engine::solved(&good, &lab.goal));
    for inputs in [
        vec![
            lab.solution[0].clone(),
            lab.solution[2].clone(),
            lab.solution[1].clone(),
        ],
        vec![
            lab.solution[1].clone(),
            lab.solution[2].clone(),
            lab.solution[0].clone(),
            lab.solution[1].clone(),
            lab.solution[2].clone(),
        ],
    ] {
        let mut state = lab.initial.clone();
        for input in inputs {
            engine::execute(&mut state, &input).unwrap();
        }
        assert!(engine::solved(&state, &lab.goal));
        engine::execute(&mut state, &command(&Action::Advance { seconds: 29 })).unwrap();
        assert!(engine::solved(&state, &lab.goal));
        for device in ["birch", "cedar"] {
            engine::execute(&mut state, &command(&advertise(device))).unwrap();
        }
        engine::execute(&mut state, &command(&Action::Advance { seconds: 29 })).unwrap();
        assert!(engine::solved(&state, &lab.goal));
        engine::execute(&mut state, &command(&Action::Advance { seconds: 1 })).unwrap();
        assert!(!engine::solved(&state, &lab.goal));
    }
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 82);
    assert!(!all.iter().any(|s| s.id == lab.id));
    assert!(engine::help(&lab.initial).contains("no CDP"));
}

#[test]
fn one_way_partial_config_and_positive_cache_cannot_replace_full_current_outcomes() {
    let lab = scenario();
    let mut partial = lab.initial.clone();
    for device in ["cedar", "birch"] {
        engine::execute(&mut partial, &command(&advertise(device))).unwrap();
    }
    assert!(
        simulation(&partial)
            .neighbor(&endpoint("cedar"))
            .unwrap()
            .is_some()
    );
    assert!(
        simulation(&partial)
            .neighbor(&endpoint("birch"))
            .unwrap()
            .is_none()
    );
    assert!(!engine::solved(&partial, &lab.goal));
    engine::execute(&mut partial, &lab.solution[0]).unwrap();
    assert!(!engine::solved(&partial, &lab.goal));
    let mut only_one = lab.initial.clone();
    for input in &lab.solution[..2] {
        engine::execute(&mut only_one, input).unwrap();
    }
    assert!(!engine::solved(&only_one, &lab.goal));
    let good = reference(&lab);
    let mut historical = serde_json::to_value(simulation(&good)).unwrap();
    let cache = historical["cache"].as_array_mut().unwrap();
    let stale = cache
        .iter_mut()
        .find(|n| n["local"]["device"] == "birch")
        .unwrap();
    stale["advertisement"]["system_name"] = serde_json::json!("Former Cedar");
    let stale = discovery::State::from_json(&historical.to_string()).unwrap();
    assert!(
        stale
            .neighbor(&endpoint("birch"))
            .unwrap()
            .unwrap()
            .remaining_seconds
            > 0
    );
    let stale = State::Discovery { simulation: stale };
    let rows = engine::checks(&stale, &lab.goal);
    assert!(!engine::solved(&stale, &lab.goal));
    assert!(
        rows.iter()
            .any(|(passed, message)| !passed && message.contains("historical"))
    );
    assert!(
        rows.last().unwrap().0,
        "full configuration is still exactly correct"
    );
    for kind in 0..3 {
        let mut wrong = good.clone();
        let mut t = simulation(&wrong).topology().clone();
        match kind {
            0 => {
                t.devices
                    .get_mut("cedar")
                    .unwrap()
                    .ports
                    .get_mut("spare")
                    .unwrap()
                    .transmit = true
            }
            1 => t.devices.get_mut("cedar").unwrap().ttl = 31,
            _ => t.links[0].up = false,
        }
        simulation_mut(&mut wrong).reconfigure(t).unwrap();
        assert!(!engine::solved(&wrong, &lab.goal));
        assert!(!engine::checks(&wrong, &lab.goal).last().unwrap().0);
    }
}

#[test]
fn native_drafts_invalidate_success_and_invalid_ttl_is_atomic_until_explicit_refresh() {
    let lab = scenario();
    let mut a = native_solved(&lab);
    a.workspace
        .as_mut()
        .unwrap()
        .discovery_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("cedar")
        .unwrap()
        .ttl = "31".into();
    assert!(engine::solved(&a.state, &lab.goal));
    assert!(!progress::current_solved(&a, &lab));
    a.workspace
        .as_mut()
        .unwrap()
        .discovery_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("cedar")
        .unwrap()
        .ttl = "bad".into();
    let before = a.state.clone();
    assert!(apply_draft(&mut a).is_err());
    assert_eq!(a.state, before);
    assert!(!progress::current_solved(&a, &lab));
    a.workspace
        .as_mut()
        .unwrap()
        .discovery_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("cedar")
        .unwrap()
        .ttl = "31".into();
    apply_draft(&mut a).unwrap();
    assert!(!progress::current_solved(&a, &lab));
    engine::execute(&mut a.state, &command(&advertise("cedar"))).unwrap();
    assert!(
        !progress::current_solved(&a, &lab),
        "refreshed but wrong TTL cannot replace the full configuration guard"
    );
    a.workspace
        .as_mut()
        .unwrap()
        .discovery_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("cedar")
        .unwrap()
        .ttl = "30".into();
    apply_draft(&mut a).unwrap();
    assert!(!progress::current_solved(&a, &lab));
    engine::execute(&mut a.state, &command(&advertise("cedar"))).unwrap();
    assert!(progress::current_solved(&a, &lab));
}

#[test]
fn separate_practice_fill_apply_events_and_try_change_do_not_touch_assignment_or_credit() {
    let lab = scenario();
    let lesson = lab.discovery_teaching.as_ref().unwrap();
    let mut a = attempt(&lab);
    let initial = a.state.clone();
    let assignment_draft = a.workspace.as_ref().unwrap().discovery_draft.clone();
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .discovery_learning
        .as_mut()
        .unwrap();
    learning.view = View::Example;
    learning.draft.devices.get_mut("demo-oak").unwrap().ttl = "unfinished".into();
    let practice_before = learning.simulation.clone();
    learning.fill_example(lesson).unwrap();
    assert_eq!(learning.simulation, practice_before);
    assert!(!learning.checked);
    learning.apply().unwrap();
    assert!(
        learning
            .simulation
            .neighbors("demo-maple")
            .unwrap()
            .is_empty()
    );
    for action in lesson
        .example_solution
        .iter()
        .filter(|action| !action.is_configuration())
    {
        learning.execute(action).unwrap();
    }
    assert!(learning.check(lesson).unwrap().iter().all(|r| r.passed()));
    for action in &lesson.try_change.actions {
        learning.execute(action).unwrap();
    }
    assert!(lesson.try_change.outcomes.iter().all(|r| {
        r.check(lesson.fixture.topology(), &learning.simulation)
            .is_ok_and(Check::passed)
    }));
    assert!(learning.check(lesson).unwrap().iter().any(|r| !r.passed()));
    assert_eq!(a.state, initial);
    assert_eq!(
        a.workspace.as_ref().unwrap().discovery_draft,
        assignment_draft
    );
    assert!(!progress::current_solved(&a, &lab));
}

#[test]
fn all_78_existing_attempts_remain_exact_while_new_unsaved_drafts_resume_and_reset_independently() {
    let all = catalog();
    let lab = all.last().unwrap();
    assert_eq!(all.len(), 79);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for old in all.iter().take(78) {
        p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    }
    let original = serde_json::to_vec(&p).unwrap();
    let mut a = native_solved(lab);
    a.workspace
        .as_mut()
        .unwrap()
        .discovery_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("birch")
        .unwrap()
        .ttl = "unapplied invalid TTL".into();
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .discovery_learning
        .as_mut()
        .unwrap();
    learning.view = View::Assignment;
    learning
        .draft
        .devices
        .get_mut("demo-maple")
        .unwrap()
        .system_name = Some("unapplied name".into());
    learning.draft.devices.get_mut("demo-oak").unwrap().ttl = "incomplete".into();
    p.attempts.insert(lab.id.clone(), a);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress-v1.json");
    progress::save(&path, &p).unwrap();
    let encoded = std::fs::read(&path).unwrap();
    assert_eq!(encoded, serde_json::to_vec(&p).unwrap());
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), encoded);
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], lab));
    let saved = &loaded.attempts[&lab.id];
    assert_eq!(
        saved
            .workspace
            .as_ref()
            .unwrap()
            .discovery_draft
            .as_ref()
            .unwrap()
            .devices["birch"]
            .ttl,
        "unapplied invalid TTL"
    );
    assert_eq!(
        saved
            .workspace
            .as_ref()
            .unwrap()
            .discovery_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    loaded.attempts.insert(lab.id.clone(), attempt(lab));
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], lab));
    loaded.attempts.remove(&lab.id);
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), original);
}

#[test]
fn malformed_goal_graph_example_and_unsolved_reference_are_rejected_by_actual_pack_validation() {
    let original = serde_json::to_value(scenario()).unwrap();
    for kind in 0..6 {
        let mut invalid = original.clone();
        match kind {
            0 => {
                invalid["goal"]["initial"]["devices"]["cedar"]["chassis_id"] =
                    serde_json::json!("foreign")
            }
            1 => {
                invalid["goal"]["requirements"][0]["peer"]["device"] = serde_json::json!("missing")
            }
            2 => invalid["solution"] = serde_json::json!([original["solution"][0].clone()]),
            3 => {
                invalid["discovery_teaching"]["fixture"] = original["initial"]["simulation"].clone()
            }
            4 => {
                invalid["discovery_teaching"]["example_solution"] =
                    serde_json::json!([{"kind":"advance","seconds":1}])
            }
            _ => invalid["solution"] = serde_json::json!(["show lldp neighbors"]),
        }
        let pack = serde_json::json!({"schema_version":1,"scenarios":[invalid]});
        assert!(pack::load(&pack.to_string()).is_err(), "case {kind}");
    }
    let mut invalid = original.clone();
    let requirement = invalid["goal"]["requirements"][0].clone();
    invalid["goal"]["requirements"] = serde_json::json!(vec![requirement; 33]);
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[invalid]}).to_string())
            .is_err()
    );
}

#[test]
fn imported_simulation_and_draft_identities_cannot_be_rewritten_or_cross_activities() {
    let all = catalog();
    let lab = all.last().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), attempt(lab));
    let original = serde_json::to_value(&p).unwrap();
    let root = format!("/attempts/{}", lab.id);
    for pointer in [
        format!("{root}/state/simulation/topology/devices/cedar/chassis_id"),
        format!("{root}/workspace/discovery_draft/identity/devices/cedar/chassis_id"),
        format!(
            "{root}/workspace/discovery_learning/simulation/topology/devices/demo-maple/chassis_id"
        ),
        format!("{root}/workspace/discovery_learning/draft/identity/devices/demo-maple/chassis_id"),
    ] {
        let mut invalid = original.clone();
        *invalid.pointer_mut(&pointer).unwrap() = serde_json::json!("foreign");
        assert!(progress::decode(&serde_json::to_vec(&invalid).unwrap(), &all).is_err());
    }
    let mut invalid = original.clone();
    for middle in [
        "state/simulation/topology",
        "workspace/discovery_draft/identity",
    ] {
        let pointer = format!("{root}/{middle}/devices/cedar/chassis_id");
        *invalid.pointer_mut(&pointer).unwrap() = serde_json::json!("foreign");
    }
    assert!(progress::decode(&serde_json::to_vec(&invalid).unwrap(), &all).is_err());
    let mut invalid = original.clone();
    for middle in ["simulation/topology", "draft/identity"] {
        let pointer =
            format!("{root}/workspace/discovery_learning/{middle}/devices/demo-maple/chassis_id");
        *invalid.pointer_mut(&pointer).unwrap() = serde_json::json!("foreign");
    }
    assert!(progress::decode(&serde_json::to_vec(&invalid).unwrap(), &all).is_err());
    let mut invalid = original.clone();
    invalid["attempts"][&lab.id]["workspace"]["discovery_draft"] =
        original["attempts"][&lab.id]["workspace"]["discovery_learning"]["draft"].clone();
    assert!(progress::decode(&serde_json::to_vec(&invalid).unwrap(), &all).is_err());
    let old = &all[0];
    let mut wrong = Attempt::new(old, 1);
    workspace::prepare(&mut wrong);
    wrong.workspace.as_mut().unwrap().discovery_learning =
        Some(Learning::new(lab.discovery_teaching.as_ref().unwrap()));
    let mut invalid = Progress {
        schema_version: 1,
        ..Default::default()
    };
    invalid.attempts.insert(old.id.clone(), wrong);
    assert!(progress::decode(&serde_json::to_vec(&invalid).unwrap(), &all).is_err());
}

#[test]
fn invalid_save_side_activity_or_draft_metadata_preserves_existing_progress_bytes() {
    let all = catalog();
    let lab = all.last().unwrap();
    let old = &all[0];
    for discovery_activity in [false, true] {
        let selected = if discovery_activity { lab } else { old };
        let mut a = if discovery_activity {
            attempt(lab)
        } else {
            Attempt::new(old, 1)
        };
        workspace::prepare(&mut a);
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(selected.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress-v1.json");
        progress::save(&path, &p).unwrap();
        let before = std::fs::read(&path).unwrap();
        let w = p
            .attempts
            .get_mut(&selected.id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if discovery_activity {
            w.discovery_draft = Some(Draft::new(
                &lab.discovery_teaching.as_ref().unwrap().fixture,
            ));
        } else {
            w.discovery_learning = Some(Learning::new(lab.discovery_teaching.as_ref().unwrap()));
        }
        assert!(progress::save(&path, &p).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(progress::decode(&before, &all).is_ok());
    }
}

#[test]
fn wrong_live_topology_identity_cannot_retain_current_success_even_without_editor_metadata() {
    let lab = scenario();
    let mut good = Attempt::new(&lab, 1);
    good.state = reference(&lab);
    assert!(progress::current_solved(&good, &lab));
    let mut t = simulation(&good.state).topology().clone();
    t.devices.get_mut("cedar").unwrap().chassis_id = "different-chassis".into();
    good.state = State::Discovery {
        simulation: discovery::State::new(t).unwrap(),
    };
    assert!(!engine::solved(&good.state, &lab.goal));
    assert!(!progress::current_solved(&good, &lab));
    let Goal::Discovery {
        initial,
        requirements,
    } = &lab.goal
    else {
        panic!()
    };
    assert!(requirements.iter().all(|r| r.validate(initial).is_ok()));
}

#[test]
fn native_discovery_routing_requires_validated_teaching_before_a_pack_is_accepted() {
    let mut raw: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/discovery-workspace-preview.json")).unwrap();
    raw["scenarios"][0]
        .as_object_mut()
        .unwrap()
        .remove("discovery_teaching");
    let error = pack::load(&raw.to_string()).unwrap_err();
    assert!(
        error.contains("require validated introduction and practice"),
        "{error}"
    );
}
