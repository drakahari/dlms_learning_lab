use dlms_learning_lab::{
    ipv4_routing::{Assignment, Prefix},
    linux_host::*,
};
fn ip(s: &str) -> std::net::Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
fn fixture() -> State {
    let profile = Profile {
        ipv6: None,
        id: "host-fixture".into(),
        links: vec![
            Link {
                name: "ens3".into(),
                network: prefix("192.0.2.0/24"),
                carrier: true,
            },
            Link {
                name: "ens4".into(),
                network: prefix("198.51.100.0/24"),
                carrier: true,
            },
        ],
        gateways: vec![
            Gateway {
                address: ip("192.0.2.1"),
                device: "ens3".into(),
                available: true,
                exports: vec![prefix("203.0.113.0/24")],
            },
            Gateway {
                address: ip("198.51.100.1"),
                device: "ens4".into(),
                available: true,
                exports: vec![prefix("203.0.113.0/24")],
            },
        ],
        nodes: [
            "192.0.2.20",
            "192.0.2.200",
            "198.51.100.20",
            "203.0.113.53",
            "203.0.113.54",
            "203.0.113.80",
            "203.0.113.90",
        ]
        .iter()
        .map(|s| Node {
            address: ip(s),
            available: true,
        })
        .collect(),
        resolvers: vec![
            Resolver {
                address: ip("203.0.113.53"),
                records: vec![Record {
                    name: "service.lab.test".into(),
                    address: ip("203.0.113.80"),
                }],
            },
            Resolver {
                address: ip("203.0.113.54"),
                records: vec![
                    Record {
                        name: "service.lab.test".into(),
                        address: ip("203.0.113.90"),
                    },
                    Record {
                        name: "backup.lab.test".into(),
                        address: ip("203.0.113.90"),
                    },
                ],
            },
        ],
    };
    let cfg = Configuration {
        ipv6: None,
        interfaces: vec![
            Interface {
                name: "ens3".into(),
                up: true,
                address: Some(Assignment {
                    address: ip("192.0.2.10"),
                    prefix: 24,
                }),
            },
            Interface {
                name: "ens4".into(),
                up: true,
                address: Some(Assignment {
                    address: ip("198.51.100.10"),
                    prefix: 24,
                }),
            },
        ],
        routes: vec![
            Route {
                destination: prefix("0.0.0.0/0"),
                via: ip("192.0.2.1"),
                device: "ens3".into(),
                metric: 100,
            },
            Route {
                destination: prefix("0.0.0.0/0"),
                via: ip("198.51.100.1"),
                device: "ens4".into(),
                metric: 200,
            },
        ],
        nameservers: vec![ip("203.0.113.53"), ip("203.0.113.54")],
    };
    State::new(profile, cfg).unwrap()
}
#[test]
fn host_connected_routes_and_u32_metrics_use_longest_prefix_before_metric() {
    let mut s = fixture();
    assert_eq!(s.select(ip("192.0.2.20")).unwrap().via, None);
    assert_eq!(s.select(ip("203.0.113.80")).unwrap().metric, 100);
    s.execute("ip -4 route add 203.0.113.80/32 via 198.51.100.1 dev ens4 metric 4294967295")
        .unwrap();
    let route = s.select(ip("203.0.113.80")).unwrap();
    assert_eq!(route.device, "ens4");
    assert_eq!(route.metric, u32::MAX);
    assert_eq!(route.source, ip("198.51.100.10"));
    assert!(s.probe(ip("203.0.113.80")).pass);
    assert_eq!(s.select(ip("203.0.113.90")).unwrap().device, "ens3");
}
#[test]
fn host_duplicate_add_replace_and_exact_deletion_change_real_state() {
    let mut s = fixture();
    let before = s.clone();
    assert!(
        s.execute("ip -4 route add default via 198.51.100.1 dev ens4 metric 100")
            .unwrap_err()
            .contains("exists")
    );
    assert_eq!(s, before);
    s.execute("ip -4 route replace default via 198.51.100.1 dev ens4 metric 100")
        .unwrap();
    assert_eq!(s.select(ip("203.0.113.80")).unwrap().device, "ens4");
    let before = s.clone();
    assert!(
        s.execute("ip -4 route del default via 192.0.2.1 dev ens3 metric 100")
            .is_err()
    );
    assert_eq!(s, before);
    s.execute("ip -4 route del default via 198.51.100.1 dev ens4 metric 100")
        .unwrap();
    assert_eq!(s.select(ip("203.0.113.80")).unwrap().metric, 200);
}
#[test]
fn host_address_delete_order_and_multiple_address_limit_are_honest_and_atomic() {
    let mut s = fixture();
    for cmd in [
        "ip -4 address del 192.0.2.10/24 dev ens3",
        "ip -4 address flush dev ens3",
        "ip -4 address add 192.0.2.11/24 dev ens3",
        "ip -4 address replace 192.0.2.11/24 dev ens3",
    ] {
        let before = s.clone();
        assert!(s.execute(cmd).is_err());
        assert_eq!(s, before);
    }
    s.execute("ip -4 route del default via 192.0.2.1 dev ens3 metric 100")
        .unwrap();
    s.execute("ip -4 address del 192.0.2.10/24 dev ens3")
        .unwrap();
    s.execute("ip -4 address add 192.0.2.11/24 dev ens3")
        .unwrap();
    assert_eq!(
        s.configuration.interfaces[0]
            .address
            .as_ref()
            .unwrap()
            .address,
        ip("192.0.2.11")
    );
    assert!(s.probe(ip("192.0.2.200")).pass);
}
#[test]
fn host_wrong_mask_wrong_lan_conflict_and_unknown_next_hop_fail_for_different_reasons() {
    let base = fixture();
    let mut s = base.clone();
    s.configuration.interfaces[0]
        .address
        .as_mut()
        .unwrap()
        .prefix = 25;
    assert!(!s.probe(ip("192.0.2.200")).pass);
    assert!(s.probe(ip("192.0.2.20")).pass);
    s = base.clone();
    s.configuration.interfaces[0]
        .address
        .as_mut()
        .unwrap()
        .address = ip("10.20.0.10");
    assert!(s.probe(ip("203.0.113.80")).reason.contains("outside"));
    s = base.clone();
    s.configuration.interfaces[0]
        .address
        .as_mut()
        .unwrap()
        .address = ip("192.0.2.20");
    assert!(s.probe(ip("203.0.113.80")).reason.contains("conflicts"));
    s = base;
    s.execute("ip -4 route replace default via 192.0.2.99 dev ens3 metric 100")
        .unwrap();
    assert!(
        s.probe(ip("203.0.113.80"))
            .reason
            .contains("No supplied gateway")
    );
}
#[test]
fn host_administrative_link_carrier_gateway_and_destination_facts_are_separate() {
    let base = fixture();
    let mut s = base.clone();
    s.execute("ip link set dev ens3 down").unwrap();
    assert!(
        s.probe(ip("203.0.113.80"))
            .reason
            .contains("administratively down")
    );
    let out = s.execute("ip link show").unwrap();
    assert!(out.contains("state DOWN"));
    assert!(!out.contains("<DOWN>"));
    s = base.clone();
    s.profile.links[0].carrier = false;
    assert!(s.probe(ip("203.0.113.80")).reason.contains("carrier"));
    s = base.clone();
    s.profile.gateways[0].available = false;
    assert!(
        s.probe(ip("203.0.113.80"))
            .reason
            .contains("gateway is unavailable")
    );
    s = base.clone();
    s.profile
        .nodes
        .iter_mut()
        .find(|n| n.address == ip("203.0.113.80"))
        .unwrap()
        .available = false;
    assert!(
        s.probe(ip("203.0.113.80"))
            .reason
            .contains("destination is unavailable")
    );
    s = base;
    assert!(!s.probe(ip("203.0.113.99")).pass);
}
#[test]
fn host_offlink_gateway_down_interface_and_bad_prefix_updates_are_atomic() {
    let mut s = fixture();
    for cmd in [
        "ip -4 route add 203.0.113.0/24 via 198.51.100.1 dev ens3 metric 10",
        "ip -4 route add 203.0.113.3/24 via 192.0.2.1 dev ens3 metric 10",
        "ip -4 route add default via 192.0.2.1 dev unknown metric 10",
        "ip -4 route add default via 192.0.2.1 dev ens3 metric -1",
        "ip -4 route add default via 192.0.2.1 dev ens3 metric 4294967296",
        "ip -4 route add 192.0.2.0/24 via 192.0.2.1 dev ens3",
    ] {
        let before = s.clone();
        assert!(s.execute(cmd).is_err(), "{cmd}");
        assert_eq!(s, before);
    }
    s.execute("ip link set dev ens3 down").unwrap();
    let before = s.clone();
    assert!(
        s.execute("ip -4 route add 203.0.113.0/24 via 192.0.2.1 dev ens3")
            .is_err()
    );
    assert_eq!(s, before);
}
#[test]
fn host_terminal_subset_never_evaluates_shell_or_unknown_flags() {
    for cmd in [
        "ip route show",
        "sudo ip link show",
        "ip -6 address show",
        "ip -4 route get 203.0.113.80",
        "ip -4 route add default dev ens3",
        "ip link show; id",
        "ip link show | cat",
        "ip link show > /tmp/result",
        "$(id)",
        "ip\nlink show",
        "ip link\u{a0}show",
    ] {
        let mut s = fixture();
        let before = s.clone();
        assert!(s.execute(cmd).is_err(), "{cmd:?}");
        assert_eq!(s, before);
    }
    let mut s = fixture();
    assert!(
        s.execute("ip -4 address show dev ens3")
            .unwrap()
            .contains("inet 192.0.2.10/24")
    );
    assert!(
        s.execute("ip -4 route show")
            .unwrap()
            .contains("metric 200")
    );
    assert_eq!(s.execute("ip link set dev ens3 up").unwrap(), "");
}
#[test]
fn host_route_bounds_do_not_confuse_interface_31_32_exclusion() {
    let mut s = fixture();
    for cidr in [
        "192.0.2.0/24",
        "192.0.2.255/24",
        "192.0.2.10/31",
        "192.0.2.10/32",
    ] {
        let cmd = format!("ip -4 address add {cidr} dev ens3");
        let before = s.clone();
        assert!(s.execute(&cmd).is_err());
        assert_eq!(s, before);
    }
    s.execute("ip -4 route add 203.0.113.80/31 via 192.0.2.1 dev ens3 metric 5")
        .unwrap();
    assert_eq!(s.select(ip("203.0.113.80")).unwrap().destination.length, 31);
    s.execute("ip -4 route add 203.0.113.80/32 via 192.0.2.1 dev ens3 metric 6")
        .unwrap();
    assert_eq!(s.select(ip("203.0.113.80")).unwrap().destination.length, 32);
}
#[test]
fn host_resolver_order_fallback_and_negative_answer_do_not_invent_results() {
    let mut s = fixture();
    assert_eq!(s.resolve("service.lab.test").unwrap(), ip("203.0.113.80"));
    assert!(
        s.resolve("backup.lab.test")
            .unwrap_err()
            .contains("first reachable")
    );
    s.profile
        .nodes
        .iter_mut()
        .find(|n| n.address == ip("203.0.113.53"))
        .unwrap()
        .available = false;
    assert_eq!(s.resolve("service.lab.test").unwrap(), ip("203.0.113.90"));
    let out = s.execute("getent ahostsv4 service.lab.test").unwrap();
    assert!(out.contains("203.0.113.90 STREAM service.lab.test"));
    assert!(s.resolve("real.example.com").is_err());
    s.configuration.nameservers.clear();
    assert!(
        s.resolve("service.lab.test")
            .unwrap_err()
            .contains("Localhost-default")
    );
}
#[test]
fn host_resolver_editor_is_virtual_bounded_and_refuses_unsupported_directives_atomically() {
    let mut s = fixture();
    s.save_resolver("# sample\nnameserver 203.0.113.54 # alternate\n")
        .unwrap();
    assert_eq!(s.resolve("service.lab.test").unwrap(), ip("203.0.113.90"));
    assert_eq!(s.resolver_text(), "nameserver 203.0.113.54\n");
    for text in [
        "search lab.test\n",
        "options rotate\n",
        "nameserver 8.8.8.8\n",
        "nameserver ::1\n",
        "nameserver 203.0.113.54\nnameserver 203.0.113.54\n",
        "nameserver 203.0.113.53\nnameserver 203.0.113.54\nnameserver 192.0.2.1\n",
    ] {
        let before = s.clone();
        assert!(s.save_resolver(text).is_err(), "{text}");
        assert_eq!(s, before);
    }
}
#[test]
fn host_codec_profile_context_corruption_and_generation_limits_are_honest() {
    let s = fixture();
    let raw = serde_json::to_vec(&s).unwrap();
    assert_eq!(State::decode(&raw, &s.profile).unwrap(), s);
    let mut p = s.profile.clone();
    p.links[0].carrier = false;
    assert!(State::decode(&raw, &p).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["extra"] = serde_json::json!(true);
    assert!(State::decode(&serde_json::to_vec(&v).unwrap(), &s.profile).is_err());
    let mut s = s;
    s.generation = u64::MAX;
    let before = s.clone();
    assert!(s.execute("ip link set dev ens3 down").is_err());
    assert_eq!(s, before);
    assert!(s.execute("ip link set dev ens3 up").is_ok());
}

#[test]
fn host_case_insensitive_names_and_raw_profile_limits_do_not_accept_corruption() {
    let s = fixture();
    assert_eq!(s.resolve("SERVICE.LAB.TEST").unwrap(), ip("203.0.113.80"));
    let mut v = serde_json::to_value(&s).unwrap();
    let extra = v["profile"]["resolvers"][0]["records"][0].clone();
    v["profile"]["resolvers"][0]["records"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    assert!(State::decode(&serde_json::to_vec(&v).unwrap(), &s.profile).is_err());
    assert!(State::decode(&vec![b'x'; 65537], &s.profile).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["configuration"]["interfaces"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"name":"extra","up":true,"address":null}));
    assert!(State::decode(&serde_json::to_vec(&v).unwrap(), &s.profile).is_err());
    let mut p = s.profile.clone();
    p.gateways[0].address = ip("192.0.2.0");
    assert!(p.validate().is_err());
}
