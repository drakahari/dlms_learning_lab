use dlms_learning_lab::{ldap_access_workspace as w, ldap_configuration as c};
fn config(root: &str) -> w::Deployment {
    w::Deployment {
        target: "olcDatabase={1}mdb,cn=config".into(),
        initial: c::Source {
            version: 1,
            databases: vec![
                c::DatabaseSource {
                    name: "olcDatabase={-1}frontend,cn=config".into(),
                    suffix: None,
                    root_dn: None,
                    access: vec![],
                },
                c::DatabaseSource {
                    name: "olcDatabase={0}config,cn=config".into(),
                    suffix: None,
                    root_dn: None,
                    access: vec!["to * by * none".into()],
                },
                c::DatabaseSource {
                    name: "olcDatabase={1}mdb,cn=config".into(),
                    suffix: Some(root.into()),
                    root_dn: None,
                    access: vec!["to * by users write by * none".into()],
                },
                c::DatabaseSource {
                    name: "olcDatabase={2}mdb,cn=config".into(),
                    suffix: Some("dc=audit,dc=test".into()),
                    root_dn: None,
                    access: vec!["to * by * read".into()],
                },
            ],
        },
    }
}
fn rules(static_policy: &str) -> Vec<String> {
    let mut rules: Vec<String> = vec![];
    for line in static_policy.lines() {
        if let Some(rule) = line.strip_prefix("access ") {
            rules.push(rule.into());
        } else if !line.trim().is_empty() {
            rules.last_mut().unwrap().push(' ');
            rules.last_mut().unwrap().push_str(line.trim());
        }
    }
    rules
}
fn document(target: &str, rules: &[String], indexed: bool) -> String {
    format!(
        "version: 1\n\ndn: {target}\nchangetype: modify\nreplace: olcAccess\n{}-\n",
        rules
            .iter()
            .enumerate()
            .map(|(i, s)| format!(
                "olcAccess: {}{s}\n",
                if indexed {
                    format!("{{{i}}}")
                } else {
                    String::new()
                }
            ))
            .collect::<String>()
    )
}
fn definition() -> w::Definition {
    let mut d: w::Definition =
        serde_json::from_str(include_str!("fixtures/ldap_access_authored.json")).unwrap();
    for (t, id) in [
        (&mut d.initial, "config-assignment-fixture"),
        (&mut d.example, "config-practice-fixture"),
    ] {
        let configuration = config(&t.cases[0].initial.suffix);
        let reference = document(&configuration.target, &rules(&t.reference), false);
        t.starter = document("olcDatabase={2}mdb,cn=config", &rules(&t.reference), false);
        t.reference = reference;
        t.configuration = Some(configuration);
        t.id = id.into();
    }
    d
}
#[test]
fn original_static_definition_serialization_and_subset_are_exact_when_config_is_absent() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ldap_access_authored.json")).unwrap();
    let d: w::Definition = serde_json::from_value(raw.clone()).unwrap();
    assert!(d.initial.configuration.is_none() && d.example.configuration.is_none());
    assert_eq!(serde_json::to_value(d).unwrap(), raw);
}
#[test]
fn taught_configuration_and_equivalent_ordered_values_grade_actual_changes_without_old_task_credit()
{
    let d = definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        let reports = t.reports(&t.reference).unwrap();
        assert_eq!(reports.len(), 3);
        assert!(reports.iter().all(|r| r.passed));
        assert_eq!(reports.iter().map(|r| r.outcomes.len()).sum::<usize>(), 43);
        let run = t.run(&t.reference, 0, 0).unwrap();
        assert!(run.allowed && run.passed);
        assert_ne!(run.before, run.after);
        let configuration = run.configuration.unwrap();
        assert!(configuration.preserved);
        assert_eq!(configuration.requests.len(), 1);
        assert_ne!(configuration.before, configuration.after);
        let target = &t.configuration.as_ref().unwrap().target;
        // Build the equivalent representation from the actual complete rules,
        // retaining their order and changing only explicit index tags.
        let source_rules: Vec<String> = t
            .reference
            .lines()
            .filter_map(|l| l.strip_prefix("olcAccess: ").map(str::to_owned))
            .collect();
        let indexed = document(target, &source_rules, true);
        assert!(t.reports(&indexed).unwrap().iter().all(|r| r.passed));
    }
    let mut s = w::Session::new(&d).unwrap();
    s.practice.edit(d.example.reference.clone()).unwrap();
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.checks(&d.example).iter().all(|r| r.0));
    assert!(d.checks(&s).iter().all(|r| !r.0));
}
#[test]
fn wrong_backend_and_unrelated_changes_fail_even_when_required_data_outcomes_match() {
    let d = definition();
    let t = &d.initial;
    assert!(t.reports(&t.starter).unwrap().iter().all(|r| !r.passed));
    let audit = document(
        "olcDatabase={2}mdb,cn=config",
        &["to * by * none".into()],
        false,
    );
    let both = t.reference.clone() + "\n" + audit.split_once("\n\n").unwrap().1;
    let reports = t.reports(&both).unwrap();
    assert!(reports.iter().all(|r| !r.passed));
    for run in reports.iter().flat_map(|r| &r.outcomes) {
        assert_eq!(run.allowed, run.expected_allow);
        assert!(!run.passed);
        assert!(!run.configuration.as_ref().unwrap().preserved);
        assert!(run.explanation.contains("unrelated database"));
        assert!(
            run.configuration
                .as_ref()
                .unwrap()
                .feedback
                .contains("olcDatabase={2}mdb")
        );
    }
}
#[test]
fn syntax_unsupported_and_later_semantic_failure_never_count_as_correct_denial_or_earlier_success()
{
    let d = definition();
    let t = &d.initial;
    for bad in [
        t.reference
            .replace("replace: olcAccess", "replace: olcRootPW"),
        t.reference
            .replace("dn: olcDatabase={1}mdb", "dn: olcDatabase={9}mdb"),
        t.reference.replace("changetype: modify", "unknown"),
    ] {
        let reports = t.reports(&bad).unwrap();
        assert!(reports.iter().all(|r| !r.passed && r.outcomes.is_empty()));
        assert!(t.run(&bad, 0, 1).is_err());
    }
    let later = document(
        "olcDatabase={7}mdb,cn=config",
        &["to * by * none".into()],
        false,
    );
    let reports = t
        .reports(&(t.reference.clone() + "\n" + later.split_once("\n\n").unwrap().1))
        .unwrap();
    assert!(
        reports.iter().all(|r| !r.passed
            && r.outcomes.is_empty()
            && r.feedback.contains("after 1 successful"))
    );
}
#[test]
fn serialized_drafts_selections_current_checks_and_selected_reset_keep_scopes_separate() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.practice.edit(d.example.reference.clone()).unwrap();
    s.practice.select(&d.example, 2, 14).unwrap();
    s.practice.run(&d.example).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    let raw = serde_json::to_string(&s).unwrap();
    assert!(!raw.contains("requests") && !raw.contains("sample-root"));
    let restored: w::Session = serde_json::from_str(&raw).unwrap();
    restored.bound(&d).unwrap();
    assert_eq!(s.assignment, restored.assignment);
    assert!(restored.practice.output.is_none());
    assert_eq!(restored.practice.draft, s.practice.draft);
    assert_eq!(restored.practice.selected_case, 2);
    assert_eq!(restored.practice.selected_probe, 14);
    assert!(d.checks(&restored).iter().all(|r| r.0));
    s.assignment.edit(d.initial.starter.clone()).unwrap();
    assert!(d.checks(&s).iter().all(|r| !r.0));
    let practice = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment.draft, d.initial.starter);
    let mut foreign = restored;
    foreign.context = "another-task".into();
    assert!(foreign.bound(&d).is_err());
}
#[test]
fn wrong_target_metadata_unreachable_reference_and_malformed_saved_work_are_rejected() {
    let mut d = definition();
    d.initial.configuration.as_mut().unwrap().target = "olcDatabase={0}config,cn=config".into();
    assert!(d.validate().is_err());
    let mut d = definition();
    d.initial.configuration.as_mut().unwrap().initial.databases[2].suffix =
        Some("dc=another,dc=test".into());
    assert!(d.validate().is_err());
    let mut d = definition();
    d.initial.reference = d.initial.starter.clone();
    assert!(d.validate().is_err());
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.checked = true;
    assert!(s.bound(&d).is_err());
}
