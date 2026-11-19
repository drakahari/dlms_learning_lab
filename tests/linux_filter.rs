//! Mathematical/behavior checks, not a real-kernel oracle.
use dlms_learning_lab::linux_filter::*;
fn packet(last: u8, port: u16) -> Packet {
    Packet {
        source: format!("10.20.30.{last}").parse().unwrap(),
        destination: "198.51.100.9".parse().unwrap(),
        ingress: Some("lan0".into()),
        egress: Some("wan0".into()),
        protocol: Protocol::Tcp,
        source_port: Some(49152),
        destination_port: Some(port),
    }
}
#[test]
fn numeric_masks_normalize_cli_host_bits_and_handle_octet_boundaries_without_shift_overflow() {
    for (text, network, prefix) in [
        ("203.0.113.77/0", "0.0.0.0", 0),
        ("10.20.159.77/20", "10.20.144.0", 20),
        ("10.20.30.129/25", "10.20.30.128", 25),
        ("10.20.30.129/32", "10.20.30.129", 32),
    ] {
        let n = Cidr::parse(text).unwrap();
        assert_eq!(n.network.to_string(), network);
        assert_eq!(n.prefix, prefix);
        assert!(n.valid());
    }
    let n = Cidr::parse("10.20.159.77/20").unwrap();
    for ip in ["10.20.144.0", "10.20.159.255"] {
        assert!(n.contains(ip.parse().unwrap()));
    }
    for ip in ["10.20.143.255", "10.20.160.0"] {
        assert!(!n.contains(ip.parse().unwrap()));
    }
    for bad in [
        "10.20.30.1/33",
        "10.20.30.1/-1",
        "10.20.30.1/+1",
        "10.20.30.999/24",
        "host.example/24",
    ] {
        assert!(Cidr::parse(bad).is_err());
    }
}
#[test]
fn matching_requires_the_whole_intersection_and_chain_and_default_are_distinct() {
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -i lan0 -o wan0 -s 10.20.30.128/25 -d 198.51.100.9/32 -p tcp --dport 443 -j ACCEPT").unwrap();
    for last in [0, 1, 127, 128, 129, 254, 255] {
        for port in [22, 80, 443, 444] {
            let p = packet(last, port);
            let d = t.evaluate(Chain::Forward, &p).unwrap();
            assert_eq!(
                d.verdict,
                if last >= 128 && port == 443 {
                    Verdict::Accept
                } else {
                    Verdict::Drop
                }
            );
            let mut local = p.clone();
            local.egress = None;
            assert_eq!(
                t.evaluate(Chain::Input, &local).unwrap().verdict,
                Verdict::Drop
            );
        }
    }
    for field in [0, 1, 2, 3] {
        let mut p = packet(129, 443);
        match field {
            0 => p.source = "10.21.30.129".parse().unwrap(),
            1 => p.destination = "198.51.100.10".parse().unwrap(),
            2 => p.ingress = Some("guest0".into()),
            _ => p.protocol = Protocol::Udp,
        };
        assert_eq!(
            t.evaluate(Chain::Forward, &p).unwrap().verdict,
            Verdict::Drop
        );
    }
}
#[test]
fn ordered_shadowing_insert_and_one_based_delete_really_change_the_selected_rule() {
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -j DROP").unwrap();
    let allow = "-p tcp --dport 443 -j ACCEPT";
    t.command(&format!("iptables -A FORWARD {allow}")).unwrap();
    let p = packet(129, 443);
    assert_eq!(
        t.evaluate(Chain::Forward, &p).unwrap(),
        Decision {
            verdict: Verdict::Drop,
            rule: Some(1)
        }
    );
    t.command(&format!("iptables -I FORWARD 1 {allow}"))
        .unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &p).unwrap(),
        Decision {
            verdict: Verdict::Accept,
            rule: Some(1)
        }
    );
    t.command("iptables -D FORWARD 1").unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &p).unwrap().verdict,
        Verdict::Drop
    );
    let before = t.clone();
    for bad in [
        "iptables -D FORWARD 0",
        "iptables -D FORWARD 4",
        "iptables -I FORWARD 9 -j ACCEPT",
        "iptables -I FORWARD 0 -j ACCEPT",
    ] {
        assert!(t.command(bad).is_err());
        assert_eq!(t, before);
    }
}
#[test]
fn canonical_listing_reparses_the_actual_policy_and_flush_preserves_default_policy() {
    let mut a = Table::new(Verdict::Drop);
    for c in [
        "iptables -P OUTPUT ACCEPT",
        "iptables -A INPUT -i lan0 -s 10.20.30.129/25 -p tcp --dport 0 -j ACCEPT",
        "iptables -A FORWARD -j DROP",
    ] {
        a.command(c).unwrap();
    }
    let before = a.clone();
    let out = a.command("iptables -t filter -S").unwrap();
    assert!(!out.changed);
    assert_eq!(a, before);
    let mut b = Table::new(Verdict::Accept);
    for line in out.text.lines() {
        b.command(&format!("iptables {line}")).unwrap();
    }
    assert_eq!(a, b);
    b.command("iptables -F INPUT").unwrap();
    assert!(b.input.rules.is_empty());
    assert_eq!(b.input.policy, Verdict::Drop);
    assert_eq!(b.forward, a.forward);
    assert_eq!(b.output, a.output);
    b.command("iptables -F").unwrap();
    assert!(b.forward.rules.is_empty());
    assert_eq!(b.output.policy, Verdict::Accept);
}
#[test]
fn unsupported_shell_extensions_and_malformed_rules_never_partially_change_policy() {
    let mut t = Table::new(Verdict::Drop);
    let before = t.clone();
    for raw in [
        "iptables -t nat -S",
        "ip6tables -S",
        "iptables -A FORWARD -m conntrack --ctstate ESTABLISHED -j ACCEPT",
        "iptables -A INPUT -s 10.0.0.0/8 -s 172.16.0.0/12 -j ACCEPT",
        "iptables -A INPUT -o wan0 -j ACCEPT",
        "iptables -A INPUT -p icmp --dport 80 -j ACCEPT",
        "iptables -A INPUT -i lan+ -j ACCEPT",
        "iptables -A INPUT -p tcp --dport 65536 -j ACCEPT",
        "iptables -A INPUT -j ACCEPT; echo changed",
        "iptables -S > /etc/hosts",
        "iptables -A INPUT -s $(whoami) -j ACCEPT",
        "iptables -A INPUT -s host.example -j ACCEPT",
        "iptables -A INPUT",
    ] {
        assert!(t.command(raw).is_err(), "{raw}");
        assert_eq!(t, before);
    }
    assert_eq!(
        t.command(&"x".repeat(2049)).unwrap_err().class,
        Class::Limit
    );
    assert_eq!(t, before);
}
#[test]
fn rule_budget_is_global_and_a_failed_add_keeps_all_previous_rules() {
    let mut t = Table::new(Verdict::Drop);
    for n in 0..MAX_RULES {
        let chain = if n % 2 == 0 { "INPUT" } else { "FORWARD" };
        t.command(&format!("iptables -A {chain} -p tcp --dport {n} -j ACCEPT"))
            .unwrap();
    }
    let before = t.clone();
    assert_eq!(
        t.command("iptables -A OUTPUT -j DROP").unwrap_err().class,
        Class::Limit
    );
    assert_eq!(t, before);
    t.command("iptables -D INPUT 1").unwrap();
    t.command("iptables -A OUTPUT -j DROP").unwrap();
}

