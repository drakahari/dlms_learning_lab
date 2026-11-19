use dlms_learning_lab::{
    smtp_config::ErrorKind,
    smtp_routing::{HopAction, Outcome},
};
use std::collections::BTreeSet;
#[path = "support/smtp_fixture.rs"]
mod setup;
use setup::{fixture, ip};
#[test]
fn each_hop_sees_its_actual_immediate_peer_and_delivery_identifies_the_endpoint() {
    let (f, a) = fixture();
    let t = f
        .trace(&a, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
        .unwrap();
    assert_eq!(
        t.hops.iter().map(|h| h.node.as_str()).collect::<Vec<_>>(),
        ["gate", "relay", "delivery"]
    );
    assert_eq!(
        t.hops
            .iter()
            .map(|h| h.client.expect("New SMTP hop has a peer"))
            .collect::<Vec<_>>(),
        [ip("203.0.113.9"), ip("192.0.2.10"), ip("192.0.2.20")]
    );
    assert_eq!(
        t.outcome,
        Outcome::Local {
            node: "delivery".into(),
            mailboxes: BTreeSet::from(["nora".into()]),
            failures: BTreeSet::new()
        }
    );
}
#[test]
fn untrusted_relay_denies_while_trusted_outbound_and_untrusted_local_delivery_still_work() {
    let (f, a) = fixture();
    assert_eq!(
        f.trace(&a, "gate", ip("203.0.113.9"), "visitor@outside.test")
            .unwrap()
            .outcome,
        Outcome::Rejected {
            node: "gate".into()
        }
    );
    let t = f
        .trace(&a, "gate", ip("192.0.2.50"), "visitor@outside.test")
        .unwrap();
    assert!(matches!(t.outcome,Outcome::Local{ref node,..}if node=="outside"));
    assert!(
        matches!(f.trace(&a,"gate",ip("203.0.113.9"),"operator@gate.lab.test").unwrap().outcome,Outcome::Local{ref node,..}if node=="gate")
    );
    let mut wrong = a.clone();
    wrong
        .get_mut("gate")
        .unwrap()
        .replace_range(.., &a["gate"].replace("192.0.2.0/24", "0.0.0.0/0"));
    assert!(
        matches!(f.trace(&wrong,"gate",ip("203.0.113.9"),"visitor@outside.test").unwrap().outcome,Outcome::Local{ref node,..}if node=="outside")
    );
}
#[test]
fn wrong_local_destination_partial_aliases_and_unknown_users_never_become_delivery_success() {
    let (mut f, mut a) = fixture();
    a.insert(
        "gate".into(),
        a["gate"].replace(
            "gate.lab.test, localhost",
            "gate.lab.test, staff.lab.test, localhost",
        ),
    );
    let t = f
        .trace(&a, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
        .unwrap();
    assert!(
        matches!(t.outcome,Outcome::Local{ref node,ref mailboxes,ref failures}if node=="gate"&&mailboxes.is_empty()&&!failures.is_empty())
    );
    let (_, a) = fixture();
    f.nodes.get_mut("delivery").unwrap().aliases = "team: nora, absent\n".into();
    let t = f
        .trace(&a, "gate", ip("203.0.113.9"), "team@staff.lab.test")
        .unwrap();
    assert!(
        matches!(t.outcome,Outcome::Local{ref node,ref mailboxes,ref failures}if node=="delivery"&&mailboxes==&BTreeSet::from(["nora".into()])&&!failures.is_empty())
    );
}
#[test]
fn loops_terminate_and_legitimate_alternate_routes_pass_without_trusting_old_output() {
    let (f, a) = fixture();
    let mut looped = a.clone();
    looped.insert(
        "relay".into(),
        a["relay"].replace("[delivery.lab.test]", "[gate.lab.test]"),
    );
    let t = f
        .trace(&looped, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
        .unwrap();
    assert_eq!(
        t.outcome,
        Outcome::Loop {
            repeats: "relay".into(),
            client: Some(ip("192.0.2.10")),
        }
    );
    assert!(t.hops.len() <= f.nodes.len() + 1);
    let mut direct = a.clone();
    direct.insert(
        "gate".into(),
        a["gate"].replace("[relay.lab.test]", "[delivery.lab.test]"),
    );
    // Final destination accepts incoming local mail without requiring trusted relay.
    let t = f
        .trace(&direct, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
        .unwrap();
    assert!(matches!(t.outcome,Outcome::Local{ref node,..}if node=="delivery"));
    assert_eq!(t.hops.len(), 2);
    assert!(matches!(
        f.trace(&looped, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .outcome,
        Outcome::Loop { .. }
    ));
}

#[test]
fn revisiting_a_server_with_a_different_peer_can_reject_instead_of_forming_a_loop() {
    let (f, mut active) = fixture();
    let gate = active["gate"].replace("192.0.2.0/24", "192.0.2.50/32");
    let relay = active["relay"].replace("[delivery.lab.test]", "[gate.lab.test]");
    active.insert("gate".into(), gate);
    active.insert("relay".into(), relay);
    let trace = f
        .trace(&active, "gate", ip("192.0.2.50"), "visitor@outside.test")
        .unwrap();
    assert_eq!(
        trace.outcome,
        Outcome::Rejected {
            node: "gate".into()
        }
    );
    assert_eq!(
        trace
            .hops
            .iter()
            .map(|h| (h.node.as_str(), h.client.expect("New SMTP hop has a peer")))
            .collect::<Vec<_>>(),
        [
            ("gate", ip("192.0.2.50")),
            ("relay", ip("192.0.2.10")),
            ("gate", ip("192.0.2.20"))
        ]
    );
}
#[test]
fn unavailable_wrong_port_unknown_target_and_missing_route_are_honest_operational_outcomes() {
    let (mut f, a) = fixture();
    f.nodes.get_mut("relay").unwrap().available = false;
    assert!(matches!(
        f.trace(&a, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .outcome,
        Outcome::Unavailable { .. }
    ));
    f.nodes.get_mut("relay").unwrap().available = true;
    for target in ["[relay.lab.test]:2525", "[not-in-model.test]"] {
        let mut wrong = a.clone();
        wrong.insert("gate".into(), a["gate"].replace("[relay.lab.test]", target));
        assert!(matches!(
            f.trace(&wrong, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
                .unwrap()
                .outcome,
            Outcome::Unavailable { .. }
        ));
    }
    let mut no_route = a.clone();
    no_route.insert("gate".into(), a["gate"].replace("[relay.lab.test]", ""));
    assert_eq!(
        f.trace(&no_route, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .outcome,
        Outcome::NoRoute {
            node: "gate".into()
        }
    );
}
#[test]
fn errors_belong_to_visited_servers_and_endpoint_literals_do_not_contact_the_host() {
    let (f, a) = fixture();
    let mut broken = a.clone();
    broken.insert("outside".into(), "unsupported = data".into());
    assert!(matches!(
        f.trace(&broken, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .outcome,
        Outcome::Local { .. }
    ));
    broken.insert("relay".into(), "unsupported = data".into());
    assert!(
        matches!(f.trace(&broken,"gate",ip("203.0.113.9"),"nora@staff.lab.test").unwrap().outcome,Outcome::ConfigurationProblem{ref node,ref error}if node=="relay"&&error.kind==ErrorKind::Unsupported)
    );
    let mut literal = a.clone();
    literal.insert(
        "gate".into(),
        a["gate"].replace("[relay.lab.test]", "[192.0.2.20]"),
    );
    assert_eq!(
        f.trace(&literal, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .outcome,
        f.trace(&a, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .outcome
    );
}
#[test]
fn malformed_foreign_and_excessive_world_inputs_refuse_before_any_route() {
    let (f, a) = fixture();
    assert!(
        f.trace(&a, "../../host", ip("203.0.113.9"), "nora@staff.lab.test")
            .is_err()
    );
    let mut foreign = a.clone();
    foreign.insert("foreign".into(), a["gate"].clone());
    assert!(
        f.trace(&foreign, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .is_err()
    );
    for address in [
        "nora",
        "Name@staff.lab.test",
        "|command@staff.lab.test",
        "nora@bad..test",
    ] {
        assert!(f.trace(&a, "gate", ip("203.0.113.9"), address).is_err());
    }
    let mut duplicate = f.clone();
    duplicate.nodes.get_mut("relay").unwrap().address = ip("192.0.2.10");
    assert!(duplicate.validate().is_err());
    let mut oversized = f.clone();
    oversized.nodes.get_mut("relay").unwrap().aliases = "x".repeat(131073);
    assert_eq!(oversized.validate().unwrap_err().kind, ErrorKind::Limit);
    let mut wrong = f.clone();
    wrong
        .routes
        .insert("staff.lab.test".into(), "[unknown.test]".into());
    assert!(wrong.validate().is_err());
    assert!(
        f.trace(&a, "gate", ip("203.0.113.9"), "nora@staff.lab.test")
            .unwrap()
            .hops
            .iter()
            .any(|h| matches!(h.action, HopAction::Relay { .. }))
    );
}

#[test]
fn already_accepted_queue_work_is_not_readmitted_but_downstream_still_checks_its_peer() {
    let (f, mut active) = fixture();
    active.insert(
        "gate".into(),
        active["gate"].replace("192.0.2.0/24", "192.0.2.99/32"),
    );
    assert!(
        matches!(f.trace(&active,"gate","192.0.2.50".parse().unwrap(),"visitor@outside.test").unwrap().outcome,Outcome::Rejected{ref node}if node=="gate")
    );
    let queued = f
        .trace_queued(&active, "gate", "visitor@outside.test")
        .unwrap();
    assert!(matches!(queued.outcome,Outcome::Local{ref node,..}if node=="outside"));
    assert_eq!(queued.hops[0].client, None);
    assert_eq!(queued.hops[1].client, Some("192.0.2.10".parse().unwrap()));
    active.insert(
        "relay".into(),
        active["relay"].replace("192.0.2.10/32", "192.0.2.99/32"),
    );
    assert!(
        matches!(f.trace_queued(&active,"gate","visitor@outside.test").unwrap().outcome,Outcome::Rejected{ref node}if node=="relay")
    );
}
