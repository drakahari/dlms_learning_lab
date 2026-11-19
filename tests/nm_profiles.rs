use dlms_learning_lab::nm_profiles::{self as n, ErrorKind};
use std::net::Ipv4Addr;
const BASIC: &str = "[connection]\nid=Example profile\nuuid=00000000-0000-4000-8000-000000000001\ntype=ethernet\ninterface-name=enp1s0\n[ipv4]\nmethod=manual\naddress1=10.20.0.20/24\ngateway=10.20.0.1\nroute-metric=100\n[ipv6]\nmethod=disabled\n";
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
#[test]
fn profile_properties_and_actual_documented_defaults_are_typed_data() {
    let p = n::parse(BASIC).unwrap();
    assert_eq!(p.id, "Example profile");
    assert_eq!(p.interface, "enp1s0");
    assert!(p.autoconnect);
    assert_eq!(p.priority, 0);
    assert_eq!(p.address.address, ip("10.20.0.20"));
    assert_eq!(p.address.prefix, 24);
    assert_eq!(p.gateway, Some(ip("10.20.0.1")));
    assert_eq!(p.route_metric, Some(100));
    assert!(!p.never_default);
    assert!(p.routes.is_empty());
    assert_eq!(
        n::parse(&BASIC.replace("route-metric=100\n", ""))
            .unwrap()
            .route_metric,
        None
    );
    assert_eq!(
        n::parse(&BASIC.replace("route-metric=100", "route-metric=-1"))
            .unwrap()
            .route_metric,
        None
    );
    assert_eq!(
        n::parse(&BASIC.replace("route-metric=100", "route-metric=0"))
            .unwrap()
            .route_metric,
        Some(0)
    );
}
#[test]
fn genuine_offline_reference_normalization_matches_all_supported_fixture_properties() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nm-profiles-reference.json")).unwrap();
    assert_eq!(v["cases"].as_array().unwrap().len(), 9);
    let (mut accepted, mut rejected) = (0, 0);
    for c in v["cases"].as_array().unwrap() {
        let raw = c["input"].as_str().unwrap();
        if c["reference_status"] == 0 {
            accepted += 1;
            let mut model = n::parse(raw).unwrap();
            model.id = "offline-reference-fixture".into();
            let normalized = n::parse(c["normalized"].as_str().unwrap()).unwrap();
            assert_eq!(model, normalized, "{}", c["id"]);
        } else {
            rejected += 1;
            assert_eq!(c["id"], "empty-override");
            assert!(
                c["diagnostic"]
                    .as_str()
                    .unwrap()
                    .contains("gateway is invalid")
            );
            assert_eq!(n::parse(raw).unwrap_err().kind, ErrorKind::Invalid);
        }
    }
    assert_eq!((accepted, rejected), (8, 1));
}
#[test]
fn supported_whitespace_order_empty_sections_and_repeated_keys_keep_actual_values() {
    let mut text = BASIC.replace(
        "interface-name=enp1s0",
        "interface-name=old0\ninterface-name=enp1s0",
    );
    text.push_str("\n[connection]\n# repeated group\nautoconnect=false\nautoconnect-priority=-9\n[ethernet]\n[proxy]\n");
    let p = n::parse(&text).unwrap();
    assert_eq!(p.interface, "enp1s0");
    assert!(!p.autoconnect);
    assert_eq!(p.priority, -9);
    assert_eq!(
        n::parse(&BASIC.replace("type=ethernet", "type=802-3-ethernet")).unwrap(),
        n::parse(BASIC).unwrap()
    );
    assert_eq!(
        n::parse(&BASIC.replace('=', " = ").replace('\n', "\r\n")).unwrap(),
        n::parse(BASIC).unwrap()
    );
}
#[test]
fn gateway_precedence_zero_and_never_default_are_not_silently_guessed() {
    for sep in [",", ";"] {
        let x = BASIC.replace(
            "address1=10.20.0.20/24",
            &format!("address1=10.20.0.20/24{sep}10.20.0.254"),
        );
        assert_eq!(n::parse(&x).unwrap().gateway, Some(ip("10.20.0.1")));
        assert_eq!(
            n::parse(&x.replace("gateway=10.20.0.1\n", ""))
                .unwrap()
                .gateway,
            Some(ip("10.20.0.254"))
        );
    }
    assert!(
        n::parse(&BASIC.replace("gateway=10.20.0.1", "gateway=0.0.0.0"))
            .unwrap()
            .gateway
            .is_none()
    );
    let p = n::parse(&BASIC.replace("[ipv4]", "[ipv4]\nnever-default=true")).unwrap();
    assert!(p.never_default);
    assert!(p.gateway.is_none());
    assert_eq!(
        n::parse(&BASIC.replace("gateway=10.20.0.1", "gateway="))
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
}
#[test]
fn route_fields_normalize_real_network_boundaries_and_retain_zero_or_inherited_metrics() {
    let x=BASIC.replace("[ipv6]","route1=172.22.1.55/16,10.20.0.1,0\nroute3=0.0.0.0/0,10.20.0.1\nroute8=10.30.1.55/32,10.20.0.1,45\n[ipv6]");
    let p = n::parse(&x).unwrap();
    assert_eq!(p.routes.len(), 3);
    assert_eq!(p.routes[0].destination.address, ip("172.22.0.0"));
    assert_eq!(p.routes[0].metric, Some(0));
    assert_eq!(p.routes[1].destination.length, 0);
    assert_eq!(p.routes[1].metric, None);
    assert_eq!(p.routes[2].destination.length, 32);
}
#[test]
fn unsupported_settings_methods_and_infrastructure_are_never_ignored() {
    for fragment in [
        "[wifi]\nssid=personal",
        "[ipv4]\naddress2=10.20.0.21/24",
        "[ipv4]\ndns=10.20.0.53;",
        "[ipv4]\nroute01=10.30.0.0/16,10.20.0.1",
        "[ipv4]\nroute9=10.30.0.0/16,10.20.0.1",
        "[ethernet]\nmac-address=00:11:22:33:44:55",
        "[connection]\nautoconnect-retries=3",
        "[Connection]\nid=wrong",
    ] {
        let e = n::parse(&format!("{BASIC}\n{fragment}\n")).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unsupported, "{fragment}: {e}");
    }
    for (a, b) in [
        ("method=manual", "method=auto"),
        ("method=disabled", "method=auto"),
        ("type=ethernet", "type=wifi"),
        ("interface-name=enp1s0", "interface-name=en*"),
        ("id=Example profile", "id=bad\\sescape"),
    ] {
        assert_eq!(
            n::parse(&BASIC.replace(a, b)).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn syntax_invalid_values_and_explicit_subset_limits_have_useful_distinct_diagnostics() {
    for x in [
        "key=value\n",
        "[connection\nid=x\n",
        "[connection]\nnoequals",
    ] {
        let e = n::parse(x).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Syntax);
        assert!(e.line > 0);
        assert!(e.to_string().contains("line"));
    }
    for (a, b) in [
        ("id=Example profile", "id="),
        ("uuid=00000000-0000-4000-8000-000000000001", "uuid=bad"),
        ("10.20.0.20/24", "10.20.0.0/24"),
        ("gateway=10.20.0.1", "gateway=300.20.0.1"),
    ] {
        assert_eq!(
            n::parse(&BASIC.replace(a, b)).unwrap_err().kind,
            ErrorKind::Invalid
        );
    }
    for p in [0, 31, 32, 33] {
        assert_eq!(
            n::parse(&BASIC.replace("10.20.0.20/24", &format!("10.20.0.20/{p}")))
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
    assert_eq!(
        n::parse(&BASIC.replace("10.20.0.20/24", "8.8.8.8/24"))
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    for k in ["uuid", "interface-name"] {
        let x = BASIC
            .lines()
            .filter(|line| !line.starts_with(&format!("{k}=")))
            .collect::<Vec<_>>()
            .join("\n");
        let e = n::parse(&x).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unsupported);
        assert!(e.message.contains(k));
    }
    assert!(n::parse(&BASIC.replace("10.20.0.20/24", "172.18.17.20/20")).is_ok());
}
#[test]
fn numeric_bounds_and_input_budgets_fail_before_unbounded_storage() {
    for n in [-999, 999] {
        let x = BASIC.replace("[ipv4]", &format!("autoconnect-priority={n}\n[ipv4]"));
        assert_eq!(n::parse(&x).unwrap().priority, n);
    }
    for n in [-1000, 1000, 32768] {
        let x = BASIC.replace("[ipv4]", &format!("autoconnect-priority={n}\n[ipv4]"));
        assert_eq!(n::parse(&x).unwrap_err().kind, ErrorKind::Invalid);
    }
    let mut x = BASIC.to_string();
    x.push('#');
    x.extend(std::iter::repeat_n('x', n::MAX_TEXT - x.len()));
    assert!(n::parse(&x).is_ok());
    x.push('x');
    assert_eq!(n::parse(&x).unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(
        n::parse(&format!("{}{}", BASIC, "\n".repeat(n::MAX_LINES)))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        n::parse(&BASIC.replace("route-metric=100", "route-metric=4294967296"))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
#[test]
fn code_shaped_values_remain_inert_data_and_controls_are_refused_even_in_comments() {
    let p =
        n::parse(&BASIC.replace("id=Example profile", "id=$(id); `whoami` /etc/shadow")).unwrap();
    assert!(p.id.contains("$(id)"));
    for c in ['\0', '\u{1b}', '\u{7f}'] {
        assert_eq!(
            n::parse(&format!("# comment {c}\n{BASIC}"))
                .unwrap_err()
                .kind,
            ErrorKind::Syntax
        );
    }
    assert_eq!(
        n::parse(&format!("# café\n{BASIC}")).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
