use dlms_learning_lab::ipv4_routing::{
    Assignment, Device, Interface, Prefix, Requirement, Topology,
};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn fixture(address: &str, prefix: u8) -> Topology {
    Topology {
        devices: BTreeMap::from([(
            "Branch".into(),
            Device {
                forwarding: false,
                routes: vec![],
                interfaces: BTreeMap::from([(
                    "lan".into(),
                    Interface {
                        link: "branch".into(),
                        up: true,
                        address: Some(Assignment {
                            address: ip(address),
                            prefix,
                        }),
                    },
                )]),
            },
        )]),
    }
}
fn requirement(minimum_hosts: u32, maximum_hosts: u32, avoid: &[&str]) -> Requirement {
    Requirement::PrivateSubnet {
        device: "Branch".into(),
        interface: "lan".into(),
        minimum_hosts,
        maximum_hosts,
        avoid: avoid.iter().map(|s| Prefix::parse(s).unwrap()).collect(),
    }
}
#[test]
fn accepts_different_private_pools_hosts_and_prefixes_by_result_not_one_answer() {
    let r = requirement(20, 254, &["10.19.0.0/16", "172.18.0.0/24"]);
    for (address, prefix) in [
        ("10.20.3.17", 27),
        ("172.20.3.19", 26),
        ("192.168.4.6", 24),
        ("10.21.0.62", 26),
    ] {
        let t = fixture(address, prefix);
        assert!(t.valid());
        assert!(r.check(&t).0, "{address}/{prefix}: {:?}", r.check(&t));
    }
}
#[test]
fn checks_entire_subnet_not_just_the_private_host_address() {
    let r = requirement(1, (1 << 24) - 2, &[]);
    for (address, prefix) in [
        ("10.1.2.3", 7),
        ("172.16.2.3", 11),
        ("192.168.2.3", 15),
        ("192.0.2.17", 24),
    ] {
        let t = fixture(address, prefix);
        assert!(t.valid());
        let (ok, message) = r.check(&t);
        assert!(!ok);
        assert!(message.contains("entire subnet"), "{message}");
        assert!(!message.contains("is a public"));
    }
    for (address, prefix) in [
        ("10.1.2.3", 8),
        ("172.31.255.254", 12),
        ("192.168.255.254", 16),
    ] {
        assert!(r.check(&fixture(address, prefix)).0);
    }
}
#[test]
fn diagnoses_capacity_missing_address_and_reserved_host_values() {
    let r = requirement(20, 62, &[]);
    for (address, prefix, reason) in [
        ("10.2.0.1", 28, "14 usable"),
        ("10.2.0.1", 24, "254 usable"),
        ("10.2.0.0", 27, "reserved"),
        ("10.2.0.31", 27, "reserved"),
    ] {
        let (ok, message) = r.check(&fixture(address, prefix));
        assert!(!ok);
        assert!(message.contains(reason), "{message}");
    }
    let mut t = fixture("10.2.0.1", 27);
    t.devices
        .get_mut("Branch")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = None;
    assert!(r.check(&t).1.contains("No address"));
}
#[test]
fn overlap_in_both_directions_and_touching_nonoverlapping_blocks() {
    let r = requirement(1, 65534, &["10.19.16.0/24"]);
    for (address, prefix) in [("10.19.16.3", 27), ("10.19.0.3", 16), ("10.19.16.3", 24)] {
        let (ok, message) = r.check(&fixture(address, prefix));
        assert!(!ok);
        assert!(message.contains("overlaps"), "{message}");
    }
    assert!(r.check(&fixture("10.19.17.1", 24)).0);
    assert!(r.check(&fixture("10.19.15.254", 24)).0);
}
#[test]
fn success_is_recomputed_after_break_and_repair_and_roundtrips() {
    let r = requirement(20, 62, &["10.19.0.0/16"]);
    let encoded = serde_json::to_string(&r).unwrap();
    let decoded: Requirement = serde_json::from_str(&encoded).unwrap();
    let mut t = fixture("10.20.0.7", 27);
    assert!(decoded.check(&t).0);
    t.devices
        .get_mut("Branch")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = ip("10.19.0.7");
    assert!(!decoded.check(&t).0);
    t.devices
        .get_mut("Branch")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = ip("192.168.2.7");
    assert!(decoded.check(&t).0);
}
#[test]
fn invalid_requirement_metadata_is_refused_before_grading() {
    let t = fixture("10.20.0.7", 27);
    for r in [
        requirement(0, 20, &[]),
        requirement(30, 20, &[]),
        requirement(1, 1 << 24, &[]),
        requirement(1, 20, &["10.0.0.0/8", "10.0.0.0/8"]),
        requirement(1, 20, &["10.0.0.0/8"; 17]),
    ] {
        assert!(!r.valid_for(&t));
        assert!(!r.check(&t).0);
    }
    let mut r = requirement(1, 20, &[]);
    if let Requirement::PrivateSubnet { device, .. } = &mut r {
        *device = "Missing".into();
    }
    assert!(!r.valid_for(&t));
    assert!(!r.check(&t).0);
    let mut value = serde_json::to_value(requirement(1, 20, &[])).unwrap();
    value["executable"] = true.into();
    assert!(serde_json::from_value::<Requirement>(value).is_err());
}

