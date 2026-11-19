use dlms_learning_lab::{management_workspace as w, script_teaching::View};
#[path = "support/management_tasks.rs"]
mod common;
fn passed(s: &w::Session, d: &w::Definition) -> bool {
    s.checks(d).iter().all(|r| r.0)
}
fn complete(s: &mut w::Session, d: &w::Definition) {
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    assert!(
        s.assignment
            .check(&d.initial)
            .unwrap()
            .iter()
            .all(w::Report::passed)
    );
    assert!(passed(s, d));
}

#[test]
fn filling_and_running_practice_never_overwrites_or_completes_assignment() {
    for source_restriction in [false, true] {
        let d = common::definition(source_restriction);
        let mut s = w::Session::new(&d).unwrap();
        assert_eq!(s.view, View::Introduction);
        assert!(!passed(&s, &d));
        let original = s.assignment.clone();
        let before = s.practice.applied.clone();
        s.fill_example(&d).unwrap();
        assert_eq!(s.assignment, original);
        assert_eq!(s.practice.applied, before);
        assert!(!s.practice.checked);
        assert!(s.practice.observation.is_none());
        assert!(
            s.practice
                .check(&d.example)
                .unwrap()
                .iter()
                .all(w::Report::passed)
        );
        assert_eq!(s.assignment, original);
        assert!(!passed(&s, &d));
        // The smaller/different practice policy cannot satisfy the full task.
        s.assignment.edit(s.practice.draft.clone()).unwrap();
        assert!(matches!(
            s.assignment.check(&d.initial),
            Err(w::Error::Document(_))
        ));
        assert_eq!(s.assignment.applied, original.applied);
        assert!(!passed(&s, &d));
    }
}

#[test]
fn edits_clear_success_and_observations_before_apply_then_allow_partial_retry() {
    let d = common::definition(true);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    let observed = s.assignment.inspect(&d.initial).unwrap().to_string();
    assert!(observed.contains("IPv4 forward path"));
    assert!(passed(&s, &d));
    let wrong = d
        .initial
        .reference
        .replace("transport input ssh", "transport input none");
    s.assignment.edit(wrong).unwrap();
    assert!(!s.assignment.checked);
    assert!(s.assignment.observation.is_none());
    assert!(!passed(&s, &d));
    assert!(s.assignment.inspect(&d.initial).is_err());
    let reports = s.assignment.check(&d.initial).unwrap();
    assert!(reports.iter().any(|r| !r.passed()));
    assert!(!passed(&s, &d));
    complete(&mut s, &d);
}

#[test]
fn malformed_drafts_and_selectors_resume_but_transient_output_does_not_claim_execution() {
    let d = common::definition(false);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    s.assignment.inspect(&d.initial).unwrap();
    s.practice
        .edit("ip ssh version 2\nline vty malformed\n".into())
        .unwrap();
    s.practice.select_sample(&d.example, 3, 2).unwrap();
    s.view = View::Example;
    let bytes = serde_json::to_string(&s).unwrap();
    assert!(!bytes.contains("\"observation\""));
    assert!(!bytes.contains("\"reference\""));
    assert!(!bytes.contains("\"network\""));
    let restored = w::Session::from_json(&d, &bytes).unwrap();
    assert_eq!(restored.view, View::Example);
    assert_eq!(restored.assignment.draft, s.assignment.draft);
    assert_eq!(restored.practice.draft, s.practice.draft);
    assert_eq!(
        (
            restored.practice.selected_case,
            restored.practice.selected_line
        ),
        (3, 2)
    );
    assert!(restored.assignment.observation.is_none());
    assert!(passed(&restored, &d));
}

#[test]
fn selected_resets_preserve_the_other_scope_and_unrelated_sessions() {
    let d = common::definition(true);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    s.fill_example(&d).unwrap();
    s.practice.check(&d.example).unwrap();
    let practice = s.practice.clone();
    let other = w::Session::new(&common::definition(false)).unwrap();
    let before = other.clone();
    s.reset_assignment(&d).unwrap();
    assert_eq!(s.practice, practice);
    assert!(!passed(&s, &d));
    complete(&mut s, &d);
    let assignment = s.assignment.clone();
    s.reset_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.practice.draft, d.example.starter);
    assert!(!s.practice.checked);
    assert_eq!(other, before);
}

#[test]
fn forged_flags_versions_source_bindings_and_unapplied_answers_do_not_fabricate_completion() {
    let d = common::definition(true);
    let s = w::Session::new(&d).unwrap();
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["checked"] = true.into();
    let restored = w::Session::from_json(&d, &value.to_string()).unwrap();
    assert!(!passed(&restored, &d));
    value["assignment"]["draft"] = d.initial.reference.clone().into();
    let restored = w::Session::from_json(&d, &value.to_string()).unwrap();
    assert!(!passed(&restored, &d));
    for (key, val) in [
        ("origin", serde_json::Value::String("another-task".into())),
        ("version", 2.into()),
        ("current_success", true.into()),
    ] {
        let mut bad = value.clone();
        bad[key] = val;
        assert!(w::Session::from_json(&d, &bad.to_string()).is_err());
    }
    let mut bad = value.clone();
    bad["practice"]["selected_case"] = 99.into();
    assert!(w::Session::from_json(&d, &bad.to_string()).is_err());
    let mut bad = value;
    bad["assignment"]["applied"] = "not an applied policy".into();
    assert!(w::Session::from_json(&d, &bad.to_string()).is_err());
}

#[test]
fn intentional_starter_errors_remain_repairable_without_accepting_imported_applied_errors() {
    let mut d = common::definition(false);
    d.initial.starter = "ip ssh version broken\n".into();
    d.validate().unwrap();
    let mut s = w::Session::new(&d).unwrap();
    s.bound(&d).unwrap();
    assert!(s.assignment.inspect(&d.initial).is_err());
    assert!(!passed(&s, &d));
    let raw = serde_json::to_string(&s).unwrap();
    let mut restored = w::Session::from_json(&d, &raw).unwrap();
    complete(&mut restored, &d);
    restored
        .assignment
        .edit("malformed retry\n".into())
        .unwrap();
    assert!(matches!(
        restored.assignment.check(&d.initial),
        Err(w::Error::Document(_))
    ));
    assert_eq!(restored.assignment.applied, d.initial.reference);
    assert!(!passed(&restored, &d));
    w::Session::from_json(&d, &serde_json::to_string(&restored).unwrap()).unwrap();
}

#[test]
fn escaped_draft_wire_bounds_are_safe_and_larger_drafts_fail_without_mutation() {
    let d = common::definition(false);
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit("\0".repeat(65536)).unwrap();
    s.practice.edit("\0".repeat(65536)).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    assert!(raw.len() <= w::MAX_SESSION);
    let restored = w::Session::from_json(&d, &raw).unwrap();
    assert_eq!(restored, s);
    assert!(!passed(&restored, &d));
    let before = s.clone();
    assert!(s.assignment.edit("x".repeat(65537)).is_err());
    assert_eq!(s, before);
    assert!(w::Session::from_json(&d, &" ".repeat(w::MAX_SESSION + 1)).is_err());
    let mut bad = d.clone();
    bad.example = bad.initial.clone();
    assert!(w::Session::new(&bad).is_err());
}
