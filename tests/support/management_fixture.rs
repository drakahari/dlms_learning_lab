use dlms_learning_lab::{
    acl,
    ipv4_routing::{Assignment, Device, Interface, Prefix, Route, Topology},
    management_path as path, management_policy as policy,
};
use std::{collections::BTreeMap, net::Ipv4Addr};

pub fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
pub fn port(link: &str, address: &str, prefix: u8) -> Interface {
    Interface {
        link: link.into(),
        up: true,
        address: Some(Assignment {
            address: ip(address),
            prefix,
        }),
    }
}
pub fn route(prefix: &str, interface: &str, via: &str) -> Route {
    Route {
        destination: Prefix::parse(prefix).unwrap(),
        interface: interface.into(),
        via: ip(via),
        distance: 1,
    }
}
pub fn device(forwarding: bool, ports: Vec<(&str, Interface)>, routes: Vec<Route>) -> Device {
    Device {
        forwarding,
        interfaces: ports.into_iter().map(|(n, p)| (n.into(), p)).collect(),
        routes,
    }
}
pub fn binding(device: &str, interface: &str) -> path::Binding {
    path::Binding {
        device: device.into(),
        interface: interface.into(),
    }
}
pub fn network() -> path::Network {
    path::Network {
        version: 1,
        management: binding("R2", "transit"),
        clients: vec![
            path::Client {
                id: "admin".into(),
                binding: binding("Client", "lan"),
            },
            path::Client {
                id: "guest".into(),
                binding: binding("Guest", "lan"),
            },
        ],
        topology: Topology {
            devices: BTreeMap::from([
                (
                    "Client".into(),
                    device(
                        false,
                        vec![("lan", port("office", "192.0.2.10", 24))],
                        vec![route("0.0.0.0/0", "lan", "192.0.2.1")],
                    ),
                ),
                (
                    "Guest".into(),
                    device(
                        false,
                        vec![("lan", port("office", "192.0.2.130", 24))],
                        vec![route("0.0.0.0/0", "lan", "192.0.2.1")],
                    ),
                ),
                (
                    "R1".into(),
                    device(
                        true,
                        vec![
                            ("office", port("office", "192.0.2.1", 24)),
                            ("transit", port("backbone", "10.0.0.1", 30)),
                        ],
                        vec![],
                    ),
                ),
                (
                    "R2".into(),
                    device(
                        true,
                        vec![
                            ("transit", port("backbone", "10.0.0.2", 30)),
                            ("branch", port("branch", "198.51.100.1", 24)),
                        ],
                        vec![route("192.0.2.0/24", "transit", "10.0.0.1")],
                    ),
                ),
            ]),
        },
    }
}
pub fn settings() -> policy::Policy {
    policy::Policy {
        version: 1,
        host_identity: Some("supplied-lab-identity".into()),
        versions: policy::Versions::Version2Only,
        lines: vec![policy::Line {
            number: 0,
            ssh: true,
            telnet: false,
            source_list: Some(acl::AccessList {
                kind: acl::Kind::Standard,
                rules: vec![acl::Rule {
                    id: "approved-admin".into(),
                    action: acl::Action::Permit,
                    selector: acl::Selector::Standard {
                        source: acl::Wildcard::host(ip("192.0.2.10")),
                    },
                }],
            }),
        }],
    }
}
