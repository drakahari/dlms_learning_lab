use dlms_learning_lab::{
    bios_loader_workspace as b, course,
    engine::{self, Goal, State},
    pack, progress,
};
const ID: &str = "lpic1-grub-bios-install";
fn scenario() -> pack::Scenario {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v4.json")).unwrap();
    serde_json::from_value(v["scenarios"].as_array().unwrap().last().unwrap().clone()).unwrap()
}
fn definition(s: &pack::Scenario) -> &b::Definition {
    let Goal::BiosLoader { definition } = &s.goal else {
        panic!()
    };
    definition
}
#[test]
fn authored_recovery_teaches_inspection_real_repair_and_preservation_with_alternate_solutions() {
    let s = scenario();
    let d = definition(&s);
    d.validate().unwrap();
    b::source(&s).unwrap();
    let mut session = b::Session::new(d).unwrap();
    for command in &d.example.reference {
        if command == b::CHECK_ACTION {
            session.practice.test(&d.example).unwrap();
        } else {
            session.practice.command = command.clone();
            session.practice.run(&d.example).unwrap();
        }
    }
    assert!(
        d.example
            .checks(&session.practice.current)
            .iter()
            .all(|c| c.0)
    );
    assert!(d.checks(&session).iter().any(|c| !c.0));
    for command in [
        "grub-install /dev/vda --boot-directory '/target/boot' --target i386-pc",
        "grub-install --boot-directory=/target/boot /dev/vda",
    ] {
        let mut state = s.initial.clone();
        engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        engine::execute_for_goal(&mut state, &s.goal, b::CHECK_ACTION).unwrap();
        assert!(engine::solved(&state, &s.goal));
        engine::execute_for_goal(&mut state, &s.goal, "grub-install /dev/vda").unwrap();
        assert!(!engine::solved(&state, &s.goal));
        engine::execute_for_goal(&mut state, &s.goal, b::CHECK_ACTION).unwrap();
        assert!(
            engine::checks(&state, &s.goal)
                .iter()
                .any(|c| !c.0 && c.1.contains("does not generate"))
        );
    }
    let mut state = s.initial.clone();
    for cmd in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, cmd).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    engine::execute_for_goal(
        &mut state,
        &s.goal,
        "grub-install --boot-directory=/target/boot /dev/vdb",
    )
    .unwrap();
    engine::execute_for_goal(&mut state, &s.goal, b::CHECK_ACTION).unwrap();
    assert!(!engine::solved(&state, &s.goal));
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|c| !c.0 && c.1.contains("/dev/vdb changed"))
    );
}
#[test]
fn registered381_preserves_all380_scenarios_units_and_prerequisites() {
    let old = pack::accepted380().unwrap();
    let now = pack::accepted381().unwrap();
    assert_eq!((old.len(), now.len()), (380, 381));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..380]).unwrap()
    );
    let old_units = course::accepted380(&old).unwrap();
    let units = course::accepted381(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (141, 142));
    for u in old_units {
        assert_eq!(
            serde_json::to_value(&u).unwrap(),
            serde_json::to_value(units.iter().find(|v| v.id == u.id).unwrap()).unwrap()
        );
    }
    let at = units
        .iter()
        .position(|u| u.id == "linux-bios-loader-recovery")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-grub-persistent-configuration");
    assert_eq!(units[at].labs, [ID]);
    assert_eq!(
        units[at].prerequisites,
        ["linux-grub-persistent-configuration", "mbr-layouts"]
    );
    for (track, count) in [
        ("CCNA", 89),
        ("LPIC-1", 125),
        ("LPIC-2", 90),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn saved_console_is_source_bound_retains_separate_drafts_and_safe_backup_restore() {
    let catalog = pack::current().unwrap();
    let s = catalog.iter().find(|s| s.id == ID).unwrap();
    let d = definition(s);
    let mut p = progress::Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    let mut a = progress::Attempt::new(s, 1);
    let State::BiosLoader { simulation } = &mut a.state else {
        panic!()
    };
    for cmd in &s.solution {
        simulation.execute(d, cmd).unwrap();
    }
    simulation.assignment.command = "grub-install --boot-directory='unfinished".into();
    simulation.practice.command = "lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS".into();
    simulation.practice.run(&d.example).unwrap();
    p.attempts.insert(ID.into(), a);
    p.favorites.insert(ID.into());
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-boot-install/profiles");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(
        progress::save(&path, &p).is_err(),
        "source-free saves cannot admit BIOS state"
    );
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let restored = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[ID], s));
    let backup = path.with_extension("backup.json");
    dlms_learning_lab::backup::export(&backup, &p, &catalog).unwrap();
    let inspected = dlms_learning_lab::backup::inspect(
        &backup,
        &catalog,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert!(
        dlms_learning_lab::backup::restore(&path, &inspected, &catalog)
            .unwrap()
            .safety_copy
            .is_some()
    );
    let before = std::fs::read(&path).unwrap();
    let State::BiosLoader { simulation } = &mut p.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation.context = "different-task".into();
    assert!(progress::save_with_catalog(&path, &p, &catalog).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let mut wrong = s.clone();
    wrong.objective.push_str(" changed");
    assert!(b::source(&wrong).is_err());
}
#[test]
fn invalid_boot_sources_unreachable_facts_and_unchecked_solutions_fail_admission() {
    let source = scenario();
    let valid = serde_json::json!({"schema_version":1,"scenarios":[source]});
    assert_eq!(pack::load(&valid.to_string()).unwrap().len(), 1);
    for mutation in 0..5 {
        let mut bad = valid.clone();
        let s = &mut bad["scenarios"][0];
        match mutation {
            0 => s["solution"] = serde_json::json!([b::CHECK_ACTION]),
            1 => {
                s["goal"]["definition"]["initial"]["facts"]["core_sectors"] =
                    serde_json::json!(2048)
            }
            2 => {
                s["initial"]["simulation"]["assignment"]["command"] =
                    serde_json::json!("fabricated input")
            }
            3 => s["initial"]["simulation"]["context"] = serde_json::json!("another-task"),
            _ => {
                s["solution"] =
                    serde_json::json!(["grub-install --boot-directory=/target/boot /dev/vda"])
            }
        }
        assert!(pack::load(&bad.to_string()).is_err(), "mutation {mutation}");
    }
}
