//! Authored named-driver task is tested before catalog admission.
use dlms_learning_lab::{
    engine::{self, State},
    kernel_module_source as source, kernel_module_workspace as work, pack, progress,
};
fn authored() -> pack::Scenario {
    let prior = pack::accepted303().unwrap();
    let mut texts = prior
        .chunks(8)
        .map(|c| {
            serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":c})).unwrap()
        })
        .collect::<Vec<_>>();
    texts.push(include_str!("../scenarios/kernel-module-names-lessons-v1.json").into());
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-module-name-handover")
        .unwrap()
}
#[test]
fn authored_named_handover_teaches_then_reaches_current_and_both_future_outcomes() {
    let lab = authored();
    let d = source::source(&lab).unwrap();
    d.validate_reachability().unwrap();
    assert!(d.assignment.named() && d.example.named());
    assert_eq!(d.assignment.target.request_names.len(), 2);
    assert!(
        d.assignment
            .catalogue
            .devices
            .iter()
            .any(|dvc| dvc.driver == d.assignment.target.module)
    );
    let mut state = lab.initial.clone();
    assert!(!engine::solved(&state, &lab.goal));
    for command in &lab.solution {
        engine::execute_for_goal(&mut state, &lab.goal, command).unwrap();
    }
    assert!(engine::solved(&state, &lab.goal));
    let mut session = work::Session::new(d).unwrap();
    let assignment = session.assignment.clone();
    for step in &d.example_steps {
        session.action(d, &step.action(), true).unwrap();
    }
    assert!(session.check(d, true).unwrap().iter().all(|r| r.0));
    assert_eq!(session.assignment, assignment);
    assert!(d.checks(&session).iter().any(|r| !r.0));
    assert!(lab.guide[0].why.contains("not another loaded module"));
    assert!(
        lab.guide[1]
            .why
            .contains("file contents, not terminal commands")
    );
    assert!(
        lab.guide
            .iter()
            .all(|g| !g.action.is_empty() && !g.commands.is_empty())
    );
}
#[test]
fn command_success_missing_name_partial_repair_and_saved_edits_receive_distinct_case_feedback() {
    let lab = authored();
    let d = source::source(&lab).unwrap();
    let mut s = work::Session::new(d).unwrap();
    assert!(
        s.run(d, "modprobe capture_current", false)
            .unwrap()
            .is_empty()
    );
    let rows = s.check(d, false).unwrap();
    assert!(!rows[0].0 && !rows[4].0 && !rows[5].0);
    assert!(rows[4].1.contains("returned success"));
    assert!(rows[5].1.contains("not supplied or configured"));
    s.edit(d,"options lab_sensor queues=1\nalias capture_current lab_sensor\nalias audit_query audit_link\n".into(),false).unwrap();
    s.save(d, false).unwrap();
    s.run(d, "modprobe capture_current", false).unwrap();
    let rows = s.check(d, false).unwrap();
    assert!(rows[0].0 && rows[4].0 && !rows[5].0);
    for step in &d.reference {
        s.action(d, &step.action(), false).unwrap();
    }
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.edit(d,"options lab_sensor queues=1\nalias capture_current audit_link\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n".into(),false).unwrap();
    assert!(!s.assignment.checked);
    s.save(d, false).unwrap();
    let rows = s.check(d, false).unwrap();
    assert!(rows[0].0 && !rows[4].0 && rows[5].0);
    s.edit(d, "alias capture_current capture_legacy\n".into(), false)
        .unwrap();
    s.save(d, false).unwrap();
    let before = s.clone();
    assert!(s.run(d, "modprobe capture_current", false).is_err());
    assert_eq!(s, before);
    assert!(
        s.check(d, false)
            .unwrap()
            .iter()
            .any(|r| !r.0 && r.1.contains("chains"))
    );
}
#[test]
fn equivalent_supported_configuration_and_manual_activation_pass_without_exact_command_grading() {
    let lab = authored();
    let d = source::source(&lab).unwrap();
    let mut s = work::Session::new(d).unwrap();
    s.edit(d,"# same mappings, different order and accepted name spelling\nalias capture-legacy lab-sensor\noptions lab_sensor queues=01\nalias audit-query audit-link\nalias capture-current lab-sensor\n".into(),false).unwrap();
    s.save(d, false).unwrap();
    for n in ["sensor_bus", "lab_sensor"] {
        s.run(
            d,
            &format!("insmod {}", d.assignment.catalogue.modules[n].path),
            false,
        )
        .unwrap();
    }
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.run(d, "rmmod audit_link", false).unwrap();
    assert!(!s.check(d, false).unwrap()[1].0);
    s.run(d, "modprobe audit_link", false).unwrap();
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.edit(d,"options lab_sensor queues=1\nalias capture_current lab_sensor\nalias capture_legacy lab_sensor\n".into(),false).unwrap();
    s.save(d, false).unwrap();
    assert!(!s.check(d, false).unwrap()[3].0);
}
#[test]
fn new_task_drafts_favorites_current_results_and_reset_survive_actual_safe_progress_round_trip() {
    let lab = authored();
    source::source(&lab).unwrap();
    let mut a = progress::Attempt::new(&lab, 1);
    for c in &lab.solution {
        engine::execute_for_goal(&mut a.state, &lab.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, &lab));
    let State::KernelModules { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.command = "modprobe capture_legacy".into();
    simulation.practice.command = "lspci -k".into();
    simulation.practice.config_draft = "# unfinished independent example draft\n".into();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    p.favorites.insert(lab.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&lab)).unwrap();
    let read = progress::load(&path, std::slice::from_ref(&lab)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&read).unwrap()
    );
    assert!(progress::current_solved(&read.attempts[&lab.id], &lab));
    let mut broken = read.attempts[&lab.id].clone();
    let State::KernelModules { simulation } = &mut broken.state else {
        panic!()
    };
    simulation.assignment.runtime.config = "alias capture_current audit_link\n".into();
    simulation.assignment.checked = true;
    assert!(!progress::current_solved(&broken, &lab));
    assert!(!progress::current_solved(
        &progress::Attempt::new(&lab, 2),
        &lab
    ));
}

#[test]
fn admitted_named_handover_preserves_all303_scenarios_and100_units_exactly() {
    let old = pack::accepted303().unwrap();
    let all = pack::accepted304().unwrap();
    assert_eq!(all.len(), 304);
    let ids = old
        .iter()
        .map(|s| s.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let retained = all
        .iter()
        .filter(|s| ids.contains(s.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = dlms_learning_lab::course::accepted303(&old).unwrap();
    let units = dlms_learning_lab::course::accepted304(&all).unwrap();
    assert_eq!(units.len(), 101);
    let ids = before
        .iter()
        .map(|u| u.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let retained = units
        .iter()
        .filter(|u| ids.contains(u.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let expected = authored();
    let actual = all.iter().find(|s| s.id == expected.id).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    for (track, n) in [
        ("CCNA", 74),
        ("LPIC-1", 107),
        ("LPIC-2", 47),
        ("Bash/Shell", 76),
    ] {
        assert_eq!(all.iter().filter(|s| s.track == track).count(), n);
    }
}
