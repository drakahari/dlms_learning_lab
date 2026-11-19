//! Checks the declared normal-flow tuple subset, not a kernel or TCP oracle.
use dlms_learning_lab::linux_connections::*;
fn endpoint(address: &str, port: u16) -> Endpoint {
    Endpoint {
        address: address.parse().unwrap(),
        port,
    }
}
fn tuple(client: u8) -> Tuple {
    Tuple {
        transport: Transport::Tcp,
        source: endpoint(&format!("192.0.2.{client}"), 40000),
        destination: endpoint("198.51.100.20", 443),
    }
}
#[test]
fn direction_seen_is_not_tcp_handshake_completion() {
    let original = tuple(10);
    let mut t = Table::default();
    assert_eq!(t.preview(original).unwrap().class, Class::New);
    let id = t.confirm(original, original, original, None).unwrap();
    assert_eq!(t.observe(original).unwrap().class, Class::New);
    let reply = t.observe(original.reverse()).unwrap();
    assert_eq!(
        (reply.id, reply.class, reply.direction),
        (Some(id), Class::Established, Direction::Reply)
    );
    assert_eq!(t.observe(original).unwrap().class, Class::Established);
}
#[test]
fn both_translation_directions_reverse_each_mapping_at_the_right_hook() {
    let original = tuple(10);
    let mut forward = original;
    forward.destination = endpoint("203.0.113.42", 8443);
    let mut wire = forward;
    wire.source = endpoint("203.0.113.1", 50000);
    let mut t = Table::default();
    t.confirm(original, forward, wire, Some("wan0".into()))
        .unwrap();
    let outward = t.preview(original).unwrap();
    assert_eq!((outward.filter, outward.outgoing), (forward, wire));
    let inward = t.preview(wire.reverse()).unwrap();
    assert_eq!(
        (inward.filter, inward.outgoing),
        (forward.reverse(), original.reverse())
    );
    assert!(!t.entries[0].seen_reply); // A preview cannot invent an observed reply.
    assert_eq!(t.preview(original.reverse()).unwrap().id, None); // Wrong incoming tuple is no mapped reply.
}
#[test]
fn first_packet_rejected_before_confirmation_has_no_state() {
    let t = Table::default();
    let before = t.clone();
    assert_eq!(t.preview(tuple(10)).unwrap().id, None);
    assert_eq!(t, before);
    // Path/filter driver must only call confirm on acceptance; this asserts the preview boundary.
}
#[test]
fn reply_observation_persists_even_if_later_filter_rejects() {
    let original = tuple(10);
    let mut t = Table::default();
    t.confirm(original, original, original, None).unwrap();
    let observed = t.observe(original.reverse()).unwrap();
    assert_eq!(observed.class, Class::Established);
    // Later filter rejection must not undo the already observed direction.
    assert!(t.entries[0].seen_reply);
    assert_eq!(t.preview(original).unwrap().class, Class::Established);
}
#[test]
fn colliding_wire_and_cross_direction_keys_fail_atomically() {
    let a = tuple(10);
    let b = tuple(11);
    let mut t = Table::default();
    let mut wire = a;
    wire.source = endpoint("203.0.113.1", 50000);
    t.confirm(a, a, wire, None).unwrap();
    let before = t.clone();
    assert!(t.confirm(b, b, wire, None).is_err());
    assert_eq!(t, before);
    assert!(t.confirm(a, a, a, None).is_err());
    assert_eq!(t, before);
    assert!(
        t.confirm(wire.reverse(), wire.reverse(), wire.reverse(), None)
            .is_err()
    );
    assert_eq!(t, before);
}
#[test]
fn protocols_and_distinct_translated_ports_identify_independent_flows() {
    let a = tuple(10);
    let b = tuple(11);
    let mut udp = a;
    udp.transport = Transport::Udp;
    let mut first = a;
    first.source = endpoint("203.0.113.1", 50000);
    let mut second = b;
    second.source = endpoint("203.0.113.1", 50001);
    let mut t = Table::default();
    let one = t.confirm(a, a, first, None).unwrap();
    let two = t.confirm(b, b, second, None).unwrap();
    let three = t.confirm(udp, udp, udp, None).unwrap();
    assert_ne!(one, two);
    assert_ne!(two, three);
    assert_eq!(t.preview(second.reverse()).unwrap().id, Some(two));
    assert_eq!(t.preview(udp.reverse()).unwrap().id, Some(three));
}
#[test]
fn invalid_translation_invariants_and_identifiers_are_not_accepted() {
    let a = tuple(10);
    let mut bad = a;
    bad.source.address = "192.0.2.99".parse().unwrap();
    let mut t = Table::default();
    let before = t.clone();
    assert!(t.confirm(a, bad, bad, None).is_err());
    assert_eq!(t, before);
    let mut bad = a;
    bad.transport = Transport::Udp;
    assert!(t.confirm(a, a, bad, None).is_err());
    assert_eq!(t, before);
    t.next_id = u64::MAX;
    let before = t.clone();
    assert!(t.confirm(a, a, a, None).is_err());
    assert_eq!(t, before);
}
#[test]
fn capacity_is_bounded_and_exhaustion_does_not_consume_an_identifier() {
    let mut t = Table::default();
    for i in 1..=MAX_CONNECTIONS {
        let a = tuple(i as u8);
        t.confirm(a, a, a, None).unwrap();
    }
    let before = t.clone();
    let a = tuple(100);
    assert!(t.confirm(a, a, a, None).is_err());
    assert_eq!(t, before);
    let mut json = serde_json::to_value(&t).unwrap();
    let extra = json["entries"][0].clone();
    json["entries"].as_array_mut().unwrap().push(extra);
    assert!(serde_json::from_value::<Table>(json).is_err());
}
#[test]
fn bounded_data_rejects_unknown_fields_and_oversize_devices() {
    let a = tuple(10);
    let mut t = Table::default();
    t.confirm(a, a, a, Some("wan0".into())).unwrap();
    assert_eq!(
        serde_json::from_str::<Table>(&serde_json::to_string(&t).unwrap()).unwrap(),
        t
    );
    let mut json = serde_json::to_value(&t).unwrap();
    json["entries"][0]["masquerade_device"] = serde_json::json!("x".repeat(16));
    assert!(serde_json::from_value::<Table>(json).is_err());
    let mut json = serde_json::to_value(&t).unwrap();
    json["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Table>(json).is_err());
}
#[test]
fn address_loss_only_forgets_explicit_masquerade_bindings() {
    let mut t = Table::default();
    for (i, d) in [(10, Some("wan0")), (11, None), (12, Some("wan1"))] {
        let a = tuple(i);
        t.confirm(a, a, a, d.map(str::to_string)).unwrap();
    }
    assert_eq!(t.forget_masquerade("wan0").unwrap(), 1);
    assert_eq!(t.entries.len(), 2);
    assert!(t.entries.iter().any(|e| e.masquerade_device.is_none()));
    let before = t.clone();
    assert!(t.forget_masquerade("../wan1").is_err());
    assert_eq!(t, before);
}

#[test]
fn same_address_loop_and_hairpin_tuples_are_explicitly_outside_normal_forwarding() {
    let mut incoming = tuple(10);
    incoming.destination.address = incoming.source.address;
    incoming.destination.port = 443;
    assert!(!incoming.valid());
    let mut t = Table::default();
    let before = t.clone();
    assert!(t.observe(incoming).is_err());
    assert_eq!(t, before);
    assert!(t.confirm(incoming, incoming, incoming, None).is_err());
    assert_eq!(t, before);
}

#[test]
fn destination_flag_comes_from_original_mapping_not_reply_packet_rewrite() {
    let original = tuple(10);
    let mut snat = original;
    snat.source = endpoint("203.0.113.1", 50000);
    let mut t = Table::default();
    t.confirm(original, original, snat, None).unwrap();
    assert!(!t.preview(snat.reverse()).unwrap().dnat); // Inverting source NAT changes the reply destination but is not DNAT status.
    let mut dnat = original;
    dnat.destination = endpoint("203.0.113.42", 8443);
    let mut t = Table::default();
    t.confirm(original, dnat, dnat, None).unwrap();
    assert!(t.preview(original).unwrap().dnat);
    assert!(t.preview(dnat.reverse()).unwrap().dnat);
    let mut port_only = original;
    port_only.destination.port = 8443;
    let mut t = Table::default();
    t.confirm(original, port_only, port_only, None).unwrap();
    assert!(t.preview(original).unwrap().dnat);
}
