use dlms_learning_lab::{acl, acl_editor as e, acl_routing as ar, ipv4_routing as ip4};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn interface(link: &str, address: &str) -> ip4::Interface {
    ip4::Interface {
        link: link.into(),
        up: true,
        address: Some(ip4::Assignment {
            address: ip(address),
            prefix: 24,
        }),
    }
}
fn default(via: &str) -> ip4::Route {
    ip4::Route {
        destination: ip4::Prefix::parse("0.0.0.0/0").unwrap(),
        interface: "lan".into(),
        via: ip(via),
        distance: 1,
    }
}
fn network() -> ar::Network {
    ar::Network {
        schema_version: 1,
        topology: ip4::Topology {
            devices: BTreeMap::from([
                (
                    "Client".into(),
                    ip4::Device {
                        forwarding: false,
                        interfaces: BTreeMap::from([(
                            "lan".into(),
                            interface("office", "192.0.2.10"),
                        )]),
                        routes: vec![default("192.0.2.1")],
                    },
                ),
                (
                    "R1".into(),
                    ip4::Device {
                        forwarding: true,
                        interfaces: BTreeMap::from([
                            ("office".into(), interface("office", "192.0.2.1")),
                            ("branch".into(), interface("branch", "198.51.100.1")),
                        ]),
                        routes: vec![],
                    },
                ),
                (
                    "Server".into(),
                    ip4::Device {
                        forwarding: false,
                        interfaces: BTreeMap::from([(
                            "lan".into(),
                            interface("branch", "198.51.100.20"),
                        )]),
                        routes: vec![default("198.51.100.1")],
                    },
                ),
                (
                    "R2".into(),
                    ip4::Device {
                        forwarding: true,
                        interfaces: BTreeMap::from([(
                            "spare".into(),
                            interface("spare", "10.0.2.1"),
                        )]),
                        routes: vec![],
                    },
                ),
            ]),
        },
        host_endpoints: BTreeMap::from([
            ("Client".into(), "lan".into()),
            ("Server".into(), "lan".into()),
        ]),
        devices: BTreeMap::new(),
    }
}
fn endpoint(device: &str) -> ar::Endpoint {
    ar::Endpoint {
        device: device.into(),
        interface: "lan".into(),
    }
}
fn flow(protocol: acl::PacketProtocol) -> ar::Flow {
    ar::Flow {
        source: endpoint("Client"),
        destination: endpoint("Server"),
        protocol,
    }
}
fn tcp(port: u16) -> ar::Flow {
    flow(acl::PacketProtocol::Tcp {
        source_port: 50000,
        destination_port: port,
    })
}
fn at() -> ar::Location {
    ar::Location {
        device: "R1".into(),
        interface: "office".into(),
        direction: ar::Direction::Inbound,
    }
}
fn definition() -> e::Definition {
    e::Definition {
        schema_version: 1,
        initial: network(),
        cases: vec![
            e::Case {
                id: "web".into(),
                label: "HTTPS remains available".into(),
                flow: tcp(443),
                expectation: e::Expectation::Allowed {},
            },
            e::Case {
                id: "dns".into(),
                label: "DNS remains available".into(),
                flow: flow(acl::PacketProtocol::Udp {
                    source_port: 53000,
                    destination_port: 53,
                }),
                expectation: e::Expectation::Allowed {},
            },
            e::Case {
                id: "ssh".into(),
                label: "SSH is denied".into(),
                flow: tcp(22),
                expectation: e::Expectation::Denied {
                    expected: e::DenialExpectation {
                        at: at(),
                        cause: None,
                    },
                },
            },
            e::Case {
                id: "mail".into(),
                label: "Other traffic uses implicit deny".into(),
                flow: tcp(25),
                expectation: e::Expectation::Denied {
                    expected: e::DenialExpectation {
                        at: at(),
                        cause: Some(ar::ExpectedCause::Implicit {}),
                    },
                },
            },
        ],
    }
}
const CORRECT: &str = "ip access-list extended edge\n 20 permit tcp any any eq 443\n 10 deny tcp any any eq 22\n 30 permit udp any any eq 53\nexit\ninterface office\n ip access-group edge in\nexit\n";
const ALTERNATE: &str = "! same outcome without sequence numbers\nip access-list extended another\n deny tcp 0.0.0.0 255.255.255.255 range 0 65535 any range 22 22\n permit tcp any range 0 65535 any range 443 443\n permit udp any any range 53 53\nexit\ninterface office\n ip access-group another in\nexit\n";
fn solution() -> ar::Network {
    let d = definition();
    e::apply_router(&d, &d.initial, "R1", CORRECT).unwrap()
}

