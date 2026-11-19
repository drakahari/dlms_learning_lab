use dlms_learning_lab::{
    mac_learning::{Event, Mac, Reason, State},
    static_cam::{Binding, Origin, Table},
};
#[path = "support/static_cam_fixture.rs"]
mod common;
use common::{fixture, mac, names};
fn binding(s: &str, v: u16, m: u8, p: &str) -> Binding {
    Binding {
        switch: s.into(),
        vlan: v,
        address: mac(m),
        port: p.into(),
    }
}
fn path_to_c() -> Table {
    Table::new(vec![
        binding("s1", 10, 3, "uplink"),
        binding("s2", 10, 3, "c"),
    ])
    .unwrap()
}
fn bytes(s: &State) -> Vec<u8> {
    serde_json::to_vec(s).unwrap()
}
#[test]
fn empty_overlay_is_exact_for_old_dynamic_state_traces_and_serialization() {
    let mut old = fixture();
    let mut now = old.clone();
    let empty = Table::default();
    for (source, destination) in [
        ("a", mac(3)),
        ("c", mac(1)),
        ("b", Mac::BROADCAST),
        ("a", mac(2)),
    ] {
        let expected = old.send(source, destination).unwrap();
        let actual = now.send_with_static(source, destination, &empty).unwrap();
        assert_eq!(expected, actual.forwarding);
        assert!(actual.configured_sources.is_empty());
        assert_eq!(bytes(&old), bytes(&now));
    }
    old.advance(301).unwrap();
    now.advance(301).unwrap();
    assert_eq!(bytes(&old), bytes(&now));
    assert_eq!(
        old.send("a", mac(3)).unwrap(),
        now.send_with_static("a", mac(3), &empty)
            .unwrap()
            .forwarding
    );
}
#[test]
fn configured_destination_routes_the_first_frame_without_flooding_or_fake_learning() {
    let mut s = fixture();
    let table = path_to_c();
    table.validate_for(s.fabric()).unwrap();
    let out = s.send_with_static("a", mac(3), &table).unwrap();
    assert_eq!(out.forwarding.arrivals, names(&["c"]));
    assert_eq!(out.forwarding.accepted_by, names(&["c"]));
    assert!(
        out.forwarding
            .events
            .iter()
            .filter_map(|e| if let Event::Forward { reason, .. } = e {
                Some(reason)
            } else {
                None
            })
            .all(|r| *r == Reason::KnownUnicast)
    );
    assert_eq!(s.entries().len(), 2);
    assert!(s.entries().iter().all(|e| e.address == mac(1)));
    assert!(out.configured_sources.is_empty());
    let effective = table.effective_rows(&s).unwrap();
    assert_eq!(effective.len(), 4);
    assert_eq!(
        effective
            .iter()
            .filter(|r| r.origin == Origin::Configured)
            .count(),
        2
    );
}
#[test]
fn a_wrong_static_location_overrides_dynamic_learning_until_configuration_is_repaired() {
    let mut s = fixture();
    s.send("c", mac(1)).unwrap();
    assert!(
        s.entries()
            .iter()
            .any(|e| e.address == mac(3) && e.switch == "s1" && e.port == "uplink")
    );
    let wrong = Table::new(vec![binding("s1", 10, 3, "b"), binding("s2", 10, 3, "c")]).unwrap();
    let out = s.send_with_static("a", mac(3), &wrong).unwrap();
    assert_eq!(out.forwarding.arrivals, names(&["b"]));
    assert!(out.forwarding.accepted_by.is_empty());
    assert!(s.entries().iter().all(|e| e.address != mac(3)));
    let source = s.send_with_static("c", mac(1), &wrong).unwrap();
    assert_eq!(source.configured_sources.len(), 2);
    assert!(
        source
            .configured_sources
            .iter()
            .any(|r| r.binding.switch == "s1" && r.binding.port == "b" && r.ingress == "uplink")
    );
    assert_eq!(
        source.forwarding.accepted_by,
        names(&["a"]),
        "static destination configuration is not an ingress allowlist"
    );
    assert!(s.entries().iter().all(|e| e.address != mac(3)));
    assert!(
        s.send_with_static("a", mac(3), &wrong)
            .unwrap()
            .forwarding
            .accepted_by
            .is_empty()
    );
    assert_eq!(
        s.send_with_static("a", mac(3), &path_to_c())
            .unwrap()
            .forwarding
            .accepted_by,
        names(&["c"])
    );
}
#[test]
fn aging_and_clearing_dynamic_entries_do_not_rewrite_configured_bindings() {
    let mut s = fixture();
    let table = path_to_c();
    let original = serde_json::to_string(&table).unwrap();
    s.send_with_static("a", mac(3), &table).unwrap();
    s.advance(1000).unwrap();
    assert!(s.entries().is_empty());
    assert_eq!(s.clear_dynamic("s1", Some(10)).unwrap(), 0);
    let out = s.send_with_static("a", mac(3), &table).unwrap();
    assert_eq!(out.forwarding.arrivals, names(&["c"]));
    assert_eq!(serde_json::to_string(&table).unwrap(), original);
    assert!(!original.contains("learned_at"));
}
#[test]
fn configured_destination_on_ingress_filters_without_unwanted_egress() {
    let mut s = fixture();
    let t = Table::new(vec![binding("s1", 10, 3, "a")]).unwrap();
    let out = s.send_with_static("a", mac(3), &t).unwrap();
    assert!(out.forwarding.arrivals.is_empty());
    assert!(
        out.forwarding
            .events
            .iter()
            .any(|e| matches!(e,Event::Filter{switch,..}if switch=="s1"))
    );
    assert!(
        !out.forwarding
            .events
            .iter()
            .any(|e| matches!(e, Event::Forward { .. }))
    );
}
#[test]
fn known_down_ports_and_cables_drop_rather_than_falling_back_to_flooding() {
    for cable in [false, true] {
        let mut s = fixture();
        let t = path_to_c();
        let mut fabric = s.fabric().clone();
        if cable {
            fabric
                .cables
                .iter_mut()
                .find(|c| c.id == "wire-c")
                .unwrap()
                .up = false;
        } else {
            fabric
                .switches
                .get_mut("s2")
                .unwrap()
                .ports
                .get_mut("c")
                .unwrap()
                .up = false;
        }
        if cable {
            // Cable availability is an authored physical fact, not a learner
            // reconfiguration. Preserve the old same-shape guard.
            let addresses = s
                .fabric()
                .hosts
                .keys()
                .map(|name| (name.clone(), s.address(name).unwrap()))
                .collect();
            s = State::new(fabric, addresses, 300).unwrap();
        } else {
            // Current administrative/error-disabled projection does use the
            // supported reconfiguration path.
            s.configure(fabric, 300).unwrap();
        }
        t.validate_for(s.fabric()).unwrap();
        let out = s.send_with_static("a", mac(3), &t).unwrap();
        assert!(out.forwarding.arrivals.is_empty());
        assert!(out.forwarding.accepted_by.is_empty());
        assert!(out.forwarding.events.iter().any(|e|matches!(e,Event::Drop{explanation,..}if explanation.contains("outgoing port or cable"))));
        assert!(out.forwarding.events.iter().all(|e| !matches!(
            e,
            Event::Forward {
                reason: Reason::UnknownUnicast,
                ..
            }
        )));
    }
}
#[test]
fn static_keys_are_vlan_scoped_and_broadcast_behavior_is_retained() {
    let mut s = fixture();
    let t = Table::new(vec![
        binding("s1", 20, 4, "uplink"),
        binding("s2", 20, 4, "d"),
    ])
    .unwrap();
    let out = s.send_with_static("a", mac(4), &t).unwrap();
    assert_eq!(out.forwarding.arrivals, names(&["b", "c"]));
    assert!(out.forwarding.accepted_by.is_empty());
    assert!(out.forwarding.events.iter().any(|e| matches!(
        e,
        Event::Forward {
            reason: Reason::UnknownUnicast,
            ..
        }
    )));
    let mut old = fixture();
    let mut new = fixture();
    assert_eq!(
        old.send("a", Mac::BROADCAST).unwrap(),
        new.send_with_static("a", Mac::BROADCAST, &t)
            .unwrap()
            .forwarding
    );
}
#[test]
fn invalid_bindings_and_requests_leave_even_shadowable_dynamic_state_unchanged() {
    let mut s = fixture();
    s.send("c", mac(1)).unwrap();
    let original = bytes(&s);
    for t in [
        Table::new(vec![binding("missing", 10, 3, "a")]).unwrap(),
        Table::new(vec![binding("s1", 10, 3, "missing")]).unwrap(),
        Table::new(vec![binding("s1", 20, 3, "a")]).unwrap(),
        Table::new(vec![binding("s1", 40, 3, "uplink")]).unwrap(),
    ] {
        assert!(s.send_with_static("a", mac(3), &t).is_err());
        assert_eq!(bytes(&s), original);
    }
    for (source, dest) in [
        ("missing", mac(3)),
        ("a", Mac([1, 0, 0, 0, 0, 1])),
        ("a", Mac([0; 6])),
    ] {
        assert!(s.send_with_static(source, dest, &path_to_c()).is_err());
        assert_eq!(bytes(&s), original);
    }
    assert!(Table::new(vec![binding("s1", 10, 3, "a"), binding("s1", 10, 3, "b")]).is_err());
    let mut bad = binding("s1", 10, 3, "a");
    bad.address = Mac::BROADCAST;
    assert!(Table::new(vec![bad]).is_err());
    assert!(Table::new(vec![binding("s1", 0, 3, "a")]).is_err());
    assert!(Table::new(vec![binding("s1", 4095, 3, "a")]).is_err());
    assert!(Table::new(vec![binding("s1", 10, 3, "../../etc")]).is_err());
}
#[test]
fn bounded_config_roundtrips_and_rejects_excess_and_unknown_fields() {
    let t = path_to_c();
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(Table::from_json(&json).unwrap(), t);
    assert!(Table::from_json(&json.replace("\"port\"", "\"unknown\"")).is_err());
    assert!(Table::from_json(&" ".repeat(dlms_learning_lab::static_cam::MAX_JSON + 1)).is_err());
    let rows: Vec<_> = (1..=65).map(|n| binding("s1", 10, n, "a")).collect();
    assert!(Table::new(rows.clone()).is_err());
    assert!(Table::from_json(&serde_json::json!({"rows":rows}).to_string()).is_err());
    let mut row = binding("s1", 10, 3, "a");
    row.switch = "x".repeat(65);
    assert!(Table::new(vec![row]).is_err());
}
#[test]
fn dynamic_capacity_failure_is_atomic_with_an_active_static_overlay() {
    let source = fixture();
    let mut raw = serde_json::to_value(&source).unwrap();
    raw["entries"] = serde_json::json!(
        (0..256u16)
            .map(|n| dlms_learning_lab::mac_learning::Entry {
                switch: "s1".into(),
                vlan: 10,
                address: Mac([2, 10, (n >> 8) as u8, 0, 0, n as u8]),
                port: "b".into(),
                learned_at: 0
            })
            .collect::<Vec<_>>()
    );
    let mut full = State::decode(&raw.to_string()).unwrap();
    let before = bytes(&full);
    assert_eq!(path_to_c().effective_rows(&full).unwrap().len(), 258);
    assert!(
        full.send_with_static("a", mac(3), &path_to_c())
            .unwrap_err()
            .contains("table limit")
    );
    assert_eq!(bytes(&full), before);
}
