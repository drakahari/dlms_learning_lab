//! State-aware behavior and literal parsing, not a host firewall oracle.
use dlms_learning_lab::{
    linux_connections::Class as State,
    linux_filter::{Chain, Cidr, Class, Packet, Protocol, Rule, Verdict},
    linux_state_filter::{Match, Table},
};
fn packet(reverse: bool) -> Packet {
    let mut p = Packet {
        source: "10.20.30.55".parse().unwrap(),
        destination: "203.0.113.9".parse().unwrap(),
        ingress: Some("lan0".into()),
        egress: Some("wan0".into()),
        protocol: Protocol::Tcp,
        source_port: Some(40000),
        destination_port: Some(443),
    };
    if reverse {
        std::mem::swap(&mut p.source, &mut p.destination);
        std::mem::swap(&mut p.source_port, &mut p.destination_port);
        std::mem::swap(&mut p.ingress, &mut p.egress);
    }
    p
}
#[test]
fn new_outgoing_and_established_replies_do_not_admit_unsolicited_inbound() {
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -i lan0 -o wan0 -p tcp --dport 443 -m conntrack --ctstate NEW -j ACCEPT").unwrap();
    t.command("iptables -A FORWARD -i wan0 -o lan0 -m conntrack --ctstate ESTABLISHED -j ACCEPT")
        .unwrap();
    for s in [State::New, State::Established] {
        for reverse in [false, true] {
            let d = t.evaluate(Chain::Forward, &packet(reverse), s).unwrap();
            assert_eq!(
                d.verdict,
                if matches!(
                    (s, reverse),
                    (State::New, false) | (State::Established, true)
                ) {
                    Verdict::Accept
                } else {
                    Verdict::Drop
                }
            );
        }
    }
}
#[test]
fn actual_rule_numbers_survive_state_specialization_and_insert_delete() {
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -m conntrack --ctstate NEW -j DROP")
        .unwrap();
    t.command("iptables -A FORWARD -m conntrack --ctstate ESTABLISHED -j ACCEPT")
        .unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &packet(false), State::Established)
            .unwrap()
            .rule,
        Some(2)
    );
    t.command("iptables -I FORWARD 2 -m conntrack --ctstate ESTABLISHED -j DROP")
        .unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &packet(false), State::Established)
            .unwrap()
            .verdict,
        Verdict::Drop
    );
    t.command("iptables -D FORWARD 2").unwrap();
    assert_eq!(t.forward, vec![Some(Match::New), Some(Match::Established)]);
    t.command("iptables -I FORWARD -j ACCEPT").unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &packet(false), State::Established)
            .unwrap()
            .rule,
        Some(1)
    );
    t.command("iptables -F FORWARD").unwrap();
    assert!(t.forward.is_empty());
}
#[test]
fn unsupported_states_modules_bad_numbers_and_malformed_commands_are_atomic() {
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -m conntrack --ctstate ESTABLISHED -j ACCEPT")
        .unwrap();
    let before = t.clone();
    for line in [
        "iptables -A FORWARD -m conntrack --ctstate RELATED -j ACCEPT",
        "iptables -A FORWARD -m conntrack --ctstate NEW,INVALID -j ACCEPT",
        "iptables -A FORWARD --ctstate NEW -j ACCEPT",
        "iptables -A FORWARD --ctstate NEW -m conntrack -j ACCEPT",
        "iptables -A FORWARD -m conntrack -j ACCEPT",
        "iptables -A FORWARD -m conntrack -m conntrack --ctstate NEW -j ACCEPT",
        "iptables -A FORWARD -m conntrack --ctstate NEW --ctstate NEW -j ACCEPT",
        "iptables -A FORWARD -m state --state NEW -j ACCEPT",
        "iptables -I FORWARD 0 -m conntrack --ctstate NEW -j ACCEPT",
        "iptables -D FORWARD 999",
        "iptables -A FORWARD -m conntrack --ctstate NEW -j ACCEPT; echo escaped",
        "iptables -t nat -A FORWARD -j ACCEPT",
        "iptables -A FORWARD -i 'lan0 -j ACCEPT' -m conntrack --ctstate NEW -j ACCEPT",
    ] {
        assert!(t.command(line).is_err(), "{line}");
        assert_eq!(t, before, "{line}");
    }
    assert_eq!(
        t.command("iptables -A FORWARD -m conntrack --ctstate RELATED -j ACCEPT")
            .unwrap_err()
            .class,
        Class::Unsupported
    );
}
#[test]
fn listed_rules_reparse_without_losing_state_and_query_is_read_only() {
    let mut t = Table::new(Verdict::Drop);
    for line in [
        "iptables -P OUTPUT ACCEPT",
        "iptables -A INPUT -i lan0 -m conntrack --ctstate ESTABLISHED,NEW -j ACCEPT",
        "iptables -A FORWARD -p tcp --dport 443 -m conntrack --ctstate NEW -j ACCEPT",
        "iptables -A FORWARD -j DROP",
    ] {
        t.command(line).unwrap();
    }
    let before = t.clone();
    let listing = t.command("iptables -t filter -S").unwrap();
    assert!(!listing.changed);
    assert_eq!(t, before);
    assert!(listing.text.contains("--ctstate NEW,ESTABLISHED"));
    let lines = listing.text.lines().collect::<Vec<_>>();
    assert!(lines[..3].iter().all(|line| line.starts_with("-P ")));
    let mut copy = Table::new(Verdict::Accept);
    for line in lines {
        copy.command(&format!("iptables {line}")).unwrap();
    }
    assert_eq!(copy, t);
}
#[test]
fn finite_specialization_reuses_the_exact_numeric_policy_comparer() {
    use dlms_learning_lab::{
        linux_filter_policy::compare_filter,
        linux_router_path::{Facts, Link},
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
        routes: vec![],
    };
    let domain = Rule {
        source: Some(Cidr::parse("10.20.30.0/24").unwrap()),
        destination: Some(Cidr::parse("203.0.113.9/32").unwrap()),
        ingress: Some("lan0".into()),
        egress: Some("wan0".into()),
        protocol: Some(Protocol::Tcp),
        source_port: None,
        destination_port: Some(443),
        target: Verdict::Accept,
    };
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -i lan0 -o wan0 -s 10.20.30.0/24 -d 203.0.113.9 -p tcp --dport 443 -m conntrack --ctstate NEW -j ACCEPT").unwrap();
    for s in [State::New, State::Established] {
        let allowed = if s == State::New {
            vec![domain.clone()]
        } else {
            vec![]
        };
        assert!(
            compare_filter(
                &t.specialize(s).unwrap(),
                Chain::Forward,
                &facts,
                std::slice::from_ref(&domain),
                &allowed
            )
            .unwrap()
            .is_none()
        );
    }
    // Replacing a subnet with only the sample client must fail the whole-domain proof.
    t.command("iptables -F FORWARD").unwrap();
    t.command("iptables -A FORWARD -s 10.20.30.55 -p tcp --dport 443 -m conntrack --ctstate NEW -j ACCEPT").unwrap();
    assert!(
        compare_filter(
            &t.specialize(State::New).unwrap(),
            Chain::Forward,
            &facts,
            std::slice::from_ref(&domain),
            std::slice::from_ref(&domain)
        )
        .unwrap()
        .is_some()
    );
}
#[test]
fn independent_small_enumeration_checks_rule_precedence_for_each_state() {
    for first in ["NEW", "ESTABLISHED", "NEW,ESTABLISHED"] {
        for second in ["NEW", "ESTABLISHED", "NEW,ESTABLISHED"] {
            let mut t = Table::new(Verdict::Drop);
            t.command(&format!(
                "iptables -A FORWARD -m conntrack --ctstate {first} -j DROP"
            ))
            .unwrap();
            t.command(&format!(
                "iptables -A FORWARD -p tcp --dport 443 -m conntrack --ctstate {second} -j ACCEPT"
            ))
            .unwrap();
            for state in [State::New, State::Established] {
                for port in [80, 443, 444] {
                    let name = if state == State::New {
                        "NEW"
                    } else {
                        "ESTABLISHED"
                    };
                    let accepts = |s: &str| s.split(',').any(|v| v == name);
                    let mut p = packet(false);
                    p.destination_port = Some(port);
                    let expected = if !accepts(first) && accepts(second) && port == 443 {
                        Verdict::Accept
                    } else {
                        Verdict::Drop
                    };
                    assert_eq!(
                        t.evaluate(Chain::Forward, &p, state).unwrap().verdict,
                        expected,
                        "{first}/{second}/{name}/{port}"
                    );
                }
            }
        }
    }
}
#[test]
fn state_annotations_and_rule_storage_are_bounded_and_not_detachable() {
    let mut t = Table::new(Verdict::Drop);
    for _ in 0..96 {
        t.command("iptables -A FORWARD -m conntrack --ctstate NEW -j ACCEPT")
            .unwrap();
    }
    let before = t.clone();
    assert!(t.command("iptables -A FORWARD -j ACCEPT").is_err());
    assert_eq!(t, before);
    let mut value = serde_json::to_value(&t).unwrap();
    value["forward"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!("New"));
    assert!(serde_json::from_value::<Table>(value).is_err());
    let mut missing = t.clone();
    missing.forward.pop();
    assert!(missing.validate().is_err());
    assert_eq!(
        serde_json::from_str::<Table>(&serde_json::to_string(&t).unwrap()).unwrap(),
        t
    );
}

