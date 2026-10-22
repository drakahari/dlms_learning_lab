use dlms_learning_lab::switching::{Fabric, Host, Mode, Port, Switch};
use std::collections::{BTreeMap, BTreeSet};
#[path = "support/switching.rs"]
mod fixtures;
use fixtures::*;
#[test]
fn access_trunk_tag_paths_keep_broadcast_domains_separate_and_both_directions_explicit() {
    let f = fabric();
    f.validate().unwrap();
    assert!(f.visibility("staff-left", "staff-right").unwrap().mutual());
    assert!(f.visibility("guest-left", "guest-right").unwrap().mutual());
    assert!(
        f.visibility("staff-left", "guest-right")
            .unwrap()
            .isolated()
    );
    let r = f.broadcast("staff-left").unwrap();
    assert_eq!(
        r.recipients.keys().map(String::as_str).collect::<Vec<_>>(),
        ["staff-right"]
    );
    let path = &r.recipients["staff-right"];
    assert_eq!(path.len(), 2);
    assert_eq!(
        (path[0].vlan, path[0].tag, path[1].vlan, path[1].tag),
        (10, Some(10), 10, None)
    );
    assert_eq!(path[0].egress, "uplink");
    assert_eq!(path[1].ingress, "uplink");
    assert!(f.visibility("missing", "staff-right").is_err());
    assert!(f.visibility("staff-left", "staff-left").is_err());
}
#[test]
fn changing_access_database_or_allowed_membership_recomputes_current_results() {
    let mut f = fabric();
    let good = f.broadcast("staff-left").unwrap();
    f.switches.get_mut("right").unwrap().vlans.remove(&10);
    let r = f.broadcast("staff-left").unwrap();
    assert!(!r.recipients.contains_key("staff-right"));
    assert!(r.drops.iter().any(|d| d.contains("not active")));
    f.switches.get_mut("right").unwrap().vlans.insert(10);
    assert_eq!(f.broadcast("staff-left").unwrap(), good);
    f.switches
        .get_mut("left")
        .unwrap()
        .ports
        .get_mut("staff")
        .unwrap()
        .mode = Mode::Access { vlan: 20 };
    assert!(!f.visibility("staff-left", "staff-right").unwrap().mutual());
    assert!(f.visibility("staff-left", "guest-right").unwrap().mutual());
    f.switches
        .get_mut("left")
        .unwrap()
        .ports
        .get_mut("staff")
        .unwrap()
        .mode = Mode::Access { vlan: 10 };
    if let Mode::Trunk { allowed, .. } = &mut f
        .switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode
    {
        allowed.remove(&10);
    }
    assert!(!f.visibility("staff-left", "staff-right").unwrap().mutual());
    assert!(
        f.broadcast("staff-left")
            .unwrap()
            .drops
            .iter()
            .any(|d| d.contains("not allowed into"))
    );
    assert!(f.visibility("guest-left", "guest-right").unwrap().mutual());
    if let Mode::Trunk { allowed, .. } = &mut f
        .switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode
    {
        allowed.insert(10);
    }
    assert!(f.visibility("staff-left", "staff-right").unwrap().mutual());
}
#[test]
fn native_tagging_is_per_hop_and_one_way_delivery_is_not_isolation() {
    let mut f = fabric();
    f.switches
        .get_mut("left")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode = trunk(10, &[10, 20, 99]).mode;
    f.switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode = trunk(20, &[10, 20, 99]).mode;
    // Restricted data-plane behavior only; real PVST inconsistency blocking is excluded.
    assert!(f.visibility("staff-left", "guest-right").unwrap().mutual());
    let path = f.broadcast("staff-left").unwrap().recipients["guest-right"].clone();
    assert_eq!(
        (path[0].vlan, path[0].tag, path[1].vlan, path[1].tag),
        (10, None, 20, None)
    );
    let one_way = f.visibility("staff-left", "staff-right").unwrap();
    assert!(!one_way.forward && one_way.reverse);
    assert!(!one_way.mutual());
    assert!(!one_way.isolated());
    for s in f.switches.values_mut() {
        s.ports.get_mut("uplink").unwrap().mode = trunk(99, &[10, 20, 99]).mode;
    }
    assert!(f.visibility("staff-left", "staff-right").unwrap().mutual());
    assert!(
        f.visibility("staff-left", "guest-right")
            .unwrap()
            .isolated()
    );
}
#[test]
fn host_port_and_cable_down_states_are_known_drops_not_cached_success() {
    for change in 0..4 {
        let mut f = fabric();
        match change {
            0 => f.hosts.get_mut("staff-left").unwrap().up = false,
            1 => f.hosts.get_mut("staff-right").unwrap().up = false,
            2 => {
                f.switches
                    .get_mut("right")
                    .unwrap()
                    .ports
                    .get_mut("staff")
                    .unwrap()
                    .up = false
            }
            _ => f.cables[0].up = false,
        }
        assert!(!f.visibility("staff-left", "staff-right").unwrap().mutual());
        let r = f.broadcast("staff-left").unwrap();
        assert!(!r.recipients.contains_key("staff-right"));
        assert!(!r.drops.is_empty());
    }
    let mut f = fabric();
    f.cables.retain(|c| c.id != "sl");
    assert!(f.broadcast("staff-left").unwrap().drops[0].contains("no cable"));
}
#[test]
fn unsupported_tagged_access_cannot_be_mistaken_for_a_valid_isolation_result() {
    let mut f = fabric();
    f.switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode = Mode::Access { vlan: 10 };
    assert!(
        f.broadcast("staff-left")
            .unwrap_err()
            .contains("Unsupported tagged ingress")
    );
    assert!(f.visibility("staff-left", "staff-right").is_err());
    let mut f = fabric();
    f.switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("staff")
        .unwrap()
        .mode = trunk(20, &[10, 20, 99]).mode;
    let r = f.broadcast("staff-left").unwrap();
    assert!(!r.recipients.contains_key("staff-right"));
    assert!(r.drops.iter().any(|d| d.contains("only untagged")));
    f.switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("staff")
        .unwrap()
        .mode = trunk(10, &[10, 20, 99]).mode;
    assert!(f.visibility("staff-left", "staff-right").unwrap().mutual());
}
#[test]
fn vlan_ids_cross_all_bit_boundaries_without_numeric_sized_allocation() {
    for n in 1..=4094 {
        let mut f = fabric();
        for s in f.switches.values_mut() {
            s.vlans = BTreeSet::from([n]);
            s.ports.get_mut("staff").unwrap().mode = Mode::Access { vlan: n };
            s.ports.get_mut("uplink").unwrap().mode = trunk(4094, &[n]).mode;
        }
        assert!(
            f.visibility("staff-left", "staff-right").unwrap().mutual(),
            "VLAN{n}"
        );
    }
}
#[test]
fn malformed_graphs_cycles_duplicate_jacks_and_invalid_configuration_fail_before_propagation() {
    let base = fabric();
    for change in 0..9 {
        let mut f = base.clone();
        match change {
            0 => f.cables[1].a = host("missing"),
            1 => f.cables[1].b = jack("left", "missing"),
            2 => f
                .cables
                .push(cable("again", host("staff-left"), jack("right", "staff"))),
            3 => {
                f.cables[0].b = jack("left", "staff");
                f.cables.retain(|c| c.id != "sl");
            }
            4 => {
                for s in f.switches.values_mut() {
                    s.ports.insert("extra".into(), trunk(99, &[10, 20, 99]));
                }
                f.cables.push(cable(
                    "redundant",
                    jack("left", "extra"),
                    jack("right", "extra"),
                ));
            }
            5 => f.hosts.get_mut("staff-left").unwrap().label = "\n".into(),
            6 => {
                f.switches
                    .get_mut("left")
                    .unwrap()
                    .ports
                    .get_mut("staff")
                    .unwrap()
                    .mode = Mode::Access { vlan: 0 }
            }
            7 => f
                .switches
                .get_mut("left")
                .unwrap()
                .vlans
                .insert(4095)
                .then_some(())
                .unwrap(),
            _ => {
                let h = f.hosts["staff-left"].clone();
                f.hosts.insert("left".into(), h);
            }
        }
        assert!(f.validate().is_err(), "change{change}");
        assert!(f.broadcast("staff-left").is_err());
    }
}
#[test]
fn declarative_decoder_rejects_duplicates_limits_unknown_fields_and_roundtrips_valid_state() {
    let f = fabric();
    let raw = serde_json::to_string(&f).unwrap();
    assert_eq!(serde_json::from_str::<Fabric>(&raw).unwrap(), f);
    let sw = serde_json::to_string(&f.switches["left"]).unwrap();
    let duplicate =
        format!("{{\"switches\":{{\"left\":{sw},\"left\":{sw}}},\"hosts\":{{}},\"cables\":[]}}");
    assert!(serde_json::from_str::<Fabric>(&duplicate).is_err());
    for replacement in [
        "[10,10,20]",
        "[0]",
        "[4095]",
        &format!(
            "[{}]",
            (1..=65)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ),
    ] {
        assert!(
            serde_json::from_str::<Fabric>(&raw.replacen("[10,20,99]", replacement, 1)).is_err()
        );
    }
    let unknown = raw.replacen("\"label\":", "\"extra\":true,\"label\":", 1);
    assert!(serde_json::from_str::<Fabric>(&unknown).is_err());
    let mut value = serde_json::to_value(&f).unwrap();
    value["cables"] =
        serde_json::Value::Array(vec![serde_json::to_value(&f.cables[0]).unwrap(); 65]);
    assert!(serde_json::from_value::<Fabric>(value).is_err());
}
#[test]
fn maximum_supported_forest_and_vlan_sets_have_complete_bounded_propagation() {
    let vlans: BTreeSet<_> = (4031..=4094).collect();
    let mut f = Fabric {
        switches: BTreeMap::new(),
        hosts: BTreeMap::new(),
        cables: vec![],
    };
    for s in 0..8 {
        let name = format!("s{s}");
        let mut ports = BTreeMap::new();
        for n in 0..16 {
            ports.insert(
                format!("p{n}"),
                if n < 4 {
                    access(4094)
                } else {
                    Port {
                        up: true,
                        mode: Mode::Trunk {
                            native: 4031,
                            allowed: vlans.clone(),
                        },
                    }
                },
            );
        }
        f.switches.insert(
            name.clone(),
            Switch {
                label: name.clone(),
                vlans: vlans.clone(),
                ports,
            },
        );
        for n in 0..4 {
            let h = format!("h{s}-{n}");
            f.hosts.insert(
                h.clone(),
                Host {
                    label: h.clone(),
                    up: true,
                },
            );
            f.cables.push(cable(
                &format!("c{s}-{n}"),
                host(&h),
                jack(&name, &format!("p{n}")),
            ));
        }
        if s > 0 {
            f.cables.push(cable(
                &format!("link{s}"),
                jack(&format!("s{}", s - 1), "p5"),
                jack(&name, "p4"),
            ));
        }
    }
    f.validate().unwrap();
    let r = f.broadcast("h0-0").unwrap();
    assert_eq!(r.recipients.len(), 31);
    assert!(r.recipients.values().all(|p| p.len() <= 8));
    assert!(r.drops.len() <= 64);
    assert_eq!(f.broadcast("h0-0").unwrap(), r);
    let mut too_big = f.clone();
    too_big.hosts.insert(
        "extra".into(),
        Host {
            label: "Extra".into(),
            up: true,
        },
    );
    assert!(too_big.validate().is_err());
}

