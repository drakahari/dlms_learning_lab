//! Development fixtures; no host routes, sysctls, sockets or iptables.
use dlms_learning_lab::{
    linux_filter::{Chain, Cidr, Class, Protocol, Table, Verdict},
    linux_router_path::*,
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
        routes: vec![Route {
            destination: Cidr::parse("0.0.0.0/0").unwrap(),
            device: "wan0".into(),
            via: Some("198.51.100.254".parse().unwrap()),
            metric: 10,
        }],
    }
}
fn header() -> Header {
    Header {
        source: "10.20.30.55".parse().unwrap(),
        destination: "203.0.113.9".parse().unwrap(),
        protocol: Protocol::Tcp,
        source_port: Some(40000),
        destination_port: Some(443),
    }
}
fn router() -> Runtime {
    Runtime {
        filter: Table::new(Verdict::Drop),
        forwarding: false,
    }
}
#[test]
fn forwarding_a_route_and_a_filter_verdict_are_separate_gates() {
    let f = facts();
    let h = header();
    let origin = Origin::Incoming("lan0".into());
    let mut r = router();
    r.filter
        .command("iptables -A INPUT -p tcp --dport 443 -j ACCEPT")
        .unwrap();
    assert_eq!(
        r.evaluate(&f, &origin, &h).unwrap().gate,
        Gate::ForwardingDisabled
    );
    r.forwarding = true;
    let trace = r.evaluate(&f, &origin, &h).unwrap();
    assert_eq!(trace.gate, Gate::FilterDrop);
    assert_eq!(trace.chain, Some(Chain::Forward));
    assert_eq!(trace.rule, None);
    r.filter
        .command(
            "iptables -A FORWARD -i lan0 -o wan0 -s 10.20.30.0/24 -p tcp --dport 443 -j ACCEPT",
        )
        .unwrap();
    let trace = r.evaluate(&f, &origin, &h).unwrap();
    assert_eq!(trace.gate, Gate::Allowed);
    assert_eq!(trace.rule, Some(1));
    assert_eq!(
        trace.route.unwrap().via,
        Some("198.51.100.254".parse().unwrap())
    );
    let mut no_route = f.clone();
    no_route.routes.clear();
    assert_eq!(
        r.evaluate(&no_route, &origin, &h).unwrap().gate,
        Gate::NoRoute
    );
    let mut bad = h.clone();
    bad.destination_port = Some(22);
    assert_eq!(
        r.evaluate(&f, &origin, &bad).unwrap().gate,
        Gate::FilterDrop
    );
}
#[test]
fn local_input_and_local_output_do_not_require_ip_forwarding() {
    let f = facts();
    let mut r = router();
    r.filter
        .command("iptables -A INPUT -i lan0 -p tcp --dport 22 -j ACCEPT")
        .unwrap();
    r.filter
        .command("iptables -A OUTPUT -o wan0 -p tcp --dport 443 -j ACCEPT")
        .unwrap();
    let mut h = header();
    h.destination = "10.20.30.1".parse().unwrap();
    h.destination_port = Some(22);
    let trace = r
        .evaluate(&f, &Origin::Incoming("lan0".into()), &h)
        .unwrap();
    assert_eq!(trace.gate, Gate::Allowed);
    assert_eq!(trace.chain, Some(Chain::Input));
    assert!(trace.route.is_none());
    h = header();
    h.source = "198.51.100.1".parse().unwrap();
    let trace = r.evaluate(&f, &Origin::Local, &h).unwrap();
    assert_eq!(trace.gate, Gate::Allowed);
    assert_eq!(trace.chain, Some(Chain::Output));
    h.source = "10.20.30.55".parse().unwrap();
    assert_eq!(
        r.evaluate(&f, &Origin::Local, &h).unwrap_err().class,
        Class::Context
    );
}
#[test]
fn longest_prefix_wins_before_lower_metric_and_equal_preference_is_not_guessed() {
    let mut f = facts();
    let r = Runtime {
        filter: Table::new(Verdict::Accept),
        forwarding: true,
    };
    let h = header();
    let o = Origin::Incoming("lan0".into());
    f.routes.push(Route {
        destination: Cidr::parse("203.0.113.0/24").unwrap(),
        device: "wan0".into(),
        via: Some("198.51.100.253".parse().unwrap()),
        metric: 100,
    });
    assert_eq!(r.evaluate(&f, &o, &h).unwrap().route.unwrap().metric, 100);
    f.routes.push(Route {
        destination: Cidr::parse("203.0.113.9/32").unwrap(),
        device: "lan0".into(),
        via: Some("10.20.30.99".parse().unwrap()),
        metric: 999,
    });
    assert_eq!(
        r.evaluate(&f, &o, &h).unwrap().route.unwrap().device,
        "lan0"
    );
    let mut ambiguous = f.clone();
    ambiguous.routes.push(Route {
        destination: Cidr::parse("203.0.113.9/32").unwrap(),
        device: "wan0".into(),
        via: Some("198.51.100.253".parse().unwrap()),
        metric: 999,
    });
    assert_eq!(
        r.evaluate(&ambiguous, &o, &h).unwrap_err().class,
        Class::Context
    );
}
#[test]
fn down_links_invalid_facts_and_unsupported_headers_do_not_become_success() {
    let r = Runtime {
        filter: Table::new(Verdict::Accept),
        forwarding: true,
    };
    let f = facts();
    let h = header();
    let o = Origin::Incoming("lan0".into());
    let mut down = f.clone();
    down.links[0].up = false;
    assert_eq!(r.evaluate(&down, &o, &h).unwrap().gate, Gate::IngressDown);
    down = f.clone();
    down.links[1].up = false;
    assert_eq!(r.evaluate(&down, &o, &h).unwrap().gate, Gate::EgressDown);
    let mut wrong = f.clone();
    wrong.routes[0].device = "unknown0".into();
    assert!(wrong.validate().is_err());
    wrong = f.clone();
    wrong.routes[0].via = Some("192.0.2.1".parse().unwrap());
    assert!(wrong.validate().is_err());
    wrong = f.clone();
    wrong.routes.push(wrong.routes[0].clone());
    assert!(wrong.validate().is_err());
    for ip in [
        "0.0.0.0",
        "127.0.0.1",
        "224.0.0.1",
        "255.255.255.255",
        "169.254.1.1",
    ] {
        let mut unsupported = h.clone();
        unsupported.destination = ip.parse().unwrap();
        assert_eq!(
            r.evaluate(&f, &o, &unsupported).unwrap_err().class,
            Class::Unsupported
        );
    }
}
