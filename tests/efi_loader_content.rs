use dlms_learning_lab::{
    course, efi_loader_workspace as work,
    engine::{self, Goal, State},
    pack, progress,
};
use serde_json::Value;
const ID: &str = "lpic1-grub-efi-registration";

fn scenario() -> pack::Scenario {
    let data: Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v5.json")).unwrap();
    serde_json::from_value(
        data["scenarios"]
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
fn authored_assignment_teaches_distinct_real_workshop_and_accepts_alternate_registration() {
    let scenario = scenario();
    let definition = definition(&scenario);
    work::source(&scenario).unwrap();
    let mut session = work::Session::new(definition).unwrap();
    for command in &definition.example.reference[..2] {
        session.practice.command = command.clone();
        session.practice.run(&definition.example).unwrap();
    }
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

    let mut state = scenario.initial.clone();
    for command in &scenario.solution {
        engine::execute_for_goal(&mut state, &scenario.goal, command).unwrap();
    }
    assert!(engine::solved(&state, &scenario.goal));
    let mut state = scenario.initial.clone();
    let alternate = [
        "grub-install --no-nvram --boot-directory '/target/boot' --bootloader-id Recovered --target x86_64-efi --efi-directory '/rescue/esp'",
        r"efibootmgr -c -d /dev/vda -p 1 -L 'New Linux path' -l '\EFI\RECOVERED\GRUBX64.EFI'",
        work::CHECK_ACTION,
    ];
    for command in alternate {
        engine::execute_for_goal(&mut state, &scenario.goal, command).unwrap();
    }
    assert!(engine::solved(&state, &scenario.goal));
    engine::execute_for_goal(
        &mut state,
        &scenario.goal,
        "efibootmgr --bootorder 0007,0000",
    )
    .unwrap();
    assert!(!engine::solved(&state, &scenario.goal));
    engine::execute_for_goal(&mut state, &scenario.goal, work::CHECK_ACTION).unwrap();
    assert!(
        engine::checks(&state, &scenario.goal)
            .iter()
            .any(|row| !row.0 && row.1.contains("another EFI partition"))
    );
}

#[test]
fn current_efi_unit_preserves_every_prior_scenario_and_unit() {
    let old = pack::accepted381().unwrap();
    let current = pack::accepted382().unwrap();
    assert_eq!((old.len(), current.len()), (381, 382));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..381]).unwrap()
    );
    assert_eq!(current.last().unwrap().id, ID);
    let old_units = course::accepted381(&old).unwrap();
    let units = course::accepted382(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (142, 143));
    let retained: Vec<_> = units
        .iter()
        .filter(|unit| unit.id != "linux-efi-loader-recovery")
        .collect();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let position = units
        .iter()
        .position(|unit| unit.id == "linux-efi-loader-recovery")
        .unwrap();
    assert_eq!(units[position - 1].id, "linux-bios-loader-recovery");
    assert_eq!(units[position].labs, [ID]);
    assert_eq!(
        units[position].prerequisites,
        ["linux-bios-loader-recovery", "linux-disk-preparation"]
    );
    for (track, count) in [
        ("CCNA", 89),
        ("LPIC-1", 126),
        ("LPIC-2", 90),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(
            current
                .iter()
                .filter(|scenario| scenario.track == track)
                .count(),
            count
        );
    }
}

#[test]
fn catalog_admission_rejects_fake_progress_wrong_facts_and_trivial_or_incomplete_solutions() {
    let good = serde_json::json!({"schema_version":1,"scenarios":[scenario()]});
    assert_eq!(pack::load(&good.to_string()).unwrap().len(), 1);
    for mutation in 0..5 {
        let mut bad = good.clone();
        let lab = &mut bad["scenarios"][0];
        match mutation {
            0 => lab["solution"] = serde_json::json!([work::CHECK_ACTION]),
            1 => lab["goal"]["definition"]["initial"]["required_esp"] = serde_json::json!("ffff"),
            2 => {
                lab["initial"]["simulation"]["assignment"]["command"] =
                    serde_json::json!("prefilled learner answer")
            }
            3 => {
                lab["goal"]["definition"]["initial"]["reference"][1] =
                    serde_json::json!("efibootmgr --bootorder 0007")
            }
            _ => {
                lab["solution"].as_array_mut().unwrap().pop();
            }
        }
        assert!(pack::load(&bad.to_string()).is_err(), "mutation {mutation}");
    }
}

#[test]
fn real_progress_and_backup_keep_checked_assignment_and_separate_unfinished_input() {
    let catalog = pack::current().unwrap();
    let scenario = catalog.iter().find(|s| s.id == ID).unwrap();
    let definition = definition(scenario);
    let mut attempt = progress::Attempt::new(scenario, 1);
    for command in &scenario.solution {
        engine::execute_for_goal(&mut attempt.state, &scenario.goal, command).unwrap();
    }
    let State::EfiLoader { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation.assignment.command = "efibootmgr --bootorder".into();
    simulation.practice.command = "lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS".into();
    simulation.practice.run(&definition.example).unwrap();
    let mut data = progress::Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    data.favorites.insert(ID.into());
    data.attempts.insert(ID.into(), attempt);
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-efi-install/profiles");
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(progress::save(&file, &data).is_err());
    progress::save_with_catalog(&file, &data, &catalog).unwrap();
    let loaded = progress::load(&file, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&data).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[ID], scenario));
    let backup = file.with_extension("backup.json");
    dlms_learning_lab::backup::export(&backup, &loaded, &catalog).unwrap();
    let inspected = dlms_learning_lab::backup::inspect(
        &backup,
        &catalog,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert!(
        dlms_learning_lab::backup::restore(&file, &inspected, &catalog)
            .unwrap()
            .safety_copy
            .is_some()
    );
    let before = std::fs::read(&file).unwrap();
    let State::EfiLoader { simulation } = &mut data.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation.assignment.current.boot_order.reverse();
    assert!(progress::save_with_catalog(&file, &data, &catalog).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
}
