//! The new authored source is deliberately unregistered at its first gate.
use dlms_learning_lab::{engine, pack, pam_workspace as w, progress::Attempt, workspace};
fn lab() -> pack::Scenario {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/pam-auth-lessons-v1.json")).unwrap();
    serde_json::from_value(raw["scenarios"][0].clone()).unwrap()
}
#[test]
fn authored_source_teaches_a_reachable_independent_repair_and_separate_practice() {
    let s = lab();
    let d = w::source(&s).unwrap();
    assert_eq!(d.initial.cases.len(), 5);
    assert_eq!(d.example.cases.len(), 5);
    assert_eq!(
        d.initial
            .run(&d.initial.starter)
            .unwrap()
            .iter()
            .filter(|r| r.passed())
            .count(),
        1
    );
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    w::context(&a, &s).unwrap();
    assert!(!engine::solved(&a.state, &s.goal));
    for step in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
    }
    assert!(engine::solved(&a.state, &s.goal));
    let engine::State::PamStack { simulation } = &mut a.state else {
        panic!()
    };
    assert_eq!(simulation.practice.draft, d.example.starter);
    simulation
        .assignment
        .edit("auth required pam_permit.so")
        .unwrap();
    assert!(!d.solved(simulation));
    simulation.assignment.run(&d.initial).unwrap();
    assert!(!d.solved(simulation));
    for alternative in [
        "auth required pam_unix.so",
        "auth sufficient pam_unix.so\nauth required pam_deny.so",
    ] {
        simulation.assignment.edit(alternative).unwrap();
        simulation.assignment.run(&d.initial).unwrap();
        assert!(d.solved(simulation));
    }
}
#[test]
fn sample_errors_do_not_count_as_rejection_and_bound_context_rejects_foreign_data() {
    let s = lab();
    let d = w::source(&s).unwrap();
    let mut a = Attempt::new(&s, 2);
    workspace::prepare(&mut a);
    let engine::State::PamStack { simulation } = &mut a.state else {
        panic!()
    };
    simulation.practice.edit(&d.example_worked).unwrap();
    simulation.practice.run(&d.example).unwrap();
    assert!(!d.solved(simulation));
    for input in [
        "auth",
        "auth required pam_unix.so nullok",
        "auth required /etc/pam.d/login",
    ] {
        simulation.assignment.edit(input).unwrap();
        let rows = simulation.assignment.run(&d.initial).unwrap();
        assert!(rows.iter().all(|r| !r.passed() && r.diagnostic.is_some()));
        assert!(!d.solved(simulation));
    }
    simulation.origin = "other-lesson".into();
    assert!(w::context(&a, &s).is_err());
    let mut wrong = s.clone();
    wrong.objective.push_str(" unsupported extra goal");
    assert!(w::source(&wrong).is_err());
}
