//! Fixed simulated facts only: no kernel routes, neighbors, sysctls or packets.
use dlms_learning_lab::{
    linux_v6_filter::{Chain, Cidr, Class, Protocol, Table, Verdict},
    linux_v6_path::*,
};
fn facts() -> Facts {
    Facts {
        links: vec![
            Link {
                name: "lan0".into(),
                address: "fd20:30:40::1".parse().unwrap(),
                link_local: "fe80::1".parse().unwrap(),
                prefix: 64,
                up: true,
            },
            Link {
                name: "wan0".into(),
                address: "2001:db8:50::1".parse().unwrap(),
                link_local: "fe80::1".parse().unwrap(),
                prefix: 64,
                up: true,
            },
        ],
        routes: vec![Route {
            destination: Cidr::parse("::/0").unwrap(),
            device: "wan0".into(),
            via: Some("fe80::fe".parse().unwrap()),
            metric: 10,
        }],
    }
}
fn header() -> Header {
    Header {
        source: "fd20:30:40::55".parse().unwrap(),
        destination: "2001:db8:60::42".parse().unwrap(),
        protocol: Protocol::Tcp,
        source_port: Some(40000),
        destination_port: Some(443),
        icmp_type: None,
        icmp_code: None,
    }
}
#[test]
fn linux_v6_path_forwarding_route_and_filter_are_independent() {
    let f = facts();
    let h = header();
    let o = Origin::Incoming("lan0".into());
    let mut r = Runtime {
        filter: Table::new(Verdict::Drop),
        forwarding: false,
    };
    r.filter
        .command("ip6tables -A INPUT -p tcp --dport 443 -j ACCEPT")
        .unwrap();
    assert_eq!(
        r.evaluate(&f, &o, &h).unwrap().gate,
        Gate::ForwardingDisabled
    );
    r.forwarding = true;
    let t = r.evaluate(&f, &o, &h).unwrap();
    assert_eq!(
        (t.gate, t.chain, t.rule),
        (Gate::FilterDrop, Some(Chain::Forward), None)
    );
    r.filter
        .command(
            "ip6tables -A FORWARD -i lan0 -o wan0 -s fd20:30:40::/64 -p tcp --dport 443 -j ACCEPT",
        )
        .unwrap();
    let t = r.evaluate(&f, &o, &h).unwrap();
    assert_eq!((t.gate, t.rule), (Gate::Allowed, Some(1)));
    let route = t.route.unwrap();
    assert_eq!(route.device, "wan0");
    assert_eq!(route.via, Some("fe80::fe".parse().unwrap()));
    let mut no_route = f.clone();
    no_route.routes.clear();
    assert_eq!(r.evaluate(&no_route, &o, &h).unwrap().gate, Gate::NoRoute);
    let mut q = h;
    q.destination_port = Some(22);
    assert_eq!(r.evaluate(&f, &o, &q).unwrap().gate, Gate::FilterDrop);
}
#[test]
fn linux_v6_path_input_output_and_local_link_local_scope_are_explicit() {
    let f = facts();
    let mut r = Runtime {
        filter: Table::new(Verdict::Drop),
        forwarding: false,
    };
    r.filter
        .command("ip6tables -A INPUT -i lan0 -p tcp --dport 22 -j ACCEPT")
        .unwrap();
    r.filter
        .command("ip6tables -A OUTPUT -o wan0 -p tcp --dport 443 -j ACCEPT")
        .unwrap();
    let mut h = header();
    h.destination = f.links[0].address;
    h.destination_port = Some(22);
    let t = r
        .evaluate(&f, &Origin::Incoming("lan0".into()), &h)
        .unwrap();
    assert_eq!(
        (t.gate, t.chain, t.route),
        (Gate::Allowed, Some(Chain::Input), None)
    );
    h = header();
    h.source = f.links[1].address;
    let t = r.evaluate(&f, &Origin::Local, &h).unwrap();
    assert_eq!((t.gate, t.chain), (Gate::Allowed, Some(Chain::Output)));
    h.source = "fd20:30:40::55".parse().unwrap();
    assert!(r.evaluate(&f, &Origin::Local, &h).is_err());
    h.source = f.links[1].address;
    h.destination = "fe80::fe".parse().unwrap();
    assert_eq!(
        r.evaluate(&f, &Origin::Local, &h).unwrap_err().class,
        Class::Unsupported
    );
    h.destination = f.links[0].address;
    assert_eq!(
        r.evaluate(&f, &Origin::Local, &h).unwrap_err().class,
        Class::Unsupported
    );
}
#[test]
fn linux_v6_path_link_local_never_crosses_router_links_even_with_accept_default() {
    let mut f = facts();
    f.links[1].link_local = "fe80::2".parse().unwrap();
    let r = Runtime {
        filter: Table::new(Verdict::Accept),
        forwarding: true,
    };
    let o = Origin::Incoming("lan0".into());
    let mut h = header();
    h.protocol = Protocol::Icmp6;
    h.source_port = None;
    h.destination_port = None;
    h.icmp_type = Some(128);
    h.icmp_code = Some(0);
    for (source, destination) in [
        ("fe80::55", "2001:db8:60::42"),
        ("fd20:30:40::55", "fe80::2"),
        ("fe80::55", "fe80::fe"),
    ] {
        h.source = source.parse().unwrap();
        h.destination = destination.parse().unwrap();
        let t = r.evaluate(&f, &o, &h).unwrap();
        assert_eq!(
            (t.gate, t.chain, t.route, t.rule),
            (Gate::ScopeBound, None, None, None)
        );
    }
    h.source = "fe80::55".parse().unwrap();
    h.destination = "fe80::1".parse().unwrap();
    let t = r.evaluate(&f, &o, &h).unwrap();
    assert_eq!((t.gate, t.chain), (Gate::Allowed, Some(Chain::Input)));
    h.destination = "fe80::2".parse().unwrap();
    let t = r
        .evaluate(&f, &Origin::Incoming("wan0".into()), &h)
        .unwrap();
    assert_eq!((t.gate, t.chain), (Gate::Allowed, Some(Chain::Input)));
}
#[test]
fn linux_v6_path_uses_all_128_bits_longest_prefix_then_metric_and_refuses_equal_choices() {
    let mut f = facts();
    let r = Runtime {
        filter: Table::new(Verdict::Accept),
        forwarding: true,
    };
    let h = header();
    let o = Origin::Incoming("lan0".into());
    f.routes.push(Route {
        destination: Cidr::parse("2001:db8:60::/64").unwrap(),
        device: "wan0".into(),
        via: Some("fe80::fd".parse().unwrap()),
        metric: 100,
    });
    assert_eq!(r.evaluate(&f, &o, &h).unwrap().route.unwrap().metric, 100);
    f.routes.push(Route {
        destination: Cidr::parse("2001:db8:60::42/128").unwrap(),
        device: "lan0".into(),
        via: Some("fd20:30:40::99".parse().unwrap()),
        metric: 999,
    });
    assert_eq!(
        r.evaluate(&f, &o, &h).unwrap().route.unwrap().device,
        "lan0"
    );
    let mut q = h;
    q.destination = "fd20:db8:60::42".parse().unwrap();
    assert_eq!(
        r.evaluate(&f, &o, &q)
            .unwrap()
            .route
            .unwrap()
            .destination
            .prefix,
        0
    );
    let mut low = f.clone();
    low.routes.push(Route {
        metric: 998,
        ..low.routes[2].clone()
    });
    assert_eq!(
        r.evaluate(&low, &o, &header())
            .unwrap()
            .route
            .unwrap()
            .metric,
        998
    );
    let mut ambiguous = f.clone();
    ambiguous.routes.push(Route {
        device: "wan0".into(),
        via: Some("fe80::fe".parse().unwrap()),
        ..f.routes[2].clone()
    });
    assert!(ambiguous.validate().is_err());
}
#[test]
fn linux_v6_path_scoped_next_hops_do_not_need_a_fake_global_gateway() {
    let f = facts();
    f.validate().unwrap(); // Repeated fe80::1 is valid on different links.
    for (device, via, valid) in [
        ("wan0", "fe80::fe", true),
        ("lan0", "fe80::fe", true),
        ("wan0", "fe80::1", false),
        ("wan0", "fd20:30:40::fe", false),
        ("lan0", "fd20:30:40::fe", true),
        ("wan0", "::", false),
        ("wan0", "ff02::1", false),
        ("wan0", "::1", false),
    ] {
        let mut q = f.clone();
        q.routes[0].device = device.into();
        q.routes[0].via = Some(via.parse().unwrap());
        assert_eq!(q.validate().is_ok(), valid, "{device} {via}");
    }
}
#[test]
fn linux_v6_path_links_header_shapes_and_saved_facts_fail_honestly() {
    let f = facts();
    let h = header();
    let r = Runtime {
        filter: Table::new(Verdict::Accept),
        forwarding: true,
    };
    let o = Origin::Incoming("lan0".into());
    for index in [0, 1] {
        let mut q = f.clone();
        q.links[index].up = false;
        assert_eq!(
            r.evaluate(&q, &o, &h).unwrap().gate,
            if index == 0 {
                Gate::IngressDown
            } else {
                Gate::EgressDown
            }
        );
    }
    assert!(
        r.evaluate(&f, &Origin::Incoming("missing0".into()), &h)
            .is_err()
    );
    for address in ["::", "::1", "ff02::1", "::ffff:192.0.2.1"] {
        let mut q = h.clone();
        q.destination = address.parse().unwrap();
        assert_eq!(
            r.evaluate(&f, &o, &q).unwrap_err().class,
            Class::Unsupported
        );
    }
    let mut q = h.clone();
    q.icmp_type = Some(1);
    assert!(r.evaluate(&f, &o, &q).is_err());
    let mut q = h;
    q.protocol = Protocol::Icmp6;
    assert!(r.evaluate(&f, &o, &q).is_err());
    let restored: Facts = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
    assert_eq!(f, restored);
    restored.validate().unwrap();
    let mut bad = f.clone();
    bad.links[1].name = bad.links[0].name.clone();
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.links[0].prefix = 129;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.routes[0].device = "missing0".into();
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.routes.push(bad.routes[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = serde_json::to_value(&f).unwrap();
    bad["links"] = serde_json::Value::Array(vec![bad["links"][0].clone(); 9]);
    assert!(serde_json::from_value::<Facts>(bad).is_err());
}
