//! Pure source-owned fixtures: no host firewall/process/network comparison.
use dlms_learning_lab::{
    linux_v6_connections::Class as State,
    linux_v6_filter::{Chain, Class, Packet, Protocol, Verdict},
    linux_v6_saved_filter as file,
    linux_v6_state_filter::Table,
};
fn fixture() -> Table {
    let mut t = Table::new(Verdict::Drop);
    for line in [
        "ip6tables -P OUTPUT ACCEPT",
        "ip6tables -A FORWARD -i lan0 -o wan0 -s fd62:70:80::/64 -d 2001:db8:99::/64 -p tcp --dport 443 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT",
        "ip6tables -A FORWARD -i wan0 -o lan0 -p tcp --sport 443 -m conntrack --ctstate ESTABLISHED -j ACCEPT",
        "ip6tables -A INPUT -p ipv6-icmp --icmpv6-type 2/0 -j ACCEPT",
        "ip6tables -A FORWARD -p udp --dport 53 -j DROP",
    ] {
        t.command(line).unwrap();
    }
    t
}
fn packet() -> Packet {
    Packet {
        source: "fd62:70:80::20".parse().unwrap(),
        destination: "2001:db8:99::80".parse().unwrap(),
        ingress: Some("lan0".into()),
        egress: Some("wan0".into()),
        protocol: Protocol::Tcp,
        source_port: Some(41000),
        destination_port: Some(443),
        icmp_type: None,
        icmp_code: None,
    }
}
#[test]
fn tracked_ipv6_dump_roundtrips_all_rule_masks_policies_and_packet_outcomes() {
    let t = fixture();
    let text = file::save(&t).unwrap();
    assert!(
        text.starts_with("*filter\n:INPUT DROP [0:0]\n:FORWARD DROP [0:0]\n:OUTPUT ACCEPT [0:0]\n")
    );
    assert!(text.ends_with("COMMIT\n"));
    let loaded = file::restore(&text).unwrap();
    assert_eq!(t, loaded);
    for state in [Some(State::New), Some(State::Established)] {
        for port in [443, 444] {
            let mut p = packet();
            p.destination_port = Some(port);
            assert_eq!(
                t.evaluate(Chain::Forward, &p, state).unwrap(),
                loaded.evaluate(Chain::Forward, &p, state).unwrap()
            );
        }
    }
    // TCP/UDP require a supplied direction observation; absence is not a fake NEW.
    assert_eq!(
        t.evaluate(Chain::Forward, &packet(), None)
            .unwrap_err()
            .class,
        Class::Context
    );
    assert_eq!(
        loaded
            .evaluate(Chain::Forward, &packet(), None)
            .unwrap_err()
            .class,
        Class::Context
    );
    let mut control = packet();
    control.protocol = Protocol::Icmp6;
    control.source_port = None;
    control.destination_port = None;
    control.icmp_type = Some(2);
    control.icmp_code = Some(0);
    control.egress = None;
    assert_eq!(
        t.evaluate(Chain::Input, &control, None).unwrap(),
        loaded.evaluate(Chain::Input, &control, None).unwrap()
    );
    assert_eq!(
        loaded
            .evaluate(Chain::Input, &control, Some(State::New))
            .unwrap_err()
            .class,
        Class::Context
    );
    assert!(text.contains("--ctstate NEW,ESTABLISHED") && text.contains("--ctstate ESTABLISHED"));
}
#[test]
fn test_only_and_failed_restore_leave_the_live_table_exact() {
    let wanted = fixture();
    let text = file::save(&wanted).unwrap();
    let mut live = Table::new(Verdict::Accept);
    let old = live.clone();
    file::apply_restore(&mut live, &text, true).unwrap();
    assert_eq!(live, old);
    assert!(
        file::apply_restore(
            &mut live,
            &text.replace("COMMIT", "-A FORWARD -j ACCEPT\nunknown instruction"),
            false
        )
        .is_err()
    );
    assert_eq!(live, old);
    file::apply_restore(&mut live, &text, false).unwrap();
    assert_eq!(live, wanted);
}
#[test]
fn malformed_and_unsupported_files_do_not_return_partial_success() {
    let good = file::save(&fixture()).unwrap();
    let bad = [
        good.replace("COMMIT\n", ""),
        good.replace("*filter", "*nat"),
        good.replace(":INPUT DROP [0:0]", ":CUSTOM DROP [0:0]"),
        good.replace(":INPUT DROP [0:0]", ":INPUT DROP [1:64]"),
        good.replace(":OUTPUT ACCEPT [0:0]", ":INPUT DROP [0:0]"),
        good.replace("NEW,ESTABLISHED", "RELATED"),
        good.replace("-j ACCEPT", "-m comment --comment note -j ACCEPT"),
        good.replace("fd62:70:80::/64", "192.0.2.0/24"),
        format!("{good}*filter\nCOMMIT\n"),
        format!("{good}bash -c 'touch /etc/host'\n"),
        good.replace("COMMIT", "-P INPUT ACCEPT\nCOMMIT"),
        good.replace("COMMIT", ":INPUT ACCEPT [0:0]\nCOMMIT"),
    ];
    for text in bad {
        assert!(
            file::restore(&text).is_err(),
            "Must reject entire unsupported snapshot: {text}"
        );
    }
}
#[test]
fn harmless_comments_whitespace_policy_order_and_numeric_protocols_are_supported() {
    let t = fixture();
    let good = file::save(&t).unwrap();
    let decorated = good
        .replace(
            ":INPUT DROP [0:0]\n:FORWARD DROP [0:0]\n:OUTPUT ACCEPT [0:0]",
            ":OUTPUT ACCEPT [0:0]\n:FORWARD DROP [0:0]\n:INPUT DROP [0:0]",
        )
        .replace("-p tcp ", "-p 6 ")
        .replace("\n", "\r\n");
    let text = format!("# Disposable simulated rules\r\n\r\n{decorated}# End\r\n");
    assert_eq!(file::restore(&text).unwrap(), t);
}
#[test]
fn byte_line_rule_and_control_limits_fail_honestly() {
    let good = file::save(&fixture()).unwrap();
    for bad in [
        format!("{}{}", "#".repeat(file::MAX_BYTES), good),
        format!("{}{}", "\n".repeat(file::MAX_LINES), good),
        format!("{good}\0"),
        format!("{good}\x1b"),
        format!("{good}# non-ASCII é\n"),
    ] {
        assert_eq!(file::restore(&bad).unwrap_err().class, Class::Limit);
    }
    let mut many =
        String::from("*filter\n:INPUT DROP [0:0]\n:FORWARD DROP [0:0]\n:OUTPUT DROP [0:0]\n");
    for _ in 0..97 {
        many.push_str("-A INPUT -p tcp --dport 443 -j ACCEPT\n");
    }
    many.push_str("COMMIT\n");
    assert_eq!(file::restore(&many).unwrap_err().class, Class::Limit);
}
