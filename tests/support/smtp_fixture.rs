//! Synthetic mail world; not authored curriculum or a protocol reference run.
use dlms_learning_lab::smtp_routing::{Facts, Node};
use std::{collections::BTreeMap, net::Ipv4Addr};
pub fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn config(host: &str, dest: &str, trust: &str, relay: &str, allowed: &str) -> String {
    format!(
        "myhostname = {host}\nmydestination = {dest}\nmynetworks = {trust}\nrelay_domains = {allowed}\nrelayhost = {relay}\nsmtpd_relay_restrictions = permit_mynetworks, reject_unauth_destination\nparent_domain_matches_subdomains =\n"
    )
}
pub fn fixture() -> (Facts, BTreeMap<String, String>) {
    let nodes = [
        ("gate", "192.0.2.10", vec!["operator"]),
        ("relay", "192.0.2.20", vec!["postmaster"]),
        ("delivery", "192.0.2.30", vec!["nora", "omar"]),
        ("outside", "192.0.2.40", vec!["visitor"]),
    ]
    .into_iter()
    .map(|(id, address, users)| {
        (
            id.into(),
            Node {
                hostname: format!("{id}.lab.test"),
                address: ip(address),
                port: 25,
                available: true,
                users: users.into_iter().map(String::from).collect(),
                aliases: String::new(),
                forwards: BTreeMap::new(),
            },
        )
    })
    .collect();
    let f = Facts {
        nodes,
        routes: BTreeMap::from([("outside.test".into(), "[outside.lab.test]".into())]),
    };
    let active = BTreeMap::from([
        (
            "gate".into(),
            config(
                "gate.lab.test",
                "gate.lab.test, localhost",
                "192.0.2.0/24",
                "[relay.lab.test]",
                "staff.lab.test",
            ),
        ),
        (
            "relay".into(),
            config(
                "relay.lab.test",
                "relay.lab.test, localhost",
                "192.0.2.10/32",
                "[delivery.lab.test]",
                "",
            ),
        ),
        (
            "delivery".into(),
            config(
                "delivery.lab.test",
                "staff.lab.test, delivery.lab.test, localhost",
                "192.0.2.20/32",
                "",
                "",
            ),
        ),
        (
            "outside".into(),
            config("outside.lab.test", "outside.test, localhost", "", "", ""),
        ),
    ]);
    (f, active)
}
