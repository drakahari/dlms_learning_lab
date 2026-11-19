#![allow(dead_code)]
#[path = "bridge_control.rs"]
mod controls;
#[path = "bridge_fabric.rs"]
mod forwarding;
use dlms_learning_lab::{
    bridge_control as control, bridge_host as bridge,
    ipv4_routing::{Assignment, Prefix},
    linux_host as host, nm_profiles as ip,
};
use std::collections::BTreeMap;
pub fn addr(s: &str) -> std::net::Ipv4Addr {
    s.parse().unwrap()
}
pub fn facts() -> bridge::Facts {
    bridge::Facts {
        version: 1,
        profiles: controls::source(),
        lan: forwarding::facts(),
        peers: vec![
            bridge::Peer {
                address: addr("10.220.0.1"),
                endpoint: "peer-a".into(),
            },
            bridge::Peer {
                address: addr("10.220.0.50"),
                endpoint: "peer-a2".into(),
            },
            bridge::Peer {
                address: addr("10.220.0.60"),
                endpoint: "peer-b".into(),
            },
        ],
    }
}
pub fn physical() -> host::Profile {
    host::Profile {
        ipv6: None,
        id: "bridge-inputs".into(),
        links: vec![host::Link {
            name: "br0".into(),
            network: Prefix::parse("10.220.0.0/24").unwrap(),
            carrier: true,
        }],
        gateways: vec![host::Gateway {
            address: addr("10.220.0.1"),
            device: "br0".into(),
            available: true,
            exports: vec![Prefix::parse("10.230.0.0/24").unwrap()],
        }],
        nodes: vec![
            host::Node {
                address: addr("10.220.0.50"),
                available: true,
            },
            host::Node {
                address: addr("10.220.0.60"),
                available: true,
            },
            host::Node {
                address: addr("10.230.0.50"),
                available: true,
            },
        ],
        resolvers: vec![],
    }
}
pub fn files() -> BTreeMap<String, String> {
    controls::files()
}
pub fn snapshot(files: &BTreeMap<String, String>) -> control::Snapshot {
    control::Snapshot::automatic(control::Plan::from_files(&controls::source(), files).unwrap())
        .unwrap()
}
pub fn selected() -> BTreeMap<String, ip::Profile> {
    BTreeMap::from([(
        "br0".into(),
        ip::Profile {
            vlan: None,
            id: "Data bridge".into(),
            uuid: controls::source().controller.uuid,
            interface: "br0".into(),
            autoconnect: true,
            priority: 0,
            address: Assignment {
                address: addr("10.220.0.20"),
                prefix: 24,
            },
            gateway: None,
            route_metric: None,
            never_default: false,
            routes: vec![],
        },
    )])
}
pub fn host_state(profiles: &BTreeMap<String, ip::Profile>) -> host::State {
    host::State::new(
        physical(),
        host::Configuration {
            ipv6: None,
            interfaces: vec![host::Interface {
                name: "br0".into(),
                up: profiles.contains_key("br0"),
                address: profiles.get("br0").map(|p| p.address.clone()),
            }],
            routes: vec![],
            nameservers: vec![],
        },
    )
    .unwrap()
}
