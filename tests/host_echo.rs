//! Finite, numeric simulated queries. No host ping, DNS, sockets or commands.
use dlms_learning_lab::{
    engine::Goal,
    host_echo as echo, host_workspace as work, pack,
    transport_core::{Disposition, echo::Phase},
};
fn fixture() -> dlms_learning_lab::linux_host::State {
    let scenarios = pack::load(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap();
    let s = scenarios
        .iter()
        .find(|s| s.id == "lpic1-host-handover")
        .unwrap();
    let Goal::HostNetwork { definition } = &s.goal else {
        panic!()
    };
    let mut session = work::Session::new(definition);
    for cmd in &s.solution {
        session.execute_reference(definition, cmd).unwrap();
    }
    assert!(definition.checks(&session, false).iter().all(|r| r.0));
    session.assignment.world
}
fn ip(s: &str) -> std::net::Ipv4Addr {
    s.parse().unwrap()
}
fn plan() -> echo::Plan {
    echo::Plan {
        peers: vec![echo::Peer {
            address: ip("203.0.113.80"),
            responds: true,
            return_sources: vec![
                dlms_learning_lab::ipv4_routing::Prefix::new(ip("192.0.2.0"), 24).unwrap(),
            ],
        }],
    }
}
#[test]
fn actual_two_events_reverse_endpoints_and_reflect_every_byte_without_mutation() {
    let world = fixture();
    let before = world.clone();
    let observed = plan().observe(&world, ip("203.0.113.80")).unwrap();
    let report = observed.report.as_ref().unwrap();
    assert!(report.round_trip());
    assert_eq!(report.events.len(), 2);
    let a = &report.events[0].packet;
    let b = &report.events[1].packet;
    assert_eq!((a.source, a.destination), (b.destination, b.source));
    assert_eq!(
        (a.identifier, a.sequence, &a.payload),
        (b.identifier, b.sequence, &b.payload)
    );
    assert_eq!(report.received.as_deref(), Some(a.payload.as_slice()));
    assert!(a.payload.contains(&0));
    assert_eq!(world, before);
    let out = observed.output();
    assert!(out.contains("matching replies: 1"));
    assert!(out.contains("no timing or real network"));
    assert!(out.contains("does not verify an application"));
    assert!(out.len() < 4096);
}
#[test]
fn available_peer_does_not_invent_a_response_or_return_route() {
    let world = fixture();
    assert!(world.probe(ip("203.0.113.80")).pass);
    let mut p = plan();
    p.peers[0].responds = false;
    let o = p.observe(&world, ip("203.0.113.80")).unwrap();
    let r = o.report.unwrap();
    assert_eq!(r.phase, Phase::ReplyReady);
    assert_eq!(r.events.len(), 1);
    assert!(!r.round_trip());
    assert!(r.received.is_none());
    p.peers[0].responds = true;
    p.peers[0].return_sources.clear();
    let o = p.observe(&world, ip("203.0.113.80")).unwrap();
    let r = o.report.unwrap();
    assert_eq!(r.phase, Phase::ReplyDropped);
    assert_eq!(r.events.len(), 2);
    assert!(!r.round_trip());
    assert_eq!(r.events[0].decision.disposition, Disposition::Delivered);
    assert_eq!(r.events[1].decision.disposition, Disposition::Dropped);
}
#[test]
fn wrong_source_changes_return_result_even_when_outward_path_is_usable() {
    let mut world = fixture();
    // This isolated fixture gives BOTH gateways an outward application path.
    // Only the explicit peer return-source policy differs; do not infer replies
    // from the older handover fixture's management-only second gateway.
    world
        .profile
        .gateways
        .iter_mut()
        .find(|g| g.device == "ens4")
        .unwrap()
        .exports
        .push(dlms_learning_lab::ipv4_routing::Prefix::new(ip("203.0.113.0"), 24).unwrap());
    world.validate().unwrap();
    world
        .execute("ip -4 route del default via 192.0.2.1 dev ens3 metric 5")
        .unwrap();
    world
        .execute("ip -4 route replace default via 198.51.100.1 dev ens4 metric 1")
        .unwrap();
    assert!(world.probe(ip("203.0.113.80")).pass);
    let o = plan().observe(&world, ip("203.0.113.80")).unwrap();
    assert_eq!(o.selection.unwrap().device, "ens4");
    assert_eq!(o.report.unwrap().phase, Phase::ReplyDropped);
}
#[test]
fn current_path_failure_prevents_reply_and_missing_route_does_not_fabricate_a_query() {
    let mut world = fixture();
    world.execute("ip link set dev ens3 down").unwrap();
    let o = plan().observe(&world, ip("203.0.113.80")).unwrap();
    let r = o.report.unwrap();
    assert_eq!(r.phase, Phase::RequestDropped);
    assert_eq!(r.events.len(), 1);
    assert!(r.received.is_none());
    let mut world = fixture();
    world.configuration.routes.clear();
    world.validate().unwrap();
    let o = plan().observe(&world, ip("203.0.113.80")).unwrap();
    assert!(o.selection.is_none() && o.report.is_none());
    assert!(o.output().contains("No query evaluated"));
}
#[test]
fn only_documented_literal_numeric_forms_are_supported() {
    for good in [
        "ping -c 1 203.0.113.80",
        "ping -4 -c 1 203.0.113.80",
        " ping\t-c  1 203.0.113.80 ",
    ] {
        assert_eq!(
            echo::Plan::command_target(good).unwrap(),
            ip("203.0.113.80")
        );
    }
    for bad in [
        "ping 203.0.113.80",
        "ping -c 2 203.0.113.80",
        "ping -c 0 203.0.113.80",
        "ping -c 99999999999999999999 203.0.113.80",
        "ping -6 -c 1 ::1",
        "ping -c 1 service.lab.test",
        "ping -c 1 203.0.113.080",
        "ping -f 203.0.113.80",
        "ping -c 1 203.0.113.80; id",
        "ping -c 1 $(id)",
        "ping -c 1 /etc/passwd",
        "ping -c 1 203.0.113.80\n",
        "ping -c 1 203.0.113.80 | cat",
    ] {
        assert!(echo::Plan::command_target(bad).is_err(), "{bad}");
    }
    assert!(echo::Plan::command_target(&"x".repeat(513)).is_err());
}
#[test]
fn bounded_source_facts_and_unsupported_destinations_fail_without_side_effects() {
    let world = fixture();
    let before = world.clone();
    let mut p = plan();
    for target in [
        "127.0.0.1",
        "8.8.8.8",
        "0.0.0.0",
        "255.255.255.255",
        "224.0.0.1",
    ] {
        assert!(p.observe(&world, ip(target)).is_err());
    }
    p.peers.push(p.peers[0].clone());
    assert!(p.validate(&world).is_err());
    let mut p = plan();
    p.peers[0].address = ip("8.8.8.8");
    assert!(p.validate(&world).is_err());
    let mut raw = serde_json::to_value(plan()).unwrap();
    raw["peers"] = serde_json::json!(vec![raw["peers"][0].clone(); 9]);
    assert!(serde_json::from_value::<echo::Plan>(raw).is_err());
    let mut raw = serde_json::to_value(plan()).unwrap();
    raw["peers"][0]["return_sources"] =
        serde_json::json!(vec![raw["peers"][0]["return_sources"][0].clone(); 5]);
    assert!(serde_json::from_value::<echo::Plan>(raw).is_err());
    let mut raw = serde_json::to_value(plan()).unwrap();
    raw["command"] = serde_json::json!("touch /tmp/no");
    assert!(serde_json::from_value::<echo::Plan>(raw).is_err());
    assert_eq!(world, before);
}

fn opted_in() -> (work::Definition, Vec<String>) {
    let scenarios = pack::load(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap();
    let source = scenarios
        .iter()
        .find(|s| s.id == "lpic1-host-handover")
        .unwrap();
    let Goal::HostNetwork { definition } = &source.goal else {
        panic!()
    };
    let mut d = (**definition).clone();
    let exercise = |target: &str, network: &str| echo::Exercise {
        plan: echo::Plan {
            peers: vec![echo::Peer {
                address: ip(target),
                responds: true,
                return_sources: vec![
                    dlms_learning_lab::ipv4_routing::Prefix::new(ip(network), 24).unwrap(),
                ],
            }],
        },
        required_replies: vec![ip(target)],
    };
    d.echo = Some(echo::Lesson {
        initial: exercise("203.0.113.80", "192.0.2.0"),
        example: exercise("10.73.0.80", "10.71.0.0"),
    });
    d.validate().unwrap();
    (d, source.solution.clone())
}
#[test]
fn opt_in_commands_are_read_only_and_old_definitions_keep_exact_serialized_shape() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap();
    for source in raw["scenarios"].as_array().unwrap() {
        let value = &source["goal"]["definition"];
        let d: work::Definition = serde_json::from_value(value.clone()).unwrap();
        assert!(d.echo.is_none());
        assert_eq!(serde_json::to_value(&d).unwrap(), *value);
        let mut s = work::Session::new(&d);
        let before = s.clone();
        assert!(
            s.run(
                &d,
                &work::Event::Command {
                    input: "ping -c 1 203.0.113.80".into()
                },
                false
            )
            .is_err()
        );
        assert_eq!(s, before);
    }
    let (d, solution) = opted_in();
    let mut s = work::Session::new(&d);
    for cmd in solution {
        s.execute_reference(&d, &cmd).unwrap();
    }
    s.assignment.checked = true;
    let before = s.assignment.clone();
    let out = s
        .run(
            &d,
            &work::Event::Command {
                input: "ping -c 1 203.0.113.80".into(),
            },
            false,
        )
        .unwrap();
    assert!(out.contains("matching replies: 1"));
    assert!(
        s.assignment.checked,
        "Read-only Echo must retain the current check summary"
    );
    assert_eq!(s.assignment.world, before.world);
    assert_eq!(s.assignment.history, before.history);
    d.bound(&s).unwrap();
    assert!(
        s.run(
            &d,
            &work::Event::Command {
                input: "help".into()
            },
            false
        )
        .unwrap()
        .contains("ping -c 1")
    );
    assert!(s.assignment.checked);
    s.run(
        &d,
        &work::Event::Command {
            input: "ip link set dev ens3 down".into(),
        },
        false,
    )
    .unwrap();
    assert!(
        !s.assignment.checked,
        "A real configuration change must clear feedback"
    );
    assert!(d.checks(&s, false).iter().any(|r| !r.0));
}
#[test]
fn source_bound_grading_recomputes_round_trips_and_separates_practice_resume_and_reset() {
    let (d, solution) = opted_in();
    let initial = work::Session::new(&d);
    let mut s = initial.clone();
    assert!(d.checks(&s, false).iter().any(|r| !r.0));
    for event in &d.example_actions {
        s.run(&d, event, true).unwrap();
    }
    assert!(d.checks(&s, true).iter().all(|r| r.0));
    assert_eq!(s.assignment, initial.assignment);
    assert!(d.checks(&s, false).iter().any(|r| !r.0));
    for cmd in solution {
        s.execute_reference(&d, &cmd).unwrap();
    }
    assert!(d.checks(&s, false).iter().all(|r| r.0));
    let saved = serde_json::to_vec(&s).unwrap();
    let mut reopened: work::Session = serde_json::from_slice(&saved).unwrap();
    d.bound(&reopened).unwrap();
    assert_eq!(s, reopened);
    reopened
        .run(
            &d,
            &work::Event::Command {
                input: "ip link set dev ens3 down".into(),
            },
            false,
        )
        .unwrap();
    let checks = d.checks(&reopened, false);
    assert!(
        checks
            .iter()
            .any(|r| !r.0 && r.1.contains("Echo round trip"))
    );
    assert!(checks.iter().any(|r| r.0));
    reopened.assignment = initial.assignment.clone();
    assert!(d.checks(&reopened, false).iter().any(|r| !r.0));
    assert_eq!(reopened.practice, s.practice);
    assert!(
        !String::from_utf8(saved).unwrap().contains("received"),
        "transient Echo reports are not progress authority"
    );
}
#[test]
fn broken_or_silent_peers_cannot_be_authorized_as_negative_grading_shortcuts() {
    let (mut d, _) = opted_in();
    d.echo.as_mut().unwrap().initial.plan.peers[0].responds = false;
    assert!(d.validate().is_err());
    let (mut d, _) = opted_in();
    d.echo.as_mut().unwrap().initial.required_replies.clear();
    assert!(d.validate().is_err());
    let (mut d, _) = opted_in();
    d.echo
        .as_mut()
        .unwrap()
        .initial
        .required_replies
        .push(ip("8.8.8.8"));
    assert!(d.validate().is_err());
    let (mut d, _) = opted_in();
    d.echo.as_mut().unwrap().initial.plan.peers[0]
        .return_sources
        .clear();
    assert!(d.validate().is_err());
}
