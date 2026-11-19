//! Source-held named-request outcomes; these fixtures are not curriculum labs.
#[path = "support/kernel_module_name_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::{self, Goal, State},
    kernel_module_source as source,
    kernel_module_workspace::*,
    pack, progress,
};
#[test]
fn two_named_requests_grade_actual_future_activation_and_leave_live_state_unchanged() {
    let d = fixture::definition();
    d.validate_reachability().unwrap();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    for step in &d.example_steps {
        s.action(&d, &step.action(), true).unwrap();
    }
    assert!(s.check(&d, true).unwrap().iter().all(|r| r.0));
    assert_eq!(s.assignment, assignment);
    assert!(d.checks(&s).iter().any(|r| !r.0));
    // The wrong name initially returns success for an unrelated already loaded module.
    let out = s.run(&d, "modprobe capture_current", false).unwrap();
    assert!(out.is_empty());
    assert!(!s.assignment.runtime.loaded.contains_key("lab_sensor"));
    let rows = s.check(&d, false).unwrap();
    assert!(!rows[0].0 && !rows[4].0);
    assert!(rows[4].1.contains("returned success"));
    assert!(rows[5].0, "the correctly saved legacy name still works");
    for step in &d.reference {
        s.action(&d, &step.action(), false).unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    let before = s.assignment.runtime.clone();
    assert!(d.checks(&s).iter().all(|r| r.0));
    assert_eq!(
        s.assignment.runtime, before,
        "grading clones must not reload the learner's state"
    );
    s.edit(&d,"alias capture_current audit_link\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n".into(),false).unwrap();
    s.save(&d, false).unwrap();
    assert!(!s.assignment.checked);
    let rows = s.check(&d, false).unwrap();
    assert!(rows[0].0);
    assert!(
        !rows[4].0 && rows[5].0,
        "current driver and individual request results must stay distinct"
    );
}
#[test]
fn direct_loads_cannot_replace_saved_requests_and_preservation_and_syntax_errors_stay_specific() {
    let d = fixture::definition();
    let mut s = Session::new(&d).unwrap();
    s.run(&d, "modprobe lab_sensor", false).unwrap();
    let rows = s.check(&d, false).unwrap();
    assert!(rows[0].0 && !rows[4].0);
    s.edit(
        &d,
        "alias capture_current lab_sensor\nalias capture_legacy lab_sensor\n".into(),
        false,
    )
    .unwrap();
    s.save(&d, false).unwrap();
    let rows = s.check(&d, false).unwrap();
    assert!(
        !rows[3].0 && rows[4].0 && rows[5].0,
        "unrelated audit_query must be preserved even when both required requests work"
    );
    s.edit(&d, "alias capture_current capture_legacy\n".into(), false)
        .unwrap();
    s.save(&d, false).unwrap();
    let before = s.clone();
    assert!(s.run(&d, "modprobe capture_current", false).is_err());
    assert_eq!(s, before);
    let rows = s.check(&d, false).unwrap();
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("chains")));
    for step in &d.reference {
        s.action(&d, &step.action(), false).unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    s.run(&d, "rmmod audit_link", false).unwrap();
    assert!(!s.check(&d, false).unwrap()[1].0);
}
#[test]
fn a_wrong_module_that_loads_the_driver_as_a_dependency_is_not_a_false_positive() {
    let mut d = fixture::definition();
    let mut wrapper = d.assignment.catalogue.modules["audit_link"].clone();
    wrapper.path = "/lib/modules/6.8.0-lab/kernel/drivers/batch_wrapper.ko".into();
    wrapper.dependencies = vec!["lab_sensor".into()];
    d.assignment
        .catalogue
        .modules
        .insert("batch_wrapper".into(), wrapper);
    d.assignment.initial.index.insert("batch_wrapper".into());
    d.validate_reachability().unwrap();
    let mut s = Session::new(&d).unwrap();
    s.run(&d, "modprobe lab_sensor", false).unwrap();
    s.edit(&d,"alias capture_current batch_wrapper\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n".into(),false).unwrap();
    s.save(&d, false).unwrap();
    let rows = s.check(&d, false).unwrap();
    assert!(rows[0].0 && !rows[4].0 && rows[5].0);
    assert!(rows[4].1.contains("unrelated provider"));
    assert!(!s.assignment.runtime.loaded.contains_key("batch_wrapper"));
}
#[test]
fn equivalent_spelling_order_and_manual_current_activation_pass_but_missing_index_does_not() {
    let d = fixture::definition();
    let mut s = Session::new(&d).unwrap();
    s.edit(&d,"# order and hyphen spelling do not define the grade\nalias audit-query audit-link\nalias capture-legacy lab-sensor\nalias capture-current lab-sensor\n".into(),false).unwrap();
    s.save(&d, false).unwrap();
    for n in ["sensor_bus", "lab_sensor"] {
        s.run(
            &d,
            &format!("insmod {}", d.assignment.catalogue.modules[n].path),
            false,
        )
        .unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    s.assignment.runtime.index.remove("lab_sensor");
    let rows = d.checks(&s);
    assert!(rows[0].0);
    assert!(!rows[4].0 && !rows[5].0);
    s.run(&d, "depmod -a", false).unwrap();
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    let mut invalid = d.clone();
    invalid.assignment.target.saved_options = true;
    assert!(invalid.validate().is_err());
    let mut invalid = d.clone();
    invalid.assignment.target.current_index = true;
    assert!(invalid.validate().is_err());
    let mut invalid = d.clone();
    invalid
        .assignment
        .target
        .request_names
        .push("capture-current".into());
    assert!(invalid.validate().is_err());
}
#[test]
fn old_targets_serialize_exactly_and_new_work_is_source_bound_on_actual_progress_round_trip() {
    let old = pack::ccna_pilot().unwrap();
    let s = old
        .iter()
        .find(|s| s.id == "lpic1-module-device-driver")
        .unwrap();
    let d = source::source(s).unwrap();
    assert!(
        !serde_json::to_value(&d.assignment.target)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("request_names")
    );
    let mut d = fixture::definition();
    let mut synthetic = s.clone();
    d.source_id = synthetic.id.clone();
    synthetic.goal = Goal::KernelModules {
        definition: Box::new(d.clone()),
    };
    synthetic.initial = State::KernelModules {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    source::source(&synthetic).unwrap();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut attempt = progress::Attempt::new(&synthetic, 1);
    for step in &d.reference {
        engine::execute_for_goal(&mut attempt.state, &synthetic.goal, &step.action()).unwrap();
    }
    engine::execute_for_goal(&mut attempt.state, &synthetic.goal, CHECK_ACTION).unwrap();
    assert!(progress::current_solved(&attempt, &synthetic));
    let State::KernelModules { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation.assignment.command = "modprobe capture_current".into();
    simulation.practice.command = "unfinished example".into();
    simulation.practice.config_draft = "# separate unsaved practice note\n".into();
    p.favorites.insert(synthetic.id.clone());
    p.attempts.insert(synthetic.id.clone(), attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &[synthetic.clone()]).unwrap();
    let read = progress::load(&path, &[synthetic.clone()]).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&read).unwrap()
    );
    assert!(progress::current_solved(
        &read.attempts[&synthetic.id],
        &synthetic
    ));
    assert!(
        !progress::current_solved(&read.attempts[&synthetic.id], s),
        "old source must not acquire named grading from saved config"
    );
    let mut old_attempt = read.attempts[&synthetic.id].clone();
    let before = old_attempt.state.clone();
    let result =
        engine::execute_for_goal(&mut old_attempt.state, &s.goal, "modprobe capture_current")
            .unwrap();
    assert!(result.contains("Failed"));
    assert_eq!(
        old_attempt.state, before,
        "saved config cannot enable alias execution in the old source"
    );
    let mut fresh = Session::new(&d).unwrap();
    let before = fresh.clone();
    assert!(
        fresh
            .run(&d, "@module-save \"alias x lab_sensor\"", false)
            .is_err()
    );
    assert_eq!(fresh, before);
}
