use super::*;
fn agent() -> Agent {
    Agent {
        uptime_ticks: 10_000,
        available: true,
        interfaces: vec![Interface {
            index: 2,
            description: "staff-uplink".into(),
            admin: LinkState::Up,
            oper: LinkState::Up,
            speed_bits_per_second: 1_000_000_000,
            in_octets: 1000,
            out_octets: 2000,
            discontinuity_ticks: 0,
        }],
    }
}
fn traffic(percent: u64) -> Stream {
    let b = agent();
    let mut a = b.clone();
    a.uptime_ticks += 6000;
    a.interfaces[0].in_octets += 7_500_000_000 * percent / 100;
    a.interfaces[0].out_octets += 750_000_000;
    Stream {
        before: Some(Snapshot {
            at_seconds: 0,
            agent: b,
        }),
        after: Snapshot {
            at_seconds: 60,
            agent: a,
        },
    }
}
fn link(admin: LinkState, oper: LinkState) -> Stream {
    let mut a = agent();
    a.interfaces[0].admin = admin;
    a.interfaces[0].oper = oper;
    Stream {
        before: None,
        after: Snapshot {
            at_seconds: 0,
            agent: a,
        },
    }
}
fn link_plan(rule: LinkRule) -> Plan {
    Plan::Link {
        description: "staff-uplink".into(),
        rule,
    }
}
fn traffic_plan(threshold: u8) -> Plan {
    Plan::Traffic {
        description: "staff-uplink".into(),
        direction: Direction::Incoming,
        width: Width::Counter64,
        threshold_percent: threshold,
    }
}
#[test]
fn link_monitor_distinguishes_unplanned_failure_from_planned_shutdown() {
    let p = link_plan(LinkRule::UnplannedDown);
    assert_eq!(
        observe(&p, &link(LinkState::Up, LinkState::Down))
            .unwrap()
            .outcome,
        Outcome::Alert
    );
    assert_eq!(
        observe(&p, &link(LinkState::Down, LinkState::Down))
            .unwrap()
            .outcome,
        Outcome::Quiet
    );
    assert_eq!(
        observe(&p, &link(LinkState::Up, LinkState::Up))
            .unwrap()
            .outcome,
        Outcome::Quiet
    );
    assert_eq!(
        observe(
            &link_plan(LinkRule::OperationalDown),
            &link(LinkState::Down, LinkState::Down)
        )
        .unwrap()
        .outcome,
        Outcome::Alert
    );
}
#[test]
fn interface_discovery_follows_ifdescr_numeric_instances_rather_than_one_fixed_index() {
    for index in [2, 10, 223] {
        let mut s = link(LinkState::Up, LinkState::Down);
        s.after.agent.interfaces[0].index = index;
        let r = observe(&link_plan(LinkRule::UnplannedDown), &s).unwrap();
        assert_eq!(r.interface_index, Some(index));
        assert_eq!(r.outcome, Outcome::Alert);
        assert!(r.polls.iter().any(|p| p.next));
        assert!(
            r.polls
                .iter()
                .any(|p| p.requested == Oid::instance(OPER, index))
        );
    }
}
#[test]
fn quiet_alert_and_unknown_come_from_multiple_cases_and_accept_equivalent_thresholds() {
    for threshold in [70, 80, 90] {
        let p = traffic_plan(threshold);
        assert_eq!(observe(&p, &traffic(90)).unwrap().outcome, Outcome::Alert);
        assert_eq!(observe(&p, &traffic(40)).unwrap().outcome, Outcome::Quiet);
    }
    assert_eq!(
        observe(&traffic_plan(91), &traffic(90)).unwrap().outcome,
        Outcome::Quiet
    );
    let r = observe(&traffic_plan(80), &traffic(90)).unwrap();
    assert_eq!(
        r.rate,
        Some(Rate {
            octets: 6_750_000_000,
            seconds: 60
        })
    );
    assert!(
        r.polls
            .iter()
            .any(|p| p.requested == Oid::instance(IN64, 2))
    );
}
#[test]
fn wrong_direction_and_ambiguous_counter_width_do_not_fake_the_intended_alert() {
    let mut p = traffic_plan(80);
    if let Plan::Traffic { direction, .. } = &mut p {
        *direction = Direction::Outgoing
    }
    assert_eq!(observe(&p, &traffic(90)).unwrap().outcome, Outcome::Quiet);
    if let Plan::Traffic {
        direction, width, ..
    } = &mut p
    {
        *direction = Direction::Incoming;
        *width = Width::Counter32
    }
    let r = observe(&p, &traffic(90)).unwrap();
    assert_eq!(r.outcome, Outcome::Unknown(UnknownCause::UnusableCounter));
    assert!(r.explanation.contains("whole counter wrap"));
    assert!(r.rate.is_none());
}
#[test]
fn resets_and_uptime_changes_produce_unknown_instead_of_a_false_service_result() {
    let p = traffic_plan(80);
    let mut s = traffic(90);
    s.after.agent.interfaces[0].discontinuity_ticks = 12000;
    let r = observe(&p, &s).unwrap();
    assert_eq!(r.outcome, Outcome::Unknown(UnknownCause::UnusableCounter));
    assert!(r.explanation.contains("discontinuity"));
    s.after.agent.interfaces[0].discontinuity_ticks = 0;
    s.after.agent.uptime_ticks = 200;
    let r = observe(&p, &s).unwrap();
    assert_eq!(r.outcome, Outcome::Unknown(UnknownCause::UnusableCounter));
    assert!(r.explanation.contains("uptime"));
}
#[test]
fn missing_unavailable_and_ambiguous_are_distinct_from_quiet() {
    let p = link_plan(LinkRule::UnplannedDown);
    let mut s = link(LinkState::Up, LinkState::Up);
    s.after.agent.available = false;
    assert_eq!(
        observe(&p, &s).unwrap().outcome,
        Outcome::Unknown(UnknownCause::UnavailableAgent)
    );
    s.after.agent.available = true;
    s.after.agent.interfaces[0].description = "visitor-uplink".into();
    assert_eq!(
        observe(&p, &s).unwrap().outcome,
        Outcome::Unknown(UnknownCause::MissingInterface)
    );
    s.after.agent.interfaces[0].description = "staff-uplink".into();
    let mut other = s.after.agent.interfaces[0].clone();
    other.index = 10;
    s.after.agent.interfaces.push(other);
    assert_eq!(
        observe(&p, &s).unwrap().outcome,
        Outcome::Unknown(UnknownCause::AmbiguousDescription)
    );
}
#[test]
fn source_and_configuration_errors_are_errors_not_successful_unknown_cases() {
    let mut p = traffic_plan(0);
    assert!(observe(&p, &traffic(90)).is_err());
    if let Plan::Traffic {
        threshold_percent, ..
    } = &mut p
    {
        *threshold_percent = 101
    }
    assert!(observe(&p, &traffic(90)).is_err());
    let mut s = traffic(90);
    s.after.agent.interfaces[0].index = 0;
    assert!(observe(&traffic_plan(80), &s).is_err());
}
#[test]
fn different_instances_and_capacity_cannot_be_stitched_into_one_rate() {
    let p = traffic_plan(80);
    let mut s = traffic(90);
    s.after.agent.interfaces[0].index = 10;
    assert_eq!(
        observe(&p, &s).unwrap().outcome,
        Outcome::Unknown(UnknownCause::ChangedInterface)
    );
    s.after.agent.interfaces[0].index = 2;
    s.after.agent.interfaces[0].speed_bits_per_second = 100_000_000;
    assert_eq!(
        observe(&p, &s).unwrap().outcome,
        Outcome::Unknown(UnknownCause::ChangedCapacity)
    );
    s.before = None;
    assert_eq!(
        observe(&p, &s).unwrap().outcome,
        Outcome::Unknown(UnknownCause::MissingMeasurement)
    );
}
#[test]
fn maximum_supported_table_has_finite_complete_discovery_and_poll_trace() {
    let mut s = traffic(90);
    for a in [&mut s.before.as_mut().unwrap().agent, &mut s.after.agent] {
        let original = a.interfaces[0].clone();
        a.interfaces = (1..=12)
            .map(|index| {
                let mut i = original.clone();
                i.index = index;
                i.description = if index == 12 {
                    "staff-uplink".into()
                } else {
                    format!("other-{index}")
                };
                i
            })
            .collect();
    }
    let r = observe(&traffic_plan(80), &s).unwrap();
    assert_eq!(r.interface_index, Some(12));
    assert_eq!(r.outcome, Outcome::Alert);
    assert_eq!(r.polls.len(), 34);
    assert!(r.polls.len() <= 40);
}
#[test]
fn serialization_is_data_only_and_strict_with_no_accepted_execution_field() {
    let p = traffic_plan(80);
    let raw = serde_json::to_value(&p).unwrap();
    assert_eq!(serde_json::from_value::<Plan>(raw.clone()).unwrap(), p);
    let mut bad = raw;
    bad["run"] = serde_json::json!("host command");
    assert!(serde_json::from_value::<Plan>(bad).is_err());
    let s = traffic(90);
    assert_eq!(
        serde_json::from_str::<Stream>(&serde_json::to_string(&s).unwrap()).unwrap(),
        s
    );
}

#[test]
fn readable_rate_feedback_does_not_round_the_behavioral_threshold_comparison() {
    let p = traffic_plan(80);
    let mut s = traffic(80);
    let r = observe(&p, &s).unwrap();
    assert_eq!(r.outcome, Outcome::Alert);
    assert!(r.explanation.contains("800.0 Mbit/s") && r.explanation.contains("80.0%"));
    s.after.agent.interfaces[0].in_octets -= 1;
    assert_eq!(observe(&p, &s).unwrap().outcome, Outcome::Quiet);
}
