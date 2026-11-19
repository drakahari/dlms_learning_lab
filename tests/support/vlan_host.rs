#![allow(dead_code)]
use dlms_learning_lab::{
    ipv4_routing::Prefix, linux_host as h, nm_policy as n, switching as s, vlan_host as v,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
pub fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
fn trunk(native: u16) -> s::Port {
    s::Port {
        up: true,
        mode: s::Mode::Trunk {
            native,
            allowed: BTreeSet::from([30, 40, 99]),
        },
    }
}
pub fn environment() -> n::Environment {
    let ports = BTreeMap::from([
        ("primary".into(), trunk(99)),
        ("native".into(), trunk(30)),
        (
            "reports".into(),
            s::Port {
                up: true,
                mode: s::Mode::Access { vlan: 30 },
            },
        ),
        (
            "audit".into(),
            s::Port {
                up: true,
                mode: s::Mode::Access { vlan: 40 },
            },
        ),
    ]);
    let names = ["primary", "native", "reports", "audit"];
    let fabric = s::Fabric {
        switches: BTreeMap::from([(
            "sw".into(),
            s::Switch {
                label: "Provided switch".into(),
                vlans: BTreeSet::from([30, 40, 99]),
                ports,
            },
        )]),
        hosts: names
            .iter()
            .map(|n| {
                (
                    (*n).into(),
                    s::Host {
                        label: (*n).into(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: names
            .iter()
            .map(|n| s::Cable {
                id: format!("link-{n}"),
                a: s::Jack::Host { host: (*n).into() },
                b: s::Jack::Switch {
                    switch: "sw".into(),
                    port: (*n).into(),
                },
                up: true,
            })
            .collect(),
    };
    n::Environment {
        bridge: None,
        bond: None,
        vlan: Some(v::Facts {
            version: 1,
            fabric,
            parents: vec![
                v::Parent {
                    device: "enp1s0".into(),
                    endpoint: "primary".into(),
                },
                v::Parent {
                    device: "enp2s0".into(),
                    endpoint: "native".into(),
                },
            ],
            interfaces: vec![
                v::InterfaceName("reports0".into()),
                v::InterfaceName("audit0".into()),
            ],
            peers: [
                ("10.20.0.1", "reports"),
                ("10.20.0.50", "reports"),
                ("172.16.5.10", "reports"),
                ("10.21.0.1", "audit"),
                ("10.21.0.50", "audit"),
            ]
            .into_iter()
            .map(|(address, endpoint)| v::Peer {
                address: ip(address),
                endpoint: endpoint.into(),
            })
            .collect(),
        }),
        physical: h::Profile {
            ipv6: None,
            id: "vlan-host-fixture".into(),
            links: vec![
                h::Link {
                    name: "reports0".into(),
                    network: prefix("10.20.0.0/24"),
                    carrier: true,
                },
                h::Link {
                    name: "audit0".into(),
                    network: prefix("10.21.0.0/24"),
                    carrier: true,
                },
            ],
            gateways: vec![
                h::Gateway {
                    address: ip("10.20.0.1"),
                    device: "reports0".into(),
                    available: true,
                    exports: vec![prefix("172.16.0.0/16")],
                },
                h::Gateway {
                    address: ip("10.21.0.1"),
                    device: "audit0".into(),
                    available: true,
                    exports: vec![prefix("172.16.0.0/16")],
                },
            ],
            nodes: vec![
                h::Node {
                    address: ip("10.20.0.50"),
                    available: true,
                },
                h::Node {
                    address: ip("10.21.0.50"),
                    available: true,
                },
                h::Node {
                    address: ip("172.16.5.10"),
                    available: true,
                },
            ],
            resolvers: vec![],
        },
        default_metrics: BTreeMap::from([("reports0".into(), 400), ("audit0".into(), 500)]),
        dhcp: BTreeMap::new(),
    }
}
pub fn files() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "reports.nmconnection".into(),
            text(
                1,
                "reports0",
                "enp1s0",
                30,
                "10.20.0.2/24",
                "10.20.0.1",
                false,
            ),
        ),
        (
            "audit.nmconnection".into(),
            text(2, "audit0", "enp2s0", 40, "10.21.0.2/24", "10.21.0.1", true),
        ),
    ])
}
pub fn text(
    uuid: u8,
    iface: &str,
    parent: &str,
    id: u16,
    address: &str,
    gateway: &str,
    no_default: bool,
) -> String {
    format!(
        "[connection]\nid={iface}\nuuid=00000000-0000-4000-8000-{uuid:012}\ntype=vlan\ninterface-name={iface}\nautoconnect=true\n[vlan]\nparent={parent}\nid={id}\nflags=1\nprotocol=802.1Q\n[ipv4]\nmethod=manual\naddress1={address}\ngateway={gateway}\nnever-default={no_default}\n[ipv6]\nmethod=disabled\n"
    )
}
pub fn choose(env: &n::Environment, files: &BTreeMap<String, String>) -> n::Choices {
    let p = n::Policy::from_texts_for(env, files).unwrap();
    n::Choices::empty().automatic(env, &p).unwrap().0
}
