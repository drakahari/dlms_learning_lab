#[path = "support/kernel_module_workspace_fixture.rs"]
mod fixture;
use dlms_learning_lab::{kernel_module_workspace::*, kernel_modules as core};
#[test]
fn source_bound_roles_separate_examples_assignment_and_current_state_grading() {
    for case in 0..3 {
        let d = fixture::definition(case);
        d.validate_reachability().unwrap();
        let mut s = Session::new(&d).unwrap();
        let old = s.assignment.clone();
        for step in &d.example_steps {
            s.action(&d, &step.action(), true).unwrap();
        }
        assert!(s.check(&d, true).unwrap().iter().all(|r| r.0));
        assert_eq!(s.assignment, old);
        assert!(d.checks(&s).iter().any(|r| !r.0));
        for step in &d.reference {
            s.action(&d, &step.action(), false).unwrap();
        }
        assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
        s.run(&d, "modprobe -r lab_sensor", false).unwrap();
        assert!(!s.assignment.checked);
        assert!(s.check(&d, false).unwrap().iter().any(|r| !r.0));
        let mut wrong = s.clone();
        wrong.origin = "other-task".into();
        assert!(d.bound(&wrong).is_err());
        let mut wrong = s.clone();
        wrong.assignment.role = Role::Practice;
        assert!(d.bound(&wrong).is_err());
        let mut wrong = s.clone();
        wrong.assignment.runtime.loaded.clear();
        wrong.assignment.checked = true;
        assert!(d.checks(&wrong).iter().any(|r| !r.0));
    }
}
#[test]
fn configuration_feedback_distinguishes_saved_effective_and_unsaved_work() {
    let d = fixture::definition(1);
    let mut s = Session::new(&d).unwrap();
    let rows = s.check(&d, false).unwrap();
    assert!(!rows[0].0 && rows[0].1.contains("repeating modprobe"));
    assert!(!rows[2].0);
    s.edit(&d, "options lab_sensor queues=4\n".into(), false)
        .unwrap();
    assert!(!s.assignment.checked);
    s.save(&d, false).unwrap();
    s.run(&d, "modprobe lab_sensor", false).unwrap();
    let rows = s.check(&d, false).unwrap();
    assert!(!rows[0].0);
    assert!(rows[2].0);
    s.run(&d, "modprobe -r lab_sensor", false).unwrap();
    s.run(&d, "modprobe lab_sensor", false).unwrap();
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    s.edit(&d, "options lab_sensor queues=2\n".into(), false)
        .unwrap();
    assert!(!s.assignment.checked);
    let rows = s.check(&d, false).unwrap();
    assert!(rows[0].0);
    assert!(!rows.last().unwrap().0);
    s.save(&d, false).unwrap();
    let rows = s.check(&d, false).unwrap();
    assert!(rows[0].0);
    assert!(
        !rows[2].0,
        "Current success needs durable saved configuration too"
    );
    s.edit(&d, "install lab_sensor /bin/sh\n".into(), false)
        .unwrap();
    s.save(&d, false).unwrap();
    s.run(&d, "modprobe -r lab_sensor", false).unwrap();
    let before = s.clone();
    assert!(s.run(&d, "modprobe lab_sensor", false).is_err());
    assert_eq!(s, before);
}
#[test]
fn alternate_manual_loading_passes_but_protected_provider_and_index_shortcuts_fail() {
    for case in [0, 2] {
        let d = fixture::definition(case);
        let mut s = Session::new(&d).unwrap();
        for n in ["sensor_bus", "lab_sensor"] {
            s.run(
                &d,
                &format!("insmod {}", d.assignment.catalogue.modules[n].path),
                false,
            )
            .unwrap();
        }
        let rows = s.check(&d, false).unwrap();
        assert!(rows[0].0 && rows[1].0);
        assert_eq!(rows[4].0, case == 0);
        if case == 2 {
            s.run(&d, "depmod -a", false).unwrap();
            assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
        }
        s.run(&d, "rmmod audit_link", false).unwrap();
        assert!(!s.check(&d, false).unwrap()[1].0);
    }
}
#[test]
fn drafts_results_and_bounded_transcripts_round_trip_without_source_authority() {
    let d = fixture::definition(0);
    let mut s = Session::new(&d).unwrap();
    s.assignment.command = "modinfo lab_sensor".into();
    s.practice.command = "lsmod".into();
    s.assignment
        .transcript
        .push("Actual diagnostic output".into());
    let raw = serde_json::to_vec(&s).unwrap();
    let copy: Session = serde_json::from_slice(&raw).unwrap();
    assert_eq!(s, copy);
    d.bound(&copy).unwrap();
    let mut bad = s.clone();
    bad.assignment.command = "x".repeat(core::MAX_COMMAND + 1);
    assert!(d.bound(&bad).is_err());
    let mut bad = s.clone();
    bad.assignment.transcript = vec!["x".repeat(8193)];
    assert!(d.bound(&bad).is_err());
    let mut bad = s.clone();
    bad.assignment.runtime.index.insert("forged-module".into());
    assert!(d.bound(&bad).is_err());
    let before = s.clone();
    assert!(s.edit(&d, "x".repeat(core::MAX_CONFIG + 1), false).is_err());
    assert_eq!(s, before);
    assert!(
        s.run(&d, "@module-save \"options lab_sensor queues=4\"", false)
            .is_err()
    );
    assert_eq!(
        s, before,
        "Native terminal cannot execute author-only actions"
    );
}

#[test]
fn promised_future_insertion_is_probed_without_mutating_or_granting_credit() {
    let d = fixture::definition(1);
    let mut s = Session::new(&d).unwrap();
    for step in &d.reference {
        s.action(&d, &step.action(), false).unwrap();
    }
    s.check(&d, false).unwrap();
    let before = s.assignment.runtime.clone();
    assert!(d.checks(&s).iter().all(|r| r.0));
    assert_eq!(s.assignment.runtime, before);
    s.assignment.runtime.index.remove("lab_sensor");
    let rows = d.checks(&s);
    assert!(rows[0].0 && rows[2].0);
    assert!(!rows[4].0);
    assert!(rows[4].1.contains("Future insertion"));
    s.run(&d, "depmod -a", false).unwrap();
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
}
