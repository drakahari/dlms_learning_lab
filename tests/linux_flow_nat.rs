//! Typed translation/allocation checks, not host NAT or kernel conformance.
use dlms_learning_lab::{
    linux_connections::{self as ct, Endpoint, Transport, Tuple},
    linux_flow_nat::{Ports, Rule, Table, Target, predicate},
};
fn tuple(last: u8, port: u16) -> Tuple {
    Tuple {
        transport: Transport::Tcp,
        source: Endpoint {
            address: format!("10.20.30.{last}").parse().unwrap(),
            port,
        },
        destination: Endpoint {
            address: "203.0.113.9".parse().unwrap(),
            port: 443,
        },
    }
}
fn public() -> std::net::Ipv4Addr {
    "198.51.100.1".parse().unwrap()
}
fn snat(ports: Option<Ports>) -> Rule {
    Rule {
        predicate: predicate(
            "iptables -A OUTPUT -o wan0 -s 10.20.30.0/24 -p tcp -j ACCEPT",
            false,
        )
        .unwrap(),
        target: Target::Snat {
            address: public(),
            ports,
        },
    }
}
#[test]
fn different_clients_with_identical_ports_get_distinct_reversible_mappings() {
    let nat = Table {
        prerouting: vec![],
        postrouting: vec![snat(Some(Ports {
            first: 40000,
            last: 40001,
        }))],
    };
    let mut tracked = ct::Table::default();
    for (last, port) in [(10, 40000), (11, 40001)] {
        let original = tuple(last, 40000);
        let (wire, rule, device) = nat.source(original, "wan0", public(), &tracked).unwrap();
        assert_eq!(wire.source.port, port);
        assert_eq!(rule, Some(1));
        assert_eq!(device, None);
        tracked.confirm(original, original, wire, device).unwrap();
    }
    for e in tracked.entries.clone() {
        let reply = tracked.observe(e.wire.reverse()).unwrap();
        assert_eq!(reply.outgoing, e.original.reverse());
        assert_eq!(reply.class, ct::Class::Established);
    }
}
#[test]
fn exhausted_explicit_range_changes_neither_connections_nor_identifier() {
    let nat = Table {
        prerouting: vec![],
        postrouting: vec![snat(Some(Ports {
            first: 45000,
            last: 45000,
        }))],
    };
    let mut tracked = ct::Table::default();
    let a = tuple(10, 40000);
    let (wire, _, m) = nat.source(a, "wan0", public(), &tracked).unwrap();
    tracked.confirm(a, a, wire, m).unwrap();
    let before = tracked.clone();
    assert!(
        nat.source(tuple(11, 40000), "wan0", public(), &tracked)
            .is_err()
    );
    assert_eq!(tracked, before);
}
#[test]
fn implicit_port_classes_preserve_ports_then_allocate_within_the_same_class() {
    let nat = Table {
        prerouting: vec![],
        postrouting: vec![snat(None)],
    };
    for port in [0, 1, 511, 512, 1023, 1024, 40000, 65535] {
        let mut tracked = ct::Table::default();
        let a = tuple(10, port);
        let (wire, _, m) = nat.source(a, "wan0", public(), &tracked).unwrap();
        assert_eq!(wire.source.port, port);
        tracked.confirm(a, a, wire, m).unwrap();
        let (next, _, _) = nat
            .source(tuple(11, port), "wan0", public(), &tracked)
            .unwrap();
        assert_ne!(next.source.port, port);
        let class = |p: u16| match p {
            0..=511 => 0,
            512..=1023 => 1,
            _ => 2,
        };
        assert_eq!(class(next.source.port), class(port));
    }
}
#[test]
fn first_matching_destination_rule_wins_and_optional_port_is_preserved() {
    let p = predicate(
        "iptables -A INPUT -i wan0 -d 198.51.100.1 -p tcp --dport 8443 -j ACCEPT",
        true,
    )
    .unwrap();
    let mut nat = Table {
        prerouting: vec![
            Rule {
                predicate: p.clone(),
                target: Target::Dnat {
                    address: "10.20.30.42".parse().unwrap(),
                    port: Some(443),
                },
            },
            Rule {
                predicate: p,
                target: Target::Dnat {
                    address: "10.20.30.99".parse().unwrap(),
                    port: Some(9443),
                },
            },
        ],
        postrouting: vec![],
    };
    let mut incoming = tuple(10, 50000);
    incoming.source.address = "203.0.113.55".parse().unwrap();
    incoming.destination = Endpoint {
        address: public(),
        port: 8443,
    };
    let (forward, index) = nat.destination(incoming, "wan0").unwrap();
    assert_eq!(index, Some(1));
    assert_eq!(forward.destination.address.to_string(), "10.20.30.42");
    assert_eq!(forward.destination.port, 443);
    assert_eq!(forward.source, incoming.source);
    if let Target::Dnat { port, .. } = &mut nat.prerouting[0].target {
        *port = None;
    }
    assert_eq!(
        nat.destination(incoming, "wan0")
            .unwrap()
            .0
            .destination
            .port,
        8443
    );
    incoming.destination.port = 8444;
    assert_eq!(nat.destination(incoming, "wan0").unwrap(), (incoming, None));
}
#[test]
fn translation_is_not_filter_permission_and_unmatched_flows_stay_unchanged() {
    let nat = Table {
        prerouting: vec![],
        postrouting: vec![snat(None)],
    };
    let tracked = ct::Table::default();
    let mut p = tuple(10, 40000);
    p.source.address = "192.0.2.55".parse().unwrap();
    assert_eq!(
        nat.source(p, "wan0", public(), &tracked).unwrap(),
        (p, None, None)
    );
    assert!(tracked.entries.is_empty());
    let (wire, _, _) = nat
        .source(tuple(10, 40000), "wan0", public(), &tracked)
        .unwrap();
    assert_eq!(wire.source.address, public());
    assert!(tracked.entries.is_empty()); // Proposal alone is neither a filter decision nor confirmation.
}
#[test]
fn masquerade_uses_outgoing_address_and_tracks_only_its_device_binding() {
    let mut rule = snat(None);
    rule.target = Target::Masquerade { ports: None };
    let nat = Table {
        prerouting: vec![],
        postrouting: vec![rule],
    };
    let mut t = ct::Table::default();
    let a = tuple(10, 40000);
    let address = "198.51.100.77".parse().unwrap();
    let (w, _, d) = nat.source(a, "wan0", address, &t).unwrap();
    assert_eq!(w.source.address, address);
    assert_eq!(d.as_deref(), Some("wan0"));
    t.confirm(a, a, w, d).unwrap();
    assert_eq!(t.forget_masquerade("wan0").unwrap(), 1);
}
#[test]
fn confirmed_flows_retain_translation_after_new_rule_changes() {
    let a = tuple(10, 40000);
    let nat = Table {
        prerouting: vec![],
        postrouting: vec![snat(None)],
    };
    let mut t = ct::Table::default();
    let (w, _, d) = nat.source(a, "wan0", public(), &t).unwrap();
    t.confirm(a, a, w, d).unwrap();
    let changed = Table::default();
    assert_eq!(
        changed
            .source(a, "wan0", public(), &ct::Table::default())
            .unwrap()
            .0,
        a
    );
    assert_eq!(t.preview(a).unwrap().outgoing, w);
    assert_eq!(t.preview(w.reverse()).unwrap().outgoing, a.reverse());
}
#[test]
fn invalid_targets_protocol_options_directions_and_ranges_are_refused() {
    let base = snat(None);
    let mut invalid = vec![Table {
        prerouting: vec![base.clone()],
        postrouting: vec![],
    }];
    let mut r = base.clone();
    r.target = Target::Snat {
        address: public(),
        ports: Some(Ports {
            first: 50,
            last: 49,
        }),
    };
    invalid.push(Table {
        prerouting: vec![],
        postrouting: vec![r],
    });
    let mut r = base.clone();
    r.predicate.ingress = Some("lan0".into());
    invalid.push(Table {
        prerouting: vec![],
        postrouting: vec![r],
    });
    let mut r = base.clone();
    r.predicate.protocol = None;
    r.target = Target::Snat {
        address: public(),
        ports: Some(Ports {
            first: 40000,
            last: 40001,
        }),
    };
    invalid.push(Table {
        prerouting: vec![],
        postrouting: vec![r],
    });
    let mut r = base;
    r.target = Target::Snat {
        address: "127.0.0.1".parse().unwrap(),
        ports: None,
    };
    invalid.push(Table {
        prerouting: vec![],
        postrouting: vec![r],
    });
    for n in invalid {
        assert!(n.validate().is_err());
    }
}
#[test]
fn translation_rules_and_serialized_data_are_bounded() {
    let mut n = Table {
        prerouting: vec![],
        postrouting: vec![snat(None); 32],
    };
    n.validate().unwrap();
    assert_eq!(
        serde_json::from_str::<Table>(&serde_json::to_string(&n).unwrap()).unwrap(),
        n
    );
    n.postrouting.push(snat(None));
    assert!(n.validate().is_err());
    assert!(serde_json::from_str::<Table>(&serde_json::to_string(&n).unwrap()).is_err());
    let mut value = serde_json::to_value(Table::default()).unwrap();
    value["unmodeled"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Table>(value).is_err());
}

#[test]
fn null_source_binding_remaps_a_collision_created_by_an_earlier_translation() {
    let old = tuple(10, 40000);
    let mut occupied = old;
    occupied.source.address = "192.0.2.55".parse().unwrap();
    let mut connections = ct::Table::default();
    connections.confirm(old, old, occupied, None).unwrap();
    let nat = Table::default();
    let (proposed, index, device) = nat
        .source(occupied, "wan0", public(), &connections)
        .unwrap();
    assert_eq!(index, None);
    assert_eq!(device, None);
    assert_eq!(proposed.source.address, occupied.source.address);
    assert_ne!(proposed.source.port, occupied.source.port);
    assert!(proposed.source.port >= 1024);
    let id = connections
        .confirm(occupied, occupied, proposed, None)
        .unwrap();
    assert_eq!(
        connections.preview(proposed.reverse()).unwrap().id,
        Some(id)
    );
    assert_eq!(
        connections.preview(proposed.reverse()).unwrap().outgoing,
        occupied.reverse()
    );
}
