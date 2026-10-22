use dlms_learning_lab::ipv4_routing::{
    self as ip4, Assignment, Device, Interface, Prefix, Route, RouteStatus, Topology,
};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
fn interface(link: &str, address: &str, n: u8) -> Interface {
    Interface {
        link: link.into(),
        up: true,
        address: Some(Assignment {
            address: ip(address),
            prefix: n,
        }),
    }
}
fn route(net: &str, port: &str, via: &str, distance: u8) -> Route {
    Route {
        destination: prefix(net),
        interface: port.into(),
        via: ip(via),
        distance,
    }
}
fn device(forwarding: bool, ports: Vec<(&str, Interface)>, routes: Vec<Route>) -> Device {
    Device {
        forwarding,
        interfaces: ports.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        routes,
    }
}
fn topology() -> Topology {
    Topology {
        devices: BTreeMap::from([
            (
                "Client".into(),
                device(
                    false,
                    vec![("lan", interface("office", "192.0.2.10", 24))],
                    vec![route("0.0.0.0/0", "lan", "192.0.2.1", 1)],
                ),
            ),
            (
                "R1".into(),
                device(
                    true,
                    vec![
                        ("office", interface("office", "192.0.2.1", 24)),
                        ("transit", interface("backbone", "10.0.0.1", 30)),
                    ],
                    vec![route("198.51.100.0/24", "transit", "10.0.0.2", 1)],
                ),
            ),
            (
                "R2".into(),
                device(
                    true,
                    vec![
                        ("transit", interface("backbone", "10.0.0.2", 30)),
                        ("branch", interface("branch", "198.51.100.1", 24)),
                    ],
                    vec![route("192.0.2.0/24", "transit", "10.0.0.1", 1)],
                ),
            ),
            (
                "Server".into(),
                device(
                    false,
                    vec![("lan", interface("branch", "198.51.100.20", 24))],
                    vec![route("0.0.0.0/0", "lan", "198.51.100.1", 1)],
                ),
            ),
        ]),
    }
}
fn port_mut<'a>(t: &'a mut Topology, d: &str, p: &str) -> &'a mut Interface {
    t.devices.get_mut(d).unwrap().interfaces.get_mut(p).unwrap()
}
#[test]
fn every_ipv4_prefix_and_octet_boundary_is_aligned_without_overflow() {
    for value in [0, 1, u32::MAX, 0xc000024d, 0x0a14ff7f, 0xac10fe81] {
        for n in 0..=32 {
            let p = Prefix::new(value.into(), n).unwrap();
            let width = 1u64 << (32 - n);
            let expected = (u64::from(value) / width) * width;
            assert_eq!(u32::from(p.address) as u64, expected);
            assert_eq!(
                u32::from(p.last_address().unwrap()) as u64,
                expected + width - 1
            );
            assert!(p.valid());
            assert!(p.contains(value.into()));
            assert_eq!(Prefix::parse(&p.text()).unwrap(), p);
            if n > 0 {
                assert!(!p.contains((value ^ (1u32 << (32 - n))).into()));
            }
        }
    }
    assert_eq!(
        prefix("0.0.0.0/0").last_address(),
        Some(ip("255.255.255.255"))
    );
    assert_eq!(
        prefix("192.0.2.77/32").last_address(),
        Some(ip("192.0.2.77"))
    );
    assert_eq!(
        prefix("192.0.2.76/31").last_address(),
        Some(ip("192.0.2.77"))
    );
    assert!(ip4::mask(33).is_none());
    for bad in [
        "192.0.2.1/24",
        "192.0.2.0/33",
        "192.0.2.0/-1",
        "192.0.2.0/+24",
        "192.000.2.0/24",
        "bad/24",
        "192.0.2.0/",
        "192.0.2.0/24/1",
    ] {
        assert!(Prefix::parse(bad).is_err(), "{bad}");
    }
    // Original conventional subnet exercise still excludes /31 and /32.
    assert!(dlms_learning_lab::ipv4::mask(31).is_none());
    assert!(dlms_learning_lab::ipv4::mask(32).is_none());
}
#[test]
fn multihop_verification_needs_forwarding_and_a_real_return_path() {
    let t = topology();
    assert!(t.valid());
    let p = t.verify_endpoint("Client", "Server", "lan").unwrap();
    assert_eq!(p.source, ip("192.0.2.10"));
    assert_eq!(p.forward.destination_device, "Server");
    assert_eq!(p.reply.destination_device, "Client");
    assert_eq!(
        p.forward
            .hops
            .iter()
            .map(|h| h.device.as_str())
            .collect::<Vec<_>>(),
        ["Client", "R1", "R2"]
    );
    assert_eq!(
        p.reply
            .hops
            .iter()
            .map(|h| h.device.as_str())
            .collect::<Vec<_>>(),
        ["Server", "R2", "R1"]
    );
    let mut broken = t.clone();
    broken.devices.get_mut("R2").unwrap().routes.clear();
    assert!(
        broken
            .verify_endpoint("Client", "Server", "lan")
            .unwrap_err()
            .contains("reply fails")
    );
    let mut broken = t.clone();
    broken.devices.get_mut("R1").unwrap().forwarding = false;
    assert!(
        broken
            .probe("Client", ip("198.51.100.20"))
            .unwrap_err()
            .contains("forwarding is disabled")
    );
    assert!(
        broken.probe("Client", ip("192.0.2.1")).is_ok(),
        "local router delivery does not require forwarding"
    );
    let mut broken = t.clone();
    port_mut(&mut broken, "Server", "lan").up = false;
    assert!(
        broken
            .verify_endpoint("Client", "Server", "lan")
            .unwrap_err()
            .contains("is down")
    );
    assert!(
        t.probe("Client", ip("8.8.8.8"))
            .unwrap_err()
            .contains("Unsupported probe")
    );
    assert!(t.probe("missing", ip("192.0.2.1")).is_err());
}
#[test]
fn connected_specific_default_and_host_routes_apply_longest_prefix_before_distance() {
    let mut t = topology();
    let r1 = t.devices.get_mut("R1").unwrap();
    r1.routes.push(route("0.0.0.0/0", "transit", "10.0.0.2", 1));
    r1.routes
        .push(route("198.51.100.20/32", "transit", "10.0.0.2", 200));
    let s = t.select("R1", ip("198.51.100.20")).unwrap();
    assert_eq!((s.destination.length, s.distance), (32, 200));
    assert_eq!(
        t.select("R1", ip("198.51.100.21"))
            .unwrap()
            .destination
            .length,
        24
    );
    assert_eq!(
        t.select("R1", ip("203.0.113.1"))
            .unwrap()
            .destination
            .length,
        0
    );
    t.devices
        .get_mut("R1")
        .unwrap()
        .routes
        .push(route("192.0.2.0/24", "transit", "10.0.0.2", 1));
    let s = t.select("R1", ip("192.0.2.10")).unwrap();
    assert_eq!(
        (s.interface.as_str(), s.distance, s.via),
        ("office", 0, None)
    );
    t.devices.get_mut("R1").unwrap().routes.push(route(
        "198.51.100.20/32",
        "transit",
        "10.0.0.2",
        20,
    ));
    assert_eq!(t.select("R1", ip("198.51.100.20")).unwrap().distance, 20);
    assert!(t.verify_endpoint("Client", "Server", "lan").is_ok());
}
#[test]
fn candidate_status_explains_reserved_offlink_self_and_missing_local_addresses() {
    for (via, status) in [
        ("10.0.0.0", RouteStatus::ReservedNextHop),
        ("10.0.0.3", RouteStatus::ReservedNextHop),
        ("10.0.0.1", RouteStatus::SelfNextHop),
        ("10.0.1.2", RouteStatus::NextHopOffLink),
    ] {
        let mut t = topology();
        t.devices.get_mut("R1").unwrap().routes[0].via = ip(via);
        assert_eq!(t.route_status("R1", 0).unwrap(), status);
        assert!(t.probe("Client", ip("198.51.100.20")).is_err());
    }
    let mut t = topology();
    t.devices.get_mut("R1").unwrap().routes[0].distance = 255;
    assert_eq!(t.route_status("R1", 0).unwrap(), RouteStatus::Distance255);
    let mut t = topology();
    port_mut(&mut t, "R1", "transit").address = None;
    assert_eq!(
        t.route_status("R1", 0).unwrap(),
        RouteStatus::NoUsableAddress
    );
    let mut t = topology();
    port_mut(&mut t, "R1", "transit").up = false;
    assert_eq!(t.route_status("R1", 0).unwrap(), RouteStatus::InterfaceDown);
    assert!(t.route_status("R1", 99).is_err());
}
#[test]
fn missing_neighbor_does_not_withdraw_a_static_route_or_fake_backup_liveness() {
    let mut t = topology();
    t.devices
        .get_mut("R1")
        .unwrap()
        .interfaces
        .insert("backup".into(), interface("backup", "10.0.1.1", 30));
    t.devices
        .get_mut("R2")
        .unwrap()
        .interfaces
        .insert("backup".into(), interface("backup", "10.0.1.2", 30));
    t.devices.get_mut("R1").unwrap().routes.push(route(
        "198.51.100.0/24",
        "backup",
        "10.0.1.2",
        150,
    ));
    t.devices
        .get_mut("R2")
        .unwrap()
        .routes
        .push(route("192.0.2.0/24", "backup", "10.0.1.1", 150));
    assert_eq!(
        t.select("R1", ip("198.51.100.20")).unwrap().interface,
        "transit"
    );
    port_mut(&mut t, "R2", "transit").up = false;
    assert_eq!(t.route_status("R1", 0).unwrap(), RouteStatus::Eligible);
    assert!(
        t.probe("Client", ip("198.51.100.20"))
            .unwrap_err()
            .contains("static route is still eligible")
    );
    port_mut(&mut t, "R1", "transit").up = false;
    assert_eq!(
        t.select("R1", ip("198.51.100.20")).unwrap().interface,
        "backup"
    );
    assert!(t.verify_endpoint("Client", "Server", "lan").is_ok());
    port_mut(&mut t, "R1", "transit").up = true;
    port_mut(&mut t, "R2", "transit").up = true;
    t.devices.get_mut("R1").unwrap().routes[1].distance = 1;
    assert!(
        t.select("R1", ip("198.51.100.20"))
            .unwrap_err()
            .contains("equal-cost")
    );
}
#[test]
fn loops_duplicates_network_broadcast_and_wrong_terminal_devices_never_verify() {
    let mut t = topology();
    t.devices.get_mut("R2").unwrap().routes.push(route(
        "198.51.100.20/32",
        "transit",
        "10.0.0.1",
        1,
    ));
    assert!(
        t.probe("Client", ip("198.51.100.20"))
            .unwrap_err()
            .contains("Routing loop")
    );
    let mut t = topology();
    t.devices.insert(
        "Imposter".into(),
        device(
            false,
            vec![("lan", interface("branch", "198.51.100.20", 24))],
            vec![],
        ),
    );
    assert!(
        t.probe("Client", ip("198.51.100.20"))
            .unwrap_err()
            .contains("duplicate")
    );
    for address in ["192.0.2.0", "192.0.2.255"] {
        let mut t = topology();
        port_mut(&mut t, "Client", "lan")
            .address
            .as_mut()
            .unwrap()
            .address = ip(address);
        assert!(t.valid());
        assert!(t.probe("Client", ip("192.0.2.1")).is_err());
    }
    let mut t = topology();
    port_mut(&mut t, "R1", "transit").address = Some(Assignment {
        address: ip("198.51.100.20"),
        prefix: 24,
    });
    let raw = t.probe("Client", ip("198.51.100.20")).unwrap();
    assert_eq!(raw.forward.destination_device, "R1");
    assert!(
        t.verify_endpoint("Client", "Server", "lan")
            .unwrap_err()
            .contains("not intended destination Server")
    );
    port_mut(&mut t, "Server", "lan").up = false;
    assert!(t.verify_endpoint("Client", "Server", "lan").is_err());
    let mut t = topology();
    port_mut(&mut t, "Client", "lan")
        .address
        .as_mut()
        .unwrap()
        .address = ip("198.51.100.20");
    assert_eq!(
        t.probe("Client", ip("198.51.100.20"))
            .unwrap()
            .forward
            .hops
            .len(),
        0
    );
    assert!(t.verify_endpoint("Client", "Server", "lan").is_err());
}
#[test]
fn decoding_rejects_duplicate_ids_oversized_collections_unknown_fields_and_unsupported_interfaces()
{
    let t = topology();
    let text = serde_json::to_string(&t).unwrap();
    assert_eq!(Topology::decode(&text).unwrap(), t);
    let client = serde_json::to_string(&t.devices["Client"]).unwrap();
    let duplicate =
        "{\"devices\":{\"Client\":".to_owned() + &client + ",\"Client\":" + &client + "}}";
    assert!(Topology::decode(&duplicate).is_err());
    let i = serde_json::to_string(&t.devices["Client"].interfaces["lan"]).unwrap();
    assert!(Topology::decode(&format!("{{\"devices\":{{\"Client\":{{\"forwarding\":false,\"interfaces\":{{\"lan\":{i},\"lan\":{i}}},\"routes\":[]}}}}}}" )).is_err());
    for n in [31, 32] {
        let mut bad = t.clone();
        port_mut(&mut bad, "Client", "lan")
            .address
            .as_mut()
            .unwrap()
            .prefix = n;
        assert!(!bad.valid());
        assert!(Topology::decode(&serde_json::to_string(&bad).unwrap()).is_err());
    }
    let mut bad = t.clone();
    bad.devices.get_mut("R1").unwrap().routes[0].interface = "missing".into();
    assert!(!bad.valid());
    assert!(bad.probe("Client", ip("198.51.100.20")).is_err());
    let mut bad = t.clone();
    bad.devices.get_mut("R1").unwrap().routes =
        vec![route("0.0.0.0/0", "transit", "10.0.0.2", 1); 33];
    assert!(Topology::decode(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut data = serde_json::to_value(&t).unwrap();
    data["devices"]["Client"]["extra"] = serde_json::json!(true);
    assert!(Topology::decode(&data.to_string()).is_err());
    assert!(Topology::decode(&" ".repeat(1_000_001)).is_err());
    let mut moved = t.clone();
    port_mut(&mut moved, "Client", "lan").link = "changed".into();
    assert!(!moved.same_shape(&t));
    let mut too_many = t.clone();
    for n in 0..8 {
        too_many
            .devices
            .get_mut("Client")
            .unwrap()
            .interfaces
            .insert(
                format!("extra{n}"),
                Interface {
                    link: format!("extra{n}"),
                    up: false,
                    address: None,
                },
            );
    }
    assert!(!too_many.valid());
    assert!(Topology::decode(&serde_json::to_string(&too_many).unwrap()).is_err());
    let mut invalid_id = serde_json::to_value(&t).unwrap();
    invalid_id["devices"]["../invalid"] = invalid_id["devices"]["Client"].clone();
    assert!(Topology::decode(&invalid_id.to_string()).is_err());
    let mut no_next_hop = serde_json::to_value(&t).unwrap();
    no_next_hop["devices"]["R1"]["routes"][0]["via"] = serde_json::Value::Null;
    assert!(Topology::decode(&no_next_hop.to_string()).is_err());
    assert!(t.same_shape(&t));
}
#[test]
fn maximum_device_chain_has_bounded_both_direction_paths_and_current_breakage_is_detected() {
    let mut t = Topology {
        devices: BTreeMap::new(),
    };
    for n in 0..12 {
        let mut ports = vec![];
        let mut routes = vec![];
        if n > 0 {
            ports.push((
                "left",
                interface(&format!("link{}", n - 1), &format!("10.0.{}.2", n - 1), 30),
            ));
            routes.push(route(
                "10.0.0.1/32",
                "left",
                &format!("10.0.{}.1", n - 1),
                1,
            ));
        }
        if n < 11 {
            ports.push((
                "right",
                interface(&format!("link{n}"), &format!("10.0.{n}.1"), 30),
            ));
            routes.push(route("0.0.0.0/0", "right", &format!("10.0.{n}.2"), 1));
        }
        let mut d = device(true, ports, routes);
        while d.interfaces.len() < 8 {
            let id = format!("unused{}", d.interfaces.len());
            d.interfaces.insert(
                id.clone(),
                Interface {
                    link: id,
                    up: false,
                    address: None,
                },
            );
        }
        while d.routes.len() < 32 {
            let port = if n < 11 { "right" } else { "left" };
            let via = if n < 11 {
                format!("10.0.{n}.2")
            } else {
                format!("10.0.{}.1", n - 1)
            };
            d.routes.push(route("203.0.113.99/32", port, &via, 254));
        }
        t.devices.insert(format!("R{n}"), d);
    }
    assert!(t.valid());
    assert_eq!(
        Topology::decode(&serde_json::to_string(&t).unwrap()).unwrap(),
        t
    );
    let p = t.verify_endpoint("R0", "R11", "left").unwrap();
    assert_eq!((p.forward.hops.len(), p.reply.hops.len()), (11, 11));
    port_mut(&mut t, "R5", "right").up = false;
    assert!(t.verify_endpoint("R0", "R11", "left").is_err());
    t.devices.insert("excess".into(), t.devices["R0"].clone());
    assert!(!t.valid());
    assert!(Topology::decode(&serde_json::to_string(&t).unwrap()).is_err());
}

#[test]
fn virtual_endpoint_scope_and_conventional_interface_host_bounds_are_explicit() {
    for address in [
        "10.0.0.1",
        "172.16.0.1",
        "172.31.255.254",
        "192.168.1.1",
        "192.0.2.1",
        "198.51.100.1",
        "203.0.113.1",
    ] {
        assert!(ip4::supported_address(ip(address)));
    }
    for address in [
        "0.0.0.0",
        "127.0.0.1",
        "169.254.1.1",
        "172.32.0.1",
        "100.64.0.1",
        "8.8.8.8",
        "224.0.0.1",
        "255.255.255.255",
    ] {
        assert!(!ip4::supported_address(ip(address)));
    }
    for n in 1..=30 {
        for address in [
            "10.0.0.0",
            "10.0.0.1",
            "192.0.2.0",
            "192.0.2.1",
            "192.0.2.3",
            "192.0.2.255",
        ] {
            let a = Assignment {
                address: ip(address),
                prefix: n,
            };
            let value = u64::from(u32::from(a.address));
            let size = 1u64 << (32 - n);
            let first = (value / size) * size;
            assert!(a.valid());
            assert_eq!(a.usable(), value != first && value != first + size - 1);
        }
    }
    let invalid = Prefix {
        address: ip("192.0.2.1"),
        length: 24,
    };
    assert!(!invalid.valid());
    assert_eq!(invalid.last_address(), None);
    let mut t = topology();
    port_mut(&mut t, "Server", "lan").address = Some(Assignment {
        address: ip("198.51.100.31"),
        prefix: 27,
    });
    assert!(t.valid());
    assert!(
        t.probe("Client", ip("198.51.100.31"))
            .unwrap_err()
            .contains("neighbor's configured prefix")
    );
}

#[test]
fn connected_neighbor_feedback_and_cross_interface_local_next_hops_are_specific() {
    let t = topology();
    let e = t.probe("Client", ip("192.0.2.99")).unwrap_err();
    assert!(e.contains("directly attached"));
    assert!(!e.contains("static route"));
    for up in [true, false] {
        let mut broken = t.clone();
        let p = port_mut(&mut broken, "R1", "office");
        p.address = Some(Assignment {
            address: ip("10.0.0.2"),
            prefix: 30,
        });
        p.up = up;
        assert!(broken.valid());
        assert_eq!(
            broken.route_status("R1", 0).unwrap(),
            RouteStatus::SelfNextHop
        );
        assert!(broken.verify_endpoint("R1", "Server", "lan").is_err());
    }
}
