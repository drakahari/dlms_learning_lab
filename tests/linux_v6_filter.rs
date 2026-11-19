//! Supported numeric IPv6 filter behavior, not kernel/real packet acceptance.
use dlms_learning_lab::linux_v6_filter::{Chain, Cidr, Class, Packet, Protocol, Table, Verdict};
fn packet() -> Packet {
    Packet {
        source: "2001:db8:50::55".parse().unwrap(),
        destination: "fd20:30:40::42".parse().unwrap(),
        ingress: Some("wan0".into()),
        egress: Some("lan0".into()),
        protocol: Protocol::Tcp,
        source_port: Some(50000),
        destination_port: Some(443),
        icmp_type: None,
        icmp_code: None,
    }
}
#[test]
fn linux_v6_prefixes_retain_all_bits_normalize_host_bits_and_handle_zero_and_128() {
    for (input, address, prefix) in [
        ("::/0", "::", 0),
        ("fd20:30:40::42/64", "fd20:30:40::", 64),
        ("2001:db8::1", "2001:db8::1", 128),
        (
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff/128",
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
            128,
        ),
    ] {
        let n = Cidr::parse(input).unwrap();
        assert_eq!(n.network, address.parse::<std::net::Ipv6Addr>().unwrap());
        assert_eq!(n.prefix, prefix);
        assert!(n.valid());
        assert!(n.contains(n.network));
    }
    let n = Cidr::parse("fd20:30:40::/64").unwrap();
    assert!(n.contains("fd20:30:40::ffff".parse().unwrap()));
    assert!(!n.contains("2001:db8:40::ffff".parse().unwrap()));
    assert!(!n.contains("fd20:30:41::1".parse().unwrap()));
    for s in [
        "::/129",
        "::/-1",
        "::/256",
        "::/",
        "192.0.2.1/24",
        "host.test",
        "fe80::1%lan0",
    ] {
        assert!(Cidr::parse(s).is_err(), "{s}");
    }
}
#[test]
fn linux_v6_actual_first_match_uses_both_full_addresses_interfaces_and_ports() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -i wan0 -o lan0 -s 2001:db8:50::/64 -d fd20:30:40::42 -p tcp --dport 443 -j ACCEPT").unwrap();
    let p = packet();
    let decision = t.evaluate(Chain::Forward, &p).unwrap();
    assert_eq!(decision.verdict, Verdict::Accept);
    assert_eq!(decision.rule, Some(1));
    for field in [
        "source",
        "destination",
        "ingress",
        "egress",
        "protocol",
        "port",
    ] {
        let mut q = p.clone();
        match field {
            "source" => q.source = "2001:db8:51::55".parse().unwrap(),
            "destination" => q.destination = "fd20:30:41::42".parse().unwrap(),
            "ingress" => q.ingress = Some("lan0".into()),
            "egress" => q.egress = Some("wan0".into()),
            "protocol" => q.protocol = Protocol::Udp,
            "port" => q.destination_port = Some(8443),
            _ => unreachable!(),
        }
        assert_eq!(
            t.evaluate(Chain::Forward, &q).unwrap().verdict,
            Verdict::Drop,
            "{field}"
        );
    }
    t.command("ip6tables -I FORWARD -s 2001:db8:50::/64 -j DROP")
        .unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &p).unwrap().verdict,
        Verdict::Drop
    );
    t.command("ip6tables -D FORWARD 1").unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &p).unwrap().verdict,
        Verdict::Accept
    );
}
#[test]
fn linux_v6_icmp_type_without_code_is_distinct_from_an_exact_type_code() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A INPUT -i lan0 -p ipv6-icmp --icmpv6-type 135 -j ACCEPT")
        .unwrap();
    let mut p = packet();
    p.ingress = Some("lan0".into());
    p.egress = None;
    p.protocol = Protocol::Icmp6;
    p.source_port = None;
    p.destination_port = None;
    p.icmp_type = Some(135);
    p.icmp_code = Some(0);
    for code in [0, 1, 255] {
        p.icmp_code = Some(code);
        assert_eq!(
            t.evaluate(Chain::Input, &p).unwrap().verdict,
            Verdict::Accept
        );
    }
    // Header matching alone does not say such a nonzero-code ND packet is valid.
    t.command("ip6tables -F INPUT").unwrap();
    t.command("ip6tables -A INPUT -i lan0 -p icmpv6 --icmpv6-type 135/0 -j ACCEPT")
        .unwrap();
    p.icmp_code = Some(0);
    assert_eq!(
        t.evaluate(Chain::Input, &p).unwrap().verdict,
        Verdict::Accept
    );
    p.icmp_code = Some(1);
    assert_eq!(t.evaluate(Chain::Input, &p).unwrap().verdict, Verdict::Drop);
    p.icmp_type = Some(134);
    p.icmp_code = Some(0);
    assert_eq!(t.evaluate(Chain::Input, &p).unwrap().verdict, Verdict::Drop);
    p.source_port = Some(80);
    assert_eq!(
        t.evaluate(Chain::Input, &p).unwrap_err().class,
        Class::Context
    );
}
#[test]
fn linux_v6_listing_roundtrips_long_options_numeric_protocols_and_icmp_codes() {
    let mut t = Table::new(Verdict::Accept);
    for command in [
        "ip6tables --table filter --policy FORWARD DROP",
        "ip6tables --append FORWARD --source fd20:30:40::99/64 --destination 2001:db8::42 --in-interface lan0 --out-interface wan0 --protocol 6 --destination-port 443 --jump ACCEPT",
        "ip6tables -A INPUT -p 58 --icmpv6-type 128/0 -j ACCEPT",
        "ip6tables -A OUTPUT -p 17 --sport 53 -j ACCEPT",
    ] {
        t.command(command).unwrap();
    }
    let text = t.command("ip6tables -S").unwrap();
    assert!(!text.changed);
    assert!(text.text.contains("fd20:30:40::/64"));
    let mut restored = Table::new(Verdict::Drop);
    for line in text.text.lines() {
        restored.command(&format!("ip6tables {line}")).unwrap();
    }
    assert_eq!(restored, t);
    let before = t.clone();
    assert!(!t.command("ip6tables -t filter -S INPUT").unwrap().changed);
    assert_eq!(before, t);
}
#[test]
fn linux_v6_unsupported_syntax_bad_values_and_quoted_flags_never_mutate_the_table() {
    let mut t = Table::new(Verdict::Drop);
    let before = t.clone();
    for command in [
        "iptables -A FORWARD -j ACCEPT",
        "sudo ip6tables -F",
        "ip6tables -t nat -F",
        "ip6tables -A FORWARD -p icmp -j ACCEPT",
        "ip6tables -A INPUT -p ipv6-icmp --icmpv6-type neighbour-solicitation -j ACCEPT",
        "ip6tables -A INPUT -p ipv6-icmp --icmpv6-type 256/0 -j ACCEPT",
        "ip6tables -A INPUT -p ipv6-icmp --icmpv6-type 135/256 -j ACCEPT",
        "ip6tables -A INPUT -p tcp --icmpv6-type 135 -j ACCEPT",
        "ip6tables -A INPUT -o wan0 -j ACCEPT",
        "ip6tables -A OUTPUT -i wan0 -j ACCEPT",
        "ip6tables -A FORWARD -p tcp --dport 80:90 -j ACCEPT",
        "ip6tables -A FORWARD --dport 443 -p tcp -j ACCEPT",
        "ip6tables -A FORWARD -m conntrack --ctstate ESTABLISHED -j ACCEPT",
        "ip6tables -A FORWARD -s '::/0 -j ACCEPT' -j DROP",
        "ip6tables -A FORWARD -s '-t' filter -j ACCEPT",
        "ip6tables -A FORWARD -j 'ACCEPT -t filter'",
        "ip6tables -P FORWARD ACCEPT; touch private",
        "ip6tables -P FORWARD ACCEPT | sh",
        "ip6tables -P FORWARD $(touch private)",
        "ip6tables -A FORWARD -s ::/0 -s ::/0 -j ACCEPT",
        "ip6tables -D FORWARD 0",
    ] {
        assert!(t.command(command).is_err(), "{command}");
        assert_eq!(t, before, "{command}");
    }
    // A flag-like literal interface stays an interface, not a global option.
    t.command("ip6tables -A FORWARD -i '-t' -j ACCEPT").unwrap();
    assert_eq!(t.forward.rules[0].ingress.as_deref(), Some("-t"));
}
#[test]
fn linux_v6_rule_storage_commands_and_json_validation_stay_bounded() {
    let mut t = Table::new(Verdict::Drop);
    for _ in 0..96 {
        t.command("ip6tables -A FORWARD -p udp --dport 53 -j ACCEPT")
            .unwrap();
    }
    let before = t.clone();
    assert_eq!(
        t.command("ip6tables -A INPUT -j ACCEPT").unwrap_err().class,
        Class::Limit
    );
    assert_eq!(t, before);
    assert_eq!(
        t.command(&"x".repeat(2049)).unwrap_err().class,
        Class::Limit
    );
    assert_eq!(t, before);
    let mut raw = serde_json::to_value(&t).unwrap();
    let extra = raw["forward"]["rules"][0].clone();
    raw["forward"]["rules"].as_array_mut().unwrap().push(extra);
    assert!(serde_json::from_value::<Table>(raw).is_err());
    let mut raw = serde_json::to_value(Table::new(Verdict::Drop)).unwrap();
    raw["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Table>(raw).is_err());
}
#[test]
fn linux_v6_malformed_saved_prefixes_and_protocol_combinations_cannot_be_evaluated() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -s fd20:30:40::/64 -p tcp --dport 443 -j ACCEPT")
        .unwrap();
    let mut wrong = t.clone();
    wrong.forward.rules[0].source.as_mut().unwrap().network = "fd20:30:40::1".parse().unwrap();
    assert!(wrong.validate().is_err());
    assert!(wrong.evaluate(Chain::Forward, &packet()).is_err());
    wrong = t.clone();
    wrong.forward.rules[0].icmp_type = Some(128);
    assert!(wrong.validate().is_err());
    wrong = t.clone();
    wrong.forward.rules[0].icmp_code = Some(0);
    assert!(wrong.validate().is_err());
    let mut p = packet();
    p.icmp_type = Some(128);
    assert!(t.evaluate(Chain::Forward, &p).is_err());
}
