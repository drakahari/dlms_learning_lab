use super::*;
const DB: &str = "http 80/tcp www\nreports-demo 8080/tcp local-reports\ndomain 53/tcp\ndomain 53/udp dns\nhttps 443/tcp\nhttps 443/udp\nssh 22/tcp\n";
fn profile() -> Profile {
    Profile {
        version: 1,
        client: "192.0.2.10".parse().unwrap(),
        server: "192.0.2.20".parse().unwrap(),
        listeners: vec![
            Listener {
                role: "resolver".into(),
                protocol: net::Protocol::Udp,
                port: 53,
            },
            Listener {
                role: "reports".into(),
                protocol: net::Protocol::Tcp,
                port: 8080,
            },
            Listener {
                role: "remote".into(),
                protocol: net::Protocol::Tcp,
                port: 22,
            },
            Listener {
                role: "web".into(),
                protocol: net::Protocol::Tcp,
                port: 443,
            },
        ],
    }
}
#[test]
fn equivalent_name_alias_and_numeric_targets_reuse_existing_transport_behavior() {
    let p = profile();
    for target in ["reports-demo", "local-reports", "8080", " 08080 "] {
        let r = run(&p, DB, net::Protocol::Tcp, target, "report-A", &[]).unwrap();
        assert_eq!(r.resolved.port, 8080);
        assert_eq!(r.transport.outcome, net::Outcome::Delivered);
        assert_eq!(r.transport.received.as_deref(), Some("report-A"));
        assert_eq!(r.transport.trace.len(), 4);
    }
    let r = run(&p, DB, net::Protocol::Udp, "dns", "question-B", &[]).unwrap();
    assert_eq!(r.transport.outcome, net::Outcome::Delivered);
    assert_eq!(r.transport.trace.len(), 1);
    assert_eq!(r.transport.received.as_deref(), Some("question-B"));
}
#[test]
fn a_successful_lookup_does_not_imply_a_listener_or_application_support() {
    let p = profile();
    assert_eq!(
        names::execute(DB, "getent services domain/tcp")
            .unwrap()
            .status,
        0
    );
    let r = run(&p, DB, net::Protocol::Tcp, "domain", "question", &[]).unwrap();
    assert_eq!(r.resolved.port, 53);
    assert_eq!(r.transport.outcome, net::Outcome::NoListener);
    assert_eq!(r.transport.received, None);
    assert_eq!(
        names::execute(DB, "getent services https/udp")
            .unwrap()
            .status,
        0
    );
    assert_eq!(
        run(&p, DB, net::Protocol::Udp, "https", "request", &[])
            .unwrap()
            .transport
            .outcome,
        net::Outcome::NoListener
    );
    assert_eq!(
        run(&p, DB, net::Protocol::Tcp, "http", "request", &[])
            .unwrap()
            .transport
            .outcome,
        net::Outcome::NoListener
    );
}
#[test]
fn service_file_changes_resolution_without_moving_the_server_socket() {
    let p = profile();
    let before = p.clone();
    let changed = DB.replace("domain 53/udp dns", "domain 54/udp dns");
    assert_eq!(
        names::execute(&changed, "getent services dns/udp")
            .unwrap()
            .status,
        0
    );
    let r = run(&p, &changed, net::Protocol::Udp, "domain", "question", &[]).unwrap();
    assert_eq!(r.resolved.port, 54);
    assert_eq!(r.transport.outcome, net::Outcome::NoListener);
    assert_eq!(p, before);
    assert_eq!(
        run(&p, DB, net::Protocol::Udp, "domain", "question", &[])
            .unwrap()
            .transport
            .outcome,
        net::Outcome::Delivered
    );
}
#[test]
fn a_wrong_open_service_can_deliver_but_never_magically_becomes_the_required_target() {
    let p = profile();
    let correct = run(&p, DB, net::Protocol::Tcp, "ssh", "opaque", &[]).unwrap();
    let wrong = run(&p, DB, net::Protocol::Tcp, "reports-demo", "opaque", &[]).unwrap();
    assert_eq!(correct.transport.outcome, net::Outcome::Delivered);
    assert_eq!(wrong.transport.outcome, net::Outcome::Delivered);
    assert_ne!(correct.resolved.port, wrong.resolved.port);
    assert_eq!(correct.resolved.port, p.listener("remote").unwrap().port);
    // Later grading MUST compare this independently, not just count deliveries.
    assert_ne!(wrong.resolved.port, p.listener("remote").unwrap().port);
}
#[test]
fn source_held_listener_unavailability_is_independent_of_name_lookup() {
    let p = profile();
    let r = run(&p, DB, net::Protocol::Tcp, "ssh", "one", &["remote".into()]).unwrap();
    assert_eq!(r.resolved.port, 22);
    assert_eq!(r.transport.outcome, net::Outcome::NoListener);
    assert!(r.transport.received.is_none());
    assert_eq!(
        names::execute(DB, "getent services ssh/tcp")
            .unwrap()
            .status,
        0
    );
    assert!(run(&p, DB, net::Protocol::Tcp, "ssh", "two", &["absent".into()]).is_err());
    assert!(
        run(
            &p,
            DB,
            net::Protocol::Tcp,
            "ssh",
            "two",
            &["remote".into(), "remote".into()]
        )
        .is_err()
    );
}
#[test]
fn numeric_ports_bypass_name_lookup_but_names_use_the_current_virtual_file() {
    let broken = "# unfinished\nhttp";
    assert_eq!(
        resolve(broken, net::Protocol::Tcp, "443")
            .unwrap()
            .canonical_name,
        None
    );
    assert_eq!(
        run(
            &profile(),
            broken,
            net::Protocol::Tcp,
            "443",
            "payload",
            &[]
        )
        .unwrap()
        .transport
        .outcome,
        net::Outcome::Delivered
    );
    assert_eq!(
        resolve(broken, net::Protocol::Tcp, "https")
            .unwrap_err()
            .kind,
        names::FaultKind::Syntax
    );
    assert!(resolve(DB, net::Protocol::Tcp, "SSH").is_err());
    assert_eq!(
        resolve(DB, net::Protocol::Tcp, "ssh")
            .unwrap()
            .canonical_name
            .as_deref(),
        Some("ssh")
    );
}
#[test]
fn malformed_unsupported_and_bounded_inputs_fail_without_mutating_profile_or_database() {
    let p = profile();
    let before = p.clone();
    for value in [
        "",
        " ",
        "0",
        "65536",
        "-1",
        "ssh/udp",
        "$(touch /private)",
        "../../private",
    ] {
        assert!(
            run(&p, DB, net::Protocol::Tcp, value, "data", &[]).is_err(),
            "{value}"
        );
    }
    assert!(run(&p, DB, net::Protocol::Tcp, &"x".repeat(65), "data", &[]).is_err());
    assert!(
        run(
            &p,
            DB,
            net::Protocol::Tcp,
            "ssh",
            &"x".repeat(net::MAX_PAYLOAD_BYTES + 1),
            &[]
        )
        .is_err()
    );
    assert_eq!(p, before);
    let mut bad = p.clone();
    bad.server = bad.client;
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.listeners.push(bad.listeners[0].clone());
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(&p).unwrap();
    value["host_path"] = serde_json::json!("/etc/services");
    assert!(serde_json::from_value::<Profile>(value).is_err());
    let mut value = serde_json::to_value(&p).unwrap();
    value["listeners"] = serde_json::json!(vec![&p.listeners[0]; 9]);
    assert!(serde_json::from_value::<Profile>(value).is_err());
}
