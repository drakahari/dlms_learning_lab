use crate::fixture::{peer, supplied};
use dlms_learning_lab::{vpn_point::*, vpn_point_task::*};
pub fn network(s: &str) -> Network {
    let (a, p) = s.split_once('/').unwrap();
    Network {
        address: a.parse().unwrap(),
        prefix: p.parse().unwrap(),
    }
}
pub fn task() -> Task {
    let mut supplied = supplied();
    let mut alias = supplied.files[0].clone();
    alias.path = "/etc/openvpn/alias-ca.crt".into();
    supplied.files.push(alias);
    let initial = World::new(
        supplied,
        [network("10.20.0.0/24"), network("10.40.0.0/24")],
        [
            peer(true),
            peer(false).replace("203.0.113.10 1194", "203.0.113.10 1195"),
        ],
    )
    .unwrap();
    let requirements = [
        Requirement {
            role: Role::Server,
            subject: "hq".into(),
            authority: "classroom-ca".into(),
            destinations: vec![network("10.40.0.0/24")],
        },
        Requirement {
            role: Role::Client,
            subject: "branch".into(),
            authority: "classroom-ca".into(),
            destinations: vec![network("10.20.0.0/24")],
        },
    ];
    let mut cases = vec![];
    for (i, source, destination, expected) in [
        (0, "10.20.0.21", "10.40.0.41", Outcome::Reachable),
        (0, "10.20.0.221", "10.40.0.241", Outcome::Reachable),
        (1, "10.40.0.42", "10.20.0.22", Outcome::Reachable),
        (1, "10.40.0.242", "10.20.0.222", Outcome::Reachable),
        (0, "10.20.0.21", "198.51.100.4", Outcome::OutsideTunnel),
        (1, "10.40.0.42", "198.51.100.5", Outcome::OutsideTunnel),
    ] {
        cases.push(Case {
            id: format!("case-{}", cases.len()),
            label: format!("LAN {i}: {source} to {destination}"),
            from: i,
            source: source.parse().unwrap(),
            destination: destination.parse().unwrap(),
            expected,
        });
    }
    Task {
        version: 1,
        initial,
        requirements,
        cases,
    }
}
pub fn repaired(t: &Task) -> World {
    let mut w = t.initial.clone();
    w.save(1, &peer(false)).unwrap();
    w.apply().unwrap();
    assert!(t.checks(&w).iter().all(|c| c.passed), "{:?}", t.checks(&w));
    w
}
