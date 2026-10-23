//! Independent finite graph/configuration arithmetic, not a vendor packet engine.
use dlms_learning_lab::{
    ipv4_routing::{Assignment, Device, Interface, Prefix, Topology},
    ospf::*,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};

fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
fn e(r: &str, p: &str) -> Endpoint {
    Endpoint::new(r, p)
}
fn port(cost: u16, passive: bool) -> PortConfig {
    PortConfig {
        enabled: true,
        passive,
        network_type: if passive {
            NetworkType::Broadcast
        } else {
            NetworkType::PointToPoint
        },
        cost,
        hello_seconds: 10,
        dead_seconds: 40,
    }
}
fn mesh(edges: &[(&str, &str, &str, u16, u16)], stubs: &[(&str, &str, u8, u16)]) -> State {
    let names: BTreeSet<_> = edges
        .iter()
        .flat_map(|(a, b, _, _, _)| [*a, *b])
        .chain(stubs.iter().map(|(r, _, _, _)| *r))
        .collect();
    let mut topology = Topology {
        devices: BTreeMap::new(),
    };
    let mut configuration = Configuration {
        routers: BTreeMap::new(),
    };
    for (i, r) in names.iter().enumerate() {
        topology.devices.insert(
            (*r).into(),
            Device {
                forwarding: true,
                interfaces: BTreeMap::new(),
                routes: vec![],
            },
        );
        configuration.routers.insert(
            (*r).into(),
            RouterConfig {
                process: ProcessConfig {
                    enabled: true,
                    process_id: 1 + i as u16,
                    router_id: Some(Ipv4Addr::new(192, 0, 2, i as u8 + 1)),
                    area: 0,
                },
                interfaces: BTreeMap::new(),
            },
        );
    }
    for (i, (a, b, label, left, right)) in edges.iter().enumerate() {
        for (offset, r, cost) in [(1, *a, *left), (2, *b, *right)] {
            let address = Ipv4Addr::from(u32::from(ip("10.0.0.0")) + i as u32 * 4 + offset);
            topology.devices.get_mut(r).unwrap().interfaces.insert(
                (*label).into(),
                Interface {
                    link: (*label).into(),
                    up: true,
                    address: Some(Assignment {
                        address,
                        prefix: 30,
                    }),
                },
            );
            configuration
                .routers
                .get_mut(r)
                .unwrap()
                .interfaces
                .insert((*label).into(), port(cost, false));
        }
    }
    for (r, address, length, cost) in stubs {
        topology.devices.get_mut(*r).unwrap().interfaces.insert(
            "lan".into(),
            Interface {
                link: format!("stub-{r}"),
                up: true,
                address: Some(Assignment {
                    address: ip(address),
                    prefix: *length,
                }),
            },
        );
        configuration
            .routers
            .get_mut(*r)
            .unwrap()
            .interfaces
            .insert("lan".into(), port(*cost, true));
    }
    State::new(topology, configuration).unwrap()
}
fn line() -> State {
    mesh(
        &[("A", "B", "ab", 2, 9), ("B", "C", "bc", 5, 13)],
        &[("A", "172.16.1.1", 24, 4), ("C", "172.16.3.1", 24, 7)],
    )
}
fn configure_port(s: &mut State, at: Endpoint, f: impl FnOnce(&mut PortConfig)) {
    let mut c = s.configuration().routers[&at.router].interfaces[&at.interface].clone();
    f(&mut c);
    s.apply(&Action::SetPort {
        endpoint: at,
        configuration: c,
    })
    .unwrap();
}
fn configure_process(s: &mut State, r: &str, f: impl FnOnce(&mut ProcessConfig)) {
    let mut c = s.configuration().routers[r].process.clone();
    f(&mut c);
    s.apply(&Action::SetProcess {
        router: r.into(),
        process: c,
    })
    .unwrap();
}
fn route(s: &State, r: &str, p: &str) -> Route {
    s.snapshot().unwrap().routers[r]
        .route(prefix(p))
        .unwrap()
        .clone()
}
fn link(s: &State, r: &str, p: &str) -> LinkView {
    s.snapshot().unwrap().routers[r]
        .links
        .iter()
        .find(|l| l.local == e(r, p))
        .unwrap()
        .clone()
}
#[test]
fn directed_sender_costs_and_destination_stub_are_added_without_reverse_costs() {
    let s = line();
    let forward = route(&s, "A", "172.16.3.0/24");
    assert_eq!(forward.cost, 2 + 5 + 7);
    assert_eq!(
        forward.next_hops,
        BTreeSet::from([NextHop {
            interface: "ab".into(),
            via: ip("10.0.0.2"),
            peer: e("B", "ab")
        }])
    );
    let backward = route(&s, "C", "172.16.1.0/24");
    assert_eq!(backward.cost, 13 + 9 + 4);
    let snapshot = s.snapshot().unwrap();
    assert_eq!(snapshot.routers["A"].distances["C"], 7);
    assert!(
        snapshot.routers["A"]
            .route(prefix("172.16.1.0/24"))
            .is_none()
    );
    assert!(matches!(
        snapshot.routers["A"].lookup(ip("172.16.1.42")),
        Some(Selection::Connected(_))
    ));
    assert!(matches!(
        snapshot.routers["A"].lookup(ip("172.16.3.42")),
        Some(Selection::Ospf(_))
    ));
}
#[test]
fn actual_numeric_example_changes_work_without_canned_topology_names() {
    let mut s = mesh(
        &[
            ("North", "Core", "uplink", 30, 7),
            ("Core", "South", "downlink", 4, 3),
        ],
        &[("South", "198.51.100.1", 24, 8)],
    );
    assert_eq!(route(&s, "North", "198.51.100.0/24").cost, 42);
    configure_port(&mut s, e("North", "uplink"), |c| c.cost = 2);
    assert_eq!(route(&s, "North", "198.51.100.0/24").cost, 14);
}
#[test]
fn equal_cost_diamond_retains_all_first_hops_after_paths_converge() {
    let mut s = mesh(
        &[
            ("A", "B", "ab", 1, 7),
            ("A", "C", "ac", 2, 20),
            ("B", "D", "bd", 2, 4),
            ("C", "D", "cd", 1, 9),
            ("D", "E", "de", 3, 6),
        ],
        &[("E", "172.20.1.1", 24, 4)],
    );
    let r = route(&s, "A", "172.20.1.0/24");
    assert_eq!(r.cost, 10);
    assert_eq!(r.next_hops.len(), 2);
    assert_eq!(
        r.next_hops
            .iter()
            .map(|h| h.interface.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["ab", "ac"])
    );
    configure_port(&mut s, e("A", "ac"), |c| c.cost = 3);
    let r = route(&s, "A", "172.20.1.0/24");
    assert_eq!(r.cost, 10);
    assert_eq!(r.next_hops.len(), 1);
    assert_eq!(r.next_hops.first().unwrap().interface, "ab");
}
#[test]
fn parallel_links_remain_distinct_equal_cost_exit_choices() {
    let s = mesh(
        &[("A", "B", "one", 5, 30), ("A", "B", "two", 5, 2)],
        &[("B", "192.168.2.1", 24, 1)],
    );
    let r = route(&s, "A", "192.168.2.0/24");
    assert_eq!(r.cost, 6);
    assert_eq!(r.next_hops.len(), 2);
    assert_eq!(
        r.next_hops.iter().map(|h| h.via).collect::<BTreeSet<_>>(),
        BTreeSet::from([ip("10.0.0.2"), ip("10.0.0.6")])
    );
}
#[test]
fn separate_equal_cost_advertisers_merge_then_lowest_total_cost_wins() {
    let mut s = mesh(
        &[("A", "B", "ab", 2, 8), ("A", "C", "ac", 3, 9)],
        &[("B", "172.21.0.1", 24, 4), ("C", "172.21.0.2", 24, 3)],
    );
    let r = route(&s, "A", "172.21.0.0/24");
    assert_eq!(r.cost, 6);
    assert_eq!(
        r.advertisers,
        BTreeSet::from([e("B", "lan"), e("C", "lan")])
    );
    assert_eq!(r.next_hops.len(), 2);
    configure_port(&mut s, e("B", "lan"), |c| c.cost = 5);
    let r = route(&s, "A", "172.21.0.0/24");
    assert_eq!(r.cost, 6);
    assert_eq!(r.advertisers, BTreeSet::from([e("C", "lan")]));
}
#[test]
fn positive_cost_cycles_terminate_and_do_not_invent_unreachable_routes() {
    let s = mesh(
        &[
            ("A", "B", "ab", 2, 2),
            ("B", "C", "bc", 2, 2),
            ("C", "A", "ca", 2, 2),
        ],
        &[("D", "172.30.1.1", 24, 1)],
    );
    let v = &s.snapshot().unwrap().routers["A"];
    assert_eq!(v.distances.len(), 3);
    assert!(!v.distances.contains_key("D"));
    assert!(v.route(prefix("172.30.1.0/24")).is_none());
    assert!(v.lookup(ip("172.30.1.42")).is_none());
}
#[test]
fn local_process_numbers_are_not_peer_compatibility_parameters() {
    let mut s = line();
    configure_process(&mut s, "B", |p| p.process_id = 65535);
    assert!(link(&s, "A", "ab").compatible);
    assert_eq!(route(&s, "A", "172.16.3.0/24").cost, 14);
}
#[test]
fn mismatched_area_hello_and_dead_parameters_are_typed_current_failures() {
    for kind in ["area", "hello", "dead"] {
        let mut s = line();
        match kind {
            "area" => configure_process(&mut s, "B", |p| p.area = 7),
            "hello" => configure_port(&mut s, e("B", "ab"), |p| p.hello_seconds = 20),
            _ => configure_port(&mut s, e("B", "ab"), |p| p.dead_seconds = 80),
        }
        let l = link(&s, "A", "ab");
        assert!(!l.compatible);
        assert!(l.conditions.iter().any(|c| matches!(
            (kind, c),
            ("area", Condition::AreaMismatch { .. })
                | ("hello", Condition::HelloMismatch { .. })
                | ("dead", Condition::DeadMismatch { .. })
        )));
        assert!(
            s.snapshot().unwrap().routers["A"]
                .route(prefix("172.16.3.0/24"))
                .is_none()
        );
    }
}
#[test]
fn point_to_point_mask_and_off_subnet_differences_warn_without_false_hello_rejection() {
    let mut s = line();
    s.apply(&Action::SetAddress {
        endpoint: e("B", "ab"),
        address: Some(Assignment {
            address: ip("192.0.2.2"),
            prefix: 24,
        }),
    })
    .unwrap();
    let l = link(&s, "A", "ab");
    assert!(l.compatible);
    assert!(l.conditions.iter().any(|c| matches!(
        c,
        Condition::PointToPointMaskIgnored {
            local: 30,
            peer: 24
        }
    )));
    assert!(
        l.conditions
            .iter()
            .any(|c| matches!(c, Condition::PointToPointSubnetIgnored))
    );
    assert!(l.conditions.iter().all(|c| !c.blocking()));
    let r = route(&s, "A", "172.16.3.0/24");
    assert_eq!(r.cost, 14);
    assert_eq!(r.next_hops.first().unwrap().via, ip("192.0.2.2"));
}
#[test]
fn router_id_missing_zero_duplicate_and_disabled_are_not_full_adjacencies() {
    for kind in ["missing", "zero", "duplicate", "disabled"] {
        let mut s = line();
        let a_id = s.configuration().routers["A"].process.router_id;
        configure_process(&mut s, "B", |p| match kind {
            "missing" => p.router_id = None,
            "zero" => p.router_id = Some(Ipv4Addr::UNSPECIFIED),
            "duplicate" => p.router_id = a_id,
            _ => p.enabled = false,
        });
        let snap = s.snapshot().unwrap();
        assert!(!snap.routers["B"].issues.is_empty());
        assert!(!link(&s, "A", "ab").compatible);
        assert!(snap.routers["A"].route(prefix("172.16.3.0/24")).is_none());
        if kind == "duplicate" {
            assert!(matches!(
                snap.routers["A"].issues[0],
                RouterIssue::DuplicateRouterId { .. }
            ));
        }
    }
    let snap = line().snapshot().unwrap();
    let text = serde_json::to_string(&snap).unwrap();
    assert!(!text.contains("Full") && !text.contains("LSDB"));
}
#[test]
fn router_ids_are_opaque_nonzero_numbers_not_interface_host_addresses() {
    for id in [
        "224.0.0.5",
        "255.255.255.255",
        "240.0.0.1",
        "127.0.0.1",
        "0.0.0.1",
    ] {
        let mut s = line();
        configure_process(&mut s, "B", |p| p.router_id = Some(ip(id)));
        assert!(s.snapshot().unwrap().routers["B"].issues.is_empty(), "{id}");
        assert!(link(&s, "A", "ab").compatible, "{id}");
        assert_eq!(route(&s, "A", "172.16.3.0/24").cost, 14);
        configure_process(&mut s, "C", |p| p.router_id = Some(ip(id)));
        assert!(matches!(
            s.snapshot().unwrap().routers["B"].issues[0],
            RouterIssue::DuplicateRouterId { .. }
        ));
    }
    for (value, expected) in [
        (None, RouterIssue::MissingRouterId),
        (Some(Ipv4Addr::UNSPECIFIED), RouterIssue::ZeroRouterId),
    ] {
        let mut s = line();
        configure_process(&mut s, "B", |p| p.router_id = value);
        assert_eq!(s.snapshot().unwrap().routers["B"].issues, vec![expected]);
    }
}
#[test]
fn passive_port_suppresses_peering_but_retains_local_stub_advertisement() {
    let mut s = line();
    configure_port(&mut s, e("B", "ab"), |p| p.passive = true);
    assert!(!link(&s, "A", "ab").compatible);
    // C still sees the operational B endpoint subnet despite the missing A peer.
    assert_eq!(route(&s, "C", "10.0.0.0/30").cost, 13 + 9);
    assert!(
        s.snapshot().unwrap().routers["C"]
            .route(prefix("172.16.1.0/24"))
            .is_none()
    );
}
#[test]
fn current_up_enable_and_usable_address_changes_recompute_without_history() {
    let original = line();
    for mode in ["down", "disabled", "address", "network-address"] {
        let mut s = original.clone();
        match mode {
            "down" => s
                .apply(&Action::SetUp {
                    endpoint: e("B", "ab"),
                    up: false,
                })
                .unwrap(),
            "disabled" => configure_port(&mut s, e("B", "ab"), |p| p.enabled = false),
            "address" => s
                .apply(&Action::SetAddress {
                    endpoint: e("B", "ab"),
                    address: None,
                })
                .unwrap(),
            _ => s
                .apply(&Action::SetAddress {
                    endpoint: e("B", "ab"),
                    address: Some(Assignment {
                        address: ip("10.0.0.0"),
                        prefix: 30,
                    }),
                })
                .unwrap(),
        }
        assert!(!link(&s, "A", "ab").compatible);
        assert!(
            s.snapshot().unwrap().routers["A"]
                .route(prefix("172.16.3.0/24"))
                .is_none()
        );
        assert!(s.valid_against(&original));
        // A retains its own operational P2P subnet advertisement to the surviving component.
        assert!(
            s.snapshot().unwrap().routers["A"]
                .connected
                .iter()
                .any(|c| c.destination == prefix("10.0.0.0/30"))
        );
    }
    assert_eq!(route(&original, "A", "172.16.3.0/24").cost, 14);
}
#[test]
fn active_broadcast_and_multiaccess_p2p_are_explicitly_unsupported_not_dr_elections() {
    let mut s = line();
    configure_port(&mut s, e("A", "ab"), |p| {
        p.network_type = NetworkType::Broadcast
    });
    assert!(
        link(&s, "A", "ab")
            .conditions
            .iter()
            .any(|c| matches!(c, Condition::UnsupportedActiveBroadcast { .. }))
    );
    let mut t = s.topology().clone();
    let mut c = s.configuration().clone();
    t.devices
        .get_mut("C")
        .unwrap()
        .interfaces
        .get_mut("bc")
        .unwrap()
        .link = "ab".into();
    c.routers
        .get_mut("A")
        .unwrap()
        .interfaces
        .get_mut("ab")
        .unwrap()
        .network_type = NetworkType::PointToPoint;
    let s = State::new(t, c).unwrap();
    assert!(
        link(&s, "A", "ab")
            .conditions
            .iter()
            .any(|c| matches!(c, Condition::UnsupportedSegment { endpoints: 3 }))
    );
}
#[test]
fn maximum_positive_cost_chain_has_checked_u32_arithmetic_and_complete_first_hop() {
    let names: Vec<_> = (0..12).map(|i| format!("R{i}")).collect();
    let labels: Vec<_> = (0..11).map(|i| format!("link{i}")).collect();
    let edges: Vec<_> = (0..11)
        .map(|i| {
            (
                names[i].as_str(),
                names[i + 1].as_str(),
                labels[i].as_str(),
                u16::MAX,
                u16::MAX,
            )
        })
        .collect();
    let s = mesh(&edges, &[("R11", "198.51.100.1", 24, u16::MAX)]);
    let r = route(&s, "R0", "198.51.100.0/24");
    assert_eq!(r.cost, 786420);
    assert_eq!(r.next_hops.len(), 1);
}
#[test]
fn longest_prefix_precedes_lower_cost_and_connected_exact_prefix_is_separate() {
    let s = mesh(
        &[("A", "B", "ab", 1, 1), ("A", "C", "ac", 50, 1)],
        &[
            ("A", "172.22.0.1", 16, 1),
            ("B", "172.22.1.1", 24, 1),
            ("C", "172.22.1.129", 25, 100),
        ],
    );
    let view = &s.snapshot().unwrap().routers["A"];
    let Some(Selection::Ospf(r)) = view.lookup(ip("172.22.1.200")) else {
        panic!()
    };
    assert_eq!(r.destination, prefix("172.22.1.128/25"));
    assert_eq!(r.cost, 150);
    assert!(matches!(
        view.lookup(ip("172.22.2.1")),
        Some(Selection::Connected(_))
    ));
}
#[test]
fn malformed_typed_actions_and_unknown_context_do_not_mutate_state() {
    let mut s = line();
    let before = s.clone();
    let mut bad = port(1, false);
    bad.cost = 0;
    for action in [
        Action::SetPort {
            endpoint: e("A", "ab"),
            configuration: bad,
        },
        Action::SetUp {
            endpoint: e("unknown", "ab"),
            up: false,
        },
        Action::SetProcess {
            router: "A".into(),
            process: ProcessConfig {
                enabled: true,
                process_id: 0,
                router_id: None,
                area: 0,
            },
        },
        Action::SetAddress {
            endpoint: e("A", "ab"),
            address: Some(Assignment {
                address: ip("10.0.0.2"),
                prefix: 30,
            }),
        },
        Action::SetAddress {
            endpoint: e("A", "ab"),
            address: Some(Assignment {
                address: ip("10.0.0.1"),
                prefix: 31,
            }),
        },
    ] {
        assert!(s.apply(&action).is_err());
        assert_eq!(s, before);
    }
    let mut configuration = s.configuration().clone();
    configuration
        .routers
        .get_mut("A")
        .unwrap()
        .interfaces
        .insert("extra".into(), port(1, false));
    assert!(s.reconfigure(&configuration).is_err());
    assert_eq!(s, before);
    let action = Action::SetUp {
        endpoint: e("A", "ab"),
        up: false,
    };
    assert_eq!(
        Action::from_json(&serde_json::to_string(&action).unwrap()).unwrap(),
        action
    );
    assert!(Action::from_json(&"x".repeat(MAX_ACTION_BYTES + 1)).is_err());
    assert!(
        Action::from_json(
            r#"{"kind":"SetUp","endpoint":{"router":"A","interface":"ab"},"up":false,"extra":true}"#
        )
        .is_err()
    );
}
#[test]
fn fixed_context_graph_identity_is_pinned_and_unsupported_static_competition_rejected() {
    let original = line();
    let mut topology = original.topology().clone();
    let config = original.configuration().clone();
    topology
        .devices
        .get_mut("A")
        .unwrap()
        .interfaces
        .get_mut("ab")
        .unwrap()
        .link = "different-link".into();
    let other = State::new(topology, config).unwrap();
    assert!(!other.valid_against(&original));
    let mut topology = original.topology().clone();
    topology
        .devices
        .get_mut("A")
        .unwrap()
        .routes
        .push(dlms_learning_lab::ipv4_routing::Route {
            destination: prefix("192.168.0.0/16"),
            interface: "ab".into(),
            via: ip("10.0.0.2"),
            distance: 1,
        });
    assert_eq!(
        State::new(topology, original.configuration().clone())
            .unwrap_err()
            .kind,
        FaultKind::Unsupported
    );
    let mut topology = original.topology().clone();
    topology
        .devices
        .get_mut("A")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .link = "ab".into();
    assert!(State::new(topology, original.configuration().clone()).is_err());
}
#[test]
fn compact_and_pretty_state_roundtrips_reject_duplicate_keys_and_oversized_inputs() {
    let s = mesh(&[], &[("A", "172.16.0.1", 24, 3)]);
    for text in [
        serde_json::to_string(&s).unwrap(),
        serde_json::to_string_pretty(&s).unwrap(),
    ] {
        assert_eq!(State::from_json(&text).unwrap(), s);
    }
    let t = serde_json::to_string(s.topology()).unwrap();
    let c = serde_json::to_string(&s.configuration().routers["A"]).unwrap();
    let duplicate = [
        r#"{"schema_version":1,"topology":"#,
        &t,
        r#", "configuration":{"routers":{"A":"#,
        &c,
        r#", "A":"#,
        &c,
        "}}}",
    ]
    .concat();
    assert!(State::from_json(&duplicate).is_err());
    assert_eq!(
        State::from_json(&" ".repeat(MAX_STATE_BYTES + 1))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let text = format!(r#"{{"{}":0}}"#, "é".repeat(1000));
    let err = State::from_json(&text).unwrap_err();
    assert!(err.message.len() <= MAX_ERROR_BYTES);
    assert!(err.message.capacity() <= MAX_ERROR_BYTES);
}
#[test]
fn maximum_topology_configuration_and_names_fit_the_persistence_ceiling() {
    let mut topology = Topology {
        devices: BTreeMap::new(),
    };
    let mut configuration = Configuration {
        routers: BTreeMap::new(),
    };
    for r in 0..12 {
        let router = format!("R{r:02}{}", "r".repeat(29));
        let mut d = Device {
            forwarding: true,
            interfaces: BTreeMap::new(),
            routes: vec![],
        };
        let mut ports = BTreeMap::new();
        for p in 0..8 {
            let interface = format!("P{p}{}", "p".repeat(30));
            let segment = format!("L{r:02}{p}{}", "l".repeat(28));
            assert_eq!(segment.len(), 32);
            d.interfaces.insert(
                interface.clone(),
                Interface {
                    link: segment,
                    up: true,
                    address: Some(Assignment {
                        address: Ipv4Addr::new(10, r, p, 1),
                        prefix: 24,
                    }),
                },
            );
            ports.insert(interface, port(u16::MAX, true));
        }
        topology.devices.insert(router.clone(), d);
        configuration.routers.insert(
            router,
            RouterConfig {
                process: ProcessConfig {
                    enabled: true,
                    process_id: u16::MAX,
                    router_id: Some(Ipv4Addr::new(192, 0, 2, r + 1)),
                    area: u32::MAX,
                },
                interfaces: ports,
            },
        );
    }
    let s = State::new(topology, configuration).unwrap();
    for text in [
        serde_json::to_string(&s).unwrap(),
        serde_json::to_string_pretty(&s).unwrap(),
    ] {
        assert!(text.len() < MAX_STATE_BYTES);
        assert_eq!(State::from_json(&text).unwrap(), s);
    }
    assert_eq!(s.snapshot().unwrap().routers.len(), 12);
}

#[test]
fn passive_multiaccess_lan_is_only_a_stub_and_never_an_elected_transit_network() {
    let s = mesh(
        &[
            ("A", "B", "ab", 1, 1),
            ("B", "C", "bc", 1, 1),
            ("C", "D", "cd", 1, 1),
        ],
        &[
            ("A", "192.168.10.1", 24, 1),
            ("B", "192.168.10.2", 24, 1),
            ("C", "192.168.10.3", 24, 1),
        ],
    );
    let mut topology = s.topology().clone();
    for router in ["A", "B", "C"] {
        topology
            .devices
            .get_mut(router)
            .unwrap()
            .interfaces
            .get_mut("lan")
            .unwrap()
            .link = "shared-lan".into();
    }
    let s = State::new(topology, s.configuration().clone()).unwrap();
    let view = link(&s, "A", "lan");
    assert!(!view.compatible && view.peer.is_none());
    assert!(
        view.conditions
            .iter()
            .any(|c| matches!(c, Condition::Passive { .. }))
    );
    assert!(!view.conditions.iter().any(|c| matches!(
        c,
        Condition::UnsupportedSegment { .. } | Condition::UnsupportedActiveBroadcast { .. }
    )));
    assert_eq!(route(&s, "D", "192.168.10.0/24").cost, 2);
}
#[test]
fn borrowed_configuration_limits_and_duplicate_port_decoding_are_rejected_atomically() {
    let mut s = mesh(&[], &[("A", "172.16.0.1", 24, 1)]);
    let before = s.clone();
    let mut config = s.configuration().clone();
    for i in 0..13 {
        config
            .routers
            .insert(format!("Extra{i}"), config.routers["A"].clone());
    }
    assert_eq!(s.reconfigure(&config).unwrap_err().kind, FaultKind::Limit);
    assert_eq!(s, before);
    let mut config = s.configuration().clone();
    for i in 0..9 {
        config
            .routers
            .get_mut("A")
            .unwrap()
            .interfaces
            .insert(format!("Extra{i}"), port(1, true));
    }
    assert!(s.reconfigure(&config).is_err());
    assert_eq!(s, before);
    for field in ["hello", "dead"] {
        let mut config = s.configuration().clone();
        let p = config
            .routers
            .get_mut("A")
            .unwrap()
            .interfaces
            .get_mut("lan")
            .unwrap();
        if field == "hello" {
            p.hello_seconds = 0;
        } else {
            p.dead_seconds = 0;
        }
        assert!(s.reconfigure(&config).is_err());
        assert_eq!(s, before);
    }
    let t = serde_json::to_string(s.topology()).unwrap();
    let process = serde_json::to_string(&s.configuration().routers["A"].process).unwrap();
    let port = serde_json::to_string(&port(1, true)).unwrap();
    let duplicate = [
        r#"{"schema_version":1,"topology":"#,
        &t,
        r#", "configuration":{"routers":{"A":{"process":"#,
        &process,
        r#", "interfaces":{"lan":"#,
        &port,
        r#", "lan":"#,
        &port,
        "}}}}}",
    ]
    .concat();
    assert!(State::from_json(&duplicate).is_err());
    let action = Action::SetUp {
        endpoint: Endpoint {
            router: "A".repeat(1000),
            interface: "lan".into(),
        },
        up: false,
    };
    assert_eq!(s.apply(&action).unwrap_err().kind, FaultKind::Context);
    assert_eq!(s, before);
}
#[test]
fn other_single_area_and_repaired_configurations_are_currently_compatible() {
    let original = line();
    let mut s = original.clone();
    for r in ["A", "B", "C"] {
        configure_process(&mut s, r, |p| p.area = 100);
    }
    assert!(link(&s, "A", "ab").compatible);
    assert_eq!(route(&s, "A", "172.16.3.0/24").cost, 14);
    configure_port(&mut s, e("B", "ab"), |p| p.hello_seconds = 5);
    assert!(!link(&s, "A", "ab").compatible);
    configure_port(&mut s, e("A", "ab"), |p| p.hello_seconds = 5);
    assert!(link(&s, "A", "ab").compatible);
    assert_eq!(route(&s, "A", "172.16.3.0/24").cost, 14);
    assert!(s.valid_against(&original));
}
