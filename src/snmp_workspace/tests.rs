use super::*;
use crate::snmp::{
    Agent, Interface, LinkState,
    monitor::{Snapshot, UnknownCause},
};
fn agent(label: &str, index: u32, admin: LinkState, oper: LinkState) -> Agent {
    Agent {
        available: true,
        uptime_ticks: 10_000,
        interfaces: vec![
            Interface {
                index,
                description: label.into(),
                admin,
                oper,
                speed_bits_per_second: 1_000_000_000,
                in_octets: 1000,
                out_octets: 2000,
                discontinuity_ticks: 0,
            },
            Interface {
                index: 99,
                description: "visitor-uplink".into(),
                admin: LinkState::Up,
                oper: LinkState::Up,
                speed_bits_per_second: 100_000_000,
                in_octets: 0,
                out_octets: 0,
                discontinuity_ticks: 0,
            },
        ],
    }
}
fn fields(label: &str) -> Fields {
    Fields {
        description: label.into(),
        link_rule: LinkRule::UnplannedDown,
        direction: Direction::Incoming,
        width: Width::Counter64,
        threshold: "80".into(),
    }
}
fn link_task(id: &str, label: &str) -> Task {
    let states = [
        ("active", LinkState::Up, LinkState::Up, Outcome::Quiet),
        ("fault", LinkState::Up, LinkState::Down, Outcome::Alert),
        (
            "maintenance",
            LinkState::Down,
            LinkState::Down,
            Outcome::Quiet,
        ),
    ];
    let mut cases = states
        .iter()
        .enumerate()
        .map(|(n, (name, admin, oper, expected))| Case {
            id: (*name).into(),
            label: format!("Supplied {name}"),
            stream: Stream {
                before: None,
                after: Snapshot {
                    at_seconds: 0,
                    agent: agent(label, 2 + n as u32 * 8, *admin, *oper),
                },
            },
            expected: *expected,
        })
        .collect::<Vec<_>>();
    let mut unavailable = cases[0].stream.clone();
    unavailable.after.agent.available = false;
    cases.push(Case {
        id: "unavailable".into(),
        label: "Unavailable agent".into(),
        stream: unavailable,
        expected: Outcome::Unknown(UnknownCause::UnavailableAgent),
    });
    let mut initial = fields("visitor-uplink");
    initial.link_rule = LinkRule::OperationalDown;
    Task {
        version: 1,
        id: id.into(),
        activity: Activity::Link,
        initial,
        reference: fields(label),
        cases,
    }
}
fn traffic_task(id: &str, label: &str) -> Task {
    let mut cases = [
        ("busy", 90, Outcome::Alert),
        ("ordinary", 40, Outcome::Quiet),
    ]
    .iter()
    .map(|(name, percent, expected)| {
        let b = agent(label, 2, LinkState::Up, LinkState::Up);
        let mut a = b.clone();
        a.uptime_ticks += 6000;
        a.interfaces[0].in_octets += 7_500_000_000 * percent / 100;
        a.interfaces[0].out_octets += 750_000_000;
        Case {
            id: (*name).into(),
            label: format!("Supplied {name}"),
            stream: Stream {
                before: Some(Snapshot {
                    at_seconds: 0,
                    agent: b,
                }),
                after: Snapshot {
                    at_seconds: 60,
                    agent: a,
                },
            },
            expected: *expected,
        }
    })
    .collect::<Vec<_>>();
    let mut reset = cases[0].stream.clone();
    reset.after.agent.interfaces[0].discontinuity_ticks = 12000;
    cases.push(Case {
        id: "counter-reset".into(),
        label: "Counter restarted".into(),
        stream: reset,
        expected: Outcome::Unknown(UnknownCause::UnusableCounter),
    });
    let mut initial = fields(label);
    initial.direction = Direction::Outgoing;
    initial.width = Width::Counter32;
    initial.threshold = "25".into();
    Task {
        version: 1,
        id: id.into(),
        activity: Activity::Traffic,
        initial,
        reference: fields(label),
        cases,
    }
}
fn definition(traffic: bool) -> Definition {
    let task = if traffic { traffic_task } else { link_task };
    Definition {
        version: 1,
        initial: task("assigned-monitor", "staff-uplink"),
        example: task("worked-monitor", "workshop-uplink"),
    }
}
#[test]
fn references_reach_distinct_alert_quiet_unknown_results_and_initial_work_is_unfinished() {
    for traffic in [false, true] {
        let d = definition(traffic);
        d.validate().unwrap();
        let mut s = Session::new(&d);
        assert!(!s.solved(&d));
        assert!(!s.assignment.check(&d.initial).unwrap().passed());
        s.assignment.fields = d.initial.reference.clone();
        assert!(!s.solved(&d));
        assert!(s.assignment.check(&d.initial).unwrap().passed());
        assert!(s.solved(&d));
    }
}
#[test]
fn multiple_legitimate_traffic_thresholds_pass_the_same_behavior_cases() {
    let d = definition(true);
    for threshold in ["70", "80", "90", "080"] {
        let mut s = Session::new(&d);
        s.assignment.fields = d.initial.reference.clone();
        s.assignment.fields.threshold = threshold.into();
        assert!(
            s.assignment.check(&d.initial).unwrap().passed(),
            "{threshold}"
        );
        assert!(s.solved(&d));
    }
    for threshold in ["0", "101", "abc", ""] {
        let mut s = Session::new(&d);
        s.assignment.fields = d.initial.reference.clone();
        s.assignment.fields.threshold = threshold.into();
        let r = s.assignment.check(&d.initial).unwrap();
        assert!(r.configuration_error.is_some());
        assert!(r.cases.is_empty());
        assert!(!s.solved(&d));
    }
}
#[test]
fn partial_repairs_keep_valid_case_results_and_wrong_direction_or_width_cannot_pass() {
    let d = definition(false);
    let mut s = Session::new(&d);
    s.assignment.fields.description = "staff-uplink".into();
    let r = s.assignment.check(&d.initial).unwrap();
    assert!(!r.passed());
    assert!(r.cases.iter().find(|c| c.id == "fault").unwrap().passed);
    assert!(
        !r.cases
            .iter()
            .find(|c| c.id == "maintenance")
            .unwrap()
            .passed
    );
    let d = definition(true);
    for change in [0, 1, 2] {
        let mut f = d.initial.reference.clone();
        match change {
            0 => f.direction = Direction::Outgoing,
            1 => f.width = Width::Counter32,
            _ => f.threshold = "99".into(),
        };
        assert!(!d.initial.report(&f).unwrap().passed());
    }
}
#[test]
fn correct_success_is_invalidated_by_raw_edits_even_without_a_changed_notification() {
    let d = definition(true);
    let mut s = Session::new(&d);
    s.assignment.fields = d.initial.reference.clone();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.solved(&d));
    s.assignment.fields.direction = Direction::Outgoing;
    assert!(!s.assignment.checked() && !s.solved(&d));
    s.assignment.check(&d.initial).unwrap();
    assert!(!s.solved(&d));
}
#[test]
fn practice_fill_poll_check_and_selected_reset_do_not_replace_independent_work_or_credit() {
    for traffic in [false, true] {
        let d = definition(traffic);
        let mut s = Session::new(&d);
        let original = s.assignment.clone();
        s.practice.fill(&d.example).unwrap();
        assert!(!s.practice.checked());
        s.practice.poll(&d.example).unwrap();
        assert!(s.practice.check(&d.example).unwrap().passed());
        assert!(!s.solved(&d));
        assert_eq!(s.assignment, original);
        s.practice.reset(&d.example);
        assert_eq!(s.practice, Draft::new(&d.example));
        assert_eq!(s.assignment, original);
    }
}
#[test]
fn browser_get_and_next_derive_observations_and_edits_clear_stale_query_display() {
    let d = definition(false);
    let mut s = Session::new(&d);
    let q = &mut s.assignment.query;
    q.oid = "1.3.6.1.2.1.2.2.1.2".into();
    q.next = true;
    let result = s.assignment.poll(&d.initial).unwrap();
    assert!(
        matches!(result,Reply::Value{value:crate::snmp::Value::Text(ref label),..}if label=="staff-uplink")
    );
    assert!(s.assignment.observed(&d.initial).unwrap().is_ok());
    s.assignment.query.oid = "1.3.6.1.2.1.1.3.1".into();
    s.assignment.query.next = false;
    assert!(s.assignment.observed(&d.initial).is_none());
    assert_eq!(
        s.assignment.poll(&d.initial).unwrap(),
        Reply::NoSuchInstance
    );
    assert!(!s.assignment.checked());
    s.assignment.query.oid = "snmpwalk host".into();
    assert!(s.assignment.poll(&d.initial).is_err());
    assert!(s.assignment.observed(&d.initial).unwrap().is_err());
    assert!(!s.solved(&d));
}
#[test]
fn bounded_wrong_drafts_resume_without_becoming_correct_and_reset_restores_only_selected_work() {
    let d = definition(true);
    let mut s = Session::new(&d);
    s.view = View::Assignment;
    s.assignment.fields.threshold = "abc".into();
    s.practice.fields.threshold = "70".into();
    s.practice.query.oid = "1.3.6.1.2.1.2.2.1.2".into();
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&bytes).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored, s);
    assert!(!restored.solved(&d));
    let practice = restored.practice.clone();
    restored.assignment.reset(&d.initial);
    assert_eq!(restored.assignment, Draft::new(&d.initial));
    assert_eq!(restored.practice, practice);
}
#[test]
fn sources_reject_wrong_identity_versions_negative_only_cases_and_unreachable_references() {
    let d = definition(false);
    let mut variants = Vec::new();
    let mut bad = d.clone();
    bad.initial.version = 2;
    variants.push(bad);
    let mut bad = d.clone();
    bad.initial.cases[0].expected = Outcome::Alert;
    variants.push(bad);
    let mut bad = d.clone();
    bad.initial
        .cases
        .retain(|c| matches!(c.expected, Outcome::Unknown(_)));
    variants.push(bad);
    let mut bad = d.clone();
    bad.example = d.initial.clone();
    variants.push(bad);
    let mut bad = d.clone();
    bad.example.cases = d.initial.cases.clone();
    variants.push(bad);
    let mut bad = d.clone();
    bad.initial.cases[1].id = bad.initial.cases[0].id.clone();
    variants.push(bad);
    for bad in variants {
        assert!(bad.validate().is_err());
    }
}
#[test]
fn unauthorized_inactive_fields_forged_draft_identity_and_unknown_fields_refuse() {
    let d = definition(false);
    let mut s = Session::new(&d);
    s.assignment.fields.width = Width::Counter32;
    assert!(s.validate(&d).is_err());
    assert!(s.assignment.check(&d.initial).is_err());
    assert!(!s.solved(&d));
    s = Session::new(&d);
    s.assignment.task_id = "foreign".into();
    assert!(s.validate(&d).is_err());
    s = Session::new(&d);
    s.version = 2;
    assert!(s.validate(&d).is_err());
    let mut raw = serde_json::to_value(Session::new(&d)).unwrap();
    raw["run"] = serde_json::json!("host command");
    assert!(serde_json::from_value::<Session>(raw).is_err());
    let mut raw = serde_json::to_value(d).unwrap();
    raw["initial"]["cases"] = serde_json::json!(vec![raw["initial"]["cases"][0].clone(); 9]);
    assert!(serde_json::from_value::<Definition>(raw).is_err());
}

