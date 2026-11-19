use dlms_learning_lab::{
    course,
    engine::{self, State},
    json_inventory_workspace as work, pack, progress,
};
use serde_json::{Value, json};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted394().unwrap())
}
fn lessons() -> Vec<&'static pack::Scenario> {
    catalog()
        .iter()
        .filter(|s| matches!(s.initial, State::JsonInventory { .. }))
        .collect()
}
fn document(task: &work::Task) -> Value {
    serde_json::from_str(&task.reference).unwrap()
}
fn text(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap()
}
fn run(draft: &mut work::Draft, task: &work::Task, source: String) {
    draft.edit(source).unwrap();
    draft.apply(task).unwrap();
    draft.test(task).unwrap();
}
fn passed(draft: &work::Draft, task: &work::Task) -> bool {
    draft.checks(task).iter().all(|c| c.0)
}

#[test]
fn authored_repairs_and_independent_examples_reach_actual_network_outcomes() {
    assert_eq!(lessons().len(), 2);
    for scenario in lessons() {
        let definition = work::source(scenario).unwrap();
        let mut session = work::Session::new(definition);
        assert!(
            session
                .assignment
                .apply(&definition.initial)
                .unwrap_err()
                .contains("Document field error")
        );
        assert!(!passed(&session.assignment, &definition.initial));
        let assignment = session.assignment.clone();
        run(
            &mut session.practice,
            &definition.example,
            definition.example.reference.clone(),
        );
        assert!(passed(&session.practice, &definition.example));
        assert_eq!(session.assignment, assignment);
        assert!(!definition.checks(&session).iter().all(|c| c.0));
        let mut state = scenario.initial.clone();
        engine::execute_for_goal(&mut state, &scenario.goal, &scenario.solution[0]).unwrap();
        assert!(engine::solved(&state, &scenario.goal));
    }
}

#[test]
fn alternatives_reordered_records_and_swapped_groups_pass_but_down_ports_do_not() {
    for scenario in lessons() {
        let task = &work::source(scenario).unwrap().initial;
        let mut doc = document(task);
        let switches = doc["switches"].as_array_mut().unwrap();
        switches.reverse();
        for switch in switches {
            let ports = switch["ports"].as_array_mut().unwrap();
            ports.reverse();
            for port in ports {
                match port["vlan"].as_u64().unwrap() {
                    10 => port["vlan"] = json!(20),
                    20 => port["vlan"] = json!(10),
                    _ => {}
                }
            }
        }
        doc["note"] = json!("equivalent group allocation");
        let mut draft = work::Draft::new(task);
        run(&mut draft, task, text(&doc));
        assert!(passed(&draft, task));
        let before = draft.applied.clone();
        doc["switches"][0]["ports"][0]["up"] = json!(false);
        draft.edit(text(&doc)).unwrap();
        assert!(!draft.checked && !passed(&draft, task));
        assert_eq!(draft.applied, before);
        assert!(draft.test(task).unwrap_err().contains("Apply"));
        draft.apply(task).unwrap();
        draft.test(task).unwrap();
        assert!(!passed(&draft, task));
        assert!(
            draft
                .checks(task)
                .iter()
                .any(|(ok, message)| !ok && message.contains("down"))
        );
    }
}

#[test]
fn type_syntax_and_behavior_failures_remain_distinct_and_failed_apply_is_atomic() {
    for scenario in lessons() {
        let task = &work::source(scenario).unwrap().initial;
        let mut draft = work::Draft::new(task);
        let original = draft.applied.clone();
        for (source, diagnostic) in [
            (task.starter.clone(), "Document field error"),
            ("{ \"switches\": [".into(), "JSON syntax"),
        ] {
            draft.edit(source).unwrap();
            assert!(draft.apply(task).unwrap_err().contains(diagnostic));
            assert_eq!(draft.applied, original);
            assert!(!draft.checked);
        }
        // A well-typed but wrong inventory must apply, then receive behavior feedback.
        run(&mut draft, task, task.initial_applied.clone());
        let checks = draft.checks(task);
        assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
        let mut all_down = document(task);
        for sw in all_down["switches"].as_array_mut().unwrap() {
            for port in sw["ports"].as_array_mut().unwrap() {
                port["up"] = json!(false);
            }
        }
        run(&mut draft, task, text(&all_down));
        assert!(draft.checks(task).iter().all(|c| !c.0));
    }
}

