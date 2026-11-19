use dlms_learning_lab::{
    acl_routing as routing,
    acl_routing::Network,
    ipv4_routing::{Assignment, Device, Interface, Prefix, Route, Topology},
    transport_core as core, transport_workspace as work,
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

fn task() -> work::Task {
    let network = network();
    let transport = core::Config {
        version: 1,
        hosts: vec![
            core::Host {
                id: "Client".into(),
                address: ip("192.0.2.10"),
            },
            core::Host {
                id: "Server".into(),
                address: ip("198.51.100.20"),
            },
        ],
        listeners: vec![
            core::Listener {
                protocol: core::Protocol::Tcp,
                endpoint: core::Endpoint {
                    host: "Server".into(),
                    port: 8443,
                },
            },
            core::Listener {
                protocol: core::Protocol::Udp,
                endpoint: core::Endpoint {
                    host: "Server".into(),
                    port: 9555,
                },
            },
        ],
        exchanges: vec![
            core::Exchange {
                id: "report".into(),
                protocol: core::Protocol::Tcp,
                client: core::Endpoint {
                    host: "Client".into(),
                    port: 50123,
                },
                server: core::Endpoint {
                    host: "Server".into(),
                    port: 8000,
                },
                payload: "report: café\n".into(),
            },
            core::Exchange {
                id: "sensor".into(),
                protocol: core::Protocol::Udp,
                client: core::Endpoint {
                    host: "Client".into(),
                    port: 51234,
                },
                server: core::Endpoint {
                    host: "Server".into(),
                    port: 9555,
                },
                payload: "temperature=21\n".into(),
            },
        ],
    };
    work::Task {
        transport,
        echoes: vec![],
        network,
        editable_listeners: vec![0],
        editable_exchanges: vec!["report".into()],
        editable_routers: vec!["R1".into(), "R2".into()],
        requirements: vec![
            work::Requirement::Receive {
                exchange: "report".into(),
                protocol: core::Protocol::Tcp,
                server_port: None,
            },
            work::Requirement::Receive {
                exchange: "sensor".into(),
                protocol: core::Protocol::Udp,
                server_port: Some(9555),
            },
        ],
    }
}
fn solved(t: &work::Task, d: &work::Draft) -> bool {
    t.checks(d).iter().all(|r| r.0)
}
fn repaired(t: &work::Task) -> work::Draft {
    let mut d = work::Draft::new(t).unwrap();
    d.exchanges[0].server_port = d.listeners[0].port.clone();
    d
}
#[test]
fn actual_route_adapter_runs_every_handshake_leg_and_preserves_payload() {
    let t = task();
    t.validate().unwrap();
    let d = work::Draft::new(&t).unwrap();
    assert!(!solved(&t, &d));
    assert!(t.checks(&d)[0].1.contains("no listener"));
    let d = repaired(&t);
    let r = work::reports(&t, &d).unwrap();
    assert!(solved(&t, &d));
    assert_eq!(r[0].trace.len(), 4);
    assert_eq!(r[1].trace.len(), 1);
    assert_eq!(r[0].received.as_deref(), Some("report: café\n"));
    assert_eq!(r[0].trace[1].packet.destination.port, 50123);
    assert_eq!(r[0].trace[1].packet.source.port, 8443);
}
#[test]
fn alternative_matching_port_and_client_port_are_real_settings_not_answer_strings() {
    let t = task();
    for p in [1024, 8001, 65535] {
        let mut d = repaired(&t);
        d.listeners[0].port = p.to_string();
        d.exchanges[0].server_port = p.to_string();
        d.exchanges[0].client_port = "60000".into();
        assert!(solved(&t, &d), "{:?}", t.checks(&d));
        let r = work::reports(&t, &d).unwrap();
        assert_eq!(r[0].server.port, p);
        assert_eq!(r[0].client.port, 60000);
    }
    let mut d = repaired(&t);
    d.listeners[0].protocol = core::Protocol::Udp;
    d.exchanges[0].protocol = core::Protocol::Udp;
    assert!(!solved(&t, &d));
    assert!(t.checks(&d)[0].1.contains("Choose TCP"));
}
#[test]
fn real_reverse_acl_can_block_syn_ack_and_specific_repair_keeps_independent_service() {
    let mut t = task();
    t.transport.exchanges[0].server.port = 8443;
    let mut d = repaired(&t);
    d.documents.routers.insert("R2".into(),"ip access-list extended RETURN\n 10 permit udp any eq 9555 any\n 20 deny tcp any any\nexit\ninterface transit\n ip access-group RETURN out\nexit\n".into());
    let r = work::reports(&t, &d).unwrap();
    assert_eq!(
        r[0].outcome,
        core::Outcome::PacketDropped {
            kind: core::PacketKind::SynAck
        }
    );
    assert_eq!(r[0].trace.len(), 2);
    assert_eq!(r[1].outcome, core::Outcome::Delivered);
    // Source-port service match is the reverse packet, not forward destination-port matching.
    let bad = d.documents.routers["R2"].replace(
        "20 deny tcp any any",
        "15 permit tcp any any eq 8443\n 20 deny tcp any any",
    );
    d.documents.routers.insert("R2".into(), bad);
    assert!(!solved(&t, &d));
    let good = d.documents.routers["R2"].replace(
        "15 permit tcp any any eq 8443",
        "15 permit tcp host 198.51.100.20 eq 8443 host 192.0.2.10 range 49152 65535",
    );
    d.documents.routers.insert("R2".into(), good);
    assert!(solved(&t, &d), "{:?}", t.checks(&d));
}
#[test]
fn unchecked_changed_and_serialized_settings_never_reuse_stale_success() {
    let t = task();
    let mut s = work::Scope::new(&t).unwrap();
    s.edit(&t, repaired(&t)).unwrap();
    assert!(!s.checks(&t).iter().all(|r| r.0));
    assert!(s.check(&t).unwrap().iter().all(|r| r.0));
    let saved = serde_json::to_vec(&s).unwrap();
    let mut reloaded: work::Scope = serde_json::from_slice(&saved).unwrap();
    reloaded.bound(&t).unwrap();
    assert!(reloaded.checks(&t).iter().all(|r| r.0));
    let mut d = reloaded.draft.clone();
    d.exchanges[0].server_port = "8444".into();
    reloaded.edit(&t, d).unwrap();
    assert!(reloaded.tested.is_none());
    assert!(!reloaded.check(&t).unwrap().iter().all(|r| r.0));
    s.draft.exchanges[0].server_port = "bad".into();
    assert!(s.bound(&t).is_err());
    assert!(!s.checks(&t).iter().all(|r| r.0));
}
#[test]
fn malformed_ports_rosters_bindings_and_protected_changes_are_diagnostic() {
    let t = task();
    for p in ["", "-1", "1.5", "65536", "0", " 80"] {
        let mut d = repaired(&t);
        d.exchanges[0].server_port = p.into();
        assert!(!solved(&t, &d));
        assert!(t.checks(&d)[0].1.contains("port"));
    }
    let mut d = repaired(&t);
    d.exchanges[1].server_port = "53".into();
    assert!(d.bound(&t).is_err());
    let mut d = repaired(&t);
    d.exchanges[0].id = "invented".into();
    assert!(d.bound(&t).is_err());
    let mut wrong = t.clone();
    wrong.transport.hosts[0].address = ip("203.0.113.99");
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.requirements[0] = work::Requirement::Receive {
        exchange: "missing".into(),
        protocol: core::Protocol::Tcp,
        server_port: None,
    };
    assert!(wrong.validate().is_err());
    assert!(!wrong.checks(&repaired(&t)).iter().all(|r| r.0));
    let mut d = repaired(&t);
    d.documents
        .routers
        .insert("R2".into(), "run host command".into());
    assert!(work::reports(&t, &d).is_err());
}
#[test]
fn blocked_requirement_needs_actual_boundary_and_positive_crossing_not_no_listener() {
    let mut t = task();
    t.transport.exchanges[0].server.port = 8443;
    let at = routing::Location {
        device: "R1".into(),
        interface: "office".into(),
        direction: routing::Direction::Inbound,
    };
    t.requirements[1] = work::Requirement::Block {
        exchange: "sensor".into(),
        at,
    };
    let mut d = repaired(&t);
    assert!(!solved(&t, &d));
    d.documents.routers.insert("R1".into(),"ip access-list extended EDGE\n 10 deny udp any any eq 9555\n 20 permit ip any any\nexit\ninterface office\n ip access-group EDGE in\nexit\n".into());
    assert!(solved(&t, &d), "{:?}", t.checks(&d));
    d.documents.routers.get_mut("R1").unwrap().replace_range(..,"ip access-list extended EDGE\n 10 deny ip any any\nexit\ninterface office\n ip access-group EDGE in\nexit\n");
    assert!(!solved(&t, &d));
    assert!(t.checks(&d).iter().any(|r| !r.0));
}
#[test]
fn independent_practice_and_source_binding_survive_encoding() {
    let assignment = task();
    let mut example = task();
    example.transport.hosts[0].address = ip("192.0.2.11");
    example
        .network
        .topology
        .devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = ip("192.0.2.11");
    let d = work::Definition {
        version: 1,
        source_id: "transport-test".into(),
        worked_assignment: repaired(&assignment),
        worked_example: repaired(&example),
        assignment,
        example,
    };
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    let original = s.assignment.clone();
    s.practice
        .edit(&d.example, d.worked_example.clone())
        .unwrap();
    assert!(s.practice.check(&d.example).unwrap().iter().all(|r| r.0));
    assert_eq!(s.assignment, original);
    assert!(!s.assignment.checks(&d.assignment).iter().all(|r| r.0));
    let decoded: work::Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&decoded).unwrap();
    assert_eq!(s, decoded);
    let mut foreign = decoded;
    foreign.origin = "other".into();
    assert!(d.bound(&foreign).is_err());
}

