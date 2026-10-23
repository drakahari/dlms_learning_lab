use dlms_learning_lab::discovery::{
    Check, Device, Endpoint, ErrorKind, GlobalSettings, Link, MAX_BYTES, MAX_STATE_BYTES, MAX_TIME,
    Outcome, Port, Requirement, State, Topology,
};
use std::collections::BTreeMap;

fn endpoint(device: &str, port: &str) -> Endpoint {
    Endpoint {
        device: device.into(),
        port: port.into(),
    }
}
fn device(name: &str, port_names: &[&str]) -> Device {
    Device {
        chassis_id: format!("chassis-{name}"),
        system_name: Some(name.into()),
        enabled: true,
        ttl: 120,
        ports: port_names
            .iter()
            .map(|p| {
                (
                    p.to_string(),
                    Port {
                        port_id: format!("port-{p}"),
                        description: Some(format!("{name} {p}")),
                        up: true,
                        transmit: true,
                        receive: true,
                    },
                )
            })
            .collect(),
    }
}
fn topology() -> Topology {
    Topology {
        devices: BTreeMap::from([
            ("left".into(), device("left", &["uplink", "unused"])),
            ("middle".into(), device("middle", &["left", "right"])),
            ("right".into(), device("right", &["uplink"])),
        ]),
        links: vec![
            Link {
                id: "left-link".into(),
                a: endpoint("left", "uplink"),
                b: endpoint("middle", "left"),
                up: true,
            },
            Link {
                id: "right-link".into(),
                a: endpoint("middle", "right"),
                b: endpoint("right", "uplink"),
                up: true,
            },
        ],
    }
}
fn state() -> State {
    State::new(topology()).unwrap()
}
fn port_mut<'a>(t: &'a mut Topology, d: &str, p: &str) -> &'a mut Port {
    t.devices.get_mut(d).unwrap().ports.get_mut(p).unwrap()
}
fn saved(s: &State) -> serde_json::Value {
    serde_json::to_value(s).unwrap()
}

#[test]
fn direct_advertisements_are_not_forwarded_or_inferred_from_connectivity() {
    let mut s = state();
    let from = endpoint("left", "uplink");
    let local = endpoint("middle", "left");
    assert!(s.neighbors("middle").unwrap().is_empty());
    assert_eq!(s.advertise(&from).unwrap().outcome, Outcome::Recorded);
    let n = s.neighbor(&local).unwrap().unwrap();
    assert_eq!(n.record.peer, from);
    assert_eq!(n.record.advertisement.chassis_id, "chassis-left");
    assert_eq!(n.record.advertisement.port_id, "port-uplink");
    assert_eq!(n.record.advertisement.system_name.as_deref(), Some("left"));
    assert_eq!(n.remaining_seconds, 120);
    assert!(n.matches_current && n.can_receive);
    assert!(s.neighbors("right").unwrap().is_empty());
    assert!(s.neighbors("left").unwrap().is_empty());
    assert!(s.neighbor(&endpoint("middle", "right")).unwrap().is_none());
    // Middle advertises its own chassis, never relays left's information.
    s.advertise(&endpoint("middle", "right")).unwrap();
    assert_eq!(
        s.neighbor(&endpoint("right", "uplink"))
            .unwrap()
            .unwrap()
            .record
            .advertisement
            .chassis_id,
        "chassis-middle"
    );
    s.validate().unwrap();
}

#[test]
fn transmit_and_receive_are_independent_and_directional() {
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    for bits in 0u8..16 {
        let (atx, arx, btx, brx) = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let mut t = topology();
        port_mut(&mut t, "left", "uplink").transmit = atx;
        port_mut(&mut t, "left", "uplink").receive = arx;
        port_mut(&mut t, "middle", "left").transmit = btx;
        port_mut(&mut t, "middle", "left").receive = brx;
        let mut s = State::new(t).unwrap();
        assert_eq!(
            s.advertise(&a).unwrap().outcome == Outcome::Recorded,
            atx && brx
        );
        assert_eq!(
            s.advertise(&b).unwrap().outcome == Outcome::Recorded,
            btx && arx
        );
        assert_eq!(s.neighbor(&b).unwrap().is_some(), atx && brx);
        assert_eq!(s.neighbor(&a).unwrap().is_some(), btx && arx);
    }
}

#[test]
fn global_disablement_down_ports_links_and_unattached_ports_do_not_discover() {
    let a = endpoint("left", "uplink");
    for kind in 0..5 {
        let mut t = topology();
        match kind {
            0 => t.devices.get_mut("left").unwrap().enabled = false,
            1 => t.devices.get_mut("middle").unwrap().enabled = false,
            2 => port_mut(&mut t, "left", "uplink").up = false,
            3 => port_mut(&mut t, "middle", "left").up = false,
            _ => t.links[0].up = false,
        }
        let mut s = State::new(t).unwrap();
        assert_eq!(
            s.advertise(&a).unwrap().outcome,
            match kind {
                0 => Outcome::TransmitDisabled,
                1 => Outcome::ReceiveDisabled,
                _ => Outcome::LinkDown,
            }
        );
        assert!(s.neighbors("middle").unwrap().is_empty());
    }
    let mut s = state();
    assert_eq!(
        s.advertise(&endpoint("left", "unused")).unwrap().outcome,
        Outcome::Unattached
    );
    assert!(s.advertise(&endpoint("missing", "unused")).is_err());
    assert!(s.neighbor(&endpoint("left", "missing")).is_err());
    assert!(s.neighbors("missing").is_err());
}

