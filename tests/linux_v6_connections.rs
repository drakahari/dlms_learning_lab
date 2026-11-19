//! Direction observations in the disclosed subset; not TCP/kernel acceptance.
use dlms_learning_lab::linux_v6_connections::*;
fn flow(client: u16) -> Tuple {
    Tuple {
        transport: Transport::Tcp,
        source: Endpoint {
            address: format!("fd20:30:40::{client:x}").parse().unwrap(),
            port: 40000,
        },
        destination: Endpoint {
            address: "2001:db8:50::42".parse().unwrap(),
            port: 443,
        },
    }
}
#[test]
fn linux_v6_preview_observation_and_confirmation_have_separate_effects() {
    let a = flow(1);
    let mut t = Table::default();
    let empty = t.clone();
    let o = t.observe(a).unwrap();
    assert_eq!(
        (o.id, o.class, o.direction),
        (None, Class::New, Direction::Original)
    );
    assert_eq!(t, empty); // Rejected first event creates no connection.
    let id = t.confirm(a).unwrap();
    assert_eq!(t.observe(a).unwrap().class, Class::New);
    let before = t.clone();
    let reply = t.preview(a.reverse()).unwrap();
    assert_eq!(
        (reply.id, reply.class, reply.direction),
        (Some(id), Class::Established, Direction::Reply)
    );
    assert_eq!(t, before); // Looking at a reply does not observe it.
    assert_eq!(t.observe(a).unwrap().class, Class::New);
    assert_eq!(t.observe(a.reverse()).unwrap(), reply);
    assert!(t.entries[0].seen_reply); // Retained even if a later filter drops this reply.
    assert_eq!(t.observe(a).unwrap().class, Class::Established);
}
#[test]
fn linux_v6_tuple_identity_retains_high_bits_ports_and_protocol() {
    let a = flow(1);
    let mut t = Table::default();
    let first = t.confirm(a).unwrap();
    let mut variations = vec![];
    for (src, dst, sp, dp, transport) in [
        (
            "2001:db8:30:40::1",
            "2001:db8:50::42",
            40000,
            443,
            Transport::Tcp,
        ),
        (
            "fd20:30:40::1",
            "fd20:db8:50::42",
            40000,
            443,
            Transport::Tcp,
        ),
        (
            "fd20:30:40::1",
            "2001:db8:50::42",
            40001,
            443,
            Transport::Tcp,
        ),
        (
            "fd20:30:40::1",
            "2001:db8:50::42",
            40000,
            8443,
            Transport::Tcp,
        ),
        (
            "fd20:30:40::1",
            "2001:db8:50::42",
            40000,
            443,
            Transport::Udp,
        ),
    ] {
        let q = Tuple {
            source: Endpoint {
                address: src.parse().unwrap(),
                port: sp,
            },
            destination: Endpoint {
                address: dst.parse().unwrap(),
                port: dp,
            },
            transport,
        };
        assert_eq!(t.preview(q).unwrap().id, None);
        let id = t.confirm(q).unwrap();
        assert_ne!(id, first);
        variations.push((q, id));
    }
    for (q, id) in variations {
        assert_eq!(t.preview(q.reverse()).unwrap().id, Some(id));
    }
    assert_eq!(t.preview(a).unwrap().id, Some(first));
}
#[test]
fn linux_v6_reverse_duplicate_invalid_endpoint_and_id_refusals_are_atomic() {
    let a = flow(1);
    let mut t = Table::default();
    t.confirm(a).unwrap();
    for q in [a, a.reverse()] {
        let before = t.clone();
        assert!(t.confirm(q).is_err());
        assert_eq!(t, before);
    }
    for bad in [
        "::",
        "::1",
        "::ffff:192.0.2.1",
        "ff02::1",
        "fe80::2",
        "febf::1",
        "2001:db8:50::42",
    ] {
        let mut q = a;
        q.source.address = bad.parse().unwrap();
        let before = t.clone();
        assert!(t.observe(q).is_err(), "{bad}");
        assert!(t.confirm(q).is_err(), "{bad}");
        assert_eq!(t, before);
    }
    t.next_id = u64::MAX;
    let before = t.clone();
    assert!(t.confirm(flow(2)).is_err());
    assert_eq!(t, before);
}
#[test]
fn linux_v6_full_table_still_observes_existing_replies_but_refuses_new_entries() {
    let mut t = Table::default();
    for i in 1..=MAX_CONNECTIONS {
        t.confirm(flow(i as u16)).unwrap();
    }
    let before = t.clone();
    assert!(t.confirm(flow(65)).is_err());
    assert_eq!(t, before);
    assert_eq!(
        t.observe(flow(64).reverse()).unwrap().class,
        Class::Established
    );
    assert_eq!(t.entries.len(), 64);
    assert!(t.entries[63].seen_reply);
}
#[test]
fn linux_v6_saved_tables_roundtrip_and_reject_ambiguity_unknown_fields_and_oversize() {
    let mut t = Table::default();
    t.confirm(flow(1)).unwrap();
    t.observe(flow(1).reverse()).unwrap();
    let text = serde_json::to_string(&t).unwrap();
    let restored: Table = serde_json::from_str(&text).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, t);
    let mut wrong = t.clone();
    wrong.entries.push(Entry {
        id: 2,
        original: flow(1).reverse(),
        seen_reply: false,
    });
    wrong.next_id = 3;
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.entries[0].id = 0;
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.next_id = 1;
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.entries[0].original.source.address = "fe80::1".parse().unwrap();
    assert!(wrong.validate().is_err());
    let mut v = serde_json::to_value(&t).unwrap();
    v["host_command"] = "anything".into();
    assert!(serde_json::from_value::<Table>(v).is_err());
    let mut v = serde_json::to_value(&t).unwrap();
    v["entries"] = serde_json::Value::Array(vec![v["entries"][0].clone(); 65]);
    assert!(serde_json::from_value::<Table>(v).is_err());
}
