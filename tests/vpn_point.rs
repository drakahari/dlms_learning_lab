//! Production-linked tests for the finite peer model; no host OpenVPN execution.
use dlms_learning_lab::vpn_point::*;
#[path = "support/vpn_point_fixture.rs"]
mod fixture;
use fixture::{configs, peer, supplied};
#[test]
fn full_supported_pair_is_reachable_and_required_lan_routes_are_mathematical() {
    let s = supplied();
    let c = configs();
    assert!(peer_checks(&s, &c).unwrap().iter().all(|r| r.passed));
    assert!(
        equivalent_routes(
            &c[0],
            &[Network {
                address: "10.40.0.0".parse().unwrap(),
                prefix: 24
            }]
        )
        .unwrap()
    );
    assert!(
        equivalent_routes(
            &c[1],
            &[Network {
                address: "10.20.0.0".parse().unwrap(),
                prefix: 24
            }]
        )
        .unwrap()
    );
    assert_eq!(c[1].remote.as_ref().unwrap().port, Some(1194));
    assert_eq!(c[0].port, 1194);
}
#[test]
fn alternative_directive_order_comments_udp4_and_explicit_default_ports_preserve_meaning() {
    for server in [false, true] {
        let original = peer(server);
        let lines: Vec<_> = original.lines().rev().collect();
        let variant = format!(
            "; alternate ordinary whole-line comment\nport 1194\n{}\n",
            lines.join("\n")
        )
        .replace("proto udp\n", "proto udp4\n");
        assert_eq!(parse(&original).unwrap(), parse(&variant).unwrap());
    }
}
#[test]
fn wrong_endpoint_port_roles_and_reciprocal_tunnel_values_fail_specific_checks() {
    let s = supplied();
    let good = configs();
    for (text, label) in [
        (
            peer(false).replace("203.0.113.10 1194", "203.0.113.11 1194"),
            "remote endpoint/port",
        ),
        (
            peer(false).replace("203.0.113.10 1194", "203.0.113.10 1195"),
            "remote endpoint/port",
        ),
        (
            peer(false).replace("tls-client", "tls-server"),
            "Complementary TLS roles",
        ),
        (
            peer(false).replace("ifconfig 10.8.0.2 10.8.0.1", "ifconfig 10.8.0.2 10.8.0.5"),
            "Reciprocal tunnel addresses",
        ),
    ] {
        let mut c = good.clone();
        c[1] = parse(&text).unwrap();
        let results = peer_checks(&s, &c).unwrap();
        assert!(
            results.iter().any(|r| r.label.contains(label) && !r.passed),
            "{label}: {results:?}"
        );
    }
}
#[test]
fn supplied_trust_key_pair_name_and_certificate_purpose_are_distinct_checks() {
    let good = configs();
    let s = supplied();
    for (text, label) in [
        (
            peer(false).replace("ca /etc/openvpn/ca.crt", "ca /etc/openvpn/other-ca.crt"),
            "supplied peer validity and authority",
        ),
        (
            peer(false).replace("key /etc/openvpn/branch.key", "key /etc/openvpn/hq.key"),
            "own certificate/key",
        ),
        (
            peer(false).replace(
                "verify-x509-name hq name",
                "verify-x509-name wrong-peer name",
            ),
            "requested peer name",
        ),
        (
            peer(false).replace("remote-cert-tls server", "remote-cert-tls client"),
            "requested peer purpose",
        ),
    ] {
        let mut c = good.clone();
        c[1] = parse(&text).unwrap();
        assert!(
            peer_checks(&s, &c)
                .unwrap()
                .iter()
                .any(|r| r.label.contains(label) && !r.passed),
            "{label}"
        );
    }
    let mut invalid = s.clone();
    let Material::Certificate { valid, .. } = &mut invalid.files[2].material else {
        panic!()
    };
    *valid = false;
    assert!(
        peer_checks(&invalid, &good)
            .unwrap()
            .iter()
            .any(|r| r.label.starts_with("branch:") && !r.passed)
    );
}
#[test]
fn split_and_overlapping_equivalent_routes_pass_but_samples_broad_defaults_and_one_way_repairs_do_not()
 {
    let mut c = configs()[0].clone();
    let goal = [Network {
        address: "10.40.0.0".parse().unwrap(),
        prefix: 24,
    }];
    let n = |a: &str, p| Network {
        address: a.parse().unwrap(),
        prefix: p,
    };
    for routes in [
        vec![n("10.40.0.0", 25), n("10.40.0.128", 25)],
        vec![n("10.40.0.0", 24), n("10.40.0.64", 26)],
    ] {
        c.routes = routes;
        assert!(equivalent_routes(&c, &goal).unwrap());
    }
    for routes in [
        vec![],
        vec![n("10.40.0.7", 32)],
        vec![n("10.40.0.0", 25)],
        vec![n("10.40.0.0", 23)],
        vec![n("0.0.0.0", 0)],
    ] {
        c.routes = routes;
        assert!(!equivalent_routes(&c, &goal).unwrap());
    }
    let other = configs()[1].clone();
    assert!(!equivalent_routes(&other, &goal).unwrap());
}
#[test]
fn route_boundary_math_handles_every_prefix_and_both_numeric_extremes_without_overflow() {
    for prefix in 0..=32 {
        for address in [0u32, 1, 0x0a208b02, u32::MAX] {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            let n = Network {
                address: (address & mask).into(),
                prefix,
            };
            let (a, b) = n.interval().unwrap();
            assert!(a <= address && address <= b);
            assert_eq!(u64::from(b) - u64::from(a) + 1, 1u64 << (32 - prefix));
        }
    }
    assert!(
        Network {
            address: "10.40.0.1".parse().unwrap(),
            prefix: 24
        }
        .validate()
        .is_err()
    );
}
#[test]
fn unsupported_directives_scripts_includes_escape_paths_and_real_targets_are_never_ignored() {
    for suffix in [
        "up /tmp/do-something",
        "down /etc/secret",
        "plugin /tmp/plugin.so",
        "script-security 2",
        "config /etc/passwd",
        "route-up /tmp/route",
        "management 127.0.0.1 7505",
        "push \"route 10.4.0.0 255.255.0.0\"",
        "remote-cert-tls server # note",
    ] {
        let e = parse(&(peer(false) + suffix + "\n")).unwrap_err();
        assert_eq!(e.class, Class::Unsupported, "{suffix}");
        assert!(e.line.is_some());
    }
    for path in [
        "/etc/passwd",
        "/etc/openvpn/../secret",
        "/etc/openvpn//ca.crt",
        "/etc/openvpn/./ca.crt",
    ] {
        let text = peer(false).replace("/etc/openvpn/ca.crt", path);
        assert_eq!(
            parse(&text).unwrap_err().class,
            Class::Unsupported,
            "{path}"
        );
    }
}
#[test]
fn malformed_and_unsupported_configurations_have_honest_distinct_diagnostics() {
    for suffix in [
        "port 0",
        "port 65536",
        "ifconfig 10.8.0.2",
        "route 10.4.0.0 255.0.255.0",
    ] {
        let mut t = peer(false);
        if suffix.starts_with("port") {
            t.push_str(suffix);
            t.push('\n');
        } else {
            t = if suffix.starts_with("ifconfig") {
                t.lines()
                    .filter(|l| !l.starts_with("ifconfig"))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                t
            };
            t.push('\n');
            t.push_str(suffix);
        }
        assert_eq!(parse(&t).unwrap_err().class, Class::Syntax, "{suffix}");
    }
    assert_eq!(
        parse(&peer(false).replace("proto udp", "proto tcp-client"))
            .unwrap_err()
            .class,
        Class::Unsupported
    );
    assert_eq!(
        parse(&peer(false).replace("dev tun", "dev tap"))
            .unwrap_err()
            .class,
        Class::Unsupported
    );
    assert_eq!(parse(&"x".repeat(8193)).unwrap_err().class, Class::Limit);
    assert_eq!(
        parse(&peer(false).replace("tls-client", "tls-client\ntls-server"))
            .unwrap_err()
            .class,
        Class::Configuration
    );
}
#[test]
fn forged_typed_values_and_unknown_virtual_files_are_refused_without_host_lookup() {
    let s = supplied();
    let mut c = configs();
    c[1].port = 0;
    assert!(peer_checks(&s, &c).is_err());
    c = configs();
    c[1].remote = None;
    assert!(peer_checks(&s, &c).is_err());
    c = configs();
    c[1].cert = "/etc/openvpn/absent.crt".into();
    assert!(
        peer_checks(&s, &c)
            .unwrap_err()
            .message
            .contains("no host lookup")
    );
    let mut d = s.clone();
    d.files.push(d.files[0].clone());
    assert!(d.validate().is_err());
    d = s;
    d.files[0].path = "/etc/openvpn/../outside".into();
    assert!(d.validate().is_err());
}

#[test]
fn absent_optional_identity_requests_are_not_reported_as_verified() {
    let mut c = configs();
    c[1].expected_role = None;
    c[1].expected_name = None;
    let checks = peer_checks(&supplied(), &c).unwrap();
    let missing: Vec<_> = checks.iter().filter(|r| !r.evaluated).collect();
    assert_eq!(missing.len(), 2);
    assert!(missing.iter().all(|r| !r.passed));
    assert!(checks.iter().filter(|r| r.evaluated).all(|r| r.passed));
}
