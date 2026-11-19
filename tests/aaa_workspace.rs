use dlms_learning_lab::{aaa_document as doc, aaa_methods as a, aaa_workspace as w};
#[path = "support/aaa_tasks.rs"]
mod fixture;
#[test]
fn references_reach_every_actual_case_and_starters_have_distinct_real_failures() {
    for role in [false, true] {
        let d = fixture::definition(role);
        d.validate().unwrap();
        for t in [&d.initial, &d.example] {
            let rows = t.reports(&t.reference).unwrap();
            assert_eq!(rows.len(), 15);
            assert!(rows.iter().all(w::Report::passed));
            let rows = t.reports(&t.starter).unwrap();
            assert!(rows.iter().any(|r| !r.passed()));
            assert!(rows.iter().any(w::Report::passed));
        }
    }
}
#[test]
fn equivalent_grouping_rules_case_keyword_spacing_and_record_order_pass() {
    let d = fixture::definition(false);
    let t = &d.initial;
    let alternate=t.reference.replace("line vty 0 15\n transport input ssh\n access-class 10 in\nexit","line vty 0 7\n transport input ssh\n access-class 10 in\nexit\nline vty 8 15\n transport input ssh\n access-class 10 in\n login authentication default\nexit").replace("permit host 192.0.2.10\n permit host 192.0.2.11","permit 192.0.2.10 0.0.0.1").replace("aaa authentication login default group Primary local-case","AAA AUTHENTICATION LOGIN DEFAULT group Primary local-case");
    assert_ne!(alternate, t.reference);
    assert!(t.reports(&alternate).unwrap().iter().all(w::Report::passed));
}
#[test]
fn local_bypass_cannot_pass_even_when_local_words_are_changed_to_central_words() {
    let t = fixture::definition(false).initial;
    let mut bad = t
        .reference
        .replace("group Primary local-case", "local-case");
    for u in &t.services.named["Primary"].identities {
        if let Some(c) = t.cases.iter().find(|c| {
            c.username == u.name
                && c.expected
                    == w::Expected::Authenticated {
                        user: u.name.clone(),
                        privilege: if u.name == "Admin" {
                            dlms_learning_lab::local_access::Privilege::Administrator
                        } else {
                            dlms_learning_lab::local_access::Privilege::User
                        },
                        via: a::Method::TrainingService {
                            name: "Primary".into(),
                        },
                    }
        }) {
            let local = t
                .cases
                .iter()
                .find(|v| {
                    v.username == u.name
                        && matches!(
                            &v.expected,
                            w::Expected::Authenticated {
                                via: a::Method::LocalCase,
                                ..
                            }
                        )
                })
                .unwrap();
            bad = bad.replace(
                &format!("secret {}", local.training_token),
                &format!("secret {}", c.training_token),
            );
        }
    }
    let rows = t.reports(&bad).unwrap();
    assert!(!rows[7].passed());
    assert!(!rows[8].passed());
    assert!(
        rows[7]
            .first_failure
            .as_ref()
            .unwrap()
            .contains("training service Primary")
    );
}
#[test]
fn a_wrong_upstream_layer_is_not_credential_rejection_or_missing_authorization() {
    let t = fixture::definition(false).initial;
    let raw = t
        .reference
        .replace("transport input ssh", "transport input none");
    let rows = t.reports(&raw).unwrap();
    assert!(rows[7..].iter().all(|r| !r.passed()));
    assert!(rows[7..].iter().all(|r| {
        r.first_failure
            .as_ref()
            .unwrap()
            .contains("Incoming policy")
    }));
}
#[test]
fn incorrect_rights_words_and_surplus_profiles_are_specific_partial_failures() {
    let t = fixture::definition(true).initial;
    let rows = t.reports(&t.starter).unwrap();
    assert_eq!(rows.iter().filter(|r| !r.passed()).count(), 2);
    assert!(
        rows.iter().filter(|r| !r.passed()).all(|r| r
            .first_failure
            .as_ref()
            .unwrap()
            .contains("level 1"))
    );
    let raw = format!(
        "username Extra privilege 15 secret Wrong-Extra\n{}",
        t.reference
    );
    let rows = t.reports(&raw).unwrap();
    assert!(!rows[0].passed());
    assert!(rows[1..].iter().all(w::Report::passed));
    let raw = t
        .reference
        .replace("secret Practice-Operator-93", "secret Incorrect");
    assert!(t.reports(&raw).unwrap().iter().any(|r| !r.passed()));
}
#[test]
fn actual_inspection_retains_separate_authentication_trace_and_authorization() {
    let t = fixture::definition(false).initial;
    let settings = doc::parse(&t.network, &t.supplied, &t.services, &t.reference).unwrap();
    match t.observe(&settings, 3, 0).unwrap() {
        w::Observation::Identity {
            authentication,
            authorization,
            ..
        } => {
            assert_eq!(authentication.steps.len(), 1);
            assert!(matches!(authentication.outcome, a::Reply::Rejected(_)));
            assert_eq!(authorization, a::ExecAuthorization::NotAuthenticated);
        }
        _ => panic!(),
    }
    match t.observe(&settings, 2, 0).unwrap() {
        w::Observation::Identity {
            authentication,
            authorization,
            ..
        } => {
            assert_eq!(authentication.steps.len(), 2);
            assert_eq!(authentication.steps[0].reply, a::Reply::Unavailable);
            assert!(matches!(
                authorization,
                a::ExecAuthorization::Authorized { .. }
            ));
        }
        _ => panic!(),
    }
    match t.observe(&settings, 5, 0).unwrap() {
        w::Observation::Identity {
            authentication,
            authorization,
            ..
        } => {
            assert!(matches!(
                authentication.outcome,
                a::Reply::Authenticated { .. }
            ));
            assert!(matches!(
                authorization,
                a::ExecAuthorization::MissingLocalRecord { .. }
            ));
        }
        _ => panic!(),
    }
}
#[test]
fn invalid_sources_cannot_supply_duplicate_trials_unknown_services_or_fake_references() {
    let good = fixture::definition(false).initial;
    let mut t = good.clone();
    t.version = 2;
    assert!(t.validate().is_err());
    let mut t = good.clone();
    t.cases[1] = t.cases[0].clone();
    assert!(t.validate().is_err());
    let mut t = good.clone();
    t.cases[0].service_states[0].service = "Missing".into();
    assert!(t.validate().is_err());
    let settings = doc::parse(
        &good.network,
        &good.supplied,
        &good.services,
        &good.reference,
    )
    .unwrap();
    assert!(
        t.observe(&settings, 0, 0).is_err(),
        "public observation must still validate typed source"
    );
    let mut t = good.clone();
    let duplicate = t.cases[0].service_states[0].clone();
    t.cases[0].service_states.push(duplicate);
    assert!(t.validate().is_err());
    let mut t = good.clone();
    t.reference = t.starter.clone();
    assert!(t.validate().is_err());
    let mut t = good.clone();
    t.starter = t.reference.clone();
    assert!(t.validate().is_err());
    let mut t = good;
    t.cases[0].expected = w::Expected::Authenticated {
        user: "Other".into(),
        privilege: dlms_learning_lab::local_access::Privilege::User,
        via: a::Method::LocalCase,
    };
    assert!(t.validate().is_err());
}
#[test]
fn malformed_or_unsupported_documents_are_diagnostics_not_failed_login_success() {
    let t = fixture::definition(false).initial;
    for raw in [
        "aaa authentication login default none",
        "username unfinished",
        "$(touch /not-a-host-operation)",
    ] {
        assert!(t.reports(raw).is_err());
    }
}
