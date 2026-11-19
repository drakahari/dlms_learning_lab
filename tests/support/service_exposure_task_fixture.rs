//! Fixed actual-model task fixture, not an authored learner lab.
use crate::server_fixture;
use dlms_learning_lab::{
    linux_filter::{Chain, Cidr, Protocol, Rule, Verdict},
    linux_router_workspace::Policy,
    processes::Requirement,
    service_exposure::{PortState, World},
    service_exposure_task::{ProbeCase, Task},
};
fn predicate(net: &str, port: Option<u16>, output: bool) -> Rule {
    let server = Some(Cidr::parse("203.0.113.10/32").unwrap());
    let client = Some(Cidr::parse(net).unwrap());
    Rule {
        source: if output { server } else { client },
        destination: if output { client } else { server },
        ingress: (!output).then(|| "eth0".into()),
        egress: output.then(|| "eth0".into()),
        protocol: Some(Protocol::Tcp),
        source_port: None,
        destination_port: port,
        target: Verdict::Accept,
    }
}
pub fn fixture() -> Task {
    let mut domains = vec![];
    let mut allowed = vec![];
    for net in ["198.51.100.0/24", "192.0.2.0/24"] {
        for port in [22, 443, 80, 9000] {
            let rule = predicate(net, Some(port), false);
            domains.push(rule.clone());
            if net == "198.51.100.0/24" || matches!(port, 443 | 80) {
                allowed.push(rule);
            }
        }
    }
    let responses = ["198.51.100.0/24", "192.0.2.0/24"]
        .iter()
        .map(|n| predicate(n, None, true))
        .collect::<Vec<_>>();
    let initial = server_fixture::fixture();
    let requirements = [10, 101, 102, 201, 202]
        .iter()
        .map(|pid| Requirement::Unchanged {
            pid: *pid,
            expected: initial.processes.observed_process(*pid).unwrap(),
        })
        .collect();
    let cases = [
        ("admin-ssh", 0, 22, PortState::Open),
        ("public-ssh", 1, 22, PortState::Filtered),
        ("public-web", 1, 443, PortState::Open),
        ("admin-preview", 0, 9000, PortState::Open),
        ("public-preview", 1, 9000, PortState::Filtered),
        ("absent-service", 1, 80, PortState::Closed),
    ]
    .into_iter()
    .map(|(id, viewpoint, port, state)| ProbeCase {
        id: id.into(),
        label: id.replace('-', " "),
        viewpoint,
        port,
        accepted: vec![state],
    })
    .collect();
    let task = Task {
        version: 1,
        initial,
        policies: vec![
            Policy {
                chain: Chain::Input,
                domains,
                allowed,
            },
            Policy {
                chain: Chain::Output,
                domains: responses.clone(),
                allowed: responses,
            },
        ],
        requirements,
        cases,
    };
    task.validate().unwrap();
    task
}
pub fn solve(t: &Task) -> World {
    let mut w = t.initial.clone();
    for port in [22, 9000] {
        w.admin_command(&format!(
            "iptables -A INPUT -s 192.0.2.0/24 -p tcp --dport {port} -j DROP"
        ))
        .unwrap();
    }
    assert!(t.checks(&w).iter().all(|c| c.passed));
    w
}
