//! Focused model checks; catalog integration and native acceptance are separate.
#[path = "support/virtual_routing_fixture.rs"]
mod setup;
use setup::fixture;
#[test]
fn accepts_overlapping_addresses_in_separate_contexts() {
    let (f, c) = fixture();
    f.validate().unwrap();
    for t in ["a", "b"] {
        let p = f
            .probe(&c, &format!("{t}-client"), &format!("{t}-server"))
            .unwrap();
        assert!(p.reached_requested);
        assert_eq!(p.actual, format!("{t}-server"));
        assert_eq!(p.routing_instances.len(), 2);
    }
}
#[test]
fn a_reply_from_the_same_numeric_address_is_not_the_requested_endpoint() {
    let (f, c) = fixture();
    let p = f.probe(&c, "a-client", "b-server").unwrap();
    assert!(!p.reached_requested);
    assert_eq!(p.actual, "a-server");
    assert_eq!(
        p.address,
        "10.0.2.20".parse::<std::net::Ipv4Addr>().unwrap()
    );
}
#[test]
fn wrong_context_membership_cannot_earn_endpoint_credit() {
    let (f, mut c) = fixture();
    for b in &mut c.bindings {
        if b.interface.ends_with("server") {
            b.context = if b.context == "blue" { "green" } else { "blue" }.into();
        }
    }
    let p = f.probe(&c, "a-client", "a-server").unwrap();
    assert!(!p.reached_requested);
    assert_eq!(p.actual, "b-server");
}
#[test]
fn equivalent_context_names_work_and_physical_failure_takes_both_down() {
    let (f, mut c) = fixture();
    for b in &mut c.bindings {
        b.context = if b.context == "blue" { "green" } else { "blue" }.into();
    }
    for t in ["a", "b"] {
        assert!(
            f.probe(&c, &format!("{t}-client"), &format!("{t}-server"))
                .unwrap()
                .reached_requested
        );
    }
    c.router_up = false;
    for t in ["a", "b"] {
        let error = f
            .probe(&c, &format!("{t}-client"), &format!("{t}-server"))
            .unwrap_err();
        assert!(error.contains("edge is unavailable"));
        assert!(!error.contains("backup"));
    }
}
#[test]
fn rejects_duplicate_address_inside_one_context_not_as_successful_isolation() {
    let (f, mut c) = fixture();
    for b in &mut c.bindings {
        b.context = "blue".into();
    }
    let e = f.project(&c).unwrap_err();
    assert!(e.contains("duplicate address"));
}
#[test]
fn rejects_missing_unknown_duplicate_and_wider_topologies() {
    let (mut f, c) = fixture();
    assert!(f.probe(&c, "vrf-0", "a-server").is_err());
    assert!(f.probe(&c, "unknown", "a-server").is_err());
    assert!(f.probe(&c, "a-client", "vrf-0").is_err());
    let mut bad = c.clone();
    bad.bindings.pop();
    assert!(f.validate_configuration(&bad).is_err());
    let mut bad = c.clone();
    bad.bindings[1].interface = bad.bindings[0].interface.clone();
    assert!(f.validate_configuration(&bad).is_err());
    let mut bad = c.clone();
    bad.bindings[0].context = "outside".into();
    assert!(f.validate_configuration(&bad).is_err());
    f.network.devices.get_mut("a-client").unwrap().forwarding = true;
    assert!(f.validate().is_err());
}
#[test]
fn all_sixteen_membership_combinations_match_an_independent_context_oracle() {
    let (f, mut c) = fixture();
    for bits in 0..16 {
        for (i, b) in c.bindings.iter_mut().enumerate() {
            b.context = if bits & (1 << i) == 0 {
                "blue"
            } else {
                "green"
            }
            .into();
        }
        let expected_valid = c.bindings[0].context != c.bindings[2].context
            && c.bindings[1].context != c.bindings[3].context;
        assert_eq!(f.project(&c).is_ok(), expected_valid, "{bits:04b}");
        if expected_valid {
            let own_group = c.bindings[0].context == c.bindings[1].context;
            let p = f.probe(&c, "a-client", "a-server").unwrap();
            assert_eq!(p.reached_requested, own_group);
            assert_eq!(p.actual, if own_group { "a-server" } else { "b-server" });
        }
    }
}
