//! Actual composed in-process events: routing, state policy and retained NAT maps.
use dlms_learning_lab::{
    linux_connection_flow::{Gate, State},
    linux_connections::{self as ct, Class, Endpoint, Transport, Tuple},
    linux_filter::{Cidr, Verdict},
    linux_flow_nat::{self as nat, Ports, Rule, Target},
    linux_router_path::{Facts, Link, Route},
    linux_state_filter,
};
fn facts() -> Facts {
    Facts {
        links: vec![
            Link {
                name: "lan0".into(),
                address: "10.20.30.1".parse().unwrap(),
                prefix: 24,
                up: true,
            },
            Link {
                name: "wan0".into(),
                address: "198.51.100.1".parse().unwrap(),
                prefix: 24,
                up: true,
            },
        ],
        routes: vec![Route {
            destination: Cidr::parse("0.0.0.0/0").unwrap(),
            device: "wan0".into(),
            via: Some("198.51.100.254".parse().unwrap()),
            metric: 10,
        }],
    }
}
fn tuple(last: u8) -> Tuple {
    Tuple {
        transport: Transport::Tcp,
        source: Endpoint {
            address: format!("10.20.30.{last}").parse().unwrap(),
            port: 40000,
        },
        destination: Endpoint {
            address: "203.0.113.9".parse().unwrap(),
            port: 443,
        },
    }
}
fn shared() -> State {
    let mut filter = linux_state_filter::Table::new(Verdict::Drop);
    for command in [
        "iptables -A FORWARD -i lan0 -o wan0 -s 10.20.30.0/24 -d 203.0.113.9 -p tcp --dport 443 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT",
        "iptables -A FORWARD -i wan0 -o lan0 -m conntrack --ctstate ESTABLISHED -j ACCEPT",
    ] {
        filter.command(command).unwrap();
    }
    State {
        filter,
        nat: nat::Table {
            prerouting: vec![],
            postrouting: vec![Rule {
                predicate: nat::predicate(
                    "iptables -A OUTPUT -o wan0 -s 10.20.30.0/24 -p tcp -j ACCEPT",
                    false,
                )
                .unwrap(),
                target: Target::Snat {
                    address: "198.51.100.1".parse().unwrap(),
                    ports: Some(Ports {
                        first: 40000,
                        last: 40001,
                    }),
                },
            }],
        },
        connections: ct::Table::default(),
    }
}
#[test]
fn actual_path_keeps_two_clients_separate_through_requests_and_replies() {
    let mut s = shared();
    let f = facts();
    let mut outgoing = vec![];
    for last in [10, 11] {
        let original = tuple(last);
        let trace = s.event(&f, true, "lan0", original).unwrap();
        assert_eq!(trace.gate, Gate::Forwarded);
        assert_eq!(trace.class, Some(Class::New));
        assert_eq!(trace.filter, original);
        let wire = trace.outgoing.unwrap();
        assert_eq!(wire.source.address.to_string(), "198.51.100.1");
        outgoing.push((original, wire));
    }
    assert_ne!(outgoing[0].1.source.port, outgoing[1].1.source.port);
    for (original, wire) in outgoing {
        let reply = s.event(&f, true, "wan0", wire.reverse()).unwrap();
        assert_eq!(reply.gate, Gate::Forwarded);
        assert_eq!(reply.class, Some(Class::Established));
        assert_eq!(reply.filter, original.reverse());
        assert_eq!(reply.outgoing, Some(original.reverse()));
        assert!(reply.retained_mapping);
        assert_eq!(
            s.event(&f, true, "lan0", original).unwrap().class,
            Some(Class::Established)
        );
    }
}
#[test]
fn unsolicited_inbound_and_filter_rejection_never_create_new_mapping() {
    let mut s = shared();
    let f = facts();
    let mut inbound = tuple(10).reverse();
    inbound.source.port = 9999;
    let trace = s.event(&f, true, "wan0", inbound).unwrap();
    assert_eq!(trace.gate, Gate::FilterDrop);
    assert_eq!(trace.class, Some(Class::New));
    assert!(s.connections.entries.is_empty());
    s.filter.command("iptables -I FORWARD 1 -j DROP").unwrap();
    assert_eq!(
        s.event(&f, true, "lan0", tuple(10)).unwrap().gate,
        Gate::FilterDrop
    );
    assert!(s.connections.entries.is_empty());
}
#[test]
fn later_reply_drop_is_distinct_from_not_observing_a_reply() {
    let mut s = shared();
    let f = facts();
    let original = tuple(10);
    let wire = s
        .event(&f, true, "lan0", original)
        .unwrap()
        .outgoing
        .unwrap();
    s.filter
        .command("iptables -I FORWARD 1 -i wan0 -j DROP")
        .unwrap();
    let reply = s.event(&f, true, "wan0", wire.reverse()).unwrap();
    assert_eq!(reply.gate, Gate::FilterDrop);
    assert_eq!(reply.class, Some(Class::Established));
    assert!(s.connections.entries[0].seen_reply);
    assert_eq!(
        s.connections.preview(original).unwrap().class,
        Class::Established
    );
}
#[test]
fn down_ingress_does_not_observe_the_reply_and_invalid_events_are_atomic() {
    let mut s = shared();
    let mut f = facts();
    let original = tuple(10);
    let wire = s
        .event(&f, true, "lan0", original)
        .unwrap()
        .outgoing
        .unwrap();
    let before = s.clone();
    f.links[1].up = false;
    assert_eq!(
        s.event(&f, true, "wan0", wire.reverse()).unwrap().gate,
        Gate::IngressDown
    );
    assert_eq!(s, before);
    assert!(s.event(&facts(), true, "not-a-link", original).is_err());
    assert_eq!(s, before);
    let mut invalid = original;
    invalid.destination.address = "224.0.0.1".parse().unwrap();
    assert!(s.event(&facts(), true, "lan0", invalid).is_err());
    assert_eq!(s, before);
}
#[test]
fn forwarding_routes_and_port_exhaustion_refuse_before_false_success() {
    let mut s = shared();
    let f = facts();
    assert_eq!(
        s.event(&f, false, "lan0", tuple(10)).unwrap().gate,
        Gate::ForwardingDisabled
    );
    assert!(s.connections.entries.is_empty());
    let mut no = f.clone();
    no.routes.clear();
    assert_eq!(
        s.event(&no, true, "lan0", tuple(10)).unwrap().gate,
        Gate::NoRoute
    );
    assert!(s.connections.entries.is_empty());
    for last in [10, 11] {
        assert_eq!(
            s.event(&f, true, "lan0", tuple(last)).unwrap().gate,
            Gate::Forwarded
        );
    }
    let before = s.connections.clone();
    let trace = s.event(&f, true, "lan0", tuple(12)).unwrap();
    assert_eq!(trace.gate, Gate::MappingRefused);
    assert!(trace.detail.contains("no free tuple"));
    assert_eq!(trace.outgoing, None);
    assert_eq!(s.connections, before);
}
fn published() -> State {
    let mut filter = linux_state_filter::Table::new(Verdict::Drop);
    for command in [
        "iptables -A FORWARD -i wan0 -o lan0 -d 10.20.30.42 -p tcp --dport 443 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT",
        "iptables -A FORWARD -i lan0 -o wan0 -m conntrack --ctstate ESTABLISHED -j ACCEPT",
    ] {
        filter.command(command).unwrap();
    }
    State {
        filter,
        nat: nat::Table {
            prerouting: vec![Rule {
                predicate: nat::predicate(
                    "iptables -A INPUT -i wan0 -d 198.51.100.1 -p tcp --dport 8443 -j ACCEPT",
                    true,
                )
                .unwrap(),
                target: Target::Dnat {
                    address: "10.20.30.42".parse().unwrap(),
                    port: Some(443),
                },
            }],
            postrouting: vec![],
        },
        connections: ct::Table::default(),
    }
}
fn incoming() -> Tuple {
    Tuple {
        transport: Transport::Tcp,
        source: Endpoint {
            address: "203.0.113.55".parse().unwrap(),
            port: 50000,
        },
        destination: Endpoint {
            address: "198.51.100.1".parse().unwrap(),
            port: 8443,
        },
    }
}
#[test]
fn destination_translation_precedes_route_and_filter_then_reverses_on_reply() {
    let mut s = published();
    let f = facts();
    let original = incoming();
    let out = s.event(&f, true, "wan0", original).unwrap();
    assert_eq!(out.gate, Gate::Forwarded);
    assert_eq!(out.filter.destination.address.to_string(), "10.20.30.42");
    assert_eq!(out.filter.destination.port, 443);
    assert_eq!(out.route.as_ref().unwrap().device, "lan0");
    let response = s
        .event(&f, true, "lan0", out.outgoing.unwrap().reverse())
        .unwrap();
    assert_eq!(response.gate, Gate::Forwarded);
    assert_eq!(response.outgoing, Some(original.reverse()));
    assert_eq!(response.filter.source.address.to_string(), "10.20.30.42");
    assert_eq!(response.filter.source.port, 443);
}
#[test]
fn filter_matching_public_tuple_is_a_real_mistake_not_a_nat_success() {
    let mut s = published();
    s.filter.command("iptables -F FORWARD").unwrap();
    s.filter
        .command("iptables -A FORWARD -d 198.51.100.1 -p tcp --dport 8443 -j ACCEPT")
        .unwrap();
    let trace = s.event(&facts(), true, "wan0", incoming()).unwrap();
    assert_eq!(trace.gate, Gate::FilterDrop);
    assert_eq!(trace.filter.destination.address.to_string(), "10.20.30.42");
    assert!(s.connections.entries.is_empty());
    let mut missing = published();
    missing.nat.prerouting.clear();
    let trace = missing.event(&facts(), true, "wan0", incoming()).unwrap();
    assert_eq!(trace.gate, Gate::RouterLocal);
    assert!(trace.detail.contains("router-local"));
    assert!(missing.connections.entries.is_empty());
}
#[test]
fn actual_events_reuse_confirmed_mappings_instead_of_reapplying_changed_rules() {
    let mut s = shared();
    let f = facts();
    let original = tuple(10);
    let first = s.event(&f, true, "lan0", original).unwrap();
    s.nat.postrouting.clear();
    let again = s.event(&f, true, "lan0", original).unwrap();
    assert_eq!(again.outgoing, first.outgoing);
    assert!(again.retained_mapping);
    let reply = s
        .event(&f, true, "wan0", first.outgoing.unwrap().reverse())
        .unwrap();
    assert_eq!(reply.outgoing, Some(original.reverse()));
    let fresh = s.event(&f, true, "lan0", tuple(11)).unwrap();
    assert_eq!(fresh.outgoing, Some(tuple(11))); // Header forwarding alone does not claim source translation or application success.
}

