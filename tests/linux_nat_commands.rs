//! Real supported literal syntax mutates typed NAT state; no host firewall oracle.
use dlms_learning_lab::{
    linux_filter::Class,
    linux_flow_nat::{Ports, Table, Target},
};
#[test]
fn real_snat_dnat_and_masquerade_forms_create_typed_rules() {
    let mut t = Table::default();
    for command in [
        "iptables -t nat -A POSTROUTING -o wan0 -s 10.20.30.0/24 -p tcp -j SNAT --to-source 198.51.100.1:40000-40001",
        "iptables --table nat --append PREROUTING --in-interface wan0 --destination 198.51.100.1 --protocol tcp --destination-port 8443 --jump DNAT --to-destination 10.20.30.42:443",
        "iptables -t nat -A POSTROUTING -o wan1 -p udp -j MASQUERADE --to-ports 50000",
    ] {
        assert!(t.command(command).unwrap().changed);
    }
    assert_eq!(
        t.prerouting[0].target,
        Target::Dnat {
            address: "10.20.30.42".parse().unwrap(),
            port: Some(443)
        }
    );
    assert_eq!(
        t.postrouting[0].target,
        Target::Snat {
            address: "198.51.100.1".parse().unwrap(),
            ports: Some(Ports {
                first: 40000,
                last: 40001
            })
        }
    );
    assert_eq!(
        t.postrouting[1].target,
        Target::Masquerade {
            ports: Some(Ports {
                first: 50000,
                last: 50000
            })
        }
    );
}
#[test]
fn query_roundtrip_preserves_target_options_and_does_not_apply_changes() {
    let mut t = Table::default();
    for c in [
        "iptables -t nat -A PREROUTING -d 198.51.100.1 -j DNAT --to-destination 10.20.30.42",
        "iptables -t nat -A POSTROUTING -s 10.20.30.0/24 -j SNAT --to-source 198.51.100.1",
        "iptables -t nat -A POSTROUTING -o wan1 -j MASQUERADE",
    ] {
        t.command(c).unwrap();
    }
    let before = t.clone();
    let output = t.command("iptables -t nat -S").unwrap();
    assert!(!output.changed);
    assert_eq!(t, before);
    assert_eq!(
        output.text.lines().filter(|s| s.starts_with("-P ")).count(),
        2
    );
    let mut copy = Table::default();
    for line in output.text.lines() {
        copy.command(&format!("iptables -t nat {line}")).unwrap();
    }
    assert_eq!(copy, t);
    assert!(
        t.command("iptables -t nat -S PREROUTING")
            .unwrap()
            .text
            .lines()
            .all(|s| s.contains("PREROUTING"))
    );
}
#[test]
fn insert_delete_and_selected_flush_have_correct_one_based_positions() {
    let mut t = Table::default();
    t.command("iptables -t nat -A POSTROUTING -o wan0 -j MASQUERADE")
        .unwrap();
    t.command("iptables -t nat -I POSTROUTING 1 -s 10.20.30.0/24 -j SNAT --to-source 198.51.100.1")
        .unwrap();
    assert!(matches!(t.postrouting[0].target, Target::Snat { .. }));
    t.command("iptables -t nat -D POSTROUTING 1").unwrap();
    assert!(matches!(t.postrouting[0].target, Target::Masquerade { .. }));
    t.command("iptables -t nat -A PREROUTING -j DNAT --to-destination 10.20.30.42")
        .unwrap();
    t.command("iptables -t nat -F POSTROUTING").unwrap();
    assert!(t.postrouting.is_empty());
    assert_eq!(t.prerouting.len(), 1);
    t.command("iptables -t nat -F").unwrap();
    assert_eq!(t, Table::default());
}
#[test]
fn malformed_unsupported_and_escape_attempts_leave_the_table_exact() {
    let mut t = Table::default();
    t.command("iptables -t nat -A POSTROUTING -o wan0 -j MASQUERADE")
        .unwrap();
    let before = t.clone();
    for c in [
        "iptables -A POSTROUTING -j MASQUERADE",
        "iptables -t nat -t nat -F",
        "iptables -t filter -F",
        "iptables -t nat -A OUTPUT -j DNAT --to-destination 10.20.30.42",
        "iptables -t nat -P POSTROUTING DROP",
        "iptables -t nat -A POSTROUTING -j DNAT --to-destination 10.20.30.42",
        "iptables -t nat -A PREROUTING -j SNAT --to-source 198.51.100.1",
        "iptables -t nat -A POSTROUTING -j SNAT --to-source 198.51.100.1:40001-40000",
        "iptables -t nat -A POSTROUTING -j SNAT --to-source 198.51.100.1:40000",
        "iptables -t nat -A POSTROUTING -p tcp -j SNAT --to-source 198.51.100.1:65536",
        "iptables -t nat -A POSTROUTING -j SNAT --to-source example.org",
        "iptables -t nat -A POSTROUTING -j SNAT --to-source 198.51.100.1-198.51.100.2",
        "iptables -t nat -A POSTROUTING -j MASQUERADE --random",
        "iptables -t nat -A POSTROUTING -j NETMAP --to-source 198.51.100.1",
        "iptables -t nat -A POSTROUTING -j MASQUERADE; echo escaped",
        "iptables -t nat -A POSTROUTING -j SNAT --to-source '$(touch private)'",
        "iptables -t nat -A POSTROUTING -o 'wan0 -j MASQUERADE' -j MASQUERADE",
        "iptables -t nat -A POSTROUTING -m conntrack --ctstate NEW -j MASQUERADE",
        "iptables -t nat -D POSTROUTING 0",
        "iptables -t nat -D POSTROUTING 9999999999999999999999999999999999999",
        "iptables -t nat -I POSTROUTING 3 -j MASQUERADE",
        "iptables -t nat -A POSTROUTING -j MASQUERADE --to-source 198.51.100.1",
        "iptables -t nat -A POSTROUTING --to-source 198.51.100.1 -j SNAT",
        "iptables -t nat -A POSTROUTING -j SNAT --to-source 198.51.100.1 --to-source 198.51.100.2",
    ] {
        assert!(t.command(c).is_err(), "{c}");
        assert_eq!(t, before, "{c}");
    }
}
#[test]
fn match_values_are_not_reinterpreted_as_target_options() {
    let mut t = Table::default();
    t.command("iptables -t nat -A POSTROUTING -o '-j' -j MASQUERADE")
        .unwrap();
    assert_eq!(t.postrouting[0].predicate.egress.as_deref(), Some("-j"));
    let before = t.clone();
    assert!(
        t.command(
            "iptables -t nat -A POSTROUTING -o 'wan0 --to-source 198.51.100.77' -j MASQUERADE"
        )
        .is_err()
    );
    assert_eq!(t, before);
}
#[test]
fn input_rule_and_listing_limits_remain_bounded() {
    let mut t = Table::default();
    for _ in 0..32 {
        t.command("iptables -t nat -A POSTROUTING -j MASQUERADE")
            .unwrap();
    }
    let before = t.clone();
    assert_eq!(
        t.command("iptables -t nat -A POSTROUTING -j MASQUERADE")
            .unwrap_err()
            .class,
        Class::Limit
    );
    assert_eq!(t, before);
    assert_eq!(
        t.command(&"x".repeat(2049)).unwrap_err().class,
        Class::Limit
    );
    assert_eq!(t, before);
    assert!(t.command("iptables -t nat -S").unwrap().text.len() < 32768);
}
#[test]
fn commands_drive_real_collision_and_reply_events_not_answer_matching() {
    use dlms_learning_lab::{
        linux_connection_flow::{self as flow, Gate},
        linux_connections::{self as ct, Endpoint, Transport, Tuple},
        linux_filter::{Cidr, Verdict},
        linux_router_path::{Facts, Link, Route},
        linux_state_filter,
    };
    let facts = Facts {
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
    };
    for action in [
        "-j SNAT --to-source 198.51.100.1:40000-40001",
        "-j MASQUERADE",
    ] {
        let mut state = flow::State {
            filter: linux_state_filter::Table::new(Verdict::Drop),
            nat: Table::default(),
            connections: ct::Table::default(),
        };
        for command in [
            "iptables -A FORWARD -i lan0 -o wan0 -p tcp --dport 443 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT",
            "iptables -A FORWARD -i wan0 -o lan0 -m conntrack --ctstate ESTABLISHED -j ACCEPT",
        ] {
            state.filter.command(command).unwrap();
        }
        state
            .nat
            .command(&format!(
                "iptables -t nat -A POSTROUTING -s 10.20.30.0/24 -o wan0 -p tcp {action}"
            ))
            .unwrap();
        let mut ports = vec![];
        for last in [10, 11] {
            let original = Tuple {
                transport: Transport::Tcp,
                source: Endpoint {
                    address: format!("10.20.30.{last}").parse().unwrap(),
                    port: 40000,
                },
                destination: Endpoint {
                    address: "203.0.113.9".parse().unwrap(),
                    port: 443,
                },
            };
            let trace = state.event(&facts, true, "lan0", original).unwrap();
            assert_eq!(trace.gate, Gate::Forwarded);
            let wire = trace.outgoing.unwrap();
            assert_eq!(wire.source.address.to_string(), "198.51.100.1");
            ports.push(wire.source.port);
            assert_eq!(
                state
                    .event(&facts, true, "wan0", wire.reverse())
                    .unwrap()
                    .outgoing,
                Some(original.reverse())
            );
        }
        assert_ne!(ports[0], ports[1]);
    }
}
