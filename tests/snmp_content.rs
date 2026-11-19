//! Two real authored monitoring assignments, not generated numerical variants.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt},
    snmp::{
        Width,
        monitor::{Direction, LinkRule, Outcome, UnknownCause},
    },
    snmp_workspace as work,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["ccna-snmp-link-monitor", "ccna-snmp-counter-monitor"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::current().unwrap())
}
fn source(n: usize) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == IDS[n]).unwrap()
}
#[test]
fn accepted428_catalog_retains_all426_prior_scenarios_and164_units_with_a_real_prerequisite_unit() {
    let old = pack::accepted426().unwrap();
    let snapshot = pack::accepted428().unwrap();
    assert_eq!((old.len(), snapshot.len()), (426, 428));
    assert_eq!(
        serde_json::to_value(&snapshot[..426]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = course::accepted426(&old).unwrap();
    let units = course::accepted428(&snapshot).unwrap();
    assert_eq!((before.len(), units.len()), (164, 165));
    let added = units
        .iter()
        .position(|u| u.id == "ccna-snmp-monitoring-foundations")
        .unwrap();
    assert_eq!(units[added - 1].id, "management-policy-foundations");
    assert_eq!(units[added].labs, IDS);
    assert_eq!(
        units[added].prerequisites,
        [
            "ccna-transport-conversations",
            "management-policy-foundations"
        ]
    );
    let retained = units
        .iter()
        .filter(|u| u.id != "ccna-snmp-monitoring-foundations")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let old_tail: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v26.json")).unwrap();
    let tail: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v27.json")).unwrap();
    assert_eq!((old_tail.scenarios.len(), tail.scenarios.len()), (53, 55));
    assert_eq!(
        serde_json::to_value(&tail.scenarios[..53]).unwrap(),
        serde_json::to_value(old_tail.scenarios).unwrap()
    );
}
#[test]
fn every_authored_reference_and_legitimate_alternative_meets_real_behavior_cases() {
    for n in 0..2 {
        let s = source(n);
        let d = work::source(s).unwrap();
        assert_ne!(
            d.initial.activity,
            if n == 0 {
                work::Activity::Traffic
            } else {
                work::Activity::Link
            }
        );
        let mut a = Attempt::new(s, 1);
        assert!(!progress::current_solved(&a, s));
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        assert!(engine::checks(&a.state, &s.goal).iter().all(|c| c.0));
        if n == 1 {
            for threshold in ["70", "80", "90", "080"] {
                let mut next = Attempt::new(s, 1);
                let State::SnmpMonitor { simulation } = &mut next.state else {
                    panic!()
                };
                simulation.assignment.fields = d.initial.reference.clone();
                simulation.assignment.fields.threshold = threshold.into();
                assert!(simulation.assignment.check(&d.initial).unwrap().passed());
                assert!(progress::current_solved(&next, s));
            }
        }
    }
}
#[test]
fn plausible_wrong_partial_and_malformed_solutions_do_not_pass_or_hide_passing_cases() {
    let s = source(0);
    let d = work::source(s).unwrap();
    let mut f = d.initial.reference.clone();
    f.link_rule = LinkRule::OperationalDown;
    let r = d.initial.report(&f).unwrap();
    assert!(!r.passed());
    assert!(
        r.cases
            .iter()
            .find(|c| c.id == "unexpected-down")
            .unwrap()
            .passed
    );
    assert!(
        !r.cases
            .iter()
            .find(|c| c.id == "maintenance")
            .unwrap()
            .passed
    );
    let s = source(1);
    let d = work::source(s).unwrap();
    for change in 0..5 {
        let mut f = d.initial.reference.clone();
        match change {
            0 => f.direction = Direction::Outgoing,
            1 => f.width = Width::Counter32,
            2 => f.threshold = "25".into(),
            3 => f.threshold = "99".into(),
            _ => f.description = "visitor-uplink".into(),
        };
        assert!(!d.initial.report(&f).unwrap().passed());
    }
    for raw in ["", "abc", "101", "٨٠"] {
        let mut a = Attempt::new(s, 1);
        let State::SnmpMonitor { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.fields = d.initial.reference.clone();
        simulation.assignment.fields.threshold = raw.into();
        simulation.validate(d).unwrap();
        let r = simulation.assignment.check(&d.initial).unwrap();
        assert!(r.configuration_error.is_some());
        assert!(r.cases.is_empty());
        assert!(!progress::current_solved(&a, s));
        let restored: Attempt = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        work::context(&restored, s).unwrap();
        assert!(!progress::current_solved(&restored, s));
    }
}
#[test]
fn worked_counter_arithmetic_matches_actual_samples_and_teaches_when32bits_are_sufficient() {
    let d = work::source(source(1)).unwrap();
    let case = d.example.cases.iter().find(|c| c.id == "busy").unwrap();
    let b = case.stream.before.as_ref().unwrap();
    let a = &case.stream.after;
    assert_eq!((b.at_seconds, a.at_seconds), (0, 20));
    assert_eq!(
        (
            b.agent.interfaces[0].in_octets,
            a.agent.interfaces[0].in_octets
        ),
        (2000, 175002000)
    );
    assert_eq!(a.agent.interfaces[0].speed_bits_per_second, 100_000_000);
    let mut reference = d.example.reference.clone();
    reference.width = Width::Counter32;
    assert!(d.example.report(&reference).unwrap().passed());
    let mut independent = d.initial.reference.clone();
    independent.width = Width::Counter32;
    let r = d.initial.report(&independent).unwrap();
    assert!(!r.passed());
    let busy = r.cases.iter().find(|c| c.id == "busy").unwrap();
    assert_eq!(
        busy.actual.outcome,
        Outcome::Unknown(UnknownCause::UnusableCounter)
    );
    assert!(busy.actual.explanation.contains("whole counter wrap"));
}
#[test]
fn actual_example_poll_fill_check_and_reset_keep_assignment_draft_and_credit_separate() {
    for n in 0..2 {
        let s = source(n);
        let d = work::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        let State::SnmpMonitor { simulation } = &mut a.state else {
            panic!()
        };
        let before = simulation.assignment.clone();
        simulation.practice.fill(&d.example).unwrap();
        assert!(!simulation.practice.checked());
        simulation.practice.poll(&d.example).unwrap();
        simulation.practice.check(&d.example).unwrap();
        assert_eq!(simulation.assignment, before);
        simulation.practice.reset(&d.example);
        assert_eq!(simulation.assignment, before);
        assert_eq!(simulation.practice, work::Draft::new(&d.example));
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn edits_break_current_success_while_view_and_browser_navigation_preserve_it() {
    for n in 0..2 {
        let s = source(n);
        let d = work::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        if let State::SnmpMonitor { simulation } = &mut a.state {
            simulation.view = dlms_learning_lab::script_teaching::View::Example;
            simulation.assignment.query.case = "unavailable".into();
            let _ = simulation.assignment.poll(&d.initial);
        }
        assert!(progress::current_solved(&a, s));
        if let State::SnmpMonitor { simulation } = &mut a.state {
            simulation.assignment.fields.description = "missing-target".into();
        }
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, "@monitor-check").unwrap();
        assert!(!progress::current_solved(&a, s));
        let resumed: Attempt = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        work::context(&resumed, s).unwrap();
        assert!(!progress::current_solved(&resumed, s));
    }
}