#[test]
fn ttl_is_from_sender_refreshes_and_expires_at_exact_boundary() {
    let mut t = topology();
    t.devices.get_mut("left").unwrap().ttl = 10;
    t.devices.get_mut("middle").unwrap().ttl = 1;
    let mut s = State::new(t).unwrap();
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    s.advertise(&a).unwrap();
    s.advance(9).unwrap();
    let n = s.neighbor(&b).unwrap().unwrap();
    assert_eq!((n.age_seconds, n.remaining_seconds), (9, 1));
    s.advertise(&a).unwrap();
    s.advance(9).unwrap();
    let n = s.neighbor(&b).unwrap().unwrap();
    assert_eq!(
        (
            n.record.received_at,
            n.record.expires_at,
            n.remaining_seconds
        ),
        (9, 19, 1)
    );
    s.advance(1).unwrap();
    assert!(s.neighbor(&b).unwrap().is_none());
    // Advancing time does not automatically repopulate a live neighbor.
    s.advance(10_000).unwrap();
    assert!(s.neighbor(&b).unwrap().is_none());
    s.advertise(&a).unwrap();
    assert_eq!(s.neighbor(&b).unwrap().unwrap().remaining_seconds, 10);
}

#[test]
fn historical_fields_are_not_current_until_refreshed_and_queries_never_refresh() {
    let mut s = state();
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    s.advertise(&a).unwrap();
    s.advance(8).unwrap();
    let mut next = s.topology().clone();
    next.devices.get_mut("left").unwrap().system_name = Some("new label".into());
    next.devices.get_mut("left").unwrap().ttl = 2;
    port_mut(&mut next, "left", "uplink").description = None;
    s.reconfigure(next).unwrap();
    let n = s.neighbor(&b).unwrap().unwrap();
    assert!(n.can_receive && !n.matches_current);
    assert_eq!(n.record.advertisement.system_name.as_deref(), Some("left"));
    assert_eq!(n.remaining_seconds, 112);
    assert_eq!(s.neighbor(&b).unwrap().unwrap(), n);
    s.advance(3).unwrap();
    assert_eq!(s.neighbor(&b).unwrap().unwrap().remaining_seconds, 109);
    s.advertise(&a).unwrap();
    let n = s.neighbor(&b).unwrap().unwrap();
    assert!(n.matches_current);
    assert_eq!(n.remaining_seconds, 2);
    assert_eq!(n.record.advertisement.port_description, None);
    s.advance(2).unwrap();
    assert!(s.neighbor(&b).unwrap().is_none());
}

#[test]
fn explicit_shutdown_withdraws_only_the_matching_direction_and_allows_readvertising() {
    let mut s = state();
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    s.advertise(&a).unwrap();
    s.advertise(&b).unwrap();
    assert_eq!(s.shutdown(&a).unwrap().outcome, Outcome::Withdrawn);
    assert!(s.neighbor(&b).unwrap().is_none());
    assert!(s.neighbor(&a).unwrap().is_some());
    assert_eq!(s.shutdown(&a).unwrap().outcome, Outcome::Withdrawn);
    s.advertise(&a).unwrap();
    assert!(s.neighbor(&b).unwrap().is_some());
    assert_eq!(
        s.shutdown(&endpoint("left", "unused")).unwrap().outcome,
        Outcome::Unattached
    );
}

#[test]
fn reconfiguration_clears_current_success_on_receive_disable_or_physical_loss() {
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    for kind in 0..5 {
        let mut s = state();
        s.advertise(&a).unwrap();
        s.advertise(&b).unwrap();
        let mut next = s.topology().clone();
        match kind {
            0 => port_mut(&mut next, "middle", "left").receive = false,
            1 => next.devices.get_mut("middle").unwrap().enabled = false,
            2 => port_mut(&mut next, "middle", "left").up = false,
            3 => next.links[0].up = false,
            _ => port_mut(&mut next, "left", "uplink").up = false,
        }
        s.reconfigure(next).unwrap();
        assert!(s.neighbor(&b).unwrap().is_none());
        assert_eq!(s.neighbor(&a).unwrap().is_some(), kind == 0);
        s.validate().unwrap();
        s.reconfigure(topology()).unwrap();
        assert!(s.neighbor(&b).unwrap().is_none());
        s.advertise(&a).unwrap();
        assert!(s.neighbor(&b).unwrap().unwrap().matches_current);
    }
}

#[test]
fn transmit_disable_withdraws_peer_cache_without_disabling_local_receive() {
    let mut s = state();
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    s.advertise(&a).unwrap();
    s.advertise(&b).unwrap();
    let mut next = s.topology().clone();
    port_mut(&mut next, "left", "uplink").transmit = false;
    s.reconfigure(next).unwrap();
    assert!(s.neighbor(&b).unwrap().is_none());
    assert!(s.neighbor(&a).unwrap().is_some());
    assert_eq!(s.advertise(&a).unwrap().outcome, Outcome::TransmitDisabled);
    assert_eq!(s.shutdown(&a).unwrap().outcome, Outcome::TransmitDisabled);
    assert_eq!(s.advertise(&b).unwrap().outcome, Outcome::Recorded);
    s.reconfigure(topology()).unwrap();
    assert!(s.neighbor(&b).unwrap().is_none());
}

