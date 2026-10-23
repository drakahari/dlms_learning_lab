use dlms_learning_lab::{
    acl::{
        self, AccessList, Action, Kind, PacketProtocol, Policy, Ports, Protocol, Rule, Selector,
        Wildcard,
    },
    acl_routing::{
        self as ar, Attachment, Baseline, DevicePolicy, Direction, Endpoint, Event, ExpectedCause,
        ExpectedDenial, FailureKind, Flow, Location, Network, Outcome,
    },
    ipv4_routing::{Assignment, Device, Interface, Prefix, Route, Topology},
};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn interface(link: &str, address: &str, prefix: u8) -> Interface {
    Interface {
        link: link.into(),
        up: true,
        address: Some(Assignment {
            address: ip(address),
            prefix,
        }),
    }
}
fn route(network: &str, interface: &str, via: &str) -> Route {
    Route {
        destination: Prefix::parse(network).unwrap(),
        interface: interface.into(),
        via: ip(via),
        distance: 1,
    }
}
fn device(forwarding: bool, ports: Vec<(&str, Interface)>, routes: Vec<Route>) -> Device {
    Device {
        forwarding,
        interfaces: ports.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        routes,
    }
}
fn network() -> Network {
    Network {
        schema_version: 1,
        host_endpoints: BTreeMap::from([
            ("Client".into(), "lan".into()),
            ("Server".into(), "lan".into()),
        ]),
        topology: Topology {
            devices: BTreeMap::from([
                (
                    "Client".into(),
                    device(
                        false,
                        vec![("lan", interface("office", "192.0.2.10", 24))],
                        vec![route("0.0.0.0/0", "lan", "192.0.2.1")],
                    ),
                ),
                (
                    "R1".into(),
                    device(
                        true,
                        vec![
                            ("office", interface("office", "192.0.2.1", 24)),
                            ("transit", interface("backbone", "10.0.0.1", 30)),
                        ],
                        vec![route("198.51.100.0/24", "transit", "10.0.0.2")],
                    ),
                ),
                (
                    "R2".into(),
                    device(
                        true,
                        vec![
                            ("transit", interface("backbone", "10.0.0.2", 30)),
                            ("branch", interface("branch", "198.51.100.1", 24)),
                        ],
                        vec![route("192.0.2.0/24", "transit", "10.0.0.1")],
                    ),
                ),
                (
                    "Server".into(),
                    device(
                        false,
                        vec![("lan", interface("branch", "198.51.100.20", 24))],
                        vec![route("0.0.0.0/0", "lan", "198.51.100.1")],
                    ),
                ),
            ]),
        },
        devices: BTreeMap::new(),
    }
}
fn endpoint(device: &str, interface: &str) -> Endpoint {
    Endpoint {
        device: device.into(),
        interface: interface.into(),
    }
}
fn flow(port: u16) -> Flow {
    Flow {
        source: endpoint("Client", "lan"),
        destination: endpoint("Server", "lan"),
        protocol: PacketProtocol::Tcp {
            source_port: 50000,
            destination_port: port,
        },
    }
}
fn location(device: &str, interface: &str, direction: Direction) -> Location {
    Location {
        device: device.into(),
        interface: interface.into(),
        direction,
    }
}
fn rule(id: &str, action: Action, protocol: Protocol) -> Rule {
    Rule {
        id: id.into(),
        action,
        selector: Selector::Extended {
            source: Wildcard::any(),
            destination: Wildcard::any(),
            protocol,
        },
    }
}
fn tcp_port(port: u16) -> Protocol {
    Protocol::Tcp {
        source_port: Ports::Any {},
        destination_port: Ports::Eq { port },
    }
}
fn attach(n: &mut Network, at: &Location, rules: Vec<Rule>) {
    let c = n.devices.entry(at.device.clone()).or_insert(DevicePolicy {
        policy: Policy {
            schema_version: 1,
            lists: BTreeMap::new(),
        },
        attachments: BTreeMap::new(),
    });
    c.policy.lists.insert(
        "edge".into(),
        AccessList {
            kind: Kind::Extended,
            rules,
        },
    );
    let a = c.attachments.entry(at.interface.clone()).or_default();
    match at.direction {
        Direction::Inbound => a.inbound = Some("edge".into()),
        Direction::Outbound => a.outbound = Some("edge".into()),
    }
}
fn explicit(at: Location) -> ExpectedDenial {
    ExpectedDenial {
        at,
        cause: ExpectedCause::Explicit { rule: None },
    }
}
fn events(o: &Outcome) -> &[Event] {
    match o {
        Outcome::Delivered { events, .. }
        | Outcome::Denied { events, .. }
        | Outcome::Failed { events, .. } => events,
    }
}

