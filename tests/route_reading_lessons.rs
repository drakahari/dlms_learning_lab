use dlms_learning_lab::{
    automation_workshop::{
        source,
        workspace::{Action, Scope, Task, Work},
    },
    course, engine, pack, progress,
};

fn lesson() -> pack::Scenario {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/ccna-route-reading-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
    .remove(0)
}

#[test]
fn route_reading_is_registered_after_ospf_and_keeps_all_saved_lab_ids() {
    let prior = pack::accepted541().unwrap();
    let current = pack::accepted542().unwrap();
    assert_eq!((prior.len(), current.len()), (541, 542));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&current[..541]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(lesson()).unwrap(),
        serde_json::to_value(&current[541]).unwrap()
    );
    let prior_units = course::accepted541(&prior).unwrap();
    let units = course::accepted542(&current).unwrap();
    let index = units
        .iter()
        .position(|unit| unit.id == "ipv4-installed-route-reading")
        .unwrap();
    assert_eq!((prior_units.len(), units.len()), (222, 223));
    assert!(
        units[..index]
            .iter()
            .any(|unit| unit.id == "ospf-p2p-foundations")
    );
    let retained = units
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, unit)| unit)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(prior_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}

#[test]
fn wrong_mask_draft_invalidates_success_without_changing_practice_or_clean_routes() {
    let scenario = lesson();
    let definition = source::source(&scenario).unwrap();
    let mut attempt = progress::Attempt::new(&scenario, 1);
    let engine::State::AutomationWorkshop { simulation } = &mut attempt.state else {
        panic!()
    };
    let Task::RouteReading { supplied } = &definition.example else {
        panic!()
    };
    let mut practice = supplied.start();
    let mut too_long = practice.clone();
    too_long.cost = "unfinished".into();
    assert!(
        simulation
            .example
            .work
            .apply(
                &definition.example,
                Action::ChangeRouteReading { draft: too_long },
            )
            .is_err()
    );
    assert_eq!(simulation.example.work, Work::new(&definition.example));
    practice.cost = "7x".into();
    simulation
        .example
        .work
        .apply(
            &definition.example,
            Action::ChangeRouteReading {
                draft: practice.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        simulation.assignment.work,
        Work::new(&definition.assignment)
    );
    assert!(!progress::current_solved(&attempt, &scenario));

    for line in &scenario.solution {
        engine::execute_for_goal(&mut attempt.state, &scenario.goal, line).unwrap();
    }
    assert!(engine::solved(&attempt.state, &scenario.goal));
    let engine::State::AutomationWorkshop { simulation } = &mut attempt.state else {
        panic!()
    };
    let Scope::RouteReading { draft } = &simulation.assignment.work.current else {
        panic!()
    };
    let mut broken = draft.clone();
    broken.mask_query = "255.255.255.0".into();
    simulation
        .assignment
        .work
        .apply(
            &definition.assignment,
            Action::ChangeRouteReading { draft: broken },
        )
        .unwrap();
    assert!(!simulation.assignment.work.checked);
    simulation
        .assignment
        .work
        .apply(&definition.assignment, Action::Check)
        .unwrap();
    let rows = simulation.assignment.work.outcomes(&definition.assignment);
    assert_eq!(rows.iter().filter(|row| row.passed).count(), 5);
    assert!(!rows.last().unwrap().passed);
    let Scope::RouteReading { draft } = &simulation.example.work.current else {
        panic!()
    };
    assert_eq!(draft, &practice);
    assert!(!engine::solved(&attempt.state, &scenario.goal));

    let store = progress::Progress {
        schema_version: 1,
        attempts: [(scenario.id.clone(), attempt)].into(),
        ..Default::default()
    };
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    let one = std::slice::from_ref(&scenario);
    progress::save_with_catalog(&path, &store, one).unwrap();
    let mut restored = progress::load(&path, one).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(store).unwrap()
    );
    let saved_attempt = restored.attempts.get_mut(&scenario.id).unwrap();
    let engine::State::AutomationWorkshop { simulation } = &mut saved_attempt.state else {
        panic!()
    };
    let saved_example = simulation.example.work.clone();
    simulation
        .assignment
        .work
        .apply(&definition.assignment, Action::Restore)
        .unwrap();
    assert_eq!(
        simulation.assignment.work,
        Work::new(&definition.assignment)
    );
    assert_eq!(simulation.example.work, saved_example);
    assert!(!progress::current_solved(saved_attempt, &scenario));
}
