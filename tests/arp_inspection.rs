use dlms_learning_lab::{arp_inspection::*, dhcp_snooping as dhcp, mac_learning::Mac};
#[path = "support/dhcp_snooping_fixture.rs"]
mod access;
use access::{fixture, mac, names};
fn client_message(message: dhcp::Message) -> dhcp::Packet {
    dhcp::Packet {
        source: mac(1),
        destination: Mac::BROADCAST,
        transaction: 7,
        client: mac(1),
        relay: "0.0.0.0".parse().unwrap(),
        option82: false,
        message,
    }
}
fn observed() -> dhcp::World {
    let mut w = dhcp::World::new(fixture(), true).unwrap();
    let mut c = w.config().clone();
    c.enabled = true;
    c.vlans = vec![10];
    c.ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = true;
    w.configure(c).unwrap();
    let request = client_message(dhcp::Message::Request);
    assert!(matches!(
        w.transmit("client", &request).unwrap(),
        dhcp::Outcome::Forwarded { .. }
    ));
    let mut ack = request;
    ack.source = mac(2);
    ack.destination = mac(1);
    ack.message = dhcp::Message::Ack {
        address: "10.1.0.10".parse().unwrap(),
        lease: 120,
    };
    assert!(matches!(
        w.transmit("server", &ack).unwrap(),
        dhcp::Outcome::Forwarded {
            learned: Some(_),
            ..
        }
    ));
    assert_eq!(w.bindings().len(), 1);
    w
}
fn world() -> World {
    let mut w = World::new(observed()).unwrap();
    let mut c = w.config().clone();
    c.vlans = vec![10];
    w.configure(c).unwrap();
    w
}
fn packet(operation: Operation) -> Packet {
    Packet {
        operation,
        ethernet_source: mac(1),
        ethernet_destination: if operation == Operation::Request {
            Mac::BROADCAST
        } else {
            mac(2)
        },
        sender_mac: mac(1),
        sender_ip: "10.1.0.10".parse().unwrap(),
        target_mac: if operation == Operation::Request {
            Mac([0; 6])
        } else {
            mac(2)
        },
        target_ip: "10.1.0.1".parse().unwrap(),
    }
}
fn allowed(outcome: Outcome, expected: Admission, recipients: &[&str]) {
    let Outcome::Forwarded {
        admission,
        forwarding,
    } = outcome
    else {
        panic!("{outcome:?}")
    };
    assert_eq!(admission, expected);
    assert_eq!(forwarding.accepted_by, names(recipients));
}
fn rejected(outcome: Outcome, expected: Rejection) {
    assert_eq!(outcome, Outcome::Rejected { cause: expected });
}
fn filter(p: &Packet, action: Action, static_only: bool) -> Filter {
    Filter {
        vlan: 10,
        static_only,
        rules: vec![Rule {
            action,
            sender_ip: p.sender_ip,
            sender_mac: p.sender_mac,
        }],
    }
}
#[test]
fn defaults_are_separate_from_dhcp_trust_and_optional_checks() {
    let d = observed();
    assert!(
        d.config()
            .ports
            .iter()
            .find(|p| p.port == "server")
            .unwrap()
            .trusted
    );
    let mut w = World::new(d).unwrap();
    assert!(w.config().vlans.is_empty());
    assert!(w.config().ports.iter().all(|p| !p.trusted));
    assert_eq!(w.config().checks, Checks::default());
    allowed(
        w.transmit("client", &packet(Operation::Request)).unwrap(),
        Admission::VlanNotInspected,
        &["server", "peer"],
    );
    let mut c = w.config().clone();
    c.vlans = vec![10];
    w.configure(c).unwrap();
    let mut p = packet(Operation::Reply);
    p.ethernet_source = mac(2);
    p.ethernet_destination = mac(1);
    p.sender_mac = mac(2);
    p.sender_ip = "10.1.0.1".parse().unwrap();
    p.target_mac = mac(1);
    rejected(w.transmit("server", &p).unwrap(), Rejection::MissingBinding);
    let mut c = w.config().clone();
    c.ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = true;
    c.checks = Checks {
        source_mac: true,
        destination_mac: true,
        ip: true,
    };
    w.configure(c).unwrap();
    p.sender_ip = "0.0.0.0".parse().unwrap();
    p.target_ip = "0.0.0.0".parse().unwrap();
    p.target_mac = mac(9);
    p.sender_mac = mac(8);
    allowed(
        w.transmit("server", &p).unwrap(),
        Admission::TrustedIngress,
        &["client"],
    );
}
#[test]
fn live_request_and_reply_use_actual_recipients_without_inventing_a_new_binding() {
    let mut w = world();
    let original = w.dhcp().bindings().to_vec();
    allowed(
        w.transmit("client", &packet(Operation::Request)).unwrap(),
        Admission::DhcpBinding,
        &["server", "peer"],
    );
    allowed(
        w.transmit("client", &packet(Operation::Reply)).unwrap(),
        Admission::DhcpBinding,
        &["server"],
    );
    assert_eq!(w.dhcp().bindings(), original);
}
#[test]
fn forged_claim_is_rejected_before_source_learning_or_recipient_delivery() {
    let mut w = world();
    let old = w.dhcp().clone();
    let mut p = packet(Operation::Request);
    p.ethernet_source = mac(3);
    p.sender_mac = mac(3);
    rejected(w.transmit("peer", &p).unwrap(), Rejection::MissingBinding);
    assert_eq!(w.dhcp(), &old);
    assert!(
        !w.dhcp()
            .forwarding()
            .entries()
            .iter()
            .any(|e| e.address == mac(3))
    );
}
#[test]
fn a_binding_in_another_vlan_does_not_authorize_a_claim() {
    let mut w = world();
    let mut p = packet(Operation::Request);
    p.sender_mac = mac(4);
    p.ethernet_source = mac(4);
    allowed(
        w.transmit("other", &p).unwrap(),
        Admission::VlanNotInspected,
        &[],
    );
    let mut c = w.config().clone();
    c.vlans = vec![10, 20];
    w.configure(c).unwrap();
    rejected(w.transmit("other", &p).unwrap(), Rejection::MissingBinding);
}
#[test]
fn an_exact_arp_acl_supports_a_static_sender_without_creating_dhcp_history() {
    let mut w = world();
    let mut p = packet(Operation::Request);
    p.ethernet_source = mac(3);
    p.sender_mac = mac(3);
    p.sender_ip = "10.1.0.99".parse().unwrap();
    rejected(w.transmit("peer", &p).unwrap(), Rejection::MissingBinding);
    let mut c = w.config().clone();
    c.filters = vec![filter(&p, Action::Permit, false)];
    w.configure(c).unwrap();
    allowed(
        w.transmit("peer", &p).unwrap(),
        Admission::ArpAcl,
        &["client", "server"],
    );
    assert_eq!(w.dhcp().bindings().len(), 1);
    assert_eq!(w.dhcp().bindings()[0].client, mac(1));
}
#[test]
fn explicit_deny_order_and_static_fallback_are_distinct_real_outcomes() {
    let mut w = world();
    let p = packet(Operation::Request);
    let mut c = w.config().clone();
    c.filters = vec![Filter {
        vlan: 10,
        static_only: false,
        rules: vec![],
    }];
    w.configure(c.clone()).unwrap();
    allowed(
        w.transmit("client", &p).unwrap(),
        Admission::DhcpBinding,
        &["peer", "server"],
    );
    c.filters[0].static_only = true;
    w.configure(c.clone()).unwrap();
    rejected(w.transmit("client", &p).unwrap(), Rejection::StaticAclMiss);
    c.filters = vec![filter(&p, Action::Deny, false)];
    w.configure(c.clone()).unwrap();
    rejected(
        w.transmit("client", &p).unwrap(),
        Rejection::ExplicitAclDeny,
    );
    c.filters[0].rules.insert(
        0,
        Rule {
            action: Action::Permit,
            sender_ip: p.sender_ip,
            sender_mac: p.sender_mac,
        },
    );
    w.configure(c.clone()).unwrap();
    allowed(
        w.transmit("client", &p).unwrap(),
        Admission::ArpAcl,
        &["peer", "server"],
    );
    c.filters[0].rules.reverse();
    w.configure(c).unwrap();
    rejected(
        w.transmit("client", &p).unwrap(),
        Rejection::ExplicitAclDeny,
    );
}
#[test]
fn source_field_comparison_is_optional_and_does_not_replace_the_receiving_nic() {
    let mut w = world();
    let mut p = packet(Operation::Request);
    p.ethernet_source = mac(9);
    allowed(
        w.transmit("client", &p).unwrap(),
        Admission::DhcpBinding,
        &["peer", "server"],
    );
    assert_eq!(w.dhcp().forwarding().address("client"), Some(mac(1)));
    let mut c = w.config().clone();
    c.checks.source_mac = true;
    w.configure(c).unwrap();
    let old = w.dhcp().clone();
    rejected(
        w.transmit("client", &p).unwrap(),
        Rejection::SourceMacMismatch,
    );
    assert_eq!(w.dhcp(), &old);
}
#[test]
fn destination_mac_validation_applies_to_replies_only() {
    let mut w = world();
    let mut c = w.config().clone();
    c.checks.destination_mac = true;
    w.configure(c).unwrap();
    allowed(
        w.transmit("client", &packet(Operation::Request)).unwrap(),
        Admission::DhcpBinding,
        &["peer", "server"],
    );
    let mut p = packet(Operation::Reply);
    p.target_mac = mac(9);
    rejected(
        w.transmit("client", &p).unwrap(),
        Rejection::DestinationMacMismatch,
    );
}
#[test]
fn ip_validation_checks_sender_always_and_target_only_in_replies() {
    for ip in ["0.0.0.0", "255.255.255.255", "224.0.0.1", "239.255.255.255"] {
        for op in [Operation::Request, Operation::Reply] {
            let mut w = world();
            let mut p = packet(op);
            p.sender_ip = ip.parse().unwrap();
            let mut c = w.config().clone();
            c.filters = vec![filter(&p, Action::Permit, false)];
            w.configure(c.clone()).unwrap();
            let recipients = if op == Operation::Request {
                vec!["peer", "server"]
            } else {
                vec!["server"]
            };
            if p.sender_ip.is_unspecified() {
                let before = w.clone();
                assert_eq!(
                    w.transmit("client", &p).unwrap_err().kind,
                    ErrorKind::Unsupported
                );
                assert_eq!(w, before);
            } else {
                allowed(
                    w.transmit("client", &p).unwrap(),
                    Admission::ArpAcl,
                    &recipients,
                );
            }
            c.checks.ip = true;
            w.configure(c).unwrap();
            rejected(w.transmit("client", &p).unwrap(), Rejection::InvalidIp);
        }
        let mut w = world();
        let mut c = w.config().clone();
        c.checks.ip = true;
        w.configure(c).unwrap();
        let mut request = packet(Operation::Request);
        request.target_ip = ip.parse().unwrap();
        allowed(
            w.transmit("client", &request).unwrap(),
            Admission::DhcpBinding,
            &["peer", "server"],
        );
        request.operation = Operation::Reply;
        rejected(
            w.transmit("client", &request).unwrap(),
            Rejection::InvalidIp,
        );
    }
}
#[test]
fn zero_sender_probe_is_not_guessed_as_a_binding_drop_when_ip_check_is_off() {
    let mut w = world();
    let mut p = packet(Operation::Request);
    p.sender_ip = "0.0.0.0".parse().unwrap();
    let before = w.clone();
    let e = w.transmit("client", &p).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert!(e.message.contains("zero sender IP"));
    assert_eq!(w, before);
    let mut c = w.config().clone();
    c.checks.ip = true;
    w.configure(c).unwrap();
    rejected(w.transmit("client", &p).unwrap(), Rejection::InvalidIp);
}
#[test]
fn expiry_and_actual_release_remove_binding_authority_without_changing_arp_policy() {
    let mut expired = world();
    let c = expired.config().clone();
    expired.advance(120).unwrap();
    assert!(expired.dhcp().bindings().is_empty());
    rejected(
        expired
            .transmit("client", &packet(Operation::Request))
            .unwrap(),
        Rejection::MissingBinding,
    );
    assert_eq!(expired.config(), &c);
    let mut released = world();
    let c = released.config().clone();
    let release = client_message(dhcp::Message::Release {
        address: "10.1.0.10".parse().unwrap(),
    });
    assert!(matches!(
        released.transmit_dhcp("client", &release).unwrap(),
        dhcp::Outcome::Forwarded { removed: true, .. }
    ));
    rejected(
        released
            .transmit("client", &packet(Operation::Request))
            .unwrap(),
        Rejection::MissingBinding,
    );
    assert_eq!(released.config(), &c);
}
#[test]
fn unresolved_cross_port_binding_is_atomic_and_not_guessed_from_ip_source_guard() {
    let mut w = world();
    let old = w.clone();
    let p = packet(Operation::Request);
    let e = w.transmit("peer", &p).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert!(e.message.contains("another port"));
    assert_eq!(w, old);
    let mut c = w.config().clone();
    c.filters = vec![filter(&p, Action::Permit, false)];
    w.configure(c).unwrap();
    // A configured VLAN ACL is a supported independent authorization rule.
    allowed(
        w.transmit("peer", &p).unwrap(),
        Admission::ArpAcl,
        &["client", "server"],
    );
}
#[test]
fn malformed_configuration_and_imports_fail_closed_and_keep_the_previous_world() {
    let mut w = world();
    let old = w.clone();
    let mut c = w.config().clone();
    c.ports.pop();
    assert!(w.configure(c).is_err());
    assert_eq!(w, old);
    let mut c = w.config().clone();
    c.vlans = vec![10, 10];
    assert!(w.configure(c).is_err());
    assert_eq!(w, old);
    let mut c = w.config().clone();
    c.filters = vec![filter(&packet(Operation::Request), Action::Permit, false); 17];
    assert!(w.configure(c).is_err());
    assert_eq!(w, old);
    let value = serde_json::to_value(&w).unwrap();
    for (key, bad) in [
        ("schema_version", serde_json::json!(2)),
        ("arrivals", serde_json::json!(9)),
        ("last_arp_second", serde_json::json!(1)),
    ] {
        let mut invalid = value.clone();
        invalid[key] = bad;
        assert!(World::from_json(&invalid.to_string()).is_err());
    }
    assert_eq!(
        World::from_json(&serde_json::to_string(&w).unwrap()).unwrap(),
        w
    );
    assert_eq!(
        World::from_json(&" ".repeat(MAX_JSON + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        Packet::from_json(&" ".repeat(4097)).unwrap_err().kind,
        ErrorKind::Limit
    );
    let mut bad = serde_json::to_value(packet(Operation::Request)).unwrap();
    bad["operation"] = serde_json::json!("rarp");
    assert!(Packet::from_json(&bad.to_string()).is_err());
    bad = serde_json::to_value(packet(Operation::Request)).unwrap();
    bad["shell"] = serde_json::json!("touch /outside");
    assert!(Packet::from_json(&bad.to_string()).is_err());
    assert_eq!(
        w.transmit("../../etc/passwd", &packet(Operation::Request))
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
    assert_eq!(w, old);
}
#[test]
fn bounded_arrivals_are_a_simulator_limit_not_a_fabricated_equipment_drop() {
    let mut w = world();
    let p = packet(Operation::Request);
    for _ in 0..MAX_ARRIVALS {
        assert!(matches!(
            w.transmit("client", &p).unwrap(),
            Outcome::Forwarded { .. }
        ));
    }
    let old = w.clone();
    let e = w.transmit("client", &p).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Limit);
    assert!(e.message.contains("hardware"));
    assert_eq!(w, old);
    w.advance(0).unwrap();
    assert!(w.transmit("client", &p).is_err());
    w.advance(1).unwrap();
    allowed(
        w.transmit("client", &p).unwrap(),
        Admission::DhcpBinding,
        &["peer", "server"],
    );
}
#[test]
fn physical_failure_is_not_reported_as_a_security_rejection() {
    for kind in 0..3 {
        let original = fixture();
        let s = &original;
        let mut f = s.fabric().clone();
        match kind {
            0 => f.hosts.get_mut("client").unwrap().up = false,
            1 => {
                f.cables
                    .iter_mut()
                    .find(|c| c.id == "wire-client")
                    .unwrap()
                    .up = false
            }
            _ => {
                f.switches
                    .get_mut("edge")
                    .unwrap()
                    .ports
                    .get_mut("client")
                    .unwrap()
                    .up = false
            }
        }
        let addresses = s
            .fabric()
            .hosts
            .keys()
            .map(|host| (host.clone(), s.address(host).unwrap()))
            .collect();
        let s = dlms_learning_lab::mac_learning::State::new(f, addresses, 300).unwrap();
        let mut w = World::new(dhcp::World::new(s, true).unwrap()).unwrap();
        let old = w.clone();
        assert!(matches!(
            w.transmit("client", &packet(Operation::Request)).unwrap(),
            Outcome::Unavailable { .. }
        ));
        assert_eq!(w, old);
    }
}
#[test]
fn equivalent_configuration_order_is_canonical_but_rule_order_remains_meaningful() {
    let mut w = world();
    let mut c = w.config().clone();
    c.vlans = vec![20, 10];
    c.ports.reverse();
    let p = packet(Operation::Request);
    c.filters = vec![
        Filter {
            vlan: 20,
            static_only: false,
            rules: vec![],
        },
        filter(&p, Action::Permit, false),
    ];
    w.configure(c).unwrap();
    assert_eq!(w.config().vlans, vec![10, 20]);
    assert!(w.config().ports.windows(2).all(|p| p[0].port < p[1].port));
    assert_eq!(
        w.config()
            .filters
            .iter()
            .map(|f| f.vlan)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    allowed(
        w.transmit("client", &p).unwrap(),
        Admission::ArpAcl,
        &["peer", "server"],
    );
}
#[test]
fn ordinary_arp_composition_preserves_pending_dhcp_and_its_current_ethernet_state() {
    let mut d = observed();
    let request = client_message(dhcp::Message::Request);
    d.transmit("client", &request).unwrap();
    assert_eq!(d.pending_count(), 1);
    let mut w = World::new(d).unwrap();
    let before = w.dhcp().bindings().to_vec();
    w.transmit("client", &packet(Operation::Request)).unwrap();
    assert_eq!(w.dhcp().pending_count(), 1);
    assert_eq!(w.dhcp().bindings(), before);
    let mut ack = request;
    ack.source = mac(2);
    ack.destination = mac(1);
    ack.message = dhcp::Message::Ack {
        address: "10.1.0.10".parse().unwrap(),
        lease: 240,
    };
    assert!(matches!(
        w.transmit_dhcp("server", &ack).unwrap(),
        dhcp::Outcome::Forwarded {
            learned: Some(_),
            ..
        }
    ));
    assert_eq!(w.dhcp().bindings()[0].expires, 240);
}