#[test]
fn no_attachment_preserves_old_forward_path_without_auto_reply_or_mutation() {
    let n = network();
    let before = serde_json::to_vec(&n).unwrap();
    let old = serde_json::to_vec(&n.topology).unwrap();
    let f = flow(443);
    let observation = n.inspect(&f).unwrap();
    assert!(observation.delivered());
    let Baseline::Deliverable(baseline) = &observation.baseline else {
        panic!()
    };
    let Outcome::Delivered { trace, events } = &observation.actual else {
        panic!()
    };
    assert_eq!(trace, baseline);
    assert_eq!(
        trace,
        &n.topology
            .verify_endpoint("Client", "Server", "lan")
            .unwrap()
            .forward
    );
    assert_eq!(observation.packet.source, ip("192.0.2.10"));
    assert_eq!(observation.packet.destination, ip("198.51.100.20"));
    assert_eq!(events.len(), 6);
    assert!(
        matches!(&events[0],Event::Boundary{at,decision:None}if *at==location("R1","office",Direction::Inbound))
    );
    assert!(
        matches!(&events[1],Event::RouteSelected{device,selected}if device=="R1"&&selected.interface=="transit")
    );
    assert!(
        matches!(&events[2],Event::Boundary{at,decision:None}if *at==location("R1","transit",Direction::Outbound))
    );
    // A missing return route does not change this directed request, unlike the old round-trip probe.
    let mut asymmetric = n.clone();
    asymmetric
        .topology
        .devices
        .get_mut("R2")
        .unwrap()
        .routes
        .clear();
    assert!(asymmetric.inspect(&f).unwrap().delivered());
    assert!(
        asymmetric
            .topology
            .verify_endpoint("Client", "Server", "lan")
            .is_err()
    );
    assert_eq!(serde_json::to_vec(&n).unwrap(), before);
    assert_eq!(serde_json::to_vec(&n.topology).unwrap(), old);
    for _ in 0..20 {
        assert_eq!(n.inspect(&f).unwrap(), observation);
    }
}

#[test]
fn inbound_denies_before_route_lookup_and_baseline_failures_never_pass_guard() {
    let mut n = network();
    let at = location("R1", "office", Direction::Inbound);
    attach(
        &mut n,
        &at,
        vec![
            rule("ssh", Action::Deny, tcp_port(22)),
            rule("rest", Action::Permit, Protocol::Ip {}),
        ],
    );
    let good = n.inspect(&flow(443)).unwrap();
    let blocked = n.inspect(&flow(22)).unwrap();
    assert!(ar::guarded_denial(
        &blocked,
        &explicit(at.clone()),
        &[&good]
    ));
    assert!(
        matches!(&blocked.actual,Outcome::Denied{denial,events}if denial.at==at&&events.len()==1&&denial.decision.cause==acl::Cause::Matched{rule:"ssh".into(),row:0})
    );
    n.topology.devices.get_mut("R1").unwrap().routes.clear();
    let broken = n.inspect(&flow(22)).unwrap();
    assert!(matches!(broken.baseline, Baseline::Failed(_)));
    assert!(matches!(&broken.actual,Outcome::Denied{events,..}if events.len()==1));
    assert!(!broken.denied_at(&explicit(at.clone())));
    assert!(!ar::guarded_denial(&broken, &explicit(at), &[&good]));
}