#[test]
fn clock_limits_overflow_and_zero_advance_are_atomic() {
    let mut s = state();
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    s.advertise(&a).unwrap();
    let before = s.clone();
    s.advance(0).unwrap();
    assert_eq!(s, before);
    for delta in [MAX_TIME + 1, u64::MAX] {
        assert_eq!(s.advance(delta).unwrap_err().kind, ErrorKind::Limit);
        assert_eq!(s, before);
    }
    s.advance(MAX_TIME - 1).unwrap();
    s.advertise(&a).unwrap();
    assert_eq!(
        s.neighbor(&b).unwrap().unwrap().record.expires_at,
        MAX_TIME + 119
    );
    s.advance(1).unwrap();
    assert_eq!(s.now(), MAX_TIME);
    let before = s.clone();
    assert!(s.advance(1).is_err());
    assert!(s.advance(u64::MAX).is_err());
    assert_eq!(s, before);
    let roundtrip = State::from_json(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(roundtrip, s);
}

#[test]
fn minimum_and_maximum_ttl_use_exact_independent_arithmetic() {
    for ttl in [1, u16::MAX] {
        let mut t = topology();
        t.devices.get_mut("left").unwrap().ttl = ttl;
        let mut s = State::new(t).unwrap();
        s.advance(12345).unwrap();
        s.advertise(&endpoint("left", "uplink")).unwrap();
        let local = endpoint("middle", "left");
        assert_eq!(s.neighbor(&local).unwrap().unwrap().remaining_seconds, ttl);
        s.advance(u64::from(ttl) - 1).unwrap();
        assert_eq!(s.neighbor(&local).unwrap().unwrap().remaining_seconds, 1);
        s.advance(1).unwrap();
        assert!(s.neighbor(&local).unwrap().is_none());
    }
}

#[test]
fn identities_and_attachments_are_pinned_while_link_order_is_irrelevant() {
    let mut s = state();
    s.advertise(&endpoint("left", "uplink")).unwrap();
    let original = s.clone();
    for kind in 0..7 {
        let mut t = topology();
        match kind {
            0 => t.devices.get_mut("left").unwrap().chassis_id = "different".into(),
            1 => port_mut(&mut t, "left", "uplink").port_id = "different".into(),
            2 => {
                t.devices.remove("right");
                t.links.pop();
            }
            3 => {
                t.devices.get_mut("left").unwrap().ports.remove("unused");
            }
            4 => t.links[0].a = endpoint("left", "unused"),
            5 => t.links[0].id = "different".into(),
            _ => t.links.clear(),
        }
        assert!(!t.same_identity(s.topology()));
        assert_eq!(s.reconfigure(t).unwrap_err().kind, ErrorKind::Identity);
        assert_eq!(s, original);
    }
    let mut next = topology();
    next.links.reverse();
    for l in &mut next.links {
        std::mem::swap(&mut l.a, &mut l.b);
    }
    assert!(next.same_identity(s.topology()));
    s.reconfigure(next).unwrap();
    assert!(s.neighbor(&endpoint("middle", "left")).unwrap().is_some());
}

#[test]
fn invalid_or_ambiguous_graphs_are_rejected_including_programmatic_inputs() {
    for kind in 0..10 {
        let mut t = topology();
        match kind {
            0 => t.links[0].a = endpoint("missing", "uplink"),
            1 => t.links[0].a = endpoint("left", "missing"),
            2 => t.links[0].b = endpoint("left", "unused"),
            3 => t.links[1].id = t.links[0].id.clone(),
            4 => t.links[1].a = t.links[0].a.clone(),
            5 => t.devices.get_mut("left").unwrap().chassis_id = "chassis-right".into(),
            6 => port_mut(&mut t, "left", "unused").port_id = "port-uplink".into(),
            7 => t.devices.get_mut("left").unwrap().ttl = 0,
            8 => {
                t.devices.get_mut("left").unwrap().ports.clear();
            }
            _ => t.devices.clear(),
        }
        assert!(t.validate().is_err(), "case {kind}");
        assert!(State::new(t).is_err());
    }
}

#[test]
fn cycles_and_distinct_parallel_device_links_do_not_require_forwarding() {
    let mut t = topology();
    t.devices.get_mut("right").unwrap().ports.insert(
        "extra".into(),
        device("x", &["extra"]).ports.remove("extra").unwrap(),
    );
    t.links.push(Link {
        id: "cycle".into(),
        a: endpoint("left", "unused"),
        b: endpoint("right", "extra"),
        up: true,
    });
    let mut s = State::new(t).unwrap();
    s.advertise(&endpoint("left", "unused")).unwrap();
    assert_eq!(s.neighbors("right").unwrap().len(), 1);
    assert!(s.neighbors("middle").unwrap().is_empty());
    let mut t = topology();
    t.links[1].b = endpoint("left", "unused");
    t.validate().unwrap();
}

#[test]
fn identifier_and_label_boundaries_are_explicit_and_duplicate_labels_are_allowed() {
    let mut t = topology();
    t.devices.get_mut("left").unwrap().chassis_id = "x".repeat(64);
    t.devices.get_mut("right").unwrap().system_name = Some("middle".into());
    port_mut(&mut t, "left", "unused").description = Some("x".repeat(80));
    t.validate().unwrap();
    for text in ["", " space", "non asciié", "has\nnewline", &"x".repeat(65)] {
        let mut t = topology();
        t.devices.get_mut("left").unwrap().chassis_id = text.into();
        assert!(t.validate().is_err());
    }
    for text in ["", "  ", "non asciié", "tab\there", &"x".repeat(81)] {
        let mut t = topology();
        port_mut(&mut t, "left", "unused").description = Some(text.into());
        assert!(t.validate().is_err());
    }
}

#[test]
fn bounded_topology_supports_all_ports_without_growing_a_neighbor_per_event() {
    let mut t = Topology {
        devices: BTreeMap::new(),
        links: Vec::new(),
    };
    for i in 0..12 {
        let names: Vec<String> = (0..16).map(|p| format!("p{p}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        t.devices
            .insert(format!("d{i}"), device(&format!("d{i}"), &refs));
    }
    for pair in 0..6 {
        for p in 0..16 {
            t.links.push(Link {
                id: format!("l{pair}-{p}"),
                a: endpoint(&format!("d{}", pair * 2), &format!("p{p}")),
                b: endpoint(&format!("d{}", pair * 2 + 1), &format!("p{p}")),
                up: true,
            });
        }
    }
    t.validate().unwrap();
    let mut s = State::new(t.clone()).unwrap();
    for _ in 0..3 {
        for d in t.devices.keys() {
            for p in t.devices[d].ports.keys() {
                s.advertise(&endpoint(d, p)).unwrap();
            }
        }
    }
    assert_eq!(
        (0..12)
            .map(|i| s.neighbors(&format!("d{i}")).unwrap().len())
            .sum::<usize>(),
        192
    );
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.len() < MAX_BYTES);
    assert_eq!(State::from_json(&json).unwrap(), s);
    let mut excessive = t.clone();
    excessive
        .devices
        .insert("extra".into(), device("extra", &["p"]));
    assert_eq!(excessive.validate().unwrap_err().kind, ErrorKind::Limit);
    let mut excessive = t.clone();
    excessive.devices.get_mut("d0").unwrap().ports.insert(
        "extra".into(),
        device("extra", &["p"]).ports.remove("p").unwrap(),
    );
    assert_eq!(excessive.validate().unwrap_err().kind, ErrorKind::Limit);
    let mut excessive = t.clone();
    excessive.links.push(t.links[0].clone());
    assert_eq!(excessive.validate().unwrap_err().kind, ErrorKind::Limit);
    for t in [excessive, {
        let mut t = t.clone();
        t.devices.insert("extra".into(), device("extra", &["p"]));
        t
    }] {
        assert!(Topology::from_json(&serde_json::to_string(&t).unwrap()).is_err());
    }
}

#[test]
fn serialization_roundtrip_preserves_history_and_rejects_duplicates_and_unknown_features() {
    let mut s = state();
    s.advertise(&endpoint("left", "uplink")).unwrap();
    s.advance(12).unwrap();
    assert_eq!(
        State::from_json(&serde_json::to_string(&s).unwrap()).unwrap(),
        s
    );
    let text = serde_json::to_string(&topology()).unwrap();
    assert_eq!(Topology::from_json(&text).unwrap(), topology());
    // Duplicate keys must be detected during parsing, before BTreeMap can collapse them.
    let duplicate = text.replacen(
        "\"devices\":{",
        &format!(
            "\"devices\":{{\"left\":{},",
            serde_json::to_string(&topology().devices["left"]).unwrap()
        ),
        1,
    );
    assert!(Topology::from_json(&duplicate).is_err());
    let duplicate_port = text.replacen(
        "\"ports\":{",
        &format!(
            "\"ports\":{{\"unused\":{},",
            serde_json::to_string(&topology().devices["left"].ports["unused"]).unwrap()
        ),
        1,
    );
    assert!(Topology::from_json(&duplicate_port).is_err());
    for field in ["cdp", "media", "management_address", "subtype"] {
        let mut v = serde_json::to_value(topology()).unwrap();
        v["devices"]["left"][field] = serde_json::json!(true);
        assert!(Topology::from_json(&v.to_string()).is_err());
    }
    assert_eq!(
        State::from_json(&" ".repeat(MAX_STATE_BYTES + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(Topology::from_json(&" ".repeat(MAX_BYTES + 1)).is_err());
    let mut v = saved(&s);
    let duplicate = v["cache"][0].clone();
    v["cache"].as_array_mut().unwrap().push(duplicate);
    assert!(State::from_json(&v.to_string()).is_err());
}

#[test]
fn malformed_or_forged_cache_shapes_cannot_enter_runtime_state() {
    let mut s = state();
    s.advertise(&endpoint("left", "uplink")).unwrap();
    s.advance(1).unwrap();
    for kind in 0..13 {
        let mut v = saved(&s);
        match kind {
            0 => v["now"] = serde_json::json!(MAX_TIME + 1),
            1 => v["cache"][0]["received_at"] = serde_json::json!(2),
            2 => v["cache"][0]["expires_at"] = serde_json::json!(1),
            3 => v["cache"][0]["expires_at"] = serde_json::json!(u64::MAX),
            4 => v["cache"][0]["advertisement"]["ttl"] = serde_json::json!(0),
            5 => v["cache"][0]["advertisement"]["chassis_id"] = serde_json::json!("chassis-right"),
            6 => v["cache"][0]["advertisement"]["port_id"] = serde_json::json!("wrong"),
            7 => v["cache"][0]["peer"]["device"] = serde_json::json!("right"),
            8 => v["cache"][0]["local"]["port"] = serde_json::json!("right"),
            9 => {
                v["topology"]["devices"]["middle"]["ports"]["left"]["receive"] =
                    serde_json::json!(false)
            }
            10 => v["topology"]["links"][0]["up"] = serde_json::json!(false),
            11 => v["cache"][0]["advertisement"]["system_name"] = serde_json::json!(" "),
            _ => v["cache"][0]["local"]["device"] = serde_json::json!("unknown"),
        }
        assert!(State::from_json(&v.to_string()).is_err(), "case {kind}");
    }
    let mut v = saved(&s);
    let entry = v["cache"][0].clone();
    v["cache"] = serde_json::json!(vec![entry; 193]);
    assert!(State::from_json(&v.to_string()).is_err());
}

#[test]
fn restored_historical_records_are_not_claimed_as_current_connectivity() {
    let mut s = state();
    s.advertise(&endpoint("left", "uplink")).unwrap();
    let mut v = saved(&s);
    v["topology"]["devices"]["left"]["ports"]["uplink"]["transmit"] = serde_json::json!(false);
    // History is structurally valid, not an authenticated event log. Current query remains honest.
    let s = State::from_json(&v.to_string()).unwrap();
    let n = s.neighbor(&endpoint("middle", "left")).unwrap().unwrap();
    assert!(!n.can_receive && !n.matches_current);
    assert_eq!(n.remaining_seconds, 120);
}

#[test]
fn invalid_reconfiguration_and_unknown_actions_preserve_all_state() {
    let mut s = state();
    s.advertise(&endpoint("left", "uplink")).unwrap();
    s.advance(5).unwrap();
    let before = s.clone();
    let mut bad = s.topology().clone();
    bad.devices.get_mut("left").unwrap().ttl = 0;
    assert_eq!(s.reconfigure(bad).unwrap_err().kind, ErrorKind::Invalid);
    assert_eq!(s, before);
    let unknown = endpoint("left", "unknown");
    assert!(s.advertise(&unknown).is_err());
    assert!(s.shutdown(&unknown).is_err());
    assert!(s.delivery(&unknown).is_err());
    assert_eq!(s, before);
    assert_eq!(
        s.delivery(&endpoint("left", "uplink")).unwrap().outcome,
        Outcome::Recorded
    );
    assert_eq!(s, before);
}

#[test]
fn clock_partitioning_and_event_order_do_not_depend_on_wall_time() {
    let mut one = state();
    let mut split = state();
    let a = endpoint("left", "uplink");
    let b = endpoint("middle", "left");
    one.advertise(&a).unwrap();
    one.advertise(&b).unwrap();
    split.advertise(&b).unwrap();
    split.advertise(&a).unwrap();
    assert_eq!(one, split);
    one.advance(119).unwrap();
    for _ in 0..119 {
        split.advance(1).unwrap();
    }
    assert_eq!(one, split);
    one.advance(1).unwrap();
    split.advance(1).unwrap();
    assert_eq!(one, split);
    assert!(one.neighbors("left").unwrap().is_empty());
}

#[test]
fn internal_key_and_port_collection_limits_hold_at_json_and_typed_boundaries() {
    let minimal = Topology {
        devices: BTreeMap::from([("d".into(), device("d", &["p"]))]),
        links: Vec::new(),
    };
    for name in ["", "UPPER", "has space", "has_underscore", &"x".repeat(65)] {
        let mut t = minimal.clone();
        let d = t.devices.remove("d").unwrap();
        t.devices.insert(name.into(), d);
        assert!(t.validate().is_err());
        assert!(Topology::from_json(&serde_json::to_string(&t).unwrap()).is_err());
    }
    let mut t = minimal.clone();
    let d = t.devices.remove("d").unwrap();
    t.devices.insert("x".repeat(64), d);
    t.validate().unwrap();
    Topology::from_json(&serde_json::to_string(&t).unwrap()).unwrap();
    let mut t = minimal;
    for i in 0..16 {
        t.devices.get_mut("d").unwrap().ports.insert(
            format!("extra-{i}"),
            Port {
                port_id: format!("extra-{i}"),
                description: None,
                up: false,
                transmit: false,
                receive: false,
            },
        );
    }
    assert_eq!(t.validate().unwrap_err().kind, ErrorKind::Limit);
    assert!(Topology::from_json(&serde_json::to_string(&t).unwrap()).is_err());
}

#[test]
fn maximum_shape_and_worst_legal_escaping_roundtrip_compact_and_pretty_snapshots() {
    fn max_key(prefix: &str, n: usize) -> String {
        let start = format!("{prefix}{n:03}");
        format!("{start}{}", "x".repeat(64 - start.len()))
    }
    fn escaped_identity(n: usize) -> String {
        // All 64 legal ASCII characters require two bytes in JSON. Last four
        // characters distinguish the twelve chassis or sixteen local ports.
        let mut text = "\\".repeat(60);
        for bit in 0..4 {
            text.push(if n & (1 << bit) == 0 { '\\' } else { '"' });
        }
        text
    }
    let mut t = Topology {
        devices: BTreeMap::new(),
        links: Vec::new(),
    };
    for d in 0..12 {
        let ports = (0..16)
            .map(|p| {
                (
                    max_key("p", p),
                    Port {
                        port_id: escaped_identity(p),
                        description: Some("\\\"".repeat(40)),
                        up: true,
                        transmit: true,
                        receive: true,
                    },
                )
            })
            .collect();
        t.devices.insert(
            max_key("d", d),
            Device {
                chassis_id: escaped_identity(d),
                system_name: Some("\\\"".repeat(40)),
                enabled: true,
                ttl: u16::MAX,
                ports,
            },
        );
    }
    for d in (0..12).step_by(2) {
        for p in 0..16 {
            t.links.push(Link {
                id: max_key("l", d * 16 + p),
                a: endpoint(&max_key("d", d), &max_key("p", p)),
                b: endpoint(&max_key("d", d + 1), &max_key("p", p)),
                up: true,
            });
        }
    }
    t.validate().unwrap();
    let topology_compact = serde_json::to_string(&t).unwrap();
    let topology_pretty = serde_json::to_string_pretty(&t).unwrap();
    for text in [&topology_compact, &topology_pretty] {
        assert!(text.len() <= MAX_BYTES);
        assert_eq!(Topology::from_json(text).unwrap(), t);
    }
    let endpoints: Vec<_> = t
        .devices
        .iter()
        .flat_map(|(d, device)| device.ports.keys().map(move |p| endpoint(d, p)))
        .collect();
    let mut s = State::new(t).unwrap();
    s.advance(MAX_TIME).unwrap();
    for e in &endpoints {
        s.advertise(e).unwrap();
    }
    s.validate().unwrap();
    assert_eq!(
        endpoints
            .iter()
            .filter(|e| s.neighbor(e).unwrap().is_some())
            .count(),
        192
    );
    let compact = serde_json::to_string(&s).unwrap();
    let pretty = serde_json::to_string_pretty(&s).unwrap();
    assert!(
        compact.len() > MAX_BYTES,
        "regression must exceed the topology-only limit"
    );
    for text in [&compact, &pretty] {
        assert!(text.len() <= MAX_STATE_BYTES);
        assert_eq!(State::from_json(text).unwrap(), s);
    }
    eprintln!(
        "worst escaped topology: compact {}, pretty {}; state: compact {}, pretty {} bytes",
        topology_compact.len(),
        topology_pretty.len(),
        compact.len(),
        pretty.len()
    );
    // Loader caps are independent, and byte excess fails before parsing.
    assert_eq!(
        Topology::from_json(&" ".repeat(MAX_BYTES + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        State::from_json(&" ".repeat(MAX_STATE_BYTES + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        Topology::from_json(&" ".repeat(MAX_BYTES))
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
    assert_eq!(
        State::from_json(&" ".repeat(MAX_STATE_BYTES))
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
}

#[test]
fn current_neighbor_requirements_accept_alternate_repairs_with_explicit_preservation() {
    let mut initial = topology();
    initial.devices.get_mut("left").unwrap().enabled = false;
    port_mut(&mut initial, "left", "uplink").transmit = false;
    let local = endpoint("middle", "left");
    let peer = endpoint("left", "uplink");
    let current = Requirement::CurrentNeighbor {
        local,
        peer: peer.clone(),
    };
    let guards = [
        Requirement::GlobalConfiguration {
            device: "middle".into(),
            expected: GlobalSettings::from_device(&initial.devices["middle"]),
        },
        Requirement::PortConfiguration {
            local: endpoint("left", "unused"),
            expected: initial.devices["left"].ports["unused"].clone(),
        },
        Requirement::PortConfiguration {
            local: endpoint("middle", "right"),
            expected: initial.devices["middle"].ports["right"].clone(),
        },
    ];
    for receive in [false, true] {
        let mut s = State::new(initial.clone()).unwrap();
        assert_eq!(
            current.check(&initial, &s).unwrap(),
            Check::MissingCurrentNeighbor
        );
        assert_eq!(
            s.advertise(&peer).unwrap().outcome,
            Outcome::TransmitDisabled
        );
        let mut repaired = initial.clone();
        repaired.devices.get_mut("left").unwrap().enabled = true;
        port_mut(&mut repaired, "left", "uplink").transmit = true;
        port_mut(&mut repaired, "left", "uplink").receive = receive;
        s.reconfigure(repaired).unwrap();
        assert!(!current.check(&initial, &s).unwrap().passed());
        s.advertise(&peer).unwrap();
        assert!(current.check(&initial, &s).unwrap().passed());
        for guard in &guards {
            assert!(guard.check(&initial, &s).unwrap().passed());
        }
        let mut changed = s.topology().clone();
        port_mut(&mut changed, "left", "unused").receive = false;
        s.reconfigure(changed).unwrap();
        assert!(current.check(&initial, &s).unwrap().passed());
        assert_eq!(
            guards[1].check(&initial, &s).unwrap(),
            Check::ConfigurationMismatch
        );
    }
}

#[test]
fn requirements_reject_stale_optional_fields_and_nondeliverable_historical_cache() {
    let initial = topology();
    let local = endpoint("middle", "left");
    let peer = endpoint("left", "uplink");
    let requirement = Requirement::CurrentNeighbor {
        local: local.clone(),
        peer: peer.clone(),
    };
    for changed_field in 0..3 {
        let mut s = State::new(initial.clone()).unwrap();
        s.advertise(&peer).unwrap();
        assert!(requirement.check(&initial, &s).unwrap().passed());
        let mut next = initial.clone();
        match changed_field {
            0 => next.devices.get_mut("left").unwrap().system_name = None,
            1 => next.devices.get_mut("left").unwrap().ttl = 119,
            _ => {
                port_mut(&mut next, "left", "uplink").description =
                    Some("Changed description".into())
            }
        }
        s.reconfigure(next).unwrap();
        assert_eq!(
            requirement.check(&initial, &s).unwrap(),
            Check::HistoricalNeighbor
        );
        assert!(s.neighbor(&local).unwrap().unwrap().remaining_seconds > 0);
        s.advertise(&peer).unwrap();
        assert!(requirement.check(&initial, &s).unwrap().passed());
    }
    let mut s = State::new(initial.clone()).unwrap();
    s.advertise(&peer).unwrap();
    let mut historical = saved(&s);
    historical["topology"]["devices"]["left"]["ports"]["uplink"]["transmit"] =
        serde_json::json!(false);
    let historical = State::from_json(&historical.to_string()).unwrap();
    assert_eq!(
        requirement.check(&initial, &historical).unwrap(),
        Check::HistoricalNeighbor
    );
    assert_eq!(
        Requirement::AbsentNeighbor { local }
            .check(&initial, &historical)
            .unwrap(),
        Check::UnexpectedNeighbor
    );
}

#[test]
fn positive_and_absent_neighbor_requirements_invalidate_at_the_exact_expiry_boundary() {
    let mut initial = topology();
    initial.devices.get_mut("left").unwrap().ttl = 2;
    let local = endpoint("middle", "left");
    let peer = endpoint("left", "uplink");
    let positive = Requirement::CurrentNeighbor {
        local: local.clone(),
        peer: peer.clone(),
    };
    let absent = Requirement::AbsentNeighbor { local };
    let mut s = State::new(initial.clone()).unwrap();
    assert!(absent.check(&initial, &s).unwrap().passed());
    s.advertise(&peer).unwrap();
    s.advance(1).unwrap();
    assert!(positive.check(&initial, &s).unwrap().passed());
    assert_eq!(
        absent.check(&initial, &s).unwrap(),
        Check::UnexpectedNeighbor
    );
    s.advance(1).unwrap();
    assert_eq!(
        positive.check(&initial, &s).unwrap(),
        Check::MissingCurrentNeighbor
    );
    assert!(absent.check(&initial, &s).unwrap().passed());
    s.advertise(&peer).unwrap();
    assert!(positive.check(&initial, &s).unwrap().passed());
    s.shutdown(&peer).unwrap();
    assert!(!positive.check(&initial, &s).unwrap().passed());
    assert!(absent.check(&initial, &s).unwrap().passed());
}

#[test]
fn requirement_validation_pins_expected_peer_and_whole_current_identity() {
    let initial = topology();
    let local = endpoint("middle", "left");
    for peer in [
        endpoint("middle", "left"),
        endpoint("right", "uplink"),
        endpoint("left", "unused"),
        endpoint("missing", "uplink"),
    ] {
        assert!(
            Requirement::CurrentNeighbor {
                local: local.clone(),
                peer
            }
            .validate(&initial)
            .is_err()
        );
    }
    assert!(
        Requirement::AbsentNeighbor {
            local: endpoint("middle", "missing")
        }
        .validate(&initial)
        .is_err()
    );
    let expected = Requirement::CurrentNeighbor {
        local: local.clone(),
        peer: endpoint("left", "uplink"),
    };
    let mut changed = initial.clone();
    changed.links[0].a = endpoint("left", "unused");
    let mut s = State::new(changed).unwrap();
    s.advertise(&endpoint("left", "unused")).unwrap();
    assert_eq!(
        expected.check(&initial, &s).unwrap(),
        Check::IdentityChanged
    );
    assert_eq!(
        Requirement::AbsentNeighbor { local }
            .check(&initial, &s)
            .unwrap(),
        Check::IdentityChanged
    );
    let mut changed = initial.clone();
    changed.devices.get_mut("left").unwrap().chassis_id = "another-chassis".into();
    let s = State::new(changed).unwrap();
    assert_eq!(
        Requirement::Configuration {
            expected: initial.clone()
        }
        .check(&initial, &s)
        .unwrap(),
        Check::IdentityChanged
    );
}

#[test]
fn configuration_requirements_preserve_whole_ports_global_settings_and_full_configuration() {
    let initial = topology();
    let global = Requirement::GlobalConfiguration {
        device: "left".into(),
        expected: GlobalSettings::from_device(&initial.devices["left"]),
    };
    let port = Requirement::PortConfiguration {
        local: endpoint("left", "uplink"),
        expected: initial.devices["left"].ports["uplink"].clone(),
    };
    let all = Requirement::Configuration {
        expected: initial.clone(),
    };
    for kind in 0..8 {
        let mut s = State::new(initial.clone()).unwrap();
        let mut changed = initial.clone();
        match kind {
            0 => changed.devices.get_mut("left").unwrap().enabled = false,
            1 => changed.devices.get_mut("left").unwrap().ttl = 1,
            2 => changed.devices.get_mut("left").unwrap().system_name = None,
            3 => port_mut(&mut changed, "left", "uplink").up = false,
            4 => port_mut(&mut changed, "left", "uplink").transmit = false,
            5 => port_mut(&mut changed, "left", "uplink").receive = false,
            6 => port_mut(&mut changed, "left", "uplink").description = None,
            _ => changed.links[0].up = false,
        }
        s.reconfigure(changed).unwrap();
        assert_eq!(global.check(&initial, &s).unwrap().passed(), kind > 2);
        assert_eq!(
            port.check(&initial, &s).unwrap().passed(),
            kind < 3 || kind == 7
        );
        assert_eq!(
            all.check(&initial, &s).unwrap(),
            Check::ConfigurationMismatch
        );
        s.reconfigure(initial.clone()).unwrap();
        assert!(all.check(&initial, &s).unwrap().passed());
    }
    let mut order = initial.clone();
    order.links.reverse();
    for link in &mut order.links {
        std::mem::swap(&mut link.a, &mut link.b);
    }
    assert!(
        all.check(&initial, &State::new(order).unwrap())
            .unwrap()
            .passed()
    );
}

#[test]
fn absence_outcome_cannot_replace_explicit_physical_and_unrelated_configuration_guards() {
    let initial = topology();
    let local = endpoint("middle", "left");
    let absent = Requirement::AbsentNeighbor {
        local: local.clone(),
    };
    let guard = Requirement::PortConfiguration {
        local,
        expected: initial.devices["middle"].ports["left"].clone(),
    };
    let mut s = State::new(initial.clone()).unwrap();
    s.advertise(&endpoint("left", "uplink")).unwrap();
    assert!(!absent.check(&initial, &s).unwrap().passed());
    let mut down = initial.clone();
    port_mut(&mut down, "middle", "left").up = false;
    s.reconfigure(down).unwrap();
    assert!(absent.check(&initial, &s).unwrap().passed());
    assert_eq!(
        guard.check(&initial, &s).unwrap(),
        Check::ConfigurationMismatch
    );
    // Port/global preservation does not guard the link's own up flag.
    // Authors of absence tasks must use the whole configuration (or a positive
    // current neighbor on this same attachment), not just these granular guards.
    let mut link_down = initial.clone();
    link_down.links[0].up = false;
    s = State::new(initial.clone()).unwrap();
    s.reconfigure(link_down).unwrap();
    assert!(absent.check(&initial, &s).unwrap().passed());
    for (name, device) in &initial.devices {
        assert!(
            Requirement::GlobalConfiguration {
                device: name.clone(),
                expected: GlobalSettings::from_device(device),
            }
            .check(&initial, &s)
            .unwrap()
            .passed()
        );
        for (name_of_port, port) in &device.ports {
            assert!(
                Requirement::PortConfiguration {
                    local: endpoint(name, name_of_port),
                    expected: port.clone(),
                }
                .check(&initial, &s)
                .unwrap()
                .passed()
            );
        }
    }
    assert_eq!(
        Requirement::Configuration {
            expected: initial.clone()
        }
        .check(&initial, &s)
        .unwrap(),
        Check::ConfigurationMismatch
    );
}

#[test]
fn validated_configuration_expectations_may_describe_a_repair_but_never_new_identity() {
    let initial = topology();
    let mut expected = initial.clone();
    expected.devices.get_mut("left").unwrap().enabled = false;
    let all = Requirement::Configuration {
        expected: expected.clone(),
    };
    all.validate(&initial).unwrap();
    let mut s = State::new(initial.clone()).unwrap();
    assert!(!all.check(&initial, &s).unwrap().passed());
    s.reconfigure(expected.clone()).unwrap();
    assert!(all.check(&initial, &s).unwrap().passed());
    let mut port = initial.devices["left"].ports["uplink"].clone();
    port.port_id = "invented-id".into();
    assert_eq!(
        Requirement::PortConfiguration {
            local: endpoint("left", "uplink"),
            expected: port
        }
        .validate(&initial)
        .unwrap_err()
        .kind,
        ErrorKind::Identity
    );
    expected.links[0].a = endpoint("left", "unused");
    assert_eq!(
        Requirement::Configuration { expected }
            .validate(&initial)
            .unwrap_err()
            .kind,
        ErrorKind::Identity
    );
    for settings in [
        GlobalSettings {
            enabled: true,
            ttl: 0,
            system_name: None,
        },
        GlobalSettings {
            enabled: true,
            ttl: 1,
            system_name: Some(" ".into()),
        },
    ] {
        assert!(
            Requirement::GlobalConfiguration {
                device: "left".into(),
                expected: settings
            }
            .validate(&initial)
            .is_err()
        );
    }
}

#[test]
fn requirement_serialization_is_bounded_validated_and_rejects_unknown_fields() {
    let initial = topology();
    let requirements = [
        Requirement::CurrentNeighbor {
            local: endpoint("middle", "left"),
            peer: endpoint("left", "uplink"),
        },
        Requirement::AbsentNeighbor {
            local: endpoint("left", "unused"),
        },
        Requirement::GlobalConfiguration {
            device: "left".into(),
            expected: GlobalSettings::from_device(&initial.devices["left"]),
        },
        Requirement::PortConfiguration {
            local: endpoint("left", "uplink"),
            expected: initial.devices["left"].ports["uplink"].clone(),
        },
        Requirement::Configuration {
            expected: initial.clone(),
        },
    ];
    for requirement in requirements {
        for text in [
            serde_json::to_string(&requirement).unwrap(),
            serde_json::to_string_pretty(&requirement).unwrap(),
        ] {
            assert_eq!(
                Requirement::from_json(&text, &initial).unwrap(),
                requirement
            );
        }
        let mut unknown = serde_json::to_value(&requirement).unwrap();
        unknown["event_history"] = serde_json::json!(true);
        assert!(Requirement::from_json(&unknown.to_string(), &initial).is_err());
    }
    assert_eq!(
        Requirement::from_json(&" ".repeat(MAX_BYTES + 1), &initial)
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(Requirement::from_json(r#"{"kind":"current_neighbor","local":{"device":"middle","port":"left"},"peer":{"device":"right","port":"uplink"}}"#, &initial).is_err());
    assert!(Requirement::from_json(r#"{"kind":"global_configuration","device":"unknown","expected":{"enabled":true,"ttl":120,"system_name":null}}"#, &initial).is_err());
    assert!(
        Requirement::from_json(
            r#"{"kind":"command_success","command":"show lldp neighbors"}"#,
            &initial
        )
        .is_err()
    );
}
