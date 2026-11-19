//! Independent point checks plus full-domain proofs and plausible wrong policies.
use dlms_learning_lab::{
    linux_connections::{Endpoint, Transport, Tuple},
    linux_filter::{self as base, Cidr, Packet, Protocol, Rule, Verdict},
    linux_flow_nat::Table,
    linux_nat_policy::{self as proof, Requirement},
    linux_router_path::{Facts, Link},
};
fn facts() -> Facts {
    Facts {
        links: vec![
            Link {
                name: "lan0".into(),
                address: "10.20.30.1".parse().unwrap(),
                prefix: 24,
                up: true,
            },
            Link {
                name: "wan0".into(),
                address: "198.51.100.1".parse().unwrap(),
                prefix: 24,
                up: true,
            },
        ],
        routes: vec![],
    }
}
fn wanted() -> Endpoint {
    Endpoint {
        address: "10.20.30.42".parse().unwrap(),
        port: 443,
    }
}
fn input_domain(protocol: Protocol) -> Rule {
    Rule {
        source: None,
        destination: None,
        ingress: Some("wan0".into()),
        egress: None,
        protocol: Some(protocol),
        source_port: None,
        destination_port: None,
        target: Verdict::Accept,
    }
}
fn published(protocol: Protocol) -> Rule {
    Rule {
        destination: Some(Cidr::parse("198.51.100.1/32").unwrap()),
        destination_port: Some(8443),
        ..input_domain(protocol)
    }
}
#[test]
fn a_precise_publication_proves_both_the_mapping_and_no_other_destination_changes() {
    let mut t = Table::default();
    t.command("iptables -t nat -A PREROUTING -i wan0 -d 198.51.100.1 -p tcp --dport 8443 -j DNAT --to-destination 10.20.30.42:443").unwrap();
    let f = facts();
    for (transport, protocol) in [
        (Transport::Tcp, Protocol::Tcp),
        (Transport::Udp, Protocol::Udp),
    ] {
        let domain = input_domain(protocol);
        let expected = if transport == Transport::Tcp {
            vec![published(protocol)]
        } else {
            vec![]
        };
        assert!(
            proof::compare(
                &t,
                &f,
                transport,
                Requirement::ChangedToDestination(wanted()),
                std::slice::from_ref(&domain),
                &expected
            )
            .unwrap()
            .is_none()
        );
        assert!(
            proof::compare(
                &t,
                &f,
                transport,
                Requirement::DestinationOrIdentity(wanted()),
                std::slice::from_ref(&domain),
                std::slice::from_ref(&domain)
            )
            .unwrap()
            .is_none()
        );
    }
}
#[test]
fn a_wrong_other_server_does_not_hide_behind_a_false_required_endpoint_property() {
    let mut t = Table::default();
    t.command("iptables -t nat -A PREROUTING -d 198.51.100.1 -p tcp --dport 8443 -j DNAT --to-destination 10.20.30.42:443").unwrap();
    t.command("iptables -t nat -A PREROUTING -p tcp --dport 9443 -j DNAT --to-destination 10.20.30.99:443").unwrap();
    let domain = input_domain(Protocol::Tcp);
    assert!(
        proof::compare(
            &t,
            &facts(),
            Transport::Tcp,
            Requirement::ChangedToDestination(wanted()),
            std::slice::from_ref(&domain),
            &[published(Protocol::Tcp)]
        )
        .unwrap()
        .is_none()
    );
    let failure = proof::compare(
        &t,
        &facts(),
        Transport::Tcp,
        Requirement::DestinationOrIdentity(wanted()),
        std::slice::from_ref(&domain),
        std::slice::from_ref(&domain),
    )
    .unwrap()
    .unwrap();
    assert_eq!(failure.packet.destination_port, Some(9443));
    assert_eq!(failure.rule, Some(2)); // Actual POST/PREROUTING index, never a derived proof row.
    assert_eq!(failure.actual, Verdict::Drop);
    assert_eq!(failure.expected, Verdict::Accept);
}
#[test]
fn broad_shadowing_wrong_ports_and_hardcoded_clients_fail_exact_proofs() {
    for command in [
        "iptables -t nat -A PREROUTING -j DNAT --to-destination 10.20.30.99",
        "iptables -t nat -A PREROUTING -d 198.51.100.1 -p tcp --dport 8443 -j DNAT --to-destination 10.20.30.42",
        "iptables -t nat -A PREROUTING -s 203.0.113.55 -d 198.51.100.1 -p tcp --dport 8443 -j DNAT --to-destination 10.20.30.42:443",
    ] {
        let mut t = Table::default();
        t.command(command).unwrap();
        let domain = input_domain(Protocol::Tcp);
        assert!(
            proof::compare(
                &t,
                &facts(),
                Transport::Tcp,
                Requirement::ChangedToDestination(wanted()),
                &[domain],
                &[published(Protocol::Tcp)]
            )
            .unwrap()
            .is_some(),
            "{command}"
        );
    }
}
#[test]
fn split_source_prefixes_are_a_valid_equivalent_translation_policy() {
    let mut t = Table::default();
    for prefix in ["0.0.0.0/1", "128.0.0.0/1"] {
        t.command(&format!("iptables -t nat -A PREROUTING -s {prefix} -d 198.51.100.1 -p tcp --dport 8443 -j DNAT --to-destination 10.20.30.42:443")).unwrap();
    }
    let d = input_domain(Protocol::Tcp);
    assert!(
        proof::compare(
            &t,
            &facts(),
            Transport::Tcp,
            Requirement::ChangedToDestination(wanted()),
            &[d],
            &[published(Protocol::Tcp)]
        )
        .unwrap()
        .is_none()
    );
}
#[test]
fn identity_targets_do_not_fabricate_destination_translation_status() {
    let mut t = Table::default();
    t.command("iptables -t nat -A PREROUTING -d 10.20.30.42 -p tcp --dport 443 -j DNAT --to-destination 10.20.30.42:443").unwrap();
    let d = input_domain(Protocol::Tcp);
    assert!(
        proof::compare(
            &t,
            &facts(),
            Transport::Tcp,
            Requirement::ChangedToDestination(wanted()),
            std::slice::from_ref(&d),
            &[]
        )
        .unwrap()
        .is_none()
    );
    assert!(
        proof::compare(
            &t,
            &facts(),
            Transport::Tcp,
            Requirement::DestinationOrIdentity(wanted()),
            std::slice::from_ref(&d),
            std::slice::from_ref(&d)
        )
        .unwrap()
        .is_none()
    );
}
#[test]
fn derived_destination_properties_match_independent_actual_translation_at_boundary_points() {
    let mut t = Table::default();
    for c in [
        "iptables -t nat -A PREROUTING -d 198.51.100.1 -p tcp --dport 8443 -j DNAT --to-destination 10.20.30.42:443",
        "iptables -t nat -A PREROUTING -p tcp --dport 9443 -j DNAT --to-destination 10.20.30.99:443",
        "iptables -t nat -A PREROUTING -d 10.20.30.0/24 -j DNAT --to-destination 10.20.30.42",
    ] {
        t.command(c).unwrap();
    }
    let w = wanted();
    for transport in [Transport::Tcp, Transport::Udp] {
        for requirement in [
            Requirement::ChangedToDestination(w),
            Requirement::DestinationOrIdentity(w),
        ] {
            let (compiled, chain) =
                proof::property_table(&t, &facts(), transport, requirement).unwrap();
            for address in [
                "198.51.100.1",
                "198.51.100.2",
                "10.20.30.42",
                "10.20.30.99",
                "203.0.113.9",
            ] {
                for port in [80, 443, 8443, 9443] {
                    let original = Tuple {
                        transport,
                        source: Endpoint {
                            address: "203.0.113.55".parse().unwrap(),
                            port: 50000,
                        },
                        destination: Endpoint {
                            address: address.parse().unwrap(),
                            port,
                        },
                    };
                    let (actual, _) = t.destination(original, "wan0").unwrap();
                    let expected = match requirement {
                        Requirement::ChangedToDestination(w) => {
                            actual.destination == w && actual.destination != original.destination
                        }
                        Requirement::DestinationOrIdentity(w) => {
                            actual.destination == w || actual.destination == original.destination
                        }
                        _ => unreachable!(),
                    };
                    let packet = Packet {
                        source: original.source.address,
                        destination: original.destination.address,
                        ingress: Some("wan0".into()),
                        egress: None,
                        protocol: if transport == Transport::Tcp {
                            Protocol::Tcp
                        } else {
                            Protocol::Udp
                        },
                        source_port: Some(50000),
                        destination_port: Some(port),
                    };
                    assert_eq!(
                        compiled.evaluate(chain, &packet).unwrap().verdict == Verdict::Accept,
                        expected,
                        "{transport:?}/{requirement:?}/{address}/{port}"
                    );
                }
            }
        }
    }
}
#[test]
fn source_address_proof_accepts_snat_and_masquerade_but_rejects_sample_only() {
    let domain = Rule {
        source: Some(Cidr::parse("10.20.30.0/24").unwrap()),
        destination: None,
        ingress: None,
        egress: Some("wan0".into()),
        protocol: Some(Protocol::Tcp),
        source_port: None,
        destination_port: Some(443),
        target: Verdict::Accept,
    };
    let f = facts();
    let target = "198.51.100.1".parse().unwrap();
    for action in ["SNAT --to-source 198.51.100.1", "MASQUERADE"] {
        let mut t = Table::default();
        t.command(&format!(
            "iptables -t nat -A POSTROUTING -s 10.20.30.0/24 -o wan0 -j {action}"
        ))
        .unwrap();
        assert!(
            proof::compare(
                &t,
                &f,
                Transport::Tcp,
                Requirement::SourceAddress(target),
                std::slice::from_ref(&domain),
                std::slice::from_ref(&domain)
            )
            .unwrap()
            .is_none()
        );
    }
    let mut t = Table::default();
    t.command("iptables -t nat -A POSTROUTING -s 10.20.30.55 -j SNAT --to-source 198.51.100.1")
        .unwrap();
    assert!(
        proof::compare(
            &t,
            &f,
            Transport::Tcp,
            Requirement::SourceAddress(target),
            std::slice::from_ref(&domain),
            std::slice::from_ref(&domain)
        )
        .unwrap()
        .is_some()
    );
}
#[test]
fn masquerade_proof_depends_on_the_actual_outgoing_interface_address() {
    let mut t = Table::default();
    t.command("iptables -t nat -A POSTROUTING -j MASQUERADE")
        .unwrap();
    let req = Requirement::SourceAddress("198.51.100.1".parse().unwrap());
    let (compiled, chain) = proof::property_table(&t, &facts(), Transport::Tcp, req).unwrap();
    for (egress, expected) in [("wan0", Verdict::Accept), ("lan0", Verdict::Drop)] {
        let packet = Packet {
            source: "192.0.2.55".parse().unwrap(),
            destination: "203.0.113.9".parse().unwrap(),
            ingress: None,
            egress: Some(egress.into()),
            protocol: Protocol::Tcp,
            source_port: Some(40000),
            destination_port: Some(443),
        };
        assert_eq!(compiled.evaluate(chain, &packet).unwrap().verdict, expected);
    }
}
#[test]
fn proof_requires_bound_protocols_and_preserves_derived_rule_limits() {
    let mut t = Table::default();
    for _ in 0..32 {
        t.command("iptables -t nat -A PREROUTING -j DNAT --to-destination 10.20.30.42")
            .unwrap();
    }
    let (compiled, _) = proof::property_table(
        &t,
        &facts(),
        Transport::Tcp,
        Requirement::ChangedToDestination(wanted()),
    )
    .unwrap();
    assert!(compiled.input.rules.len() <= base::MAX_RULES);
    compiled.validate().unwrap();
    let mut unbound = input_domain(Protocol::Tcp);
    unbound.protocol = None;
    assert!(
        proof::compare(
            &t,
            &facts(),
            Transport::Tcp,
            Requirement::ChangedToDestination(wanted()),
            &[unbound],
            &[]
        )
        .is_err()
    );
}
