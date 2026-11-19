//! Router administration on disposable typed fixtures; no host network or kernel calls.
use dlms_learning_lab::{
    ipv4_routing::{Assignment, Prefix},
    linux_filter::{Class, Protocol, Table, Verdict},
    linux_host as host,
    linux_router_admin::{self as admin, FILTER_FILE, FORWARD_FILE, State},
    linux_router_path::{Gate, Header, Origin},
};
fn ip(s: &str) -> std::net::Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
fn state() -> State {
    let profile = host::Profile {
        ipv6: None,
        id: "router-admin-fixture".into(),
        links: vec![
            host::Link {
                name: "lan0".into(),
                network: prefix("10.20.30.0/24"),
                carrier: true,
            },
            host::Link {
                name: "wan0".into(),
                network: prefix("198.51.100.0/24"),
                carrier: true,
            },
        ],
        gateways: vec![host::Gateway {
            address: ip("198.51.100.254"),
            device: "wan0".into(),
            available: true,
            exports: vec![prefix("203.0.113.0/24")],
        }],
        nodes: vec![host::Node {
            address: ip("203.0.113.9"),
            available: true,
        }],
        resolvers: vec![],
    };
    let config = host::Configuration {
        ipv6: None,
        interfaces: vec![
            host::Interface {
                name: "lan0".into(),
                up: true,
                address: Some(Assignment {
                    address: ip("10.20.30.1"),
                    prefix: 24,
                }),
            },
            host::Interface {
                name: "wan0".into(),
                up: true,
                address: Some(Assignment {
                    address: ip("198.51.100.1"),
                    prefix: 24,
                }),
            },
        ],
        routes: vec![],
        nameservers: vec![],
    };
    let filter = Table::new(Verdict::Drop);
    let result = State {
        host: host::State::new(profile, config).unwrap(),
        filter_file: admin::save_filter(&filter).unwrap(),
        filter,
        forwarding: false,
        forward_file: "# Saved forwarding setting\nnet.ipv4.ip_forward = 0\n".into(),
    };
    result.validate().unwrap();
    result
}
fn header() -> Header {
    Header {
        source: ip("10.20.30.55"),
        destination: ip("203.0.113.9"),
        protocol: Protocol::Tcp,
        source_port: Some(49152),
        destination_port: Some(443),
    }
}
fn ready(s: &mut State) {
    for line in [
        "ip -4 route add default via 198.51.100.254 dev wan0 metric 10",
        "sysctl -w net.ipv4.ip_forward=1",
        "iptables -A FORWARD -i lan0 -o wan0 -s 10.20.30.0/24 -p tcp --dport 443 -j ACCEPT",
    ] {
        s.command(line).unwrap();
    }
}
#[test]
fn route_commands_reuse_host_behavior_without_replacing_a_transit_source_address() {
    let mut s = state();
    let h = header();
    let o = Origin::Incoming("lan0".into());
    assert_eq!(s.evaluate(&o, &h).unwrap().gate, Gate::NoRoute);
    ready(&mut s);
    let trace = s.evaluate(&o, &h).unwrap();
    assert_eq!(trace.gate, Gate::Allowed);
    assert_eq!(trace.route.unwrap().via, Some(ip("198.51.100.254")));
    let before = s.clone();
    let shown = s.command("ip -4 route show").unwrap();
    assert!(!shown.changed);
    assert!(shown.text.contains("198.51.100.254"));
    assert_eq!(s, before);
    s.command("ip link set dev wan0 down").unwrap();
    assert_eq!(s.evaluate(&o, &h).unwrap().gate, Gate::EgressDown);
    s.command("ip link set dev wan0 up").unwrap();
    assert_eq!(s.evaluate(&o, &h).unwrap().gate, Gate::Allowed);
    let previous = s.clone();
    for line in [
        "ip -4 route replace default via 203.0.113.7 dev wan0 metric 10",
        "ip -4 address flush dev lan0",
        "ip link set dev 'wan0 up' down",
    ] {
        assert!(s.command(line).is_err());
        assert_eq!(s, previous);
    }
}
#[test]
fn stdout_export_save_dry_test_and_restore_have_distinct_effects() {
    let mut s = state();
    ready(&mut s);
    let active = s.filter.clone();
    let initial_file = s.filter_file.clone();
    let export = s.command("iptables-save -t filter").unwrap();
    assert!(!export.changed);
    assert!(export.text.starts_with("*filter\n:INPUT DROP [0:0]"));
    assert!(export.text.ends_with("COMMIT\n"));
    assert_eq!(s.filter_file, initial_file);
    s.command(&format!("iptables-save -t filter -f {FILTER_FILE}"))
        .unwrap();
    assert_eq!(s.filter_file, export.text);
    s.command("iptables -F").unwrap();
    let empty = s.filter.clone();
    let before = s.clone();
    assert!(
        !s.command(&format!("iptables-restore --test {FILTER_FILE}"))
            .unwrap()
            .changed
    );
    assert_eq!(s, before);
    assert_eq!(s.filter, empty);
    s.command(&format!("iptables-restore {FILTER_FILE}"))
        .unwrap();
    assert_eq!(s.filter, active);
}
#[test]
fn saved_forwarding_is_separate_and_empty_files_are_valid_no_ops() {
    let mut s = state();
    assert_eq!(
        s.command("cat /proc/sys/net/ipv4/ip_forward").unwrap().text,
        "0\n"
    );
    s.command("sysctl -w net.ipv4.ip_forward=1").unwrap();
    assert!(s.forwarding);
    assert!(!admin::parse_forwarding(&s.forward_file).unwrap().unwrap());
    s.command(&format!("sysctl -p {FORWARD_FILE}")).unwrap();
    assert!(!s.forwarding);
    s.write_file(
        FORWARD_FILE,
        "# Explain the setting\nnet.ipv4.ip_forward = 1 # transit\n",
    )
    .unwrap();
    assert!(!s.forwarding);
    s.command(&format!("sysctl -p {FORWARD_FILE}")).unwrap();
    assert!(s.forwarding);
    s.write_file(FORWARD_FILE, "# No assignment\n").unwrap();
    let before = s.clone();
    let out = s.command(&format!("sysctl -p {FORWARD_FILE}")).unwrap();
    assert_eq!(out.text, "");
    assert!(!out.changed);
    assert_eq!(s, before);
    s.write_file(
        FORWARD_FILE,
        "net.ipv4.ip_forward=0\nnet.ipv4.ip_forward=1\n",
    )
    .unwrap();
    let before = s.clone();
    assert_eq!(
        s.command(&format!("sysctl -p {FORWARD_FILE}"))
            .unwrap_err()
            .class,
        Class::Unsupported
    );
    assert_eq!(s, before);
}
#[test]
fn malformed_and_unmodeled_snapshots_never_partially_replace_active_rules() {
    let mut s = state();
    ready(&mut s);
    let valid = admin::save_filter(&s.filter).unwrap();
    for text in [
        valid.replace("COMMIT\n", ""),
        valid.replace("[0:0]", "[1:20]"),
        valid.replace("*filter", "*nat"),
        format!("{valid}*filter\nCOMMIT\n"),
        valid.replace("-A FORWARD", "-A UNKNOWN"),
        valid.replace(
            "COMMIT",
            "-A INPUT -j DROP\n-A FORWARD -m conntrack --ctstate ESTABLISHED -j ACCEPT\nCOMMIT",
        ),
    ] {
        s.write_file(FILTER_FILE, &text).unwrap();
        let before = s.clone();
        assert!(
            s.command(&format!("iptables-restore {FILTER_FILE}"))
                .is_err()
        );
        assert_eq!(s, before);
    }
}
#[test]
fn equivalent_snapshot_header_order_comments_and_quotes_restore_real_predicates() {
    let input = "# This is data, not a shell script\n*filter\n:OUTPUT DROP [0:0]\n:INPUT DROP [0:0]\n:FORWARD DROP [0:0]\n-A FORWARD -s '10.20.30.0/24' -p tcp --dport 443 -j ACCEPT\nCOMMIT\n# End\n";
    let table = admin::restore_filter(input).unwrap();
    assert_eq!(table.forward.rules.len(), 1);
    assert_eq!(table.forward.rules[0].destination_port, Some(443));
    assert_eq!(
        admin::restore_filter(&admin::save_filter(&table).unwrap()).unwrap(),
        table
    );
}
#[test]
fn arbitrary_host_files_shell_operators_and_out_of_profile_commands_are_refused() {
    let mut s = state();
    let before = s.clone();
    for line in [
        "cat /etc/passwd",
        "cat ../../etc/passwd",
        "iptables-save -f /tmp/rules",
        "iptables-restore ../../etc/passwd",
        "sysctl -p /etc/sysctl.conf",
        "sysctl -w kernel.hostname=bad",
        "iptables -A FORWARD -j ACCEPT; touch /tmp/bad",
        "iptables-save > /etc/lab-filter.rules",
        "ip -6 route show",
    ] {
        assert!(s.command(line).is_err(), "{line}");
        assert_eq!(s, before);
    }
    assert_eq!(
        s.command(&"x".repeat(2049)).unwrap_err().class,
        Class::Limit
    );
    assert!(s.write_file("/etc/passwd", "bad").is_err());
    assert!(s.write_file(FILTER_FILE, &"x".repeat(32769)).is_err());
    assert!(s.write_file(FILTER_FILE, "bad\0text").is_err());
    assert_eq!(s, before);
}
#[test]
fn separate_router_states_and_saved_text_roundtrip_without_replay_or_invented_repairs() {
    let mut assignment = state();
    let practice = state();
    ready(&mut assignment);
    assignment
        .write_file(FILTER_FILE, "# unfinished snapshot\n*filter\n")
        .unwrap();
    assignment
        .write_file(FORWARD_FILE, "net.ipv4.ip_forward = 0\n")
        .unwrap();
    let raw = serde_json::to_vec(&assignment).unwrap();
    let loaded: State = serde_json::from_slice(&raw).unwrap();
    loaded.validate().unwrap();
    assert_eq!(loaded, assignment);
    assert_ne!(loaded, practice);
    assert!(loaded.forwarding);
    assert!(
        !admin::parse_forwarding(&loaded.forward_file)
            .unwrap()
            .unwrap()
    );
    assert!(admin::restore_filter(&loaded.filter_file).is_err());
    assert_eq!(practice, state());
}

#[test]
fn a_valid_on_link_route_via_an_unavailable_supplied_gateway_is_not_reachability() {
    let mut s = state();
    ready(&mut s);
    let h = header();
    let o = Origin::Incoming("lan0".into());
    s.command("ip -4 route replace default via 198.51.100.253 dev wan0 metric 10")
        .unwrap();
    assert_eq!(s.evaluate(&o, &h).unwrap().gate, Gate::PeerUnavailable);
    assert!(!s.host.probe(h.destination).pass);
    s.command("ip -4 route replace default via 198.51.100.254 dev wan0 metric 10")
        .unwrap();
    assert_eq!(s.evaluate(&o, &h).unwrap().gate, Gate::Allowed);
}
