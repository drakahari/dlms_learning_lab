use dlms_learning_lab::ldap_access_workspace as w;
#[path = "support/ldap_directory.rs"]
mod fixture;
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
fn probe(label: &str, actor: &str, target: &str, allow: bool, fields: &[(&str, &str)]) -> w::Probe {
    w::Probe {
        label: label.into(),
        actor: actor.into(),
        target: target.into(),
        allow,
        fields: fields
            .iter()
            .map(|(k, v)| w::Field {
                attribute: k.to_string(),
                value: v.to_string(),
            })
            .collect(),
    }
}
fn definition() -> w::Definition {
    let one = w::Case {
        id: "direct".into(),
        label: "Direct contacts".into(),
        initial: fixture::fixture(),
        identities: vec![w::Name(ALEX.into()), w::Name(SAM.into())],
        probes: vec![
            probe(
                "Own mail",
                ALEX,
                ALEX,
                true,
                &[("mail", "new-alex@case.test")],
            ),
            probe(
                "Another person mail",
                ALEX,
                SAM,
                false,
                &[("mail", "unrequested@case.test")],
            ),
            probe(
                "Own combined change",
                SAM,
                SAM,
                true,
                &[("description", "New desk"), ("mail", "new-sam@case.test")],
            ),
            probe(
                "Another person description",
                SAM,
                ALEX,
                false,
                &[("description", "Unrequested desk")],
            ),
        ],
    };
    let mut two = one.clone();
    two.id = "existing-data".into();
    two.label = "Existing optional fields".into();
    let e = two
        .initial
        .entries
        .iter_mut()
        .find(|e| e.dn == SAM)
        .unwrap();
    e.attributes.push(fixture::attr(
        "description",
        &["Preserve old desk unless changed"],
    ));
    two.probes[0].fields[0].value = "different-new@case.test".into();
    let initial=w::Task{version:1,configuration:None,id:"acl-assignment-fixture".into(),instructions:"Permit only each person's own contact updates; keep all other data unchanged.".into(),starter:"access to * by * write".into(),reference:"access to dn.subtree=\"ou=People,dc=example,dc=test\" attrs=mail,description by self write by * none".into(),cases:vec![one,two]};
    let mut example: w::Task = serde_json::from_str(
        &serde_json::to_string(&initial)
            .unwrap()
            .replace("example", "workshop"),
    )
    .unwrap();
    example.id = "acl-practice-fixture".into();
    example.instructions = "Practice self changes in the separate workshop directory.".into();
    w::Definition {
        version: 1,
        initial,
        example,
    }
}
#[test]
fn references_and_supported_equivalent_policies_reach_actual_states_and_starters_do_not() {
    let d = definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        assert!(t.reports(&t.reference).unwrap().iter().all(|r| r.passed));
        assert!(t.reports(&t.starter).unwrap().iter().all(|r| !r.passed));
        let own = t.run(&t.reference, 0, 0).unwrap();
        assert!(own.allowed && own.passed);
        assert_ne!(own.before, own.after);
        let other = t.run(&t.reference, 0, 1).unwrap();
        assert!(!other.allowed && other.passed);
        assert_eq!(other.before, other.after);
    }
    let alternative = "access to attrs=description,mail by self write stop by * none";
    assert!(
        d.initial
            .reports(alternative)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
}
#[test]
fn malformed_policy_and_unexpected_engine_errors_cannot_count_as_expected_denials() {
    let d = definition();
    for policy in [
        "access to *",
        "include /etc/passwd",
        "access to * by * read",
    ] {
        let rows = d.initial.reports(policy).unwrap();
        assert!(rows.iter().all(|r| !r.passed));
    }
    let rows = d
        .initial
        .reports("access to attrs=mail by self write by * none")
        .unwrap();
    assert!(rows.iter().all(|r| !r.passed));
    assert!(rows[0].outcomes[0].passed);
    assert!(!rows[0].outcomes[2].passed);
    assert!(rows[0].feedback.contains("blocked"));
    let rows = d.initial.reports("access to * by * write").unwrap();
    assert!(rows[0].feedback.contains("protect"));
}
#[test]
fn invalid_sources_noop_requests_and_unreachable_references_are_rejected() {
    let d = definition();
    let mut wrong = d.clone();
    wrong.initial.cases[0].probes[0].actor = "uid=absent,dc=example,dc=test".into();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.cases[0].probes[0].target = "uid=absent,dc=example,dc=test".into();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.cases[0].probes[0].fields[0].attribute = "userPassword".into();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.cases[0].probes[0].fields = vec![w::Field {
        attribute: "description".into(),
        value: "Existing".into(),
    }];
    wrong.initial.cases[0]
        .initial
        .entries
        .iter_mut()
        .find(|e| e.dn == ALEX)
        .unwrap()
        .attributes
        .push(fixture::attr("description", &["Existing"]));
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.reference = "access to * by * none".into();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.starter = wrong.initial.reference.clone();
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.example = wrong.initial.clone();
    assert!(wrong.validate().is_err());
}
#[test]
fn practice_drafts_run_results_reset_and_checked_success_are_separate() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let initial = s.assignment.clone();
    s.practice.edit(d.example.reference.clone()).unwrap();
    s.practice.run(&d.example).unwrap();
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert_eq!(s.assignment, initial);
    assert!(s.assignment.checks(&d.initial).iter().all(|r| !r.0));
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.assignment.checks(&d.initial).iter().all(|r| r.0));
    s.assignment.edit("access to * by * read".into()).unwrap();
    assert!(!s.assignment.checked);
    assert!(s.assignment.output.is_none());
    assert!(s.assignment.checks(&d.initial).iter().all(|r| !r.0));
    let practice = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment, initial);
}
#[test]
fn saved_work_has_no_proofs_or_runtime_state_and_forged_success_is_regraded() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.assignment.run(&d.initial).unwrap();
    s.assignment.select(&d.initial, 1, 2).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    assert!(
        !raw.contains("Fictional-") && !raw.contains("decisions") && !raw.contains("authority")
    );
    let restored: w::Session = serde_json::from_str(&raw).unwrap();
    restored.bound(&d).unwrap();
    assert!(restored.assignment.output.is_none());
    assert!(restored.assignment.checks(&d.initial).iter().all(|r| r.0));
    let mut forged = restored.clone();
    forged.assignment.draft = d.initial.starter.clone();
    forged.assignment.checked_draft = Some(forged.assignment.draft.clone());
    assert!(forged.assignment.checks(&d.initial).iter().all(|r| !r.0));
    let mut other = restored;
    other.context = "foreign-task".into();
    assert!(other.bound(&d).is_err());
}
#[test]
fn oversized_edits_clear_stale_success_keep_the_previous_draft_and_deserialization_is_bounded() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    let before = s.assignment.draft.clone();
    assert!(s.assignment.edit("x".repeat(w::MAX_DRAFT + 1)).is_err());
    assert_eq!(s.assignment.draft, before);
    assert!(!s.assignment.checked);
    assert!(s.assignment.checks(&d.initial).iter().all(|r| !r.0));
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["checked_draft"] = serde_json::json!("x".repeat(w::MAX_DRAFT + 1));
    assert!(serde_json::from_value::<w::Session>(value).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<w::Session>(value).is_err());
}

#[test]
fn required_change_denials_explain_the_access_level_and_matching_rule_in_plain_language() {
    let d = definition();
    let run = d.initial.run("access to * by * read", 0, 0).unwrap();
    assert!(!run.passed && !run.allowed);
    assert!(run.feedback.contains("write access"));
    assert!(run.feedback.contains("read access"));
    assert!(run.feedback.contains("Rule 1, caller clause 1"));
    assert!(!run.feedback.contains("Matched"));
    let run = d
        .initial
        .run("access to attrs=description by self write", 0, 0)
        .unwrap();
    assert!(run.feedback.contains("No rule matches"));
    assert_eq!(run.before, run.after);
}
