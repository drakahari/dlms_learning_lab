use dlms_learning_lab::{
    course,
    engine::{self, State},
    network_architecture_source as binding, pack, progress,
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("../scenarios/leaf-spine-lessons-v1.json"))
        .unwrap()
        .remove(0)
}
#[test]
fn every_previous_scenario_and_unit_and_order_are_preserved_exactly() {
    let old = pack::accepted454().unwrap();
    let now = pack::accepted455().unwrap();
    assert_eq!((old.len(), now.len()), (454, 455));
    let kept = now
        .iter()
        .filter(|s| s.id != "ccna-leaf-spine-rack-expansion")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = course::accepted454(&old).unwrap();
    let units = course::accepted455(&now).unwrap();
    assert_eq!((before.len(), units.len()), (180, 181));
    let kept = units
        .iter()
        .filter(|u| u.id != "ccna-leaf-spine-fabrics")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ccna-leaf-spine-fabrics")
        .unwrap();
    assert_eq!(units[at - 1].id, "ccna-campus-architectures");
    assert_eq!(now.iter().filter(|s| s.track == "CCNA").count(), 110);
}
#[test]
fn old_campus_definition_and_session_json_have_no_new_optional_fields() {
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/campus-architecture-lessons-v1.json"
    ))
    .unwrap();
    for s in raw["scenarios"].as_array().unwrap() {
        let d: dlms_learning_lab::network_architecture_workspace::Definition =
            serde_json::from_value(s["goal"]["definition"].clone()).unwrap();
        assert_eq!(serde_json::to_value(&d).unwrap(), s["goal"]["definition"]);
        assert_eq!(
            serde_json::to_value(d.new_session().unwrap()).unwrap(),
            s["initial"]["simulation"]
        );
    }
}
#[test]
fn separate_practice_partial_work_wrong_shortcuts_alternatives_and_resume_use_actual_state() {
    let s = scenario();
    let d = binding::source(&s).unwrap();
    let mut state = s.initial.clone();
    let State::CampusArchitecture { simulation } = &mut state else {
        panic!()
    };
    simulation
        .practice
        .edit(&d.example, d.example.reference.clone())
        .unwrap();
    simulation.practice.check(&d.example).unwrap();
    assert!(simulation.practice.solved(&d.example));
    assert!(!simulation.assignment.solved(&d.assignment));
    let mut partial = d.assignment.initial.clone();
    partial.connections[9] = d.assignment.reference.connections[9].clone();
    simulation
        .assignment
        .edit(&d.assignment, partial.clone())
        .unwrap();
    simulation.assignment.check(&d.assignment).unwrap();
    assert!(!simulation.assignment.solved(&d.assignment));
    assert!(
        d.assignment
            .checks(&partial)
            .iter()
            .any(|c| !c.passed && c.detail.contains("1 distinct internal spine alternatives"))
    );
    partial.connections[10] = d.assignment.reference.connections[10].clone();
    simulation.assignment.edit(&d.assignment, partial).unwrap();
    simulation.assignment.check(&d.assignment).unwrap();
    assert!(!simulation.assignment.solved(&d.assignment));
    // Every port on Rack4 is equivalent if its other end still reaches a different spine.
    let mut alternative = d.assignment.reference.clone();
    for c in &mut alternative.connections[9..] {
        let j = c.a.as_mut().unwrap();
        j.port = match j.port.as_str() {
            "p1" => "p3",
            "p2" => "p1",
            _ => "p2",
        }
        .into();
        std::mem::swap(&mut c.a, &mut c.b);
    }
    alternative.connections.reverse();
    simulation
        .assignment
        .edit(&d.assignment, alternative)
        .unwrap();
    simulation.assignment.check(&d.assignment).unwrap();
    assert!(simulation.assignment.solved(&d.assignment));
    let resumed: dlms_learning_lab::network_architecture_workspace::Session =
        serde_json::from_slice(&serde_json::to_vec(&simulation).unwrap()).unwrap();
    d.context(&resumed).unwrap();
    assert_eq!(**simulation, resumed);
    simulation
        .assignment
        .edit(&d.assignment, d.assignment.initial.clone())
        .unwrap();
    assert!(!simulation.assignment.checked);
    assert!(!simulation.assignment.solved(&d.assignment));
    let mut attempt = progress::Attempt::new(&s, 0);
    attempt.state = state;
    binding::context(&attempt, &s).unwrap();
    let mut repaired = s.initial.clone();
    for action in &s.solution {
        engine::execute_for_goal(&mut repaired, &s.goal, action).unwrap();
    }
    assert!(engine::solved(&repaired, &s.goal));
    let State::CampusArchitecture { simulation } = &mut repaired else {
        panic!()
    };
    let mut changed = simulation.assignment.draft.clone();
    changed.connections[0].b = None;
    assert!(simulation.assignment.edit(&d.assignment, changed).is_err());
    assert!(simulation.assignment.solved(&d.assignment));
}
#[test]
fn authoring_and_consolidation_are_exact_and_stay_within_existing_pack_limit() {
    let a: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/authoring/leaf-spine-v1.json")).unwrap();
    let b: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/leaf-spine-lessons-v1.json")).unwrap();
    assert_eq!(a, b);
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v41.json")).unwrap();
    let now: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v42.json")).unwrap();
    let expected = old["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .chain(b["scenarios"].as_array().unwrap())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(now["scenarios"], serde_json::json!(expected));
    assert!(include_str!("../scenarios/boot-entry-lessons-v42.json").len() < pack::MAX_PACK_BYTES);
}