#[test]
fn valid_json_with_mixed_groups_or_wrong_monitoring_vlan_cannot_receive_credit() {
    let scenario = lessons()[1];
    let task = &work::source(scenario).unwrap().initial;
    let mut draft = work::Draft::new(task);
    let mut doc = document(task);
    doc["switches"][1]["ports"][0]["vlan"] = json!(20);
    run(&mut draft, task, text(&doc));
    assert!(!passed(&draft, task));
    let checks = draft.checks(task);
    assert_eq!(checks.len(), 8);
    assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
    let mut doc = document(task);
    for sw in doc["switches"].as_array_mut().unwrap() {
        sw["ports"][2]["vlan"] = json!(10);
    }
    run(&mut draft, task, text(&doc));
    assert!(!passed(&draft, task));
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(ok, message)| !ok && message.contains("99"))
    );
}

#[test]
fn source_binding_reachability_and_saved_data_limits_are_enforced() {
    let scenario = lessons()[0];
    let def = work::source(scenario).unwrap();
    let mut bad = def.clone();
    bad.initial.reference = bad.initial.initial_applied.clone();
    assert!(bad.validate().is_err());
    let mut bad = def.clone();
    bad.initial.requirements.clear();
    assert!(bad.validate().is_err());
    let mut foreign = progress::Attempt::new(scenario, 1);
    let State::JsonInventory { simulation } = &mut foreign.state else {
        panic!()
    };
    simulation.context = "another-lesson".into();
    assert!(work::context(&foreign, scenario).is_err());
    assert!(!progress::current_solved(&foreign, scenario));
    let mut value = serde_json::to_value(work::Session::new(def)).unwrap();
    value["assignment"]["source"] = json!("x".repeat(8193));
    assert!(serde_json::from_value::<work::Session>(value).is_err());
    let mut draft = work::Draft::new(&def.initial);
    let before = draft.clone();
    assert!(draft.edit("x".repeat(8193)).is_err());
    assert_eq!(draft, before);
    let mut wrong = scenario.clone();
    wrong.objective = "A different task".into();
    assert!(work::source(&wrong).is_err());
}

#[test]
fn saved_work_reset_and_upgrade_preserve_drafts_favorites_and_existing_credit() {
    let old = pack::accepted392().unwrap();
    let current = catalog();
    assert_eq!((old.len(), current.len()), (392, 394));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..392]).unwrap()
    );
    let old_units = course::accepted392(&old).unwrap();
    let units = course::accepted394(current).unwrap();
    assert_eq!((old_units.len(), units.len()), (150, 151));
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "structured-network-inventory")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "structured-network-inventory")
        .unwrap();
    assert_eq!(units[at - 1].id, "vlan-data-foundations");
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    let retained = &old[0];
    let mut attempt = progress::Attempt::new(retained, 1);
    for command in &retained.solution {
        engine::execute_for_goal(&mut attempt.state, &retained.goal, command).unwrap();
    }
    assert!(progress::current_solved(&attempt, retained));
    profile.attempts.insert(retained.id.clone(), attempt);
    profile.favorites.insert(retained.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &profile, &old).unwrap();
    profile = progress::load(&path, current).unwrap();
    assert_eq!(profile.attempts.len(), 1);
    assert!(progress::current_solved(
        &profile.attempts[&retained.id],
        retained
    ));
    for scenario in lessons() {
        let def = work::source(scenario).unwrap();
        let mut attempt = progress::Attempt::new(scenario, 2);
        let State::JsonInventory { simulation } = &mut attempt.state else {
            panic!()
        };
        run(
            &mut simulation.assignment,
            &def.initial,
            def.initial.reference.clone(),
        );
        simulation
            .practice
            .edit("{ unfinished practice".into())
            .unwrap();
        assert!(progress::current_solved(&attempt, scenario));
        profile.attempts.insert(scenario.id.clone(), attempt);
    }
    progress::save_with_catalog(&path, &profile, current).unwrap();
    let mut loaded = progress::load(&path, current).unwrap();
    assert_eq!(
        serde_json::to_value(&profile).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    let second = lessons()[1];
    let keep = serde_json::to_value(&loaded.attempts[&second.id]).unwrap();
    let first = lessons()[0];
    loaded
        .attempts
        .insert(first.id.clone(), progress::Attempt::new(first, 3));
    assert!(!progress::current_solved(
        &loaded.attempts[&first.id],
        first
    ));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&second.id]).unwrap(),
        keep
    );
}
