use dlms_learning_lab::{
    dhcp_snooping::{ErrorKind, MAX_JSON, Message, Outcome, Packet, Rejection, World},
    mac_learning::{Event, Mac},
};
use std::net::Ipv4Addr;
#[path = "support/dhcp_snooping_fixture.rs"]
mod common;
use common::{fixture, mac, names};
fn ip(n: u8) -> Ipv4Addr {
    Ipv4Addr::new(10, 0, 0, n)
}
fn packet(message: Message, client: u8) -> Packet {
    Packet {
        source: if matches!(
            message,
            Message::Offer { .. } | Message::Ack { .. } | Message::Nak | Message::LeaseQuery
        ) {
            mac(2)
        } else {
            mac(client)
        },
        destination: Mac::BROADCAST,
        transaction: u32::from(client),
        client: mac(client),
        relay: Ipv4Addr::UNSPECIFIED,
        option82: false,
        message,
    }
}
fn world() -> World {
    let mut w = World::new(fixture(), true).unwrap();
    let mut c = w.config().clone();
    c.enabled = true;
    c.vlans = vec![10];
    c.ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = true;
    w.configure(c).unwrap();
    w
}
fn bytes(w: &World) -> Vec<u8> {
    serde_json::to_vec(w).unwrap()
}
fn request(w: &mut World, client: u8) {
    w.transmit("client", &packet(Message::Request, client))
        .unwrap();
}
fn lease(w: &mut World, client: u8, seconds: u32) -> Outcome {
    request(w, client);
    w.transmit(
        "server",
        &packet(
            Message::Ack {
                address: ip(client + 10),
                lease: seconds,
            },
            client,
        ),
    )
    .unwrap()
}
fn observation(o: Outcome) -> dlms_learning_lab::mac_learning::Observation {
    match o {
        Outcome::Forwarded { forwarding, .. } | Outcome::Bypassed { forwarding } => forwarding,
        other => panic!("Expected actual forwarding: {other:?}"),
    }
}

