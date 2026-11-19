use dlms_learning_lab::auth_filter_task::{Expected, Task};
#[path = "support/auth_filter_fixture.rs"]
mod f;
#[test]
fn source_cases_grade_real_results_and_supported_alternatives() {
    for health in [false, true] {
        let t = f::task(health);
        t.validate().unwrap();
        assert!(!t.solved(&t.initial_filter));
        assert!(t.solved(&f::canonical(health)));
        assert!(t.solved(&f::alternative(health)));
        assert!(t.checks(&f::canonical(health)).iter().all(|c| c.passed));
    }
}
#[test]
fn quiet_or_overbroad_filters_and_first_case_hardcodes_fail() {
    let t = f::task(false);
    for bad in [
        "[Definition]\nfailregex = ^never <HOST>$\n",
        "[Definition]\nfailregex = ^.* <HOST> port [0-9]+ source=[a-z]+$\n",
        "[Definition]\nfailregex = ^portal rejected user=alex from <HOST> port [0-9]+ source=[a-z]+$\n",
    ] {
        assert!(!t.solved(bad));
    }
    let first_only = "[Definition]\nfailregex = ^portal rejected user=(alex|probe) from <HOST> port [0-9]+ source=[a-z]+$\n";
    let result = t.checks(first_only);
    assert!(result[0].passed);
    assert!(!result[1].passed);
}
#[test]
fn exceptions_need_to_preserve_real_failures_and_successes() {
    let t = f::task(true);
    let broad = format!("{}ignoreregex = ^.*$\n", f::canonical(false));
    let rows = t.checks(&broad);
    assert!(!rows.iter().all(|c| c.passed));
    assert!(rows[0].detail.contains("also hides a genuine failure"));
    let by_user = format!(
        "{}ignoreregex = ^portal rejected user=probe \n",
        f::canonical(false)
    );
    assert!(!t.solved(&by_user));
    assert!(
        t.checks(&f::canonical(false))[0]
            .detail
            .contains("exception needs its own narrow ignore rule")
    );
    let too_broad_events = "[Definition]\nfailregex = ^portal [a-z]+ user=[A-Za-z0-9_-]+ from <HOST> port [0-9]+ source=[a-z]+$\n";
    assert!(
        t.checks(too_broad_events)[0]
            .detail
            .contains("also selects a non-failure")
    );
}
#[test]
fn syntax_unsupported_and_wrong_output_are_different_feedback() {
    let t = f::task(false);
    assert!(
        t.checks("[Definition]\nfailregex=missing tag\n")[0]
            .detail
            .starts_with("Syntax error:")
    );
    assert!(
        t.checks("[Definition]\nfailregex=^portal \\w+ from <HOST>$\n")[0]
            .detail
            .starts_with("Unsupported pattern or input:")
    );
    let wrong = t.checks(&t.initial_filter);
    assert_eq!(wrong.len(), 3);
    assert!(
        wrong[0]
            .detail
            .contains("Line 1 should be matched as a failure")
    );
    assert!(!wrong[0].detail.contains("failregex = ^portal rejected"));
}
#[test]
fn edits_do_not_leave_cached_success_and_inspection_preserves_source() {
    let t = f::task(true);
    let before = t.clone();
    assert!(t.solved(&f::canonical(true)));
    assert!(!t.solved(&t.initial_filter));
    let rows = t.observe(&f::canonical(true), 0).unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].text, t.cases[0].lines[0]);
    assert_eq!(t, before);
    assert!(t.observe(&f::canonical(true), 99).is_err());
}
#[test]
fn invalid_tasks_and_traversal_are_not_covered_or_successful() {
    let good = f::task(false);
    let mut bad = good.clone();
    bad.filter_path = "/etc/../passwd".into();
    assert!(bad.validate().is_err());
    assert!(!bad.solved(&f::canonical(false)));
    let mut bad = good.clone();
    bad.cases[1].id = bad.cases[0].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    bad.cases[0].expected.pop();
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    bad.cases[0].lines[0] = "two 192.0.2.1 and 192.0.2.2".into();
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    for c in &mut bad.cases {
        c.expected.fill(Expected::Missed);
    }
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    bad.initial_filter = "x".repeat(2049);
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(good).unwrap();
    value["execute_on_host"] = true.into();
    assert!(serde_json::from_value::<Task>(value).is_err());
}
#[test]
fn source_task_roundtrip_is_exact_without_inventing_completion() {
    let task = f::task(true);
    let data = serde_json::to_vec(&task).unwrap();
    let restored: Task = serde_json::from_slice(&data).unwrap();
    assert_eq!(task, restored);
    assert!(!restored.solved(&restored.initial_filter));
}
