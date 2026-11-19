use dlms_learning_lab::{ldap_schema_query_workspace as w, script_teaching::View};
fn definition() -> w::Definition {
    serde_json::from_str(include_str!("fixtures/ldap_schema_query_authored.json")).unwrap()
}
#[test]
fn taught_schema_and_filter_reach_actual_matching_entries_in_every_world() {
    let d = definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        let rows = t.reports(&t.reference_schema, &t.reference_filter).unwrap();
        assert_eq!(rows.len(), 4);
        for r in rows {
            assert!(r.ran && r.passed && r.selection_matches);
            assert_eq!(
                r.actual.len(),
                if r.label == "Changed names and IDs" {
                    6
                } else {
                    2
                }
            );
            assert_eq!(r.rule_checks.len(), 5);
            assert!(r.rule_checks.iter().all(|r| r.0));
        }
    }
}
#[test]
fn independent_rule_and_query_repairs_show_partial_results_and_common_mistakes() {
    let d = definition();
    let t = &d.initial;
    for (schema, filter) in [
        (&t.starter_schema, &t.starter_filter),
        (&t.reference_schema, &t.starter_filter),
        (&t.starter_schema, &t.reference_filter),
    ] {
        assert!(t.reports(schema, filter).unwrap().iter().any(|r| !r.passed));
    }
    let partial = t
        .reference_schema
        .replace("EQUALITY caseExactMatch", "EQUALITY caseIgnoreMatch");
    let rows = t.reports(&partial, &t.reference_filter).unwrap();
    assert!(rows.iter().all(|r| !r.passed));
    assert!(rows.iter().all(|r| r.rule_checks.iter().any(|r| r.0)));
    for filter in [
        "(accessTeam=*)",
        "(accessState=active)",
        "(!(unregistered=active))",
        "(accessState=Active)",
        "(cn=Iris Lane)",
        "(uid=sample1-person1)",
    ] {
        assert!(
            t.reports(&t.reference_schema, filter)
                .unwrap()
                .iter()
                .any(|r| !r.passed),
            "{filter}"
        );
    }
    let hardcoded = "(|(cn=Iris Lane)(cn=Sam Lake))";
    let rows = t.reports(&t.reference_schema, hardcoded).unwrap();
    assert!(rows[0].passed);
    assert!(!rows[1].passed && !rows[2].passed);
    assert!(rows[0].issues.is_empty());
    let rows = t
        .reports(&t.reference_schema, "(!(notRegistered=x))")
        .unwrap();
    assert!(rows.iter().all(|r| {
        r.observations
            .iter()
            .all(|o| o.truth == dlms_learning_lab::ldap_query::Truth::Undefined)
    }));
    assert!(
        rows.iter()
            .all(|r| r.issues.iter().any(|s| s.contains("Undefined")))
    );
}
#[test]
fn supported_equivalent_schema_oids_and_boolean_forms_pass() {
    let d = definition();
    let t = &d.initial;
    let schema = t
        .reference_schema
        .replace("caseIgnoreMatch", "2.5.13.2")
        .replace("caseExactMatch", "2.5.13.5");
    for filter in [
        "(&(accessState=active)(accessTeam=ASTER))",
        "(!(|(!(accessTeam=aster))(!(accessState=active))))",
        "(&(objectClass=accessPerson)(accessTeam=aster)(accessState=active))",
    ] {
        assert!(
            t.reports(&schema, filter).unwrap().iter().all(|r| r.passed),
            "{filter}"
        );
    }
}
#[test]
fn drafts_examples_checked_markers_and_reset_remain_separate_and_current() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    s.view = View::Example;
    s.practice
        .edit_schema(d.example.reference_schema.clone())
        .unwrap();
    s.practice
        .edit_filter(d.example.reference_filter.clone())
        .unwrap();
    s.practice.run(&d.example).unwrap();
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert_eq!(s.assignment, assignment);
    assert!(d.checks(&s).iter().all(|r| !r.0));
    s.assignment
        .edit_schema(d.initial.reference_schema.clone())
        .unwrap();
    s.assignment
        .edit_filter(d.initial.reference_filter.clone())
        .unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.contains("observations"));
    let restored: w::Session = serde_json::from_str(&json).unwrap();
    assert!(d.checks(&restored).iter().all(|r| r.0));
    assert!(restored.practice.output.is_none());
    s.assignment.edit_filter("(accessTeam=*)".into()).unwrap();
    assert!(d.checks(&s).iter().all(|r| !r.0));
    assert!(s.assignment.output.is_none());
    s.assignment.checked = true;
    s.assignment.checked_schema = s.assignment.schema.clone();
    assert!(d.checks(&s).iter().all(|r| !r.0));
    let practice = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.practice, practice);
}
#[test]
fn syntax_unsupported_bounds_foreign_data_and_per_case_preflight_are_honest() {
    let d = definition();
    let t = &d.initial;
    assert!(
        t.run(&t.reference_schema, "accessTeam=Aster", 0)
            .unwrap_err()
            .contains("Syntax")
    );
    assert!(
        t.run(&t.reference_schema, "(accessTeam=A*)", 0)
            .unwrap_err()
            .contains("Unsupported")
    );
    let mut s = w::Session::new(&d).unwrap();
    let before = s.assignment.schema.clone();
    assert!(s.assignment.edit_schema("x".repeat(8193)).is_err());
    assert_eq!(s.assignment.schema, before);
    assert!(s.assignment.edit_filter("x".repeat(4097)).is_err());
    assert!(s.assignment.select(t, 4).is_err());
    s.context = "another-task".into();
    assert!(s.bound(&d).is_err());
    let mut task = t.clone();
    task.cases[1].entries[0]
        .fields
        .push(dlms_learning_lab::ldap_schema::Field {
            name: "unknown".into(),
            values: vec!["value".into()],
        });
    let rows = task
        .reports(&task.reference_schema, &task.reference_filter)
        .unwrap();
    assert!(rows[0].passed && rows[2].passed);
    assert!(!rows[1].ran && !rows[1].passed);
    assert!(rows[1].issues[0].contains("Supplied entry"));
    assert!(task.validate().is_err());
    let mut invalid = d.clone();
    let duplicate = invalid.initial.cases[0].expected[0].clone();
    invalid.initial.cases[0].expected.push(duplicate);
    assert!(invalid.validate().is_err());
}

