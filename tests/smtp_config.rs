use dlms_learning_lab::smtp_config::{Config, ErrorKind, Ipv4Prefix, RelayDecision};
fn file() -> String {
    "myhostname = gate.lab.test\nmydestination = gate.lab.test, localhost\nmynetworks = 127.0.0.0/8, 192.0.2.0/24\nrelay_domains = staff.lab.test\nrelayhost = [delivery.lab.test]\nsmtpd_relay_restrictions = permit_mynetworks, reject_unauth_destination\nparent_domain_matches_subdomains =\n".into()
}
fn ip(value: &str) -> std::net::Ipv4Addr {
    value.parse().unwrap()
}
#[test]
fn ordered_relay_policy_distinguishes_trusted_outgoing_local_and_untrusted_relay() {
    let c = Config::parse(&file()).unwrap();
    for (client, target, wanted) in [
        ("192.0.2.40", "outside.test", RelayDecision::Permit),
        ("203.0.113.9", "gate.lab.test", RelayDecision::Permit),
        ("203.0.113.9", "staff.lab.test", RelayDecision::Permit),
        ("203.0.113.9", "outside.test", RelayDecision::Reject),
        ("203.0.113.9", "sub.staff.lab.test", RelayDecision::Reject),
    ] {
        assert_eq!(c.relay_decision(ip(client), target).unwrap(), wanted);
    }
    let wrong = Config::parse(&file().replace(
        "permit_mynetworks, reject_unauth_destination",
        "reject_unauth_destination, permit_mynetworks",
    ))
    .unwrap();
    assert_eq!(
        wrong
            .relay_decision(ip("192.0.2.40"), "outside.test")
            .unwrap(),
        RelayDecision::Reject
    );
}
#[test]
fn broad_trust_is_operationally_wrong_and_authorized_destinations_do_not_short_circuit_following_reject()
 {
    let c = Config::parse(&file().replace("192.0.2.0/24", "0.0.0.0/0")).unwrap();
    assert_eq!(
        c.relay_decision(ip("203.0.113.9"), "outside.test").unwrap(),
        RelayDecision::Permit,
        "An open trust boundary must not be silently repaired by the parser"
    );
    let c = Config::parse(&file().replace(
        "permit_mynetworks, reject_unauth_destination",
        "reject_unauth_destination, reject",
    ))
    .unwrap();
    assert_eq!(
        c.relay_decision(ip("203.0.113.9"), "gate.lab.test")
            .unwrap(),
        RelayDecision::Reject
    );
    let c = Config::parse(&file().replace("reject_unauth_destination", "defer_unauth_destination"))
        .unwrap();
    assert_eq!(
        c.relay_decision(ip("203.0.113.9"), "outside.test").unwrap(),
        RelayDecision::Defer
    );
}
#[test]
fn comments_continuations_last_definition_and_original_values_have_defined_behavior() {
    let f=file().replace("mydestination = gate.lab.test, localhost","# Source comment\nmydestination = wrong.test\nmydestination = GATE.LAB.TEST.,\n    localhost\n    # A comment does not become a list item\n");
    let c = Config::parse(&f).unwrap();
    assert_eq!(c.warnings.len(), 1);
    assert!(c.destinations.contains("gate.lab.test."));
    assert!(!c.destinations.contains("gate.lab.test"));
    assert!(c.destinations.contains("localhost"));
    assert!(!c.destinations.contains("wrong.test"));
    assert_eq!(c.parameters["mydestination"], "GATE.LAB.TEST., localhost");
    assert!(c.warnings[0].contains("overrides"));
}
#[test]
fn literal_destination_patterns_keep_their_final_dot_but_recipient_domains_drop_one() {
    let c = Config::parse(
        &file()
            .replace("gate.lab.test, localhost", "GATE.LAB.TEST., localhost")
            .replace(
                "relay_domains = staff.lab.test",
                "relay_domains = STAFF.LAB.TEST.",
            ),
    )
    .unwrap();
    assert!(c.destinations.contains("gate.lab.test."));
    assert!(c.relay_domains.contains("staff.lab.test."));
    for domain in [
        "gate.lab.test",
        "gate.lab.test.",
        "staff.lab.test",
        "staff.lab.test.",
    ] {
        assert_eq!(
            c.relay_decision(ip("203.0.113.9"), domain).unwrap(),
            RelayDecision::Reject
        );
    }
    let ordinary = Config::parse(&file()).unwrap();
    assert_eq!(
        ordinary
            .relay_decision(ip("203.0.113.9"), "gate.lab.test.")
            .unwrap(),
        RelayDecision::Permit
    );
    assert_eq!(
        dlms_learning_lab::smtp_routing::Recipient::parse("nora@staff.lab.test.")
            .unwrap()
            .domain,
        "staff.lab.test"
    );
    assert!(dlms_learning_lab::smtp_routing::Recipient::parse("nora@staff.lab.test..").is_err());
}
#[test]
fn relay_targets_have_explicit_brackets_addresses_and_ports_without_dns_or_network() {
    for (value, host, port) in [
        ("[delivery.lab.test]", "delivery.lab.test", 25),
        ("[192.0.2.20]:2525", "192.0.2.20", 2525),
    ] {
        let c = Config::parse(&file().replace("[delivery.lab.test]", value)).unwrap();
        let relay = c.relay.unwrap();
        assert_eq!(relay.host, host);
        assert_eq!(relay.port, port);
    }
    for value in [
        "[host.test]:0",
        "[host.test]:65536",
        "[host.test]more",
        "[host.test",
    ] {
        assert_eq!(
            Config::parse(&file().replace("[delivery.lab.test]", value))
                .unwrap_err()
                .kind,
            ErrorKind::Invalid
        );
    }
    for value in ["delivery.lab.test", "[::1]"] {
        assert_eq!(
            Config::parse(&file().replace("[delivery.lab.test]", value))
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn every_ipv4_prefix_and_both_address_extremes_match_an_independent_range_oracle() {
    for prefix in 0..=32u8 {
        let span = 1u64 << (32 - prefix);
        let at = u64::from(u32::from(ip("198.51.100.217")));
        let first = (at / span) * span;
        let range = Ipv4Prefix::parse(&format!(
            "{}/{prefix}",
            std::net::Ipv4Addr::from(first as u32)
        ))
        .unwrap();
        assert_eq!(range.prefix(), prefix);
        assert_eq!(u64::from(u32::from(range.network())), first);
        for candidate in [0, first, first + span - 1, u32::MAX as u64] {
            assert_eq!(
                range.contains(std::net::Ipv4Addr::from(candidate as u32)),
                (first..first + span).contains(&candidate)
            );
        }
        if first > 0 {
            assert!(!range.contains(std::net::Ipv4Addr::from((first - 1) as u32)));
        }
        if first + span <= u32::MAX as u64 {
            assert!(!range.contains(std::net::Ipv4Addr::from((first + span) as u32)));
        }
    }
    assert_eq!(Ipv4Prefix::parse("192.0.2.9").unwrap().prefix(), 32);
    for bad in [
        "192.0.2.1/24",
        "192.0.2.0/33",
        "192.0.2.0/-1",
        "[::1]/128",
        "999.2.3.4/24",
    ] {
        assert!(Ipv4Prefix::parse(bad).is_err());
    }
}
#[test]
fn unsupported_settings_variables_patterns_and_missing_defaults_fail_honestly() {
    for change in [
        file() + "inet_interfaces = all\n",
        file().replace("gate.lab.test, localhost", "$myhostname, localhost"),
        file().replace(
            "relay_domains = staff.lab.test",
            "relay_domains = .staff.lab.test",
        ),
        file().replace(
            "parent_domain_matches_subdomains =",
            "parent_domain_matches_subdomains = relay_domains",
        ),
        file().replace("myhostname = gate.lab.test\n", ""),
        file().replace("gate.lab.test, localhost", "gate.lab.test # comment"),
        file().replace("reject_unauth_destination", "permit_sasl_authenticated"),
    ] {
        assert_eq!(
            Config::parse(&change).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    assert_eq!(
        Config::parse(&file().replace(", reject_unauth_destination", ""))
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
}
#[test]
fn invalid_and_oversized_text_cannot_create_a_configuration() {
    for value in [
        "  orphan continuation\n".to_owned(),
        file().replace("myhostname =", "myhostname "),
        file().replace("gate.lab.test, localhost", "broken..test"),
        file() + "\0",
    ] {
        assert_eq!(Config::parse(&value).unwrap_err().kind, ErrorKind::Invalid);
    }
    for value in [
        "x".repeat(8193),
        "# line\n".repeat(97),
        file().replace(
            "mynetworks = 127.0.0.0/8, 192.0.2.0/24",
            &format!("mynetworks = {}", ["192.0.2.0/24"; 17].join(",")),
        ),
    ] {
        assert_eq!(Config::parse(&value).unwrap_err().kind, ErrorKind::Limit);
    }
}

#[test]
fn broader_valid_forms_are_unsupported_instead_of_being_silently_reinterpreted() {
    for source in [
        file().replace("[delivery.lab.test]", "[delivery.lab.test]:smtp"),
        file().replace("192.0.2.0/24", "192.0.2.0/255.255.255.0"),
    ] {
        assert_eq!(
            Config::parse(&source).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn errors_identify_the_setting_and_limit_untrusted_diagnostic_text() {
    for (key, value) in [
        ("myhostname", "broken..name"),
        ("mynetworks", "192.0.2.9/24"),
        ("relayhost", "[host.test]:0"),
    ] {
        let mut source = file();
        let line = source
            .lines()
            .find(|l| l.starts_with(&format!("{key} =")))
            .unwrap()
            .to_owned();
        source = source.replace(&line, &format!("{key} = {value}"));
        let e = Config::parse(&source).unwrap_err();
        assert!(e.message.contains(key), "{e}");
    }
    let source = format!("{} = x\n{}", "é".repeat(2000), file());
    let e = Config::parse(&source).unwrap_err();
    assert!(e.message.len() <= dlms_learning_lab::smtp_config::MAX_NOTICE);
    assert!(e.message.ends_with("..."));
}
