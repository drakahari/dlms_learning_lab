//! Whole IPv6 predicate spaces with independent typed witness rechecks.
use dlms_learning_lab::{
    linux_v6_filter::{Chain, Class, Packet, Protocol, Rule, Table, Verdict},
    linux_v6_policy::compare_filter,
};
fn names() -> Vec<String> {
    vec!["lan0".into(), "wan0".into()]
}
fn rule(chain: &str, flags: &str) -> Rule {
    let mut t = Table::new(Verdict::Drop);
    t.command(&format!("ip6tables -A {chain} {flags} -j ACCEPT"))
        .unwrap();
    match chain {
        "INPUT" => t.input.rules.remove(0),
        "FORWARD" => t.forward.rules.remove(0),
        "OUTPUT" => t.output.rules.remove(0),
        _ => panic!(),
    }
}
fn domain() -> Rule {
    rule("FORWARD", "-i lan0 -o wan0 -p tcp")
}
fn web(source: &str) -> String {
    format!("-i lan0 -o wan0 -s {source} -d 2001:db8:99::42 -p tcp --dport 443")
}
fn allow() -> Rule {
    rule("FORWARD", &web("fd20:30:40::/64"))
}
fn typed(t: &Table, p: &Packet) -> Verdict {
    t.evaluate(Chain::Forward, p).unwrap().verdict
}
#[test]
fn linux_v6_complete_prefixes_and_equivalent_split_rules_pass_beyond_displayed_clients() {
    let mut t = Table::new(Verdict::Drop);
    for prefix in ["fd20:30:40::/65", "fd20:30:40:0:8000::/65"] {
        t.command(&format!("ip6tables -A FORWARD {} -j ACCEPT", web(prefix)))
            .unwrap();
    }
    assert!(
        compare_filter(&t, Chain::Forward, &names(), &[domain()], &[allow()])
            .unwrap()
            .is_none()
    );
    let reversed = vec!["wan0".into(), "lan0".into()];
    assert!(
        compare_filter(&t, Chain::Forward, &reversed, &[domain()], &[allow()])
            .unwrap()
            .is_none()
    );
}
#[test]
fn linux_v6_hardcoded_clients_low_address_bits_wrong_prefixes_and_default_accept_fail() {
    for flags in [
        web("fd20:30:40::55"),
        web("::55"),
        web("fd20:30:41::/64"),
        "-i lan0 -o wan0 -p tcp --dport 443".into(),
    ] {
        let mut t = Table::new(Verdict::Drop);
        t.command(&format!("ip6tables -A FORWARD {flags} -j ACCEPT"))
            .unwrap();
        let c = compare_filter(&t, Chain::Forward, &names(), &[domain()], &[allow()])
            .unwrap()
            .unwrap();
        assert_eq!(typed(&t, &c.packet), c.actual);
        assert_eq!(
            if allow().matches(&c.packet) {
                Verdict::Accept
            } else {
                Verdict::Drop
            },
            c.expected
        );
        assert_ne!(c.actual, c.expected);
    }
    let t = Table::new(Verdict::Accept);
    let c = compare_filter(&t, Chain::Forward, &names(), &[domain()], &[allow()])
        .unwrap()
        .unwrap();
    assert_eq!(c.actual, Verdict::Accept);
    assert_eq!(c.expected, Verdict::Drop);
    assert_eq!(c.rule, None);
}
#[test]
fn linux_v6_first_match_witness_reports_original_rule_number_even_after_unreachable_rule() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -i typo0 -j ACCEPT")
        .unwrap();
    t.command("ip6tables -A FORWARD -j ACCEPT").unwrap();
    let c = compare_filter(&t, Chain::Forward, &names(), &[domain()], &[allow()])
        .unwrap()
        .unwrap();
    assert_eq!(c.rule, Some(2));
    assert_eq!(typed(&t, &c.packet), c.actual);
    t.command("ip6tables -F FORWARD").unwrap();
    t.command("ip6tables -A FORWARD -s fd20:30:40::/64 -j DROP")
        .unwrap();
    t.command(&format!(
        "ip6tables -A FORWARD {} -j ACCEPT",
        web("fd20:30:40::/64")
    ))
    .unwrap();
    let c = compare_filter(&t, Chain::Forward, &names(), &[domain()], &[allow()])
        .unwrap()
        .unwrap();
    assert_eq!(c.rule, Some(1));
    assert_eq!(c.actual, Verdict::Drop);
    assert_eq!(c.expected, Verdict::Accept);
}
#[test]
fn linux_v6_icmp_codes_are_proved_separately_without_inventing_neighbor_success() {
    let d = rule("INPUT", "-i lan0 -p ipv6-icmp");
    let a = rule(
        "INPUT",
        "-i lan0 -s fe80::/10 -d ff02::1:ff00:0/104 -p ipv6-icmp --icmpv6-type 135/0",
    );
    let mut t = Table::new(Verdict::Drop);
    t.input.rules.push(a.clone());
    assert!(
        compare_filter(
            &t,
            Chain::Input,
            &names(),
            std::slice::from_ref(&d),
            std::slice::from_ref(&a)
        )
        .unwrap()
        .is_none()
    );
    t.input.rules[0].icmp_code = None;
    let c = compare_filter(&t, Chain::Input, &names(), &[d], std::slice::from_ref(&a))
        .unwrap()
        .unwrap();
    assert_eq!(c.packet.protocol, Protocol::Icmp6);
    assert_eq!(c.packet.icmp_type, Some(135));
    assert_ne!(c.packet.icmp_code, Some(0));
    assert!(c.packet.source_port.is_none());
    assert_eq!(
        t.evaluate(Chain::Input, &c.packet).unwrap().verdict,
        c.actual
    );
    assert!(!a.matches(&c.packet));
}
#[test]
fn linux_v6_input_output_and_protocol_worlds_keep_absent_devices_and_ports_correct() {
    for (chain, label, flags) in [
        (Chain::Input, "INPUT", "-i lan0 -p udp --dport 53"),
        (Chain::Output, "OUTPUT", "-o wan0 -p tcp --dport 443"),
    ] {
        let d = rule(
            label,
            if chain == Chain::Input {
                "-i lan0"
            } else {
                "-o wan0"
            },
        );
        let a = rule(label, flags);
        let mut t = Table::new(Verdict::Drop);
        t.command(&format!("ip6tables -A {label} {flags} -j ACCEPT"))
            .unwrap();
        assert!(
            compare_filter(
                &t,
                chain,
                &names(),
                std::slice::from_ref(&d),
                std::slice::from_ref(&a)
            )
            .unwrap()
            .is_none()
        );
        t.command(&format!("ip6tables -F {label}")).unwrap();
        let c = compare_filter(&t, chain, &names(), &[d], &[a])
            .unwrap()
            .unwrap();
        assert_eq!(c.packet.ingress.is_none(), chain == Chain::Output);
        assert_eq!(c.packet.egress.is_none(), chain == Chain::Input);
        assert!(c.packet.icmp_type.is_none());
        assert!(c.packet.source_port.is_some());
        assert_ne!(c.actual, c.expected);
    }
}
#[test]
fn linux_v6_source_destination_and_port_boundaries_match_independent_small_enumeration() {
    let base = u128::from("fd20:30:40::".parse::<std::net::Ipv6Addr>().unwrap());
    let target = u128::from("2001:db8:99::42".parse::<std::net::Ipv6Addr>().unwrap());
    let mut t = Table::new(Verdict::Drop);
    t.command(&format!(
        "ip6tables -A FORWARD {} -j ACCEPT",
        web("fd20:30:40::/126")
    ))
    .unwrap();
    let a = rule("FORWARD", &web("fd20:30:40::/126"));
    assert!(
        compare_filter(
            &t,
            Chain::Forward,
            &names(),
            &[domain()],
            std::slice::from_ref(&a)
        )
        .unwrap()
        .is_none()
    );
    for s in [base, base + 1, base + 3, base + 4, base + (1u128 << 64)] {
        for dest in [target - 1, target, target + 1] {
            for port in [442, 443, 444] {
                let p = Packet {
                    source: s.into(),
                    destination: dest.into(),
                    ingress: Some("lan0".into()),
                    egress: Some("wan0".into()),
                    protocol: Protocol::Tcp,
                    source_port: Some(40000),
                    destination_port: Some(port),
                    icmp_type: None,
                    icmp_code: None,
                };
                let expected = if (base..=base + 3).contains(&s) && dest == target && port == 443 {
                    Verdict::Accept
                } else {
                    Verdict::Drop
                };
                assert_eq!(typed(&t, &p), expected);
                assert_eq!(a.matches(&p), expected == Verdict::Accept);
            }
        }
    }
}
#[test]
fn linux_v6_invalid_requirement_context_and_limits_never_become_success() {
    let t = Table::new(Verdict::Drop);
    let d = domain();
    for n in [
        vec![],
        vec!["lan0".into(), "lan0".into()],
        vec!["bad device".into()],
        (0..9).map(|i| format!("eth{i}")).collect(),
    ] {
        assert_eq!(
            compare_filter(&t, Chain::Forward, &n, std::slice::from_ref(&d), &[])
                .unwrap_err()
                .class,
            Class::Context
        );
    }
    assert!(compare_filter(&t, Chain::Forward, &names(), &[], &[]).is_err());
    assert!(compare_filter(&t, Chain::Forward, &names(), &vec![d.clone(); 17], &[]).is_err());
    assert!(
        compare_filter(
            &t,
            Chain::Forward,
            &names(),
            std::slice::from_ref(&d),
            &vec![d.clone(); 33]
        )
        .is_err()
    );
    let mut bad = d.clone();
    bad.target = Verdict::Drop;
    assert!(
        compare_filter(
            &t,
            Chain::Forward,
            &names(),
            std::slice::from_ref(&bad),
            &[]
        )
        .is_err()
    );
    bad = d.clone();
    bad.ingress = Some("unknown0".into());
    assert!(compare_filter(&t, Chain::Forward, &names(), &[bad], &[]).is_err());
    bad = d;
    bad.icmp_type = Some(135);
    assert!(compare_filter(&t, Chain::Forward, &names(), &[bad], &[]).is_err());
}
