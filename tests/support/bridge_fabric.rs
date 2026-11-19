#![allow(dead_code)]
use dlms_learning_lab::{bridge_fabric as b, switching as s};
use std::collections::{BTreeMap, BTreeSet};
pub fn facts() -> b::Facts {
    let mut switches = BTreeMap::new();
    let mut hosts = BTreeMap::new();
    let mut cables = Vec::new();
    for (sw, nic, peers, vlan) in [
        ("left", "nic-a", vec!["peer-a", "peer-a2"], 30),
        ("right", "nic-b", vec!["peer-b"], 40),
    ] {
        let mut ports = BTreeMap::new();
        for endpoint in std::iter::once(nic).chain(peers) {
            hosts.insert(
                endpoint.into(),
                s::Host {
                    label: endpoint.into(),
                    up: true,
                },
            );
            ports.insert(
                endpoint.into(),
                s::Port {
                    up: true,
                    mode: s::Mode::Access { vlan },
                },
            );
            cables.push(s::Cable {
                id: format!("wire-{endpoint}"),
                a: s::Jack::Host {
                    host: endpoint.into(),
                },
                b: s::Jack::Switch {
                    switch: sw.into(),
                    port: endpoint.into(),
                },
                up: true,
            });
        }
        switches.insert(
            sw.into(),
            s::Switch {
                label: sw.into(),
                vlans: BTreeSet::from([vlan]),
                ports,
            },
        );
    }
    b::Facts {
        version: 1,
        interface: b::Interface("br0".into()),
        fabric: s::Fabric {
            switches,
            hosts,
            cables,
        },
        ports: vec![
            b::Attachment {
                interface: b::Interface("enp1s0".into()),
                endpoint: "nic-a".into(),
            },
            b::Attachment {
                interface: b::Interface("enp2s0".into()),
                endpoint: "nic-b".into(),
            },
        ],
    }
}
pub fn current() -> b::Current {
    b::Current {
        up: true,
        members: vec![b::Interface("enp1s0".into()), b::Interface("enp2s0".into())],
    }
}
