use dlms_learning_lab::{
    mac_learning::{Event, Mac, Reason, State},
    port_security::{
        Cause, MAX_COUNTER, MAX_JSON, Notice, Origin, Outcome, ViolationMode as V, World,
    },
    static_cam::Table,
};
#[path = "support/static_cam_fixture.rs"]
mod common;
use common::{fixture, mac, names};
fn world() -> World {
    World::new(fixture(), "s1", &["a", "b"]).unwrap()
}
fn configure(w: &mut World, p: &str, mode: V, ids: Vec<Mac>) {
    w.configure(p, true, 1, mode, ids).unwrap();
}
fn admitted(out: Outcome) -> dlms_learning_lab::mac_learning::Observation {
    match out {
        Outcome::Admitted { forwarding, .. } => forwarding,
        other => panic!("Expected admitted frame: {other:?}"),
    }
}
fn bytes(w: &World) -> Vec<u8> {
    serde_json::to_vec(w).unwrap()
}
#[test]
fn disabled_security_preserves_old_actual_forwarding_and_state() {
    let mut w = world();
    let mut old = fixture();
    for (host, to) in [
        ("a", mac(3)),
        ("c", mac(1)),
        ("b", Mac::BROADCAST),
        ("a", mac(2)),
    ] {
        assert_eq!(
            admitted(w.send(host, old.address(host).unwrap(), to).unwrap()),
            old.send(host, to).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(w.forwarding()).unwrap(),
            serde_json::to_vec(&old).unwrap()
        );
    }
    assert!(w.secure_rows().is_empty());
}
#[test]
fn capacity_is_not_an_identity_allowlist_and_unexpected_source_can_win_first() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![]);
    let out = w.send("a", mac(9), mac(3)).unwrap();
    assert!(
        matches!(out,Outcome::Admitted{learned:Some(ref row),ref forwarding} if row.address==mac(9) && row.origin==Origin::DynamicSecure && forwarding.accepted_by==names(&["c"]))
    );
    assert!(matches!(
        w.send("a", mac(1), mac(3)).unwrap(),
        Outcome::Violation {
            cause: Cause::Capacity,
            counter: 1,
            ..
        }
    ));
    assert_eq!(w.port("a").unwrap().dynamic, vec![mac(9)]);
    assert_eq!(
        w.baseline().address("a"),
        Some(mac(1)),
        "Replay identity must not overwrite the authored endpoint"
    );
}
#[test]
fn configured_identity_blocks_unexpected_first_and_preserves_authorized_traffic() {
    for mode in [V::Protect, V::Restrict] {
        let mut w = world();
        configure(&mut w, "a", mode, vec![mac(1)]);
        assert!(matches!(
            w.send("a", mac(9), mac(3)).unwrap(),
            Outcome::Violation {
                cause: Cause::Capacity,
                ..
            }
        ));
        let out = admitted(w.send("a", mac(1), mac(3)).unwrap());
        assert_eq!(out.accepted_by, names(&["c"]));
        assert_eq!(w.secure_rows().len(), 1);
        assert_eq!(w.secure_rows()[0].origin, Origin::Configured);
        assert!(
            !w.forwarding()
                .entries()
                .iter()
                .any(|e| e.switch == "s1" && e.address == mac(9))
        );
    }
}
#[test]
fn protect_and_restrict_differ_in_counter_and_notification_not_forwarding() {
    for (mode, counter, notice) in [
        (V::Protect, 0, None),
        (V::Restrict, 2, Some(Notice::RestrictViolation)),
    ] {
        let mut w = world();
        configure(&mut w, "a", mode, vec![mac(1)]);
        for _ in 0..2 {
            let out = w.send("a", mac(9), mac(3)).unwrap();
            assert!(matches!(out,Outcome::Violation{mode:m,notice:n,..}if m==mode&&n==notice));
        }
        assert_eq!(w.port("a").unwrap().violations, counter);
        assert!(w.port("a").unwrap().operational());
        assert_eq!(
            admitted(w.send("c", mac(3), mac(1)).unwrap()).accepted_by,
            names(&["a"])
        );
    }
}
#[test]
fn shutdown_blocks_ingress_and_egress_until_actual_manual_recovery() {
    let mut w = world();
    configure(&mut w, "a", V::Shutdown, vec![mac(1)]);
    assert!(matches!(
        w.send("a", mac(9), mac(3)).unwrap(),
        Outcome::Violation {
            mode: V::Shutdown,
            counter: 1,
            notice: None,
            ..
        }
    ));
    assert!(w.port("a").unwrap().error_disabled);
    assert!(!w.forwarding().fabric().switches["s1"].ports["a"].up);
    for _ in 0..2 {
        assert!(matches!(
            w.send("a", mac(1), mac(3)).unwrap(),
            Outcome::Unavailable { .. }
        ));
    }
    assert_eq!(
        w.port("a").unwrap().violations,
        1,
        "An already-disabled port is not another violation"
    );
    let out = admitted(w.send("c", mac(3), mac(1)).unwrap());
    assert!(
        out.arrivals.is_empty(),
        "Known disabled destination must drop rather than flood"
    );
    assert!(out.events.iter().any(|e| matches!(e, Event::Drop { .. })));
    w.configure("a", true, 1, V::Restrict, vec![mac(1)])
        .unwrap();
    w.set_admin("a", true).unwrap();
    assert!(
        w.port("a").unwrap().error_disabled,
        "Mode edit and repeated Up are not recovery"
    );
    w.set_admin("a", false).unwrap();
    assert!(!w.port("a").unwrap().error_disabled);
    assert!(!w.port("a").unwrap().operational());
    w.set_admin("a", true).unwrap();
    assert_eq!(
        admitted(w.send("a", mac(1), mac(3)).unwrap()).accepted_by,
        names(&["c"])
    );
    assert_eq!(w.port("a").unwrap().violations, 1);
}
#[test]
fn same_vlan_secure_movement_violates_even_with_free_capacity() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![mac(1)]);
    w.configure("b", true, 2, V::Restrict, vec![]).unwrap();
    assert!(
        matches!(w.send("b",mac(1),mac(3)).unwrap(),Outcome::Violation{cause:Cause::SecuredElsewhere{ref port},counter:1,..}if port=="a")
    );
    assert!(w.port("b").unwrap().dynamic.is_empty());
    assert_eq!(
        admitted(w.send("b", mac(2), mac(3)).unwrap()).accepted_by,
        names(&["c"])
    );
    assert_eq!(
        w.secure_rows()
            .iter()
            .filter(|r| r.address == mac(1))
            .count(),
        1
    );
}
#[test]
fn secure_dynamic_binding_forwards_without_ordinary_aging_or_fake_provenance() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![]);
    admitted(w.send("a", mac(1), mac(3)).unwrap());
    w.advance(301).unwrap();
    assert!(w.forwarding().entries().is_empty());
    assert_eq!(w.secure_rows()[0].origin, Origin::DynamicSecure);
    let out = admitted(w.send("c", mac(3), mac(1)).unwrap());
    assert_eq!(out.arrivals, names(&["a"]));
    assert!(
        out.events.iter().any(
            |e| matches!(e,Event::Forward{switch,reason:Reason::KnownUnicast,..}if switch=="s1")
        )
    );
    assert!(
        !w.forwarding()
            .entries()
            .iter()
            .any(|e| e.switch == "s1" && e.address == mac(1))
    );
    w.set_admin("a", false).unwrap();
    assert!(w.port("a").unwrap().dynamic.is_empty());
    w.set_admin("a", true).unwrap();
    assert!(w.secure_rows().is_empty());
}
#[test]
fn promotion_and_explicit_replacement_preserve_other_ports_and_counter_history() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![]);
    configure(&mut w, "b", V::Protect, vec![mac(2)]);
    admitted(w.send("a", mac(1), mac(3)).unwrap());
    let protected = w.port("b").unwrap().clone();
    w.configure("a", true, 1, V::Restrict, vec![mac(1)])
        .unwrap();
    assert!(w.port("a").unwrap().dynamic.is_empty());
    assert_eq!(
        w.secure_rows()
            .iter()
            .find(|r| r.port == "a")
            .unwrap()
            .origin,
        Origin::Configured
    );
    w.send("a", mac(9), mac(3)).unwrap();
    w.configure("a", true, 1, V::Restrict, vec![mac(9)])
        .unwrap();
    assert_eq!(
        admitted(w.send("a", mac(9), mac(3)).unwrap()).accepted_by,
        names(&["c"])
    );
    assert_eq!(w.port("b"), Some(&protected));
    assert_eq!(w.port("a").unwrap().violations, 1);
    assert_eq!(w.port("a").unwrap().configured, vec![mac(9)]);
}
#[test]
fn invalid_reconfiguration_and_requests_are_atomic_not_simulated_drops() {
    let mut w = world();
    w.configure("a", true, 2, V::Restrict, vec![]).unwrap();
    w.send("a", mac(1), mac(3)).unwrap();
    w.send("a", mac(9), mac(3)).unwrap();
    let old = bytes(&w);
    for result in [
        w.configure("a", true, 1, V::Restrict, vec![]),
        w.configure("a", true, 0, V::Restrict, vec![]),
        w.configure("b", true, 2, V::Restrict, vec![mac(1)]),
        w.configure("b", true, 2, V::Restrict, vec![mac(2), mac(2)]),
        w.configure("b", true, 1, V::Restrict, vec![Mac::BROADCAST]),
        w.configure("b", false, 1, V::Restrict, vec![mac(2)]),
        w.set_admin("../../not-a-port", false),
        w.advance(86401),
    ] {
        assert!(result.is_err());
        assert_eq!(bytes(&w), old);
    }
    for (host, source, dest) in [
        ("missing", mac(1), mac(3)),
        ("a", Mac::BROADCAST, mac(3)),
        ("a", mac(1), Mac([1, 0, 0, 0, 0, 1])),
        ("a", Mac([0; 6]), mac(3)),
    ] {
        assert!(w.send(host, source, dest).is_err());
        assert_eq!(bytes(&w), old);
    }
}
#[test]
fn down_source_is_not_a_violation_and_does_not_teach_a_secure_address() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![]);
    w.set_admin("a", false).unwrap();
    let old = bytes(&w);
    assert!(matches!(
        w.send("a", mac(9), mac(3)).unwrap(),
        Outcome::Unavailable { .. }
    ));
    assert_eq!(bytes(&w), old);
    for field in ["host", "cable"] {
        let mut state = serde_json::to_value(fixture()).unwrap();
        if field == "host" {
            state["fabric"]["hosts"]["a"]["up"] = false.into();
        } else {
            state["fabric"]["cables"][0]["up"] = false.into();
        }
        let initial: State = serde_json::from_value(state).unwrap();
        let mut w = World::new(initial, "s1", &["a"]).unwrap();
        configure(&mut w, "a", V::Restrict, vec![]);
        assert!(matches!(
            w.send("a", mac(9), mac(3)).unwrap(),
            Outcome::Unavailable { .. }
        ));
        assert!(w.secure_rows().is_empty());
    }
}
#[test]
fn bounded_restore_rejects_forged_runtime_projection_versions_and_collections() {
    let mut w = world();
    configure(&mut w, "a", V::Shutdown, vec![mac(1)]);
    w.send("a", mac(9), mac(3)).unwrap();
    let raw = serde_json::to_string(&w).unwrap();
    assert_eq!(World::from_json(&raw).unwrap(), w);
    let base = serde_json::to_value(&w).unwrap();
    for (path, val) in [
        ("/schema_version", 2.into()),
        ("/ports/0/error_disabled", false.into()),
        ("/ports/0/violations", (MAX_COUNTER + 1).into()),
        ("/ports/0/maximum", 0.into()),
        ("/ports/0/name", "unknown".into()),
        ("/active/fabric/switches/s1/ports/a/up", true.into()),
        ("/active/host_addresses/a", mac(9).text().into()),
    ] {
        let mut v = base.clone();
        *v.pointer_mut(path).unwrap() = val;
        assert!(
            World::from_json(&v.to_string()).is_err(),
            "Accepted forged {path}"
        );
    }
    for key in ["configured", "dynamic"] {
        let mut v = base.clone();
        v["ports"][0][key] = serde_json::to_value(vec![mac(1); 17]).unwrap();
        assert!(World::from_json(&v.to_string()).is_err());
    }
    let mut v = base.clone();
    v["ports"] = serde_json::Value::Array(vec![v["ports"][0].clone(); 9]);
    assert!(World::from_json(&v.to_string()).is_err());
    assert!(World::from_json(&" ".repeat(MAX_JSON + 1)).is_err());
    assert!(World::from_json(&raw.replacen("{", "{\"extra\":1,", 1)).is_err());
}
#[test]
fn unsupported_topology_and_duplicate_management_are_rejected_before_projection() {
    for (switch, ports) in [
        ("s1", vec!["uplink"]),
        ("s1", vec!["a", "a"]),
        ("s1", vec!["unknown"]),
        ("unknown", vec!["a"]),
        ("s1", vec![]),
    ] {
        assert!(World::new(fixture(), switch, &ports).is_err());
    }
    let mut not_initial = fixture();
    not_initial.send("a", mac(3)).unwrap();
    assert!(World::new(not_initial, "s1", &["a"]).is_err());
}
#[test]
fn metadata_source_override_keeps_old_identity_and_honest_destination_acceptance() {
    let mut s = fixture();
    let old_address = s.address("a");
    let out = s
        .send_frame_with_static("a", mac(9), mac(3), &Table::default())
        .unwrap();
    assert_eq!(s.address("a"), old_address);
    assert_eq!(out.forwarding.accepted_by, names(&["c"]));
    assert!(
        s.entries()
            .iter()
            .any(|e| e.switch == "s1" && e.port == "a" && e.address == mac(9))
    );
    let reply = s.send("c", mac(9)).unwrap();
    assert_eq!(reply.arrivals, names(&["a"]));
    assert!(
        reply.accepted_by.is_empty(),
        "Original NIC identity is unchanged by a one-frame source override"
    );
    let before = serde_json::to_vec(&s).unwrap();
    assert!(
        s.send_frame_with_static("a", Mac::BROADCAST, mac(3), &Table::default())
            .is_err()
    );
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
}

