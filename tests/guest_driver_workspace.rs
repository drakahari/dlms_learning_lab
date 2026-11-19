#[path = "support/guest_driver_tasks.rs"]
mod setup;
use dlms_learning_lab::{guest_driver_workspace::*, rpm_packages as rpm};
use setup::{definition, run};

#[test]
fn taught_methods_reach_all_outcomes_and_example_never_earns_assignment_credit() {
    let d = definition();
    d.validate_reachability().unwrap();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    for a in &d.example_steps {
        s.action(&d, a, true).unwrap();
    }
    assert!(s.practice.checked);
    assert!(
        d.example
            .checks(&s.practice.runtime)
            .iter()
            .all(|r| r.passed)
    );
    assert_eq!(s.assignment, assignment);
    assert!(!d.current(&s));
    for a in &d.reference {
        s.action(&d, a, false).unwrap();
    }
    assert!(d.current(&s));
    assert!(s.assignment.output.contains("$ modprobe lab_sensor"));
    assert!(s.practice.output.contains("$ modprobe guide_reader"));
}

#[test]
fn partial_install_and_current_only_repair_fail_with_specific_feedback_and_success_invalidates() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.action(&d, &d.reference[0], false).unwrap();
    let rows = s.action(&d, &Action::Check, false).unwrap();
    assert!(!rows[0].passed && !rows[1].passed);
    assert!(rows[0].detail.contains("installing a package alone"));
    s.action(&d, &run("depmod -a"), false).unwrap();
    let rows = s.action(&d, &Action::Check, false).unwrap();
    assert!(!rows[0].passed && rows[1].passed);
    assert!(!d.current(&s));
    s.action(&d, &run("modprobe lab_sensor"), false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.current(&s));
    let erase = Action::Package {
        action: rpm::Action::EraseSet {
            packages: vec![
                d.assignment.facts.packages.archives["support-current"]
                    .identity
                    .clone(),
            ],
        },
    };
    s.action(&d, &erase, false).unwrap();
    assert!(!s.assignment.checked && !d.current(&s));
    let rows = s.action(&d, &Action::Check, false).unwrap();
    assert!(rows[0].passed && !rows[1].passed);
    assert!(!d.current(&s));
    assert!(
        rows[1].detail.contains("running instance alone")
            && rows[1].detail.contains("No such file")
    );
    s.assignment.checked = true;
    assert!(
        !d.current(&s),
        "A saved check flag is not successful grading"
    );
}

#[test]
fn observation_and_unsupported_commands_do_not_break_or_invent_correct_configuration() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    for a in &d.reference {
        s.action(&d, a, false).unwrap();
    }
    assert!(d.current(&s));
    s.action(&d, &run("lsmod"), false).unwrap();
    assert!(d.current(&s));
    assert!(s.assignment.output.contains("Module"));
    let before = s.clone();
    assert!(s.action(&d, &run("rpm -i host.rpm"), false).is_err());
    assert_eq!(s, before);
    assert!(s.action(&d, &run("sh -c anything"), false).is_err());
    assert_eq!(s, before);
    // An imported saved flag must not hide a changed underlying virtual image.
    s.assignment
        .runtime
        .remove_object(&d.assignment.facts, "lab_sensor")
        .unwrap();
    assert!(s.assignment.checked);
    assert!(!d.current(&s));
}

#[test]
fn independent_drafts_restore_from_disk_and_reset_only_selected_scope() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.assignment.input = "modinfo lab_sensor".into();
    s.practice.input = "uname -r".into();
    s.action(&d, &d.example_steps[0], true).unwrap();
    s.action(&d, &d.reference[0], false).unwrap();
    s.view = dlms_learning_lab::script_teaching::View::Assignment;
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("guest-drafts.json");
    let raw = serde_json::to_vec(&s).unwrap();
    std::fs::write(&path, &raw).unwrap();
    let mut restored: Session = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(!d.current(&restored));
    let assignment = restored.assignment.clone();
    restored.reset(&d, true).unwrap();
    assert_eq!(restored.assignment, assignment);
    let example = restored.practice.clone();
    restored.reset(&d, false).unwrap();
    assert_eq!(restored.practice, example);
    assert_eq!(restored.assignment.runtime, d.assignment.initial);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        raw,
        "Reset never touches an unrelated retained backup"
    );
}

#[test]
fn invalid_goals_references_and_saved_authority_are_refused_without_false_credit() {
    let d = definition();
    let mut bad = d.clone();
    bad.assignment.devices.push("00:05.0".into());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.protected_drivers = vec!["not_loaded".into()];
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.reference = vec![Action::Check];
    assert!(bad.validate_reachability().is_err());
    let mut bad = d.clone();
    bad.example = bad.assignment.clone();
    assert!(
        bad.validate().is_err(),
        "The same supplied assignment is not a separate teaching example"
    );
    let mut bad = d.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
    let mut s = Session::new(&d).unwrap();
    let before = s.clone();
    assert!(s.action(&d, &run(&"x".repeat(4097)), false).is_err());
    assert_eq!(s, before);
    s.assignment.runtime.packages.available.clear();
    assert!(d.bound(&s).is_err());
    assert!(!d.current(&s));
    let mut raw = serde_json::to_value(&before).unwrap();
    raw["assignment"]["input"] = serde_json::json!("x".repeat(4097));
    assert!(serde_json::from_value::<Session>(raw).is_err());
    let mut bad = before.clone();
    bad.source_id = "another-lesson".into();
    assert!(d.bound(&bad).is_err());
    let mut bad = before.clone();
    bad.schema_version = 2;
    assert!(d.bound(&bad).is_err());
    let mut bad = before.clone();
    bad.assignment.runtime.modules.config = "# changed\n".into();
    assert!(d.bound(&bad).is_err());
    let mut bad = before.clone();
    bad.assignment.selected_archive = "foreign".into();
    assert!(d.bound(&bad).is_err());
}

#[test]
fn terminal_history_is_bounded_utf8_and_never_changes_independent_configuration() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.runtime.clone();
    let practice = s.practice.clone();
    for _ in 0..240 {
        s.assignment
            .note_failure("not-supported", &"échec ".repeat(180));
    }
    assert!(s.assignment.output.len() <= 32768);
    assert!(
        s.assignment
            .output
            .starts_with("[Older terminal results omitted]")
    );
    assert!(s.assignment.output.contains("Not applied"));
    assert_eq!(s.assignment.runtime, assignment);
    assert_eq!(s.practice, practice);
    assert!(!d.current(&s));
    d.bound(&s).unwrap();
}
