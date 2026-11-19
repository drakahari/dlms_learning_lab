use dlms_learning_lab::{
    acl_editor::Category, local_access_workspace as w, management_policy as policy,
};
#[path = "support/local_access_tasks.rs"]
mod common;
fn passed(t: &w::Task, raw: &str) -> bool {
    t.reports(raw)
        .is_ok_and(|r| r.iter().all(w::Report::passed))
}
#[test]
fn actual_references_are_reachable_and_starters_do_not_complete_the_task() {
    for handover in [false, true] {
        let d = common::definition(handover);
        d.validate().unwrap();
        assert!(passed(&d.initial, &d.initial.reference));
        assert!(passed(&d.example, &d.example.reference));
        assert!(!passed(&d.initial, &d.initial.starter));
        assert!(!passed(&d.initial, &d.example.reference));
        let reports = d.initial.reports(&d.initial.reference).unwrap();
        assert_eq!(reports.len(), 13);
        assert_eq!(reports[0].scope, "inventory checks");
        assert!(reports[1..].iter().all(|r| r.total == 16));
    }
}
#[test]
fn alternate_record_order_line_grouping_and_equivalent_source_rules_are_accepted() {
    for handover in [false, true] {
        let t = common::task(handover);
        let alternate=t.reference.replace("line vty 0 15\n transport input ssh\n access-class 10 in\nexit","line vty 0 7\n transport input ssh\n access-class 10 in\n login authentication default\nexit\nline vty 8 15\n transport input ssh\n access-class 10 in\nexit").replace("permit host 192.0.2.10\n permit host 192.0.2.11","permit 192.0.2.10 0.0.0.1");
        assert_ne!(alternate, t.reference);
        assert!(passed(&t, &alternate));
        let mut lines = t.reference.lines().map(str::to_string).collect::<Vec<_>>();
        let positions = lines
            .iter()
            .enumerate()
            .filter(|(_, s)| s.starts_with("username"))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        lines.swap(positions[0], positions[1]);
        assert!(passed(&t, &lines.join("\n")));
    }
}
#[test]
fn partial_privilege_and_missing_operators_explain_the_actual_mismatch() {
    let t = common::task(false);
    let raw = t
        .reference
        .replace("username Admin privilege 15", "username Admin privilege 1");
    let reports = t.reports(&raw).unwrap();
    assert_eq!(reports.iter().filter(|r| r.passed()).count(), 12);
    let fail = reports.iter().find(|r| !r.passed()).unwrap();
    assert!(fail.first_failure.as_ref().unwrap().contains("level 1"));
    assert_eq!((fail.matched, fail.total), (0, 16));
    let raw = t.reference.replace(
        "username Operator privilege 1 secret Practice-Operator-93\n",
        "",
    );
    let reports = t.reports(&raw).unwrap();
    assert!(!reports[0].passed());
    assert!(reports.iter().any(|r| {
        r.first_failure
            .as_deref()
            .is_some_and(|s| s.contains("no matching local record"))
    }));
}
#[test]
fn retiring_a_name_is_not_satisfied_by_rotating_or_renaming_a_surplus_account() {
    let t = common::task(true);
    for extra in [
        "username Former privilege 15 secret Rotated-Training\n",
        "username Hidden privilege 15 secret Hidden-Training\n",
    ] {
        let raw = format!("{extra}{}", t.reference);
        let reports = t.reports(&raw).unwrap();
        assert!(!reports[0].passed());
        assert!(
            reports[0]
                .first_failure
                .as_ref()
                .unwrap()
                .contains("unexpected")
        );
        assert!(
            reports[1..].iter().all(w::Report::passed),
            "Inventory must reject a surplus record even when individual credential cases pass"
        );
    }
}
#[test]
fn upstream_policy_blocks_never_substitute_for_local_credential_rejections() {
    let t = common::task(true);
    for raw in [
        t.reference
            .replace("transport input ssh", "transport input none"),
        t.reference
            .replace("permit host 192.0.2.10", "deny host 192.0.2.10"),
        t.reference.replace("ip ssh version 2", "no ip ssh version"),
    ] {
        let reports = t.reports(&raw).unwrap();
        assert!(reports.iter().any(|r| !r.passed()));
        if raw.contains("input none") {
            assert!(reports[7..].iter().all(|r| !r.passed()));
            assert!(
                reports[7]
                    .first_failure
                    .as_ref()
                    .unwrap()
                    .contains("before the local record check")
            );
        }
    }
    let raw = t.reference.replace("permit host 192.0.2.10", "permit any");
    assert!(!passed(&t, &raw));
}
#[test]
fn document_errors_are_not_accepted_as_successful_negative_trials() {
    let t = common::task(false);
    for raw in [
        "aaa authentication login default none",
        "username Admin privilege 15 secret",
        "ssh /personal",
    ] {
        assert!(matches!(t.reports(raw), Err(w::Error::Document(_))));
    }
    let error = t
        .reports("aaa authentication login default none")
        .unwrap_err();
    assert!(matches!(error,w::Error::Document(d)if d.category==Category::Unsupported));
}
#[test]
fn source_rejects_unreachable_references_duplicate_cases_and_weakened_coverage() {
    let good = common::task(false);
    for alter in [0, 1, 2, 3, 4, 5] {
        let mut t = good.clone();
        match alter {
            0 => t.reference = t.starter.clone(),
            1 => t.cases.push(t.cases[0].clone()),
            2 => {
                t.cases[0].expected = w::Expected::Authenticated {
                    user: "SomeoneElse".into(),
                    privilege: dlms_learning_lab::local_access::Privilege::Administrator,
                }
            }
            3 => t
                .entry_cases
                .retain(|c| c.protocol != policy::Protocol::Telnet),
            4 => t.permitted_users.push("Unused".into()),
            5 => t
                .network
                .topology
                .devices
                .get_mut("R2")
                .unwrap()
                .routes
                .clear(),
            _ => unreachable!(),
        };
        assert!(
            t.validate().is_err(),
            "Invalid source variation {alter} must not admit a lab"
        );
    }
}
#[test]
fn source_bounds_and_unknown_serialized_authority_fail_before_grading() {
    let t = common::task(false);
    let mut bad = t.clone();
    bad.cases.extend(t.cases.clone());
    assert!(bad.reports(&bad.reference).is_err());
    let mut raw = serde_json::to_value(&t).unwrap();
    raw["cases"][0]["passed"] = true.into();
    assert!(serde_json::from_value::<w::Task>(raw).is_err());
    let mut raw = serde_json::to_value(&t).unwrap();
    raw["definition"] = serde_json::json!({"complete":true});
    assert!(serde_json::from_value::<w::Task>(raw).is_err());
    assert_eq!(
        serde_json::from_str::<w::Task>(&serde_json::to_string(&t).unwrap()).unwrap(),
        t
    );
}
