use dlms_learning_lab::{
    ids_rule::{Decision, Event, Protocol},
    ids_rule_task::{Case, Task},
};

fn record(source: &str, destination: &str, port: u16, payload: &str) -> Event {
    Event {
        protocol: Protocol::Tcp,
        source: source.parse().unwrap(),
        source_port: 41000,
        destination: destination.parse().unwrap(),
        destination_port: port,
        payload: payload.into(),
    }
}
fn task() -> Task {
    Task {
        id: "local-api-detection".into(),
        rule_path: "/etc/snort/rules/local.rules".into(),
        initial_rule: "alert tcp any any -> 198.51.100.8 80 (content:\"POST\";sid:1000100;)".into(),
        cases: vec![
            Case {
                id: "primary".into(),
                label: "API destination and payload".into(),
                records: vec![
                    record(
                        "192.0.2.4",
                        "198.51.100.8",
                        8080,
                        "POST /admin HTTP/1.1\r\n",
                    ),
                    record(
                        "192.0.2.5",
                        "198.51.100.8",
                        8080,
                        "GET /health HTTP/1.1\r\n",
                    ),
                    record(
                        "192.0.2.4",
                        "198.51.100.9",
                        8080,
                        "POST /admin HTTP/1.1\r\n",
                    ),
                ],
                expected_alerts: vec![true, false, false],
            },
            Case {
                id: "contrasts".into(),
                label: "Different clients and traffic".into(),
                records: vec![
                    record(
                        "192.0.2.29",
                        "198.51.100.8",
                        8080,
                        "POST /admin HTTP/1.0\r\n",
                    ),
                    record("192.0.2.4", "198.51.100.8", 80, "POST /admin HTTP/1.1\r\n"),
                    record(
                        "192.0.2.4",
                        "198.51.100.8",
                        8080,
                        "POST /public HTTP/1.1\r\n",
                    ),
                ],
                expected_alerts: vec![true, false, false],
            },
        ],
    }
}
const SOLUTION: &str =
    "alert tcp any any -> 198.51.100.8 8080 (content:\"POST /admin\";sid:1000100;)";
const ALTERNATIVE: &str = "alert tcp any any -> 198.51.100.8/32 8080 (sid:1000101;msg:\"API action\";content:\"POST\";content:\"/admin\";)";

#[test]
fn fresh_trials_accept_equivalent_supported_rules_and_clear_broken_success() {
    let t = task();
    t.validate().unwrap();
    assert!(!t.solved(&t.initial_rule));
    assert!(t.solved(SOLUTION));
    assert!(t.solved(ALTERNATIVE));
    assert!(!t.solved(&t.initial_rule));
    assert!(t.checks(SOLUTION).iter().all(|row| row.passed));
}

#[test]
fn silence_alert_everything_wrong_ports_and_client_hardcodes_fail() {
    let t = task();
    for wrong in [
        "alert tcp any any -> any any (sid:1;)",
        "alert tcp any any -> any any (content:\"never-present\";sid:1;)",
        "alert tcp any any -> 198.51.100.8 any (content:\"POST /admin\";sid:1;)",
        "alert tcp 192.0.2.4 any -> 198.51.100.8 8080 (content:\"POST /admin\";sid:1;)",
        "alert tcp any any -> any 8080 (content:\"POST /admin\";sid:1;)",
    ] {
        assert!(!t.solved(wrong), "{wrong}");
    }
    let hardcoded = "alert tcp 192.0.2.4 any -> 198.51.100.8 8080 (content:\"POST /admin\";sid:1;)";
    let rows = t.checks(hardcoded);
    assert!(rows[0].passed);
    assert!(!rows[1].passed);
    assert!(rows[1].detail.contains("Record 1 should alert"));
    assert!(rows[1].detail.contains("addresses and ports"));
}

#[test]
fn syntax_unsupported_limit_and_behavior_mistakes_are_distinct() {
    let t = task();
    assert!(
        t.checks("alert tcp any any -> any any (sid:1)")[0]
            .detail
            .starts_with("Syntax error:")
    );
    assert!(
        t.checks("alert tcp any any -> any any (sid:1;flow:established;)")[0]
            .detail
            .starts_with("Unsupported rule or input:")
    );
    assert!(
        t.checks(&"x".repeat(2049))[0]
            .detail
            .starts_with("Simulation limit:")
    );
    let rows = t.checks(&t.initial_rule);
    assert_eq!(rows.len(), 2);
    assert!(
        rows[0]
            .detail
            .contains("No alert: address, port or direction differs")
    );
    assert!(!rows[0].detail.contains("8080"));
    let broad = t.checks("alert tcp any any -> any any (sid:1;)");
    assert!(broad[0].detail.contains("should not alert"));
    assert!(broad[0].detail.contains("traffic the task excludes"));
    let content = t.checks("alert tcp any any -> 198.51.100.8 8080 (sid:1;content:\"PUT\";)");
    assert!(content[0].detail.contains("content condition 1 is missing"));
    assert!(content[0].detail.contains("letter case"));
}

#[test]
fn observation_is_actual_ungraded_behavior_without_mutating_source() {
    let t = task();
    let before = t.clone();
    assert_eq!(
        t.observe(SOLUTION, 0).unwrap(),
        vec![
            Decision::Alert,
            Decision::MissingContent(0),
            Decision::EndpointMismatch
        ]
    );
    assert!(t.observe(SOLUTION, 2).is_err());
    assert_eq!(t, before);
    assert!(!t.solved(&t.initial_rule));
}

#[test]
fn invalid_cases_paths_shape_and_budgets_never_grade_success() {
    let valid = task();
    let mut variants = Vec::new();
    let mut t = valid.clone();
    t.rule_path = "/etc/../passwd".into();
    variants.push(t);
    let mut t = valid.clone();
    t.cases[1].id = t.cases[0].id.clone();
    variants.push(t);
    let mut t = valid.clone();
    t.cases[0].expected_alerts.pop();
    variants.push(t);
    let mut t = valid.clone();
    t.cases[0].records[0].payload = "\0".into();
    variants.push(t);
    let mut t = valid.clone();
    t.cases[0].label = "\n".into();
    variants.push(t);
    let mut t = valid.clone();
    t.cases.truncate(1);
    variants.push(t);
    let mut t = valid.clone();
    t.initial_rule = "x".repeat(2049);
    variants.push(t);
    let mut t = valid.clone();
    for c in &mut t.cases {
        c.expected_alerts.fill(false);
    }
    variants.push(t);
    let mut t = valid.clone();
    for c in &mut t.cases {
        c.expected_alerts.fill(true);
    }
    variants.push(t);
    let mut t = valid.clone();
    for c in &mut t.cases {
        for e in &mut c.records {
            e.payload = "\\\"".repeat(2048);
        }
        c.records.extend(c.records.clone());
        c.expected_alerts.extend(c.expected_alerts.clone());
    }
    assert!(serde_json::to_vec(&t).unwrap().len() > 64 * 1024);
    variants.push(t);
    for t in variants {
        assert!(t.validate().is_err());
        assert!(!t.solved(SOLUTION));
    }
    let mut value = serde_json::to_value(valid).unwrap();
    value["host_command"] = "snort".into();
    assert!(serde_json::from_value::<Task>(value).is_err());
}

#[test]
fn serialized_task_retains_starter_without_inventing_a_solution() {
    let t = task();
    let restored: Task = serde_json::from_slice(&serde_json::to_vec(&t).unwrap()).unwrap();
    assert_eq!(restored, t);
    assert!(!restored.solved(&restored.initial_rule));
}