#[test]
fn secure_keys_are_vlan_scoped_and_counters_saturate_honestly() {
    let mut state = serde_json::to_value(fixture()).unwrap();
    state["fabric"]["switches"]["s1"]["ports"]["b"]["mode"]["vlan"] = 20.into();
    let mut w = World::new(serde_json::from_value(state).unwrap(), "s1", &["a", "b"]).unwrap();
    configure(&mut w, "a", V::Restrict, vec![mac(1)]);
    configure(&mut w, "b", V::Restrict, vec![mac(1)]);
    assert_eq!(
        admitted(w.send("b", mac(1), mac(4)).unwrap()).accepted_by,
        names(&["d"])
    );
    assert_eq!(w.secure_rows().len(), 2);
    let mut saved = serde_json::to_value(w).unwrap();
    saved["ports"][0]["violations"] = MAX_COUNTER.into();
    let mut restored = World::from_json(&saved.to_string()).unwrap();
    assert!(matches!(
        restored.send("a", mac(9), mac(3)).unwrap(),
        Outcome::Violation {
            counter: MAX_COUNTER,
            ..
        }
    ));
    assert_eq!(restored.port("a").unwrap().violations, MAX_COUNTER);
}
#[test]
fn configured_mac_filter_does_not_pretend_to_authenticate_the_sender() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![mac(1)]);
    // The source field matches; the simulator cannot infer a human/device identity.
    assert_eq!(
        admitted(w.send("a", mac(1), mac(3)).unwrap()).accepted_by,
        names(&["c"])
    );
}