#[test]
fn grammar_covers_standard_extended_both_ports_wildcards_bindings_and_exact_render_roundtrip() {
    let n = network();
    let text = "! example with different addresses\nIP ACCESS-LIST STANDARD Team\n 65535 DENY host 203.0.113.9\n 1 PERMIT 10.1.2.91 0.0.254.255\nexit\nip access-list extended Services\n permit ip 255.255.255.255 255.255.255.255 host 198.51.100.20\n deny icmp any any\n permit tcp host 192.0.2.10 range 0 65535 any eq 443\n permit udp any eq 53 any range 0 65535\nexit\ninterface office\n ip access-group Team in\n ip access-group Services out\nexit\ninterface branch\nexit\n";
    let parsed = e::parse_router(&n, "R1", text).unwrap();
    assert_eq!(parsed.policy.lists["Team"].rules[0].id, "seq-1");
    assert_eq!(parsed.policy.lists["Team"].rules[1].id, "seq-65535");
    assert_eq!(
        parsed.attachments["office"].inbound.as_deref(),
        Some("Team")
    );
    assert_eq!(
        parsed.attachments["office"].outbound.as_deref(),
        Some("Services")
    );
    assert_eq!(parsed.attachments["branch"], ar::Attachment::default());
    let mut applied = n.clone();
    applied.devices.insert("R1".into(), parsed.clone());
    let rendered = e::render_router(&applied, "R1").unwrap();
    assert_eq!(e::parse_router(&n, "R1", &rendered).unwrap(), parsed);
    assert!(
        matches!(parsed.policy.lists["Services"].rules[0].selector,acl::Selector::Extended{source:acl::Wildcard{base,wildcard},..}if base==Ipv4Addr::BROADCAST&&wildcard==Ipv4Addr::BROADCAST)
    );
    let crlf = text.replace('\n', "\r\n");
    assert_eq!(e::parse_router(&n, "R1", &crlf).unwrap(), parsed);
    assert!(e::parse_router(&n, "R1", "ip access-list standard x\n permit any").is_ok());
}

#[test]
fn explicit_numeric_order_not_document_order_and_unnumbered_order_both_control_first_match() {
    let d = definition();
    let result = e::apply_router(&d, &d.initial, "R1", CORRECT).unwrap();
    assert!(
        e::grade(&d, &result, &e::Drafts::default())
            .unwrap()
            .complete
    );
    let alternate = e::apply_router(&d, &d.initial, "R1", ALTERNATE).unwrap();
    assert!(
        e::grade(&d, &alternate, &e::Drafts::default())
            .unwrap()
            .complete
    );
    let wrong = "ip access-list extended edge\n 1 permit ip any any\n 2 deny tcp any any eq 22\nexit\ninterface office\n ip access-group edge in";
    let bad = e::apply_router(&d, &result, "R1", wrong).unwrap();
    let report = e::grade(&d, &bad, &e::Drafts::default()).unwrap();
    assert!(!report.complete);
    assert!(report.cases[0].passed);
    assert!(!report.cases[2].passed);
    let unnumbered = wrong
        .replace("1 permit", "permit")
        .replace("2 deny", "deny");
    let parsed = e::parse_router(&d.initial, "R1", &unnumbered).unwrap();
    assert_eq!(parsed.policy.lists["edge"].rules[0].id, "row-1");
}

