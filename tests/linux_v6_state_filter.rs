//! Bounded state extension, atomic commands and original-row feedback.
use dlms_learning_lab::{
    linux_v6_connections::Class as State,
    linux_v6_filter::{Chain, Class, Packet, Protocol, Rule, Verdict},
    linux_v6_state_filter::{Match, Table},
};
fn packet() -> Packet {
    Packet {
        source: "fd20:30:40::5".parse().unwrap(),
        destination: "2001:db8:50::42".parse().unwrap(),
        ingress: Some("lan0".into()),
        egress: Some("wan0".into()),
        protocol: Protocol::Tcp,
        source_port: Some(40000),
        destination_port: Some(443),
        icmp_type: None,
        icmp_code: None,
    }
}
fn rule(text: &str) -> Rule {
    let mut t = dlms_learning_lab::linux_v6_filter::Table::new(Verdict::Drop);
    t.command(&format!("ip6tables -A FORWARD {text} -j ACCEPT"))
        .unwrap();
    t.forward.rules.remove(0)
}
#[test]
fn linux_v6_state_matches_original_and_observed_reply_without_inventing_icmp_state() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -p tcp -m conntrack --ctstate ESTABLISHED -j ACCEPT")
        .unwrap();
    t.command("ip6tables -A FORWARD -i lan0 -o wan0 -p tcp --dport 443 -m conntrack --ctstate NEW -j ACCEPT").unwrap();
    let p = packet();
    assert_eq!(
        t.evaluate(Chain::Forward, &p, Some(State::New))
            .unwrap()
            .rule,
        Some(2)
    );
    let q = Packet {
        source: p.destination,
        destination: p.source,
        ingress: p.egress.clone(),
        egress: p.ingress.clone(),
        source_port: p.destination_port,
        destination_port: p.source_port,
        ..p.clone()
    };
    assert_eq!(
        t.evaluate(Chain::Forward, &q, Some(State::New))
            .unwrap()
            .verdict,
        Verdict::Drop
    );
    assert_eq!(
        t.evaluate(Chain::Forward, &q, Some(State::Established))
            .unwrap()
            .rule,
        Some(1)
    );
    assert!(t.evaluate(Chain::Forward, &p, None).is_err());
    let icmp = Packet {
        protocol: Protocol::Icmp6,
        source_port: None,
        destination_port: None,
        icmp_type: Some(2),
        icmp_code: Some(0),
        ..p
    };
    assert!(
        t.evaluate(Chain::Forward, &icmp, Some(State::Established))
            .is_err()
    );
    assert_eq!(
        t.evaluate(Chain::Forward, &icmp, None).unwrap().verdict,
        Verdict::Drop
    );
    t.command("ip6tables -A FORWARD -p ipv6-icmp --icmpv6-type 2/0 -j ACCEPT")
        .unwrap();
    assert_eq!(
        t.evaluate(Chain::Forward, &icmp, None).unwrap().rule,
        Some(3)
    );
}
#[test]
fn linux_v6_state_union_deduplication_and_stateless_rules_keep_first_match_order() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -p tcp -m conntrack --ctstate ESTABLISHED,NEW,NEW -j ACCEPT")
        .unwrap();
    assert_eq!(t.forward, [Some(Match::Both)]);
    for s in [State::New, State::Established] {
        assert_eq!(
            t.evaluate(Chain::Forward, &packet(), Some(s)).unwrap().rule,
            Some(1)
        );
    }
    t.command("ip6tables -I FORWARD 1 -p tcp -j DROP").unwrap();
    for s in [State::New, State::Established] {
        let d = t.evaluate(Chain::Forward, &packet(), Some(s)).unwrap();
        assert_eq!((d.rule, d.verdict), (Some(1), Verdict::Drop));
    }
}
#[test]
fn linux_v6_state_all_edit_operations_preserve_mask_alignment_and_listing_roundtrip() {
    let mut t = Table::new(Verdict::Drop);
    for command in [
        "ip6tables -t filter --append FORWARD -p tcp --match conntrack --ctstate NEW --jump ACCEPT",
        "ip6tables --insert FORWARD -p udp -m conntrack --ctstate ESTABLISHED -j DROP",
        "ip6tables -I FORWARD 2 -p tcp -j DROP",
        "ip6tables -A INPUT -p tcp -m conntrack --ctstate NEW -j ACCEPT",
        "ip6tables -A OUTPUT -p udp -m conntrack --ctstate NEW -j ACCEPT",
        "ip6tables --policy FORWARD ACCEPT",
    ] {
        t.command(command).unwrap();
        t.validate().unwrap();
    }
    assert_eq!(
        t.forward,
        [Some(Match::Established), None, Some(Match::New)]
    );
    let text = t.command("ip6tables -S").unwrap().text;
    let mut restored = Table::new(Verdict::Accept);
    for line in text.lines() {
        restored.command(&format!("ip6tables {line}")).unwrap();
    }
    assert_eq!(restored, t);
    t.command("ip6tables --delete FORWARD 2").unwrap();
    assert_eq!(t.forward, [Some(Match::Established), Some(Match::New)]);
    t.command("ip6tables --flush FORWARD").unwrap();
    assert!(t.forward.is_empty());
    assert_eq!(t.input.len(), 1);
    assert_eq!(t.output.len(), 1);
    t.command("ip6tables -F").unwrap();
    assert!(t.input.is_empty() && t.output.is_empty());
    t.validate().unwrap();
}
#[test]
fn linux_v6_state_unsupported_malformed_and_injected_values_fail_atomically() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -p tcp -j DROP").unwrap();
    for command in [
        "ip6tables -A FORWARD -m conntrack --ctstate NEW -j ACCEPT",
        "ip6tables -A FORWARD -p ipv6-icmp -m conntrack --ctstate ESTABLISHED -j ACCEPT",
        "ip6tables -A FORWARD -p all -m conntrack --ctstate NEW -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack --ctstate RELATED -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack --ctstate DNAT -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack --ctstate NEW, -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack --ctstate '' -j ACCEPT",
        "ip6tables -A FORWARD -p tcp --ctstate NEW -m conntrack -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack --ctstate NEW --ctstate ESTABLISHED -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack -m conntrack --ctstate NEW -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m state --state NEW -j ACCEPT",
        "ip6tables -S -m conntrack --ctstate NEW",
        "ip6tables -P FORWARD ACCEPT -m conntrack --ctstate NEW",
        "ip6tables -D FORWARD 0",
        "ip6tables -I FORWARD 0 -p tcp -j ACCEPT",
        "ip6tables -I FORWARD 99 -p tcp -j ACCEPT",
        "ip6tables -D FORWARD 99",
        "ip6tables -A FORWARD -s '-m conntrack --ctstate NEW' -p tcp -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -j 'ACCEPT -m conntrack --ctstate NEW'",
        "ip6tables -A FORWARD -p tcp -j ACCEPT; touch /anything",
        "ip6tables -A FORWARD -p tcp -j $(anything)",
        "ip6tables -t nat -A FORWARD -p tcp -m conntrack --ctstate NEW -j ACCEPT",
    ] {
        let before = t.clone();
        assert!(t.command(command).is_err(), "{command}");
        assert_eq!(t, before, "{command}");
    }
}
#[test]
fn linux_v6_state_saved_mask_bounds_and_protocol_mismatches_are_rejected() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -p tcp -m conntrack --ctstate NEW -j ACCEPT")
        .unwrap();
    let restored: Table = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(t, restored);
    let mut bad = t.clone();
    bad.forward.clear();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.base.forward.rules[0].protocol = Some(Protocol::Icmp6);
    assert_eq!(bad.validate().unwrap_err().class, Class::Unsupported);
    let mut v = serde_json::to_value(&t).unwrap();
    v["forward"] = serde_json::json!(vec![None::<Match>; 97]);
    assert!(serde_json::from_value::<Table>(v).is_err());
    for _ in 1..96 {
        t.command("ip6tables -A FORWARD -p tcp -m conntrack --ctstate NEW -j ACCEPT")
            .unwrap();
    }
    let before = t.clone();
    assert!(
        t.command("ip6tables -A FORWARD -p tcp -m conntrack --ctstate NEW -j ACCEPT")
            .is_err()
    );
    assert_eq!(t, before);
}
#[test]
fn linux_v6_state_exact_comparison_rechecks_actual_live_rule_after_inactive_masks() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -p tcp -m conntrack --ctstate ESTABLISHED -j DROP")
        .unwrap();
    t.command("ip6tables -A FORWARD -p tcp -d 2001:db8:50::42 --dport 443 -m conntrack --ctstate NEW -j ACCEPT").unwrap();
    let domain = rule("-p tcp");
    let allowed = rule("-p tcp -d 2001:db8:50::42 --dport 443");
    let names = vec!["lan0".into(), "wan0".into()];
    assert!(
        t.compare(
            Chain::Forward,
            Some(State::New),
            &names,
            std::slice::from_ref(&domain),
            std::slice::from_ref(&allowed)
        )
        .unwrap()
        .is_none()
    );
    t.command("ip6tables -A FORWARD -p tcp -j ACCEPT").unwrap();
    let witness = t
        .compare(
            Chain::Forward,
            Some(State::New),
            &names,
            &[domain],
            &[allowed],
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        (witness.rule, witness.actual, witness.expected),
        (Some(3), Verdict::Accept, Verdict::Drop)
    );
    assert_eq!(
        t.evaluate(Chain::Forward, &witness.packet, Some(State::New))
            .unwrap()
            .rule,
        Some(3)
    );
}
#[test]
fn linux_v6_state_exact_split_prefix_alternative_and_comparison_domains_are_honest() {
    let mut t = Table::new(Verdict::Drop);
    for subnet in ["fd20:30:40::/65", "fd20:30:40:0:8000::/65"] {
        t.command(&format!("ip6tables -A FORWARD -s {subnet} -p udp --dport 53 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT")).unwrap();
    }
    let d = rule("-p udp");
    let allowed = rule("-s fd20:30:40::/64 -p udp --dport 53");
    let names = vec!["lan0".into(), "wan0".into()];
    for state in [State::New, State::Established] {
        assert!(
            t.compare(
                Chain::Forward,
                Some(state),
                &names,
                std::slice::from_ref(&d),
                std::slice::from_ref(&allowed)
            )
            .unwrap()
            .is_none()
        );
    }
    assert!(
        t.compare(Chain::Forward, None, &names, &[d], &[allowed])
            .is_err()
    );
    let icmp = rule("-p ipv6-icmp");
    assert!(
        t.compare(
            Chain::Forward,
            Some(State::New),
            &names,
            std::slice::from_ref(&icmp),
            &[]
        )
        .is_err()
    );
    assert!(
        t.compare(Chain::Forward, None, &names, &[icmp], &[])
            .unwrap()
            .is_none()
    );
    assert!(
        t.compare(Chain::Forward, Some(State::New), &names, &[rule("")], &[])
            .is_err()
    );
}

