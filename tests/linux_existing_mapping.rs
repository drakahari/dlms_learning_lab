//! Current registration plus exact old-source retention behind the mapping correction.
use dlms_learning_lab::{engine, pack};
#[test]
fn existing_driver_and_filesystem_assignments_retain_exact_sources_and_taught_outcomes() {
    let current = pack::current().unwrap();
    // Validate the complete retained catalog, including each lab's prerequisites.
    let ids = [
        "lpic1-module-device-driver",
        "lpic1-module-index-repair",
        "lpic2-module-options-reload",
        "lpic1-fs-offline-accounting",
        "lpic1-fs-retained-recovery",
    ];
    let prior = pack::accepted303()
        .unwrap()
        .into_iter()
        .filter(|s| ids.contains(&s.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(prior.len(), 5);
    for old in prior {
        let actual = current.iter().find(|s| s.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(&old).unwrap(),
            "{}",
            old.id
        );
        assert!(!engine::solved(&actual.initial, &actual.goal));
        let mut state = actual.initial.clone();
        for action in &actual.solution {
            engine::execute_for_goal(&mut state, &actual.goal, action).unwrap();
        }
        assert!(engine::solved(&state, &actual.goal), "{}", old.id);
    }
}