#[test]
fn untrusted_client_really_reaches_only_trusted_output_and_no_binding_on_offer() {
    let mut w = world();
    let o = observation(w.transmit("client", &packet(Message::Discover, 1)).unwrap());
    assert_eq!(o.arrivals, names(&["server"]));
    assert!(
        !o.events
            .iter()
            .any(|e| matches!(e,Event::Forward{egress,..}if egress=="peer"))
    );
    assert_eq!(w.pending_count(), 1);
    let o = observation(
        w.transmit("server", &packet(Message::Offer { address: ip(11) }, 1))
            .unwrap(),
    );
    assert_eq!(o.arrivals, names(&["client"]));
    assert!(w.bindings().is_empty());
}
#[test]
fn initial_request_ack_records_actual_vlan_port_mac_address_and_expiry() {
    let mut w = world();
    let out = lease(&mut w, 1, 120);
    let Outcome::Forwarded {
        forwarding,
        learned: Some(b),
        removed: false,
    } = out
    else {
        panic!("ACK must create an observed lease")
    };
    assert_eq!(forwarding.accepted_by, names(&["client"]));
    assert_eq!(
        (b.client, b.address, b.vlan, b.port.as_str(), b.expires),
        (mac(1), ip(11), 10, "client", 120)
    );
    assert_eq!(w.bindings(), &[b]);
    assert_eq!(w.pending_count(), 0);
}
#[test]
fn rogue_server_types_and_spoofed_client_metadata_are_rejected_without_state_change() {
    let mut w = world();
    request(&mut w, 1);
    for message in [
        Message::Offer { address: ip(90) },
        Message::Ack {
            address: ip(90),
            lease: 60,
        },
        Message::Nak,
        Message::LeaseQuery,
    ] {
        let before = bytes(&w);
        let mut p = packet(message, 1);
        p.source = mac(3);
        assert_eq!(
            w.transmit("peer", &p).unwrap(),
            Outcome::Rejected {
                cause: Rejection::UntrustedServer
            }
        );
        assert_eq!(bytes(&w), before);
    }
    let mut p = packet(Message::Discover, 1);
    p.source = mac(3);
    let before = bytes(&w);
    assert_eq!(
        w.transmit("client", &p).unwrap(),
        Outcome::Rejected {
            cause: Rejection::SourceMismatch
        }
    );
    assert_eq!(bytes(&w), before);
}
#[test]
fn trust_and_source_verification_have_real_effects_instead_of_reference_matching() {
    let mut w = world();
    let mut c = w.config().clone();
    c.ports
        .iter_mut()
        .find(|p| p.port == "peer")
        .unwrap()
        .trusted = true;
    c.verify_source = false;
    w.configure(c).unwrap();
    let mut p = packet(Message::Discover, 1);
    p.source = mac(9);
    let o = observation(w.transmit("client", &p).unwrap());
    assert_eq!(o.accepted_by, names(&["server", "peer"]));
    let mut offer = packet(Message::Offer { address: ip(90) }, 1);
    offer.source = mac(3);
    let o = observation(w.transmit("peer", &offer).unwrap());
    assert_eq!(o.accepted_by, names(&["client"]));
    assert!(
        w.bindings().is_empty(),
        "A permitted rogue Offer is not an ACK or authenticated lease"
    );
}
#[test]
fn global_and_vlan_enablement_are_both_required_and_inactive_path_matches_old_state() {
    for global in [false, true] {
        let mut w = world();
        let mut c = w.config().clone();
        c.enabled = global;
        c.vlans = if global { vec![20] } else { vec![10] };
        w.configure(c).unwrap();
        let mut old = w.forwarding().clone();
        let expected = old.send("client", Mac::BROADCAST).unwrap();
        assert_eq!(
            w.transmit("client", &packet(Message::Discover, 1)).unwrap(),
            Outcome::Bypassed {
                forwarding: expected
            }
        );
        assert_eq!(w.forwarding(), &old);
        assert_eq!(w.pending_count(), 0);
        assert!(w.bindings().is_empty());
    }
}
#[test]
fn no_trusted_path_does_not_invent_an_observed_exchange() {
    let mut w = world();
    let mut c = w.config().clone();
    for p in &mut c.ports {
        p.trusted = false;
    }
    w.configure(c).unwrap();
    let o = observation(w.transmit("client", &packet(Message::Discover, 1)).unwrap());
    assert!(o.arrivals.is_empty());
    assert_eq!(w.pending_count(), 0);
    assert!(w.bindings().is_empty());
}
#[test]
fn trusted_client_exchange_works_without_creating_an_untrusted_client_binding() {
    let mut w = world();
    let mut c = w.config().clone();
    c.ports
        .iter_mut()
        .find(|p| p.port == "client")
        .unwrap()
        .trusted = true;
    w.configure(c).unwrap();
    let o = lease(&mut w, 1, 30);
    assert!(matches!(o, Outcome::Forwarded { learned: None, .. }));
    assert!(w.bindings().is_empty());
}
#[test]
fn unsolicited_ack_ack_before_request_and_failed_client_delivery_are_honest_unsupported_errors() {
    let mut w = world();
    let before = bytes(&w);
    assert_eq!(
        w.transmit(
            "server",
            &packet(
                Message::Ack {
                    address: ip(11),
                    lease: 30
                },
                1
            )
        )
        .unwrap_err()
        .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(bytes(&w), before);
    w.transmit("client", &packet(Message::Discover, 1)).unwrap();
    let before = bytes(&w);
    assert_eq!(
        w.transmit(
            "server",
            &packet(
                Message::Ack {
                    address: ip(11),
                    lease: 30
                },
                1
            )
        )
        .unwrap_err()
        .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(bytes(&w), before);
    request(&mut w, 1);
    let mut p = packet(
        Message::Ack {
            address: ip(11),
            lease: 30,
        },
        1,
    );
    p.destination = mac(90);
    let before = bytes(&w);
    assert_eq!(
        w.transmit("server", &p).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(bytes(&w), before);
}
#[test]
fn matching_release_removes_only_its_lease_and_wrong_port_release_is_rejected() {
    let mut w = world();
    lease(&mut w, 1, 30);
    lease(&mut w, 5, 60);
    let before = bytes(&w);
    assert_eq!(
        w.transmit("peer", &packet(Message::Release { address: ip(11) }, 1))
            .unwrap(),
        Outcome::Rejected {
            cause: Rejection::WrongReleasePort
        }
    );
    assert_eq!(bytes(&w), before);
    let out = w
        .transmit("client", &packet(Message::Release { address: ip(11) }, 1))
        .unwrap();
    assert!(matches!(out, Outcome::Forwarded { removed: true, .. }));
    assert_eq!(w.bindings().len(), 1);
    assert_eq!(w.bindings()[0].client, mac(5));
}
#[test]
fn expiry_boundaries_and_pending_timeout_are_explicit_and_resume_exactly() {
    let mut w = world();
    lease(&mut w, 1, 30);
    request(&mut w, 5);
    w.advance(29).unwrap();
    assert_eq!(w.bindings().len(), 1);
    assert_eq!(w.pending_count(), 1);
    let raw = serde_json::to_string(&w).unwrap();
    let mut restored = World::from_json(&raw).unwrap();
    assert_eq!(restored, w);
    restored.advance(1).unwrap();
    assert!(restored.bindings().is_empty());
    assert_eq!(restored.pending_count(), 1);
    restored.advance(30).unwrap();
    assert_eq!(restored.pending_count(), 0);
}
#[test]
fn optional_relay_checks_do_not_pretend_to_implement_transformations() {
    let mut w = world();
    for option82 in [false, true] {
        let mut p = packet(Message::Discover, 1);
        p.option82 = option82;
        if !option82 {
            p.relay = ip(254);
        }
        let before = bytes(&w);
        assert_eq!(
            w.transmit("client", &p).unwrap(),
            Outcome::Rejected {
                cause: Rejection::RelayMetadata
            }
        );
        assert_eq!(bytes(&w), before);
        let before = bytes(&w);
        assert_eq!(
            w.transmit("server", &p).unwrap_err().kind,
            ErrorKind::Unsupported
        );
        assert_eq!(bytes(&w), before);
    }
    assert_eq!(
        World::new(fixture(), false).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn other_protocol_flows_are_diagnosed_and_invalid_metadata_is_atomic() {
    let mut w = world();
    for message in [
        Message::Inform,
        Message::Decline,
        Message::Nak,
        Message::LeaseQuery,
    ] {
        let before = bytes(&w);
        assert_eq!(
            w.transmit("server", &packet(message, 1)).unwrap_err().kind,
            ErrorKind::Unsupported
        );
        assert_eq!(bytes(&w), before);
    }
    for p in [
        packet(
            Message::Ack {
                address: ip(11),
                lease: 0,
            },
            1,
        ),
        packet(
            Message::Offer {
                address: Ipv4Addr::UNSPECIFIED,
            },
            1,
        ),
        Packet {
            source: Mac([0; 6]),
            ..packet(Message::Discover, 1)
        },
    ] {
        let before = bytes(&w);
        assert_eq!(
            w.transmit("client", &p).unwrap_err().kind,
            ErrorKind::Invalid
        );
        assert_eq!(bytes(&w), before);
    }
}
#[test]
fn changed_policy_clears_sample_routing_cache_without_erasing_unexpired_records() {
    let mut w = world();
    lease(&mut w, 1, 30);
    request(&mut w, 5);
    let saved = w.bindings().to_vec();
    let mut c = w.config().clone();
    c.verify_source = false;
    w.configure(c).unwrap();
    assert_eq!(w.pending_count(), 0);
    assert_eq!(w.bindings(), saved);
    let before = bytes(&w);
    let mut c = w.config().clone();
    c.ports.pop();
    assert!(w.configure(c).is_err());
    assert_eq!(bytes(&w), before);
}
#[test]
fn binding_and_transaction_resource_limits_leave_the_exact_state_unchanged() {
    let mut pending = world();
    for n in 1..=64 {
        request(&mut pending, n);
    }
    let before = bytes(&pending);
    assert_eq!(
        pending
            .transmit("client", &packet(Message::Request, 65))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(bytes(&pending), before);
    let mut bound = world();
    for n in 1..=64 {
        lease(&mut bound, n, 120);
    }
    request(&mut bound, 65);
    let before = bytes(&bound);
    assert_eq!(
        bound
            .transmit(
                "server",
                &packet(
                    Message::Ack {
                        address: ip(100),
                        lease: 120
                    },
                    65
                )
            )
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(bytes(&bound), before);
}
#[test]
fn schema_malformed_duplicate_expired_and_oversized_imports_fail_closed() {
    let mut w = world();
    lease(&mut w, 1, 30);
    request(&mut w, 5);
    let value = serde_json::to_value(&w).unwrap();
    for (field, replacement) in [
        ("schema_version", serde_json::json!(2)),
        ("information_option_disabled", serde_json::json!(false)),
        (
            "pending",
            serde_json::json!([value["pending"][0].clone(), value["pending"][0].clone()]),
        ),
        (
            "bindings",
            serde_json::json!([value["bindings"][0].clone(), value["bindings"][0].clone()]),
        ),
    ] {
        let mut bad = value.clone();
        bad[field] = replacement;
        assert!(World::from_json(&bad.to_string()).is_err(), "{field}");
    }
    let mut bad = value.clone();
    bad["bindings"][0]["expires"] = serde_json::json!(0);
    assert!(World::from_json(&bad.to_string()).is_err());
    assert_eq!(
        World::from_json(&" ".repeat(MAX_JSON + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(Packet::from_json(r#"{"kind":"a-real-wire-packet"}"#).is_err());
    assert_eq!(
        Packet::from_json(&" ".repeat(4097)).unwrap_err().kind,
        ErrorKind::Limit
    );
}

#[test]
fn physical_source_failures_are_explained_and_never_misreported_as_security_rejections() {
    for cause in ["endpoint", "port", "cable"] {
        let mut raw = serde_json::to_value(fixture()).unwrap();
        let f = &mut raw["fabric"];
        match cause {
            "endpoint" => f["hosts"]["client"]["up"] = serde_json::json!(false),
            "port" => f["switches"]["edge"]["ports"]["client"]["up"] = serde_json::json!(false),
            _ => f["cables"][0]["up"] = serde_json::json!(false),
        }
        let initial = dlms_learning_lab::mac_learning::State::decode(&raw.to_string()).unwrap();
        let mut w = World::new(initial, true).unwrap();
        let mut c = w.config().clone();
        c.enabled = true;
        c.vlans = vec![10];
        w.configure(c).unwrap();
        let before = bytes(&w);
        let out = w.transmit("client", &packet(Message::Discover, 1)).unwrap();
        assert!(
            matches!(out,Outcome::Unavailable{reason}if reason.contains(cause)),
            "{cause}"
        );
        assert_eq!(bytes(&w), before);
    }
}

#[test]
fn matching_release_is_observed_even_without_an_output_and_wrong_ip_keeps_the_record() {
    let mut w = world();
    lease(&mut w, 1, 60);
    assert!(matches!(
        w.transmit("client", &packet(Message::Release { address: ip(99) }, 1))
            .unwrap(),
        Outcome::Forwarded { removed: false, .. }
    ));
    assert_eq!(w.bindings().len(), 1);
    let mut c = w.config().clone();
    for p in &mut c.ports {
        p.trusted = false;
    }
    w.configure(c).unwrap();
    let out = w
        .transmit("client", &packet(Message::Release { address: ip(11) }, 1))
        .unwrap();
    assert!(
        matches!(out,Outcome::Forwarded{forwarding,removed:true,..}if forwarding.arrivals.is_empty())
    );
    assert!(w.bindings().is_empty());
}

#[test]
fn conflicting_addresses_and_ambiguous_attachments_fail_atomically_with_specific_boundaries() {
    let mut w = world();
    lease(&mut w, 1, 60);
    request(&mut w, 5);
    let before = bytes(&w);
    let err = w
        .transmit(
            "server",
            &packet(
                Message::Ack {
                    address: ip(11),
                    lease: 60,
                },
                5,
            ),
        )
        .unwrap_err();
    assert_eq!(err.kind, ErrorKind::Unsupported);
    assert!(err.message.contains("Conflicting"));
    assert_eq!(bytes(&w), before);
    let err = w
        .transmit("peer", &packet(Message::Request, 5))
        .unwrap_err();
    assert_eq!(err.kind, ErrorKind::Unsupported);
    assert!(err.message.contains("competing"));
    assert_eq!(bytes(&w), before);
}

#[test]
fn equivalent_config_order_preserves_the_observed_exchange_and_supported_values_are_not_hardcoded()
{
    let mut w = world();
    request(&mut w, 1);
    let mut c = w.config().clone();
    c.ports.reverse();
    w.configure(c).unwrap();
    assert_eq!(w.pending_count(), 1);
    let address = Ipv4Addr::new(172, 22, 7, 203);
    let out = w
        .transmit(
            "server",
            &packet(
                Message::Ack {
                    address,
                    lease: 86_400,
                },
                1,
            ),
        )
        .unwrap();
    assert!(
        matches!(out,Outcome::Forwarded{learned:Some(b),..}if b.address==address&&b.expires==86_400)
    );
    w.advance(86_399).unwrap();
    assert_eq!(w.bindings().len(), 1);
    w.advance(1).unwrap();
    assert!(w.bindings().is_empty());
    lease(&mut w, 1, 1);
    w.advance(1).unwrap();
    assert!(w.bindings().is_empty());
}

#[test]
fn clock_and_advance_bounds_reject_without_partial_learning_or_overflow() {
    let mut w = world();
    let before = bytes(&w);
    assert_eq!(w.advance(86_401).unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(bytes(&w), before);
    let mut raw = serde_json::to_value(&w).unwrap();
    raw["baseline"]["now"] = serde_json::json!(u64::MAX);
    raw["active"]["now"] = serde_json::json!(u64::MAX);
    let mut w = World::from_json(&raw.to_string()).unwrap();
    let before = bytes(&w);
    assert_eq!(
        w.transmit("client", &packet(Message::Discover, 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(bytes(&w), before);
    assert_eq!(w.advance(1).unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(bytes(&w), before);
}
