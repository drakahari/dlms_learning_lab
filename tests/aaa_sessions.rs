use dlms_learning_lab::{aaa_workspace as w, script_teaching::View};
#[path = "support/aaa_tasks.rs"]
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
fn practice_edit_apply_check_and_fill_never_award_assignment_credit() {
    for handover in [false, true] {
        let d = common::definition(handover);
        let mut s = w::Session::new(&d).unwrap();
        let assignment = s.assignment.clone();
        s.fill_example(&d).unwrap();
        assert_eq!(s.assignment, assignment);
        assert!(!s.practice.checked);
        assert!(
            s.practice
                .check(&d.example)
                .unwrap()
                .iter()
                .all(w::Report::passed)
        );
        assert_eq!(s.assignment, assignment);
        assert!(!passed(&s, &d));
        s.assignment.edit(d.example.reference.clone()).unwrap();
        assert!(s.assignment.check(&d.initial).is_err());
        assert!(!passed(&s, &d));
    }
}
#[test]
fn edits_break_current_success_and_specific_partial_feedback_then_allow_retry() {
    let d = common::definition(true);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    s.assignment.inspect(&d.initial).unwrap();
    s.assignment.edit(d.initial.starter.clone()).unwrap();
    assert!(!s.assignment.checked);
    assert!(s.assignment.observation.is_none());
    assert!(!passed(&s, &d));
    let reports = s.assignment.check(&d.initial).unwrap();
    assert!(reports.iter().any(|r| r.passed()));
    assert!(reports.iter().any(|r| !r.passed()));
    assert!(!passed(&s, &d));
    complete(&mut s, &d);
}
#[test]
fn malformed_apply_is_atomic_and_not_a_passing_rejected_login() {
    let d = common::definition(false);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    let applied = s.assignment.applied.clone();
    s.assignment
        .edit("aaa authentication login default none".into())
        .unwrap();
    assert!(matches!(
        s.assignment.apply(&d.initial),
        Err(w::Error::Document(_))
    ));
    assert_eq!(s.assignment.applied, applied);
    assert!(!passed(&s, &d));
    assert!(
        s.assignment
            .inspect(&d.initial)
            .unwrap_err()
            .contains("Apply")
    );
}
#[test]
fn raw_drafts_restore_separately_but_transient_observations_are_never_imported_as_execution() {
    let d = common::definition(true);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    let actual = s.assignment.inspect(&d.initial).unwrap();
    assert!(
        actual.contains("training service Primary: authenticated Admin")
            && actual.contains("Local EXEC authorization: Admin receives level 15")
    );
    assert!(!actual.contains("Central-Practice-Admin-42"));
    s.fill_example(&d).unwrap();
    s.practice.draft = "username unfinished".into();
    s.view = View::Example;
    let raw = serde_json::to_string(&s).unwrap();
    let restored = w::Session::from_json(&d, &raw).unwrap();
    assert_eq!(restored.practice.draft, "username unfinished");
    assert!(restored.assignment.observation.is_none());
    assert!(passed(&restored, &d));
    let mut bad = serde_json::to_value(&s).unwrap();
    bad["assignment"]["observation"] = "fabricated execution".into();
    assert!(w::Session::from_json(&d, &bad.to_string()).is_err());
}
#[test]
fn selectors_clear_output_and_selected_resets_preserve_other_scope() {
    let d = common::definition(true);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    s.assignment.inspect(&d.initial).unwrap();
    s.assignment.select_sample(&d.initial, 3, 15).unwrap();
    assert!(s.assignment.observation.is_none());
    assert!(s.assignment.checked);
    let actual = s.assignment.inspect(&d.initial).unwrap();
    assert!(actual.contains("does not match"));
    let assignment = s.assignment.clone();
    s.fill_example(&d).unwrap();
    s.reset_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
    let practice = s.practice.clone();
    s.reset_assignment(&d).unwrap();
    assert_eq!(s.practice, practice);
    assert!(!passed(&s, &d));
}
#[test]
fn forged_flags_foreign_origins_bad_applied_documents_and_oversize_restore_fail() {
    let d = common::definition(false);
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.checked = true;
    assert!(!passed(&s, &d));
    let mut bad = s.clone();
    bad.origin = "different-lesson".into();
    assert!(w::Session::from_json(&d, &serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad = s.clone();
    bad.assignment.selected_line = 98;
    assert!(bad.bound(&d).is_err());
    let mut bad = s.clone();
    bad.assignment.applied = "not an authored starter or parsed policy".into();
    assert!(bad.bound(&d).is_err());
    assert!(w::Session::from_json(&d, &" ".repeat(w::MAX_SESSION + 1)).is_err());
    assert!(w::Session::from_json(&d, "{broken").is_err());
}
#[test]
fn overlarge_edits_keep_bounded_text_but_clear_current_check_and_observation() {
    let d = common::definition(false);
    let mut s = w::Session::new(&d).unwrap();
    complete(&mut s, &d);
    let draft = s.assignment.draft.clone();
    s.assignment.inspect(&d.initial).unwrap();
    assert!(s.assignment.edit("X".repeat(65537)).is_err());
    assert_eq!(s.assignment.draft, draft);
    assert!(!s.assignment.checked);
    assert!(s.assignment.observation.is_none());
    assert!(!passed(&s, &d));
}
