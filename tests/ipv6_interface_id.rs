use dlms_learning_lab::ipv6::{self, Assignment, Prefix, Requirement, Topology, interface_id};
use std::net::Ipv6Addr;
fn network(text: &str) -> Prefix {
    Prefix::parse(text).unwrap()
}
fn topology(a: Option<Assignment>) -> Topology {
    let mut t: Topology = serde_json::from_str(r#"{"devices":{"Client":{"forwarding":false,"interfaces":{"lan":{"link":"office","up":true,"link_local":"fe80::10","address":null}},"routes":[]}}}"#).unwrap();
    t.devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = a;
    assert!(t.valid());
    t
}
fn assigned(text: &str, prefix: u8, on_link: bool) -> Option<Assignment> {
    Some(Assignment {
        address: text.parse().unwrap(),
        prefix,
        on_link,
    })
}
fn requirement() -> Requirement {
    Requirement::ModifiedEui64 {
        device: "Client".into(),
        interface: "lan".into(),
        mac: "06:25:96:ab:cd:ef".into(),
        network: network("2001:db8:74::/64"),
    }
}
#[test]
fn known_vectors_insert_in_the_middle_and_toggle_in_both_directions() {
    for (mac, result) in [
        ("00:16:3e:12:34:56", "fd64:7:10:0:216:3eff:fe12:3456"),
        ("06:25:96:ab:cd:ef", "fd64:7:10:0:425:96ff:feab:cdef"),
        ("AA:BB:CC:DD:EE:FF", "fd64:7:10:0:a8bb:ccff:fedd:eeff"),
        ("02:00:00:00:00:01", "fd64:7:10::ff:fe00:1"),
    ] {
        assert_eq!(
            interface_id::address(mac, &network("fd64:7:10::/64")).unwrap(),
            result.parse::<Ipv6Addr>().unwrap()
        );
    }
}
#[test]
fn every_unicast_first_byte_uses_xor_and_preserves_all_other_mac_bytes() {
    for first in (0u8..=254).step_by(2) {
        let mac = format!("{first:02x}:b2:c3:d4:e5:f6");
        let bytes = interface_id::modified_eui64(&mac).unwrap();
        assert_eq!(bytes, [first ^ 2, 0xb2, 0xc3, 0xff, 0xfe, 0xd4, 0xe5, 0xf6]);
        for prefix in ["2001:db8:abcd:1234::/64", "fd76:99:aa:bb::/64"] {
            let p = network(prefix);
            let address = interface_id::address(&mac, &p).unwrap();
            assert_eq!(&address.octets()[..8], &p.address.octets()[..8]);
            assert_eq!(&address.octets()[8..], &bytes);
        }
    }
}
#[test]
fn unsupported_format_group_zero_and_noncanonical_network_facts_are_rejected() {
    for mac in [
        "",
        "00:11:22:33:44",
        "0:11:22:33:44:55",
        "00-11-22-33-44-55",
        "00:11:22:33:44:gg",
        "00:11:22:33:44:55\n",
        "01:11:22:33:44:55",
        "ff:ff:ff:ff:ff:ff",
        "00:00:00:00:00:00",
    ] {
        assert!(interface_id::modified_eui64(mac).is_err(), "{mac:?}");
    }
    for p in [
        Prefix {
            address: "2001:db8::1".parse().unwrap(),
            length: 64,
        },
        network("2001:db8::/48"),
        network("ff02::/64"),
        network("fe80::/64"),
        network("::/64"),
    ] {
        assert!(interface_id::address("00:11:22:33:44:55", &p).is_err());
    }
}
#[test]
fn feedback_distinguishes_empty_prefix_network_toggle_and_other_identifier_errors() {
    let r = requirement();
    for (a, needle) in [
        (None, "Enter the full IPv6 address"),
        (
            assigned("2001:db8:74:0:425:96ff:feab:cdef", 48, true),
            "Keep /64",
        ),
        (
            assigned("2001:db8:75:0:425:96ff:feab:cdef", 64, true),
            "network half",
        ),
        (
            assigned("2001:db8:74:0:625:96ff:feab:cdef", 64, true),
            "not toggled",
        ),
        (
            assigned("2001:db8:74:0:425:96ab:cdff:feef", 64, true),
            "insert ff:fe",
        ),
        (
            assigned("2001:db8:74:0:425:96ff:feab:cdef", 64, false),
            "on-link",
        ),
    ] {
        let t = topology(a);
        let (passed, message) = r.check(&t);
        assert!(!passed, "{message}");
        assert!(message.contains(needle), "{message}");
    }
    for good in [
        "2001:db8:74:0:425:96ff:feab:cdef",
        "2001:0DB8:0074:0000:0425:96FF:FEAB:CDEF",
    ] {
        assert!(r.check(&topology(assigned(good, 64, true))).0);
    }
}
#[test]
fn invalid_goal_facts_and_post_success_damage_never_retain_success() {
    let r = requirement();
    let mut t = topology(assigned("2001:db8:74:0:425:96ff:feab:cdef", 64, true));
    assert!(r.valid_for(&t));
    assert!(r.check(&t).0);
    t.devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = "2001:db8:74:0:625:96ff:feab:cdef".parse().unwrap();
    assert!(!r.check(&t).0);
    let bad = Requirement::ModifiedEui64 {
        device: "Client".into(),
        interface: "absent".into(),
        mac: "00:16:3e:12:34:56".into(),
        network: network("2001:db8::/64"),
    };
    assert!(!bad.valid_for(&t));
    assert!(!bad.check(&t).0);
    let json = serde_json::to_value(r).unwrap();
    assert!(serde_json::from_value::<Requirement>(json).is_ok());
    assert_eq!(
        ipv6::category("ff02::1".parse().unwrap()),
        ipv6::Category::Multicast
    );
}