#[test]
fn ordinary_denial_accepts_explicit_and_implicit_equivalents_but_requested_causes_are_exact() {
    let mut d = definition();
    d.cases.retain(|c| c.id != "mail");
    d.validate().unwrap();
    let explicit = "ip access-list extended edge\n deny tcp any any eq 22\n permit ip any any\nexit\ninterface office\n ip access-group edge in";
    let implicit = "ip access-list extended edge\n permit tcp any any eq 443\n permit udp any any eq 53\nexit\ninterface office\n ip access-group edge in";
    let a = e::apply_router(&d, &d.initial, "R1", explicit).unwrap();
    let b = e::apply_router(&d, &d.initial, "R1", implicit).unwrap();
    assert!(e::grade(&d, &a, &e::Drafts::default()).unwrap().complete);
    assert!(e::grade(&d, &b, &e::Drafts::default()).unwrap().complete);
    let text = serde_json::to_string(&d).unwrap();
    assert!(!text.contains("\"cause\""));
    assert_eq!(e::Definition::from_json(text.as_bytes()).unwrap(), d);
    for (cause, a_pass, b_pass) in [
        (ar::ExpectedCause::Explicit { rule: None }, true, false),
        (ar::ExpectedCause::Implicit {}, false, true),
    ] {
        let mut specific = d.clone();
        if let e::Expectation::Denied { expected } = &mut specific.cases[2].expectation {
            expected.cause = Some(cause);
        }
        assert_eq!(
            e::grade(&specific, &a, &e::Drafts::default())
                .unwrap()
                .complete,
            a_pass
        );
        assert_eq!(
            e::grade(&specific, &b, &e::Drafts::default())
                .unwrap()
                .complete,
            b_pass
        );
    }
}

#[test]
fn unsupported_and_malformed_syntax_has_bounded_line_diagnostics_without_effects() {
    let n = network();
    let before = serde_json::to_vec(&n).unwrap();
    for text in [
        "configure terminal",
        "no ip access-list standard edge",
        "exit",
        "interface missing\nexit",
        "interface office\n ip access-group missing in\nexit",
        "ip access-list standard edge\nexit",
        "ip access-list standard edge\n 0 permit any",
        "ip access-list standard edge\n 65536 permit any",
        "ip access-list standard edge\n 10 permit any\n deny any",
        "ip access-list standard edge\n 10 permit any\n 10 deny any",
        "ip access-list standard edge\n permit any\ninterface office",
        "ip access-list standard edge\n permit any\nexit\nip access-list extended edge\n permit ip any any",
        "interface office\n ip access-group edge in\n ip access-group edge in",
        "interface office\nexit\ninterface office",
        "interface office\n description label",
        "ip access-list standard edge\n permit 192.0.2.10",
        "ip access-list standard edge\n permit 192.0.2.0/24",
        "ip access-list standard edge\n permit any log",
        "ip access-list standard edge\n remark allowed",
        "ip access-list standard edge\n permit any ! inline comment",
        "ip access-list extended edge\n permit tcp any any eq https",
        "ip access-list extended edge\n permit tcp any any range 443 22",
        "ip access-list extended edge\n permit tcp any any eq -1",
        "ip access-list extended edge\n permit tcp any any eq 65536",
        "ip access-list extended edge\n permit tcp any any established",
        "ip access-list extended edge\n permit ip any any eq 22",
        "ip access-list extended edge\n permit 6 any any",
        "ip access-list standard ../edge\n permit any",
        "ip access-list standard edge\n permit $(echo unsafe)",
        "ip access-list standard edge\n permit any\0",
    ] {
        let error = e::parse_router(&n, "R1", text).unwrap_err();
        assert!(error.message.len() <= 240);
        assert!(
            error.line.is_some() || error.category == e::Category::Resolution,
            "{text}: {error:?}"
        );
        assert_eq!(serde_json::to_vec(&n).unwrap(), before);
    }
    let error = e::parse_router(
        &n,
        "R1",
        "! heading\nip access-list standard edge\n permit 192.0.2.10",
    )
    .unwrap_err();
    assert_eq!(error.line, Some(3));
    assert!(error.message.contains("host ADDRESS"));
    let named = e::parse_router(
        &n,
        "R1",
        "ip access-list extended edge\n permit tcp any any eq https",
    )
    .unwrap_err();
    assert_eq!(named.category, e::Category::Unsupported);
    let malformed = e::parse_router(
        &n,
        "R1",
        "ip access-list extended edge\n permit tcp any any range 443 22",
    )
    .unwrap_err();
    assert_eq!(malformed.category, e::Category::Syntax);
    assert!(e::parse_router(&n, "Client", "").is_err());
    assert!(e::parse_router(&n, "Missing", "").is_err());
}