#[test]
fn egress_checks_after_route_choice_before_neighbor_and_never_runs_without_route() {
    let mut n = network();
    let at = location("R1", "transit", Direction::Outbound);
    attach(
        &mut n,
        &at,
        vec![rule("block", Action::Deny, Protocol::Ip {})],
    );
    let blocked = n.inspect(&flow(443)).unwrap();
    let observed = events(&blocked.actual);
    assert!(matches!(&observed[0],Event::Boundary{at,..}if at.direction==Direction::Inbound));
    assert!(matches!(&observed[1],Event::RouteSelected{device,..}if device=="R1"));
    assert!(matches!(&observed[2],Event::Boundary{at:hit,..}if hit==&at));
    // Neighbor resolution is downstream of outbound filtering. Its failure remains independent.
    n.topology
        .devices
        .get_mut("R2")
        .unwrap()
        .interfaces
        .get_mut("transit")
        .unwrap()
        .up = false;
    let no_neighbor = n.inspect(&flow(443)).unwrap();
    assert!(matches!(no_neighbor.baseline, Baseline::Failed(_)));
    assert!(matches!(no_neighbor.actual, Outcome::Denied { .. }));
    assert!(!no_neighbor.denied_at(&explicit(at.clone())));
    n.topology.devices.get_mut("R1").unwrap().routes.clear();
    let no_route = n.inspect(&flow(443)).unwrap();
    assert!(matches!(no_route.actual, Outcome::Failed { .. }));
    assert_eq!(events(&no_route.actual).len(), 1);
}

#[test]
fn attachment_direction_and_reverse_packets_are_independent_and_names_are_local() {
    let mut n = network();
    let outbound = location("R1", "office", Direction::Outbound);
    attach(
        &mut n,
        &outbound,
        vec![rule("reply-block", Action::Deny, Protocol::Ip {})],
    );
    assert!(n.inspect(&flow(443)).unwrap().delivered());
    let reply = Flow {
        source: endpoint("Server", "lan"),
        destination: endpoint("Client", "lan"),
        protocol: PacketProtocol::Tcp {
            source_port: 443,
            destination_port: 50000,
        },
    };
    let blocked = n.inspect(&reply).unwrap();
    assert!(blocked.denied_at(&explicit(outbound.clone())));
    assert!(!blocked.denied_at(&explicit(location("R1", "office", Direction::Inbound))));
    let other = location("R2", "branch", Direction::Outbound);
    attach(
        &mut n,
        &other,
        vec![rule("different-edge", Action::Permit, Protocol::Ip {})],
    );
    assert!(n.inspect(&flow(443)).unwrap().delivered());
    assert_eq!(
        n.devices["R1"].policy.lists["edge"].rules[0].action,
        Action::Deny
    );
    assert_eq!(
        n.devices["R2"].policy.lists["edge"].rules[0].action,
        Action::Permit
    );
    assert!(n.inspect(&reply).unwrap().denied_at(&explicit(outbound)));
}