#[test]
fn publication_requires_destination_translation_not_direct_private_address_access() {
    let mut s = published();
    s.filter.command("iptables -F FORWARD").unwrap();
    s.filter.command("iptables -A FORWARD -i wan0 -o lan0 -d 10.20.30.42 -p tcp --dport 443 -m conntrack --ctstate DNAT -j ACCEPT").unwrap();
    s.filter
        .command("iptables -A FORWARD -m conntrack --ctstate ESTABLISHED -j ACCEPT")
        .unwrap();
    let mut direct = incoming();
    direct.destination.address = "10.20.30.42".parse().unwrap();
    direct.destination.port = 443;
    let refused = s.event(&facts(), true, "wan0", direct).unwrap();
    assert_eq!(refused.gate, Gate::FilterDrop);
    assert_eq!(refused.dnat, Some(false));
    assert!(s.connections.entries.is_empty());
    let published = s.event(&facts(), true, "wan0", incoming()).unwrap();
    assert_eq!(published.gate, Gate::Forwarded);
    assert_eq!(published.dnat, Some(true));
    let reply = s
        .event(
            &facts(),
            true,
            "lan0",
            published.outgoing.unwrap().reverse(),
        )
        .unwrap();
    assert_eq!(reply.gate, Gate::Forwarded);
    assert_eq!(reply.dnat, Some(true));
    assert_eq!(reply.outgoing, Some(incoming().reverse()));
}
#[test]
fn source_nat_reply_is_established_without_becoming_destination_nat_status() {
    let mut s = shared();
    let original = tuple(10);
    let request = s.event(&facts(), true, "lan0", original).unwrap();
    assert_eq!(request.dnat, Some(false));
    let reply = s
        .event(&facts(), true, "wan0", request.outgoing.unwrap().reverse())
        .unwrap();
    assert_eq!(reply.gate, Gate::Forwarded);
    assert_eq!(reply.class, Some(Class::Established));
    assert_eq!(reply.dnat, Some(false));
}