#[test]
fn destination_translation_flag_is_independent_and_comma_lists_mean_or() {
    for names in [
        "NEW",
        "ESTABLISHED",
        "DNAT",
        "NEW,DNAT",
        "ESTABLISHED,DNAT",
        "NEW,ESTABLISHED,DNAT",
        "DNAT,NEW",
        "DNAT,DNAT",
    ] {
        let mut t = Table::new(Verdict::Drop);
        t.command(&format!(
            "iptables -A FORWARD -m conntrack --ctstate {names} -j ACCEPT"
        ))
        .unwrap();
        for class in [State::New, State::Established] {
            for dnat in [false, true] {
                let state = if class == State::New {
                    "NEW"
                } else {
                    "ESTABLISHED"
                };
                let expected = names
                    .split(',')
                    .any(|s| s == state || (s == "DNAT" && dnat));
                assert_eq!(
                    t.evaluate_context(Chain::Forward, &packet(false), class, dnat)
                        .unwrap()
                        .verdict,
                    if expected {
                        Verdict::Accept
                    } else {
                        Verdict::Drop
                    },
                    "{names}/{state}/{dnat}"
                );
            }
        }
        let listed = t.command("iptables -S").unwrap();
        let mut copy = Table::new(Verdict::Accept);
        for line in listed.text.lines() {
            copy.command(&format!("iptables {line}")).unwrap();
        }
        assert_eq!(copy, t);
    }
}
#[test]
fn exact_policy_specializes_both_connection_class_and_destination_translation() {
    use dlms_learning_lab::{
        linux_filter_policy::compare_filter,
        linux_router_path::{Facts, Link},
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
        routes: vec![],
    };
    let domain = Rule {
        source: None,
        destination: Some(Cidr::parse("10.20.30.42/32").unwrap()),
        ingress: Some("wan0".into()),
        egress: Some("lan0".into()),
        protocol: Some(Protocol::Tcp),
        source_port: None,
        destination_port: Some(443),
        target: Verdict::Accept,
    };
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -i wan0 -o lan0 -d 10.20.30.42 -p tcp --dport 443 -m conntrack --ctstate DNAT -j ACCEPT").unwrap();
    for class in [State::New, State::Established] {
        for dnat in [false, true] {
            let allowed = if dnat { vec![domain.clone()] } else { vec![] };
            assert!(
                compare_filter(
                    &t.specialize_context(class, dnat).unwrap(),
                    Chain::Forward,
                    &facts,
                    std::slice::from_ref(&domain),
                    &allowed
                )
                .unwrap()
                .is_none()
            );
        }
    }
    // NEW,DNAT matches either state and incorrectly admits a directly addressed NEW packet.
    t.command("iptables -F FORWARD").unwrap();
    t.command("iptables -A FORWARD -i wan0 -o lan0 -m conntrack --ctstate NEW,DNAT -j ACCEPT")
        .unwrap();
    assert!(
        compare_filter(
            &t.specialize_context(State::New, false).unwrap(),
            Chain::Forward,
            &facts,
            std::slice::from_ref(&domain),
            &[]
        )
        .unwrap()
        .is_some()
    );
}
