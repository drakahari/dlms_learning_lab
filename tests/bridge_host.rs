//! Actual two-way untagged paths plus separate IPv4/current-state checks.
#[path = "support/bridge_host.rs"]
mod fixture;
use dlms_learning_lab::{bridge_control as control, bridge_host as b, nm_profiles as ip};
use fixture::addr;
#[test]
fn manual_controller_reaches_each_attached_segment_and_peers_forward_between_ports() {
    let f = fixture::facts();
    let h = fixture::physical();
    let s = fixture::snapshot(&fixture::files());
    let p = fixture::selected();
    f.validate(&h).unwrap();
    f.validate_selection(&s, &p).unwrap();
    for a in ["10.220.0.1", "10.220.0.50", "10.220.0.60"] {
        assert!(f.delivery(&h, &s, &p, "br0", addr(a)).unwrap().round_trip());
    }
    assert!(
        f.peer_delivery(&h, &s, addr("10.220.0.50"), addr("10.220.0.60"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn missing_binding_and_physical_down_do_not_create_a_bond_failover() {
    let mut f = fixture::facts();
    let h = fixture::physical();
    let p = fixture::selected();
    let files = fixture::files();
    let mut unbound = files.clone();
    let r = unbound.get_mut("right.nmconnection").unwrap();
    *r = r.replace("controller=br0\nport-type=bridge\n", "").replace(
        "[bridge-port]",
        "[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled",
    );
    let s = fixture::snapshot(&unbound);
    assert!(
        f.delivery(&h, &s, &p, "br0", addr("10.220.0.50"))
            .unwrap()
            .round_trip()
    );
    assert!(
        !f.delivery(&h, &s, &p, "br0", addr("10.220.0.60"))
            .unwrap()
            .round_trip()
    );
    assert!(
        !f.peer_delivery(&h, &s, addr("10.220.0.50"), addr("10.220.0.60"))
            .unwrap()
            .round_trip()
    );
    f.lan.fabric.hosts.get_mut("nic-a").unwrap().up = false;
    let s = fixture::snapshot(&files);
    assert!(
        !f.delivery(&h, &s, &p, "br0", addr("10.220.0.50"))
            .unwrap()
            .round_trip()
    );
    assert!(
        f.delivery(&h, &s, &p, "br0", addr("10.220.0.60"))
            .unwrap()
            .round_trip()
    );
    assert_eq!(s.current.members.len(), 2);
}
#[test]
fn wrong_ip_is_not_rescued_by_a_working_bridge_and_duplicate_peer_ip_is_rejected() {
    let f = fixture::facts();
    let h = fixture::physical();
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\naddress1=10.221.0.20/24\n");
    let s = fixture::snapshot(&files);
    let mut p = fixture::selected();
    p.get_mut("br0").unwrap().address.address = addr("10.221.0.20");
    let l2 = f
        .delivery(&h, &s, &p, "br0", addr("10.220.0.60"))
        .unwrap()
        .round_trip();
    assert!(l2);
    let l3 = fixture::host_state(&p).probe(addr("10.220.0.60")).pass;
    assert!(!l3);
    assert!(!(l3 && l2));
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\naddress1=10.220.0.60/24\n");
    let s = fixture::snapshot(&files);
    p.get_mut("br0").unwrap().address.address = addr("10.220.0.60");
    assert!(
        f.delivery(&h, &s, &p, "br0", addr("10.220.0.60"))
            .unwrap_err()
            .message
            .contains("duplicates")
    );
}
#[test]
fn another_usable_controller_address_and_optional_stp_choice_are_valid() {
    let f = fixture::facts();
    let h = fixture::physical();
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[bridge]\nstp=false\n[ipv4]\naddress1=10.220.0.21/24\n");
    let s = fixture::snapshot(&files);
    let mut p = fixture::selected();
    p.get_mut("br0").unwrap().address.address = addr("10.220.0.21");
    assert!(!s.plan.controller().stp);
    assert!(
        f.delivery(&h, &s, &p, "br0", addr("10.220.0.60"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn selected_ip_cannot_be_reinterpreted_with_a_foreign_or_edited_current_snapshot() {
    let f = fixture::facts();
    let p = fixture::selected();
    let files = fixture::files();
    let current = fixture::snapshot(&files);
    let before = current.clone();
    let mut changed = files.clone();
    changed
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\naddress1=10.220.0.21/24\n");
    let future = fixture::snapshot(&changed);
    assert!(f.validate_selection(&future, &p).is_err());
    assert!(f.validate_selection(&current, &p).is_ok());
    assert_eq!(current, before);
    let mut source = f.profiles.clone();
    source.controller.uuid = "30000000-0000-4000-8000-000000000099".into();
    let mut foreign = files;
    *foreign.get_mut("bridge.nmconnection").unwrap() = foreign["bridge.nmconnection"].replace(
        "30000000-0000-4000-8000-000000000001",
        &source.controller.uuid,
    );
    let foreign =
        control::Snapshot::automatic(control::Plan::from_files(&source, &foreign).unwrap())
            .unwrap();
    assert!(f.current(&foreign).is_err());
    assert!(f.validate_selection(&current, &Default::default()).is_err());
}
#[test]
fn source_peer_mapping_and_logical_interface_roles_are_complete_and_closed() {
    let original = fixture::facts();
    let h = fixture::physical();
    for case in 0..6 {
        let mut f = original.clone();
        match case {
            0 => {
                f.peers.pop();
            }
            1 => f.peers[1].address = f.peers[0].address,
            2 => f.peers[0].endpoint = "nic-a".into(),
            3 => f.profiles.members[0].interface = "other0".into(),
            4 => f.peers[0].address = addr("10.221.0.1"),
            _ => f.peers[0].address = addr("10.220.0.0"),
        }
        assert!(f.validate(&h).is_err(), "invalidsource{case}");
    }
    let mut h = h;
    h.links[0].name = "enp1s0".into();
    assert!(original.validate(&h).is_err());
    let json = serde_json::to_string(&original).unwrap();
    assert_eq!(serde_json::from_str::<b::Facts>(&json).unwrap(), original);
    assert!(
        serde_json::from_str::<b::Facts>(
            &json.replace("\"version\":1", "\"version\":1,\"exec\":\"host\"")
        )
        .is_err()
    );
}
#[test]
fn routed_targets_need_the_real_gateway_endpoint_not_an_invented_remote_lan_peer() {
    let f = fixture::facts();
    let h = fixture::physical();
    let s = fixture::snapshot(&fixture::files());
    let p = fixture::selected();
    assert!(
        f.delivery(&h, &s, &p, "br0", addr("10.220.0.1"))
            .unwrap()
            .round_trip()
    );
    assert!(f.delivery(&h, &s, &p, "br0", addr("10.230.0.50")).is_err());
    assert!(
        f.delivery(&h, &s, &p, "enp1s0", addr("10.220.0.1"))
            .is_err()
    );
    assert!(
        f.peer_delivery(&h, &s, addr("10.220.0.50"), addr("10.230.0.50"))
            .is_err()
    );
    let mut p = p;
    p.get_mut("br0").unwrap().vlan = Some(ip::Vlan {
        parent: "enp1s0".into(),
        id: 30,
    });
    assert!(f.validate_selection(&s, &p).is_err());
}
