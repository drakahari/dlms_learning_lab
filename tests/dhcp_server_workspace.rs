#[path = "support/dhcp_server.rs"]
mod fixture;
use dlms_learning_lab::{dhcp_server_workspace as w, script_teaching::View};
use fixture::*;
#[test]
fn actual_clients_multiple_cases_and_alternative_scopes_are_state_graded() {
    let d = definition();
    d.validate().unwrap();
    let rows = d.initial.run(&config(30)).unwrap();
    assert!(rows.iter().all(w::CaseResult::passed));
    assert_eq!(rows.len(), 2);
    for r in rows {
        assert_eq!(r.events.len(), 3);
        assert_eq!(r.checks.len(), 11);
        let after = r.after.unwrap();
        assert_eq!(after.leases().len(), 4);
        assert_eq!(
            after.leases()["office-management"],
            d.initial.cases[0].initial.leases()["office-management"]
        );
    }
    let alternative = config(30)
        .replace(
            "range 10.30.0.100 10.30.0.103;",
            "range 10.30.0.100 10.30.0.104;",
        )
        .replace("option domain-name-servers 203.0.113.53;", "")
        .replace(
            "option routers 10.30.0.1;",
            "option routers 10.30.0.1; option domain-name-servers 203.0.113.53;",
        );
    assert!(
        d.initial
            .run(&alternative)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
}
#[test]
fn mixed_case_failure_is_not_hidden_by_correct_individual_rows() {
    let t = task("office", 30);
    let narrow = config(30).replace("10.30.0.103", "10.30.0.102");
    let rows = t.run(&narrow).unwrap();
    assert!(rows[0].passed());
    assert!(!rows[1].passed());
    assert!(rows[1].checks.iter().any(|r| r.0));
    assert!(rows[1].checks.iter().any(|r| !r.0));
    assert!(
        rows[1]
            .events
            .iter()
            .any(|e| !e.ok && e.message.contains("No available address"))
    );
    let wrong = t.run(&t.starter).unwrap();
    assert!(wrong.iter().all(|r| !r.passed()));
    assert!(wrong.iter().all(|r| {
        r.checks
            .iter()
            .any(|(ok, s)| !ok && s.contains("203.0.113.99"))
    }));
    let syntax = t.run("subnet {").unwrap();
    assert!(
        syntax
            .iter()
            .all(|r| !r.passed() && r.diagnostic.is_some() && r.after.is_none())
    );
    let unsupported = t.run("include \"/etc/shadow\";").unwrap();
    assert!(
        unsupported
            .iter()
            .all(|r| r.diagnostic.as_ref().unwrap().contains("Unsupported"))
    );
}
#[test]
fn stepwise_stop_and_changed_drafts_never_publish_partial_credit() {
    let t = task("office", 30);
    let source = t.clone();
    let mut scope = w::Scope {
        draft: config(30),
        tested: None,
    };
    let expected = t.run(&scope.draft).unwrap();
    let mut run = scope.begin(&t).unwrap();
    let mut ticks = 0;
    while !run.tick() {
        ticks += 1;
        assert!(ticks < w::MAX_TICKS);
        assert!(run.results().is_none());
    }
    assert_eq!(run.results().unwrap(), expected);
    scope.finish(&t, &run).unwrap();
    assert!(scope.passed(&t));
    let mut run = scope.begin(&t).unwrap();
    assert!(!run.tick());
    run.cancel();
    assert!(run.results().is_none());
    assert!(!run.done());
    assert!(!run.tick());
    assert!(scope.finish(&t, &run).is_err());
    assert!(!scope.passed(&t));
    let mut run = scope.begin(&t).unwrap();
    while !run.tick() {}
    scope.replace("# edited").unwrap();
    assert!(scope.finish(&t, &run).is_err());
    assert!(!scope.passed(&t));
    assert_eq!(t, source);
}
#[test]
fn practice_assignment_drafts_resume_reset_and_stale_success_remain_separate() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let initial = s.clone();
    s.view = View::Example;
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.passed(&d.example));
    assert!(!d.solved(&s));
    assert_eq!(s.assignment, initial.assignment);
    s.view = View::Assignment;
    s.assignment.replace(&config(30)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    assert!(d.checks(&s).iter().all(|r| r.0));
    let text = serde_json::to_string(&s).unwrap();
    assert_eq!(w::Session::from_json(&text, &d).unwrap(), s);
    s.assignment.replace(&d.initial.starter).unwrap();
    assert!(!d.solved(&s));
    assert_eq!(d.checks(&s).len(), 1);
    assert!(d.checks(&s)[0].1.contains("Not checked"));
    s.assignment = initial.assignment;
    assert!(s.practice.passed(&d.example));
}
#[test]
fn imported_marks_cannot_fabricate_mastery_and_invalid_imports_leave_original() {
    let d = definition();
    let s = w::Session::new(&d).unwrap();
    let original = s.clone();
    let mut forged = s.clone();
    forged.assignment.tested = Some(forged.assignment.draft.clone());
    assert!(!d.solved(&forged));
    let value = serde_json::to_value(&s).unwrap();
    for bad in [
        {
            let mut v = value.clone();
            v["passed"] = true.into();
            v
        },
        {
            let mut v = value.clone();
            v["origin"] = "foreign".into();
            v
        },
        {
            let mut v = value.clone();
            v["version"] = 9.into();
            v
        },
        {
            let mut v = value.clone();
            v["assignment"]["draft"] = "x".repeat(16385).into();
            v
        },
        {
            let mut v = value.clone();
            v["assignment"]["draft"] = "bad\u{0}draft".into();
            v
        },
    ] {
        assert!(w::Session::from_json(&bad.to_string(), &d).is_err());
        assert_eq!(s, original);
    }
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION + 1), &d).is_err());
    let mut scope = s.assignment.clone();
    for bad in ["x".repeat(16385), "bad\u{1b}".into()] {
        assert!(scope.replace(&bad).is_err());
        assert_eq!(scope, s.assignment);
    }
}
#[test]
fn validation_rejects_unknowns_duplicates_oversize_and_crossed_practice_sources() {
    let d = definition();
    assert_eq!(
        w::Definition::from_json(&serde_json::to_string(&d).unwrap()).unwrap(),
        d
    );
    let mut t = d.initial.clone();
    t.cases[0].actions.push(w::Operation::Acquire {
        client: "unknown".into(),
    });
    assert!(t.validate().is_err());
    let mut t = d.initial.clone();
    let duplicate = t.cases[0].requirements[0].clone();
    t.cases[0].requirements.push(duplicate);
    assert!(t.validate().is_err());
    let mut t = d.initial.clone();
    t.cases[0]
        .requirements
        .retain(|r| !matches!(r, w::Requirement::PoolsOnly { .. }));
    assert!(t.validate().is_err());
    let mut t = d.initial.clone();
    t.cases[0]
        .actions
        .resize(17, w::Operation::Advance { seconds: 1 });
    assert!(t.validate().is_err());
    let mut t = d.initial.clone();
    t.cases[0]
        .actions
        .push(w::Operation::Advance { seconds: 86401 });
    assert!(t.validate().is_err());
    let mut t = d.initial.clone();
    t.cases[0].label = "bad\nlabel".into();
    assert!(t.validate().is_err());
    let mut crossed = d.clone();
    crossed.example = d.initial.clone();
    assert!(crossed.validate().is_err());
    let mut solved = d.clone();
    solved.initial.starter = config(30);
    assert!(solved.validate().is_err());
    let mut wrong = d.clone();
    wrong.example_worked = d.example.starter.clone();
    assert!(wrong.validate().is_err());
    let mut value = serde_json::to_value(&d).unwrap();
    let arr = value["initial"]["cases"].as_array_mut().unwrap();
    arr.resize(3, arr[0].clone());
    arr.push(serde_json::json!({"this body must not be parsed":[1,2,3]}));
    let error = serde_json::from_value::<w::Definition>(value)
        .unwrap_err()
        .to_string();
    assert!(error.contains("item limit"), "{error}");
}
#[test]
fn unsupported_renewal_context_has_failed_execution_row_even_when_preservation_passes() {
    let mut t = task("office", 30);
    for c in &mut t.cases {
        c.actions = vec![
            w::Operation::Advance { seconds: 300 },
            w::Operation::Renew {
                client: "office-management".into(),
            },
        ];
        c.requirements.retain(|r| {
            matches!(
                r,
                w::Requirement::PreserveBinding { .. } | w::Requirement::PoolsOnly { .. }
            )
        });
    }
    let rows = t.run(&config(30)).unwrap();
    for r in rows {
        assert!(!r.passed());
        assert!(!r.checks[0].0);
        assert!(r.checks[1..].iter().all(|r| r.0));
        assert!(r.checks[0].1.contains("Unsupported renewal context"));
    }
}
#[test]
fn distinct_range_and_option_teaching_fixtures_are_reachable_but_not_solved_starters() {
    for (name, definition, answer) in [
        ("ranges", range_definition(), range_config(50)),
        ("options", options_definition(), options_config(51)),
    ] {
        definition.validate().unwrap();
        assert!(
            definition
                .initial
                .run(&answer)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert!(
            !definition
                .initial
                .run(&definition.initial.starter)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let encoded = serde_json::to_string(&definition).unwrap();
        assert_eq!(w::Definition::from_json(&encoded).unwrap(), definition);
        println!("DHCP_AUTHORING_{name}={encoded}");
    }
}

#[test]
fn unsupported_held_lease_rediscovery_cannot_fake_success_through_a_preservation_goal() {
    let mut t = task("office", 30);
    for c in &mut t.cases {
        c.actions = vec![w::Operation::Acquire {
            client: "office-management".into(),
        }];
        c.requirements.retain(|r| {
            matches!(
                r,
                w::Requirement::PreserveBinding { .. } | w::Requirement::PoolsOnly { .. }
            )
        });
    }
    for r in t.run(&config(30)).unwrap() {
        assert!(!r.passed());
        assert!(!r.checks[0].0);
        assert!(r.checks[1..].iter().all(|r| r.0));
        assert!(r.events[0].message.contains("Unsupported exercise context"));
        assert!(r.events[0].message.contains("rediscovery/cache"));
    }
}
