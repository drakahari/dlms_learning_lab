use dlms_learning_lab::{raid_assembly as c, raid_assembly_workspace as w};
#[path = "support/raid_assembly.rs"]
mod f;
#[test]
fn canonical_teaching_method_experiment_and_retry_are_reachable() {
    let d = f::definition();
    d.validate_reachability().unwrap();
    assert!(d.checks(&f::solved(&d)).iter().all(|r| r.0));
}
#[test]
fn practice_execution_never_earns_assignment_credit_or_overwrites_drafts() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.edit(&d, "# my unfinished assignment\n".into(), false)
        .unwrap();
    let original = s.assignment.clone();
    for step in &d.example_steps {
        s.apply(&d, &step.operation, true).unwrap();
    }
    s.run(&d, w::CHECK, true).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert_eq!(s.assignment, original);
    assert!(d.checks(&s).iter().any(|r| !r.0));
}
#[test]
fn partial_saved_only_and_manual_only_fail_different_requirements() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.apply(&d, &f::save(&d.assignment), false).unwrap();
    s.run(&d, w::CHECK, false).unwrap();
    let rows = d.checks(&s);
    assert!(
        rows.iter()
            .filter(|r| r.1.contains("Next assembly"))
            .all(|r| r.0)
    );
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("stopped")));
    let mut s = w::Session::new(&d).unwrap();
    s.run(
        &d,
        &format!(
            "mdadm -A /dev/md0 --uuid={} /dev/vda1 /dev/vdb1",
            f::uuid(1).display()
        ),
        false,
    )
    .unwrap();
    s.run(&d, w::CHECK, false).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("Next assembly"))
    );
}
#[test]
fn alternate_policy_and_manual_reversed_assembly_pass_behavioral_grading() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    let alternate = format!(
        "# all provided devices are default scan candidates\narr /dev/md1 UUID={}\nARRAY /dev/md0 uuid={} # original\n",
        f::uuid(2).display(),
        f::uuid(1).display()
    );
    s.edit(&d, alternate, false).unwrap();
    s.save(&d, false).unwrap();
    s.run(
        &d,
        &format!(
            "mdadm -A /dev/md0 --uuid={} /dev/vdb1 /dev/vda1",
            f::uuid(1).display()
        ),
        false,
    )
    .unwrap();
    s.run(&d, w::CHECK, false).unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
}
#[test]
fn wrong_active_identity_with_equal_sample_bytes_cannot_pass() {
    let mut d = f::definition();
    let copy = d.assignment.context.replicas.devices["/dev/vda1"]
        .content
        .clone();
    for dev in ["/dev/vdc1", "/dev/vdd1"] {
        d.assignment
            .context
            .replicas
            .devices
            .get_mut(dev)
            .unwrap()
            .content = copy.clone();
    }
    let mut s = w::Session::new(&d).unwrap();
    s.run(&d, "mdadm --stop /dev/md1", false).unwrap();
    s.run(
        &d,
        &format!(
            "mdadm -A /dev/md0 --uuid={} /dev/vdc1 /dev/vdd1",
            f::uuid(2).display()
        ),
        false,
    )
    .unwrap();
    s.run(&d, w::CHECK, false).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("Active array"))
    );
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("Original data"))
    );
}
#[test]
fn stopping_or_editing_after_success_clears_current_success() {
    let d = f::definition();
    let mut s = f::solved(&d);
    s.run(&d, "mdadm --stop /dev/md0", false).unwrap();
    assert!(!s.assignment.checked);
    s.run(&d, w::CHECK, false).unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let mut s = f::solved(&d);
    s.edit(&d, "# broken draft\n".into(), false).unwrap();
    assert!(!s.assignment.checked);
    s.run(&d, w::CHECK, false).unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0 && r.1.contains("draft")));
    s.save(&d, false).unwrap();
    s.run(&d, w::CHECK, false).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("Next assembly"))
    );
}
#[test]
fn source_replay_rejects_forged_runtime_results_and_other_source() {
    let d = f::definition();
    let s = f::solved(&d);
    let mut forged = s.clone();
    forged.assignment.history.clear();
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.transcript[0].result.output = "invented success".into();
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.origin = "other-source".into();
    assert!(d.bound(&forged).is_err());
}
#[test]
fn unfinished_draft_failed_action_and_progress_roundtrip_keep_exact_state() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.edit(&d, "# draft for tomorrow\n".into(), false).unwrap();
    let out = s
        .run(&d, "mdadm --assemble --scan /dev/md0", false)
        .unwrap();
    assert!(out.contains("Failed"));
    let raw = serde_json::to_string(&s).unwrap();
    let restored: w::Session = serde_json::from_str(&raw).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert_eq!(restored.assignment.runtime.live.len(), 1);
}
#[test]
fn unsupported_command_rejects_without_history_or_credit() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    assert!(
        s.run(&d, "mdadm --create /dev/md0", false)
            .unwrap_err()
            .contains("Unsupported")
    );
    assert_eq!(s, before);
}
#[test]
fn reset_replaces_only_selected_work_and_oversized_draft_is_atomic() {
    let d = f::definition();
    let mut s = f::solved(&d);
    s.edit(&d, "# practice draft\n".into(), true).unwrap();
    let example = s.practice.clone();
    s.assignment = w::Work::new(&d.assignment).unwrap();
    assert_eq!(s.practice, example);
    let before = s.clone();
    assert!(s.edit(&d, "x".repeat(4097), false).is_err());
    assert_eq!(s, before);
    assert!(d.checks(&s).iter().any(|r| !r.0));
}
#[test]
fn bounded_history_and_transcript_refuse_growth_without_mutation() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    for _ in 0..64 {
        s.run(&d, "cat /proc/mdstat", false).unwrap();
    }
    assert_eq!(s.assignment.transcript.len(), 32);
    let before = s.clone();
    assert!(
        s.run(&d, "cat /proc/mdstat", false)
            .unwrap_err()
            .contains("64")
    );
    assert_eq!(s, before);
}
#[test]
fn invalid_teaching_definitions_do_not_become_finished_labs() {
    let mut d = f::definition();
    d.example = d.assignment.clone();
    assert!(d.validate().is_err());
    let mut d = f::definition();
    d.reference.clear();
    assert!(d.validate().is_err());
    let mut d = f::definition();
    d.reference = vec![f::step(f::cmd("cat /proc/mdstat"))];
    assert!(d.validate_reachability().is_err());
    let mut d = f::definition();
    d.experiment = f::step(f::cmd("cat /proc/mdstat"));
    assert!(d.validate_reachability().is_err());
    let _ = c::CONFIG;
}
