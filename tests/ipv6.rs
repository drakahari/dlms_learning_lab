use dlms_learning_lab::ipv6::{self, Category, Prefix, Topology};
mod support;
use support::*;
#[test]
fn every_prefix_is_correct_across_all_128_bits_without_overflow() {
    let address = ip("abcd:1234:5678:9abc:def0:1357:2468:ffff");
    for p in 0..=128 {
        let prefix = Prefix::new(address, p).unwrap();
        assert!(prefix.valid());
        assert!(prefix.contains(address));
        assert_eq!(
            u128::from(prefix.address),
            u128::from(address) & ipv6::mask(p).unwrap()
        );
        assert_eq!(Prefix::parse(&prefix.text()).unwrap(), prefix);
        if p > 0 {
            assert!(!prefix.contains((u128::from(address) ^ (1_u128 << (128 - p))).into()));
        }
        if p < 128 {
            assert!(prefix.contains((u128::from(address) ^ 1).into()));
        }
    }
    assert_eq!(
        Prefix::new(ip("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"), 0)
            .unwrap()
            .text(),
        "::/0"
    );
    assert_eq!(
        Prefix::new(ip("2001:db8:1234:5678:9abc::1"), 63)
            .unwrap()
            .text(),
        "2001:db8:1234:5678::/63"
    );
    assert!(ipv6::mask(129).is_none());
    assert!(
        Prefix::parse("2001:db8::1/64")
            .unwrap_err()
            .contains("nonzero bits")
    );
    for bad in [
        "2001:::1/64",
        "2001:db8::/129",
        "2001:db8::/-1",
        "2001:db8::/+64",
        "2001:db8::/64/1",
    ] {
        assert!(Prefix::parse(bad).is_err(), "{bad}");
    }
    assert_eq!(
        ip("2001:0DB8:0000:0000:0000:0000:0000:ABCD"),
        ip("2001:db8::abcd")
    );
}
#[test]
fn classification_uses_current_documentation_ranges_without_claiming_broadcast_or_anycast() {
    for (text, expected) in [
        ("::", Category::Unspecified),
        ("::1", Category::Loopback),
        ("::ffff:192.0.2.1", Category::Ipv4Mapped),
        ("ff02::1", Category::Multicast),
        ("fe80::1", Category::LinkLocal),
        ("febf::1", Category::LinkLocal),
        ("fec0::1", Category::OtherUnicast),
        ("fc00::1", Category::UniqueLocal),
        ("fdff::1", Category::UniqueLocal),
        ("2001:db8::1", Category::Documentation),
        ("3fff:fff:ffff::1", Category::Documentation),
        ("3fff:1000::1", Category::OtherUnicast),
    ] {
        assert_eq!(ipv6::category(ip(text)), expected, "{text}");
    }
    // All-one host bits are an ordinary configured address here, not IPv4 broadcast.
    let mut t = topology();
    t.devices
        .get_mut("server")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = ip("2001:db8:20:0:ffff:ffff:ffff:ffff");
    assert!(
        t.ping("workstation", ip("2001:db8:20:0:ffff:ffff:ffff:ffff"), None)
            .is_ok()
    );
}
#[test]
fn forwarding_and_reply_are_derived_from_current_typed_state() {
    let mut t = topology();
    assert!(t.valid());
    let ping = t.ping("workstation", ip("2001:db8:20::20"), None).unwrap();
    assert_eq!(ping.source, ip("2001:db8:10::10"));
    assert_eq!(ping.forward.hops.len(), 2);
    assert_eq!(ping.reply.hops.len(), 2);
    assert_eq!(ping.forward.hops[0].next_hop, ip("fe80::1"));
    t.devices.get_mut("server").unwrap().routes.clear();
    let error = t
        .ping("workstation", ip("2001:db8:20::20"), None)
        .unwrap_err();
    assert!(
        error.contains("Forward delivery works, but the reply fails"),
        "{error}"
    );
    t = topology();
    t.devices.get_mut("router").unwrap().forwarding = false;
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("forwarding is disabled")
    );
    t = topology();
    t.devices
        .get_mut("router")
        .unwrap()
        .interfaces
        .get_mut("right")
        .unwrap()
        .up = false;
    assert!(t.ping("workstation", ip("2001:db8:20::20"), None).is_err());
    t = topology();
    t.devices.get_mut("workstation").unwrap().routes[0].via = Some(ip("fe80::99"));
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("no up neighbor")
    );
    t = topology();
    t.devices.get_mut("workstation").unwrap().routes[0].via = Some(ip("2001:db8:99::1"));
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("no active route")
    );
}
#[test]
fn link_local_scope_is_per_link_and_cannot_escape_through_router() {
    let t = topology();
    assert!(
        t.ping("router", ip("fe80::10"), None)
            .unwrap_err()
            .contains("outgoing interface")
    );
    assert_eq!(
        t.ping("router", ip("fe80::10"), Some("left"))
            .unwrap()
            .forward
            .destination_device,
        "workstation"
    );
    assert!(
        t.ping("router", ip("fe80::10"), Some("right"))
            .unwrap_err()
            .contains("no up neighbor")
    );
    assert!(t.ping("workstation", ip("fe80::20"), Some("lan")).is_err());
    // Same textual router address on two different links is valid, not a duplicate.
    assert!(t.ping("server", ip("fe80::1"), Some("lan")).is_ok());
    // Scope must constrain the receiving interface, too: a different router port cannot answer.
    let mut t = t.clone();
    t.devices
        .get_mut("router")
        .unwrap()
        .interfaces
        .get_mut("right")
        .unwrap()
        .link_local = ip("fe80::99");
    assert!(t.ping("workstation", ip("fe80::99"), Some("lan")).is_err());
}
#[test]
fn longest_prefix_precedes_distance_and_equal_cost_is_honest_unsupported() {
    let mut t = topology();
    t.devices.get_mut("workstation").unwrap().routes.push(route(
        "2001:db8:20::/64",
        "lan",
        "fe80::99",
        200,
    ));
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("fe80::99")
    );
    t.devices.get_mut("workstation").unwrap().routes[1].via = Some(ip("fe80::1"));
    assert!(t.ping("workstation", ip("2001:db8:20::20"), None).is_ok());
    t.devices.get_mut("workstation").unwrap().routes.push(route(
        "2001:db8:20::/64",
        "lan",
        "fe80::88",
        201,
    ));
    assert!(t.ping("workstation", ip("2001:db8:20::20"), None).is_ok());
    t.devices.get_mut("workstation").unwrap().routes[2].distance = 200;
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("equal-cost")
    );
    t.devices.get_mut("workstation").unwrap().routes[2].distance = 255;
    assert!(t.ping("workstation", ip("2001:db8:20::20"), None).is_ok());
    // A more-specific host route wins over the connected prefix, even at higher distance.
    let mut t = topology();
    t.devices.get_mut("workstation").unwrap().routes.push(route(
        "2001:db8:10::1/128",
        "lan",
        "fe80::99",
        250,
    ));
    assert!(
        t.ping("workstation", ip("2001:db8:10::1"), None)
            .unwrap_err()
            .contains("fe80::99")
    );
}
#[test]
fn an_address_prefix_is_not_implicitly_an_on_link_declaration() {
    let mut t = topology();
    let i = t
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap();
    i.address.as_mut().unwrap().on_link = false;
    // Link-local next-hop route still works even without a connected global prefix.
    assert!(t.ping("workstation", ip("2001:db8:20::20"), None).is_ok());
    t.devices.get_mut("workstation").unwrap().routes[0].via = Some(ip("2001:db8:10::1"));
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("no active route")
    );
    t.devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .on_link = true;
    assert!(t.ping("workstation", ip("2001:db8:20::20"), None).is_ok());
}
#[test]
fn duplicates_route_loops_and_invalid_graphs_are_bounded_failures() {
    let mut t = topology();
    let duplicate = t.devices["server"].clone();
    t.devices.insert("duplicate".into(), duplicate);
    assert!(t.valid());
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("duplicate")
    );
    let mut t = topology();
    t.devices.get_mut("router").unwrap().routes.push(route(
        "2001:db8:99::/64",
        "left",
        "fe80::10",
        1,
    ));
    t.devices.get_mut("workstation").unwrap().forwarding = true;
    assert!(
        t.ping("workstation", ip("2001:db8:99::1"), None)
            .unwrap_err()
            .contains("Routing loop")
    );
    for edit in 0..6 {
        let mut t = topology();
        let d = t.devices.get_mut("workstation").unwrap();
        match edit {
            0 => d.routes[0].interface = "unknown".into(),
            1 => d.interfaces.get_mut("lan").unwrap().link = "../host".into(),
            2 => {
                d.interfaces
                    .get_mut("lan")
                    .unwrap()
                    .address
                    .as_mut()
                    .unwrap()
                    .prefix = 129
            }
            3 => d.routes[0].destination.address = ip("::1"),
            4 => d.interfaces.get_mut("lan").unwrap().link_local = ip("fe80:1::1"),
            _ => d.routes[0].distance = 0,
        }
        assert!(!t.valid());
        assert!(
            t.ping("workstation", ip("2001:db8:20::20"), None)
                .unwrap_err()
                .contains("Invalid IPv6")
        );
    }
    let mut t = topology();
    let seed = t.devices["server"].clone();
    for index in 0..13 {
        t.devices.insert(format!("extra{index}"), seed.clone());
    }
    assert!(!t.valid());
    let before = serde_json::to_value(topology()).unwrap();
    let decoded: Topology = serde_json::from_value(before.clone()).unwrap();
    assert_eq!(decoded, topology());
    let mut unknown = before;
    unknown["execute"] = "real command".into();
    assert!(serde_json::from_value::<Topology>(unknown).is_err());
}

