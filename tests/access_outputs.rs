use dlms_learning_lab::{
    mac_learning::{Event, Mac, State, access_outputs::AccessOutputs},
    switching::{Cable, Fabric, Host, Jack, Mode, Port, Switch},
};
use std::collections::{BTreeMap, BTreeSet};
fn mac(n: u8) -> Mac {
    Mac([2, 0, 0, 0, 0, n])
}
fn names(s: &[&str]) -> BTreeSet<String> {
    s.iter().map(|v| (*v).into()).collect()
}
fn fixture() -> State {
    let mut fabric = Fabric {
        switches: BTreeMap::from([(
            "edge".into(),
            Switch {
                label: "Supplied access switch".into(),
                vlans: BTreeSet::from([10, 20]),
                ports: BTreeMap::new(),
            },
        )]),
        hosts: BTreeMap::new(),
        cables: vec![],
    };
    let mut addresses = BTreeMap::new();
    for (i, name) in ["client", "server", "peer", "other"]
        .into_iter()
        .enumerate()
    {
        fabric.switches.get_mut("edge").unwrap().ports.insert(
            name.into(),
            Port {
                up: true,
                mode: Mode::Access {
                    vlan: if name == "other" { 20 } else { 10 },
                },
            },
        );
        fabric.hosts.insert(
            name.into(),
            Host {
                label: name.into(),
                up: true,
            },
        );
        addresses.insert(name.into(), mac(i as u8 + 1));
        fabric.cables.push(Cable {
            id: format!("wire-{name}"),
            a: Jack::Host { host: name.into() },
            b: Jack::Switch {
                switch: "edge".into(),
                port: name.into(),
            },
            up: true,
        });
    }
    State::new(fabric, addresses, 300).unwrap()
}
fn outputs(s: &State, ports: &[&str]) -> AccessOutputs {
    AccessOutputs::new(s.fabric(), "edge", names(ports)).unwrap()
}
fn sent(o: &dlms_learning_lab::mac_learning::Observation) -> Vec<&str> {
    o.events
        .iter()
        .filter_map(|e| {
            if let Event::Forward { egress, .. } = e {
                Some(egress.as_str())
            } else {
                None
            }
        })
        .collect()
}
fn with_fabric(s: &State, change: impl FnOnce(&mut Fabric)) -> State {
    let mut fabric = s.fabric().clone();
    change(&mut fabric);
    State::new(
        fabric,
        ["client", "server", "peer", "other"]
            .into_iter()
            .enumerate()
            .map(|(i, n)| (n.into(), mac(i as u8 + 1)))
            .collect(),
        300,
    )
    .unwrap()
}
#[test]
fn broadcast_is_restricted_before_forward_and_nic_arrival_without_rewiring() {
    let mut s = fixture();
    let before = s.fabric().clone();
    let p = outputs(&s, &["server"]);
    let o = s
        .send_frame_on_access_ports("client", mac(1), Mac::BROADCAST, &p)
        .unwrap();
    assert_eq!(sent(&o), vec!["server"]);
    assert_eq!(o.arrivals, names(&["server"]));
    assert_eq!(o.accepted_by, names(&["server"]));
    assert_eq!(s.fabric(), &before);
    assert_eq!(s.entries().len(), 1);
    assert_eq!(s.entries()[0].port, "client");
    assert!(o.events.iter().any(|e|matches!(e,Event::Drop{location, explanation}if location=="edge/peer"&&explanation.contains("output policy"))));
}
#[test]
fn unknown_unicast_keeps_actual_arrival_distinct_from_nic_acceptance() {
    let mut s = fixture();
    let p = outputs(&s, &["server"]);
    let o = s
        .send_frame_on_access_ports("client", mac(1), mac(99), &p)
        .unwrap();
    assert_eq!(sent(&o), vec!["server"]);
    assert_eq!(o.arrivals, names(&["server"]));
    assert!(o.accepted_by.is_empty());
}
#[test]
fn excluding_the_known_destination_does_not_flood_a_permitted_neighbor() {
    let mut s = fixture();
    s.send("peer", Mac::BROADCAST).unwrap();
    let p = outputs(&s, &["server"]);
    let o = s
        .send_frame_on_access_ports("client", mac(1), mac(3), &p)
        .unwrap();
    assert!(sent(&o).is_empty());
    assert!(o.arrivals.is_empty());
    assert!(o.accepted_by.is_empty());
    assert!(
        s.entries()
            .iter()
            .any(|e| e.address == mac(3) && e.port == "peer")
    );
}
#[test]
fn selecting_all_ports_preserves_old_observations_and_serialized_learning_exactly() {
    let mut old = fixture();
    let mut new = old.clone();
    let p = outputs(&new, &["client", "server", "peer", "other"]);
    for (source, to) in [
        ("client", mac(3)),
        ("server", Mac::BROADCAST),
        ("peer", mac(1)),
        ("client", mac(2)),
    ] {
        assert_eq!(
            new.send_frame_on_access_ports(source, new.address(source).unwrap(), to, &p)
                .unwrap(),
            old.send(source, to).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&new).unwrap(),
            serde_json::to_vec(&old).unwrap()
        );
    }
}
#[test]
fn output_selection_does_not_override_vlan_or_egress_physical_state() {
    let original = fixture();
    let mut other = original.clone();
    let p = outputs(&other, &["other"]);
    assert!(
        other
            .send_frame_on_access_ports("client", mac(1), Mac::BROADCAST, &p)
            .unwrap()
            .arrivals
            .is_empty()
    );
    for cable in [true, false] {
        let mut s = with_fabric(&original, |f| {
            if cable {
                f.cables
                    .iter_mut()
                    .find(|c| c.id == "wire-server")
                    .unwrap()
                    .up = false
            } else {
                f.switches
                    .get_mut("edge")
                    .unwrap()
                    .ports
                    .get_mut("server")
                    .unwrap()
                    .up = false
            }
        });
        let p = outputs(&s, &["server"]);
        let o = s
            .send_frame_on_access_ports("client", mac(1), Mac::BROADCAST, &p)
            .unwrap();
        assert!(sent(&o).is_empty());
        assert!(o.arrivals.is_empty());
    }
}
#[test]
fn stopped_source_and_wrong_ingress_do_not_gain_delivery_from_output_selection() {
    let original = fixture();
    let mut s = with_fabric(&original, |f| f.hosts.get_mut("client").unwrap().up = false);
    let p = outputs(&s, &["server"]);
    let before = serde_json::to_vec(&s).unwrap();
    assert!(
        s.send_frame_on_access_ports("client", mac(1), Mac::BROADCAST, &p)
            .unwrap()
            .arrivals
            .is_empty()
    );
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
    let mut wire = with_fabric(&original, |f| f.cables.retain(|c| c.id != "wire-client"));
    let p = outputs(&wire, &["server"]);
    let before = serde_json::to_vec(&wire).unwrap();
    assert!(
        wire.send_frame_on_access_ports("client", mac(1), Mac::BROADCAST, &p)
            .unwrap_err()
            .contains("directly attached")
    );
    assert_eq!(serde_json::to_vec(&wire).unwrap(), before);
}
#[test]
fn empty_selection_delivers_nothing_but_learns_the_real_supplied_source() {
    let mut s = fixture();
    let p = outputs(&s, &[]);
    let o = s
        .send_frame_on_access_ports("client", mac(9), Mac::BROADCAST, &p)
        .unwrap();
    assert!(sent(&o).is_empty());
    assert!(o.arrivals.is_empty());
    assert_eq!(s.entries()[0].address, mac(9));
    assert_eq!(s.address("client"), Some(mac(1)));
}
#[test]
fn policy_is_per_frame_and_not_hidden_in_saved_state_or_the_legacy_path() {
    let mut s = fixture();
    let p = outputs(&s, &["server"]);
    s.send_frame_on_access_ports("client", mac(1), Mac::BROADCAST, &p)
        .unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    let mut restored = State::decode(&raw).unwrap();
    assert_eq!(restored, s);
    assert_eq!(
        restored.send("client", Mac::BROADCAST).unwrap().accepted_by,
        names(&["server", "peer"])
    );
}
#[test]
fn invalid_input_and_policy_reuse_on_a_changed_fabric_are_atomic() {
    let original = fixture();
    let p = outputs(&original, &["server"]);
    for (source, sm, to) in [
        ("missing", mac(1), mac(2)),
        ("client", Mac([0; 6]), mac(2)),
        ("client", mac(1), Mac([1, 0, 0, 0, 0, 1])),
    ] {
        let mut s = original.clone();
        let before = serde_json::to_vec(&s).unwrap();
        assert!(s.send_frame_on_access_ports(source, sm, to, &p).is_err());
        assert_eq!(serde_json::to_vec(&s).unwrap(), before);
    }
    let mut renamed = with_fabric(&original, |f| {
        let sw = f.switches.remove("edge").unwrap();
        f.switches.insert("replacement".into(), sw);
        for cable in &mut f.cables {
            if let Jack::Switch { switch, .. } = &mut cable.b {
                *switch = "replacement".into();
            }
        }
    });
    let before = serde_json::to_vec(&renamed).unwrap();
    assert!(
        renamed
            .send_frame_on_access_ports("client", mac(1), mac(2), &p)
            .is_err()
    );
    assert_eq!(serde_json::to_vec(&renamed).unwrap(), before);
}
#[test]
fn policy_rejects_unknown_ports_multiswitch_and_trunks_without_approximating_them() {
    let s = fixture();
    assert!(
        AccessOutputs::new(s.fabric(), "edge", names(&["missing"]))
            .unwrap_err()
            .contains("unknown")
    );
    let mut f = s.fabric().clone();
    f.switches
        .insert("second".into(), f.switches["edge"].clone());
    assert!(
        AccessOutputs::new(&f, "edge", names(&["server"]))
            .unwrap_err()
            .contains("one supplied")
    );
    let mut f = s.fabric().clone();
    f.switches
        .get_mut("edge")
        .unwrap()
        .ports
        .get_mut("server")
        .unwrap()
        .mode = Mode::Trunk {
        native: 10,
        allowed: BTreeSet::from([10, 20]),
    };
    assert!(
        AccessOutputs::new(&f, "edge", names(&["server"]))
            .unwrap_err()
            .contains("Trunks")
    );
}

#[test]
fn oversized_selection_and_unknown_switch_are_rejected_without_retention() {
    let s = fixture();
    let ports = (0..17).map(|n| format!("port-{n}")).collect();
    assert!(
        AccessOutputs::new(s.fabric(), "edge", ports)
            .unwrap_err()
            .contains("at most16")
    );
    assert!(
        AccessOutputs::new(s.fabric(), "missing", names(&[]))
            .unwrap_err()
            .contains("supplied access switch")
    );
    assert_eq!(s, fixture());
}
