//! Original ideal-graph regressions; no host STP/packet/reference execution.
use dlms_learning_lab::spanning_tree::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn graph(n: usize, edges: &[(usize, usize, u32, u32)]) -> State {
    let mut t = Topology {
        system_id_extension: 1,
        bridges: BTreeMap::new(),
        links: BTreeMap::new(),
    };
    let mut c = Configuration {
        bridges: BTreeMap::new(),
        links: BTreeMap::new(),
    };
    for i in 0..n {
        t.bridges.insert(
            format!("b{i}"),
            Bridge {
                mac: Mac([2, 0, 0, 0, 0, (i + 1) as u8]),
                ports: BTreeMap::new(),
            },
        );
        c.bridges.insert(
            format!("b{i}"),
            BridgeConfig {
                priority: 32768,
                online: true,
                ports: BTreeMap::new(),
            },
        );
    }
    for (i, (a, b, ca, cb)) in edges.iter().enumerate() {
        let mut ends = Vec::new();
        for (v, cost) in [(*a, *ca), (*b, *cb)] {
            let bridge = format!("b{v}");
            let count = t.bridges[&bridge].ports.len();
            let port = format!("p{count}");
            t.bridges
                .get_mut(&bridge)
                .unwrap()
                .ports
                .insert(port.clone(), count as u16 + 1);
            c.bridges.get_mut(&bridge).unwrap().ports.insert(
                port.clone(),
                PortConfig {
                    priority: 128,
                    cost,
                    up: true,
                },
            );
            ends.push(Endpoint { bridge, port });
        }
        let id = format!("l{i}");
        t.links.insert(
            id.clone(),
            Link {
                a: ends[0].clone(),
                b: ends[1].clone(),
            },
        );
        c.links.insert(id, true);
    }
    State::new(t, c).unwrap()
}
fn change(s: &mut State, f: impl FnOnce(&mut Configuration)) {
    let mut c = s.configuration().clone();
    f(&mut c);
    s.reconfigure(c).unwrap();
}
fn reject(s: &mut State, a: Action, kind: FaultKind) {
    let b = serde_json::to_vec(s).unwrap();
    assert_eq!(s.apply(&a).unwrap_err().kind, kind);
    assert_eq!(serde_json::to_vec(s).unwrap(), b);
}
#[test]
fn isolated_online_and_offline_bridges_have_honest_component_roots() {
    let mut s = graph(2, &[]);
    let q = s.snapshot().unwrap();
    assert_eq!(q.components().len(), 2);
    for b in ["b0", "b1"] {
        assert_eq!(q.bridge(b).unwrap().root_bridge.as_deref(), Some(b));
        assert_eq!(q.bridge(b).unwrap().root_cost, Some(0));
        assert!(q.bridge(b).unwrap().root_port.is_none());
    }
    assert_eq!(q.forwarding_path("b0", "b1").unwrap(), None);
    assert_eq!(q.forwarding_path("b0", "b0").unwrap(), Some(vec![]));
    s.apply(&Action::BridgeOnline {
        bridge: "b0".into(),
        online: false,
    })
    .unwrap();
    let q = s.snapshot().unwrap();
    assert_eq!(q.components().len(), 1);
    assert!(q.bridge("b0").unwrap().root.is_none());
    assert_eq!(q.forwarding_path("b0", "b0").unwrap(), None);
}
#[test]
fn triangle_designated_root_and_alternate_form_one_forwarding_tree() {
    let s = graph(3, &[(0, 1, 4, 4), (0, 2, 4, 4), (1, 2, 4, 4)]);
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b0").unwrap().root_cost, Some(0));
    assert_eq!(q.bridge("b1").unwrap().root_port.as_deref(), Some("p0"));
    assert_eq!(q.bridge("b2").unwrap().root_port.as_deref(), Some("p0"));
    assert_eq!(q.bridge("b1").unwrap().ports["p1"].role, Role::Designated);
    assert_eq!(q.bridge("b2").unwrap().ports["p1"].role, Role::Alternate);
    assert_eq!(
        q.bridge("b2").unwrap().ports["p1"].state,
        PortState::Discarding
    );
    assert!(!q.links()["l2"].forwarding);
    assert_eq!(
        q.forwarding_path("b1", "b2").unwrap(),
        Some(vec!["l0".into(), "l1".into()])
    );
}
#[test]
fn root_priority_overrides_mac_and_extension_is_part_of_derived_identifier() {
    let mut s = graph(2, &[(0, 1, 4, 4)]);
    s.apply(&Action::BridgePriority {
        bridge: "b1".into(),
        priority: 4096,
    })
    .unwrap();
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b0").unwrap().root_bridge.as_deref(), Some("b1"));
    assert_eq!(q.bridge("b1").unwrap().id.priority, 4097);
    assert_eq!(q.bridge("b0").unwrap().ports["p0"].role, Role::Root);
    assert_eq!(q.bridge("b1").unwrap().ports["p0"].role, Role::Designated);
}
#[test]
fn equal_cost_paths_choose_sender_bridge_identity_before_local_port_priority() {
    let mut s = graph(4, &[(0, 1, 4, 4), (0, 2, 4, 4), (1, 3, 4, 4), (2, 3, 4, 4)]);
    change(&mut s, |c| {
        c.bridges
            .get_mut("b3")
            .unwrap()
            .ports
            .get_mut("p0")
            .unwrap()
            .priority = 240;
        c.bridges
            .get_mut("b3")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .priority = 0;
    });
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b3").unwrap().root_cost, Some(8));
    assert_eq!(q.bridge("b3").unwrap().root_port.as_deref(), Some("p0"));
    assert_eq!(q.bridge("b3").unwrap().ports["p1"].role, Role::Alternate);
}
#[test]
fn parallel_paths_choose_remote_port_id_before_receiving_port_id() {
    let mut s = graph(2, &[(0, 1, 4, 4), (0, 1, 4, 4)]);
    change(&mut s, |c| {
        c.bridges
            .get_mut("b1")
            .unwrap()
            .ports
            .get_mut("p0")
            .unwrap()
            .priority = 240;
        c.bridges
            .get_mut("b1")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .priority = 0;
    });
    assert_eq!(
        s.snapshot()
            .unwrap()
            .bridge("b1")
            .unwrap()
            .root_port
            .as_deref(),
        Some("p0")
    );
    change(&mut s, |c| {
        c.bridges
            .get_mut("b0")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .priority = 0
    });
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b1").unwrap().root_port.as_deref(), Some("p1"));
    assert!(!q.links()["l0"].forwarding);
    assert!(q.links()["l1"].forwarding);
}
#[test]
fn root_paths_add_receiving_local_cost_and_do_not_charge_root_sender_cost() {
    let mut s = graph(
        3,
        &[(0, 1, MAX_COST, 10), (0, 2, MAX_COST, 1), (1, 2, 1, 99)],
    );
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b1").unwrap().root_cost, Some(2));
    assert_eq!(q.bridge("b1").unwrap().root_port.as_deref(), Some("p1"));
    assert_eq!(q.bridge("b2").unwrap().root_cost, Some(1));
    assert_eq!(q.bridge("b2").unwrap().ports["p1"].role, Role::Designated);
    change(&mut s, |c| {
        for p in c.bridges.get_mut("b0").unwrap().ports.values_mut() {
            p.cost = 1;
        }
    });
    assert_eq!(
        s.snapshot().unwrap().bridge("b1").unwrap().root_cost,
        Some(2)
    );
}
#[test]
fn designated_advertisement_uses_bridge_root_cost_not_that_ports_cost() {
    let mut s = graph(3, &[(0, 1, 4, 4), (0, 2, 4, 4), (1, 2, MAX_COST, 1)]);
    let q = s.snapshot().unwrap();
    assert_eq!(q.links()["l2"].designated.as_ref().unwrap().bridge, "b1");
    assert_eq!(q.bridge("b1").unwrap().ports["p1"].role, Role::Designated);
    assert_eq!(q.bridge("b2").unwrap().ports["p1"].role, Role::Alternate);
    change(&mut s, |c| {
        c.bridges
            .get_mut("b2")
            .unwrap()
            .ports
            .get_mut("p0")
            .unwrap()
            .cost = 3
    });
    assert_eq!(
        s.snapshot().unwrap().links()["l2"]
            .designated
            .as_ref()
            .unwrap()
            .bridge,
        "b2"
    );
}
#[test]
fn fresh_snapshot_recalculates_roles_after_link_loss_without_timing_or_history() {
    let mut s = graph(3, &[(0, 1, 4, 4), (0, 2, 4, 4), (1, 2, 4, 4)]);
    let old = s.snapshot().unwrap();
    s.apply(&Action::LinkUp {
        link: "l1".into(),
        up: false,
    })
    .unwrap();
    let now = s.snapshot().unwrap();
    assert_eq!(old.bridge("b2").unwrap().ports["p1"].role, Role::Alternate);
    assert_eq!(now.bridge("b2").unwrap().ports["p1"].role, Role::Root);
    assert_eq!(now.bridge("b2").unwrap().root_cost, Some(8));
    assert_eq!(
        now.bridge("b2").unwrap().ports["p0"].unavailable,
        Some(Unavailable::LinkDown)
    );
    assert_eq!(
        now.forwarding_path("b0", "b2").unwrap(),
        Some(vec!["l0".into(), "l2".into()])
    );
}
#[test]
fn port_peer_and_bridge_availability_are_distinct_and_disconnections_are_not_hidden() {
    let mut s = graph(3, &[(0, 1, 4, 4), (1, 2, 4, 4)]);
    change(&mut s, |c| {
        c.bridges
            .get_mut("b1")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .up = false
    });
    let q = s.snapshot().unwrap();
    assert_eq!(q.components().len(), 2);
    assert_eq!(
        q.bridge("b1").unwrap().ports["p1"].unavailable,
        Some(Unavailable::LocalPortDown)
    );
    assert_eq!(
        q.bridge("b2").unwrap().ports["p0"].unavailable,
        Some(Unavailable::PeerPortDown)
    );
    assert_eq!(q.forwarding_path("b0", "b2").unwrap(), None);
    s.apply(&Action::BridgeOnline {
        bridge: "b0".into(),
        online: false,
    })
    .unwrap();
    let q = s.snapshot().unwrap();
    assert_eq!(
        q.bridge("b0").unwrap().ports["p0"].unavailable,
        Some(Unavailable::BridgeOffline)
    );
    assert_eq!(
        q.bridge("b1").unwrap().ports["p0"].unavailable,
        Some(Unavailable::PeerBridgeOffline)
    );
    assert_eq!(q.bridge("b1").unwrap().root_bridge.as_deref(), Some("b1"));
}
#[test]
fn unconnected_port_is_disabled_not_an_invented_host_edge() {
    let s = graph(1, &[]);
    let (mut t, mut c) = (s.topology().clone(), s.configuration().clone());
    t.bridges
        .get_mut("b0")
        .unwrap()
        .ports
        .insert("unused".into(), 4095);
    c.bridges.get_mut("b0").unwrap().ports.insert(
        "unused".into(),
        PortConfig {
            priority: 240,
            cost: 1,
            up: true,
        },
    );
    let q = State::new(t, c).unwrap().snapshot().unwrap();
    let p = &q.bridge("b0").unwrap().ports["unused"];
    assert_eq!(p.role, Role::Disabled);
    assert_eq!(p.state, PortState::Discarding);
    assert_eq!(p.unavailable, Some(Unavailable::NoLink));
    assert_eq!(p.id.encoded(), 65535);
}
#[test]
fn maximum_directed_cost_path_uses_checked_u32_arithmetic() {
    let edges: Vec<_> = (0..11).map(|i| (i, i + 1, MAX_COST, MAX_COST)).collect();
    let s = graph(12, &edges);
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b11").unwrap().root_cost, Some(2_200_000_000));
    assert_eq!(q.forwarding_path("b0", "b11").unwrap().unwrap().len(), 11);
}
#[test]
fn typed_priority_vectors_order_all_fields_and_use_no_label_tie_break() {
    let low = BridgeId {
        priority: 1,
        mac: Mac([2, 0, 0, 0, 0, 1]),
    };
    let high = BridgeId {
        priority: 1,
        mac: Mac([2, 0, 0, 0, 0, 2]),
    };
    let p = PortId {
        priority: 128,
        number: 1,
    };
    let base = RootPathPriority {
        root: low,
        cost: 4,
        designated_bridge: low,
        designated_port: p,
        receiving_port: p,
    };
    for worse in [
        RootPathPriority { root: high, ..base },
        RootPathPriority { cost: 5, ..base },
        RootPathPriority {
            designated_bridge: high,
            ..base
        },
        RootPathPriority {
            designated_port: PortId { priority: 144, ..p },
            ..base
        },
        RootPathPriority {
            receiving_port: PortId { number: 2, ..p },
            ..base
        },
    ] {
        assert!(base < worse);
    }
    let a = Advertisement {
        root: low,
        cost: 4,
        bridge: low,
        port: p,
    };
    assert!(a < Advertisement { bridge: high, ..a });
}
#[test]
fn malformed_ambiguous_or_unsupported_graphs_reject_before_calculation() {
    let s = graph(2, &[(0, 1, 4, 4)]);
    for mutate in 0..6 {
        let mut t = s.topology().clone();
        match mutate {
            0 => t.bridges.get_mut("b1").unwrap().mac = t.bridges["b0"].mac,
            1 => t.links.get_mut("l0").unwrap().b.bridge = "missing".into(),
            2 => t.links.get_mut("l0").unwrap().b = t.links["l0"].a.clone(),
            3 => {
                t.links.insert("again".into(), t.links["l0"].clone());
            }
            4 => t.bridges.get_mut("b0").unwrap().mac = Mac([1, 0, 0, 0, 0, 1]),
            _ => t.system_id_extension = 4096,
        };
        assert!(t.validate().is_err());
    }
    let mut t = s.topology().clone();
    t.bridges
        .get_mut("b0")
        .unwrap()
        .ports
        .insert("duplicate-number".into(), 1);
    assert_eq!(t.validate().unwrap_err().kind, FaultKind::Identity);
}
#[test]
fn invalid_unknown_and_malformed_actions_are_atomic_and_bounded() {
    let mut s = graph(2, &[(0, 1, 4, 4)]);
    for a in [
        Action::BridgePriority {
            bridge: "b0".into(),
            priority: 1,
        },
        Action::Port {
            bridge: "b0".into(),
            port: "p0".into(),
            settings: PortConfig {
                priority: 128,
                cost: 0,
                up: true,
            },
        },
        Action::Port {
            bridge: "b0".into(),
            port: "p0".into(),
            settings: PortConfig {
                priority: 1,
                cost: 1,
                up: true,
            },
        },
        Action::Port {
            bridge: "b0".into(),
            port: "p0".into(),
            settings: PortConfig {
                priority: 128,
                cost: MAX_COST + 1,
                up: true,
            },
        },
    ] {
        reject(&mut s, a, FaultKind::Invalid);
    }
    reject(
        &mut s,
        Action::BridgeOnline {
            bridge: "missing".into(),
            online: false,
        },
        FaultKind::Identity,
    );
    reject(
        &mut s,
        Action::LinkUp {
            link: "missing".into(),
            up: false,
        },
        FaultKind::Identity,
    );
    assert!(Action::from_json(br#"{"kind":"Port","bridge":"b0","port":"p0","settings":{"priority":128,"cost":0,"up":true}}"#).is_err());
    assert!(
        Action::from_json(
            br#"{"kind":"BridgeOnline","bridge":"b0","online":true,"command":"anything"}"#
        )
        .is_err()
    );
    assert_eq!(
        Action::from_json(&vec![b' '; MAX_ACTION_BYTES + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
}
#[test]
fn immutable_topology_context_pins_ids_macs_port_numbers_links_and_extension() {
    let s = graph(3, &[(0, 1, 4, 4), (1, 2, 4, 4)]);
    for mutate in 0..4 {
        let mut t = s.topology().clone();
        match mutate {
            0 => t.system_id_extension = 2,
            1 => t.bridges.get_mut("b0").unwrap().mac = Mac([2, 0, 0, 0, 1, 1]),
            2 => {
                *t.bridges
                    .get_mut("b0")
                    .unwrap()
                    .ports
                    .get_mut("p0")
                    .unwrap() = 100
            }
            _ => {
                let l = t.links.get_mut("l0").unwrap();
                std::mem::swap(&mut l.a, &mut l.b);
            }
        }
        assert!(t.validate().is_ok());
        assert_eq!(s.validate_context(&t).unwrap_err().kind, FaultKind::Context);
    }
    let mut c = s.configuration().clone();
    c.bridges.get_mut("b0").unwrap().ports.clear();
    let before = serde_json::to_vec(&s).unwrap();
    let mut current = s.clone();
    assert!(current.reconfigure(c).is_err());
    assert_eq!(serde_json::to_vec(&current).unwrap(), before);
}
#[test]
fn raw_and_typed_readers_reject_duplicate_keys_excess_values_and_unknown_fields() {
    let s = graph(2, &[(0, 1, 4, 4)]);
    let bytes = serde_json::to_vec(&s).unwrap();
    let source = String::from_utf8(bytes.clone()).unwrap();
    let duplicate = source.replacen(
        "\"system_id_extension\":1",
        "\"system_id_extension\":1,\"system_id_extension\":1",
        1,
    );
    assert!(State::from_json(duplicate.as_bytes()).is_err());
    let mut v: Value = serde_json::from_slice(&bytes).unwrap();
    v["extra"] = true.into();
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v: Value = serde_json::from_slice(&bytes).unwrap();
    v["configuration"]["bridges"]["b0"]["ports"]["x".repeat(33)] =
        json!({"priority":128,"cost":4,"up":true});
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
    assert_eq!(
        State::from_json(&vec![b' '; MAX_STATE_BYTES + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    // The13th typed body is refused after its valid bounded key, before trying to
    // decode the invalid boolean as a Bridge. No raw JSON streaming claim is made.
    let valid = "{\"mac\":\"02:00:00:00:00:01\",\"ports\":{}}";
    let mut fields: Vec<_> = (0..12).map(|i| format!("\"b{i}\":{valid}")).collect();
    fields.push("\"excess\":true".into());
    let raw = format!(
        "{{\"system_id_extension\":0,\"bridges\":{{{}}},\"links\":{{}}}}",
        fields.join(",")
    );
    let err = serde_json::from_str::<Topology>(&raw).unwrap_err();
    assert!(err.to_string().contains("count/duplicate"));
}
#[test]
fn direct_oversized_and_shape_mismatched_configurations_reject_without_publication() {
    let s = graph(2, &[(0, 1, 4, 4)]);
    let mut c = s.configuration().clone();
    let b = c.bridges["b0"].clone();
    for i in 0..13 {
        c.bridges.insert(format!("extra{i}"), b.clone());
    }
    assert_eq!(
        c.validate_for(s.topology()).unwrap_err().kind,
        FaultKind::Limit
    );
    let mut current = s.clone();
    reject(
        &mut current,
        Action::Reconfigure { configuration: c },
        FaultKind::Limit,
    );
    let mut c = s.configuration().clone();
    c.links.clear();
    reject(
        &mut current,
        Action::Reconfigure { configuration: c },
        FaultKind::Context,
    );
}
#[test]
fn scalar_identifier_edges_and_retained_diagnostics_have_fresh_small_bounds() {
    let mut s = graph(2, &[(0, 1, 4, 4)]);
    s.apply(&Action::BridgePriority {
        bridge: "b0".into(),
        priority: 61440,
    })
    .unwrap();
    let q = s.snapshot().unwrap();
    assert_eq!(q.bridge("b0").unwrap().id.priority, 61441);
    let mut t = s.topology().clone();
    t.system_id_extension = 4095;
    let c = s.configuration().clone();
    let q = State::new(t, c).unwrap().snapshot().unwrap();
    assert_eq!(q.bridge("b0").unwrap().id.priority, 65535);
    let unknown = "🦀".repeat(8192);
    let raw = format!("{{\"{}\":null}}", unknown);
    let err = State::from_json(raw.as_bytes()).unwrap_err();
    assert!(err.message.len() <= 512 && err.message.capacity() <= 512);
    assert!(!err.message.contains("🦀"));
    let q = s.snapshot().unwrap();
    assert!(q.forwarding_path(&"x".repeat(1000), "b0").is_err());
}
#[test]
fn maximum_legal_shape_and_identifiers_roundtrip_compact_pretty_and_full_action() {
    let edges: Vec<_> = (0..12)
        .flat_map(|i| (0..4).map(move |_| (i, (i + 1) % 12, MAX_COST, MAX_COST)))
        .collect();
    let s = graph(12, &edges);
    let bridge_name = |old: &str| {
        format!(
            "bridge-{:02}-{}",
            old[1..].parse::<usize>().unwrap(),
            "b".repeat(22)
        )
    };
    let port_name = |old: &str| format!("{old}-{}", "p".repeat(29));
    let mut t = Topology {
        system_id_extension: 4095,
        bridges: BTreeMap::new(),
        links: BTreeMap::new(),
    };
    let mut c = Configuration {
        bridges: BTreeMap::new(),
        links: BTreeMap::new(),
    };
    for (k, b) in &s.topology().bridges {
        let ports: BTreeMap<_, _> = b
            .ports
            .keys()
            .enumerate()
            .map(|(n, p)| (port_name(p), 4095 - n as u16))
            .collect();
        let cfg = ports
            .keys()
            .map(|p| {
                (
                    p.clone(),
                    PortConfig {
                        priority: 240,
                        cost: MAX_COST,
                        up: true,
                    },
                )
            })
            .collect();
        t.bridges
            .insert(bridge_name(k), Bridge { mac: b.mac, ports });
        c.bridges.insert(
            bridge_name(k),
            BridgeConfig {
                priority: 61440,
                online: true,
                ports: cfg,
            },
        );
    }
    for (i, l) in s.topology().links.values().enumerate() {
        let id = format!("link-{i:02}-{}", "l".repeat(24));
        let endpoint = |e: &Endpoint| Endpoint {
            bridge: bridge_name(&e.bridge),
            port: port_name(&e.port),
        };
        t.links.insert(
            id.clone(),
            Link {
                a: endpoint(&l.a),
                b: endpoint(&l.b),
            },
        );
        c.links.insert(id, true);
    }
    let s = State::new(t, c).unwrap();
    assert_eq!(s.topology().bridges.len(), 12);
    assert_eq!(s.topology().links.len(), 48);
    assert_eq!(
        s.topology()
            .bridges
            .values()
            .map(|b| b.ports.len())
            .sum::<usize>(),
        96
    );
    for bytes in [
        serde_json::to_vec(&s).unwrap(),
        serde_json::to_vec_pretty(&s).unwrap(),
    ] {
        assert!(bytes.len() <= MAX_STATE_BYTES);
        assert_eq!(State::from_json(&bytes).unwrap(), s);
    }
    let action = Action::Reconfigure {
        configuration: s.configuration().clone(),
    };
    let bytes = serde_json::to_vec_pretty(&action).unwrap();
    assert!(bytes.len() <= MAX_ACTION_BYTES);
    assert_eq!(Action::from_json(&bytes).unwrap(), action);
    let q = s.snapshot().unwrap();
    assert_eq!(q.links().values().filter(|l| l.forwarding).count(), 11);
    assert_eq!(q.bridges().len(), 12);
}
fn raw_bid(s: &State, b: &str) -> BridgeId {
    BridgeId {
        priority: s.configuration().bridges[b].priority | s.topology().system_id_extension,
        mac: s.topology().bridges[b].mac,
    }
}
fn raw_pid(s: &State, e: &Endpoint) -> PortId {
    PortId {
        priority: s.configuration().bridges[&e.bridge].ports[&e.port].priority,
        number: s.topology().bridges[&e.bridge].ports[&e.port],
    }
}
// Independently enumerate every simple rootward path in a tiny graph. Unlike
// production relaxation this explores complete paths and excludes revisits.
fn enumerate(
    s: &State,
    node: &str,
    target: &str,
    seen: &mut BTreeSet<String>,
    cost: u32,
    first: Option<(Endpoint, Endpoint)>,
    out: &mut Vec<(u32, Option<(Endpoint, Endpoint)>)>,
) {
    if node == target {
        out.push((cost, first));
        return;
    }
    for l in s.topology().links.values() {
        let (local, peer) = if l.a.bridge == node {
            (&l.a, &l.b)
        } else if l.b.bridge == node {
            (&l.b, &l.a)
        } else {
            continue;
        };
        if seen.contains(&peer.bridge) {
            continue;
        }
        let cost = cost + s.configuration().bridges[node].ports[&local.port].cost;
        seen.insert(peer.bridge.clone());
        enumerate(
            s,
            &peer.bridge,
            target,
            seen,
            cost,
            first.clone().or(Some((local.clone(), peer.clone()))),
            out,
        );
        seen.remove(&peer.bridge);
    }
}
fn oracle_paths(s: &State, from: &str, to: &str) -> Vec<(u32, Option<(Endpoint, Endpoint)>)> {
    let mut out = Vec::new();
    enumerate(
        s,
        from,
        to,
        &mut BTreeSet::from([from.into()]),
        0,
        None,
        &mut out,
    );
    out
}
#[test]
fn independent_simple_path_oracle_covers_all_four_bridge_simple_graphs_and_cost_orders() {
    let possible = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    for seed in 0..4usize {
        for mask in 0..64u8 {
            let edges: Vec<_> = possible
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(i, (a, b))| {
                    (
                        *a,
                        *b,
                        1 + ((i + seed) % 3) as u32,
                        1 + ((i * 2 + seed) % 5) as u32,
                    )
                })
                .collect();
            let mut s = graph(4, &edges);
            if seed > 0 {
                change(&mut s, |c| {
                    c.bridges.get_mut(&format!("b{}", seed)).unwrap().priority = 4096
                });
            }
            let q = s.snapshot().unwrap();
            let mut expected = BTreeMap::new();
            for n in s.topology().bridges.keys() {
                let root = s
                    .topology()
                    .bridges
                    .keys()
                    .filter(|other| !oracle_paths(&s, n, other).is_empty())
                    .min_by_key(|other| raw_bid(&s, other))
                    .unwrap();
                let paths = oracle_paths(&s, n, root);
                let min = paths.iter().map(|p| p.0).min().unwrap();
                let first = if n == root {
                    None
                } else {
                    Some(
                        paths
                            .into_iter()
                            .filter(|p| p.0 == min)
                            .min_by_key(|(_, first)| {
                                let (local, peer) = first.as_ref().unwrap();
                                (
                                    raw_bid(&s, &peer.bridge),
                                    raw_pid(&s, peer),
                                    raw_pid(&s, local),
                                )
                            })
                            .unwrap()
                            .1
                            .unwrap()
                            .0
                            .port,
                    )
                };
                expected.insert(n.clone(), (root.clone(), min, first));
            }
            for (n, (root, cost, port)) in &expected {
                let got = q.bridge(n).unwrap();
                assert_eq!(got.root_bridge.as_ref(), Some(root), "mask{mask}seed{seed}");
                assert_eq!(got.root_cost, Some(*cost));
                assert_eq!(&got.root_port, port);
            }
            for (id, l) in &s.topology().links {
                let ad = |e: &Endpoint| Advertisement {
                    root: raw_bid(&s, &expected[&e.bridge].0),
                    cost: expected[&e.bridge].1,
                    bridge: raw_bid(&s, &e.bridge),
                    port: raw_pid(&s, e),
                };
                let winner = if ad(&l.a) < ad(&l.b) { &l.a } else { &l.b };
                assert_eq!(q.links()[id].designated.as_ref(), Some(winner));
                for e in [&l.a, &l.b] {
                    let wanted = if e == winner {
                        Role::Designated
                    } else if expected[&e.bridge].2.as_ref() == Some(&e.port) {
                        Role::Root
                    } else {
                        Role::Alternate
                    };
                    assert_eq!(q.bridge(&e.bridge).unwrap().ports[&e.port].role, wanted);
                }
            }
            // Independent disjoint-set cycle check on the derived forwarding links.
            let mut parent: BTreeMap<_, _> = s
                .topology()
                .bridges
                .keys()
                .map(|k| (k.clone(), k.clone()))
                .collect();
            let find = |p: &BTreeMap<String, String>, start: &str| {
                let mut n = start.to_owned();
                for _ in 0..4 {
                    if p[&n] == n {
                        return n;
                    }
                    n = p[&n].clone();
                }
                panic!("test union cycle")
            };
            for l in q.links().values().filter(|l| l.forwarding) {
                let a = find(&parent, &l.a.bridge);
                let b = find(&parent, &l.b.bridge);
                assert_ne!(a, b, "derived forwarding cycle");
                parent.insert(a, b);
            }
            for n in s.topology().bridges.keys() {
                assert_eq!(find(&parent, n), find(&parent, &expected[n].0));
            }
        }
    }
}
