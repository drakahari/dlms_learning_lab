use dlms_learning_lab::{
    ipv4_routing::{Assignment, Prefix},
    linux_host as h, nm_policy as n, nm_workspace as w,
};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
pub fn environment() -> n::Environment {
    n::Environment {
        bridge: None,
        bond: None,
        vlan: None,
        physical: h::Profile {
            ipv6: None,
            id: "nm-fixture".into(),
            links: vec![
                h::Link {
                    name: "enp1s0".into(),
                    network: prefix("10.20.0.0/24"),
                    carrier: true,
                },
                h::Link {
                    name: "enp2s0".into(),
                    network: prefix("10.21.0.0/24"),
                    carrier: true,
                },
            ],
            gateways: vec![
                h::Gateway {
                    address: ip("10.20.0.1"),
                    device: "enp1s0".into(),
                    available: true,
                    exports: vec![prefix("172.16.0.0/16"), prefix("10.20.0.0/24")],
                },
                h::Gateway {
                    address: ip("10.21.0.1"),
                    device: "enp2s0".into(),
                    available: true,
                    exports: vec![prefix("172.16.0.0/16"), prefix("10.40.0.0/24")],
                },
            ],
            nodes: vec![
                h::Node {
                    address: ip("172.16.5.10"),
                    available: true,
                },
                h::Node {
                    address: ip("10.20.0.50"),
                    available: true,
                },
                h::Node {
                    address: ip("10.40.0.50"),
                    available: true,
                },
            ],
            resolvers: vec![],
        },
        default_metrics: BTreeMap::from([("enp1s0".into(), 100), ("enp2s0".into(), 200)]),
        dhcp: BTreeMap::new(),
    }
}
fn text(
    id: &str,
    uuid: u8,
    device: &str,
    address: &str,
    gateway: &str,
    priority: i16,
    metric: &str,
) -> String {
    format!(
        "[connection]\nid={id}\nuuid=00000000-0000-4000-8000-{uuid:012}\ntype=ethernet\ninterface-name={device}\nautoconnect-priority={priority}\n[ipv4]\nmethod=manual\naddress1={address}\ngateway={gateway}\nroute-metric={metric}\n[ipv6]\nmethod=disabled\n"
    )
}
pub fn files() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.nmconnection".into(),
            text(
                "Application",
                1,
                "enp1s0",
                "10.20.0.20/24",
                "10.20.0.1",
                10,
                "100",
            ),
        ),
        (
            "management.nmconnection".into(),
            text(
                "Management",
                2,
                "enp2s0",
                "10.21.0.20/24",
                "10.21.0.1",
                10,
                "200",
            ),
        ),
    ])
}

pub fn task(id: &str, base: u8) -> w::Task {
    let mut env = environment();
    let change = |s: &str| {
        s.replace("10.20.", &format!("10.{base}."))
            .replace("10.21.", &format!("10.{}.", base + 1))
    };
    env = serde_json::from_str(&change(&serde_json::to_string(&env).unwrap())).unwrap();
    let current: w::Files = files()
        .into_iter()
        .map(|(name, text)| {
            (
                name,
                change(&text)
                    .replace("000000000001", &format!("{base:012}"))
                    .replace("000000000002", &format!("{:012}", base + 1)),
            )
        })
        .collect();
    let mut starter = current.clone();
    let app = starter.get_mut("app.nmconnection").unwrap();
    *app = app.replace(&format!("10.{base}.0.20/24"), &format!("10.{base}.0.99/24"));
    let requirement = vec![
        w::Requirement::CurrentUnchanged {
            interface: "enp2s0".into(),
        },
        w::Requirement::FutureAddress {
            interface: "enp1s0".into(),
            address: Assignment {
                address: ip(&format!("10.{base}.0.20")),
                prefix: 24,
            },
        },
        w::Requirement::FuturePath {
            target: ip("172.16.5.10"),
            interface: "enp1s0".into(),
        },
        w::Requirement::DefaultOwner {
            interface: "enp1s0".into(),
        },
    ];
    let mut second = env.clone();
    second.default_metrics.insert("enp1s0".into(), 600);
    second.default_metrics.insert("enp2s0".into(), 100);
    w::Task{id:id.into(),instructions:"Repair the supplied application address and next-startup path, preserving current management settings and supplied filenames/UUIDs.".into(),starter,protected_profiles:vec![],cases:vec![
        w::Case{current_bridge:None,current_controls:None,id:"first".into(),label:"First supplied automatic metrics".into(),environment:env,current_files:current.clone(),initially_selected:vec![format!("00000000-0000-4000-8000-{:012}",base+1)],requirements:requirement.clone()},
        w::Case{current_bridge:None,current_controls:None,id:"changed-defaults".into(),label:"Different supplied automatic metrics".into(),environment:second,current_files:current,initially_selected:vec![format!("00000000-0000-4000-8000-{:012}",base+1)],requirements:requirement},
    ]}
}
pub fn solved(t: &w::Task) -> w::Files {
    t.cases[0].current_files.clone()
}
pub fn definition() -> w::Definition {
    let example = task("network-practice", 30);
    w::Definition {
        initial: task("network-assignment", 20),
        example_worked: solved(&example),
        example,
    }
}
