use dlms_learning_lab::{
    automation_workshop::{
        ospf_broadcast::workspace::Draft,
        source,
        workspace::{Action, Scope, Task, Work},
    },
    course, engine, pack, progress,
};

fn lesson() -> pack::Scenario {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/ccna-ospf-broadcast-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
    .remove(0)
}

#[test]
fn broadcast_handover_follows_the_installed_route_lesson_without_reordering_earlier_labs() {
    let before = pack::accepted542().unwrap();
    let current = pack::current().unwrap();
    assert_eq!((before.len(), current.len()), (542, 543));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..542]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(lesson()).unwrap(),
        serde_json::to_value(&current[542]).unwrap()
    );
    let before_units = course::accepted542(&before).unwrap();
    let units = course::current(&current).unwrap();
    let position = units
        .iter()
        .position(|u| u.id == "ospf-broadcast-designated-roles")
        .unwrap();
    assert_eq!(units[position - 1].id, "ipv4-installed-route-reading");
    let retained = units
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != position)
        .map(|(_, unit)| unit)
        .collect::<Vec<_>>();
    assert_eq!((before_units.len(), units.len()), (223, 224));
    assert_eq!(
        serde_json::to_value(before_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}

#[test]
fn assignment_checks_derived_role_changes_and_resumes_a_separate_practice_edit() {
    let lab = lesson();
    let definition = source::source(&lab).unwrap();
    let mut attempt = progress::Attempt::new(&lab, 1);
    let engine::State::AutomationWorkshop { simulation } = &mut attempt.state else {
        panic!()
    };
    let Task::OspfBroadcast { supplied } = &definition.example else {
        panic!()
    };
    let mut practice = supplied.start();
    practice.priorities[0] = "14x".into();
    simulation
        .example
        .work
        .apply(
            &definition.example,
            Action::ChangeOspfBroadcast {
                draft: practice.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        simulation.assignment.work,
        Work::new(&definition.assignment)
    );

    for line in &lab.solution {
        engine::execute_for_goal(&mut attempt.state, &lab.goal, line).unwrap();
    }
    assert!(engine::solved(&attempt.state, &lab.goal));
    let engine::State::AutomationWorkshop { simulation } = &mut attempt.state else {
        panic!()
    };
    let alternate = Draft {
        priorities: ["100", "100", "0"].map(str::to_owned),
    };
    simulation
        .assignment
        .work
        .apply(
            &definition.assignment,
            Action::ChangeOspfBroadcast {
                draft: alternate.clone(),
            },
        )
        .unwrap();
    assert!(!simulation.assignment.work.checked);
    simulation
        .assignment
        .work
        .apply(&definition.assignment, Action::Check)
        .unwrap();
    assert!(
        simulation
            .assignment
            .work
            .outcomes(&definition.assignment)
            .iter()
            .all(|row| row.passed)
    );
    let Scope::OspfBroadcast { draft } = &simulation.example.work.current else {
        panic!()
    };
    assert_eq!(draft, &practice);

    let store = progress::Progress {
        schema_version: 1,
        attempts: [(lab.id.clone(), attempt)].into(),
        ..Default::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let catalog = std::slice::from_ref(&lab);
    progress::save_with_catalog(&path, &store, catalog).unwrap();
    let saved = progress::load(&path, catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        serde_json::to_value(store).unwrap()
    );
    assert!(engine::solved(&saved.attempts[&lab.id].state, &lab.goal));
}
