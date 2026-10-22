use dlms_learning_lab::ipv6::{Assignment, Device, Interface, Prefix, Route, Topology};
use std::{collections::BTreeMap, net::Ipv6Addr};
pub fn ip(text: &str) -> Ipv6Addr {
    text.parse().unwrap()
}
pub fn port(link: &str, address: &str, local: &str) -> Interface {
    Interface {
        link: link.into(),
        up: true,
        link_local: ip(local),
        address: Some(Assignment {
            address: ip(address),
            prefix: 64,
            on_link: true,
        }),
    }
}
pub fn route(prefix: &str, interface: &str, via: &str, distance: u8) -> Route {
    Route {
        destination: Prefix::parse(prefix).unwrap(),
        interface: interface.into(),
        via: Some(ip(via)),
        distance,
    }
}
pub fn topology() -> Topology {
    Topology {
        devices: BTreeMap::from([
            (
                "workstation".into(),
                Device {
                    forwarding: false,
                    interfaces: BTreeMap::from([(
                        "lan".into(),
                        port("left", "2001:db8:10::10", "fe80::10"),
                    )]),
                    routes: vec![route("::/0", "lan", "fe80::1", 1)],
                },
            ),
            (
                "router".into(),
                Device {
                    forwarding: true,
                    interfaces: BTreeMap::from([
                        ("left".into(), port("left", "2001:db8:10::1", "fe80::1")),
                        ("right".into(), port("right", "2001:db8:20::1", "fe80::1")),
                    ]),
                    routes: vec![],
                },
            ),
            (
                "server".into(),
                Device {
                    forwarding: false,
                    interfaces: BTreeMap::from([(
                        "lan".into(),
                        port("right", "2001:db8:20::20", "fe80::20"),
                    )]),
                    routes: vec![route("::/0", "lan", "fe80::1", 1)],
                },
            ),
        ]),
    }
}
