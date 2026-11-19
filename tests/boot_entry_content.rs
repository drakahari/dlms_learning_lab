use dlms_learning_lab::{
    boot_entry_workspace as b, course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
fn labs() -> Vec<pack::Scenario> {
    pack::current()
        .unwrap()
        .into_iter()
        .filter(|s| matches!(&s.goal, Goal::BootEntry { definition } if definition.initial.menu.is_none() && definition.initial.generator.is_none()))
        .collect()
}
fn parts(s: &pack::Scenario) -> (&b::Definition, &b::Session) {
    let (Goal::BootEntry { definition }, State::BootEntry { simulation }) = (&s.goal, &s.initial)
    else {
        panic!()
    };
    (definition, simulation)
}
#[test]
fn all_authored_type1_boot_lessons_have_working_taught_and_alternate_repairs() {
    let labs = labs();
    assert_eq!(labs.len(), 3);
    for s in labs {
        let (d, initial) = parts(&s);
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for command in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let mut a = initial.clone();
        a.practice.edit(d.example.reference.clone()).unwrap();
        a.practice.check(&d.example).unwrap();
        assert!(!d.checks(&a).iter().all(|c| c.0));
        assert_eq!(a.assignment, initial.assignment);
        let alt = d
            .initial
            .reference
            .lines()
            .filter(|line| !line.starts_with("title ") && !line.starts_with("version "))
            .collect::<Vec<_>>()
            .join("\n")
            .replace(" ro", " quiet ro");
        a.assignment.edit(alt).unwrap();
        a.assignment.check(&d.initial).unwrap();
        assert!(d.checks(&a).iter().all(|c| c.0));
        a.assignment.edit(d.initial.starter.clone()).unwrap();
        assert!(a.assignment.checked.is_none());
        assert!(!d.checks(&a).iter().all(|c| c.0));
    }
}
#[test]
fn complete_current_catalog_and_units_preserve_every_previous_object() {
    let old = pack::accepted373().unwrap();
    let all = pack::current().unwrap();
    assert_eq!(old.len(), 373);
    assert_eq!(all.len(), 402);
    assert_eq!(
        serde_json::to_value(&all[..old.len()]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(all.iter().filter(|s| s.track == "CCNA").count(), 93);
    assert_eq!(all.iter().filter(|s| s.track == "LPIC-1").count(), 133);
    let units = course::current(&all).unwrap();
    let before = course::accepted373(&old).unwrap();
    assert_eq!(units.len(), 154);
    assert_eq!(before.len(), 138);
    for u in before {
        assert_eq!(
            serde_json::to_value(units.iter().find(|n| n.id == u.id).unwrap()).unwrap(),
            serde_json::to_value(u).unwrap()
        );
    }
    let unit = units
        .iter()
        .find(|u| u.id == "linux-boot-entry-recovery")
        .unwrap();
    assert_eq!(unit.labs.len(), 3);
}
#[test]
fn wrong_partial_malformed_and_correct_but_unrequested_outcomes_fail_usefully() {
    for s in labs() {
        let (d, initial) = parts(&s);
        let mut a = initial.clone();
        a.assignment.check(&d.initial).unwrap();
        assert!(d.checks(&a).iter().any(|c| !c.0));
        for bad in [
            d.initial
                .reference
                .replace(&d.initial.required_root, &d.initial.inventory.roots[1].uuid),
            d.initial.reference.replace(" ro", " rw"),
            d.initial.reference.replace(
                &format!("linux /system/{}/linux", d.initial.required_release),
                "linux /missing",
            ),
            "linux=/not-valid".into(),
            "linux /missing\noptions $(unsafe)".into(),
        ] {
            a.assignment.edit(bad).unwrap();
            let rows = a.assignment.check(&d.initial).unwrap();
            assert!(rows.iter().any(|c| !c.0), "{} {:?}", s.id, rows);
            assert!(rows.iter().any(|c| c.1.contains("filesystem")
                || c.1.contains("Syntax")
                || c.1.contains("Unsupported")
                || c.1.contains("kernel")
                || c.1.contains("mode")));
        }
        if d.initial.required_target == "rescue.target" {
            a.assignment
                .edit(
                    d.initial
                        .reference
                        .replace("options systemd.unit=rescue.target\n", ""),
                )
                .unwrap();
            let checks = a.assignment.check(&d.initial).unwrap();
            assert!(
                checks
                    .iter()
                    .any(|c| c.0 && c.1 == "Modeled init handoff reached.")
            );
            assert!(checks.iter().any(|c| !c.0 && c.1.contains("rescue.target")));
        }
    }
}
#[test]
fn saved_drafts_resume_reset_and_import_cannot_fabricate_success_or_task_facts() {
    let scenarios = labs();
    let s = &scenarios[0];
    let (d, _) = parts(s);
    let mut a = Attempt::new(s, 10);
    let State::BootEntry { simulation } = &mut a.state else {
        panic!()
    };
    simulation.view = View::Assignment;
    simulation
        .assignment
        .edit(d.initial.reference.clone())
        .unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    simulation
        .practice
        .edit("# separate unfinished practice\nlinux /missing".into())
        .unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a.clone());
    assert!(progress::current_solved(&a, s));
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-boot-entry/profiles");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    progress::save_with_catalog(&path, &p, &pack::current().unwrap()).unwrap();
    let loaded = progress::load(&path, &pack::current().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let catalog = pack::current().unwrap();
    let archive = path.with_extension("backup.json");
    dlms_learning_lab::backup::export(&archive, &p, &catalog).unwrap();
    let inspected = dlms_learning_lab::backup::inspect(
        &archive,
        &catalog,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    let restored = dlms_learning_lab::backup::restore(&path, &inspected, &catalog).unwrap();
    assert!(restored.safety_copy.is_some());
    assert_eq!(
        serde_json::to_value(progress::load(&path, &catalog).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let mut wrong = a.clone();
    let State::BootEntry { simulation } = &mut wrong.state else {
        panic!()
    };
    simulation.assignment.source = d.initial.starter.clone();
    assert!(!progress::current_solved(&wrong, s));
    assert!(b::context(&wrong, s).is_err());
    let mut wrong = a.clone();
    let State::BootEntry { simulation } = &mut wrong.state else {
        panic!()
    };
    simulation.assignment.checked = None;
    assert!(!progress::current_solved(&wrong, s));
    let fresh = Attempt::new(s, 11);
    assert!(!progress::current_solved(&fresh, s));
    assert_eq!(fresh.state, s.initial);
    assert_eq!(p.attempts[&s.id].state, a.state);
}
#[test]
fn invalid_boot_packs_fail_before_catalog_credit() {
    let mut original = serde_json::from_str::<serde_json::Value>(include_str!(
        "../scenarios/boot-entry-lessons-v1.json"
    ))
    .unwrap();
    original["scenarios"][0]["prerequisites"] = serde_json::json!([]);
    assert!(pack::load(&original.to_string()).is_ok());
    for field in ["id", "required_root", "required_release", "reference"] {
        let mut bad = original.clone();
        bad["scenarios"][0]["goal"]["definition"]["initial"][field] = serde_json::json!("invalid");
        assert!(pack::load(&bad.to_string()).is_err(), "{field}");
    }
    let mut bad = original;
    bad["scenarios"][0]["initial"]["simulation"]["assignment"]["checked"] =
        bad["scenarios"][0]["initial"]["simulation"]["assignment"]["source"].clone();
    assert!(pack::load(&bad.to_string()).is_err());
}

#[test]
fn blocked_early_boot_does_not_mislabel_correct_root_and_target_configuration() {
    let scenarios = labs();
    let s = &scenarios[1];
    let (d, _) = parts(s);
    let rows = d.initial.evaluate(&d.initial.starter);
    assert!(rows.iter().any(|c| !c.0 && c.1.contains("Early userspace")));
    assert!(
        rows.iter()
            .any(|c| c.0 && c.1.starts_with("Configured root=UUID="))
    );
    assert!(
        rows.iter()
            .any(|c| c.0 && c.1.starts_with("Configured startup request"))
    );
    assert!(
        rows.iter()
            .any(|c| !c.0 && c.1.contains("handoff is blocked"))
    );
}
