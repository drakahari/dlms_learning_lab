use dlms_learning_lab::dhcp_config::{self as c, ErrorKind};
use std::net::Ipv4Addr;
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
const BASIC: &str = "subnet 192.0.2.0 netmask 255.255.255.0 { range 192.0.2.100 192.0.2.120; }";
#[test]
fn actual_scopes_inherit_defaults_and_apply_local_overrides_without_losing_other_options() {
    let text = "default-lease-time 900; max-lease-time 600; option domain-name-servers 203.0.113.53, 203.0.113.54;\nsubnet 192.0.2.0 netmask 255.255.255.0 { default-lease-time 120; option routers 192.0.2.1; range 192.0.2.100 192.0.2.120; }\nsubnet 198.51.100.0 netmask 255.255.255.0 { option routers 198.51.100.1; range 198.51.100.40; }";
    let parsed = c::parse(text).unwrap();
    assert_eq!(parsed.scopes.len(), 2);
    let a = parsed.scopes[0].effective(&parsed.defaults);
    let b = parsed.scopes[1].effective(&parsed.defaults);
    assert_eq!(a.router, Some(ip("192.0.2.1")));
    assert_eq!(b.router, Some(ip("198.51.100.1")));
    assert_eq!(a.dns, Some(vec![ip("203.0.113.53"), ip("203.0.113.54")]));
    assert_eq!(a.dns, b.dns);
    assert_eq!((a.default_duration(), b.default_duration()), (120, 600));
    assert_eq!(
        parsed.scopes[1].range,
        Some((ip("198.51.100.40"), ip("198.51.100.40")))
    );
}
#[test]
fn whitespace_comments_case_and_equivalent_single_range_forms_really_parse() {
    let a=c::parse("SuBnEt# comment\n192.0.2.0 NeTmAsK\t255.255.255.0 { # quoted \"comment\"\n RaNgE 192.0.2.100;\n}").unwrap();
    let b = c::parse("subnet 192.0.2.0 netmask 255.255.255.0 { range 192.0.2.100 192.0.2.100; }")
        .unwrap();
    assert_eq!(a, b);
}
#[test]
fn rangeless_subnet_is_not_an_invented_dynamic_pool_and_actual_default_is_preserved() {
    let p = c::parse("subnet 192.0.2.0 netmask 255.255.255.0 { }").unwrap();
    assert!(p.scopes[0].range.is_none());
    assert_eq!(
        p.scopes[0].effective(&p.defaults).default_duration(),
        43_200
    );
}
#[test]
fn syntactic_options_can_be_wrong_for_the_task_without_becoming_fake_syntax_errors() {
    let p=c::parse("option routers 203.0.113.1; subnet 172.18.16.0 netmask 255.255.240.0 { option subnet-mask 255.255.255.0; range 172.18.17.10 172.18.17.20; }").unwrap();
    assert_eq!(p.scopes[0].subnet.prefix, 20);
    let e = p.scopes[0].effective(&p.defaults);
    assert_eq!(e.subnet_mask, Some(ip("255.255.255.0")));
    assert_eq!(e.router, Some(ip("203.0.113.1")));
}
#[test]
fn network_math_covers_octet_boundaries_and_excludes_conventional_special_cases() {
    for (network, mask, first, last) in [
        (
            "172.18.16.0",
            "255.255.240.0",
            "172.18.16.1",
            "172.18.31.254",
        ),
        ("192.0.2.0", "255.255.255.252", "192.0.2.1", "192.0.2.2"),
    ] {
        let end = if mask == "255.255.240.0" {
            "172.18.16.8"
        } else {
            last
        };
        let text = format!("subnet {network} netmask {mask} {{ range {first} {end}; }}");
        let p = c::parse(&text).unwrap();
        assert!(p.scopes[0].subnet.host(ip(last)));
    }
    for mask in ["0.0.0.0", "255.255.255.254", "255.255.255.255"] {
        let e = c::parse(&format!("subnet 0.0.0.0 netmask {mask} {{ }}")).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unsupported);
    }
    for text in [
        "subnet 192.0.2.77 netmask 255.255.255.0 { }",
        "subnet 192.0.2.0 netmask 255.0.255.0 { }",
        "subnet 300.0.2.0 netmask 255.255.255.0 { }",
    ] {
        assert_eq!(c::parse(text).unwrap_err().kind, ErrorKind::Invalid);
    }
}
#[test]
fn ranges_are_inclusive_bounded_and_checked_against_the_full_subnet() {
    let text = "subnet 10.0.0.0 netmask 255.255.0.0 { range 10.0.0.1 10.0.1.0; }";
    assert!(c::parse(text).is_ok());
    assert_eq!(
        c::parse(&text.replace("10.0.1.0", "10.0.1.1"))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    for range in [
        "192.0.2.120 192.0.2.100",
        "192.0.2.0 192.0.2.10",
        "192.0.2.250 192.0.2.255",
        "198.51.100.10 198.51.100.20",
    ] {
        let t = format!("subnet 192.0.2.0 netmask 255.255.255.0 {{ range {range}; }}");
        assert_eq!(c::parse(&t).unwrap_err().kind, ErrorKind::Invalid);
    }
}
#[test]
fn unsupported_features_are_never_ignored_or_partly_accepted() {
    for text in [
        "include \"/etc/personal.conf\";",
        "on commit { execute ( \"sh\" ); }",
        "authoritative;",
        "option domain-name \"example.test\";",
        "option routers 192.0.2.1, 192.0.2.2;",
        "group { }",
        "host printer { fixed-address 192.0.2.20; }",
        "subnet6 2001:db8::/64 { }",
        "option routers gateway.example.test;",
        "option routers 192.000.2.1;",
    ] {
        let e = c::parse(&format!("{text}\n{BASIC}")).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unsupported, "{text}: {e}");
        assert!(e.to_string().contains("Unsupported"));
        assert!(e.to_string().len() <= 512);
    }
    let p = c::parse(BASIC).unwrap();
    assert_eq!(c::parse(BASIC).unwrap(), p);
}
#[test]
fn duplicate_and_late_parameters_multiple_ranges_and_overlapping_scopes_refuse_honestly() {
    for text in [
        format!("option routers 192.0.2.1; option routers 192.0.2.2; {BASIC}"),
        format!("{BASIC} default-lease-time 600;"),
        "subnet 192.0.2.0 netmask 255.255.255.0 { range 192.0.2.10; option routers 192.0.2.1; }"
            .into(),
        "subnet 192.0.2.0 netmask 255.255.255.0 { range 192.0.2.10; range 192.0.2.20; }".into(),
        format!("{BASIC}\n{BASIC}"),
        format!("{BASIC} subnet 192.0.2.0 netmask 255.255.255.128 {{ }}"),
    ] {
        assert_eq!(
            c::parse(&text).unwrap_err().kind,
            ErrorKind::Unsupported,
            "{text}"
        );
    }
}
#[test]
fn malformed_structure_and_eof_have_honest_source_positions_and_categories() {
    for text in [
        "}",
        ";",
        "subnet 192.0.2.0 netmask 255.255.255.0",
        "subnet 192.0.2.0 netmask 255.255.255.0 { range 192.0.2.100 }",
        "default-lease-time -1;",
    ] {
        let e = c::parse(text).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Syntax, "{text}: {e}");
        assert!(e.to_string().contains("Syntax error"));
    }
    let e =
        c::parse("subnet 192.0.2.0 netmask 255.255.255.0 {\n  # trailing comment\n  ").unwrap_err();
    assert_eq!((e.line, e.column), (3, 3));
    for text in ["", "   # only a comment\n"] {
        assert_eq!(c::parse(text).unwrap_err().kind, ErrorKind::Invalid);
    }
}
#[test]
fn source_token_scope_dns_and_numeric_limits_are_explicit_and_do_not_panic() {
    let mut text = BASIC.to_owned();
    text.push('#');
    text.extend(std::iter::repeat_n('x', c::MAX_TEXT - text.len()));
    assert!(c::parse(&text).is_ok());
    text.push('x');
    assert_eq!(c::parse(&text).unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(
        c::parse(&";".repeat(c::MAX_TOKENS + 1)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        c::parse(&"x".repeat(65)).unwrap_err().kind,
        ErrorKind::Limit
    );
    let scopes = (0..8)
        .map(|n| format!("subnet 10.{n}.0.0 netmask 255.255.0.0 {{ }}"))
        .collect::<String>();
    assert_eq!(c::parse(&scopes).unwrap().scopes.len(), 8);
    assert_eq!(
        c::parse(&(scopes + "subnet 10.8.0.0 netmask 255.255.0.0 { }"))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        c::parse(&format!(
            "option domain-name-servers 192.0.2.1,192.0.2.2,192.0.2.3,192.0.2.4,192.0.2.5; {BASIC}"
        ))
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        c::parse("default-lease-time 4294967296;").unwrap_err().kind,
        ErrorKind::Limit
    );
}
#[test]
fn controls_unicode_and_code_shaped_data_have_no_evaluation_or_import_route() {
    for control in ['\0', '\x1b', '\x7f'] {
        assert_eq!(
            c::parse(&format!("# ignored? {control}\n{BASIC}"))
                .unwrap_err()
                .kind,
            ErrorKind::Syntax
        );
    }
    assert_eq!(
        c::parse(&format!("# café\n{BASIC}")).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    for text in ["$(touch /tmp/nope)", "`id`", "../private", "/etc/shadow"] {
        assert!(c::parse(&format!("option routers {text}; {BASIC}")).is_err());
    }
}

mod compilation {
    use super::*;
    use dlms_learning_lab::dhcpv4 as core;
    use std::collections::{BTreeMap, BTreeSet};
    const TEXT: &str = "default-lease-time 600; max-lease-time 3600; option domain-name-servers 203.0.113.53; subnet 192.0.2.0 netmask 255.255.255.0 { option routers 192.0.2.1; range 192.0.2.100 192.0.2.103; } subnet 198.51.100.0 netmask 255.255.255.0 { option routers 198.51.100.1; range 198.51.100.50 198.51.100.53; }";
    fn topology(occupied: Option<&str>) -> core::Topology {
        let segment = |network, iface| core::Segment {
            subnet: core::Subnet {
                network: ip(network),
                prefix: 24,
            },
            server_interface: Some(ip(iface)),
            relay_interface: None,
            relay_to_server: false,
            client_unicast_to_server: true,
            occupied: BTreeSet::new(),
        };
        let mut t = core::Topology {
            server: ip("192.0.2.1"),
            segments: BTreeMap::from([
                ("alpha".into(), segment("192.0.2.0", "192.0.2.1")),
                ("beta".into(), segment("198.51.100.0", "198.51.100.1")),
            ]),
            clients: BTreeMap::from([
                (
                    "alpha-a".into(),
                    core::Client {
                        segment: "alpha".into(),
                    },
                ),
                (
                    "alpha-b".into(),
                    core::Client {
                        segment: "alpha".into(),
                    },
                ),
                (
                    "beta-a".into(),
                    core::Client {
                        segment: "beta".into(),
                    },
                ),
            ]),
        };
        if let Some(a) = occupied {
            t.segments.get_mut("alpha").unwrap().occupied.insert(ip(a));
        }
        t.validate().unwrap();
        t
    }
    fn acquire(s: &mut core::State, client: &str) -> core::Binding {
        let core::Outcome::Offered(o) = s.discover(client).unwrap() else {
            panic!("actual offer")
        };
        let core::Outcome::Ack { binding, .. } = s.request(client, o.token).unwrap() else {
            panic!("actual ACK")
        };
        binding
    }
    #[test]
    fn compiled_config_drives_real_distinct_client_bindings_across_deterministic_inputs() {
        for occupied in [None, Some("192.0.2.100"), Some("192.0.2.101")] {
            let t = topology(occupied);
            let policy = c::compile(TEXT, &t, &BTreeMap::new()).unwrap();
            let mut s = core::State::new(t, policy).unwrap();
            let a = acquire(&mut s, "alpha-a");
            let b = acquire(&mut s, "alpha-b");
            let remote = acquire(&mut s, "beta-a");
            assert_ne!(a.address, b.address);
            assert_eq!(s.leases().len(), 3);
            for result in [&a, &b] {
                assert!(result.address >= ip("192.0.2.100") && result.address <= ip("192.0.2.103"));
                assert_ne!(Some(result.address), occupied.map(ip));
                assert_eq!(result.parameters.router, Some(ip("192.0.2.1")));
                assert_eq!(result.parameters.dns, [ip("203.0.113.53")]);
                assert_eq!(result.granted_seconds, 600);
            }
            assert!(remote.address >= ip("198.51.100.50") && remote.address <= ip("198.51.100.53"));
            assert_eq!(remote.parameters.router, Some(ip("198.51.100.1")));
            s.validate().unwrap();
        }
    }
    #[test]
    fn equivalent_supported_global_or_local_settings_and_declaration_order_compile_equally() {
        let t = topology(None);
        let a = c::compile(TEXT, &t, &BTreeMap::new()).unwrap();
        let alternate = "subnet 198.51.100.0 netmask 255.255.255.0 { default-lease-time 600; max-lease-time 3600; option domain-name-servers 203.0.113.53; option routers 198.51.100.1; range 198.51.100.50 198.51.100.53; } subnet 192.0.2.0 netmask 255.255.255.0 { option subnet-mask 255.255.255.0; default-lease-time 600; max-lease-time 3600; option domain-name-servers 203.0.113.53; option routers 192.0.2.1; range 192.0.2.100 192.0.2.103; }";
        let b = c::compile(alternate, &t, &BTreeMap::new()).unwrap();
        assert_eq!(a, b);
    }
    #[test]
    fn wrong_options_are_observed_and_missing_ranges_do_not_create_fake_leases() {
        let t = topology(None);
        let wrong = TEXT.replace("option routers 192.0.2.1;", "option routers 203.0.113.1;");
        let mut s =
            core::State::new(t.clone(), c::compile(&wrong, &t, &BTreeMap::new()).unwrap()).unwrap();
        let a = acquire(&mut s, "alpha-a");
        assert_eq!(a.parameters.router, Some(ip("203.0.113.1")));
        assert_ne!(a.parameters.router, Some(ip("192.0.2.1")));
        let mut s = core::State::new(
            t.clone(),
            c::compile(
                "subnet 192.0.2.0 netmask 255.255.255.0 { }",
                &t,
                &BTreeMap::new(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            s.discover("alpha-a").unwrap(),
            core::Outcome::NoReply(core::Reason::NoPool)
        );
        assert!(s.leases().is_empty());
    }
    #[test]
    fn unrepresentable_masks_and_durations_refuse_without_replacing_valid_state() {
        let t = topology(None);
        let s =
            core::State::new(t.clone(), c::compile(TEXT, &t, &BTreeMap::new()).unwrap()).unwrap();
        let before = s.clone();
        for raw in [
            TEXT.replace(
                "option routers 192.0.2.1;",
                "option subnet-mask 255.255.255.128; option routers 192.0.2.1;",
            ),
            TEXT.replace("default-lease-time 600;", "default-lease-time 4;"),
            TEXT.replace(
                "default-lease-time 600; max-lease-time 3600;",
                "default-lease-time 100000; max-lease-time 100000;",
            ),
        ] {
            assert!(
                c::parse(&raw).is_ok(),
                "syntactically supported values are retained"
            );
            let e = c::compile(&raw, &t, &BTreeMap::new()).unwrap_err();
            assert_eq!(e.kind, ErrorKind::Unsupported);
            assert_eq!((e.line, e.column), (0, 0));
            assert!(!e.to_string().contains("line 0"));
            assert!(e.to_string().len() <= 512);
            assert_eq!(s, before);
        }
    }
    #[test]
    fn reload_preserves_held_bindings_until_a_supported_renewal_updates_only_that_client() {
        let t = topology(None);
        let mut s =
            core::State::new(t.clone(), c::compile(TEXT, &t, &BTreeMap::new()).unwrap()).unwrap();
        let a = acquire(&mut s, "alpha-a");
        let b = acquire(&mut s, "beta-a");
        let updated = TEXT.replace("203.0.113.53", "203.0.113.54");
        s.replace_policy(c::compile(&updated, &t, &BTreeMap::new()).unwrap())
            .unwrap();
        assert_eq!(s.leases()["alpha-a"], a);
        assert_eq!(s.leases()["beta-a"], b);
        s.advance(300).unwrap();
        let core::Outcome::Ack {
            binding,
            kind: core::AckKind::Renewed,
        } = s.renew("alpha-a").unwrap()
        else {
            panic!("renewal ACK")
        };
        assert_eq!(binding.address, a.address);
        assert_eq!(binding.parameters.dns, [ip("203.0.113.54")]);
        assert_eq!(s.leases()["beta-a"], b);
    }
    #[test]
    fn supplied_relay_or_topology_limits_fail_before_an_invalid_model_can_be_published() {
        let t = topology(None);
        let relays = BTreeMap::from([(
            "foreign".into(),
            core::Relay {
                enabled: true,
                target: t.server,
                giaddr: ip("192.0.2.2"),
            },
        )]);
        assert_eq!(
            c::compile(TEXT, &t, &relays).unwrap_err().kind,
            ErrorKind::Invalid
        );
        let oversized = (0..9)
            .map(|n| {
                (
                    format!("relay-{n}"),
                    core::Relay {
                        enabled: true,
                        target: t.server,
                        giaddr: ip("192.0.2.2"),
                    },
                )
            })
            .collect();
        assert_eq!(
            c::compile(TEXT, &t, &oversized).unwrap_err().kind,
            ErrorKind::Limit
        );
        let mut bad = t.clone();
        bad.clients.get_mut("alpha-a").unwrap().segment = "missing".into();
        assert_eq!(
            c::compile(TEXT, &bad, &BTreeMap::new()).unwrap_err().kind,
            ErrorKind::Invalid
        );
    }
}
