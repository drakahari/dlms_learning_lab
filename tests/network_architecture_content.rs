use dlms_learning_lab::{
    course,
    engine::{self, State},
    network_architecture_source as binding, pack,
};
const IDS: [&str; 2] = [
    "ccna-collapsed-campus-uplinks",
    "ccna-distribution-core-paths",
];
#[test]
fn catalog_retains_all_452_original_values_order_and_179_units_and_adds_only_two_authored_tasks() {
    let old = pack::accepted452().unwrap();
    let current = pack::accepted454().unwrap();
    assert_eq!((old.len(), current.len()), (452, 454));
    let retained: Vec<_> = current
        .iter()
        .filter(|s| old.iter().any(|o| o.id == s.id))
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        current
            .iter()
            .filter(|s| !old.iter().any(|o| o.id == s.id))
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let before = course::accepted452(&old).unwrap();
    let units = course::accepted454(&current).unwrap();
    assert_eq!((before.len(), units.len()), (179, 180));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| before.iter().any(|o| o.id == u.id))
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ccna-campus-architectures")
        .unwrap();
    assert_eq!(units[at - 1].id, "ccna-wired-device-paths");
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at].prerequisites, vec!["ccna-wired-device-paths"]);
    for (track, count) in [
        ("CCNA", 109),
        ("LPIC-1", 151),
        ("LPIC-2", 114),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn authored_starters_worked_examples_independent_solutions_and_partial_outages_are_real() {
    for s in pack::load(include_str!(
        "../scenarios/campus-architecture-lessons-v1.json"
    ))
    .unwrap()
    {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let State::CampusArchitecture { simulation } = &mut state else {
            panic!()
        };
        let untouched = simulation.assignment.clone();
        simulation.practice.check(&d.example).unwrap();
        assert!(!simulation.practice.solved(&d.example));
        simulation
            .practice
            .edit(&d.example, d.example.reference.clone())
            .unwrap();
        assert!(!simulation.practice.checked);
        simulation.practice.check(&d.example).unwrap();
        assert!(simulation.practice.solved(&d.example));
        assert_eq!(simulation.assignment, untouched);
        assert!(!engine::solved(&state, &s.goal));
        for event in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, event).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        let rows = d.assignment.checks(&d.assignment.initial);
        assert!(rows.iter().any(|r| r.passed));
        assert!(rows.iter().any(|r| !r.passed));
    }
}
#[test]
fn new_pack_is_exact_authoring_data_and_consolidation_keeps_every_predecessor_object() {
    let original: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/authoring/campus-architecture-v1.json"
    ))
    .unwrap();
    let added: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/campus-architecture-lessons-v1.json"
    ))
    .unwrap();
    assert_eq!(original, added);
    let before: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v40.json")).unwrap();
    let after: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v41.json")).unwrap();
    let expected = before["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .chain(added["scenarios"].as_array().unwrap())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(after["scenarios"], serde_json::json!(expected));
    assert!(include_str!("../scenarios/boot-entry-lessons-v41.json").len() < pack::MAX_PACK_BYTES);
}