#[test]
fn equivalent_ordered_policies_pass_and_partial_permit_all_deny_all_or_wrong_cause_fail() {
    let mut n = network();
    let at = location("R1", "office", Direction::Inbound);
    attach(
        &mut n,
        &at,
        vec![
            rule("ssh", Action::Deny, tcp_port(22)),
            rule("web", Action::Permit, tcp_port(443)),
            rule(
                "dns",
                Action::Permit,
                Protocol::Udp {
                    source_port: Ports::Any {},
                    destination_port: Ports::Eq { port: 53 },
                },
            ),
        ],
    );
    let good = n.inspect(&flow(443)).unwrap();
    let blocked = n.inspect(&flow(22)).unwrap();
    assert!(ar::guarded_denial(
        &blocked,
        &explicit(at.clone()),
        &[&good]
    ));
    let implicit = ExpectedDenial {
        at: at.clone(),
        cause: ExpectedCause::Implicit {},
    };
    assert!(!blocked.denied_at(&implicit));
    let other = n.inspect(&flow(25)).unwrap();
    assert!(other.denied_at(&implicit));
    let named = ExpectedDenial {
        at: at.clone(),
        cause: ExpectedCause::Explicit {
            rule: Some("ssh".into()),
        },
    };
    assert!(blocked.denied_at(&named));
    let mut alternate = n.clone();
    let list = &mut alternate
        .devices
        .get_mut("R1")
        .unwrap()
        .policy
        .lists
        .get_mut("edge")
        .unwrap()
        .rules;
    list[0].id = "equivalent-name".into();
    if let Selector::Extended { protocol, .. } = &mut list[0].selector {
        *protocol = Protocol::Tcp {
            source_port: Ports::Range {
                first: 0,
                last: 65535,
            },
            destination_port: Ports::Range {
                first: 22,
                last: 22,
            },
        };
    }
    let alt_good = alternate.inspect(&flow(443)).unwrap();
    let alt_block = alternate.inspect(&flow(22)).unwrap();
    assert!(ar::guarded_denial(
        &alt_block,
        &explicit(at.clone()),
        &[&alt_good]
    ));
    assert!(!alt_block.denied_at(&named));
    for rules in [
        vec![rule("deny-all", Action::Deny, Protocol::Ip {})],
        vec![rule("permit-all", Action::Permit, Protocol::Ip {})],
        vec![rule("web-only", Action::Permit, tcp_port(443))],
        vec![
            rule("shadow", Action::Permit, Protocol::Ip {}),
            rule("ssh", Action::Deny, tcp_port(22)),
        ],
    ] {
        let mut bad = n.clone();
        attach(&mut bad, &at, rules);
        let b = bad.inspect(&flow(22)).unwrap();
        let p = bad.inspect(&flow(443)).unwrap();
        assert!(!ar::guarded_denial(&b, &explicit(at.clone()), &[&p]));
    }
    assert!(!ar::guarded_denial(&blocked, &explicit(at.clone()), &[]));
    assert!(!ar::guarded_denial(
        &blocked,
        &explicit(at),
        &[&good; ar::MAX_POSITIVES + 1]
    ));
}

#[test]
fn unrelated_positive_bypass_does_not_justify_a_denied_boundary_and_all_positives_must_work() {
    let mut n = network();
    let at = location("R1", "transit", Direction::Outbound);
    attach(
        &mut n,
        &at,
        vec![
            rule("ssh", Action::Deny, tcp_port(22)),
            rule("rest", Action::Permit, Protocol::Ip {}),
        ],
    );
    let blocked = n.inspect(&flow(22)).unwrap();
    let good = n.inspect(&flow(443)).unwrap();
    let reverse = Flow {
        source: endpoint("Server", "lan"),
        destination: endpoint("Client", "lan"),
        protocol: PacketProtocol::Icmp {},
    };
    let unrelated = n.inspect(&reverse).unwrap();
    assert!(unrelated.delivered());
    assert!(!ar::guarded_denial(
        &blocked,
        &explicit(at.clone()),
        &[&unrelated]
    ));
    assert!(ar::guarded_denial(
        &blocked,
        &explicit(at.clone()),
        &[&unrelated, &good]
    ));
    assert!(!ar::guarded_denial(
        &blocked,
        &explicit(at),
        &[&good, &blocked]
    ));
}