#[test]
fn applying_a_router_is_atomic_replaces_omissions_and_keeps_all_other_configuration() {
    let d = definition();
    let current = solution();
    let with_other=e::apply_router(&d,&current,"R2","ip access-list standard spare\n permit any\nexit\ninterface spare\n ip access-group spare out").unwrap();
    let other = serde_json::to_vec(&with_other.devices["R2"]).unwrap();
    let bytes = serde_json::to_vec(&with_other).unwrap();
    assert!(e::apply_router(&d,&with_other,"R1","ip access-list standard broken\n permit any\nexit\ninterface office\n ip access-group absent in").is_err());
    assert_eq!(serde_json::to_vec(&with_other).unwrap(), bytes);
    let cleared = e::apply_router(&d, &with_other, "R1", "").unwrap();
    assert!(!cleared.devices.contains_key("R1"));
    assert_eq!(serde_json::to_vec(&cleared.devices["R2"]).unwrap(), other);
    assert_eq!(cleared.topology, with_other.topology);
    assert_eq!(cleared.host_endpoints, with_other.host_endpoints);
    assert!(
        !e::grade(&d, &cleared, &e::Drafts::default())
            .unwrap()
            .complete
    );
    let replaced = e::apply_router(&d, &with_other, "R1", ALTERNATE).unwrap();
    assert!(!replaced.devices["R1"].policy.lists.contains_key("edge"));
    assert!(replaced.devices["R1"].policy.lists.contains_key("another"));
    assert_eq!(serde_json::to_vec(&replaced.devices["R2"]).unwrap(), other);
}

#[test]
fn renderer_refuses_opaque_or_reordered_ids_instead_of_silently_renaming_rules() {
    let mut n = solution();
    n.devices
        .get_mut("R1")
        .unwrap()
        .policy
        .lists
        .get_mut("edge")
        .unwrap()
        .rules[0]
        .id = "opaque".into();
    assert!(n.validate().is_ok());
    assert_eq!(
        e::render_router(&n, "R1").unwrap_err().category,
        e::Category::Unsupported
    );
    let mut n = solution();
    n.devices
        .get_mut("R1")
        .unwrap()
        .policy
        .lists
        .get_mut("edge")
        .unwrap()
        .rules
        .swap(0, 1);
    assert_eq!(
        e::render_router(&n, "R1").unwrap_err().category,
        e::Category::Unsupported
    );
    let mut d = definition();
    d.initial = n;
    assert!(d.validate().is_err());
}

#[test]
fn fresh_grading_rejects_broken_success_broad_policies_wrong_protocol_port_and_cause() {
    let d = definition();
    let n = solution();
    let earlier = e::grade(&d, &n, &e::Drafts::default()).unwrap();
    assert!(earlier.complete);
    for text in [
        "ip access-list extended edge\n deny ip any any\nexit\ninterface office\n ip access-group edge in",
        "ip access-list extended edge\n permit ip any any\nexit\ninterface office\n ip access-group edge in",
        "ip access-list extended edge\n deny tcp any any eq 22\n permit ip any any\nexit\ninterface office\n ip access-group edge in",
        &CORRECT.replace("permit tcp any any eq 443", "permit udp any any eq 443"),
        &CORRECT.replace("permit tcp any any eq 443", "permit tcp any eq 443 any"),
        &CORRECT.replace("edge in", "edge out"),
        &CORRECT.replace("office", "branch"),
        &CORRECT.replace(
            "permit tcp any any eq 443",
            "permit tcp host 192.0.2.11 any eq 443",
        ),
        &CORRECT.replace(
            "permit tcp any any eq 443",
            "permit tcp any host 198.51.100.21 eq 443",
        ),
        &format!(
            "{}\n 40 deny ip any any\nexit\ninterface office\n ip access-group edge in\n",
            CORRECT.split("exit").next().unwrap()
        ),
    ] {
        let broken = e::apply_router(&d, &n, "R1", text).unwrap();
        let report = e::grade(&d, &broken, &e::Drafts::default()).unwrap();
        assert!(!report.complete, "{text}");
        assert!(earlier.complete);
    }
    let mut prescribed = d.clone();
    if let e::Expectation::Denied { expected } = &mut prescribed.cases[2].expectation {
        expected.cause = Some(ar::ExpectedCause::Explicit {
            rule: Some("seq-10".into()),
        });
    }
    assert!(
        e::grade(&prescribed, &n, &e::Drafts::default())
            .unwrap()
            .complete
    );
    let alt = e::apply_router(&d, &n, "R1", ALTERNATE).unwrap();
    assert!(
        !e::grade(&prescribed, &alt, &e::Drafts::default())
            .unwrap()
            .complete
    );
}

