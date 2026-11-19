//! Real bounded broadcast paths; no bridge lesson, kernel operation or ping claim.
#[path = "support/bridge_fabric.rs"]
mod fixture;
use b::Endpoint::{Local, Peer};
use dlms_learning_lab::{bridge_fabric as b, switching as s};
#[test]
fn bridge_joins_both_ports_and_local_endpoint_without_selecting_one_primary() {
    let f = fixture::facts();
    let c = fixture::current();
    let before = f.clone();
    f.validate().unwrap();
    for (a, z) in [
        (Local, Peer("peer-a")),
        (Local, Peer("peer-b")),
        (Peer("peer-a"), Peer("peer-b")),
    ] {
        assert!(f.visibility(&c, a, z).unwrap().mutual());
    }
    let projected = f.project(&c).unwrap();
    assert_eq!(projected.switches.len(), 3);
    assert!(!projected.hosts.contains_key("nic-a") && !projected.hosts.contains_key("nic-b"));
    assert_eq!(f, before, "projection must not mutate physical facts");
    let mut reversed = c.clone();
    reversed.members.reverse();
    assert_eq!(
        f.project(&reversed).unwrap(),
        projected,
        "both ports forward regardless of membership order"
    );
}
#[test]
fn detaching_a_port_blocks_that_segment_without_hiding_other_working_paths() {
    let f = fixture::facts();
    let mut c = fixture::current();
    c.members.pop();
    assert!(f.visibility(&c, Local, Peer("peer-a")).unwrap().mutual());
    assert!(f.visibility(&c, Local, Peer("peer-b")).unwrap().isolated());
    assert!(
        f.visibility(&c, Peer("peer-a"), Peer("peer-b"))
            .unwrap()
            .isolated()
    );
    c.members.clear();
    assert!(f.visibility(&c, Local, Peer("peer-a")).unwrap().isolated());
    assert!(
        f.visibility(&c, Peer("peer-a"), Peer("peer-a2"))
            .unwrap()
            .mutual()
    );
}
#[test]
fn bridge_down_does_not_break_external_same_segment_traffic() {
    let f = fixture::facts();
    let mut c = fixture::current();
    c.up = false;
    assert!(f.visibility(&c, Local, Peer("peer-a")).unwrap().isolated());
    assert!(
        f.visibility(&c, Peer("peer-a"), Peer("peer-b"))
            .unwrap()
            .isolated()
    );
    assert!(
        f.visibility(&c, Peer("peer-a"), Peer("peer-a2"))
            .unwrap()
            .mutual()
    );
}
#[test]
fn each_actual_physical_down_condition_blocks_only_its_attachment() {
    for cause in 0..3 {
        let mut f = fixture::facts();
        let c = fixture::current();
        match cause {
            0 => f.fabric.hosts.get_mut("nic-a").unwrap().up = false,
            1 => {
                f.fabric
                    .cables
                    .iter_mut()
                    .find(|c| c.id == "wire-nic-a")
                    .unwrap()
                    .up = false
            }
            _ => {
                f.fabric
                    .switches
                    .get_mut("left")
                    .unwrap()
                    .ports
                    .get_mut("nic-a")
                    .unwrap()
                    .up = false
            }
        }
        assert!(f.visibility(&c, Local, Peer("peer-a")).unwrap().isolated());
        assert!(f.visibility(&c, Local, Peer("peer-b")).unwrap().mutual());
        assert_eq!(
            c.members.len(),
            2,
            "a down physical path does not invent detachment"
        );
    }
}
#[test]
fn external_vlan_mismatch_is_a_delivery_failure_not_a_member_failover() {
    let mut f = fixture::facts();
    let c = fixture::current();
    let left = f.fabric.switches.get_mut("left").unwrap();
    left.vlans.insert(31);
    left.ports.get_mut("peer-a").unwrap().mode = s::Mode::Access { vlan: 31 };
    assert!(f.visibility(&c, Local, Peer("peer-a")).unwrap().isolated());
    assert!(f.visibility(&c, Local, Peer("peer-a2")).unwrap().mutual());
    assert!(f.visibility(&c, Local, Peer("peer-b")).unwrap().mutual());
    assert!(f.project(&c).unwrap().switches["sim-bridge"].ports["port-0"].up);
}
#[test]
fn stopped_peers_do_not_receive_and_each_direction_has_actual_paths() {
    let mut f = fixture::facts();
    let c = fixture::current();
    let rows = f.broadcast(&c, Local, None).unwrap();
    assert!(
        rows.recipients["peer-a"]
            .iter()
            .any(|h| h.switch == "sim-bridge")
    );
    assert!(
        rows.recipients["peer-b"]
            .iter()
            .any(|h| h.switch == "right")
    );
    f.fabric.hosts.get_mut("peer-a").unwrap().up = false;
    assert!(f.visibility(&c, Local, Peer("peer-a")).unwrap().isolated());
    assert!(f.visibility(&c, Local, Peer("peer-b")).unwrap().mutual());
}
#[test]
fn tags_trunks_unknown_endpoints_and_foreign_or_duplicate_members_fail_honestly() {
    let f = fixture::facts();
    let c = fixture::current();
    for tag in [0, 1, 30, 4095] {
        assert!(
            f.broadcast(&c, Local, Some(tag))
                .unwrap_err()
                .contains("Tagged")
        );
    }
    for endpoint in ["nic-a", "unknown", "../../etc/passwd"] {
        assert!(f.visibility(&c, Local, Peer(endpoint)).is_err());
    }
    assert!(f.visibility(&c, Local, Local).is_err());
    let mut duplicate = c.clone();
    duplicate.members.push(duplicate.members[0].clone());
    assert!(f.project(&duplicate).is_err());
    duplicate.members.truncate(2);
    duplicate.members[1] = duplicate.members[0].clone();
    assert!(f.project(&duplicate).is_err());
    let mut foreign = c.clone();
    foreign.members[1] = b::Interface("other0".into());
    assert!(f.project(&foreign).is_err());
    let mut trunk = f.clone();
    trunk
        .fabric
        .switches
        .get_mut("left")
        .unwrap()
        .ports
        .get_mut("nic-a")
        .unwrap()
        .mode = s::Mode::Trunk {
        native: 30,
        allowed: [30].into(),
    };
    assert!(trunk.validate().unwrap_err().contains("untagged"));
}
#[test]
fn a_potential_loop_is_rejected_even_if_its_extra_cable_is_down() {
    for up in [false, true] {
        let mut f = fixture::facts();
        for name in ["left", "right"] {
            f.fabric.switches.get_mut(name).unwrap().ports.insert(
                "cross".into(),
                s::Port {
                    up: true,
                    mode: s::Mode::Access { vlan: 30 },
                },
            );
        }
        f.fabric.cables.push(s::Cable {
            id: "cross".into(),
            a: s::Jack::Switch {
                switch: "left".into(),
                port: "cross".into(),
            },
            b: s::Jack::Switch {
                switch: "right".into(),
                port: "cross".into(),
            },
            up,
        });
        f.fabric.validate().unwrap();
        assert!(f.validate().unwrap_err().contains("cyclic"));
    }
}
#[test]
fn closed_source_and_snapshot_parsing_enforce_bounds_and_identifiers() {
    let f = fixture::facts();
    let original = serde_json::to_value(&f).unwrap();
    assert_eq!(b::Facts::from_json(&original.to_string()).unwrap(), f);
    for change in 0..6 {
        let mut v = original.clone();
        match change {
            0 => v["version"] = 2.into(),
            1 => v["host_command"] = "ip link set br0 up".into(),
            2 => v["ports"][1] = v["ports"][0].clone(),
            3 => v["ports"][0]["interface"] = "../../other".into(),
            4 => v["interface"] = "enp1s0".into(),
            _ => v["ports"][0]["endpoint"] = "missing".into(),
        }
        assert!(
            b::Facts::from_json(&v.to_string()).is_err(),
            "invalid source{change}"
        );
    }
    let mut oversized = original.to_string();
    oversized.push_str(&" ".repeat(65536));
    assert!(b::Facts::from_json(&oversized).is_err());
    assert!(
        serde_json::from_str::<b::Current>(r#"{"up":true,"members":["enp1s0","enp2s0","third"]}"#)
            .is_err()
    );
    assert!(
        serde_json::from_str::<b::Current>(r#"{"up":true,"members":[],"execute":"sh"}"#).is_err()
    );
}
#[test]
fn malformed_source_never_reaches_unchecked_projection_indexing() {
    let f = fixture::facts();
    let c = fixture::current();
    for case in 0..4 {
        let mut broken = f.clone();
        match case {
            0 => broken.fabric.hosts.clear(),
            1 => broken.ports.clear(),
            2 => broken.fabric.cables.clear(),
            _ => {
                broken.fabric.switches.clear();
            }
        }
        assert!(broken.project(&c).is_err());
    }
    let mut collision = f.clone();
    collision.fabric.hosts.insert(
        "sim-bridge-self".into(),
        s::Host {
            label: "collision".into(),
            up: true,
        },
    );
    assert!(collision.validate().unwrap_err().contains("reserved"));
}