#[test]
fn floating_route_recovers_after_interface_failure_and_reports_inactive_causes() {
    let mut t = topology();
    t.devices.get_mut("workstation").unwrap().interfaces.insert(
        "backup".into(),
        port("backup", "2001:db8:30::10", "fe80::10"),
    );
    t.devices
        .get_mut("router")
        .unwrap()
        .interfaces
        .insert("backup".into(), port("backup", "2001:db8:30::1", "fe80::1"));
    t.devices
        .get_mut("workstation")
        .unwrap()
        .routes
        .push(route("::/0", "backup", "fe80::1", 10));
    assert_eq!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap()
            .forward
            .hops[0]
            .interface,
        "lan"
    );
    t.devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .up = false;
    let ping = t.ping("workstation", ip("2001:db8:20::20"), None).unwrap();
    assert_eq!(ping.forward.hops[0].interface, "backup");
    assert_eq!(ping.source, ip("2001:db8:30::10"));
    assert_eq!(
        t.route_status("workstation", 0).unwrap(),
        ipv6::RouteStatus::InterfaceDown
    );
    t.devices.get_mut("workstation").unwrap().routes[1].via = Some(ip("2001:db8:99::1"));
    assert_eq!(
        t.route_status("workstation", 1).unwrap(),
        ipv6::RouteStatus::NextHopOffLink
    );
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("outside the explicitly on-link")
    );
    t.devices.get_mut("workstation").unwrap().routes[1].distance = 255;
    assert_eq!(
        t.route_status("workstation", 1).unwrap(),
        ipv6::RouteStatus::Distance255
    );
}
#[test]
fn wrong_prefix_missing_source_unknown_scope_and_unsupported_destinations_are_useful_errors() {
    let mut t = topology();
    t.devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .prefix = 32;
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("no up neighbor")
    );
    let mut t = topology();
    t.devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = None;
    assert!(
        t.ping("workstation", ip("2001:db8:20::20"), None)
            .unwrap_err()
            .contains("source address")
    );
    assert!(t.ping("workstation", ip("fe80::1"), Some("lan")).is_ok());
    assert!(
        t.ping("workstation", ip("fe80::1"), Some("missing"))
            .unwrap_err()
            .contains("Unknown outgoing")
    );
    assert!(
        t.ping("missing", ip("fe80::1"), Some("lan"))
            .unwrap_err()
            .contains("Unknown source")
    );
    for destination in [
        "::",
        "::1",
        "ff02::1",
        "::ffff:192.0.2.1",
        "fe90::1",
        "fc00::1",
        "2606:4700:4700::1111",
    ] {
        assert!(
            t.ping("workstation", ip(destination), Some("lan"))
                .unwrap_err()
                .contains("Unsupported probe"),
            "{destination}"
        );
    }
}
