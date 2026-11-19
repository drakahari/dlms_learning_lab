use dlms_learning_lab::{
    boot_entry_workspace as b, course,
    engine::{self, Goal, State},
    pack, progress,
};
const IDS: [&str; 2] = ["lpic1-grub-persistent-default", "lpic1-grub-argument-scope"];
fn authored() -> Vec<pack::Scenario> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v3.json")).unwrap();
    v["scenarios"].as_array().unwrap()[5..]
        .iter()
        .map(|s| serde_json::from_value(s.clone()).unwrap())
        .collect()
}
fn definition(s: &pack::Scenario) -> &b::Definition {
    let Goal::BootEntry { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn pass(v: &[(bool, String)]) -> bool {
    v.iter().all(|r| r.0)
}
fn run(a: &mut b::Draft, t: &b::Task) {
    a.generate(t).unwrap();
    a.check(t).unwrap();
}
#[test]
fn taught_methods_generate_then_test_separate_practice_and_accept_equivalent_settings() {
    let labs = authored();
    assert_eq!(labs.len(), 2);
    for s in labs {
        let d = definition(&s);
        d.validate().unwrap();
        assert_eq!(s.objective, d.initial.instructions);
        let mut a = b::Session::new(d).unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(match &s.initial {
                State::BootEntry { simulation } => &**simulation,
                _ => panic!(),
            })
            .unwrap()
        );
        a.practice.edit(d.example.reference.clone()).unwrap();
        run(&mut a.practice, &d.example);
        assert!(pass(&a.practice.checks(&d.example)));
        assert!(!pass(&d.checks(&a)));
        assert_eq!(a.assignment.source, d.initial.starter);
        for common in ["", "ro"] {
            let source = d.initial.reference.replace(
                "GRUB_CMDLINE_LINUX=\"\"",
                &format!("GRUB_CMDLINE_LINUX='{common}'"),
            );
            a.assignment.edit(source).unwrap();
            run(&mut a.assignment, &d.initial);
            assert!(pass(&d.checks(&a)), "{:?}", d.checks(&a));
        }
        if s.id == IDS[0] {
            for selector in ["0", "rollback", "'Rollback Linux'", "\"Rollback Linux\""] {
                a.assignment
                    .edit(
                        d.initial
                            .reference
                            .replace("GRUB_DEFAULT=rollback", &format!("GRUB_DEFAULT={selector}")),
                    )
                    .unwrap();
                run(&mut a.assignment, &d.initial);
                assert!(pass(&d.checks(&a)));
            }
        }
        let mut state = s.initial.clone();
        for c in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn no_generate_outdated_menu_partial_and_wrong_but_bootable_results_cannot_pass() {
    for s in authored() {
        let d = definition(&s);
        let mut a = b::Session::new(d).unwrap();
        a.assignment.edit(d.initial.reference.clone()).unwrap();
        a.assignment.check(&d.initial).unwrap();
        assert!(!pass(&d.checks(&a)));
        assert!(d.checks(&a)[0].1.contains("Generate"));
        assert_eq!(
            a.assignment.report(&d.initial).unwrap_err().kind,
            dlms_learning_lab::boot_entry::ErrorKind::Invalid
        );
        run(&mut a.assignment, &d.initial);
        assert!(pass(&d.checks(&a)));
        let prior = a.assignment.generated.clone();
        a.assignment
            .edit(
                d.initial
                    .reference
                    .replace("GRUB_TIMEOUT=", "GRUB_TIMEOUT=0\n# was "),
            )
            .unwrap();
        assert_eq!(prior, a.assignment.generated);
        assert!(a.assignment.checked.is_none());
        assert!(!pass(&d.checks(&a)));
        a.assignment.check(&d.initial).unwrap();
        assert!(d.checks(&a)[0].1.contains("older generated menu"));
        run(&mut a.assignment, &d.initial);
        let wrong = d.checks(&a);
        assert!(!pass(&wrong));
        assert!(wrong.iter().any(|(ok, msg)| !ok && msg.contains("timeout")));
        a.assignment.edit(d.initial.starter.clone()).unwrap();
        run(&mut a.assignment, &d.initial);
        assert!(!pass(&d.checks(&a)));
        if s.id == IDS[1] {
            let partial = d
                .initial
                .reference
                .replace("GRUB_CMDLINE_LINUX=\"\"", "GRUB_CMDLINE_LINUX=\"rw\"")
                .replace(
                    "GRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"",
                    "GRUB_CMDLINE_LINUX_DEFAULT=\"ro quiet\"",
                );
            a.assignment.edit(partial).unwrap();
            run(&mut a.assignment, &d.initial);
            let checks = d.checks(&a);
            assert!(
                checks
                    .iter()
                    .any(|(ok, msg)| *ok && msg.contains("entry ordinary"))
            );
            assert!(
                checks
                    .iter()
                    .any(|(ok, msg)| !*ok && msg.contains("entry repair"))
            );
            a.assignment
                .edit(
                    d.initial
                        .reference
                        .replace("GRUB_CMDLINE_LINUX=\"\"", "GRUB_CMDLINE_LINUX=\"quiet\""),
                )
                .unwrap();
            run(&mut a.assignment, &d.initial);
            assert!(d.checks(&a).iter().any(|(ok, msg)| !*ok
                && msg.contains("recovery entry repair")
                && msg.contains("quiet")));
        }
        a.assignment.edit(d.initial.reference.clone()).unwrap();
        run(&mut a.assignment, &d.initial);
        assert!(pass(&d.checks(&a)));
    }
}
#[test]
fn source_owned_facts_saved_output_and_context_are_not_replaceable_authority() {
    for s in authored() {
        let d = definition(&s);
        let mut a = b::Session::new(d).unwrap();
        a.assignment.edit(d.initial.reference.clone()).unwrap();
        run(&mut a.assignment, &d.initial);
        let saved = serde_json::to_string(&a).unwrap();
        let restored: b::Session = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored, a);
        assert!(pass(&d.checks(&restored)));
        let mut forged = a.clone();
        forged
            .assignment
            .generated
            .as_mut()
            .unwrap()
            .menu
            .push_str("# changed\n");
        assert!(forged.bound(d).is_err());
        assert!(!pass(&d.checks(&forged)));
        let mut wrong = a.clone();
        wrong.context = "other-lab".into();
        assert!(!pass(&d.checks(&wrong)));
        let mut changed = d.clone();
        let root = "00112233-4455-6677-8899-aabbccddeeff";
        for t in [&mut changed.initial, &mut changed.example] {
            let old = t.required_root.clone();
            t.required_root = root.into();
            for r in &mut t.inventory.roots {
                if r.uuid == old {
                    r.uuid = root.into();
                }
            }
            let g = t.generator.as_mut().unwrap();
            g.templates = g.templates.replace(&old, root);
        }
        // Evaluate actual changed facts; do not fake a second definition with identical example IDs/UUIDs.
        changed.initial.validate().unwrap();
        let mut draft = b::Draft::new(&changed.initial);
        draft.edit(changed.initial.reference.clone()).unwrap();
        run(&mut draft, &changed.initial);
        assert!(pass(&draft.checks(&changed.initial)));
        assert!(draft.generated.as_ref().unwrap().menu.contains(root));
        assert_ne!(draft.generated, a.assignment.generated);
        let mut malformed = d.initial.clone();
        malformed
            .generator
            .as_mut()
            .unwrap()
            .normal_ids
            .push("missing".into());
        assert!(malformed.shape().is_err());
    }
}
#[test]
fn registered380_preserves_every378_scenario_unit_and_new_prerequisite_order() {
    let old = pack::accepted378().unwrap();
    let current = pack::accepted380().unwrap();
    assert_eq!((old.len(), current.len()), (378, 380));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..378]).unwrap()
    );
    let old_units = course::accepted378(&old).unwrap();
    let units = course::accepted380(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (140, 141));
    for u in old_units {
        assert_eq!(
            serde_json::to_value(&u).unwrap(),
            serde_json::to_value(units.iter().find(|v| v.id == u.id).unwrap()).unwrap()
        );
    }
    let at = units
        .iter()
        .position(|u| u.id == "linux-grub-persistent-configuration")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-grub-menu-recovery");
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at].prerequisites, ["linux-grub-menu-recovery"]);
    for (track, count) in [
        ("CCNA", 89),
        ("LPIC-1", 124),
        ("LPIC-2", 90),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn actual_progress_roundtrip_retains_generated_old_output_and_separate_unfinished_drafts() {
    let catalog = pack::current().unwrap();
    let s = catalog.iter().find(|s| s.id == IDS[0]).unwrap();
    let d = definition(s);
    let mut attempt = progress::Attempt::new(s, 1);
    let State::BootEntry { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.reference.clone())
        .unwrap();
    run(&mut simulation.assignment, &d.initial);
    simulation
        .assignment
        .edit("GRUB_TIMEOUT='unfinished".into())
        .unwrap();
    simulation
        .practice
        .edit(d.example.reference.clone())
        .unwrap();
    run(&mut simulation.practice, &d.example);
    let mut p = progress::Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), attempt);
    p.favorites.insert(s.id.clone());
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-grub-menu/persistent-profiles");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    progress::save_with_catalog(&file, &p, &catalog).unwrap();
    let restored = progress::load(&file, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(!progress::current_solved(&restored.attempts[IDS[0]], s));
    let archive = file.with_extension("backup.json");
    dlms_learning_lab::backup::export(&archive, &restored, &catalog).unwrap();
    let inspected = dlms_learning_lab::backup::inspect(
        &archive,
        &catalog,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    let restored_backup = dlms_learning_lab::backup::restore(&file, &inspected, &catalog).unwrap();
    assert!(restored_backup.safety_copy.is_some());
    assert_eq!(
        serde_json::to_value(progress::load(&file, &catalog).unwrap()).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    let before = std::fs::read(&file).unwrap();
    let mut invalid = restored;
    let State::BootEntry { simulation } = &mut invalid.attempts.get_mut(IDS[0]).unwrap().state
    else {
        panic!()
    };
    simulation.assignment.generated.as_mut().unwrap().timeout += 1;
    assert!(progress::save_with_catalog(&file, &invalid, &catalog).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
}
#[test]
fn invalid_generator_packs_and_untested_taught_solutions_fail_before_registration() {
    let mut valid: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v3.json")).unwrap();
    let identities = valid["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    for s in valid["scenarios"].as_array_mut().unwrap() {
        if let Some(v) = s.get_mut("prerequisites") {
            v.as_array_mut()
                .unwrap()
                .retain(|p| identities.iter().any(|id| Some(id.as_str()) == p.as_str()));
        }
    }
    assert_eq!(pack::load(&valid.to_string()).unwrap().len(), 7);
    for mutation in 0..5 {
        let mut bad = valid.clone();
        let lab = &mut bad["scenarios"][5];
        match mutation {
            0 => {
                lab["goal"]["definition"]["initial"]["generator"]["normal_ids"] =
                    serde_json::json!([])
            }
            1 => {
                lab["goal"]["definition"]["initial"]["generator"]["execute_host_script"] =
                    serde_json::json!(true)
            }
            2 => {
                lab["goal"]["definition"]["initial"]["generator"]["timeout"] =
                    serde_json::json!(301)
            }
            3 => {
                let source = lab["goal"]["definition"]["initial"]["reference"]
                    .as_str()
                    .unwrap()
                    .to_owned();
                lab["goal"]["definition"]["initial"]["reference"] = serde_json::json!(
                    source.replace("GRUB_DEFAULT=rollback", "GRUB_DEFAULT=missing")
                );
            }
            _ => {
                let source = lab["goal"]["definition"]["initial"]["reference"]
                    .as_str()
                    .unwrap();
                lab["solution"] = serde_json::json!([format!(
                    "BOOT ENTRY WORK\n{}",
                    serde_json::json!({"source":source})
                )]);
            }
        }
        assert!(pack::load(&bad.to_string()).is_err(), "mutation {mutation}");
    }
}