#[test]
fn divergent_or_malformed_drafts_clear_current_completion_without_overwriting_work() {
    let d = definition();
    let n = solution();
    let mut drafts = e::Drafts::default();
    drafts.routers.insert("R1".into(), CORRECT.into());
    assert!(e::grade(&d, &n, &drafts).unwrap().complete);
    drafts
        .routers
        .insert("R1".into(), format!("! harmless comment\n{CORRECT}"));
    assert!(e::grade(&d, &n, &drafts).unwrap().complete);
    let broken = CORRECT.replace("eq 22", "eq 23");
    drafts.routers.insert("R1".into(), broken);
    let before = serde_json::to_vec(&drafts).unwrap();
    assert_eq!(
        e::grade(&d, &n, &drafts).unwrap_err().category,
        e::Category::Unapplied
    );
    assert_eq!(serde_json::to_vec(&drafts).unwrap(), before);
    drafts.routers.insert(
        "R1".into(),
        "ip access-list extended edge\n permit tcp any".into(),
    );
    let bytes = serde_json::to_vec(&drafts).unwrap();
    let restored = e::Drafts::from_json(&bytes, &n).unwrap();
    assert_eq!(restored, drafts);
    assert!(e::grade(&d, &n, &restored).is_err());
    assert!(e::grade(&d, &n, &e::Drafts::default()).unwrap().complete);
}

#[test]
fn full_topology_and_host_role_pins_apply_before_grading_or_router_changes() {
    let d = definition();
    let n = solution();
    for change in 0..7 {
        let mut bad = n.clone();
        match change {
            0 => {
                bad.topology
                    .devices
                    .get_mut("Client")
                    .unwrap()
                    .interfaces
                    .get_mut("lan")
                    .unwrap()
                    .address
                    .as_mut()
                    .unwrap()
                    .address = ip("192.0.2.11")
            }
            1 => {
                bad.topology
                    .devices
                    .get_mut("R1")
                    .unwrap()
                    .interfaces
                    .get_mut("office")
                    .unwrap()
                    .address
                    .as_mut()
                    .unwrap()
                    .prefix = 25
            }
            2 => bad.topology.devices.get_mut("R1").unwrap().forwarding = false,
            3 => {
                bad.topology
                    .devices
                    .get_mut("R1")
                    .unwrap()
                    .interfaces
                    .get_mut("branch")
                    .unwrap()
                    .up = false
            }
            4 => bad
                .topology
                .devices
                .get_mut("R1")
                .unwrap()
                .routes
                .push(ip4::Route {
                    destination: ip4::Prefix::parse("203.0.113.0/24").unwrap(),
                    interface: "office".into(),
                    via: ip("192.0.2.2"),
                    distance: 1,
                }),
            5 => {
                bad.topology
                    .devices
                    .get_mut("R1")
                    .unwrap()
                    .interfaces
                    .get_mut("office")
                    .unwrap()
                    .link = "different".into()
            }
            6 => {
                bad.topology.devices.get_mut("R2").unwrap().forwarding = false;
                bad.host_endpoints.insert("R2".into(), "spare".into());
            }
            _ => unreachable!(),
        }
        let before = serde_json::to_vec(&bad).unwrap();
        assert!(e::grade(&d, &bad, &e::Drafts::default()).is_err());
        assert!(e::apply_router(&d, &bad, "R1", CORRECT).is_err());
        assert_eq!(serde_json::to_vec(&bad).unwrap(), before);
    }
}

