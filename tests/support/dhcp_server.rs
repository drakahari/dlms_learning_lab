use dlms_learning_lab::{dhcp_config, dhcp_server_workspace as w, dhcpv4 as d};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
pub fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
pub fn config(n: u8) -> String {
    format!(
        "default-lease-time 600; max-lease-time 3600; option domain-name-servers 203.0.113.53;\nsubnet 10.{n}.0.0 netmask 255.255.255.0 {{ option routers 10.{n}.0.1; range 10.{n}.0.100 10.{n}.0.103; }}\nsubnet 10.{n}.1.0 netmask 255.255.255.0 {{ option routers 10.{n}.1.1; option domain-name-servers 203.0.113.54; range 10.{n}.1.50 10.{n}.1.53; }}"
    )
}
pub fn task(name: &str, n: u8) -> w::Task {
    let text = config(n);
    let cases = (0..2)
        .map(|busy| {
            let seg = |octet: u8| d::Segment {
                subnet: d::Subnet {
                    network: ip(&format!("10.{n}.{octet}.0")),
                    prefix: 24,
                },
                server_interface: Some(ip(&format!("10.{n}.{octet}.1"))),
                relay_interface: None,
                relay_to_server: false,
                client_unicast_to_server: true,
                occupied: BTreeSet::new(),
            };
            let clients = [("management", 0), ("new-a", 0), ("new-b", 0), ("remote", 1)];
            let mut topology = d::Topology {
                server: ip(&format!("10.{n}.0.1")),
                segments: BTreeMap::from([("alpha".into(), seg(0)), ("beta".into(), seg(1))]),
                clients: clients
                    .iter()
                    .map(|(role, octet)| {
                        (
                            format!("{name}-{role}"),
                            d::Client {
                                segment: if *octet == 0 { "alpha" } else { "beta" }.into(),
                            },
                        )
                    })
                    .collect(),
            };
            if busy == 1 {
                topology
                    .segments
                    .get_mut("alpha")
                    .unwrap()
                    .occupied
                    .extend([ip(&format!("10.{n}.0.100")), ip(&format!("10.{n}.0.101"))]);
            }
            let bootstrap = text
                .replace(
                    &format!("range 10.{n}.0.100 10.{n}.0.103;"),
                    &format!("range 10.{n}.0.10;"),
                )
                .replace("203.0.113.53", "203.0.113.10");
            let mut initial = d::State::new(
                topology.clone(),
                dhcp_config::compile(&bootstrap, &topology, &BTreeMap::new()).unwrap(),
            )
            .unwrap();
            let management = format!("{name}-management");
            let d::Outcome::Offered(offer) = initial.discover(&management).unwrap() else {
                panic!("offer")
            };
            assert!(matches!(
                initial.request(&management, offer.token).unwrap(),
                d::Outcome::Ack { .. }
            ));
            let mut requirements = vec![w::Requirement::PreserveBinding { client: management }];
            let actions = [("new-a", 0, 53), ("new-b", 0, 53), ("remote", 1, 54)]
                .into_iter()
                .map(|(role, octet, dns)| {
                    let client = format!("{name}-{role}");
                    requirements.push(w::Requirement::Parameters {
                        client: client.clone(),
                        subnet: topology.segments[if octet == 0 { "alpha" } else { "beta" }].subnet,
                        router: Some(ip(&format!("10.{n}.{octet}.1"))),
                        dns: vec![ip(&format!("203.0.113.{dns}"))],
                    });
                    requirements.push(w::Requirement::Duration {
                        client: client.clone(),
                        seconds: 600,
                    });
                    w::Operation::Acquire { client }
                })
                .collect();
            requirements.extend([
                w::Requirement::PoolWindow {
                    subnet: topology.segments["alpha"].subnet,
                    first: ip(&format!("10.{n}.0.100")),
                    last: ip(&format!("10.{n}.0.110")),
                },
                w::Requirement::PoolWindow {
                    subnet: topology.segments["beta"].subnet,
                    first: ip(&format!("10.{n}.1.50")),
                    last: ip(&format!("10.{n}.1.60")),
                },
                w::Requirement::PoolsOnly {
                    subnets: topology.segments.values().map(|s| s.subnet).collect(),
                },
            ]);
            w::Case {
                id: format!("{name}-case-{busy}"),
                label: if busy == 0 {
                    "Available addresses"
                } else {
                    "Two occupied addresses"
                }
                .into(),
                initial,
                actions,
                requirements,
            }
        })
        .collect();
    w::Task{id:name.into(),instructions:"Serve the supplied new clients with the required subnet-specific options and preserve the existing management binding.".into(),starter:text.replace("203.0.113.53","203.0.113.99"),cases}
}
pub fn definition() -> w::Definition {
    w::Definition {
        initial: task("office", 30),
        example: task("workshop", 40),
        example_worked: config(40),
    }
}
pub fn range_task(name: &str, n: u8) -> w::Task {
    let mut t = task(name, n);
    for c in &mut t.cases {
        let mut topology = c.initial.topology().clone();
        topology.segments.remove("beta");
        topology.clients.remove(&format!("{name}-remote"));
        let reference = config(n)
            .split("\nsubnet")
            .take(2)
            .collect::<Vec<_>>()
            .join("\nsubnet");
        let bootstrap = reference.replace(
            &format!("range 10.{n}.0.100 10.{n}.0.103;"),
            &format!("range 10.{n}.0.10;"),
        );
        let mut initial = d::State::new(
            topology.clone(),
            dhcp_config::compile(&bootstrap, &topology, &BTreeMap::new()).unwrap(),
        )
        .unwrap();
        let client = format!("{name}-management");
        let d::Outcome::Offered(o) = initial.discover(&client).unwrap() else {
            panic!()
        };
        assert!(matches!(
            initial.request(&client, o.token).unwrap(),
            d::Outcome::Ack { .. }
        ));
        c.initial = initial;
        c.actions
            .retain(|a| !matches!(a,w::Operation::Acquire{client}if client.ends_with("-remote")));
        c.requirements.retain(|r|!matches!(r,w::Requirement::Parameters{client,..}|w::Requirement::Duration{client,..}if client.ends_with("-remote"))&&!matches!(r,w::Requirement::PoolWindow{subnet,..}if subnet.network==ip(&format!("10.{n}.1.0"))));
        for r in &mut c.requirements {
            if let w::Requirement::PoolsOnly { subnets } = r {
                subnets.retain(|s| s.network == ip(&format!("10.{n}.0.0")));
            }
        }
    }
    t.starter = range_config(n).replace(&format!("10.{n}.0.103"), &format!("10.{n}.0.102"));
    t
}
pub fn range_config(n: u8) -> String {
    config(n)
        .split("\nsubnet")
        .take(2)
        .collect::<Vec<_>>()
        .join("\nsubnet")
}
pub fn range_definition() -> w::Definition {
    w::Definition {
        initial: range_task("lpic2-dhcp-dynamic-ranges", 50),
        example: range_task("workshop-dynamic-ranges", 60),
        example_worked: range_config(60),
    }
}
pub fn options_config(n: u8) -> String {
    config(n).replace(
        &format!("range 10.{n}.0.100 10.{n}.0.103;"),
        &format!("range 10.{n}.0.10 10.{n}.0.15;"),
    )
}
pub fn options_task(name: &str, n: u8) -> w::Task {
    let mut t = task(name, n);
    for c in &mut t.cases {
        let mut topology = c.initial.topology().clone();
        let alpha = topology.segments.get_mut("alpha").unwrap();
        alpha.occupied = alpha
            .occupied
            .iter()
            .map(|a| ip(&a.to_string().replace(".100", ".11").replace(".101", ".12")))
            .collect();
        topology.clients.insert(
            format!("{name}-untouched"),
            d::Client {
                segment: "beta".into(),
            },
        );
        let bootstrap = options_config(n)
            .replace(
                &format!("range 10.{n}.0.10 10.{n}.0.15;"),
                &format!("range 10.{n}.0.10;"),
            )
            .replace("203.0.113.53", "203.0.113.10");
        let mut initial = d::State::new(
            topology.clone(),
            dhcp_config::compile(&bootstrap, &topology, &BTreeMap::new()).unwrap(),
        )
        .unwrap();
        for role in ["management", "untouched"] {
            let client = format!("{name}-{role}");
            let d::Outcome::Offered(o) = initial.discover(&client).unwrap() else {
                panic!()
            };
            assert!(matches!(
                initial.request(&client, o.token).unwrap(),
                d::Outcome::Ack { .. }
            ));
        }
        c.initial = initial;
        c.actions.splice(
            0..0,
            [
                w::Operation::Advance { seconds: 300 },
                w::Operation::Renew {
                    client: format!("{name}-management"),
                },
            ],
        );
        c.requirements
            .retain(|r| !matches!(r, w::Requirement::PreserveBinding { .. }));
        c.requirements.extend([
            w::Requirement::Parameters {
                client: format!("{name}-management"),
                subnet: topology.segments["alpha"].subnet,
                router: Some(ip(&format!("10.{n}.0.1"))),
                dns: vec![ip("203.0.113.53")],
            },
            w::Requirement::Duration {
                client: format!("{name}-management"),
                seconds: 600,
            },
            w::Requirement::PreserveBinding {
                client: format!("{name}-untouched"),
            },
        ]);
        for r in &mut c.requirements {
            if let w::Requirement::PoolWindow {
                subnet,
                first,
                last,
            } = r
                && subnet.network == ip(&format!("10.{n}.0.0"))
            {
                *first = ip(&format!("10.{n}.0.10"));
                *last = ip(&format!("10.{n}.0.20"));
            }
        }
    }
    t.starter = options_config(n)
        .replace("203.0.113.53", "203.0.113.99")
        .replace("203.0.113.54", "203.0.113.53");
    t
}
pub fn options_definition() -> w::Definition {
    w::Definition {
        initial: options_task("lpic2-dhcp-option-scope", 51),
        example: options_task("workshop-option-scope", 61),
        example_worked: options_config(61),
    }
}