#[test]
fn path_breaks_shutdown_forwarding_loops_and_relocated_endpoints_never_become_policy_success() {
    let mut n = network();
    let at = location("R1", "office", Direction::Inbound);
    attach(
        &mut n,
        &at,
        vec![
            rule("ssh", Action::Deny, tcp_port(22)),
            rule("rest", Action::Permit, Protocol::Ip {}),
        ],
    );
    let good = n.inspect(&flow(443)).unwrap();
    for what in 0..5 {
        let mut broken = n.clone();
        match what {
            0 => broken.topology.devices.get_mut("R1").unwrap().forwarding = false,
            1 => {
                broken
                    .topology
                    .devices
                    .get_mut("R1")
                    .unwrap()
                    .interfaces
                    .get_mut("transit")
                    .unwrap()
                    .up = false
            }
            2 => broken
                .topology
                .devices
                .get_mut("R2")
                .unwrap()
                .routes
                .push(route("198.51.100.20/32", "transit", "10.0.0.1")),
            3 => {
                broken
                    .topology
                    .devices
                    .get_mut("Server")
                    .unwrap()
                    .interfaces
                    .get_mut("lan")
                    .unwrap()
                    .address = None;
                broken
                    .topology
                    .devices
                    .get_mut("R1")
                    .unwrap()
                    .interfaces
                    .get_mut("transit")
                    .unwrap()
                    .address = Some(Assignment {
                    address: ip("198.51.100.20"),
                    prefix: 24,
                });
            }
            4 => {
                broken
                    .topology
                    .devices
                    .get_mut("R2")
                    .unwrap()
                    .interfaces
                    .get_mut("branch")
                    .unwrap()
                    .address = Some(Assignment {
                    address: ip("198.51.100.20"),
                    prefix: 24,
                })
            }
            _ => unreachable!(),
        }
        if let Ok(observation) = broken.inspect(&flow(22)) {
            assert!(
                !ar::guarded_denial(&observation, &explicit(at.clone()), &[&good]),
                "mutation {what}"
            );
        }
    }
}