#[test]
fn different_ids_do_not_make_identical_example_measurements_independent_practice() {
    let mut d = definition(false);
    let separate_id = d.example.id.clone();
    d.example = d.initial.clone();
    d.example.id = separate_id;
    assert!(d.initial.validate().is_ok() && d.example.validate().is_ok());
    assert!(
        d.validate()
            .unwrap_err()
            .contains("different virtual measurements")
    );
}

// Synthetic source-binding fixture only: these IDs are never in the application catalog.
fn scenario(traffic: bool) -> crate::pack::Scenario {
    let d = definition(traffic);
    let solution = vec![
        format!(
            "@monitor-set {}",
            serde_json::to_string(&d.initial.reference).unwrap()
        ),
        "@monitor-check".into(),
    ];
    serde_json::from_value(serde_json::json!({
        "id":d.initial.id,"version":1,"track":"CCNA","topic":"Network monitoring","title":"Synthetic virtual monitoring binding fixture",
        "summary":"Verify monitoring against supplied virtual measurements.","objective":"Preserve required alert, quiet and unknown outcomes.",
        "difficulty":"Intermediate","minutes":20,"steps":["Inspect supplied virtual MIB values and configure the monitor."],
        "hints":["Read the interface description and the measurement type."],"sources":["https://www.rfc-editor.org/rfc/rfc3416.html"],
        "simplifications":"Supplied MIB snapshots only; no live SNMP, packets, security or host execution.",
        "initial":crate::engine::State::SnmpMonitor{simulation:Box::new(Session::new(&d))},
        "goal":crate::engine::Goal::SnmpMonitor{definition:Box::new(d)},"solution":solution
    })).unwrap()
}
#[test]
fn source_engine_actions_use_typed_current_state_and_unsupported_input_is_atomic() {
    for traffic in [false, true] {
        let s = scenario(traffic);
        source(&s).unwrap();
        let mut a = crate::progress::Attempt::new(&s, 1);
        let original = serde_json::to_value(&a.state).unwrap();
        for raw in [
            "touch should-not-run",
            "sh -c id",
            "@monitor-set {}",
            "@monitor-check extra",
            "@monitor-set {\"run\":\"host command\"}",
        ] {
            assert!(crate::engine::execute_for_goal(&mut a.state, &s.goal, raw).is_err());
            assert_eq!(serde_json::to_value(&a.state).unwrap(), original);
        }
        for raw in &s.solution {
            crate::engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
        }
        assert!(crate::progress::current_solved(&a, &s));
        assert!(crate::engine::checks(&a.state, &s.goal).iter().all(|c| c.0));
        let crate::engine::State::SnmpMonitor { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.fields.description = "not-a-supplied-link".into();
        assert!(!crate::progress::current_solved(&a, &s));
        crate::engine::execute_for_goal(&mut a.state, &s.goal, "@monitor-check").unwrap();
        assert!(!crate::progress::current_solved(&a, &s));
    }
}
#[test]
fn actual_pack_loader_rejects_mismatched_prechecked_and_unreachable_monitor_sources() {
    let s = scenario(true);
    let pack = |s: &crate::pack::Scenario| {
        serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[s]})).unwrap()
    };
    assert_eq!(crate::pack::load_catalog(&[&pack(&s)]).unwrap().len(), 1);
    let mut bad = s.clone();
    if let crate::engine::Goal::SnmpMonitor { definition } = &mut bad.goal {
        definition.initial.id = "foreign-monitor".into();
    }
    assert!(crate::pack::load_catalog(&[&pack(&bad)]).is_err());
    let mut bad = s.clone();
    if let crate::engine::State::SnmpMonitor { simulation } = &mut bad.initial {
        simulation.assignment.checked_fields = Some(simulation.assignment.fields.clone());
    }
    assert!(crate::pack::load_catalog(&[&pack(&bad)]).is_err());
    let mut bad = s.clone();
    if let crate::engine::Goal::SnmpMonitor { definition } = &mut bad.goal {
        definition.initial.reference = definition.initial.initial.clone();
    }
    assert!(crate::pack::load_catalog(&[&pack(&bad)]).is_err());
    let mut a = crate::progress::Attempt::new(&s, 1);
    a.scenario_version = 2;
    assert!(context(&a, &s).is_err());
    assert!(!crate::progress::current_solved(&a, &s));
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::SnmpMonitor { simulation } = &mut a.state {
        simulation.assignment.task_id = "foreign-monitor".into();
    }
    assert!(context(&a, &s).is_err());
    assert!(!crate::progress::current_solved(&a, &s));
}
#[test]
fn raw_shapes_allow_wrong_bounded_values_but_reject_versions_and_foreign_browser_cases() {
    let d = definition(true);
    let mut s = Session::new(&d);
    s.assignment.fields.threshold = "abc".into();
    s.assignment.query.oid = "not a numeric oid".into();
    s.shape().unwrap();
    s.validate(&d).unwrap();
    s.version = 2;
    assert!(s.shape().is_err());
    s = Session::new(&d);
    s.assignment.query.case = "foreign-case".into();
    assert!(s.validate(&d).is_err());
    s = Session::new(&d);
    s.assignment.query.oid = "a\nb".into();
    assert!(s.shape().is_err());
}
#[test]
fn disk_roundtrip_preserves_all_426_prior_attempts_favorites_and_both_new_drafts() {
    let mut sources = crate::pack::accepted426().unwrap();
    assert_eq!(sources.len(), 426);
    assert_eq!(crate::course::accepted426(&sources).unwrap().len(), 164);
    let s = scenario(true);
    assert!(!sources.iter().any(|v| v.id == s.id));
    sources.push(s.clone());
    let mut p = crate::progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &sources[..426] {
        let mut a = crate::progress::Attempt::new(source, 1);
        crate::workspace::prepare(&mut a);
        if crate::command_location_workspace::has_source(source) {
            crate::command_location_workspace::prepare(&mut a, source, true).unwrap();
        }
        if crate::accounts_template_workspace::has_source(source) {
            crate::accounts_template_workspace::prepare(&mut a, source, true).unwrap();
        }
        p.attempts.insert(source.id.clone(), a);
    }
    let prior = serde_json::to_value(&p.attempts).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::SnmpMonitor { simulation } = &mut a.state {
        simulation.assignment.fields.threshold = "abc".into();
        simulation.assignment.query.oid = "1.3.6.1.2.1.2.2.1.2".into();
        simulation.assignment.query.next = true;
        simulation
            .assignment
            .poll(&source(&s).unwrap().initial)
            .unwrap();
        simulation
            .practice
            .fill(&source(&s).unwrap().example)
            .unwrap();
        simulation
            .practice
            .check(&source(&s).unwrap().example)
            .unwrap();
    }
    p.attempts.insert(s.id.clone(), a.clone());
    p.favorites.insert(s.id.clone());
    p.favorites.insert(sources[0].id.clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261112/next-snmp/progress-tests");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    crate::progress::save_with_catalog(&path, &p, &sources).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let restored = crate::progress::load(&path, &sources).unwrap();
    assert_eq!(restored.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
    let mut old = restored.attempts.clone();
    old.remove(&s.id);
    assert_eq!(serde_json::to_value(old).unwrap(), prior);
    assert!(crate::progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let mut forged: crate::progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    if let crate::engine::State::SnmpMonitor { simulation } =
        &mut forged.attempts.get_mut(&s.id).unwrap().state
    {
        simulation.assignment.fields.link_rule = LinkRule::OperationalDown;
    }
    assert!(crate::progress::save_with_catalog(&path, &forged, &sources).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    p.attempts.get_mut(&s.id).unwrap().scenario_version = 2;
    assert!(crate::progress::save_with_catalog(&path, &p, &sources).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