#[test]
fn new_requirement_selects_its_device_without_revealing_a_solution() {
    let r = requirement(20, 62, &[]);
    assert_eq!(
        dlms_learning_lab::ipv4_routing_teaching::preferred_device(&[r]).as_deref(),
        Some("Branch")
    );
}

fn paired(left: &str, ln: u8, right: &str, rn: u8) -> Topology {
    let mut t = fixture(left, ln);
    let mut peer = t.devices["Branch"].clone();
    peer.interfaces.get_mut("lan").unwrap().address = Some(Assignment {
        address: ip(right),
        prefix: rn,
    });
    t.devices.insert("Peer".into(), peer);
    assert!(t.valid());
    t
}
fn same() -> Requirement {
    Requirement::SameSubnet {
        left_device: "Branch".into(),
        left_interface: "lan".into(),
        right_device: "Peer".into(),
        right_interface: "lan".into(),
    }
}
#[test]
fn same_subnet_accepts_alternate_blocks_hosts_and_octet_crossings() {
    let r = same();
    for (left, right, n) in [
        ("10.2.1.1", "10.2.1.30", 27),
        ("172.20.2.10", "172.20.3.254", 23),
        ("192.168.2.2", "192.168.2.61", 26),
        ("10.1.0.1", "10.1.1.254", 23),
    ] {
        let t = paired(left, n, right, n);
        assert!(r.valid_for(&t));
        assert!(r.check(&t).0);
    }
    let encoded = serde_json::to_string(&r).unwrap();
    let decoded: Requirement = serde_json::from_str(&encoded).unwrap();
    assert!(decoded.check(&paired("10.2.1.1", 27, "10.2.1.3", 27)).0);
    assert_eq!(
        dlms_learning_lab::ipv4_routing_teaching::preferred_device(&[decoded]).as_deref(),
        Some("Branch")
    );
}
#[test]
fn successful_communication_cannot_hide_mismatched_subnet_masks() {
    let mut t = paired("10.2.1.1", 27, "10.2.1.2", 26);
    assert!(
        t.probe("Branch", ip("10.2.1.2")).is_ok(),
        "Both paths really work with these mismatched masks"
    );
    let (ok, message) = same().check(&t);
    assert!(!ok);
    assert!(message.contains("matching prefixes"));
    t.devices
        .get_mut("Peer")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .prefix = 27;
    assert!(same().check(&t).0);
    t.devices
        .get_mut("Peer")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = ip("10.2.2.2");
    assert!(!same().check(&t).0);
    assert!(same().check(&t).1.contains("different subnets"));
}
#[test]
fn duplicate_reserved_missing_or_unrelated_peers_never_pass() {
    for (left, right, n, reason) in [
        ("10.2.1.1", "10.2.1.1", 27, "different host"),
        ("10.2.1.0", "10.2.1.3", 27, "reserved"),
        ("10.2.1.1", "10.2.1.31", 27, "reserved"),
    ] {
        let (ok, message) = same().check(&paired(left, n, right, n));
        assert!(!ok);
        assert!(message.contains(reason), "{message}");
    }
    let mut t = paired("10.2.1.1", 27, "10.2.1.3", 27);
    t.devices
        .get_mut("Peer")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = None;
    assert!(same().check(&t).1.contains("both computers"));
    t.devices
        .get_mut("Peer")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .link = "elsewhere".into();
    assert!(!same().valid_for(&t));
    assert!(!same().check(&t).0);
    let r = Requirement::SameSubnet {
        left_device: "Branch".into(),
        left_interface: "lan".into(),
        right_device: "Branch".into(),
        right_interface: "lan".into(),
    };
    assert!(!r.valid_for(&t));
    assert!(!r.check(&t).0);
}
