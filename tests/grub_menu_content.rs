use dlms_learning_lab::{
    boot_entry_workspace as b,
    engine::{self, Goal, State},
    pack,
};
const IDS: [&str; 2] = ["lpic1-grub-default-fallback", "lpic1-grub-recovery-entry"];
fn authored() -> Vec<pack::Scenario> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v2.json")).unwrap();
    v["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["id"].as_str().is_some_and(|id| IDS.contains(&id)))
        .map(|s| serde_json::from_value(s.clone()).unwrap())
        .collect()
}
fn definition(s: &pack::Scenario) -> &b::Definition {
    let Goal::BootEntry { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn passed(rows: &[(bool, String)]) -> bool {
    rows.iter().all(|r| r.0)
}
#[test]
fn both_authored_menus_teach_working_methods_and_accept_equivalent_selectors() {
    let labs = authored();
    assert_eq!(labs.len(), 2);
    for s in labs {
        let d = definition(&s);
        d.validate().unwrap();
        assert_eq!(d.initial.instructions, s.objective);
        assert_eq!(d.initial.id, s.id);
        assert_eq!(
            serde_json::to_value(b::Session::new(d).unwrap()).unwrap(),
            serde_json::to_value(match &s.initial {
                State::BootEntry { simulation } => &**simulation,
                _ => panic!(),
            })
            .unwrap()
        );
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for command in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        let mut session = b::Session::new(d).unwrap();
        session.practice.edit(d.example.reference.clone()).unwrap();
        assert!(session.practice.checked.is_none());
        assert!(!passed(&d.checks(&session)));
        session.practice.check(&d.example).unwrap();
        assert!(passed(&session.practice.checks(&d.example)));
        assert!(!passed(&d.checks(&session)));
        assert_eq!(session.assignment.source, d.initial.starter);
        let id = if s.id == IDS[0] { "spare" } else { "recovery" };
        let title = if s.id == IDS[0] {
            "Spare Linux"
        } else {
            "Recovery Linux"
        };
        for selector in [
            "1".to_owned(),
            id.into(),
            format!("'{title}'"),
            format!("\"{title}\""),
        ] {
            let source = d
                .initial
                .reference
                .replacen(
                    &format!("set default={id}"),
                    &format!("set default={selector}"),
                    1,
                )
                .replace(" ro systemd.unit=", " systemd.unit=");
            // Keep mode explicit, but reorder the selected command's argument tokens.
            let source = source
                .replace("multi-user.target\n", "multi-user.target ro\n")
                .replace("rescue.target\n", "rescue.target ro\n");
            session.assignment.edit(source).unwrap();
            assert!(!passed(&d.checks(&session)));
            session.assignment.check(&d.initial).unwrap();
            assert!(
                passed(&d.checks(&session)),
                "{} {selector}: {:?}",
                s.id,
                d.checks(&session)
            );
        }
        session.assignment.edit(d.initial.starter.clone()).unwrap();
        assert!(session.assignment.checked.is_none());
        assert!(!passed(&d.checks(&session)));
    }
}
#[test]
fn plausible_wrong_choices_partial_repairs_and_protected_entry_changes_fail_specifically() {
    for s in authored() {
        let d = definition(&s);
        let t = &d.initial;
        assert!(!passed(&t.evaluate(&t.starter)));
        let bad = if s.id == IDS[0] {
            t.reference
                .replacen("set default=spare", "set default=maintenance", 1)
        } else {
            t.reference.replace(
                "systemd.unit=rescue.target",
                "systemd.unit=multi-user.target",
            )
        };
        assert!(
            t.report(&bad).unwrap().passed(),
            "A bootable wrong target must still fail the task"
        );
        assert!(
            t.evaluate(&bad)
                .iter()
                .any(|r| !r.0 && r.1.contains("startup request"))
        );
        let changed = t
            .reference
            .replacen("'Primary Linux'", "'Renamed upgrade'", 1)
            .replacen("'Normal Linux'", "'Renamed normal'", 1);
        assert!(
            t.evaluate(&changed)
                .iter()
                .any(|r| !r.0 && r.1.contains("title changed")),
            "{:?}",
            t.evaluate(&changed)
        );
        let wrong_id = t
            .reference
            .replacen("set default=", "set default=absent-", 1);
        assert!(
            t.evaluate(&wrong_id)
                .iter()
                .any(|r| !r.0 && r.1.contains("Unsupported") && r.1.contains("default"))
        );
        let unsupported = t.reference.clone() + "systemctl reboot\n";
        assert!(
            t.evaluate(&unsupported).iter().any(|r| !r.0
                && r.1.contains("Unsupported")
                && r.1.contains("nothing was executed"))
        );
        let mut session = b::Session::new(d).unwrap();
        session.assignment.edit(t.reference.clone()).unwrap();
        session.assignment.check(t).unwrap();
        assert!(passed(&d.checks(&session)));
        session.assignment.source = bad;
        assert!(
            !passed(&d.checks(&session)),
            "Forged stale marker must not hold success"
        );
    }
}
#[test]
fn supplied_input_changes_are_evaluated_instead_of_recognizing_authored_answer_strings() {
    let s = authored().remove(1);
    let mut d = definition(&s).clone();
    for t in [&mut d.initial, &mut d.example] {
        let previous = t.required_root.clone();
        let fresh = if t.id.starts_with("workshop-") {
            "00112233-4455-6677-8899-aabbccddeeff"
        } else {
            "ffeeddcc-bbaa-9988-7766-554433221100"
        };
        t.inventory
            .roots
            .iter_mut()
            .find(|r| r.uuid == previous)
            .unwrap()
            .uuid = fresh.into();
        t.required_root = fresh.into();
        t.reference = t.reference.replace(&previous, fresh);
        t.starter = t.starter.replace(&previous, fresh);
    }
    d.validate().unwrap();
    let mut a = b::Session::new(&d).unwrap();
    a.assignment.edit(d.initial.reference.clone()).unwrap();
    a.assignment.check(&d.initial).unwrap();
    assert!(passed(&d.checks(&a)));
    a.assignment
        .edit(definition(&s).initial.reference.clone())
        .unwrap();
    a.assignment.check(&d.initial).unwrap();
    assert!(!passed(&d.checks(&a)));
    a.context = IDS[0].into();
    assert!(
        !passed(&d.checks(&a)),
        "Another task's context cannot claim credit"
    );
}
#[test]
fn invalid_pack_policy_reachability_and_prefilled_credit_are_rejected() {
    let mut original: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v2.json")).unwrap();
    // This isolated5lab pack excludes only its first external driver prerequisite;
    // its actual production prerequisite is validated by the complete catalog gate.
    original["scenarios"][0]["prerequisites"] = serde_json::json!([]);
    assert!(pack::load(&original.to_string()).is_ok());
    for policy in [
        serde_json::json!({"preserve":[],"selected_id":""}),
        serde_json::json!({"preserve":["normal","normal"],"selected_id":"recovery"}),
        serde_json::json!({"preserve":["missing"],"selected_id":"recovery"}),
    ] {
        let mut bad = original.clone();
        bad["scenarios"][4]["goal"]["definition"]["initial"]["menu"] = policy;
        assert!(pack::load(&bad.to_string()).is_err());
    }
    let mut bad = original.clone();
    bad["scenarios"][3]["goal"]["definition"]["initial"]["reference"] =
        serde_json::json!("set default=absent");
    assert!(pack::load(&bad.to_string()).is_err());
    let mut bad = original;
    bad["scenarios"][4]["initial"]["simulation"]["assignment"]["checked"] =
        bad["scenarios"][4]["initial"]["simulation"]["assignment"]["source"].clone();
    assert!(pack::load(&bad.to_string()).is_err());
}

#[test]
fn registered_catalog_and_unit_preserve_every_preexisting_object_and_id() {
    let before = pack::accepted376().unwrap();
    let current = pack::current().unwrap();
    assert_eq!(before.len(), 376);
    assert_eq!(current.len(), 402);
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..376]).unwrap()
    );
    let old = dlms_learning_lab::course::accepted376(&before).unwrap();
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!(old.len(), 139);
    assert_eq!(units.len(), 154);
    let retained = units
        .iter()
        .filter(|u| !matches!(u.id.as_str(), "linux-grub-menu-recovery" | "linux-grub-persistent-configuration" | "linux-bios-loader-recovery" | "linux-efi-loader-recovery" | "linux-removable-efi-recovery" | "linux-temporary-grub-recovery" | "linux-rescue-target-repair" | "linux-clock-source-diagnosis" | "linux-copied-journal" | "linux-local-mail-routing" | "ccna-wired-device-paths" | "structured-network-inventory" | "linux-workload-disk-layout" | "shell-measurement-reports" | "linux-resource-observations"))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let unit = units
        .iter()
        .find(|u| u.id == "linux-grub-menu-recovery")
        .unwrap();
    assert_eq!(unit.labs, IDS);
    assert_eq!(unit.prerequisites, ["linux-boot-entry-recovery"]);
    assert_eq!(
        units
            .iter()
            .position(|u| u.id == "linux-grub-menu-recovery")
            .unwrap(),
        units
            .iter()
            .position(|u| u.id == "linux-boot-entry-recovery")
            .unwrap()
            + 1
    );
    assert_eq!(
        current
            .iter()
            .filter(|s| matches!(s.initial, State::BootEntry { .. }))
            .count(),
        7
    );
    assert_eq!(current.iter().filter(|s| s.track == "CCNA").count(), 93);
    assert_eq!(current.iter().filter(|s| s.track == "LPIC-1").count(), 133);
    assert_eq!(current.iter().filter(|s| s.track == "LPIC-2").count(), 93);
    assert_eq!(
        current.iter().filter(|s| s.track == "Bash/Shell").count(),
        77
    );
    let origin: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v1.json")).unwrap();
    let extended: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v2.json")).unwrap();
    assert_eq!(
        origin["scenarios"].as_array().unwrap(),
        &extended["scenarios"].as_array().unwrap()[..3]
    );
    for s in authored() {
        assert_eq!(
            serde_json::to_value(current.iter().find(|x| x.id == s.id).unwrap()).unwrap(),
            serde_json::to_value(s).unwrap()
        );
    }
}