#[test]
fn linux_v6_state_flag_like_interface_is_one_literal_value_not_an_extension_option() {
    let mut t = Table::new(Verdict::Drop);
    t.command("ip6tables -A FORWARD -i '--ctstate' -p tcp -m conntrack --ctstate NEW -j ACCEPT")
        .unwrap();
    assert_eq!(
        t.base.forward.rules[0].ingress.as_deref(),
        Some("--ctstate")
    );
    assert_eq!(t.forward, [Some(Match::New)]);
    let p = packet();
    assert_eq!(
        t.evaluate(Chain::Forward, &p, Some(State::New))
            .unwrap()
            .verdict,
        Verdict::Drop
    );
    let mut literal = p;
    literal.ingress = Some("--ctstate".into());
    assert_eq!(
        t.evaluate(Chain::Forward, &literal, Some(State::New))
            .unwrap()
            .verdict,
        Verdict::Accept
    );
    assert_eq!(
        t.evaluate(Chain::Forward, &literal, Some(State::Established))
            .unwrap()
            .verdict,
        Verdict::Drop
    );
    let text = t.listing(None).unwrap();
    let mut restored = Table::new(Verdict::Accept);
    for line in text.lines() {
        restored.command(&format!("ip6tables {line}")).unwrap();
    }
    assert_eq!(t, restored);
}
