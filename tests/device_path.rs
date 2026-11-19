use dlms_learning_lab::device_path as projection;
use dlms_learning_lab::{ipv4_routing, switching};
use projection::{Attachment, Facts};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};

fn assignment(address: &str) -> ipv4_routing::Assignment {
    ipv4_routing::Assignment {
        address: address.parse().unwrap(),
        prefix: 24,
    }
}

fn interface(link: &str, address: &str) -> ipv4_routing::Interface {
    ipv4_routing::Interface {
        link: link.into(),
        up: true,
        address: Some(assignment(address)),
    }
}

fn host(link: &str, address: &str, gateway: &str) -> ipv4_routing::Device {
    ipv4_routing::Device {
        forwarding: false,
        interfaces: BTreeMap::from([("eth0".into(), interface(link, address))]),
        routes: vec![ipv4_routing::Route {
            destination: ipv4_routing::Prefix::new(Ipv4Addr::UNSPECIFIED, 0).unwrap(),
            interface: "eth0".into(),
            via: gateway.parse().unwrap(),
            distance: 1,
        }],
    }
}

fn cable(id: &str, endpoint: &str, switch: &str, port: &str) -> switching::Cable {
    switching::Cable {
        id: id.into(),
        a: switching::Jack::Host {
            host: endpoint.into(),
        },
        b: switching::Jack::Switch {
            switch: switch.into(),
            port: port.into(),
        },
        up: true,
    }
}

