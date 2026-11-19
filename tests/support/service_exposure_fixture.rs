//! Source-owned deterministic virtual server; no host operations.
use dlms_learning_lab::{linux_filter as filter, processes as p, service_exposure as audit};
use std::{collections::BTreeMap, net::Ipv4Addr};
pub fn fixture() -> audit::World {
    let root = p::Identity {
        name: "root".into(),
        uid: 0,
    };
    let mut processes = BTreeMap::new();
    for (id, comm) in [
        (1, "init"),
        (10, "controller"),
        (101, "previewd"),
        (102, "previewd"),
        (201, "sshd"),
        (202, "webd"),
    ] {
        processes.insert(
            id,
            p::Process {
                parent: if id == 1 { 0 } else { 1 },
                owner: root.clone(),
                session: if id == 1 { 1 } else { 10 },
                comm: comm.into(),
                nice: 0,
                phase: p::Phase::Active {
                    activity: p::Activity::Sleeping,
                },
            },
        );
    }
    let listeners = vec![
        (101, 9000, Ipv4Addr::UNSPECIFIED),
        (102, 9001, Ipv4Addr::LOCALHOST),
        (201, 22, Ipv4Addr::UNSPECIFIED),
        (202, 443, "203.0.113.10".parse().unwrap()),
    ]
    .into_iter()
    .map(|(pid, port, address)| p::Listener {
        protocol: p::Protocol::Tcp,
        address,
        port,
        pid,
        fd: 3,
        backlog: 128,
    })
    .collect();
    let w = audit::World {
        server: "203.0.113.10".parse().unwrap(),
        device: "eth0".into(),
        viewpoints: vec![
            audit::Viewpoint {
                id: "admin".into(),
                address: "198.51.100.20".parse().unwrap(),
            },
            audit::Viewpoint {
                id: "public".into(),
                address: "192.0.2.30".parse().unwrap(),
            },
        ],
        processes: p::Table {
            user: root,
            session: 10,
            processes,
            listeners,
        },
        filter: filter::Table::new(filter::Verdict::Accept),
    };
    w.validate().unwrap();
    w
}
