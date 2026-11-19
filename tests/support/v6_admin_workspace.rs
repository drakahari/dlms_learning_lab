//! Synthetic source-only tasks, never counted as authored catalog lessons.
use dlms_learning_lab::{
    host_workspace::Requirement,
    linux_v6_admin as core, linux_v6_admin_workspace as work,
    linux_v6_filter::{Cidr, Protocol},
    linux_v6_path::*,
};
#[path = "host_ipv6.rs"]
mod host_fixture;
pub fn task() -> work::Task {
    let mut host = host_fixture::fixture();
    for l in &mut host.profile.links {
        l.carrier = true;
    }
    for i in &mut host.configuration.interfaces {
        i.up = true;
    }
    for g in &mut host.profile.ipv6.as_mut().unwrap().gateways {
        g.available = true;
    }
    let initial = core::State::new(host).unwrap();
    let mut expected = initial.facts().unwrap();
    expected.routes.push(Route {
        destination: Cidr::parse("2001:db8:90::/64").unwrap(),
        device: "ens3".into(),
        via: Some("fe80::1".parse().unwrap()),
        metric: 10,
    });
    let network = |n: usize| initial.host.profile.ipv6.as_ref().unwrap().lans[n].network;
    let requirements = vec![
        Requirement::PreserveIpv4 {},
        Requirement::Ipv6Address {
            device: "ens3".into(),
            network: network(0),
        },
        Requirement::Ipv6Address {
            device: "ens4".into(),
            network: network(1),
        },
    ];
    work::Task {
        id: "configuration-assignment".into(),
        initial,
        requirements,
        expected_routes: expected,
        settings: vec![work::Setting {
            key: "net.ipv6.conf.all.forwarding".into(),
            value: 1,
            label: "IPv6 forwarding on".into(),
            why: "Enable IPv6 transit between the supplied interfaces.".into(),
        }],
        paths: vec![work::PathTarget {
            target: "2001:db8:90::80".parse().unwrap(),
            device: "ens3".into(),
        }],
        messages: vec![],
        frames: vec![
            work::Frame {
                id: "outgoing-transit".into(),
                label: "Client to supplied remote host".into(),
                ingress: "ens4".into(),
                header: Header {
                    source: "2001:db8:20::20".parse().unwrap(),
                    destination: "2001:db8:90::80".parse().unwrap(),
                    protocol: Protocol::Tcp,
                    source_port: Some(40000),
                    destination_port: Some(443),
                    icmp_type: None,
                    icmp_code: None,
                },
                expected: Gate::Allowed,
            },
            work::Frame {
                id: "link-local-scope".into(),
                label: "Link-local source stays on its link".into(),
                ingress: "ens4".into(),
                header: Header {
                    source: "fe80::20".parse().unwrap(),
                    destination: "2001:db8:90::80".parse().unwrap(),
                    protocol: Protocol::Tcp,
                    source_port: Some(40000),
                    destination_port: Some(443),
                    icmp_type: None,
                    icmp_code: None,
                },
                expected: Gate::ScopeBound,
            },
        ],
        active_learned: None,
        file: None,
    }
}
pub fn commands(example: bool) -> Vec<work::Action> {
    vec![
        work::Action::Command {
            input: format!(
                "ip -6 route add 2001:db8:{}::/64 via fe80::1 dev ens3 metric 10",
                if example { "190" } else { "90" }
            ),
        },
        work::Action::Command {
            input: "sysctl -w net.ipv6.conf.all.forwarding=1".into(),
        },
        work::Action::Check,
    ]
}
pub fn example(t: &work::Task) -> work::Task {
    let s = serde_json::to_string(t)
        .unwrap()
        .replace("2001:db8:10:", "2001:db8:110:")
        .replace("2001:db8:20:", "2001:db8:120:")
        .replace("2001:db8:90:", "2001:db8:190:");
    let mut t: work::Task = serde_json::from_str(&s).unwrap();
    t.id = "configuration-example".into();
    t.initial.host.profile.id = "configuration-example".into();
    t
}
pub fn definition() -> work::Definition {
    let assignment = task();
    let example = example(&assignment);
    work::Definition {
        source_id: "configuration-proof".into(),
        assignment,
        example,
        worked: commands(true),
    }
}
pub fn ra_definition() -> work::Definition {
    let mut a = task();
    a.expected_routes.routes = vec![Route {
        destination: Cidr::parse("::/0").unwrap(),
        device: "ens3".into(),
        via: Some("fe80::1".parse().unwrap()),
        metric: 1024,
    }];
    for (dev, value) in [("ens3", 2), ("ens4", 0)] {
        a.settings.push(work::Setting {
            key: format!("net.ipv6.conf.{dev}.accept_ra"),
            value,
            label: format!("Advertisement policy on {dev}"),
            why: "Accept supplied upstream messages while ignoring the client-facing link.".into(),
        });
    }
    a.messages = vec![
        work::Message {
            id: "upstream-message".into(),
            label: "Accept supplied upstream message".into(),
            advertisement: core::Advertisement {
                device: "ens3".into(),
                gateway: "fe80::1".parse().unwrap(),
                router_lifetime: 600,
            },
            accepted: true,
            learns_default: true,
        },
        work::Message {
            id: "client-link-message".into(),
            label: "Ignore client-facing message".into(),
            advertisement: core::Advertisement {
                device: "ens4".into(),
                gateway: "fe80::1".parse().unwrap(),
                router_lifetime: 600,
            },
            accepted: false,
            learns_default: false,
        },
    ];
    a.active_learned = Some("ens3".into());
    work::Definition {
        source_id: "configuration-ra-proof".into(),
        example: example(&a),
        assignment: a,
        worked: ra_commands(),
    }
}
pub fn ra_commands() -> Vec<work::Action> {
    vec![
        work::Action::Command {
            input: "sysctl -w net.ipv6.conf.all.forwarding=1".into(),
        },
        work::Action::Command {
            input: "sysctl -w net.ipv6.conf.ens3.accept_ra=2".into(),
        },
        work::Action::Command {
            input: "sysctl -w net.ipv6.conf.ens4.accept_ra=0".into(),
        },
        work::Action::Observe { message: 0 },
        work::Action::Check,
    ]
}
pub const GOOD_FILE: &str = "# Router role and explicit per-link message policy\nnet.ipv6.conf.all.forwarding = 1\nnet.ipv6.conf.ens3.accept_ra = 2\nnet.ipv6.conf.ens4.accept_ra = 0\n";
pub fn file_definition() -> work::Definition {
    let mut d = ra_definition();
    let initial = "net.ipv6.conf.all.forwarding = 0\nnet.ipv6.conf.ens3.accept_ra = 1\nnet.ipv6.conf.ens4.accept_ra = 1\n";
    d.assignment.file = Some(work::SettingsFile {
        initial_text: initial.into(),
    });
    d.example.file = d.assignment.file.clone();
    d.source_id = "configuration-file-proof".into();
    d.worked = vec![
        work::Action::EditFile {
            text: GOOD_FILE.into(),
        },
        work::Action::SaveFile,
        work::Action::Command {
            input: format!(
                "sysctl -p {}",
                dlms_learning_lab::linux_v6_saved_settings::FILE
            ),
        },
        work::Action::Observe { message: 0 },
        work::Action::Check,
    ];
    d
}