#[test]
fn every_original_name_or_id_cannot_bypass_changed_identity_verification() {
    let d = definition();
    let t = &d.initial;
    for filter in [
        "(|(cn=Iris Lane)(cn=Sam Lake)(cn=Mira Vale)(cn=Oren Beck)(cn=Wren Moss)(cn=Cory Glen))",
        "(|(uid=sample1-person1)(uid=sample1-person2)(uid=sample2-person1)(uid=sample2-person2)(uid=sample3-person1)(uid=sample3-person2))",
        "(|(name=Iris Lane)(name=Sam Lake)(name=Mira Vale)(name=Oren Beck)(name=Wren Moss)(name=Cory Glen)(cn=VerifierIdentity0)(cn= VERIFIERIDENTITY1 ))",
    ] {
        let rows = t.reports(&t.reference_schema, filter).unwrap();
        assert_eq!(rows.len(), 4);
        assert!(
            rows[..3].iter().all(|r| r.passed),
            "Original correct results must be preserved"
        );
        let renamed = &rows[3];
        assert!(renamed.ran);
        assert!(!renamed.passed && !renamed.selection_matches);
        assert_eq!(renamed.expected.len(), 6);
        assert!(renamed.actual.is_empty());
        assert!(
            renamed
                .observations
                .iter()
                .all(|o| o.truth == dlms_learning_lab::ldap_query::Truth::False)
        );
        assert!(renamed.issues[0].contains("names and user IDs change"));
    }
    for filter in [
        "(&(cn=*)(accessTeam=Aster)(accessState=active))",
        "(&(uid=*)(accessState=active)(accessTeam=aster))",
    ] {
        assert!(
            t.reports(&t.reference_schema, filter)
                .unwrap()
                .iter()
                .all(|r| r.passed),
            "Behaviorally valid identity-presence alternatives remain accepted"
        );
    }
}
#[test]
fn copying_required_fields_cannot_replace_actual_person_inheritance() {
    let d = definition();
    let t = &d.initial;
    let wrong = t
        .reference_schema
        .replace("SUP person STRUCTURAL", "SUP top STRUCTURAL MUST (cn $ sn)");
    assert!(
        t.reports(&wrong, &t.reference_filter)
            .unwrap()
            .iter()
            .all(|r| !r.passed)
    );
    assert!(
        t.run(&wrong, &t.reference_filter, 0)
            .unwrap_err()
            .contains("structural")
    );
}
