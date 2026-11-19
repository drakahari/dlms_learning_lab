use dlms_learning_lab::grub_menu::temporary;
use dlms_learning_lab::grub_once_workspace::{Draft, Input, Task};

fn tasks() -> (Task, Task) {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v7.json")).unwrap();
    let definition = &pack["scenarios"][10]["goal"]["definition"];
    (
        serde_json::from_value(definition["initial"].clone()).unwrap(),
        serde_json::from_value(definition["example"].clone()).unwrap(),
    )
}

#[test]
fn edit_handoffs_and_next_ordinary_boot_remain_distinct() {
    let (task, practice) = tasks();
    task.validate().unwrap();
    practice.validate().unwrap();
    let correct = task.report(&task.reference).unwrap();
    assert!(correct.edited.passed() && correct.ordinary.passed());
    assert_eq!(correct.edited.target.as_deref(), Some("rescue.target"));
    assert_eq!(
        correct.ordinary.target.as_deref(),
        Some("multi-user.target")
    );
    assert_ne!(correct.edited.release, correct.ordinary.release);
    assert_ne!(correct.edited.root, correct.ordinary.root);
    assert!(correct.edited.read_only);
    assert!(!correct.ordinary.read_only);
    let before = task.report(&task.select("inspection").unwrap()).unwrap();
    assert!(!before.edited.passed());
    assert_eq!(
        before.edited.stages.last().unwrap().title,
        "Root filesystem"
    );
    assert_eq!(correct.ordinary, before.ordinary);
}

#[test]
fn alternate_arguments_and_target_partial_credit_are_state_based() {
    let (task, _) = tasks();
    let alternate = Input {
        entry_id: "inspection".into(),
        linux: format!(
            "linux '/ridge-inspection' systemd.unit=rescue.target ro root=UUID={}",
            task.required_root
        ),
    };
    let mut draft = Draft::new(&task).unwrap();
    draft.edit(alternate.clone()).unwrap();
    assert!(!draft.checks(&task).iter().all(|r| r.0));
    draft.test(&task).unwrap();
    assert!(draft.checks(&task).iter().all(|r| r.0));
    draft
        .edit(Input {
            linux: alternate
                .linux
                .replace("rescue.target", "multi-user.target"),
            ..alternate
        })
        .unwrap();
    assert!(draft.observed.is_none());
    draft.test(&task).unwrap();
    let checks = draft.checks(&task);
    assert!(
        checks
            .iter()
            .any(|r| !r.0 && r.1.contains("request rescue.target"))
    );
    assert!(checks.iter().any(|r| r.0 && r.1.contains("ordinary boot")));
}

#[test]
fn invalid_edits_and_mismatched_images_have_distinct_feedback() {
    let (task, _) = tasks();
    for command in [
        "",
        "boot",
        "linux /../outside",
        "linux /x\nboot",
        "linux $unknown",
        "linux /x ; boot",
    ] {
        assert!(temporary::test(&task.inventory, &task.saved_menu, "inspection", command).is_err());
    }
    assert!(
        temporary::test(
            &task.inventory,
            &task.saved_menu,
            "missing",
            &task.reference.linux
        )
        .is_err()
    );
    let wrong_entry = temporary::test(
        &task.inventory,
        &task.saved_menu,
        "stable",
        &task.reference.linux,
    )
    .unwrap();
    assert!(!wrong_entry.edited.passed());
    assert!(wrong_entry.ordinary.passed());
    let mut draft = Draft::new(&task).unwrap();
    let mut pasted = task.reference.clone();
    pasted.linux.push_str("\nboot");
    draft.edit(pasted.clone()).unwrap();
    draft.test(&task).unwrap();
    assert_eq!(draft.input, pasted);
    assert!(draft.checks(&task)[0].1.contains("single linux line"));
}

#[test]
fn practice_and_assignment_stay_separate_and_stale_observation_is_rejected() {
    let (task, practice) = tasks();
    let assignment = Draft::new(&task).unwrap();
    let mut example = Draft::new(&practice).unwrap();
    example.edit(practice.reference.clone()).unwrap();
    example.test(&practice).unwrap();
    assert!(example.checks(&practice).iter().all(|r| r.0));
    assert!(assignment.observed.is_none());
    let snapshot = serde_json::to_string(&example).unwrap();
    assert_eq!(serde_json::from_str::<Draft>(&snapshot).unwrap(), example);
    example.input = practice.starter().unwrap();
    assert!(example.bound(&practice).is_err());
}

#[test]
fn authored_lesson_passes_reference_and_restores_both_drafts_without_changing_saved_menu() {
    use dlms_learning_lab::{engine, grub_once_workspace as work, pack, progress};
    let authored: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v7.json")).unwrap();
    let scenario: pack::Scenario =
        serde_json::from_value(authored["scenarios"][10].clone()).unwrap();
    let definition = work::source(&scenario).unwrap();
    let mut attempt = progress::Attempt::new(&scenario, 1);
    assert!(!progress::current_solved(&attempt, &scenario));
    engine::execute_for_goal(&mut attempt.state, &scenario.goal, &scenario.solution[0]).unwrap();
    assert!(progress::current_solved(&attempt, &scenario));
    let engine::State::GrubOnce { simulation } = &mut attempt.state else {
        panic!()
    };
    let mut practice = definition.example.select("inspection").unwrap();
    practice.linux.push_str(" systemd.unit=rescue.target");
    simulation.practice.edit(practice).unwrap();

    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    profile
        .attempts
        .insert(scenario.id.clone(), attempt.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &profile, std::slice::from_ref(&scenario)).unwrap();
    let mut restored = progress::load(&path, std::slice::from_ref(&scenario)).unwrap();
    let saved = restored.attempts.get_mut(&scenario.id).unwrap();
    assert_eq!(
        serde_json::to_value(&attempt).unwrap(),
        serde_json::to_value(&*saved).unwrap()
    );
    assert!(progress::current_solved(saved, &scenario));

    let engine::State::GrubOnce { simulation } = &mut saved.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(definition.initial.select("inspection").unwrap())
        .unwrap();
    assert!(!progress::current_solved(saved, &scenario));
}

#[test]
fn temporary_recovery_preserves_preceding_catalog_and_course_values() {
    use dlms_learning_lab::{course, pack};
    let prior = pack::accepted383().unwrap();
    let current = pack::accepted384().unwrap();
    assert_eq!((prior.len(), current.len()), (383, 384));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&current[..383]).unwrap()
    );
    let old_units = course::accepted383(&prior).unwrap();
    let units = course::accepted384(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (144, 145));
    let position = units
        .iter()
        .position(|unit| unit.id == "linux-temporary-grub-recovery")
        .unwrap();
    assert_eq!(units[position - 1].id, "linux-removable-efi-recovery");
    assert_eq!(units[position].labs, ["lpic2-grub-one-boot-recovery"]);
    let retained = units
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != position)
        .map(|(_, unit)| unit)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}
