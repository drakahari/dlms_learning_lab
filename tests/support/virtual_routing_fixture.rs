//! Focused model checks; catalog integration and native acceptance are separate.
use dlms_learning_lab::{ipv4_routing as ip, virtual_routing::*};
use std::collections::BTreeMap;
pub fn fixture() -> (Facts, Configuration) {
    let interface = |link: &str, address: &str| ip::Interface {
        link: link.into(),
        up: true,
        address: Some(ip::Assignment {
            address: address.parse().unwrap(),
            prefix: 24,
        }),
    };
    let mut devices = BTreeMap::new();
    let mut router_interfaces = BTreeMap::new();
    let mut bindings = Vec::new();
    for (tenant, context) in [("a", "blue"), ("b", "green")] {
        for (role, network, last) in [("client", 1, 10), ("server", 2, 20)] {
            let link = format!("{tenant}-{role}");
            let port = format!("{tenant}-{role}");
            let gateway = format!("10.0.{network}.1");
            router_interfaces.insert(port.clone(), interface(&link, &gateway));
            bindings.push(Binding {
                interface: port,
                context: context.into(),
            });
            devices.insert(
                format!("{tenant}-{role}"),
                ip::Device {
                    forwarding: false,
                    interfaces: BTreeMap::from([(
                        "nic".into(),
                        interface(&link, &format!("10.0.{network}.{last}")),
                    )]),
                    routes: vec![ip::Route {
                        destination: ip::Prefix::new("0.0.0.0".parse().unwrap(), 0).unwrap(),
                        interface: "nic".into(),
                        via: gateway.parse().unwrap(),
                        distance: 1,
                    }],
                },
            );
        }
    }
    devices.insert(
        "edge".into(),
        ip::Device {
            forwarding: true,
            interfaces: router_interfaces,
            routes: vec![],
        },
    );
    (
        Facts {
            router: "edge".into(),
            contexts: vec![
                Context { id: "blue".into() },
                Context { id: "green".into() },
            ],
            network: ip::Topology { devices },
        },
        Configuration {
            bindings,
            router_up: true,
        },
    )
}
