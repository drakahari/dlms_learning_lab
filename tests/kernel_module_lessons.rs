use dlms_learning_lab::{
    engine::{self, State},
    kernel_module_source as source, kernel_module_workspace as work, pack, progress,
};
fn authored() -> Vec<pack::Scenario> {
    let old = pack::accepted300().unwrap();
    let mut texts = old
        .chunks(8)
        .map(|chunk| {
            serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":chunk}))
                .unwrap()
        })
        .collect::<Vec<_>>();
    texts.push(include_str!("../scenarios/kernel-module-lessons-v1.json").into());
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
        .unwrap()
        .into_iter()
        .filter(|s| s.id.starts_with("lpic1-module-") || s.id.starts_with("lpic2-module-"))
        .collect()
}
#[test]
fn every_authored_task_and_separate_worked_method_reaches_real_typed_outcomes() {
    let labs = authored();
    assert_eq!(labs.len(), 3);
    for s in &labs {
        let d = source::source(s).unwrap();
        d.validate_reachability().unwrap();
        if s.id == "lpic1-module-device-driver" {
            assert!(s.guide[0].why.contains("core of Linux"));
            assert!(s.guide[1].why.contains("must be loaded first"));
        }
        if s.id == "lpic1-module-index-repair" {
            assert!(s.guide[0].why.contains("list modprobe uses"));
        }
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for input in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let mut session = work::Session::new(d).unwrap();
        let initial = session.assignment.clone();
        for step in &d.example_steps {
            session.action(d, &step.action(), true).unwrap();
        }
        assert!(session.check(d, true).unwrap().iter().all(|r| r.0));
        assert_eq!(session.assignment, initial);
        assert!(d.checks(&session).iter().any(|r| !r.0));
        assert!(
            s.guide
                .iter()
                .all(|g| !g.why.is_empty() && !g.action.is_empty() && !g.commands.is_empty())
        );
    }
}
#[test]
fn plausible_incomplete_one_off_and_protected_provider_repairs_fail_with_specific_feedback() {
    for s in authored() {
        let d = source::source(&s).unwrap();
        let mut session = work::Session::new(d).unwrap();
        let out = session.run(d, "lspci -k", false).unwrap();
        assert!(out.contains("Teaching sensor controller"));
        assert!(session.check(d, false).unwrap().iter().any(|r| !r.0));
        if s.id == "lpic1-module-index-repair" {
            assert!(
                session
                    .run(d, "modprobe lab_sensor", false)
                    .unwrap()
                    .contains("not found")
            );
            for n in ["sensor_bus", "lab_sensor"] {
                session
                    .run(
                        d,
                        &format!("insmod {}", d.assignment.catalogue.modules[n].path),
                        false,
                    )
                    .unwrap();
            }
            let rows = session.check(d, false).unwrap();
            assert!(rows[0].0);
            assert!(!rows[4].0);
            assert!(rows[4].1.contains("one-off"));
        } else if s.id == "lpic2-module-options-reload" {
            session
                .edit(d, "options lab_sensor queues=4\n".into(), false)
                .unwrap();
            session.save(d, false).unwrap();
            session.run(d, "modprobe lab_sensor", false).unwrap();
            let rows = session.check(d, false).unwrap();
            assert!(!rows[0].0);
            assert!(rows[2].0);
        }
        for step in &d.reference {
            session.action(d, &step.action(), false).unwrap();
        }
        assert!(session.check(d, false).unwrap().iter().all(|r| r.0));
        session.run(d, "rmmod audit_link", false).unwrap();
        assert!(!session.assignment.checked);
        assert!(!session.check(d, false).unwrap()[1].0);
    }
}
#[test]
fn equivalent_manual_dependency_loading_and_saved_numeric_values_are_accepted() {
    for s in authored() {
        let d = source::source(&s).unwrap();
        let mut session = work::Session::new(d).unwrap();
        if d.assignment.target.saved_options {
            session.edit(d,"# whitespace and integer spelling are not a command contract\noptions lab-sensor queues=04\n".into(),false).unwrap();
            session.save(d, false).unwrap();
            session.run(d, "modprobe -r lab_sensor", false).unwrap();
        }
        for n in ["sensor_bus", "lab_sensor"] {
            let extra = if n == "lab_sensor" {
                format!(" queues={}", d.assignment.target.parameters["queues"])
            } else {
                String::new()
            };
            session
                .run(
                    d,
                    &format!("insmod {}{extra}", d.assignment.catalogue.modules[n].path),
                    false,
                )
                .unwrap();
        }
        if d.assignment.target.current_index {
            session.run(d, "depmod -a", false).unwrap();
        }
        assert!(
            session.check(d, false).unwrap().iter().all(|r| r.0),
            "{}",
            s.id
        );
    }
}
#[test]
fn actual_progress_save_load_preserves_two_drafts_favorites_and_honest_current_completion() {
    let labs = authored();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        p.attempts
            .insert(s.id.clone(), progress::Attempt::new(s, 1));
        let a = p.attempts.get_mut(&s.id).unwrap();
        for cmd in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, cmd).unwrap();
        }
        let State::KernelModules { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "modinfo lab_sensor".into();
        simulation.practice.command = "dmesg".into();
        assert!(progress::current_solved(a, s));
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(loaded.favorites, p.favorites);
    for s in &labs {
        let a = &loaded.attempts[&s.id];
        source::context(a, s).unwrap();
        assert!(progress::current_solved(a, s));
        let State::KernelModules { simulation } = &a.state else {
            panic!()
        };
        assert_eq!(simulation.assignment.command, "modinfo lab_sensor");
        assert_eq!(simulation.practice.command, "dmesg");
    }
    assert!(
        progress::save(&path, &p).is_err(),
        "A source-free saver must not authorize module state"
    );
    let s = &labs[0];
    let mut a = loaded.attempts[&s.id].clone();
    let State::KernelModules { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.runtime.loaded.clear();
    simulation.assignment.checked = true;
    assert!(!progress::current_solved(&a, s));
    let before = a.state.clone();
    let badgoal = pack::accepted300().unwrap().remove(0).goal;
    assert!(engine::execute_for_goal(&mut a.state, &badgoal, "modprobe lab_sensor").is_err());
    assert_eq!(before, a.state);
    assert!(!progress::current_solved(&progress::Attempt::new(s, 2), s));
}

#[test]
fn registered303_catalog_preserves_all300_scenarios_and98_units_exactly() {
    let before = pack::accepted300().unwrap();
    let all = pack::accepted303().unwrap();
    assert_eq!(all.len(), 303);
    let old_ids = before
        .iter()
        .map(|s| s.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let retained = all
        .iter()
        .filter(|s| old_ids.contains(s.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let old_units = dlms_learning_lab::course::accepted300(&before).unwrap();
    let units = dlms_learning_lab::course::accepted303(&all).unwrap();
    assert_eq!(units.len(), 100);
    let ids = old_units
        .iter()
        .map(|u| u.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let retained = units
        .iter()
        .filter(|u| ids.contains(u.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    for s in authored() {
        let current = all.iter().find(|v| v.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(current).unwrap(),
            serde_json::to_value(s).unwrap()
        );
    }
    for (t, n) in [
        ("CCNA", 74),
        ("LPIC-1", 107),
        ("LPIC-2", 46),
        ("Bash/Shell", 76),
    ] {
        assert_eq!(all.iter().filter(|s| s.track == t).count(), n);
    }
}
