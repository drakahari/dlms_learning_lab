use dlms_learning_lab::{
    course, efi_loader_workspace as work,
    engine::{self, Goal, State},
    pack, progress,
};
use serde_json::Value;

const ID: &str = "lpic2-efi-removable-recovery";

fn scenario() -> pack::Scenario {
    let pack: Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v6.json")).unwrap();
    serde_json::from_value(
        pack["scenarios"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone(),
    )
    .unwrap()
}

fn definition(scenario: &pack::Scenario) -> &work::Definition {
    let Goal::EfiLoader { definition } = &scenario.goal else {
        panic!()
    };
    definition
}

#[test]
fn workshop_exposes_two_distinct_dependencies_and_preserves_assignment() {
    let scenario = scenario();
    let definition = work::source(&scenario).unwrap();
    let mut session = work::Session::new(definition).unwrap();
    session.practice.test(&definition.example).unwrap();
    let blocked = definition.example.checks(&session.practice);
    assert!(
        blocked
            .iter()
            .any(|row| !row.0 && row.1.contains("unavailable disk /dev/vdc"))
    );
    assert!(
        blocked
            .iter()
            .any(|row| row.0 && row.1.contains("Normal startup still reaches Linux"))
    );
    session.practice.command = definition.example.reference[0].clone();
    session.practice.run(&definition.example).unwrap();
    assert!(session.practice.checked.is_none());
    session.practice.test(&definition.example).unwrap();
    let copied = definition.example.checks(&session.practice);
    assert!(
        copied.iter().any(|row| !row.0
            && row.1.contains("Root filesystem")
            && row.1.contains("unavailable"))
    );
    assert!(
        !copied
            .iter()
            .any(|row| !row.0 && row.1.contains("/workshop/copied-boot is on"))
    );
    session.practice.command = definition.example.reference[1].clone();
    session.practice.run(&definition.example).unwrap();
    session.practice.test(&definition.example).unwrap();
    assert!(
        definition
            .example
            .checks(&session.practice)
            .iter()
            .all(|row| row.0)
    );
    assert_eq!(session.assignment.current, definition.initial.initial);
    assert!(definition.checks(&session).iter().any(|row| !row.0));
}

#[test]
fn acceptable_installation_alternatives_and_changed_firmware_or_root_are_graded() {
    let scenario = scenario();
    for command in [
        scenario.solution[0].as_str(),
        "grub-install --removable --no-nvram --boot-directory '/rescue/recovery-boot' --target x86_64-efi --efi-directory /rescue/removable-esp",
    ] {
        let mut state = scenario.initial.clone();
        engine::execute_for_goal(&mut state, &scenario.goal, command).unwrap();
        assert!(!engine::solved(&state, &scenario.goal));
        engine::execute_for_goal(&mut state, &scenario.goal, work::CHECK_ACTION).unwrap();
        assert!(engine::solved(&state, &scenario.goal));
        engine::execute_for_goal(&mut state, &scenario.goal,
            r"efibootmgr -c -d /dev/vdb -p 1 -L 'Unneeded recovery entry' -l '\EFI\BOOT\BOOTX64.EFI'").unwrap();
        assert!(!engine::solved(&state, &scenario.goal));
        engine::execute_for_goal(&mut state, &scenario.goal, work::CHECK_ACTION).unwrap();
        assert!(
            engine::checks(&state, &scenario.goal)
                .iter()
                .any(|row| !row.0 && row.1.contains("Keep the original firmware entries"))
        );
    }
    let mut state = scenario.initial.clone();
    for command in &scenario.solution {
        engine::execute_for_goal(&mut state, &scenario.goal, command).unwrap();
    }
    let copied = scenario.solution[0].replace("/rescue/recovery-boot", "/rescue/copied-boot");
    engine::execute_for_goal(&mut state, &scenario.goal, &copied).unwrap();
    assert!(!engine::solved(&state, &scenario.goal));
    engine::execute_for_goal(&mut state, &scenario.goal, work::CHECK_ACTION).unwrap();
    let checks = engine::checks(&state, &scenario.goal);
    assert!(
        checks.iter().any(|row| !row.0
            && row.1.contains("Root filesystem")
            && row.1.contains("unavailable"))
    );
    assert!(
        checks
            .iter()
            .any(|row| row.0 && row.1.contains("original firmware entries"))
    );
}

#[test]
fn removable_unit_retains_every_named_loader_scenario_and_unit() {
    let old = pack::accepted382().unwrap();
    let now = pack::accepted383().unwrap();
    assert_eq!((old.len(), now.len()), (382, 383));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..382]).unwrap()
    );
    let old_units = course::accepted382(&old).unwrap();
    let units = course::accepted383(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (143, 144));
    let retained: Vec<_> = units
        .iter()
        .filter(|unit| unit.id != "linux-removable-efi-recovery")
        .collect();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let position = units
        .iter()
        .position(|unit| unit.id == "linux-removable-efi-recovery")
        .unwrap();
    assert_eq!(units[position - 1].id, "linux-unit-manager-foundations");
    assert_eq!(units[position].labs, [ID]);
    assert_eq!(
        units[position].prerequisites,
        [
            "linux-efi-loader-recovery",
            "linux-unit-manager-foundations"
        ]
    );
    assert_eq!(now.iter().filter(|lab| lab.track == "LPIC-2").count(), 91);
}

#[test]
fn media_policy_is_source_bound_and_persisted_without_fabricating_results() {
    let s = scenario();
    let d = definition(&s);
    let mut attempt = progress::Attempt::new(&s, 1);
    let State::EfiLoader { simulation } = &mut attempt.state else {
        panic!()
    };
    for command in &s.solution {
        simulation.execute_reference(d, command).unwrap();
    }
    simulation.assignment.command = "grub-install --removable --efi-directory".into();
    let original = attempt.clone();
    let serialized = serde_json::to_string(&attempt).unwrap();
    let loaded: progress::Attempt = serde_json::from_str(&serialized).unwrap();
    work::context(&loaded, &s).unwrap();
    assert!(progress::current_solved(&loaded, &s));
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    let mut invalid = s.clone();
    let Goal::EfiLoader { definition } = &mut invalid.goal else {
        panic!()
    };
    definition.initial.media.as_mut().unwrap().unavailable_disk = "/dev/vdb".into();
    assert!(work::source(&invalid).is_err());
    let mut wrong_track = s.clone();
    wrong_track.track = "LPIC-1".into();
    assert!(work::source(&wrong_track).is_err());
}