#[test]
fn an_unwanted_delivered_probe_is_not_evidence_of_a_working_required_service() {
    let mut t = task();
    let mut extra = t.transport.exchanges[1].clone();
    extra.id = "unwanted".into();
    extra.client.port = 60000;
    t.transport.exchanges.push(extra);
    let at = routing::Location {
        device: "R1".into(),
        interface: "office".into(),
        direction: routing::Direction::Inbound,
    };
    t.requirements[1] = work::Requirement::Block {
        exchange: "sensor".into(),
        at: at.clone(),
    };
    t.requirements.push(work::Requirement::Block {
        exchange: "unwanted".into(),
        at,
    });
    let mut d = work::Draft::new(&t).unwrap();
    d.documents.routers.insert("R1".into(),"ip access-list extended EDGE\n 10 permit udp any eq 60000 any eq 9555\n 20 deny udp any any eq 9555\n 30 permit ip any any\nexit\ninterface office\n ip access-group EDGE in\nexit\n".into());
    let reports = work::reports(&t, &d).unwrap();
    assert_eq!(reports[0].outcome, core::Outcome::NoListener);
    assert_eq!(reports[2].outcome, core::Outcome::Delivered);
    assert!(
        !t.checks(&d)[1].0,
        "An unwanted probe cannot substitute for a positive service requirement"
    );
    d.exchanges[0].server_port = "8443".into();
    let checks = t.checks(&d);
    assert!(checks[0].0 && checks[1].0);
    assert!(!checks[2].0);
    let text = d.documents.routers.get_mut("R1").unwrap();
    *text = text.replace(" 10 permit udp any eq 60000 any eq 9555\n", "");
    assert!(solved(&t, &d));
}