#[test]
fn live_physical_route_is_independent_of_vlan_filtering_but_rejects_disabling_or_unplugging() {
    let base = fabric();
    assert!(base.live_path("staff-left", "guest-right").unwrap());
    assert!(
        base.visibility("staff-left", "guest-right")
            .unwrap()
            .isolated()
    );
    for change in 0..5 {
        let mut f = base.clone();
        match change {
            0 => f.hosts.get_mut("guest-right").unwrap().up = false,
            1 => f.cables[0].up = false,
            2 => {
                f.switches
                    .get_mut("right")
                    .unwrap()
                    .ports
                    .get_mut("uplink")
                    .unwrap()
                    .up = false
            }
            3 => f.cables.retain(|c| c.id != "gr"),
            _ => {
                f.switches
                    .get_mut("left")
                    .unwrap()
                    .ports
                    .get_mut("staff")
                    .unwrap()
                    .up = false
            }
        }
        assert!(!f.live_path("staff-left", "guest-right").unwrap());
    }
    assert!(base.live_path("staff-left", "staff-left").is_err());
    assert!(base.live_path("missing", "staff-left").is_err());
}

#[test]
fn single_link_native_allowed_and_database_matrix_matches_independent_transition_equation() {
    let subsets = [
        BTreeSet::new(),
        BTreeSet::from([10]),
        BTreeSet::from([20]),
        BTreeSet::from([10, 20]),
    ];
    for ln in [10, 20, 99] {
        for rn in [10, 20, 99] {
            for la in &subsets {
                for ra in &subsets {
                    for ld in &subsets {
                        for rd in &subsets {
                            let mut f = fabric();
                            for (id, native, allowed, db) in
                                [("left", ln, la, ld), ("right", rn, ra, rd)]
                            {
                                let s = f.switches.get_mut(id).unwrap();
                                s.vlans = db.clone();
                                s.ports.get_mut("uplink").unwrap().mode = Mode::Trunk {
                                    native,
                                    allowed: allowed.clone(),
                                };
                            }
                            let reaches =
                                |source,
                                 dest,
                                 native,
                                 remote_native,
                                 allowed: &BTreeSet<u16>,
                                 remote_allowed: &BTreeSet<u16>,
                                 db: &BTreeSet<u16>,
                                 remote_db: &BTreeSet<u16>| {
                                    let received = if source == native {
                                        remote_native
                                    } else {
                                        source
                                    };
                                    db.contains(&source)
                                        && allowed.contains(&source)
                                        && received == dest
                                        && remote_db.contains(&received)
                                        && remote_allowed.contains(&received)
                                };
                            for (a, src) in [("staff-left", 10), ("guest-left", 20)] {
                                for (b, dst) in [("staff-right", 10), ("guest-right", 20)] {
                                    let actual = f.visibility(a, b).unwrap();
                                    let forward = reaches(src, dst, ln, rn, la, ra, ld, rd);
                                    let reverse = reaches(dst, src, rn, ln, ra, la, rd, ld);
                                    assert_eq!(
                                        (actual.forward, actual.reverse),
                                        (forward, reverse)
                                    );
                                    assert_eq!(actual.isolated(), !forward && !reverse);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
