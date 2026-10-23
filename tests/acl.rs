use dlms_learning_lab::acl::{
    self, AccessList, Action, Cause, ErrorKind, Kind, Packet, PacketProtocol, Policy, Ports,
    Protocol, Rule, Selector, Wildcard,
};
use std::{collections::BTreeMap, net::Ipv4Addr};

fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn host(s: &str) -> Wildcard {
    Wildcard::host(ip(s))
}
fn wildcard(base: &str, wildcard: &str) -> Wildcard {
    Wildcard {
        base: ip(base),
        wildcard: ip(wildcard),
    }
}
fn tcp(s: u16, d: u16) -> PacketProtocol {
    PacketProtocol::Tcp {
        source_port: s,
        destination_port: d,
    }
}
fn udp(s: u16, d: u16) -> PacketProtocol {
    PacketProtocol::Udp {
        source_port: s,
        destination_port: d,
    }
}
fn packet(protocol: PacketProtocol) -> Packet {
    Packet {
        source: ip("192.0.2.12"),
        destination: ip("198.51.100.10"),
        protocol,
    }
}
fn standard(id: &str, action: Action, source: Wildcard) -> Rule {
    Rule {
        id: id.into(),
        action,
        selector: Selector::Standard { source },
    }
}
fn extended(
    id: &str,
    action: Action,
    source: Wildcard,
    destination: Wildcard,
    protocol: Protocol,
) -> Rule {
    Rule {
        id: id.into(),
        action,
        selector: Selector::Extended {
            source,
            destination,
            protocol,
        },
    }
}
fn policy(kind: Kind, rules: Vec<Rule>) -> Policy {
    Policy {
        schema_version: 1,
        lists: BTreeMap::from([("edge".into(), AccessList { kind, rules })]),
    }
}
fn action(policy: &Policy, p: &Packet) -> Action {
    policy.evaluate("edge", p).unwrap().action
}

#[test]
fn first_match_has_explicit_identity_and_reordering_changes_outcome_not_id_sorting() {
    let mut p = policy(
        Kind::Standard,
        vec![
            standard("z-specific", Action::Permit, host("192.0.2.12")),
            standard("a-broad", Action::Deny, Wildcard::any()),
        ],
    );
    let input = packet(tcp(50000, 443));
    let d = p.evaluate("edge", &input).unwrap();
    assert_eq!(d.list, "edge");
    assert_eq!(d.action, Action::Permit);
    assert_eq!(
        d.cause,
        Cause::Matched {
            rule: "z-specific".into(),
            row: 0
        }
    );
    p.lists.get_mut("edge").unwrap().rules.swap(0, 1);
    let d = p.evaluate("edge", &input).unwrap();
    assert_eq!(d.action, Action::Deny);
    assert_eq!(
        d.cause,
        Cause::Matched {
            rule: "a-broad".into(),
            row: 0
        }
    );
    p.lists.get_mut("edge").unwrap().rules.remove(0);
    let mut other = input;
    other.source = ip("192.0.2.13");
    assert_eq!(
        p.evaluate("edge", &other).unwrap().cause,
        Cause::ImplicitDeny
    );
    p.lists.get_mut("edge").unwrap().rules.push(standard(
        "explicit-end",
        Action::Deny,
        Wildcard::any(),
    ));
    assert_eq!(
        p.evaluate("edge", &other).unwrap().cause,
        Cause::Matched {
            rule: "explicit-end".into(),
            row: 1
        }
    );
    let mut shifted = policy(
        Kind::Standard,
        vec![
            standard("unrelated", Action::Deny, host("203.0.113.99")),
            standard("z-specific", Action::Permit, host("192.0.2.12")),
        ],
    );
    assert_eq!(
        shifted.evaluate("edge", &input).unwrap().cause,
        Cause::Matched {
            rule: "z-specific".into(),
            row: 1
        }
    );
    shifted.lists.get_mut("edge").unwrap().rules.swap(0, 1);
    assert_eq!(
        shifted.evaluate("edge", &input).unwrap().cause,
        Cause::Matched {
            rule: "z-specific".into(),
            row: 0
        }
    );
}