fn fixture() -> Facts {
    let switch = |label: &str| switching::Switch {
        label: label.into(),
        vlans: BTreeSet::from([1]),
        ports: (1..=4)
            .map(|index| {
                (
                    format!("p{index}"),
                    switching::Port {
                        up: true,
                        mode: switching::Mode::Access { vlan: 1 },
                    },
                )
            })
            .collect(),
    };
    let names = [
        ("nic-client", "client", "eth0"),
        ("nic-monitor", "monitor", "eth0"),
        ("nic-router-office", "edge", "office"),
        ("nic-router-server", "edge", "servers"),
        ("nic-server", "server", "eth0"),
    ];
    let fabric = switching::Fabric {
        switches: BTreeMap::from([
            ("office".into(), switch("Office switch")),
            ("servers".into(), switch("Server switch")),
        ]),
        hosts: names
            .iter()
            .map(|(endpoint, _, _)| {
                (
                    endpoint.to_string(),
                    switching::Host {
                        label: endpoint.to_string(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: vec![
            cable("c-client", "nic-client", "office", "p1"),
            cable("c-monitor", "nic-monitor", "office", "p2"),
            cable("c-router-office", "nic-router-office", "office", "p3"),
            cable("c-server", "nic-server", "servers", "p1"),
            cable("c-router-server", "nic-router-server", "servers", "p2"),
        ],
    };
    let network = ipv4_routing::Topology {
        devices: BTreeMap::from([
            ("client".into(), host("office", "192.0.2.10", "192.0.2.1")),
            ("monitor".into(), host("office", "192.0.2.20", "192.0.2.1")),
            (
                "server".into(),
                host("servers", "198.51.100.10", "198.51.100.1"),
            ),
            (
                "edge".into(),
                ipv4_routing::Device {
                    forwarding: true,
                    interfaces: BTreeMap::from([
                        ("office".into(), interface("office", "192.0.2.1")),
                        ("servers".into(), interface("servers", "198.51.100.1")),
                    ]),
                    routes: Vec::new(),
                },
            ),
        ]),
    };
    Facts {
        fabric,
        network,
        attachments: names
            .iter()
            .map(|(endpoint, device, interface)| Attachment {
                endpoint: endpoint.to_string(),
                device: device.to_string(),
                interface: interface.to_string(),
            })
            .collect(),
    }
}

#[test]
fn switches_join_local_hosts_and_the_router_carries_a_real_round_trip() {
    let facts = fixture();
    let (fabric, network) = facts.project(&facts.fabric.cables).unwrap();
    assert!(
        fabric
            .visibility("nic-client", "nic-monitor")
            .unwrap()
            .mutual()
    );
    assert!(
        fabric
            .visibility("nic-client", "nic-server")
            .unwrap()
            .isolated()
    );
    assert_ne!(
        network.devices["edge"].interfaces["office"].link,
        network.devices["edge"].interfaces["servers"].link
    );
    let remote = network
        .probe("client", "198.51.100.10".parse().unwrap())
        .unwrap();
    assert_eq!(remote.forward.destination_device, "server");
    assert_eq!((remote.forward.hops.len(), remote.reply.hops.len()), (2, 2));
    assert!(remote.forward.hops.iter().any(|hop| hop.device == "edge"));
    let local = network
        .probe("client", "192.0.2.20".parse().unwrap())
        .unwrap();
    assert_eq!(local.forward.hops.len(), 1);
    assert!(local.forward.hops.iter().all(|hop| hop.device != "edge"));
}

#[test]
fn wrong_patch_endpoints_break_the_gateway_and_equivalent_ports_work() {
    let facts = fixture();
    let mut wrong = facts.fabric.cables.clone();
    let first = wrong[0].b.clone();
    wrong[0].b = wrong[3].b.clone();
    wrong[3].b = first;
    let (_, network) = facts.project(&wrong).unwrap();
    assert!(
        network
            .probe("client", "198.51.100.10".parse().unwrap())
            .is_err()
    );
    assert!(
        network
            .probe("monitor", "192.0.2.1".parse().unwrap())
            .is_ok()
    );
    let mut alternate = facts.fabric.cables.clone();
    alternate[0].b = switching::Jack::Switch {
        switch: "office".into(),
        port: "p4".into(),
    };
    let (_, network) = facts.project(&alternate).unwrap();
    assert!(
        network
            .probe("client", "198.51.100.10".parse().unwrap())
            .is_ok()
    );
}

#[test]
fn a_shared_ethernet_domain_can_have_working_ip_and_still_leak_broadcasts() {
    let mut facts = fixture();
    facts.fabric.cables.push(switching::Cable {
        id: "c-bridge".into(),
        a: switching::Jack::Switch {
            switch: "office".into(),
            port: "p4".into(),
        },
        b: switching::Jack::Switch {
            switch: "servers".into(),
            port: "p3".into(),
        },
        up: true,
    });
    let (fabric, network) = facts.project(&facts.fabric.cables).unwrap();
    assert!(
        fabric
            .visibility("nic-client", "nic-server")
            .unwrap()
            .mutual()
    );
    assert!(
        network
            .probe("client", "198.51.100.10".parse().unwrap())
            .is_ok()
    );
    let mut separated = facts.fabric.cables.clone();
    separated.last_mut().unwrap().up = false;
    let (fabric, network) = facts.project(&separated).unwrap();
    assert!(
        fabric
            .visibility("nic-client", "nic-server")
            .unwrap()
            .isolated()
    );
    assert!(
        network
            .probe("client", "198.51.100.10".parse().unwrap())
            .is_ok()
    );
    for cable in &mut separated {
        cable.up = false;
    }
    let (fabric, network) = facts.project(&separated).unwrap();
    assert!(
        fabric
            .visibility("nic-client", "nic-server")
            .unwrap()
            .isolated()
    );
    assert!(
        network
            .probe("client", "198.51.100.10".parse().unwrap())
            .is_err()
    );
}

#[test]
fn bad_port_bindings_and_duplicate_jacks_do_not_replace_supplied_equipment() {
    let facts = fixture();
    let original = facts.clone();
    let mut duplicate = facts.fabric.cables.clone();
    duplicate[0].b = duplicate[1].b.clone();
    assert!(facts.project(&duplicate).is_err());
    assert_eq!(facts, original);
    let mut wrong = facts.clone();
    wrong.attachments.pop();
    assert!(wrong.validate().is_err());
    let mut wrong = facts.clone();
    wrong.attachments[1] = wrong.attachments[0].clone();
    assert!(wrong.validate().is_err());
}