fn echo_task() -> work::Task {
    let mut t = task();
    t.echoes.push(core::echo::Query {
        id: "reachability".into(),
        source: "Client".into(),
        destination: "Server".into(),
        identifier: 901,
        sequence: 7,
        payload: vec![0, 255, 10, 82],
    });
    t.requirements.push(work::Requirement::EchoReceive {
        query: "reachability".into(),
    });
    t
}
#[test]
fn optional_echoes_are_absent_from_all_existing_source_values_and_saved_drafts() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/transport-lessons-v1.json")).unwrap();
    for s in raw["scenarios"].as_array().unwrap() {
        for name in ["assignment", "example"] {
            let before = &s["goal"]["definition"][name];
            let task: work::Task = serde_json::from_value(before.clone()).unwrap();
            assert!(task.echoes.is_empty());
            task.validate().unwrap();
            assert_eq!(serde_json::to_value(&task).unwrap(), *before);
            let draft = work::Draft::new(&task).unwrap();
            assert!(work::echo::reports(&task, &draft).unwrap().is_empty());
        }
    }
}
#[test]
fn echo_round_trip_is_independent_of_a_missing_tcp_listener_and_has_no_service_ports() {
    let t = echo_task();
    t.validate().unwrap();
    let d = work::Draft::new(&t).unwrap();
    let reports = work::echo::reports(&t, &d).unwrap();
    assert!(reports[0].round_trip());
    assert_eq!(reports[0].events.len(), 2);
    let rows = t.checks(&d);
    assert!(!rows[0].0 && rows[0].1.contains("no listener"));
    assert!(rows[2].0 && rows[2].1.contains("does not prove"));
    assert!(!solved(&t, &d));
}
#[test]
fn actual_reverse_icmp_filter_explains_why_request_receipt_does_not_complete_echo() {
    let t = echo_task();
    let mut d = repaired(&t);
    d.documents.routers.insert("R2".into(),"ip access-list extended RETURN\n 10 permit tcp any any\n 20 deny icmp any any\n 30 permit ip any any\nexit\ninterface transit\n ip access-group RETURN out\nexit\n".into());
    let r = work::echo::reports(&t, &d).unwrap();
    assert_eq!(r[0].phase, core::echo::Phase::ReplyDropped);
    assert_eq!(
        r[0].events[0].decision.disposition,
        core::Disposition::Delivered
    );
    assert_eq!(r[0].events[1].packet.destination, ip("192.0.2.10"));
    let rows = t.checks(&d);
    assert!(rows[0].0 && rows[1].0);
    assert!(!rows[2].0);
    assert!(rows[2].1.contains("reply was filtered on its return path"));
    d.documents.routers.get_mut("R2").unwrap().replace_range(..,"ip access-list extended RETURN\n 10 permit ip any any\nexit\ninterface transit\n ip access-group RETURN out\nexit\n");
    assert!(solved(&t, &d), "{:?}", t.checks(&d));
}
#[test]
fn alternate_policy_repairs_allow_echo_and_tcp_but_permit_all_or_disable_all_fail() {
    let mut t = echo_task();
    t.requirements[1] = work::Requirement::Block {
        exchange: "sensor".into(),
        at: routing::Location {
            device: "R1".into(),
            interface: "office".into(),
            direction: routing::Direction::Inbound,
        },
    };
    t.validate().unwrap();
    for rules in [
        "10 deny udp any any eq 9555\n 20 permit ip any any",
        "10 permit tcp any any\n 20 permit icmp any any\n 30 deny ip any any",
    ] {
        let mut d = repaired(&t);
        d.documents.routers.insert("R1".into(),format!("ip access-list extended EDGE\n {rules}\nexit\ninterface office\n ip access-group EDGE in\nexit\n"));
        assert!(solved(&t, &d), "{:?}", t.checks(&d));
    }
    for rules in [
        "10 permit ip any any",
        "10 deny ip any any",
        "10 permit icmp any any\n 20 deny ip any any",
    ] {
        let mut d = repaired(&t);
        d.documents.routers.insert("R1".into(),format!("ip access-list extended EDGE\n {rules}\nexit\ninterface office\n ip access-group EDGE in\nexit\n"));
        assert!(!solved(&t, &d), "{:?}", t.checks(&d));
    }
}
#[test]
fn echo_state_is_recomputed_after_encoding_and_broken_policy_cannot_reuse_success() {
    let t = echo_task();
    let mut s = work::Scope::new(&t).unwrap();
    s.edit(&t, repaired(&t)).unwrap();
    assert!(s.check(&t).unwrap().iter().all(|r| r.0));
    let mut s: work::Scope = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    s.bound(&t).unwrap();
    assert!(s.checks(&t).iter().all(|r| r.0));
    let mut bad = s.draft.clone();
    bad.documents.routers.insert("R1".into(),"ip access-list extended NOECHO\n 10 deny icmp any any\n 20 permit ip any any\nexit\ninterface office\n ip access-group NOECHO in\nexit\n".into());
    s.edit(&t, bad).unwrap();
    assert!(s.tested.is_none());
    let rows = s.check(&t).unwrap();
    assert!(rows[0].0 && rows[1].0);
    assert!(!rows[2].0);
    assert!(rows[2].1.contains("request was filtered"));
    // The checked snapshot is not success authority; all outcomes are recomputed.
    s.tested = Some(s.draft.clone());
    assert!(!s.checks(&t).iter().all(|r| r.0));
}
#[test]
fn echo_source_ids_bounds_and_requirement_kinds_are_strict() {
    let t = echo_task();
    let mut bad = t.clone();
    bad.echoes[0].id = "report".into();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.requirements[2] = work::Requirement::EchoReceive {
        query: "sensor".into(),
    };
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.requirements.pop();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.echoes[0].source = "missing".into();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.echoes[0].payload.resize(core::echo::MAX_PAYLOAD + 1, 0);
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(&t).unwrap();
    value["echoes"][0]["port"] = serde_json::json!(8);
    assert!(serde_json::from_value::<work::Task>(value).is_err());
}