#[test]
fn nonsensical_path_context_is_rejected_instead_of_silently_treating_transit_as_local() {
    let t = Table::new(Verdict::Accept);
    let p = packet(129, 443);
    assert_eq!(
        t.evaluate(Chain::Input, &p).unwrap_err().class,
        Class::Context
    );
    assert_eq!(
        t.evaluate(Chain::Output, &p).unwrap_err().class,
        Class::Context
    );
    let mut no_ports = p.clone();
    no_ports.destination_port = None;
    assert_eq!(
        t.evaluate(Chain::Forward, &no_ports).unwrap_err().class,
        Class::Context
    );
}

#[test]
fn literal_quotes_and_backslashes_group_arguments_without_expansion_or_execution() {
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A 'FORWARD' -s \"10.20.30.128/25\" -p tcp --dport '443' -j ACCEPT")
        .unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &packet(129, 443))
            .unwrap()
            .verdict,
        Verdict::Accept
    );
    let before = t.clone();
    for line in [
        "iptables -A FORWARD -s \"$network\" -j ACCEPT",
        "iptables -A FORWARD -i \"wan0",
        "iptables -A FORWARD --dport 443 -p tcp -j ACCEPT",
    ] {
        assert!(t.command(line).is_err());
        assert_eq!(t, before);
    }
}

#[test]
fn an_unimplemented_listing_is_unsupported_without_inventing_a_missing_chain() {
    let mut t = Table::new(Verdict::Drop);
    let before = t.clone();
    let e = t.command("iptables -L").unwrap_err();
    assert_eq!(e.class, Class::Unsupported);
    assert!(e.message.contains("Use -S"));
    assert_eq!(t, before);
}
