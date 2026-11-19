use dlms_learning_lab::{
    automation_workshop::{
        source,
        workspace::{Action, Scope, Task, Work},
    },
    course, engine, pack, progress,
};

fn lesson() -> pack::Scenario {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/nfs-export-operations-v1.json"))
        .unwrap()
        .scenarios
        .remove(0)
}

#[test]
fn one_lpic2_lesson_follows_nfs_and_preserves_every_existing_lab() {
    let before = pack::accepted540().unwrap();
    let current = pack::accepted541().unwrap();
    assert_eq!((before.len(), current.len()), (540, 541));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..540]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(lesson()).unwrap(),
        serde_json::to_value(&current[540]).unwrap()
    );
    let previous = course::accepted540(&before).unwrap();
    let current_units = course::accepted541(&current).unwrap();
    assert_eq!((previous.len(), current_units.len()), (221, 222));
    let new = "linux-nfs-export-operations";
    let index = current_units
        .iter()
        .position(|unit| unit.id == new)
        .unwrap();
    assert!(
        current_units[..index]
            .iter()
            .any(|u| u.id == "nfs-network-file-sharing")
    );
    let carried = current_units
        .iter()
        .filter(|u| u.id != new)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(previous).unwrap(),
        serde_json::to_value(carried).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [127, 158, 176, 80]
    );
}

#[test]
fn assignments_need_actual_work_and_saved_progress_keeps_practice_separate() {
    let lab = lesson();
    let definition = source::source(&lab).unwrap();
    let mut a = progress::Attempt::new(&lab, 1);
    let engine::State::AutomationWorkshop { simulation } = &mut a.state else {
        panic!()
    };
    let Task::NfsExportRetirement { supplied: guided } = &definition.example else {
        panic!()
    };
    let address = guided.definition.initial.inventory().servers[0].address;
    simulation
        .example
        .work
        .apply(
            &definition.example,
            Action::NfsTerminalDraft {
                text: format!("showmount -e {address}"),
            },
        )
        .unwrap();
    assert_eq!(
        simulation.assignment.work,
        Work::new(&definition.assignment)
    );
    assert!(!progress::current_solved(&a, &lab));
    for step in &lab.solution[..lab.solution.len() - 1] {
        engine::execute_for_goal(&mut a.state, &lab.goal, step).unwrap();
    }
    assert!(!engine::solved(&a.state, &lab.goal));
    engine::execute_for_goal(&mut a.state, &lab.goal, lab.solution.last().unwrap()).unwrap();
    assert!(engine::solved(&a.state, &lab.goal));
    let engine::State::AutomationWorkshop { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .work
        .apply(
            &definition.assignment,
            Action::NfsTerminalRun {
                text: "exportfs -v".into(),
            },
        )
        .unwrap();
    assert!(simulation.assignment.work.checked);
    simulation
        .assignment
        .work
        .apply(
            &definition.assignment,
            Action::NfsExportDraft {
                text: "# unfinished policy\n".into(),
            },
        )
        .unwrap();
    assert!(!simulation.assignment.work.checked);
    let Scope::NfsExportRetirement { terminal } = &simulation.example.work.current else {
        panic!()
    };
    assert_eq!(terminal.draft, format!("showmount -e {address}"));
    let store = progress::Progress {
        schema_version: 1,
        attempts: [(lab.id.clone(), a)].into(),
        ..Default::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &store, std::slice::from_ref(&lab)).unwrap();
    let reopened = progress::load(&path, std::slice::from_ref(&lab)).unwrap();
    assert_eq!(
        serde_json::to_value(store).unwrap(),
        serde_json::to_value(&reopened).unwrap()
    );
    assert!(!progress::current_solved(&reopened.attempts[&lab.id], &lab));
}