#[test]
fn invalid_definitions_cannot_justify_bad_baselines_host_locations_or_unrelated_positives() {
    let d = definition();
    d.validate().unwrap();
    for change in 0..9 {
        let mut bad = d.clone();
        match change {
            0 => bad.cases[1].id = bad.cases[0].id.clone(),
            1 => bad.cases[0].label = "x".repeat(81),
            2 => bad.cases[0].label = "hidden\nline".into(),
            3 => bad
                .cases
                .retain(|c| matches!(c.expectation, e::Expectation::Allowed {})),
            4 => {
                bad.initial
                    .topology
                    .devices
                    .get_mut("R1")
                    .unwrap()
                    .forwarding = false
            }
            5 => {
                if let e::Expectation::Denied { expected } = &mut bad.cases[2].expectation {
                    expected.at.device = "Client".into();
                    expected.at.interface = "lan".into();
                }
            }
            6 => {
                for c in &mut bad.cases {
                    if matches!(c.expectation, e::Expectation::Allowed {}) {
                        std::mem::swap(&mut c.flow.source, &mut c.flow.destination);
                    }
                }
            }
            7 => {
                if let e::Expectation::Denied { expected } = &mut bad.cases[2].expectation {
                    expected.cause = Some(ar::ExpectedCause::Explicit {
                        rule: Some("row-1".into()),
                    });
                }
            }
            8 => {
                if let e::Expectation::Denied { expected } = &mut bad.cases[2].expectation {
                    expected.at.device = "R2".into();
                    expected.at.interface = "spare".into();
                }
            }
            _ => unreachable!(),
        }
        assert!(bad.validate().is_err(), "change {change}");
    }
}

fn many_routers() -> ar::Network {
    let mut n = network();
    for i in 3..=5 {
        n.topology.devices.insert(
            format!("R{i}"),
            ip4::Device {
                forwarding: true,
                interfaces: BTreeMap::from([(
                    "unused".into(),
                    ip4::Interface {
                        link: format!("unused{i}"),
                        up: false,
                        address: None,
                    },
                )]),
                routes: vec![],
            },
        );
    }
    n
}
#[test]
fn raw_draft_json_preserves_maximum_escaped_malformed_text_and_rejects_total_plus_one() {
    let n = many_routers();
    let mut drafts = e::Drafts::default();
    for key in ["R1", "R2", "R3", "R4"] {
        drafts.routers.insert(key.into(), "\0".repeat(e::MAX_TEXT));
    }
    drafts.validate(&n).unwrap();
    let encoded = serde_json::to_vec_pretty(&drafts).unwrap();
    assert!(encoded.len() < e::MAX_JSON);
    assert_eq!(e::Drafts::from_json(&encoded, &n).unwrap(), drafts);
    assert!(drafts.matches(&n).is_err());
    let mut bad = drafts.clone();
    bad.routers.insert("R5".into(), "x".into());
    assert_eq!(bad.validate(&n).unwrap_err().category, e::Category::Limit);
    assert!(e::Drafts::from_json(&serde_json::to_vec(&bad).unwrap(), &n).is_err());
    let mut one = e::Drafts::default();
    one.routers.insert("R1".into(), "x".repeat(e::MAX_TEXT + 1));
    assert!(one.validate(&n).is_err());
    assert!(e::Drafts::from_json(&serde_json::to_vec(&one).unwrap(), &n).is_err());
    let valid = "{\"schema_version\":1,\"routers\":{\"R1\":\"first\"}}";
    let duplicate = valid.replace("\"R1\":\"first\"", "\"R1\":\"first\",\"R1\":\"second\"");
    assert!(e::Drafts::from_json(duplicate.as_bytes(), &n).is_err());
    for raw in [
        valid.replace("\"R1\"", "\"Client\""),
        valid.replace("\"schema_version\":1", "\"schema_version\":2"),
        valid.replace(
            "\"schema_version\":1",
            "\"schema_version\":1,\"extra\":true",
        ),
    ] {
        assert!(e::Drafts::from_json(raw.as_bytes(), &n).is_err());
    }
    assert!(e::Drafts::from_json(b"\xff", &n).is_err());
    assert_eq!(
        e::Drafts::from_json(&vec![b' '; e::MAX_JSON + 1], &n)
            .unwrap_err()
            .category,
        e::Category::Limit
    );
}

