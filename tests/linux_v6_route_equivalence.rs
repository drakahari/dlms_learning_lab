use dlms_learning_lab::{
    linux_v6_filter::Cidr,
    linux_v6_path::{Facts, Link, Route},
    linux_v6_route_equivalence::compare,
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
        routes: vec![route("2001:db8:60::/64", "wan0", 10)],
    }
}
fn route(prefix: &str, device: &str, metric: u32) -> Route {
    Route {
        destination: Cidr::parse(prefix).unwrap(),
        device: device.into(),
        via: Some("fe80::fe".parse().unwrap()),
        metric,
    }
}
#[test]
fn identical_paths_split_prefixes_and_irrelevant_metrics_are_equivalent() {
    let expected = facts();
    assert_eq!(compare(&expected, &expected).unwrap(), None);
    let mut actual = expected.clone();
    actual.routes = vec![
        route("2001:db8:60::/65", "wan0", 77),
        route("2001:db8:60:0:8000::/65", "wan0", 41),
    ];
    assert_eq!(compare(&actual, &expected).unwrap(), None);
    // A different interface identifier in the same connected prefix is legitimate.
    actual.links[0].address = "fd20:30:40::ab".parse().unwrap();
    assert_eq!(compare(&actual, &expected).unwrap(), None);
}
#[test]
fn displayed_host_routes_cannot_fake_full_remote_prefix_coverage() {
    let expected = facts();
    let mut actual = expected.clone();
    actual.routes = vec![
        route("2001:db8:60::42/128", "wan0", 10),
        route("2001:db8:60::99/128", "wan0", 10),
    ];
    let d = compare(&actual, &expected).unwrap().unwrap();
    assert_eq!(
        d.address,
        "2001:db8:60::".parse::<std::net::Ipv6Addr>().unwrap()
    );
    assert!(d.actual.is_none());
    assert_eq!(d.expected.unwrap().0, "wan0");
}
#[test]
fn extra_default_route_and_connected_prefix_broadening_are_detected() {
    let expected = facts();
    let mut actual = expected.clone();
    actual.routes.push(route("::/0", "wan0", 500));
    let d = compare(&actual, &expected).unwrap().unwrap();
    assert_eq!(d.address, std::net::Ipv6Addr::UNSPECIFIED);
    assert!(d.expected.is_none());
    let mut actual = expected.clone();
    actual.links[0].prefix = 48;
    assert!(compare(&actual, &expected).unwrap().is_some());
}
#[test]
fn more_specific_diversion_wrong_scoped_device_and_next_hop_are_not_equivalent() {
    let expected = facts();
    for altered in [
        route("2001:db8:60::42/128", "lan0", 1),
        route("2001:db8:60::/64", "lan0", 10),
        Route {
            via: Some("fe80::ff".parse().unwrap()),
            ..route("2001:db8:60::/64", "wan0", 10)
        },
    ] {
        let mut actual = expected.clone();
        if altered.destination.prefix == 128 {
            actual.routes.push(altered)
        } else {
            actual.routes = vec![altered]
        };
        assert!(compare(&actual, &expected).unwrap().is_some());
    }
    // Merely leaving a lower-preference obsolete row is equivalent if it never wins.
    let mut actual = expected.clone();
    actual.routes.push(route("2001:db8:60::/64", "lan0", 99));
    assert!(compare(&actual, &expected).unwrap().is_none());
}
#[test]
fn numeric_interval_comparison_handles_zero_and_last_u128_without_overflow() {
    let mut expected = facts();
    expected.routes = vec![route("::/0", "wan0", 1)];
    let mut actual = expected.clone();
    actual.routes.push(route(
        "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff/128",
        "lan0",
        1,
    ));
    let d = compare(&actual, &expected).unwrap().unwrap();
    assert_eq!(u128::from(d.address), u128::MAX);
    // Numeric route-selector boundary evidence, not multicast delivery acceptance.
    let mut actual = expected.clone();
    actual.routes.push(route("::/128", "lan0", 1));
    assert_eq!(
        compare(&actual, &expected).unwrap().unwrap().address,
        std::net::Ipv6Addr::UNSPECIFIED
    );
}
#[test]
fn invalid_equal_preference_or_noncanonical_profiles_refuse_instead_of_guessing() {
    let expected = facts();
    let mut actual = expected.clone();
    actual.routes.push(route("2001:db8:60::/64", "lan0", 10));
    assert!(compare(&actual, &expected).is_err());
    let mut actual = expected.clone();
    actual.routes[0].destination.network = "2001:db8:60::42".parse().unwrap();
    assert!(compare(&actual, &expected).is_err());
    let mut actual = expected.clone();
    actual.routes[0].device = "../wan0".into();
    assert!(compare(&actual, &expected).is_err());
}
