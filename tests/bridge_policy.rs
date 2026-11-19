//! Current/future bridge+IP choices use real modeled paths, never keyfile strings as grades.
#[path = "support/bridge_policy.rs"]
mod fixture;
use dlms_learning_lab::{nm_policy as n, nm_profiles::ErrorKind};
use fixture::model::addr;
#[test]
fn automatic_bridge_choices_join_all_ports_and_real_ip_paths() {
    let e = fixture::environment();
    let files = fixture::files();
    e.validate().unwrap();
    let p = n::Policy::from_texts_for(&e, &files).unwrap();
    assert!(p.control().is_none());
    assert!(p.bridge_control().is_some());
    assert_eq!(p.profiles().count(), 1);
    assert!(n::Policy::from_texts(&files).is_err());
    let c = fixture::future(&e, &files);
    assert_eq!(c.bridge_controls().unwrap().current.members.len(), 2);
    for target in ["10.220.0.1", "10.220.0.50", "10.220.0.60"] {
        let r = c.probe(&e, addr(target));
        assert!(r.pass, "{}", r.reason);
        assert_eq!(r.selection.unwrap().route.device, "br0");
    }
    assert!(
        c.bridge_peer_delivery(&e, addr("10.220.0.50"), addr("10.220.0.60"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn missing_port_and_actual_down_segment_fail_without_fabricated_fallback() {
    let e = fixture::environment();
    let mut files = fixture::files();
    let r = files.get_mut("right.nmconnection").unwrap();
    *r = r.replace("controller=br0\nport-type=bridge\n", "").replace(
        "[bridge-port]",
        "[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled",
    );
    let c = fixture::future(&e, &files);
    assert!(c.probe(&e, addr("10.220.0.50")).pass);
    assert!(!c.probe(&e, addr("10.220.0.60")).pass);
    let mut e = e;
    e.bridge
        .as_mut()
        .unwrap()
        .lan
        .fabric
        .hosts
        .get_mut("nic-a")
        .unwrap()
        .up = false;
    let c = fixture::future(&e, &fixture::files());
    assert!(!c.probe(&e, addr("10.220.0.50")).pass);
    assert!(c.probe(&e, addr("10.220.0.60")).pass);
    assert_eq!(c.bridge_controls().unwrap().current.members.len(), 2);
}
#[test]
fn bad_ip_and_colliding_peer_are_not_successful_on_a_working_bridge() {
    let e = fixture::environment();
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\naddress1=10.221.0.20/24\n");
    let c = fixture::future(&e, &files);
    assert!(!c.probe(&e, addr("10.220.0.60")).pass);
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\naddress1=10.220.0.60/24\n");
    let p = n::Policy::from_texts_for(&e, &files).unwrap();
    assert!(
        n::Choices::empty()
            .automatic(&e, &p)
            .unwrap_err()
            .message
            .contains("duplicates")
    );
}
#[test]
fn actual_gateway_selection_uses_its_l2_endpoint_for_remote_destinations() {
    let mut e = fixture::environment();
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\nroute1=10.230.0.0/24,10.220.0.1,70\n");
    let c = fixture::future(&e, &files);
    let r = c.probe(&e, addr("10.230.0.50"));
    assert!(r.pass, "{}", r.reason);
    assert_eq!(r.selection.unwrap().route.via, Some(addr("10.220.0.1")));
    e.bridge
        .as_mut()
        .unwrap()
        .lan
        .fabric
        .hosts
        .get_mut("nic-a")
        .unwrap()
        .up = false;
    assert!(!c.probe(&e, addr("10.230.0.50")).pass);
    assert!(c.probe(&e, addr("10.220.0.60")).pass);
}
#[test]
fn atomic_supplied_current_selection_is_not_replaced_by_edited_future_policy() {
    let e = fixture::environment();
    let files = fixture::files();
    let p = n::Policy::from_texts_for(&e, &files).unwrap();
    let current = fixture::model::snapshot(&files).current;
    let c = n::Choices::empty()
        .seed_current_bridge(&e, &p, &current)
        .unwrap();
    assert!(c.for_device("br0").is_some());
    let before = c.clone();
    let mut edited = files;
    edited
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect-ports=0\n[ipv4]\naddress1=10.221.0.20/24\n");
    let p = n::Policy::from_texts_for(&e, &edited).unwrap();
    let (warm, _) = c.automatic(&e, &p).unwrap();
    assert_eq!(warm, before);
    assert!(warm.probe(&e, addr("10.220.0.60")).pass);
    let (future, _) = n::Choices::empty().automatic(&e, &p).unwrap();
    assert!(!future.probe(&e, addr("10.220.0.60")).pass);
    assert!(future.bridge_controls().unwrap().current.members.is_empty());
    assert!(warm.seed_current_bridge(&e, &p, &current).is_err());
}
#[test]
fn protected_controller_checks_port_start_and_stp_not_only_its_ip_properties() {
    let e = fixture::environment();
    let files = fixture::files();
    let original = n::Policy::from_texts_for(&e, &files).unwrap();
    assert!(!original.protected_unchanged(&original, "unknown.nmconnection"));
    for (name, tail) in [
        (
            "bridge.nmconnection",
            "\n[connection]\nautoconnect-ports=0\n",
        ),
        ("bridge.nmconnection", "\n[bridge]\nstp=false\n"),
        ("right.nmconnection", "\n[connection]\nautoconnect=true\n"),
    ] {
        let mut changed = files.clone();
        changed.get_mut(name).unwrap().push_str(tail);
        let changed = n::Policy::from_texts_for(&e, &changed).unwrap();
        assert!(!original.protected_unchanged(&changed, name));
    }
    let mut equivalent = files;
    equivalent
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n# equivalent declaration\n[bridge]\nstp=true\nvlan-filtering=false\n");
    assert!(original.protected_unchanged(
        &n::Policy::from_texts_for(&e, &equivalent).unwrap(),
        "bridge.nmconnection"
    ));
}
#[test]
fn source_gate_rejects_nested_capabilities_and_ip_on_members_or_an_extra_controller() {
    let e = fixture::environment();
    let files = fixture::files();
    for interface in ["br0", "enp1s0"] {
        let mut files = files.clone();
        files.insert("replacement.nmconnection".into(),format!("[connection]\nid=Replacement\nuuid=30000000-0000-4000-8000-000000000099\ntype=ethernet\ninterface-name={interface}\n[ipv4]\nmethod=manual\naddress1=10.220.0.21/24\n[ipv6]\nmethod=disabled\n"));
        assert_eq!(
            n::Policy::from_texts_for(&e, &files).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    let mut nested = e.clone();
    nested.dhcp.insert(
        "br0".into(),
        dlms_learning_lab::nm_dhcp::Outcome::Unavailable {},
    );
    assert_eq!(nested.validate().unwrap_err().kind, ErrorKind::Unsupported);
    let mut foreign = e;
    foreign.bridge = None;
    let c = fixture::future(&fixture::environment(), &files);
    assert!(!c.probe(&foreign, addr("10.220.0.60")).pass);
    assert!(
        c.bridge_peer_delivery(&foreign, addr("10.220.0.50"), addr("10.220.0.60"))
            .is_err()
    );
}
#[test]
fn equivalent_addresses_port_flags_and_stp_choices_are_graded_by_paths() {
    let e = fixture::environment();
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[bridge]\nstp=false\n[ipv4]\naddress1=10.220.0.21/24\n");
    for name in ["left.nmconnection", "right.nmconnection"] {
        files
            .get_mut(name)
            .unwrap()
            .push_str("\n[connection]\nautoconnect=true\n");
    }
    let c = fixture::future(&e, &files);
    assert_eq!(
        c.for_device("br0").unwrap().address.address,
        addr("10.220.0.21")
    );
    assert!(c.probe(&e, addr("10.220.0.60")).pass);
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect-ports=0\n");
    let p = n::Policy::from_texts_for(&e, &files).unwrap();
    assert_eq!(
        n::Choices::empty().automatic(&e, &p).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
