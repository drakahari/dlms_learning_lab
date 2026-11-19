use dlms_learning_lab::{ldap_workspace as w, script_teaching::View};
#[path = "support/ldap_tasks.rs"]
mod fixture;
fn solved() -> (w::Definition, w::Session) {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment
        .edit_config(d.initial.reference_config.clone())
        .unwrap();
    s.assignment
        .edit_command(d.initial.reference_query.clone())
        .unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.checks(&d).iter().all(|r| r.0));
    (d, s)
}
#[test]
fn source_requires_reachable_reference_unfinished_starter_and_independent_cases() {
    let d = fixture::definition();
    d.validate().unwrap();
    let mut b = d.clone();
    b.initial.reference_query = "ldapsearch -x -LLL '(uid=alex)' uid cn".into();
    assert!(b.validate().is_err());
    let mut b = d.clone();
    b.initial.starter_config = b.initial.reference_config.clone();
    b.initial.starter_query = b.initial.reference_query.clone();
    assert!(b.validate().is_err());
    let mut b = d.clone();
    b.example = b.initial.clone();
    assert!(b.validate().is_err());
    let mut b = d.clone();
    b.initial.cases[1].id = b.initial.cases[0].id.clone();
    assert!(b.validate().is_err());
    let mut b = d.clone();
    b.initial.cases.retain(|c| c.probe.is_empty());
    assert!(b.validate().is_err());
    let mut b = d.clone();
    b.initial.cases[0].directories[0].directory.entries[2].attributes[0].values[0] =
        "wrong-rdn".into();
    assert!(b.validate().is_err());
}
#[test]
fn alternate_supported_query_forms_and_config_spacing_pass_actual_behavior() {
    let d = fixture::definition();
    for (config,command)in[(d.initial.reference_config.clone(),"ldapsearch -x -LLL '(&(objectClass=person)(uid=*))' cn UID"),("# explicit defaults\nbase\tou=People,dc=example,dc=test\nuri ldap://DIRECTORY.example.test:389/\n".into(),"ldapsearch -LLL -s sub -x \"(objectClass=person)\" uid cn")]{let r=d.initial.reports(&config,command).unwrap();assert!(r.iter().all(|r|r.passed),"{r:?}");}
}
#[test]
fn partial_fixes_and_common_wrong_work_do_not_pass_every_case() {
    let d = fixture::definition();
    let wrong = [
        (
            d.initial.starter_config.as_str(),
            d.initial.reference_query.as_str(),
        ),
        (
            d.initial.reference_config.as_str(),
            d.initial.starter_query.as_str(),
        ),
        (
            d.initial.reference_config.as_str(),
            "ldapsearch -x -LLL -s one '(objectClass=person)' uid cn",
        ),
        (
            d.initial.reference_config.as_str(),
            "ldapsearch -x -LLL '(objectClass=person)' '*'",
        ),
    ];
    for (config, command) in wrong {
        let r = d.initial.reports(config, command).unwrap();
        assert!(r.iter().any(|r| !r.passed));
    }
    let partial = d
        .initial
        .reports(&d.initial.reference_config, &d.initial.starter_query)
        .unwrap();
    assert_eq!(partial.iter().filter(|r| r.passed).count(), 2);
    assert!(partial[0].feedback.contains("returned entries"));
    assert!(partial[0].expected.contains("uid: lee"));
    assert!(partial[0].actual.contains("uid: alex"));
}
#[test]
fn explicit_command_override_cannot_hide_broken_default_configuration() {
    let d = fixture::definition();
    let command =
        "ldapsearch -x -LLL -b 'ou=People,dc=example,dc=test' '(objectClass=person)' uid cn";
    let r = d
        .initial
        .reports(&d.initial.starter_config, command)
        .unwrap();
    assert!(r[0].passed && r[1].passed);
    assert!(!r[2].passed && !r[3].passed);
}
#[test]
fn hardcoded_single_identity_is_exposed_by_changed_directory_inputs() {
    let d = fixture::definition();
    let r = d
        .initial
        .reports(
            &d.initial.reference_config,
            "ldapsearch -x -LLL '(uid=alex)' uid cn",
        )
        .unwrap();
    assert!(!r[0].passed && !r[1].passed);
    assert_eq!(r[1].actual, "");
    let r = d
        .initial
        .reports(
            &d.initial.reference_config,
            "ldapsearch -x -LLL '(!(unknownAttribute=*))' uid cn",
        )
        .unwrap();
    assert!(!r[0].passed);
    assert_eq!(r[0].actual, "");
}
#[test]
fn feedback_distinguishes_syntax_unsupported_missing_base_and_unavailable() {
    let d = fixture::definition();
    let t = &d.initial;
    for (config, command, phrase) in [
        (
            t.reference_config.as_str(),
            "ldapsearch -x -LLL '(uid=alex'",
            "Syntax:",
        ),
        (
            t.reference_config.as_str(),
            "ldapsearch -x -LLL '(uid=al*)'",
            "Unsupported:",
        ),
        (
            "URI ldap://directory.example.test\nBASE ou=absent,dc=example,dc=test",
            t.reference_query.as_str(),
            "base does not exist",
        ),
        (
            "URI ldap://missing.example.test\nBASE ou=People,dc=example,dc=test",
            t.reference_query.as_str(),
            "URI selected no supplied directory",
        ),
    ] {
        let r = t.reports(config, command).unwrap();
        assert!(!r[0].passed);
        assert!(r[0].feedback.contains(phrase), "{r:?}");
    }
}
#[test]
fn edits_and_forged_flags_cannot_keep_stale_success() {
    let (d, mut s) = solved();
    s.assignment
        .edit_command("ldapsearch -x -LLL '(uid=alex)' uid cn".into())
        .unwrap();
    assert!(!s.assignment.checked);
    assert!(s.checks(&d).iter().any(|r| !r.0));
    s.assignment.checked = true;
    assert!(s.checks(&d).iter().any(|r| !r.0));
    s.assignment.checked_command = s.assignment.command.clone();
    assert!(s.checks(&d).iter().any(|r| !r.0));
    let (d, mut s) = solved();
    s.assignment
        .edit_config(d.initial.starter_config.clone())
        .unwrap();
    assert!(!s.assignment.checked);
    assert!(s.checks(&d).iter().any(|r| !r.0));
}
#[test]
fn separate_example_fill_run_check_reset_never_awards_assignment_credit() {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    s.fill_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
    assert!(!s.practice.checked);
    assert!(s.practice.output.is_none());
    s.practice.apply().unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.output.as_ref().unwrap().contains("dn:"));
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert!(s.checks(&d).iter().any(|r| !r.0));
    s.reset_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
}
#[test]
fn atomic_apply_retains_applied_config_and_a_bad_edit_can_resume() {
    let (d, mut s) = solved();
    let applied = s.assignment.applied.clone();
    s.assignment.edit_config("URI broken\nBASE".into()).unwrap();
    assert!(s.assignment.apply().is_err());
    assert_eq!(s.assignment.applied, applied);
    assert!(s.assignment.run(&d.initial).is_err());
    assert!(s.assignment.output.is_none());
    let raw = serde_json::to_string(&s).unwrap();
    let restored = w::Session::from_json(&d, &raw).unwrap();
    assert_eq!(restored.assignment.config, "URI broken\nBASE");
    assert!(restored.checks(&d).iter().any(|r| !r.0));
}
#[test]
fn save_resume_preserves_text_view_and_results_but_not_transient_output() {
    let (d, mut s) = solved();
    s.view = View::Assignment;
    s.assignment.run(&d.initial).unwrap();
    s.practice
        .edit_command("ldapsearch -x -LLL '(uid=missing)' uid".into())
        .unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    assert!(!raw.contains("output"));
    let r = w::Session::from_json(&d, &raw).unwrap();
    assert!(r.checks(&d).iter().all(|r| r.0));
    assert_eq!(r.view, View::Assignment);
    assert_eq!(r.practice.command, s.practice.command);
    assert!(r.assignment.output.is_none());
}
#[test]
fn selected_reset_keeps_the_other_scope_and_foreign_identity_is_rejected() {
    let (d, mut s) = solved();
    s.fill_example(&d).unwrap();
    let practice = s.practice.clone();
    s.reset_assignment(&d).unwrap();
    assert_eq!(s.practice, practice);
    assert!(!s.assignment.checked);
    s.origin = "another-directory-lesson".into();
    assert!(s.bound(&d).is_err());
    assert!(s.checks(&d).iter().any(|r| !r.0));
}
#[test]
fn bounded_text_import_and_invalid_case_selection_do_not_invent_work() {
    let (d, mut s) = solved();
    let old = s.assignment.command.clone();
    assert!(s.assignment.edit_command("x".repeat(4097)).is_err());
    assert_eq!(s.assignment.command, old);
    assert!(!s.assignment.checked);
    assert!(s.assignment.edit_config("x".repeat(8193)).is_err());
    assert!(s.assignment.select_case(&d.initial, 99).is_err());
    assert!(w::Session::from_json(&d, &" ".repeat(w::MAX_SESSION + 1)).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["assignment"]["selected_case"] = 99.into();
    assert!(w::Session::from_json(&d, &v.to_string()).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["foreign"] = true.into();
    assert!(w::Session::from_json(&d, &v.to_string()).is_err());
}
#[test]
fn changed_trial_clears_observations_without_destroying_assignment_text() {
    let (d, mut s) = solved();
    s.assignment.run(&d.initial).unwrap();
    let command = s.assignment.command.clone();
    s.assignment.select_case(&d.initial, 1).unwrap();
    assert!(s.assignment.output.is_none());
    assert_eq!(s.assignment.command, command);
    assert!(s.assignment.checked);
    s.assignment.run(&d.initial).unwrap();
    assert!(s.assignment.output.as_ref().unwrap().contains("robin"));
}
#[test]
fn source_expected_rows_are_validated_and_internal_helpers_are_not_terminal_commands() {
    let d = fixture::definition();
    let mut b = d.clone();
    if let w::Expected::Entries { entries } = &mut b.initial.cases[0].expected {
        entries.push(entries[0].clone());
    }
    assert!(b.validate().is_err());
    let mut s = w::Session::new(&d).unwrap();
    let old = s.clone();
    assert!(
        s.execute_reference(&d, "ldapsearch -x -LLL '(uid=alex)'")
            .is_err()
    );
    assert_eq!(s, old);
    let data = serde_json::json!({"config":d.initial.reference_config,"command":d.initial.reference_query});
    s.execute_reference(&d, &format!("{}{}", w::PREFIX, data))
        .unwrap();
    assert!(s.checks(&d).iter().all(|r| r.0));
}

#[test]
fn maximum_escaped_unfinished_drafts_fit_the_declared_session_import_budget() {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    for scope in [&mut s.assignment, &mut s.practice] {
        scope.config = "\\".repeat(8192);
        scope.checked_config = "\\".repeat(8192);
        scope.command = "\\".repeat(4096);
        scope.checked_command = "\\".repeat(4096);
    }
    let raw = serde_json::to_string(&s).unwrap();
    assert!(raw.len() <= w::MAX_SESSION);
    let r = w::Session::from_json(&d, &raw).unwrap();
    assert_eq!(s, r);
    assert!(r.checks(&d).iter().any(|r| !r.0));
}

#[test]
fn reports_distinguish_not_run_from_a_successful_empty_search() {
    let d = fixture::definition();
    let t = &d.initial;
    let bad = t
        .reports(&t.reference_config, "ldapsearch -x -LLL '(uid=alex'")
        .unwrap();
    assert!(!bad[0].ran);
    assert!(!bad[0].passed);
    assert!(bad[0].feedback.contains("Syntax:"));
    let empty = t
        .reports(
            &t.reference_config,
            "ldapsearch -x -LLL '(uid=absent)' uid cn",
        )
        .unwrap();
    assert!(empty[0].ran);
    assert_eq!(empty[0].actual, "");
    assert!(!empty[0].passed);
}