#[test]
fn real_save_resume_backup_restore_and_selected_reset_preserve_drafts_and_legacy_work() {
    use dlms_learning_lab::{
        backup,
        progress::{self, Attempt, Progress},
    };
    let all = pack::current().unwrap();
    let first = all.iter().find(|s| s.id == IDS[0]).unwrap();
    let second = all.iter().find(|s| s.id == IDS[1]).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(first, 10);
    let State::BootEntry { simulation } = &mut a.state else {
        panic!()
    };
    let d = definition(first);
    simulation
        .assignment
        .edit(d.initial.reference.clone())
        .unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    simulation
        .practice
        .edit("set default='unfinished workshop".into())
        .unwrap();
    assert!(progress::current_solved(&a, first));
    p.attempts.insert(first.id.clone(), a);
    let mut a = Attempt::new(second, 20);
    let State::BootEntry { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit("menuentry 'Unfinished' --id=recovery {\n".into())
        .unwrap();
    p.attempts.insert(second.id.clone(), a);
    let legacy = &all[0];
    p.attempts
        .insert(legacy.id.clone(), Attempt::new(legacy, 47));
    p.favorites.extend([first.id.clone(), legacy.id.clone()]);
    let baseline = serde_json::to_value(&p).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-grub-menu/profiles");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), baseline);
    assert!(progress::current_solved(&loaded.attempts[IDS[0]], first));
    assert!(!progress::current_solved(&loaded.attempts[IDS[1]], second));
    let archive = path.with_extension("backup.json");
    backup::export(&archive, &p, &all).unwrap();
    let inspected = backup::inspect(
        &archive,
        &all,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    let restored = backup::restore(&path, &inspected, &all).unwrap();
    assert!(restored.safety_copy.is_some());
    assert_eq!(
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap(),
        baseline
    );
    let mut forged: Progress = serde_json::from_value(baseline.clone()).unwrap();
    let State::BootEntry { simulation } = &mut forged.attempts.get_mut(IDS[0]).unwrap().state
    else {
        panic!()
    };
    simulation.assignment.source = definition(first).initial.starter.clone();
    assert!(!progress::current_solved(&forged.attempts[IDS[0]], first));
    assert!(progress::save_with_catalog(&path, &forged, &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    p.attempts.insert(first.id.clone(), Attempt::new(first, 30));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], first));
    assert_eq!(
        serde_json::to_value(&p.attempts[IDS[1]]).unwrap(),
        baseline["attempts"][IDS[1]]
    );
    assert_eq!(
        serde_json::to_value(&p.attempts[&legacy.id]).unwrap(),
        baseline["attempts"][&legacy.id]
    );
    assert_eq!(p.favorites, loaded.favorites);
}
