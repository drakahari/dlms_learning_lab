//! Standalone authored-source checks before catalog registration.
use dlms_learning_lab::{aging_workspace as w, engine, pack, progress::Attempt, workspace};
fn sources() -> Vec<pack::Scenario> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/aging-lessons-v1.json")).unwrap();
    serde_json::from_value(v["scenarios"].clone()).unwrap()
}
#[test]
fn three_substantive_tasks_have_reachable_taught_and_separate_practice_work() {
    let labs = sources();
    assert_eq!(labs.len(), 3);
    assert_eq!(
        labs.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        [
            "lpic1-aging-rotation",
            "lpic1-aging-grace",
            "lpic1-aging-contract-reset"
        ]
    );
    for s in &labs {
        let d = w::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        w::context(&a, s).unwrap();
        assert!(!engine::solved(&a.state, &s.goal));
        for step in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
        }
        assert!(engine::solved(&a.state, &s.goal), "{}", s.id);
        let engine::State::Aging { simulation } = &mut a.state else {
            panic!()
        };
        assert_eq!(simulation.practice.world, d.example.initial);
        let assignment = simulation.assignment.clone();
        simulation.fill_example(d).unwrap();
        simulation.practice.check(&d.example).unwrap();
        assert!(
            d.example
                .checks(&simulation.practice.world)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(simulation.assignment, assignment);
        let raw = serde_json::to_string(simulation).unwrap();
        assert_eq!(w::Session::from_json(&raw, d).unwrap(), **simulation);
    }
}
#[test]
fn source_identity_sample_boundaries_and_wrong_shortcuts_remain_checked() {
    for s in sources() {
        let d = w::source(&s).unwrap();
        let mut a = Attempt::new(&s, 2);
        workspace::prepare(&mut a);
        let engine::State::Aging { simulation } = &mut a.state else {
            panic!()
        };
        for cmd in [
            format!("chage -M -1 {}", d.initial.target),
            format!("chage -d -1 {}", d.initial.target),
            format!("chage -E -1 {}", d.initial.target),
        ] {
            simulation.assignment.world = d.initial.initial.clone();
            simulation.assignment.checked = false;
            simulation.assignment.edit_input(&cmd).unwrap();
            simulation.assignment.execute(&d.initial).unwrap();
            simulation.assignment.check(&d.initial).unwrap();
            assert!(!d.solved(simulation), "{}: {cmd}", s.id);
        }
        let saved = simulation.assignment.world.clone();
        simulation
            .assignment
            .edit_input(&format!("chage -M 45 --prefix=/ {}", d.initial.target))
            .unwrap();
        assert!(simulation.assignment.execute(&d.initial).is_err());
        assert_eq!(simulation.assignment.world, saved);
        simulation.origin = "foreign-task".into();
        assert!(w::context(&a, &s).is_err());
        let mut foreign = s.clone();
        foreign.objective.push_str(" Extra unsupported objective.");
        assert!(w::source(&foreign).is_err());
    }
}