#[test]
fn forwarding_resource_failure_does_not_commit_secure_learning() {
    let mut w = world();
    configure(&mut w, "a", V::Restrict, vec![]);
    let mut saved = serde_json::to_value(w).unwrap();
    let entries:Vec<_>=(0u16..256).map(|n|serde_json::json!({
        "switch":"s1", "vlan":10, "address":Mac([2,1,0,0,(n>>8) as u8,n as u8]).text(), "port":"b", "learned_at":0
    })).collect();
    saved["active"]["entries"] = serde_json::Value::Array(entries);
    let mut w = World::from_json(&saved.to_string()).unwrap();
    let old = bytes(&w);
    assert!(
        w.send("a", mac(1), mac(3)).is_err(),
        "Other switch cannot learn when the ordinary global CAM budget is full"
    );
    assert_eq!(bytes(&w), old);
    assert!(w.port("a").unwrap().dynamic.is_empty());
}
#[test]
fn global_secure_resource_limit_is_atomic_and_not_an_invented_security_violation() {
    use dlms_learning_lab::switching::{Cable, Fabric, Host, Jack, Mode, Port, Switch};
    use std::collections::BTreeMap;
    let mut f = Fabric {
        switches: BTreeMap::new(),
        hosts: BTreeMap::new(),
        cables: vec![],
    };
    let mut sw = Switch {
        label: "Access switch".into(),
        vlans: [10].into(),
        ports: BTreeMap::new(),
    };
    let mut addresses = BTreeMap::new();
    let mut managed = vec![];
    for n in 0..8u8 {
        let host = format!("h{n}");
        let port = format!("p{n}");
        sw.ports.insert(
            port.clone(),
            Port {
                up: true,
                mode: Mode::Access { vlan: 10 },
            },
        );
        f.hosts.insert(
            host.clone(),
            Host {
                label: format!("Desk {n}"),
                up: true,
            },
        );
        f.cables.push(Cable {
            id: format!("wire-{n}"),
            a: Jack::Host { host: host.clone() },
            b: Jack::Switch {
                switch: "s".into(),
                port: port.clone(),
            },
            up: true,
        });
        addresses.insert(host, mac(n + 1));
        managed.push(port);
    }
    f.switches.insert("s".into(), sw);
    let mut w = World::new(
        State::new(f, addresses, 300).unwrap(),
        "s",
        &managed.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
    )
    .unwrap();
    for n in 0..8u8 {
        w.configure(
            &format!("p{n}"),
            true,
            16,
            V::Restrict,
            (0..8).map(|i| mac(20 + n * 8 + i)).collect(),
        )
        .unwrap();
    }
    assert_eq!(w.secure_rows().len(), 64);
    let old = bytes(&w);
    assert!(w.send("h0", mac(200), mac(2)).is_err());
    assert_eq!(bytes(&w), old);
    assert_eq!(w.port("p0").unwrap().violations, 0);
}
