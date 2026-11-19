use dlms_learning_lab::{
    pam_control::{Code, Facts},
    pam_workspace as w,
    script_teaching::View,
};
fn task(id: &str, starter: &str) -> w::Task {
    w::Task {
        id: id.into(),
        instructions: format!(
            "Repair {id}: accept valid supplied credentials and reject invalid, unknown or unavailable credentials."
        ),
        starter: starter.into(),
        cases: [
            (Code::Success, true),
            (Code::AuthError, false),
            (Code::UserUnknown, false),
            (Code::AuthInfoUnavailable, false),
            (Code::Ignore, false),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (code, accepted))| w::Case {
            id: format!("{id}-{i}"),
            label: format!("{id} sample {i}"),
            facts: Facts { unix_result: code },
            accepted,
        })
        .collect(),
    }
}
fn definition() -> w::Definition {
    w::Definition {
        initial: task(
            "assignment",
            "auth sufficient pam_permit.so\nauth required pam_unix.so\nauth optional pam_deny.so",
        ),
        example: task(
            "practice",
            "auth optional pam_unix.so\nauth required pam_permit.so",
        ),
        example_worked: "auth required pam_unix.so\nauth optional pam_deny.so".into(),
    }
}
#[test]
fn behavior_accepts_equivalent_stacks_and_rejects_hardcoded_allow_or_deny() {
    let d = definition();
    d.validate().unwrap();
    for solution in [
        "auth required pam_unix.so",
        "auth requisite pam_unix.so",
        "auth sufficient pam_unix.so\nauth required pam_deny.so",
        "# direct check\nauth required pam_unix.so\nauth optional pam_deny.so",
    ] {
        assert!(
            d.initial
                .run(solution)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed),
            "{solution}"
        );
    }
    for wrong in [
        "auth required pam_permit.so",
        "auth required pam_deny.so",
        "auth optional pam_unix.so\nauth required pam_permit.so",
        &d.initial.starter,
    ] {
        assert!(
            d.initial.run(wrong).unwrap().iter().any(|r| !r.passed()),
            "{wrong}"
        );
    }
}
#[test]
fn errors_never_count_as_successful_rejection_and_feedback_explains_a_bypass() {
    let d = definition();
    for wrong in [
        "auth",
        "auth required pam_exec.so /bin/sh",
        "auth required pam_unix.so nullok",
    ] {
        let rows = d.initial.run(wrong).unwrap();
        assert!(
            rows.iter()
                .all(|r| !r.passed() && r.diagnostic.is_some() && r.actual.is_none())
        );
    }
    let rows = d.initial.run(&d.initial.starter).unwrap();
    assert!(rows[0].passed());
    assert!(!rows[1].passed());
    assert!(rows[1].feedback().contains("before checking"));
    let rows = d.example.run(&d.example.starter).unwrap();
    assert!(rows[1].feedback().contains("failure"));
    assert!(
        rows[1]
            .feedback()
            .contains("Expected Rejected; actual Accepted")
    );
    let rows = d.initial.run("auth required pam_deny.so").unwrap();
    assert!(!rows[0].passed());
    assert!(
        rows[0]
            .feedback()
            .contains("Expected Accepted; actual Rejected")
    );
}
#[test]
fn separate_practice_never_checks_assignment_and_edits_invalidate_success() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let initial = s.assignment.clone();
    s.practice.edit(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert_eq!(s.assignment, initial);
    assert!(!d.solved(&s));
    s.assignment.edit("auth required pam_unix.so").unwrap();
    assert!(!d.solved(&s));
    assert!(d.checks(&s)[0].1.contains("Not checked"));
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    s.assignment.edit("auth required pam_permit.so").unwrap();
    assert!(s.assignment.tested.is_none());
    assert!(!d.solved(&s));
    s.assignment.run(&d.initial).unwrap();
    assert!(!d.solved(&s));
    s.assignment.edit("auth requisite pam_unix.so").unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
}
#[test]
fn stop_changed_draft_and_wrong_task_cannot_complete_a_run() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit("auth required pam_unix.so").unwrap();
    let mut run = s.assignment.begin(&d.initial).unwrap();
    assert!(!run.tick());
    assert!(run.results().is_none());
    run.cancel();
    assert!(!run.tick());
    assert!(run.results().is_none());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&s));
    let mut run = s.assignment.begin(&d.initial).unwrap();
    for _ in 0..10 {
        run.tick();
    }
    assert!(run.done());
    assert_eq!(run.results().unwrap().len(), 5);
    assert!(s.assignment.finish(&d.example, &run).is_err());
    s.assignment.edit("auth required pam_permit.so").unwrap();
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&s));
}
#[test]
fn incomplete_and_checked_drafts_roundtrip_and_source_changes_are_regraded() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.view = View::Assignment;
    s.assignment
        .edit("# unfinished\nauth optional pam_unix.so")
        .unwrap();
    s.practice
        .edit("# exploring\nauth requisite pam_deny.so")
        .unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    assert_eq!(w::Session::from_json(&raw, &d).unwrap(), s);
    assert!(!d.solved(&s));
    s.assignment.edit("auth required pam_unix.so").unwrap();
    s.assignment.run(&d.initial).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    let restored = w::Session::from_json(&raw, &d).unwrap();
    assert!(d.solved(&restored));
    assert_eq!(restored.practice, s.practice);
    let mut changed = d.clone();
    changed.initial.cases[0].accepted = false;
    changed.initial.cases[1].accepted = true;
    assert!(!changed.solved(&restored));
    let mut foreign = s.clone();
    foreign.origin = "another-task".into();
    assert!(d.bound(&foreign).is_err());
    let mut future = s.clone();
    future.version = 2;
    assert!(future.validate().is_err());
}
#[test]
fn invalid_sources_and_excess_data_fail_without_discarding_a_draft() {
    let d = definition();
    for mode in 0..6 {
        let mut bad = d.clone();
        match mode {
            0 => bad.example.id = bad.initial.id.clone(),
            1 => bad.initial.cases.clear(),
            2 => bad.initial.cases[1].id = bad.initial.cases[0].id.clone(),
            3 => bad.example_worked = "auth required pam_permit.so".into(),
            4 => bad.initial.starter = "auth required pam_unix.so".into(),
            _ => bad.initial.cases.iter_mut().for_each(|c| c.accepted = true),
        };
        assert!(bad.validate().is_err(), "mode{mode}");
    }
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    assert!(s.assignment.edit(&"x".repeat(8193)).is_err());
    assert_eq!(s, before);
    assert!(s.assignment.edit("auth\0required pam_unix.so").is_err());
    assert_eq!(s, before);
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION + 1), &d).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["draft"] = serde_json::json!("x".repeat(8193));
    assert!(serde_json::from_value::<w::Session>(value).is_err());
    let mut value = serde_json::to_value(&d.initial).unwrap();
    value["cases"] = serde_json::json!(vec![d.initial.cases[0].clone(); 9]);
    assert!(serde_json::from_value::<w::Task>(value).is_err());
}

#[test]
fn bounded_unfinished_and_tested_drafts_survive_worst_case_json_escaping() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    // Invalid code is still legitimate work to retain. Both characters require
    // JSON escaping; testing the invalid draft records diagnostics, never credit.
    let draft = "\"\\".repeat(4096);
    assert_eq!(draft.len(), 8192);
    s.assignment.edit(&draft).unwrap();
    s.assignment.run(&d.initial).unwrap();
    s.practice.edit(&draft).unwrap();
    s.practice.run(&d.example).unwrap();
    s.validate().unwrap();
    assert!(!d.solved(&s));
    let encoded = serde_json::to_string(&s).unwrap();
    assert!(encoded.len() > 64 * 1024);
    assert_eq!(w::Session::from_json(&encoded, &d).unwrap(), s);
    assert!(encoded.len() <= w::MAX_SESSION);
    assert!(s.assignment.edit(&(draft + "x")).is_err());
}