#[test]
fn standard_rules_ignore_every_destination_protocol_and_port_but_validate_packet_shape() {
    let p = policy(
        Kind::Standard,
        vec![standard("trusted", Action::Permit, host("192.0.2.12"))],
    );
    let mut protocols = vec![
        PacketProtocol::Icmp {},
        tcp(0, 65535),
        tcp(65535, 0),
        udp(53, 1),
        udp(0, 65535),
    ];
    protocols.extend(
        (0..=255)
            .filter(|n| ![1, 6, 17].contains(n))
            .map(|number| PacketProtocol::Other { number }),
    );
    for protocol in protocols {
        for destination in [
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::BROADCAST,
            ip("203.0.113.4"),
        ] {
            let input = Packet {
                destination,
                ..packet(protocol)
            };
            assert_eq!(action(&p, &input), Action::Permit);
            let other = Packet {
                source: ip("192.0.2.13"),
                ..input
            };
            assert_eq!(action(&p, &other), Action::Deny);
        }
    }
    for number in [1, 6, 17] {
        let invalid = packet(PacketProtocol::Other { number });
        assert_eq!(
            p.evaluate("edge", &invalid).unwrap_err().kind,
            ErrorKind::Invalid
        );
    }
}

#[test]
fn wildcard_matching_uses_all_independent_bits_and_ignores_wildcard_base_bits() {
    for bit in 0..32 {
        let base = 0xa55a_f00fu32;
        let mask = 1u32 << bit;
        let exact = Wildcard::host(base.into());
        assert!(exact.matches(base.into()));
        assert!(!exact.matches((base ^ mask).into()));
        let ignored = Wildcard {
            base: base.into(),
            wildcard: mask.into(),
        };
        assert!(ignored.matches((base ^ mask).into()));
        assert_eq!(
            ignored.matches(0x12345678u32.into()),
            Wildcard {
                base: (base ^ mask).into(),
                wildcard: mask.into()
            }
            .matches(0x12345678u32.into())
        );
    }
    let noncontiguous = wildcard("10.1.2.91", "0.0.254.255");
    for third in 0..=255 {
        assert_eq!(
            noncontiguous.matches(Ipv4Addr::new(10, 1, third, 231)),
            third % 2 == 0
        );
    }
    assert!(!noncontiguous.matches(ip("10.2.2.91")));
    for base in [0, 1, u32::MAX, 0x93c5_2001] {
        let any = Wildcard {
            base: base.into(),
            wildcard: Ipv4Addr::BROADCAST,
        };
        for address in [0, u32::MAX, 0x0808_0808, 0x8000_0000] {
            assert!(any.matches(address.into()));
        }
    }
}

#[test]
fn independent_per_bit_oracle_agrees_for_65536_low_byte_and_4096_full_width_cases() {
    fn oracle(base: u32, wildcard: u32, address: u32) -> bool {
        (0..32).all(|bit| (wildcard >> bit) % 2 == 1 || (base >> bit) % 2 == (address >> bit) % 2)
    }
    for mask in 0..=255u32 {
        for byte in 0..=255u32 {
            let base = 0xc000_02a5;
            let address = 0xc000_0200 | byte;
            assert_eq!(
                Wildcard {
                    base: base.into(),
                    wildcard: mask.into()
                }
                .matches(address.into()),
                oracle(base, mask, address)
            );
        }
    }
    let mut state = 0x6375_9184u32;
    for _ in 0..4096 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let base = state;
        state = state.rotate_left(11).wrapping_add(0xc045_1137);
        let mask = state;
        state = state.wrapping_mul(22695477).wrapping_add(1);
        assert_eq!(
            Wildcard {
                base: base.into(),
                wildcard: mask.into()
            }
            .matches(state.into()),
            oracle(base, mask, state)
        );
    }
}

#[test]
fn extended_rules_require_all_address_protocol_and_both_port_conditions() {
    let p = policy(
        Kind::Extended,
        vec![extended(
            "https",
            Action::Permit,
            wildcard("192.0.2.255", "0.0.0.255"),
            host("198.51.100.10"),
            Protocol::Tcp {
                source_port: Ports::Range {
                    first: 49152,
                    last: 65535,
                },
                destination_port: Ports::Eq { port: 443 },
            },
        )],
    );
    for source_port in [49152, 53000, 65535] {
        assert_eq!(action(&p, &packet(tcp(source_port, 443))), Action::Permit);
    }
    for input in [
        Packet {
            source: ip("192.0.3.12"),
            ..packet(tcp(49152, 443))
        },
        Packet {
            destination: ip("198.51.100.11"),
            ..packet(tcp(49152, 443))
        },
        packet(tcp(49151, 443)),
        packet(tcp(49152, 442)),
        packet(tcp(49152, 444)),
        packet(udp(49152, 443)),
        packet(PacketProtocol::Icmp {}),
        packet(PacketProtocol::Other { number: 47 }),
    ] {
        assert_eq!(action(&p, &input), Action::Deny);
    }
    let udp_policy = policy(
        Kind::Extended,
        vec![extended(
            "dns",
            Action::Permit,
            Wildcard::any(),
            Wildcard::any(),
            Protocol::Udp {
                source_port: Ports::Any {},
                destination_port: Ports::Eq { port: 53 },
            },
        )],
    );
    assert_eq!(action(&udp_policy, &packet(udp(0, 53))), Action::Permit);
    assert_eq!(action(&udp_policy, &packet(udp(65535, 53))), Action::Permit);
    assert_eq!(action(&udp_policy, &packet(tcp(65535, 53))), Action::Deny);
    assert_eq!(action(&udp_policy, &packet(udp(53, 53000))), Action::Deny);
}