#[test]
fn editor_limits_are_checked_at_exact_and_next_byte_line_token_rule_and_list_boundaries() {
    let n = network();
    let exact = ("!".to_owned() + &"x".repeat(510) + "\n").repeat(128);
    assert_eq!(exact.len(), e::MAX_TEXT);
    e::parse_router(&n, "R1", &exact).unwrap();
    assert_eq!(
        e::parse_router(&n, "R1", &(exact + " "))
            .unwrap_err()
            .category,
        e::Category::Limit
    );
    e::parse_router(&n, "R1", &("!".to_owned() + &"x".repeat(e::MAX_LINE - 1))).unwrap();
    assert_eq!(
        e::parse_router(&n, "R1", &("!".to_owned() + &"x".repeat(e::MAX_LINE)))
            .unwrap_err()
            .category,
        e::Category::Limit
    );
    e::parse_router(&n, "R1", &"\n".repeat(e::MAX_LINES)).unwrap();
    assert_eq!(
        e::parse_router(&n, "R1", &"\n".repeat(e::MAX_LINES + 1))
            .unwrap_err()
            .category,
        e::Category::Limit
    );
    assert_ne!(
        e::parse_router(&n, "R1", &"x ".repeat(e::MAX_TOKENS))
            .unwrap_err()
            .category,
        e::Category::Limit
    );
    assert_eq!(
        e::parse_router(&n, "R1", &"x ".repeat(e::MAX_TOKENS + 1))
            .unwrap_err()
            .category,
        e::Category::Limit
    );
    let list = format!(
        "ip access-list standard one\n{}exit\n",
        " permit any\n".repeat(64)
    );
    let maximum = e::parse_router(&n, "R1", &list).unwrap();
    assert_eq!(maximum.policy.lists["one"].rules.len(), 64);
    assert_eq!(
        e::parse_router(&n, "R1", &list.replace("exit", " permit any\nexit"))
            .unwrap_err()
            .category,
        e::Category::Limit
    );
    let eight = (0..8)
        .map(|i| format!("ip access-list standard list{i}\n permit any\nexit\n"))
        .collect::<String>();
    assert_eq!(
        e::parse_router(&n, "R1", &eight)
            .unwrap()
            .policy
            .lists
            .len(),
        8
    );
    assert_eq!(
        e::parse_router(
            &n,
            "R1",
            &(eight + "ip access-list standard ninth\n permit any\nexit")
        )
        .unwrap_err()
        .category,
        e::Category::Limit
    );
}

#[test]
fn network_wide_limits_are_checked_before_candidate_clone_and_definitions_are_strict() {
    let d = definition();
    let eight = (0..8)
        .map(|i| {
            format!(
                "ip access-list standard list{i}\n{}exit\n",
                " permit any\n".repeat(64)
            )
        })
        .collect::<String>();
    let maximum = e::apply_router(&d, &d.initial, "R1", &eight).unwrap();
    assert_eq!(
        maximum.devices["R1"]
            .policy
            .lists
            .values()
            .map(|l| l.rules.len())
            .sum::<usize>(),
        512
    );
    let before = serde_json::to_vec(&maximum).unwrap();
    assert!(
        e::apply_router(
            &d,
            &maximum,
            "R2",
            "ip access-list standard ninth\n permit any"
        )
        .is_err()
    );
    assert_eq!(serde_json::to_vec(&maximum).unwrap(), before);
    let mut sixteen = d.clone();
    while sixteen.cases.len() < 16 {
        let mut c = d.cases[0].clone();
        c.id = format!("case{}", sixteen.cases.len());
        sixteen.cases.push(c);
    }
    sixteen.validate().unwrap();
    let encoded = serde_json::to_vec(&sixteen).unwrap();
    assert_eq!(e::Definition::from_json(&encoded).unwrap(), sixteen);
    let mut extra = sixteen.clone();
    let mut c = d.cases[0].clone();
    c.id = "seventeenth".into();
    extra.cases.push(c);
    assert!(extra.validate().is_err());
    assert!(e::Definition::from_json(&serde_json::to_vec(&extra).unwrap()).is_err());
    let valid = serde_json::to_string(&d).unwrap();
    for text in [
        valid.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
        valid.replacen("\"id\":\"web\"", "\"id\":\"web\",\"id\":\"other\"", 1),
        valid.replacen(
            "\"kind\":\"allowed\"",
            "\"kind\":\"allowed\",\"cached_pass\":true",
            1,
        ),
    ] {
        assert!(e::Definition::from_json(text.as_bytes()).is_err());
    }
    assert_eq!(
        e::Definition::from_json(&vec![b' '; e::MAX_JSON + 1])
            .unwrap_err()
            .category,
        e::Category::Limit
    );
}