#[test]
fn explicit_endpoint_interface_and_transit_role_restrictions_are_not_silent_approximations() {
    let n = network();
    let f = flow(443);
    for device in ["Client", "Server"] {
        let mut bad = n.clone();
        bad.topology.devices.get_mut(device).unwrap().forwarding = true;
        assert_eq!(bad.inspect(&f).unwrap_err().kind, FailureKind::Unsupported);
    }
    let mut stopped_router = n.clone();
    stopped_router
        .topology
        .devices
        .get_mut("R1")
        .unwrap()
        .forwarding = false;
    let router_source = Flow {
        source: endpoint("R1", "office"),
        ..f.clone()
    };
    let router_target = Flow {
        destination: endpoint("R1", "office"),
        ..f.clone()
    };
    for invalid in [&router_source, &router_target] {
        assert_eq!(
            stopped_router.inspect(invalid).unwrap_err().kind,
            FailureKind::Unsupported
        );
    }
    let mut same = f.clone();
    same.destination = same.source.clone();
    assert_eq!(n.inspect(&same).unwrap_err().kind, FailureKind::Unsupported);
    let mut host_acl = n.clone();
    attach(
        &mut host_acl,
        &location("Client", "lan", Direction::Outbound),
        vec![rule("permit", Action::Permit, Protocol::Ip {})],
    );
    assert_eq!(
        host_acl.inspect(&f).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut multi = n.clone();
    multi
        .topology
        .devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .insert("other".into(), interface("unused", "203.0.113.10", 24));
    let mut wrong = f.clone();
    wrong.source.interface = "other".into();
    assert_eq!(
        multi.inspect(&wrong).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut local = n.clone();
    local.topology.devices.insert(
        "Neighbor".into(),
        device(
            false,
            vec![("lan", interface("office", "192.0.2.20", 24))],
            vec![],
        ),
    );
    local.host_endpoints.insert("Neighbor".into(), "lan".into());
    let mut same_link = f.clone();
    same_link.destination = endpoint("Neighbor", "lan");
    assert_eq!(
        local.inspect(&same_link).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut down = n.clone();
    down.topology
        .devices
        .get_mut("Server")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .up = false;
    assert_eq!(down.inspect(&f).unwrap_err().kind, FailureKind::Routing);
}

#[test]
fn host_declarations_are_bounded_pinned_identity_not_inferred_from_forwarding_flags() {
    let n = network();
    let mut policy_edit = n.clone();
    attach(
        &mut policy_edit,
        &location("R1", "office", Direction::Inbound),
        vec![rule("changed", Action::Permit, Protocol::Ip {})],
    );
    assert!(policy_edit.same_identity(&n));
    let mut role_edit = n.clone();
    role_edit.topology.devices.get_mut("R1").unwrap().forwarding = false;
    role_edit
        .host_endpoints
        .insert("R1".into(), "office".into());
    assert!(role_edit.validate().is_ok());
    assert!(!role_edit.same_identity(&n));
    let mut removed = n.clone();
    removed.host_endpoints.remove("Client");
    assert!(!removed.same_identity(&n));
    assert_eq!(
        removed.inspect(&flow(443)).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut bad = n.clone();
    bad.host_endpoints.insert("Client".into(), "absent".into());
    assert!(bad.validate().is_err());
    assert!(!bad.same_identity(&n));
    let compact = serde_json::to_string(&n).unwrap();
    let duplicate = compact.replace(
        "\"Client\":\"lan\"",
        "\"Client\":\"lan\",\"Client\":\"lan\"",
    );
    assert_ne!(duplicate, compact);
    assert!(Network::from_json(duplicate.as_bytes()).is_err());
    let invalid_value = compact.replace("\"Client\":\"lan\"", "\"Client\":\"../lan\"");
    assert!(Network::from_json(invalid_value.as_bytes()).is_err());
    let mut too_many = n.clone();
    too_many.host_endpoints = (0..13)
        .map(|i| (format!("host{i}"), "lan".into()))
        .collect();
    assert!(too_many.validate().is_err());
    assert!(Network::from_json(&serde_json::to_vec(&too_many).unwrap()).is_err());
}

#[test]
fn none_missing_empty_foreign_and_invalid_direct_configuration_are_distinct_and_atomic() {
    let n = network();
    let before = serde_json::to_vec(&n).unwrap();
    let at = location("R1", "office", Direction::Inbound);
    let mut no_list = n.clone();
    no_list.devices.insert(
        "R1".into(),
        DevicePolicy {
            policy: Policy {
                schema_version: 1,
                lists: BTreeMap::new(),
            },
            attachments: BTreeMap::from([("office".into(), Attachment::default())]),
        },
    );
    assert!(no_list.inspect(&flow(443)).unwrap().delivered());
    let mut missing = no_list.clone();
    missing
        .devices
        .get_mut("R1")
        .unwrap()
        .attachments
        .get_mut("office")
        .unwrap()
        .inbound = Some("not-defined".into());
    assert_eq!(
        missing.inspect(&flow(443)).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut empty = n.clone();
    attach(&mut empty, &at, vec![]);
    assert_eq!(
        empty.inspect(&flow(443)).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut bad = n.clone();
    attach(
        &mut bad,
        &location("R1", "unknown", Direction::Inbound),
        vec![rule("permit", Action::Permit, Protocol::Ip {})],
    );
    assert_eq!(
        bad.inspect(&flow(443)).unwrap_err().kind,
        FailureKind::Invalid
    );
    let mut bad = n.clone();
    attach(
        &mut bad,
        &location("Missing", "office", Direction::Inbound),
        vec![rule("permit", Action::Permit, Protocol::Ip {})],
    );
    assert_eq!(
        bad.inspect(&flow(443)).unwrap_err().kind,
        FailureKind::Invalid
    );
    let mut bad = n.clone();
    bad.schema_version = 2;
    assert_eq!(
        bad.inspect(&flow(443)).unwrap_err().kind,
        FailureKind::Unsupported
    );
    let mut bad_flow = flow(443);
    bad_flow.source.device = "../Client".into();
    assert!(n.inspect(&bad_flow).is_err());
    bad_flow = flow(443);
    bad_flow.protocol = PacketProtocol::Other { number: 6 };
    assert!(n.inspect(&bad_flow).is_err());
    let bad_expected = explicit(location("R1", "unknown", Direction::Inbound));
    assert!(!bad_expected.valid_for(&n));
    assert!(explicit(at).valid_for(&n));
    assert_eq!(serde_json::to_vec(&n).unwrap(), before);
}

#[test]
fn strict_json_rejects_duplicate_bounded_maps_unknown_shapes_and_incomplete_references() {
    let mut n = network();
    let at = location("R1", "office", Direction::Inbound);
    attach(
        &mut n,
        &at,
        vec![rule("permit", Action::Permit, Protocol::Ip {})],
    );
    let compact = serde_json::to_string(&n).unwrap();
    assert_eq!(Network::from_json(compact.as_bytes()).unwrap(), n);
    assert_eq!(
        serde_json::from_value::<Network>(serde_json::to_value(&n).unwrap()).unwrap(),
        n
    );
    let config = serde_json::to_string(&n.devices["R1"]).unwrap();
    let attachment = serde_json::to_string(&n.devices["R1"].attachments["office"]).unwrap();
    for text in [
        compact.replacen(
            &format!("\"R1\":{config}"),
            &format!("\"R1\":{config},\"R1\":{config}"),
            1,
        ),
        compact.replace(
            &format!("\"office\":{attachment}"),
            &format!("\"office\":{attachment},\"office\":{attachment}"),
        ),
        compact.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
        compact.replace("\"inbound\":\"edge\"", "\"inbound\":\"missing\""),
        compact.replace(
            "\"inbound\":\"edge\"",
            "\"inbound\":\"edge\",\"local_traffic\":true",
        ),
        compact.replace(
            "\"inbound\":\"edge\"",
            &format!("\"inbound\":\"{}\"", "a".repeat(40000)),
        ),
    ] {
        assert!(Network::from_json(text.as_bytes()).is_err());
    }
    let f = flow(443);
    let good = serde_json::to_string(&f).unwrap();
    assert_eq!(Flow::from_json(good.as_bytes()).unwrap(), f);
    for text in [
        good.replace("\"tcp\"", "\"icmp\""),
        good.replace("\"source_port\":50000", "\"source_port\":-1"),
        good.replace(
            "\"source_port\":50000",
            "\"source_port\":50000,\"fragment\":true",
        ),
        good.replacen(
            "\"device\":\"Client\"",
            "\"device\":\"Client\",\"device\":\"Server\"",
            1,
        ),
    ] {
        assert!(Flow::from_json(text.as_bytes()).is_err());
    }
    assert_eq!(
        Network::from_json(&vec![b' '; ar::MAX_JSON_BYTES + 1])
            .unwrap_err()
            .kind,
        FailureKind::Limit
    );
    assert_eq!(
        Flow::from_json(&vec![b' '; ar::MAX_FLOW_BYTES + 1])
            .unwrap_err()
            .kind,
        FailureKind::Limit
    );
    assert!(
        serde_json::from_str::<ExpectedCause>("{\"kind\":\"implicit\",\"rule\":\"ignored\"}")
            .is_err()
    );
}

fn maximum() -> Network {
    let mut topology = Topology {
        devices: BTreeMap::new(),
    };
    for i in 0..12 {
        let mut ports = BTreeMap::new();
        let mut routes = Vec::new();
        if i > 0 {
            ports.insert(
                "left".into(),
                interface(&format!("link{}", i - 1), &format!("10.0.{}.2", i - 1), 30),
            );
            routes.push(route("10.0.0.0/30", "left", &format!("10.0.{}.1", i - 1)));
        }
        if i < 11 {
            ports.insert(
                "right".into(),
                interface(&format!("link{i}"), &format!("10.0.{i}.1"), 30),
            );
            routes.push(route("10.0.10.0/30", "right", &format!("10.0.{i}.2")));
        }
        while ports.len() < 8 {
            ports.insert(
                format!("dummy{}", ports.len()),
                Interface {
                    link: format!("unused{i}-{}", ports.len()),
                    up: false,
                    address: None,
                },
            );
        }
        while routes.len() < 32 {
            routes.push(routes[0].clone());
        }
        topology.devices.insert(
            format!("D{i}"),
            Device {
                forwarding: i > 0 && i < 11,
                interfaces: ports,
                routes,
            },
        );
    }
    let mut n = Network {
        schema_version: 1,
        topology,
        host_endpoints: BTreeMap::from([
            ("D0".into(), "right".into()),
            ("D11".into(), "left".into()),
        ]),
        devices: BTreeMap::new(),
    };
    for i in 1..=8 {
        let mut rules: Vec<_> = (0..63)
            .map(|j| Rule {
                id: format!("rule{j:028}"),
                action: Action::Deny,
                selector: Selector::Standard {
                    source: Wildcard::host(ip("203.0.113.99")),
                },
            })
            .collect();
        rules.push(Rule {
            id: "last-permit".into(),
            action: Action::Permit,
            selector: Selector::Standard {
                source: Wildcard::any(),
            },
        });
        n.devices.insert(
            format!("D{i}"),
            DevicePolicy {
                policy: Policy {
                    schema_version: 1,
                    lists: BTreeMap::from([(
                        "edge".into(),
                        AccessList {
                            kind: Kind::Standard,
                            rules,
                        },
                    )]),
                },
                attachments: BTreeMap::from([(
                    "left".into(),
                    Attachment {
                        inbound: Some("edge".into()),
                        outbound: None,
                    },
                )]),
            },
        );
    }
    n
}
#[test]
fn maximum_cardinality_network_has_bounded_events_and_full_shape_json_roundtrip() {
    let n = maximum();
    n.validate().unwrap();
    assert_eq!(n.topology.devices.len(), 12);
    assert_eq!(
        n.topology
            .devices
            .values()
            .map(|d| d.interfaces.len())
            .sum::<usize>(),
        96
    );
    assert_eq!(
        n.topology
            .devices
            .values()
            .map(|d| d.routes.len())
            .sum::<usize>(),
        384
    );
    let f = Flow {
        source: endpoint("D0", "right"),
        destination: endpoint("D11", "left"),
        protocol: PacketProtocol::Icmp {},
    };
    let result = n.inspect(&f).unwrap();
    assert!(result.delivered());
    let Outcome::Delivered { trace, events } = result.actual else {
        panic!()
    };
    assert_eq!(trace.hops.len(), 11);
    assert_eq!(events.len(), 30);
    assert!(events.len() <= ar::MAX_EVENTS);
    for bytes in [
        serde_json::to_vec(&n).unwrap(),
        serde_json::to_vec_pretty(&n).unwrap(),
    ] {
        assert!(bytes.len() < ar::MAX_JSON_BYTES);
        assert_eq!(Network::from_json(&bytes).unwrap(), n);
    }
    let mut excess = n.clone();
    let ninth = excess.devices["D2"].policy.lists["edge"].clone();
    excess
        .devices
        .get_mut("D1")
        .unwrap()
        .policy
        .lists
        .insert("ninth".into(), ninth);
    assert_eq!(excess.validate().unwrap_err().kind, FailureKind::Limit);
    assert!(Network::from_json(&serde_json::to_vec(&excess).unwrap()).is_err());
    let mut invalid = n.clone();
    invalid
        .devices
        .get_mut("D1")
        .unwrap()
        .attachments
        .insert("ninth-port".into(), Attachment::default());
    assert!(invalid.validate().is_err());
}