#[test]
fn port_bounds_are_inclusive_and_ip_and_icmp_protocols_are_distinct() {
    let p = policy(
        Kind::Extended,
        vec![extended(
            "range",
            Action::Permit,
            Wildcard::any(),
            Wildcard::any(),
            Protocol::Tcp {
                source_port: Ports::Range {
                    first: 0,
                    last: 65535,
                },
                destination_port: Ports::Range {
                    first: 100,
                    last: 200,
                },
            },
        )],
    );
    for port in 0..=65535 {
        assert_eq!(
            action(&p, &packet(tcp(0, port))),
            if (100..=200).contains(&port) {
                Action::Permit
            } else {
                Action::Deny
            }
        );
    }
    for protocol in [Protocol::Ip {}, Protocol::Icmp {}] {
        let p = policy(
            Kind::Extended,
            vec![extended(
                "protocol",
                Action::Permit,
                Wildcard::any(),
                Wildcard::any(),
                protocol,
            )],
        );
        assert_eq!(action(&p, &packet(PacketProtocol::Icmp {})), Action::Permit);
        for packet_protocol in [
            tcp(0, 0),
            udp(65535, 65535),
            PacketProtocol::Other { number: 0 },
            PacketProtocol::Other { number: 255 },
        ] {
            assert_eq!(
                action(&p, &packet(packet_protocol)),
                if matches!(protocol, Protocol::Ip {}) {
                    Action::Permit
                } else {
                    Action::Deny
                }
            );
        }
    }
}

#[test]
fn equivalent_rules_pass_behavior_cases_and_wrong_partial_broad_policies_do_not() {
    let permit = extended(
        "allow",
        Action::Permit,
        host("192.0.2.12"),
        host("198.51.100.10"),
        Protocol::Tcp {
            source_port: Ports::Any {},
            destination_port: Ports::Eq { port: 443 },
        },
    );
    let p = policy(Kind::Extended, vec![permit.clone()]);
    let mut equivalent = permit.clone();
    if let Selector::Extended { protocol, .. } = &mut equivalent.selector {
        *protocol = Protocol::Tcp {
            source_port: Ports::Range {
                first: 0,
                last: 65535,
            },
            destination_port: Ports::Range {
                first: 443,
                last: 443,
            },
        };
    }
    let alternate = policy(
        Kind::Extended,
        vec![
            equivalent,
            extended(
                "end",
                Action::Deny,
                Wildcard::any(),
                Wildcard::any(),
                Protocol::Ip {},
            ),
        ],
    );
    let cases = [
        packet(tcp(30000, 443)),
        packet(tcp(30000, 22)),
        Packet {
            source: ip("192.0.2.13"),
            ..packet(tcp(30000, 443))
        },
        packet(udp(30000, 443)),
    ];
    let expected = [Action::Permit, Action::Deny, Action::Deny, Action::Deny];
    for p in [&p, &alternate] {
        for (case, expected) in cases.iter().zip(expected) {
            assert_eq!(action(p, case), expected);
        }
    }
    let deny_all = policy(
        Kind::Extended,
        vec![extended(
            "block",
            Action::Deny,
            Wildcard::any(),
            Wildcard::any(),
            Protocol::Ip {},
        )],
    );
    let permit_all = policy(
        Kind::Extended,
        vec![extended(
            "broad",
            Action::Permit,
            Wildcard::any(),
            Wildcard::any(),
            Protocol::Ip {},
        )],
    );
    for wrong in [deny_all, permit_all] {
        assert!(
            cases
                .iter()
                .zip(expected)
                .any(|(case, expected)| action(&wrong, case) != expected)
        );
    }
    let snapshot = serde_json::to_vec(&alternate).unwrap();
    for _ in 0..100 {
        alternate.evaluate("edge", &cases[0]).unwrap();
    }
    assert_eq!(serde_json::to_vec(&alternate).unwrap(), snapshot);
}

