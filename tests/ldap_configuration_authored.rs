use dlms_learning_lab::ldap_access_workspace as w;
fn definition() -> w::Definition {
    serde_json::from_str(include_str!("fixtures/ldap_configuration_authored.json")).unwrap()
}
#[test]
fn taught_deployment_reaches_all43_actual_config_and_contact_requirements_in_each_scope() {
    let d = definition();
    d.validate().unwrap();
    assert_eq!(d.initial.id, "lpic2-ldap-config-deployment");
    for t in [&d.initial, &d.example] {
        let rows = t.reports(&t.reference).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r.passed));
        assert_eq!(rows.iter().map(|r| r.outcomes.len()).sum::<usize>(), 43);
        for run in rows.iter().flat_map(|r| &r.outcomes) {
            let deployment = run.configuration.as_ref().unwrap();
            assert!(deployment.preserved);
            assert_eq!(deployment.requests.len(), 1);
            assert_eq!(
                deployment.requests[0].name.text(),
                t.configuration.as_ref().unwrap().target
            );
            assert_ne!(deployment.before, deployment.after);
            assert_eq!(run.allowed, run.expected_allow);
            assert_eq!(run.before == run.after, !run.allowed);
        }
    }
    assert_ne!(
        d.initial.configuration.as_ref().unwrap().target,
        d.example.configuration.as_ref().unwrap().target
    );
    assert_ne!(d.initial.cases, d.example.cases);
}
#[test]
fn equivalent_explicit_indices_dn_scope_alias_and_read_fallbacks_pass_write_behavior() {
    let d = definition();
    let t = &d.initial;
    let mut n = 0;
    let indexed = t
        .reference
        .lines()
        .map(|line| {
            if let Some(rule) = line.strip_prefix("olcAccess: ") {
                let line = format!("olcAccess: {{{n}}}{rule}");
                n += 1;
                line
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    for value in [
        indexed,
        t.reference.replace("dn.exact=", "dn.base="),
        t.reference.replace("by * none", "by * read"),
    ] {
        assert!(t.reports(&value).unwrap().iter().all(|r| r.passed));
    }
}
#[test]
fn wrong_backend_workshop_copy_and_extra_neighbor_changes_fail_without_false_credit() {
    let d = definition();
    let t = &d.initial;
    let wrong = t.reference.replace(
        &t.configuration.as_ref().unwrap().target,
        "olcDatabase={1}mdb,cn=config",
    );
    let copied = d
        .example
        .reference
        .replace("deployment-workshop", "deployment");
    let additional = t.reference.clone()
        + "\ndn: olcDatabase={1}mdb,cn=config\nchangetype: modify\nreplace: olcAccess\nolcAccess: to * by * none\n-\n";
    for bad in [&t.starter, &wrong, &copied, &additional] {
        assert!(t.reports(bad).unwrap().iter().all(|r| !r.passed));
    }
    let rows = t.reports(&additional).unwrap();
    assert!(
        rows.iter()
            .flat_map(|r| &r.outcomes)
            .all(|r| r.allowed == r.expected_allow)
    );
    assert!(
        rows.iter()
            .flat_map(|r| &r.outcomes)
            .all(|r| !r.configuration.as_ref().unwrap().preserved)
    );
}
#[test]
fn partial_direct_child_policy_keeps_two_true_passes_and_explains_the_nested_failure() {
    let d = definition();
    let t = &d.initial;
    let rows = t
        .reports(&t.reference.replace("dn.subtree=", "dn.one="))
        .unwrap();
    assert!(rows[0].passed && rows[1].passed && !rows[2].passed);
    assert!(
        rows[2]
            .feedback
            .contains("Nested caller updates own fields")
    );
    assert!(rows[2].feedback.contains("required change was blocked"));
}
#[test]
fn malformed_unsupported_and_later_request_errors_never_grade_a_correct_policy_as_complete() {
    let d = definition();
    let t = &d.initial;
    let later = t.reference.clone()
        + "\ndn: olcDatabase={8}mdb,cn=config\nchangetype: modify\nreplace: olcAccess\nolcAccess: to * by * read\n-\n";
    for bad in [
        t.reference
            .replace("replace: olcAccess", "replace: olcRootPW"),
        t.reference.replace("dn:", "dn"),
        t.reference.replace("to dn.subtree=", "to dn.regex="),
        later,
    ] {
        let rows = t.reports(&bad).unwrap();
        assert!(rows.iter().all(|r| !r.passed && r.outcomes.is_empty()));
    }
}
#[test]
fn practice_drafts_edits_invalidation_checked_resume_and_selected_reset_remain_independent() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.practice.edit(d.example.reference.clone()).unwrap();
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert!(d.checks(&s).iter().all(|r| !r.0));
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    let restored: w::Session = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, restored);
    restored.bound(&d).unwrap();
    assert!(d.checks(&restored).iter().all(|r| r.0));
    s.assignment.edit(d.initial.starter.clone()).unwrap();
    assert!(d.checks(&s).iter().all(|r| !r.0));
    let practice = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment.draft, d.initial.starter);
}

#[test]
fn displayed_folded_worked_example_is_real_ldif_and_reaches_every_practice_requirement() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    let row = rows
        .iter()
        .find(|r| r["id"] == "lpic2-ldap-config-deployment")
        .unwrap();
    let text = row["worked"][1]["value"].as_str().unwrap();
    assert!(text.contains("\n  by "));
    let d = definition();
    assert!(d.example.reports(text).unwrap().iter().all(|r| r.passed));
    assert!(
        row["concepts"][2]["text"]
            .as_str()
            .unwrap()
            .contains("two spaces")
    );
    assert!(d.initial.reports(text).unwrap().iter().all(|r| !r.passed));
}
