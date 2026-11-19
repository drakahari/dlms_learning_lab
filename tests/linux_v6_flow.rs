//! In-process synthetic packet events only. No sockets, actual TCP or kernel oracle.
use dlms_learning_lab::{
    linux_v6_connections as ct, linux_v6_filter as filter, linux_v6_flow::State,
    linux_v6_path as path, linux_v6_state_filter::Table,
};
fn facts() -> path::Facts {
    path::Facts {
        links: vec![
            path::Link {
                name: "lan0".into(),
                address: "fd20:30:40::1".parse().unwrap(),
                link_local: "fe80::1".parse().unwrap(),
                prefix: 64,
                up: true,
            },
            path::Link {
                name: "wan0".into(),
                address: "2001:db8:50::1".parse().unwrap(),
                link_local: "fe80::2".parse().unwrap(),
                prefix: 64,
                up: true,
            },
        ],
        routes: vec![path::Route {
            destination: filter::Cidr::parse("::/0").unwrap(),
            device: "wan0".into(),
            via: Some("fe80::fe".parse().unwrap()),
            metric: 10,
        }],
    }
}
fn header() -> path::Header {
    path::Header {
        source: "fd20:30:40::55".parse().unwrap(),
        destination: "2001:db8:60::42".parse().unwrap(),
        protocol: filter::Protocol::Tcp,
        source_port: Some(40000),
        destination_port: Some(443),
        icmp_type: None,
        icmp_code: None,
    }
}
fn reverse(h: &path::Header) -> path::Header {
    path::Header {
        source: h.destination,
        destination: h.source,
        source_port: h.destination_port,
        destination_port: h.source_port,
        ..h.clone()
    }
}
fn state() -> State {
    let mut s = State {
        filter: Table::new(filter::Verdict::Drop),
        connections: ct::Table::default(),
    };
    for c in [
        "ip6tables -A FORWARD -i lan0 -o wan0 -p tcp --dport 443 -m conntrack --ctstate NEW -j ACCEPT",
        "ip6tables -A FORWARD -p tcp -m conntrack --ctstate ESTABLISHED -j ACCEPT",
    ] {
        s.filter.command(c).unwrap();
    }
    s
}
#[test]
fn linux_v6_flow_tracks_request_actual_reply_and_later_original_without_nat() {
    let mut s = state();
    let f = facts();
    let h = header();
    let first = s.event(&f, true, "lan0", &h).unwrap();
    assert_eq!(first.path.gate, path::Gate::Allowed);
    assert_eq!(first.observation.unwrap().class, ct::Class::New);
    assert_eq!(first.path.rule, Some(1));
    let id = first.connection.unwrap();
    let repeated = s.event(&f, true, "lan0", &h).unwrap();
    assert_eq!(repeated.observation.unwrap().class, ct::Class::New);
    assert_eq!(repeated.connection, Some(id));
    assert_eq!(s.connections.entries.len(), 1);
    let reply = s.event(&f, true, "wan0", &reverse(&h)).unwrap();
    assert_eq!(
        (reply.path.gate, reply.path.rule),
        (path::Gate::Allowed, Some(2))
    );
    let o = reply.observation.unwrap();
    assert_eq!(
        (o.class, o.direction, o.id),
        (ct::Class::Established, ct::Direction::Reply, Some(id))
    );
    assert_eq!(
        s.event(&f, true, "lan0", &h)
            .unwrap()
            .observation
            .unwrap()
            .class,
        ct::Class::Established
    );
}
#[test]
fn linux_v6_flow_unsolicited_reverse_is_new_and_dropped_not_invented_as_a_reply() {
    let mut s = state();
    let before = s.clone();
    let h = reverse(&header());
    let t = s.event(&facts(), true, "wan0", &h).unwrap();
    assert_eq!(t.path.gate, path::Gate::FilterDrop);
    assert_eq!(
        (t.observation.unwrap().class, t.connection),
        (ct::Class::New, None)
    );
    assert_eq!(s, before);
}
#[test]
fn linux_v6_flow_reply_observation_survives_filter_route_and_forwarding_failure() {
    for gate in [
        path::Gate::FilterDrop,
        path::Gate::NoRoute,
        path::Gate::ForwardingDisabled,
        path::Gate::EgressDown,
    ] {
        let mut s = state();
        let mut f = facts();
        let h = header();
        s.event(&f, true, "lan0", &h).unwrap();
        let mut reply = reverse(&h);
        match gate {
            path::Gate::FilterDrop => {
                s.filter
                    .command(
                        "ip6tables -I FORWARD -p tcp -m conntrack --ctstate ESTABLISHED -j DROP",
                    )
                    .unwrap();
            }
            path::Gate::NoRoute => {
                f.routes.clear();
                f.links[0].prefix = 128;
            }
            path::Gate::EgressDown => f.links[0].up = false,
            path::Gate::ForwardingDisabled => {}
            _ => unreachable!(),
        }
        // Keep the true recorded reverse tuple, including all 128 address bits.
        reply.destination = h.source;
        let t = s
            .event(&f, gate != path::Gate::ForwardingDisabled, "wan0", &reply)
            .unwrap();
        assert_eq!(t.path.gate, gate);
        assert_eq!(t.observation.unwrap().class, ct::Class::Established);
        assert!(s.connections.entries[0].seen_reply);
    }
}
#[test]
fn linux_v6_flow_down_ingress_and_rejected_initial_packets_do_not_create_state() {
    let mut s = state();
    let h = header();
    let mut f = facts();
    f.links[0].up = false;
    let before = s.clone();
    let t = s.event(&f, true, "lan0", &h).unwrap();
    assert_eq!(
        (t.path.gate, t.observation),
        (path::Gate::IngressDown, None)
    );
    assert_eq!(s, before);
    let mut q = h;
    q.destination_port = Some(22);
    let t = s.event(&facts(), true, "lan0", &q).unwrap();
    assert_eq!(t.path.gate, path::Gate::FilterDrop);
    assert!(s.connections.entries.is_empty());
    let t = s.event(&facts(), false, "lan0", &header()).unwrap();
    assert_eq!(t.path.gate, path::Gate::ForwardingDisabled);
    assert!(s.connections.entries.is_empty());
}
#[test]
fn linux_v6_flow_numeric_icmp_headers_are_stateless_and_link_local_cannot_route() {
    let mut s = state();
    s.filter
        .command("ip6tables -A FORWARD -p ipv6-icmp --icmpv6-type 2/0 -j ACCEPT")
        .unwrap();
    let f = facts();
    let mut h = header();
    h.protocol = filter::Protocol::Icmp6;
    h.source_port = None;
    h.destination_port = None;
    h.icmp_type = Some(2);
    h.icmp_code = Some(0);
    let t = s.event(&f, true, "lan0", &h).unwrap();
    assert_eq!(
        (t.path.gate, t.observation, t.connection),
        (path::Gate::Allowed, None, None)
    );
    assert!(s.connections.entries.is_empty());
    h.icmp_code = Some(1);
    assert_eq!(
        s.event(&f, true, "lan0", &h).unwrap().path.gate,
        path::Gate::FilterDrop
    );
    h.destination = "fe80::fe".parse().unwrap();
    assert_eq!(
        s.event(&f, true, "lan0", &h).unwrap().path.gate,
        path::Gate::ScopeBound
    );
    assert!(s.connections.entries.is_empty());
    s.filter
        .command("ip6tables -A INPUT -i lan0 -p ipv6-icmp --icmpv6-type 128/0 -j ACCEPT")
        .unwrap();
    h.source = "fe80::55".parse().unwrap();
    h.destination = "fe80::1".parse().unwrap();
    h.icmp_type = Some(128);
    h.icmp_code = Some(0);
    let t = s.event(&f, true, "lan0", &h).unwrap();
    assert_eq!(
        (t.path.gate, t.path.chain),
        (path::Gate::Allowed, Some(filter::Chain::Input))
    );
    assert!(s.connections.entries.is_empty());
}
#[test]
fn linux_v6_flow_capacity_failure_has_an_explicit_non_success_gate() {
    let mut s = state();
    let f = facts();
    let h = header();
    for i in 0..64 {
        let mut q = h.clone();
        q.source_port = Some(40000 + i);
        assert_eq!(
            s.event(&f, true, "lan0", &q).unwrap().path.gate,
            path::Gate::Allowed
        );
    }
    let before = s.clone();
    let mut q = h.clone();
    q.source_port = Some(50000);
    let t = s.event(&f, true, "lan0", &q).unwrap();
    assert_eq!(t.path.gate, path::Gate::TrackingRefused);
    assert!(t.tracking_refused.is_some());
    assert_eq!(t.connection, None);
    assert_eq!(s, before);
    assert_eq!(
        s.event(&f, true, "wan0", &reverse(&h)).unwrap().path.gate,
        path::Gate::Allowed
    );
}
#[test]
fn linux_v6_flow_malformed_events_and_saved_state_fail_atomically_and_roundtrip() {
    let mut s = state();
    let f = facts();
    let h = header();
    s.event(&f, true, "lan0", &h).unwrap();
    s.event(&f, true, "wan0", &reverse(&h)).unwrap();
    let restored: State = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    restored.validate().unwrap();
    for bad in ["::", "::1", "ff02::1", "fe80::55"] {
        let mut q = h.clone();
        q.source = bad.parse().unwrap();
        let before = s.clone();
        assert!(s.event(&f, true, "lan0", &q).is_err(), "{bad}");
        assert_eq!(s, before);
    }
    let mut q = h;
    q.source_port = None;
    let before = s.clone();
    assert!(s.event(&f, true, "lan0", &q).is_err());
    assert_eq!(s, before);
    let before = s.clone();
    assert!(s.event(&f, true, "unknown0", &header()).is_err());
    assert_eq!(s, before);
    let mut bad = f;
    bad.links[0].prefix = 129;
    assert!(s.event(&bad, true, "lan0", &header()).is_err());
    assert_eq!(s, before);
}