#[test]
fn empty_and_missing_lists_are_unsupported_outcomes_not_successful_denials() {
    let mut p = policy(Kind::Standard, vec![]);
    assert_eq!(
        p.evaluate("edge", &packet(tcp(1, 2))).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    p.lists.clear();
    assert!(p.validate().is_ok());
    assert_eq!(
        p.evaluate("edge", &packet(tcp(1, 2))).unwrap_err().kind,
        ErrorKind::UnknownList
    );
}

#[test]
fn direct_constructed_values_cannot_skip_class_id_range_version_and_count_validation() {
    let base = policy(
        Kind::Standard,
        vec![standard("ok", Action::Permit, Wildcard::any())],
    );
    let mut invalid = vec![];
    let mut p = base.clone();
    p.schema_version = 2;
    invalid.push(p);
    let mut p = base.clone();
    p.lists.get_mut("edge").unwrap().kind = Kind::Extended;
    invalid.push(p);
    let mut p = base.clone();
    let duplicate = p.lists["edge"].rules[0].clone();
    p.lists.get_mut("edge").unwrap().rules.push(duplicate);
    invalid.push(p);
    let mut p = base.clone();
    p.lists.get_mut("edge").unwrap().rules[0].id = "!bad".into();
    invalid.push(p);
    let mut p = base.clone();
    p.lists.insert("../bad".into(), p.lists["edge"].clone());
    invalid.push(p);
    invalid.push(policy(
        Kind::Extended,
        vec![extended(
            "range",
            Action::Permit,
            Wildcard::any(),
            Wildcard::any(),
            Protocol::Udp {
                source_port: Ports::Any {},
                destination_port: Ports::Range { first: 3, last: 2 },
            },
        )],
    ));
    let mut p = base.clone();
    for i in 0..acl::MAX_LISTS {
        p.lists
            .insert(format!("list{i}"), base.lists["edge"].clone());
    }
    invalid.push(p);
    let mut p = base.clone();
    p.lists.get_mut("edge").unwrap().rules = (0..=acl::MAX_RULES)
        .map(|i| standard(&format!("rule{i}"), Action::Permit, Wildcard::any()))
        .collect();
    invalid.push(p);
    for p in invalid {
        assert!(p.evaluate("edge", &packet(tcp(1, 2))).is_err());
    }
}

#[test]
fn strict_json_rejects_duplicate_unknown_malformed_and_unsupported_shapes() {
    let p = policy(
        Kind::Standard,
        vec![standard("ok", Action::Permit, Wildcard::any())],
    );
    let good = serde_json::to_string(&p).unwrap();
    assert_eq!(Policy::from_json(good.as_bytes()).unwrap(), p);
    let acl_json = serde_json::to_string(&p.lists["edge"]).unwrap();
    let duplicate_lists =
        format!("{{\"schema_version\":1,\"lists\":{{\"edge\":{acl_json},\"edge\":{acl_json}}}}}");
    let rule_json = serde_json::to_string(&p.lists["edge"].rules[0]).unwrap();
    let duplicate_rule = good.replace(
        &format!("[{rule_json}]"),
        &format!("[{rule_json},{rule_json}]"),
    );
    for text in [
        duplicate_lists,
        duplicate_rule,
        good.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
        good.replace(
            "\"action\":\"permit\"",
            "\"action\":\"permit\",\"flags\":\"established\"",
        ),
        good.replace("\"id\":\"ok\"", "\"id\":\"ok\",\"id\":\"second\""),
        good.replace("\"id\":\"ok\"", "\"id\":\"../bad\""),
        good.replace("0.0.0.0", "2001:db8::1"),
        good.replace("standard", "future"),
        good.replace(
            "\"id\":\"ok\"",
            &format!("\"id\":\"{}\"", "a".repeat(200_000)),
        ),
    ] {
        let e = Policy::from_json(text.as_bytes()).unwrap_err();
        assert!(e.message.len() < 256);
    }
    let input = packet(tcp(12, 443));
    let text = serde_json::to_string(&input).unwrap();
    assert_eq!(Packet::from_json(text.as_bytes()).unwrap(), input);
    assert_eq!(
        Packet::from_json(&serde_json::to_vec_pretty(&input).unwrap()).unwrap(),
        input
    );
    assert_eq!(
        serde_json::from_value::<Policy>(serde_json::to_value(&p).unwrap()).unwrap(),
        p
    );
    for text in [
        text.replace("\"tcp\"", "\"icmp\""),
        text.replace("\"tcp\"", "\"other\""),
        text.replace("\"source_port\":12", "\"source_port\":65536"),
        text.replace("\"source_port\":12", "\"source_port\":-1"),
        text.replace("\"source_port\":12", "\"source_port\":12,\"fragment\":true"),
        text.replace("\"source_port\":12", "\"destination_port\":12"),
    ] {
        assert!(Packet::from_json(text.as_bytes()).is_err(), "{text}");
    }
    for number in [1, 6, 17] {
        assert!(
            Packet::from_json(
                &serde_json::to_vec(&packet(PacketProtocol::Other { number })).unwrap()
            )
            .is_err()
        );
    }
    for body in [
        "{\"kind\":\"icmp\",\"source_port\":12}",
        "{\"kind\":\"other\",\"number\":47,\"destination_port\":443}",
        "{\"kind\":\"tcp\",\"source_port\":12}",
    ] {
        let value = format!(
            "{{\"source\":\"192.0.2.12\",\"destination\":\"198.51.100.10\",\"protocol\":{body}}}"
        );
        assert!(Packet::from_json(value.as_bytes()).is_err());
    }
    for value in [
        "{\"kind\":\"ip\",\"source_port\":12}",
        "{\"kind\":\"icmp\",\"type\":8}",
    ] {
        assert!(serde_json::from_str::<Protocol>(value).is_err());
    }
    assert!(serde_json::from_str::<Ports>("{\"kind\":\"any\",\"port\":443}").is_err());
}

#[test]
fn maximum_policy_roundtrips_compact_and_pretty_and_next_entries_or_bytes_fail() {
    let maximum = Policy {
        schema_version: 1,
        lists: (0..acl::MAX_LISTS)
            .map(|i| {
                (
                    format!("list{i:028}"),
                    AccessList {
                        kind: Kind::Extended,
                        rules: (0..acl::MAX_RULES)
                            .map(|j| {
                                extended(
                                    &format!("rule{j:028}"),
                                    Action::Permit,
                                    wildcard("255.255.255.255", "255.255.255.255"),
                                    wildcard("255.255.255.255", "255.255.255.255"),
                                    Protocol::Tcp {
                                        source_port: Ports::Range {
                                            first: 65535,
                                            last: 65535,
                                        },
                                        destination_port: Ports::Range {
                                            first: 65535,
                                            last: 65535,
                                        },
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect(),
    };
    maximum.validate().unwrap();
    for bytes in [
        serde_json::to_vec(&maximum).unwrap(),
        serde_json::to_vec_pretty(&maximum).unwrap(),
    ] {
        assert!(bytes.len() < acl::MAX_JSON_BYTES);
        assert_eq!(Policy::from_json(&bytes).unwrap(), maximum);
    }
    let mut slowest = maximum.clone();
    let (name, list) = slowest.lists.iter_mut().next().unwrap();
    let name = name.clone();
    for rule in &mut list.rules[..acl::MAX_RULES - 1] {
        if let Selector::Extended { source, .. } = &mut rule.selector {
            *source = host("10.0.0.1");
        }
    }
    assert_eq!(
        slowest
            .evaluate(&name, &packet(tcp(65535, 65535)))
            .unwrap()
            .cause,
        Cause::Matched {
            rule: format!("rule{:028}", acl::MAX_RULES - 1),
            row: acl::MAX_RULES - 1
        }
    );
    let mut excessive = maximum.clone();
    excessive.lists.insert(
        "ninth".into(),
        maximum.lists.values().next().unwrap().clone(),
    );
    assert!(Policy::from_json(&serde_json::to_vec(&excessive).unwrap()).is_err());
    let mut excessive = maximum.clone();
    excessive
        .lists
        .values_mut()
        .next()
        .unwrap()
        .rules
        .push(extended(
            "sixty-fifth",
            Action::Permit,
            Wildcard::any(),
            Wildcard::any(),
            Protocol::Ip {},
        ));
    assert!(Policy::from_json(&serde_json::to_vec(&excessive).unwrap()).is_err());
    assert_eq!(
        Policy::from_json(&vec![b' '; acl::MAX_JSON_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        Packet::from_json(&vec![b' '; acl::MAX_PACKET_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
