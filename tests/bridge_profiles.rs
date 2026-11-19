//! Source-enabled bridge-controller keyfiles; no activation/runtime claim.
use dlms_learning_lab::nm_profiles::{self as n, Addressing, ErrorKind};
const NORMALIZED: &str = "[connection]\nid=offline-controller\nuuid=30000000-0000-4000-8000-000000000003\ntype=bridge\nautoconnect-ports=1\ninterface-name=lab-bridge\n\n[ethernet]\n\n[bridge]\n\n[ipv4]\naddress1=10.200.0.20/24\nmethod=manual\n\n[ipv6]\naddr-gen-mode=default\nmethod=disabled\n\n[proxy]\n";
#[test]
fn observed_default_shape_keeps_bridge_role_manual_address_and_nm_stp_default() {
    let p = n::parse_bridge_settings(NORMALIZED).unwrap();
    assert!(p.stp && p.autoconnect_ports && p.ipv4.autoconnect);
    assert_eq!(p.ipv4.interface, "lab-bridge");
    assert!(p.ipv4.vlan.is_none());
    let Addressing::Manual { address, gateway } = &p.ipv4.addressing else {
        panic!()
    };
    assert_eq!(address.address.to_string(), "10.200.0.20");
    assert_eq!(address.prefix, 24);
    assert!(gateway.is_none());
}
#[test]
fn existing_apis_stay_strict_and_bridge_api_does_not_swallow_other_roles() {
    assert_eq!(
        n::parse_settings(NORMALIZED).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        n::parse_vlan_settings(NORMALIZED).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        n::parse(NORMALIZED).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    for ty in ["ethernet", "vlan", "bond"] {
        assert_eq!(
            n::parse_bridge_settings(&NORMALIZED.replace("type=bridge", &format!("type={ty}")))
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn stp_false_and_explicit_unfiltered_are_supported_without_inventing_election() {
    let text = NORMALIZED
        .replace("[bridge]", "[bridge]\nstp=false\nvlan-filtering=false")
        .replace("autoconnect-ports=1", "autoconnect-ports=0");
    let p = n::parse_bridge_settings(&text).unwrap();
    assert!(!p.stp && !p.autoconnect_ports);
    let changed = text.replace(
        "autoconnect-ports=0",
        "autoconnect-ports=1\nautoconnect=false",
    );
    let p = n::parse_bridge_settings(&changed).unwrap();
    assert!(p.autoconnect_ports && !p.ipv4.autoconnect);
}
#[test]
fn repeated_groups_and_keys_use_last_supported_values() {
    let text = NORMALIZED.to_string()
        + "\n[bridge]\nstp=false\n[bridge]\nstp=true\n[connection]\nautoconnect-ports=0\nautoconnect-ports=1\n";
    let p = n::parse_bridge_settings(&text).unwrap();
    assert!(p.stp && p.autoconnect_ports);
}
#[test]
fn inherited_port_policy_is_diagnosed_instead_of_guessed() {
    for v in ["-1", "true", "2", ""] {
        let text = NORMALIZED.replace("autoconnect-ports=1", &format!("autoconnect-ports={v}"));
        let e = n::parse_bridge_settings(&text).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unsupported);
        assert!(e.message.contains("autoconnect-ports"));
    }
    assert!(
        n::parse_bridge_settings(&NORMALIZED.replace("autoconnect-ports=1\n", ""))
            .unwrap_err()
            .message
            .contains("autoconnect-ports")
    );
}
#[test]
fn unsupported_vlan_filtering_timers_port_or_nested_controller_properties_do_not_pass() {
    for text in [
        NORMALIZED.replace("[bridge]", "[bridge]\nvlan-filtering=true"),
        NORMALIZED.replace("[bridge]", "[bridge]\nforward-delay=15"),
        NORMALIZED.replace("[bridge]", "[bridge-port]"),
        NORMALIZED.replace("[bridge]", "[bridge]\nmulticast-snooping=false"),
        NORMALIZED.replace("type=bridge", "type=bridge\ncontroller=bond0"),
        NORMALIZED.replace(
            "[ethernet]",
            "[ethernet]\ncloned-mac-address=00:11:22:33:44:55",
        ),
    ] {
        assert_eq!(
            n::parse_bridge_settings(&text).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn dhcp_shared_and_ipv6_configuration_are_explicitly_unsupported() {
    for method in ["auto", "shared", "link-local", "disabled"] {
        let text = NORMALIZED.replace("method=manual", &format!("method={method}"));
        assert_eq!(
            n::parse_bridge_settings(&text).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    for text in [
        NORMALIZED.replace("method=disabled", "method=auto"),
        NORMALIZED.replace("addr-gen-mode=default", "addr-gen-mode=stable-privacy"),
    ] {
        assert_eq!(
            n::parse_bridge_settings(&text).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn manual_route_and_preservation_properties_retain_the_existing_typed_semantics() {
    let text=NORMALIZED.replace("method=manual","method=manual\ngateway=10.200.0.1\nroute-metric=75\nnever-default=true\nroute1=10.210.0.0/24,10.200.0.1,80");
    let p = n::parse_bridge_settings(&text).unwrap();
    assert_eq!(p.ipv4.route_metric, Some(75));
    assert!(p.ipv4.never_default);
    assert_eq!(p.ipv4.routes.len(), 1);
    let Addressing::Manual { gateway, .. } = p.ipv4.addressing else {
        panic!()
    };
    assert!(gateway.is_none());
    assert_eq!(p.ipv4.routes[0].via.to_string(), "10.200.0.1");
    assert_eq!(p.ipv4.routes[0].metric, Some(80));
}
#[test]
fn malformed_and_oversized_input_have_honest_bounded_diagnostics() {
    for (text, kind) in [
        (NORMALIZED.replace("[bridge]", "[bridge"), ErrorKind::Syntax),
        (
            NORMALIZED.replace("address1=10.200.0.20/24", "address1=not-an-address"),
            ErrorKind::Unsupported,
        ),
        (
            NORMALIZED.replace("[bridge]", "[bridge]\nstp=maybe"),
            ErrorKind::Unsupported,
        ),
        (NORMALIZED.to_string() + &"\n".repeat(257), ErrorKind::Limit),
        (
            NORMALIZED.to_string() + &"#".repeat(16384),
            ErrorKind::Limit,
        ),
        (
            NORMALIZED.to_string() + "\n# hidden\0command",
            ErrorKind::Syntax,
        ),
    ] {
        assert_eq!(n::parse_bridge_settings(&text).unwrap_err().kind, kind);
    }
}
