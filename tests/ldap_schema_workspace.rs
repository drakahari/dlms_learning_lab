use dlms_learning_lab::{ldap_schema as model, ldap_schema_workspace as w};
#[path = "support/ldap_schema_task.rs"]
mod fixture;
fn solved(s: &w::Session, d: &w::Definition) -> bool {
    d.checks(s).iter().all(|r| r.0)
}
#[test]
fn references_and_legitimate_alternatives_grade_actual_data() {
    let d = fixture::fixture();
    d.validate().unwrap();
    for task in [&d.initial, &d.example] {
        let reports = task.reports(&task.reference).unwrap();
        assert_eq!(reports.len(), 3);
        assert!(reports.iter().all(|r| r.passed));
        assert_eq!(reports.iter().map(|r| r.runs.len()).sum::<usize>(), 33);
        for r in reports {
            for run in r.runs {
                if run.accepted {
                    assert_ne!(run.before, run.after);
                } else {
                    assert_eq!(run.before, run.after);
                    assert_eq!(run.error.unwrap().category, model::Category::Violation);
                }
            }
        }
        let alternate = task
            .reference
            .replace("SUP person", "SUP 2.5.6.6")
            .replace("EQUALITY caseExactMatch", "EQUALITY 2.5.13.5")
            .replace("EQUALITY integerMatch", "EQUALITY 2.5.13.14");
        assert_ne!(alternate, task.reference);
        assert!(task.reports(&alternate).unwrap().iter().all(|r| r.passed));
        let compatible = task
            .reference
            .replace("EQUALITY caseExactMatch", "EQUALITY caseIgnoreMatch");
        assert!(task.reports(&compatible).unwrap().iter().all(|r| r.passed));
    }
}
#[test]
fn partial_and_plausible_wrong_schemas_explain_failing_cases() {
    let d = fixture::fixture();
    let t = &d.initial;
    let rows = t.reports(&t.starter).unwrap();
    assert!(rows.iter().all(|r| !r.passed));
    assert!(rows.iter().all(|r| r.runs.iter().any(|p| p.passed)));
    assert!(rows[0].feedback.contains("Reject missing badge"));
    assert!(rows[0].feedback.contains("Accepted unexpectedly"));
    for raw in [
        t.reference.replace(" SINGLE-VALUE", ""),
        t.reference.replace(
            "MUST trainingBadge\n  MAY (uid $ trainingPoints)",
            "MAY (uid $ trainingBadge $ trainingPoints)",
        ),
        t.reference
            .replace("integerMatch", "caseExactMatch")
            .replace(
                "1.3.6.1.4.1.1466.115.121.1.27",
                "1.3.6.1.4.1.1466.115.121.1.15",
            ),
    ] {
        assert!(t.reports(&raw).unwrap().iter().all(|r| !r.passed));
    }
}
#[test]
fn malformed_unsupported_wrong_branch_and_wrong_module_never_earn_credit() {
    let d = fixture::fixture();
    for draft in [
        "(broken)".into(),
        d.initial
            .reference
            .replace("SINGLE-VALUE", "X-HOST 'run-me'"),
        d.initial.reference.replace("32473.220", "32473.999"),
        d.initial.reference.replace("cn=training,", "cn=elsewhere,"),
    ] {
        let mut s = w::Session::new(&d).unwrap();
        s.assignment.edit(draft).unwrap();
        s.assignment.check(&d.initial).unwrap();
        assert!(!solved(&s, &d));
        assert!(s.assignment.checks(&d.initial).iter().all(|r| !r.0));
    }
}
#[test]
fn scopes_preserve_drafts_and_clear_stale_success_and_runtime_output() {
    let d = fixture::fixture();
    let mut s = w::Session::new(&d).unwrap();
    let original = s.assignment.clone();
    s.practice.edit(d.example.reference.clone()).unwrap();
    s.practice.check(&d.example).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert_eq!(s.assignment, original);
    assert!(!solved(&s, &d));
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(solved(&s, &d));
    s.assignment.run(&d.initial).unwrap();
    assert!(s.assignment.output.is_some());
    s.assignment.edit(d.initial.starter.clone()).unwrap();
    assert!(!s.assignment.checked);
    assert!(s.assignment.output.is_none());
    assert!(!solved(&s, &d));
    let old = s.assignment.draft.clone();
    assert!(s.assignment.edit("x".repeat(8193)).is_err());
    assert_eq!(s.assignment.draft, old);
    let practice = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment, w::Scope::new(&d.initial));
}
#[test]
fn serialized_resume_is_draft_only_and_forged_success_is_regraded() {
    let d = fixture::fixture();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.assignment.run(&d.initial).unwrap();
    s.practice.edit("unfinished example".into()).unwrap();
    s.assignment.select(&d.initial, 2, 4).unwrap();
    s.assignment.run(&d.initial).unwrap();
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: w::Session = serde_json::from_slice(&bytes).unwrap();
    restored.bound(&d).unwrap();
    assert!(restored.assignment.output.is_none());
    assert!(solved(&restored, &d));
    assert_eq!(restored.assignment.selected_case, 2);
    assert_eq!(restored.assignment.selected_probe, 4);
    assert_eq!(restored.practice.draft, "unfinished example");
    restored.assignment.draft = d.initial.starter.clone();
    restored.assignment.checked_draft = Some(d.initial.starter.clone());
    assert!(!solved(&restored, &d));
    assert!(!String::from_utf8(bytes).unwrap().contains("modules"));
}
#[test]
fn invalid_authored_sources_and_saved_work_are_rejected() {
    let good = fixture::fixture();
    let mut d = good.clone();
    d.initial.reference = d.initial.starter.clone();
    assert!(d.validate().is_err());
    let mut d = good.clone();
    d.example = d.initial.clone();
    assert!(d.validate().is_err());
    let mut d = good.clone();
    d.initial.cases[0].probes[2].accept = true;
    assert!(d.validate().is_err());
    let mut s = w::Session::new(&good).unwrap();
    s.context = "wrong".into();
    assert!(s.bound(&good).is_err());
    let mut s = w::Session::new(&good).unwrap();
    s.assignment.selected_probe = 255;
    assert!(s.bound(&good).is_err());
    let mut s = w::Session::new(&good).unwrap();
    s.assignment.checked = true;
    assert!(s.validate().is_err());
    let mut json = serde_json::to_value(w::Session::new(&good).unwrap()).unwrap();
    json["assignment"]["draft"] = serde_json::json!("x".repeat(8193));
    assert!(serde_json::from_value::<w::Session>(json).is_err());
    let mut json = serde_json::to_value(&good).unwrap();
    json["initial"]["cases"][0]["probes"][0]["entry"]["fields"][0]["values"] =
        serde_json::json!(vec!["x"; 17]);
    assert!(serde_json::from_value::<w::Definition>(json).is_err());
}
#[test]
fn hand_constructed_oversized_fixtures_fail_before_run_or_copy() {
    let mut d = fixture::fixture();
    let source = d.initial.cases[0].initial[0].clone();
    d.initial.cases[0].initial = vec![source; 5];
    assert!(d.initial.run(&d.initial.reference, 0, 0).is_err());
    let mut d = fixture::fixture();
    d.initial.cases[0].probes[0].entry.fields[0].values[0] = "x".repeat(257);
    assert!(d.initial.run(&d.initial.reference, 0, 0).is_err());
}
#[test]
fn copying_person_fields_without_inheriting_person_does_not_satisfy_the_task() {
    let d = fixture::fixture();
    let t = &d.initial;
    let wrong = t
        .reference
        .replace("SUP person", "SUP top")
        .replace("MUST trainingBadge", "MUST (cn $ sn $ trainingBadge)")
        .replace(
            "MAY (uid $ trainingPoints)",
            "MAY (uid $ trainingPoints $ description)",
        );
    assert_ne!(wrong, t.reference);
    let reports = t.reports(&wrong).unwrap();
    assert!(reports.iter().all(|r| !r.passed));
    for row in reports {
        assert!(!row.runs[0].passed);
        assert!(!row.runs[0].accepted);
        assert!(row.runs[0].feedback.contains("unrelated"));
    }
}
