use dlms_learning_lab::{management_policy as p, management_workspace as w};
#[path = "support/management_tasks.rs"]
mod common;
fn passes(t: &w::Task, text: &str) -> bool {
    t.reports(text).unwrap().iter().all(w::Report::passed)
}

#[test]
fn source_tasks_have_reachable_references_and_genuinely_unfinished_starters() {
    for source_restriction in [false, true] {
        common::definition(source_restriction).validate().unwrap();
        let t = common::task(source_restriction);
        t.validate().unwrap();
        assert!(passes(&t, &t.reference));
        assert!(!passes(&t, &t.starter));
        let restored: w::Task = serde_json::from_slice(&serde_json::to_vec(&t).unwrap()).unwrap();
        assert_eq!(restored, t);
        restored.validate().unwrap();
    }
}

#[test]
fn valid_alternatives_use_equivalent_behavior_not_reference_string_matching() {
    let t = common::task(true);
    let alternative=t.reference.replace("permit host 192.0.2.10\n permit host 192.0.2.11","permit 192.0.2.10 0.0.0.1")
        .replace("line vty 0 15\n transport input ssh\n access-class 10 in\nexit", "line vty 5 15\n transport input ssh\n access-class 10 in\nexit\nline vty 0 4\n transport input ssh\n access-class 10 in\nexit");
    assert_ne!(alternative, t.reference);
    assert!(passes(&t, &alternative));
}

#[test]
fn permit_everyone_block_everyone_and_one_hardcoded_approved_host_do_not_pass() {
    let t = common::task(true);
    for wrong in [
        t.starter.clone(),
        t.reference.replace(
            "permit host 192.0.2.10\n permit host 192.0.2.11",
            "permit host 192.0.2.10",
        ),
        t.reference
            .replace("transport input ssh", "transport input none"),
        t.reference.replace(
            "permit host 192.0.2.10\n permit host 192.0.2.11",
            "permit 192.0.2.0 0.0.0.127",
        ),
    ] {
        assert_ne!(wrong, t.reference);
        assert!(!passes(&t, &wrong), "{wrong}");
    }
    let reports = t.reports(&t.starter).unwrap();
    assert_eq!(reports.iter().filter(|r| r.passed()).count(), 4);
    assert!(
        reports.iter().filter(|r| !r.passed()).all(|r| r
            .first_failure
            .as_ref()
            .unwrap()
            .contains("source list"))
    );
}

#[test]
fn partial_line_repairs_name_the_remaining_group_without_marking_correct_cases_failed() {
    let t = common::task(false);
    let partial = "ip ssh version 2\nline vty 0 4\ntransport input ssh\nexit\nline vty 5 15\ntransport input ssh telnet\nexit\n";
    let reports = t.reports(partial).unwrap();
    assert_eq!(reports.iter().filter(|r| r.passed()).count(), 3);
    let r = reports.iter().find(|r| !r.passed()).unwrap();
    assert_eq!((r.matched_lines, r.total_lines), (5, 16));
    assert!(r.first_failure.as_ref().unwrap().contains("VTY 5"));
    assert!(r.label.contains("Telnet"));
    assert!(passes(&t, &t.reference));
    assert!(!passes(&t, partial));
    assert!(passes(&t, &t.reference));
}

#[test]
fn source_denials_cannot_be_replaced_by_a_wrong_layer_or_a_broken_ip_path() {
    let mut t = common::task(true);
    let disabled = t
        .reference
        .replace("transport input ssh", "transport input none");
    for r in t
        .reports(&disabled)
        .unwrap()
        .iter()
        .filter(|r| r.label.starts_with("Unapproved"))
    {
        assert!(!r.passed());
        assert_eq!(r.matched_lines, 0);
        assert!(
            r.first_failure
                .as_ref()
                .unwrap()
                .contains("incoming protocol is disabled")
        );
    }
    t.network
        .topology
        .devices
        .get_mut("R2")
        .unwrap()
        .routes
        .clear();
    let reports = t.reports(&t.reference).unwrap();
    assert!(reports.iter().all(|r| !r.passed()));
    assert!(reports.iter().all(|r| {
        r.first_failure
            .as_ref()
            .unwrap()
            .contains("not a policy rejection")
    }));
    assert!(t.validate().is_err());
}

#[test]
fn invalid_candidate_documents_are_diagnostics_not_accepted_negative_results() {
    let t = common::task(true);
    for source in [
        "",
        "ip ssh version 2\nline vty 0 15\ntransport input all\nexit\n",
        "not a configuration",
    ] {
        assert!(matches!(t.reports(source), Err(w::Error::Document(_))));
    }
    let mut invalid = t.clone();
    invalid.cases[0].client = "missing".into();
    assert!(matches!(
        invalid.reports(&invalid.reference),
        Err(w::Error::Source(_))
    ));
}

#[test]
fn source_validation_rejects_duplicate_cases_wrong_protocols_and_fake_reachability() {
    let t = common::task(true);
    let mut bad = t.clone();
    bad.cases[1].label = bad.cases[0].label.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[1].client = bad.cases[0].client.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[4].protocol = p::Protocol::Telnet;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].protocol = p::Protocol::Ssh1;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[3].protocol = p::Protocol::Ssh2;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.reference = bad.starter.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.starter = bad.reference.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.supplied.host_identity = None;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases.remove(1);
    assert!(bad.validate().is_err());
}

#[test]
fn bad_source_bounds_and_extra_serialized_authority_are_rejected() {
    let t = common::task(false);
    let mut bad = t.clone();
    bad.id = "../../task".into();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.reference = "x".repeat(65537);
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].label = "bad\nlabel".into();
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(&t).unwrap();
    value["verified"] = true.into();
    assert!(serde_json::from_value::<w::Task>(value).is_err());
    let mut value = serde_json::to_value(&t).unwrap();
    let case = value["cases"][0].clone();
    value["cases"] = serde_json::Value::Array(vec![case; 9]);
    assert!(serde_json::from_value::<w::Task>(value).is_err());
}
