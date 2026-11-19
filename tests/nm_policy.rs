use dlms_learning_lab::{
    ipv4_routing::Prefix, linux_host as h, nm_policy as n, nm_profiles::ErrorKind,
};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn prefix(s: &str) -> Prefix {
    Prefix::parse(s).unwrap()
}
fn environment() -> n::Environment {
    n::Environment {
        bridge: None,
        bond: None,
        vlan: None,
        physical: h::Profile {
            ipv6: None,
            id: "nm-fixture".into(),
            links: vec![
                h::Link {
                    name: "enp1s0".into(),
                    network: prefix("10.20.0.0/24"),
                    carrier: true,
                },
                h::Link {
                    name: "enp2s0".into(),
                    network: prefix("10.21.0.0/24"),
                    carrier: true,
                },
            ],
            gateways: vec![
                h::Gateway {
                    address: ip("10.20.0.1"),
                    device: "enp1s0".into(),
                    available: true,
                    exports: vec![prefix("172.16.0.0/16"), prefix("10.20.0.0/24")],
                },
                h::Gateway {
                    address: ip("10.21.0.1"),
                    device: "enp2s0".into(),
                    available: true,
                    exports: vec![prefix("172.16.0.0/16"), prefix("10.40.0.0/24")],
                },
            ],
            nodes: vec![
                h::Node {
                    address: ip("172.16.5.10"),
                    available: true,
                },
                h::Node {
                    address: ip("10.20.0.50"),
                    available: true,
                },
                h::Node {
                    address: ip("10.40.0.50"),
                    available: true,
                },
            ],
            resolvers: vec![],
        },
        default_metrics: BTreeMap::from([("enp1s0".into(), 100), ("enp2s0".into(), 200)]),
        dhcp: BTreeMap::new(),
    }
}
fn text(
    id: &str,
    uuid: u8,
    device: &str,
    address: &str,
    gateway: &str,
    priority: i16,
    metric: &str,
) -> String {
    format!(
        "[connection]\nid={id}\nuuid=00000000-0000-4000-8000-{uuid:012}\ntype=ethernet\ninterface-name={device}\nautoconnect-priority={priority}\n[ipv4]\nmethod=manual\naddress1={address}\ngateway={gateway}\nroute-metric={metric}\n[ipv6]\nmethod=disabled\n"
    )
}
fn files() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.nmconnection".into(),
            text(
                "Application",
                1,
                "enp1s0",
                "10.20.0.20/24",
                "10.20.0.1",
                10,
                "100",
            ),
        ),
        (
            "management.nmconnection".into(),
            text(
                "Management",
                2,
                "enp2s0",
                "10.21.0.20/24",
                "10.21.0.1",
                10,
                "200",
            ),
        ),
    ])
}
const APP: &str = "00000000-0000-4000-8000-000000000001";
#[test]
fn automatic_selection_uses_priorities_not_filenames_and_does_not_invent_timestamp_ties() {
    let env = environment();
    let mut fs = files();
    fs.insert(
        "00-low.nmconnection".into(),
        text("Low", 3, "enp1s0", "10.20.0.30/24", "10.20.0.1", -5, "100"),
    );
    let policy = n::Policy::from_texts(&fs).unwrap();
    let (chosen, notes) = n::Choices::empty().automatic(&env, &policy).unwrap();
    assert_eq!(chosen.for_device("enp1s0").unwrap().uuid, APP);
    assert_eq!(notes.len(), 2);
    assert!(notes.iter().any(|d| d.reason.contains("priority 10")));
    fs.insert(
        "00-low.nmconnection".into(),
        text("Tie", 3, "enp1s0", "10.20.0.30/24", "10.20.0.1", 10, "100"),
    );
    let tie = n::Policy::from_texts(&fs).unwrap();
    let before = chosen.clone();
    let e = n::Choices::empty().automatic(&env, &tie).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert!(e.message.contains("timestamp/history"));
    assert_eq!(chosen, before);
}
#[test]
fn reloading_changed_profiles_is_not_current_reconfiguration_or_automatic_preemption() {
    let env = environment();
    let mut fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\naddress1=10.20.0.10/24\n");
    let old = n::Policy::from_texts(&fs).unwrap();
    let current = n::Choices::empty().manual(&env, &old, APP).unwrap();
    assert_eq!(
        current.for_device("enp1s0").unwrap().address.address,
        ip("10.20.0.10")
    );
    let next = n::Policy::from_texts(&files()).unwrap();
    assert_ne!(old, next);
    let (retained, notes) = current.automatic(&env, &next).unwrap();
    assert_eq!(
        retained.for_device("enp1s0").unwrap(),
        current.for_device("enp1s0").unwrap()
    );
    assert!(notes[0].reason.contains("do not preempt"));
    let applied = retained.manual(&env, &next, APP).unwrap();
    assert_eq!(
        applied.for_device("enp1s0").unwrap().address.address,
        ip("10.20.0.20")
    );
    assert_eq!(
        n::Choices::empty()
            .automatic(&env, &next)
            .unwrap()
            .0
            .for_device("enp1s0")
            .unwrap()
            .address
            .address,
        ip("10.20.0.20")
    );
}
#[test]
fn disabled_autoconnect_allows_explicit_selection_and_does_not_preempt_existing_settings() {
    let env = environment();
    let mut fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect=false\n");
    let policy = n::Policy::from_texts(&fs).unwrap();
    let (boot, _) = n::Choices::empty().automatic(&env, &policy).unwrap();
    assert!(boot.for_device("enp1s0").is_none());
    let manual = boot.manual(&env, &policy, APP).unwrap();
    assert!(manual.for_device("enp1s0").is_some());
    assert_eq!(manual.automatic(&env, &policy).unwrap().0, manual);
}
#[test]
fn connected_default_and_static_route_metrics_are_real_profile_values() {
    let env = environment();
    let policy = n::Policy::from_texts(&files()).unwrap();
    let (chosen, _) = n::Choices::empty().automatic(&env, &policy).unwrap();
    let local = chosen.select(&env, ip("10.20.0.50")).unwrap();
    assert_eq!(
        (local.route.origin, local.route.metric),
        (n::Origin::Connected, 100)
    );
    assert_eq!(
        chosen.select(&env, ip("172.16.5.10")).unwrap().route.device,
        "enp1s0"
    );
    assert!(chosen.probe(&env, ip("172.16.5.10")).pass);
    let mut fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\nroute1=10.20.0.0/24,10.20.0.1,50\n");
    let (specific, _) = n::Choices::empty()
        .automatic(&env, &n::Policy::from_texts(&fs).unwrap())
        .unwrap();
    let got = specific.select(&env, ip("10.20.0.50")).unwrap();
    assert_eq!(got.route.origin, n::Origin::Static);
    assert_eq!(got.route.metric, 50);
    assert_eq!(got.route.via, Some(ip("10.20.0.1")));
    assert!(
        specific.probe(&env, ip("10.20.0.50")).pass,
        "physical reuse must follow the NM-selected path, not the old core's implicit connected metric0"
    );
}
#[test]
fn automatic_metrics_come_from_supplied_contexts_not_a_universal_default() {
    let mut fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\nroute-metric=-1\n");
    let policy = n::Policy::from_texts(&fs).unwrap();
    for (default, wanted) in [(100, "enp1s0"), (600, "enp2s0")] {
        let mut env = environment();
        env.default_metrics.insert("enp1s0".into(), default);
        let (selected, _) = n::Choices::empty().automatic(&env, &policy).unwrap();
        assert_eq!(
            selected
                .select(&env, ip("172.16.5.10"))
                .unwrap()
                .route
                .device,
            wanted
        );
        assert_eq!(
            selected
                .select(&env, ip("10.20.0.50"))
                .unwrap()
                .route
                .metric,
            default
        );
    }
}
#[test]
fn longest_prefix_wins_before_metric_and_never_default_only_has_supported_effects() {
    let mut fs = files();
    fs.get_mut("management.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\nnever-default=true\nroute1=10.40.0.0/24,10.21.0.1,900\n");
    let env = environment();
    let (c, _) = n::Choices::empty()
        .automatic(&env, &n::Policy::from_texts(&fs).unwrap())
        .unwrap();
    assert_eq!(
        c.select(&env, ip("10.40.0.50")).unwrap().route.device,
        "enp2s0"
    );
    assert!(c.probe(&env, ip("10.40.0.50")).pass);
    assert!(
        !c.route_table(&env)
            .unwrap()
            .iter()
            .any(|r| r.device == "enp2s0" && r.destination.length == 0)
    );
    fs.get_mut("management.nmconnection")
        .unwrap()
        .push_str("route2=0.0.0.0/0,10.21.0.1,40\n");
    let (c, _) = n::Choices::empty()
        .automatic(&env, &n::Policy::from_texts(&fs).unwrap())
        .unwrap();
    assert_eq!(
        c.route_table(&env).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn wrong_valid_profiles_have_specific_path_feedback_and_no_host_packet_activity() {
    let env = environment();
    let mut fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\ngateway=10.99.0.1\n");
    let (c, _) = n::Choices::empty()
        .automatic(&env, &n::Policy::from_texts(&fs).unwrap())
        .unwrap();
    let fail = c.probe(&env, ip("172.16.5.10"));
    assert!(!fail.pass);
    assert!(fail.reason.contains("not on-link"));
    fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[connection]\ninterface-name=old0\n");
    let (c, notes) = n::Choices::empty()
        .automatic(&env, &n::Policy::from_texts(&fs).unwrap())
        .unwrap();
    assert!(c.for_device("enp1s0").is_none());
    assert!(notes.iter().any(|n| n.reason.contains("No eligible")));
}
#[test]
fn opaque_history_and_ambiguous_or_unrepresentable_contexts_are_refused_transactionally() {
    let env = environment();
    let mut fs = files();
    fs.get_mut("app.nmconnection")
        .unwrap()
        .push_str("\n[ipv4]\nroute1=10.20.0.0/24,10.20.0.1,100\n");
    let (c, _) = n::Choices::empty()
        .automatic(&env, &n::Policy::from_texts(&fs).unwrap())
        .unwrap();
    assert_eq!(
        c.select(&env, ip("10.20.0.50")).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    let p = n::Policy::from_texts(&files()).unwrap();
    let original = n::Choices::empty();
    assert!(original.manual(&env, &p, "foreign").is_err());
    assert_eq!(original, n::Choices::empty());
    let mut bad_env = env.clone();
    bad_env.default_metrics.remove("enp1s0");
    assert!(original.automatic(&bad_env, &p).is_err());
    let mut down = env.clone();
    down.physical.links[0].carrier = false;
    assert!(original.manual(&down, &p, APP).is_err());
    let (c, notes) = original.automatic(&down, &p).unwrap();
    assert!(c.for_device("enp1s0").is_none());
    assert!(notes[0].reason.contains("carrier"));
}
#[test]
fn virtual_bundle_bounds_identity_and_paths_refuse_before_publication() {
    let source = files();
    let p = n::Policy::from_texts(&source).unwrap();
    assert_eq!(p.profiles().count(), 2);
    let before = p.clone();
    for name in [
        "../outside.nmconnection",
        "/etc/NetworkManager/personal.nmconnection",
        ".hidden.nmconnection",
        "wrong.txt",
    ] {
        let fs = BTreeMap::from([(name.into(), source["app.nmconnection"].clone())]);
        assert_eq!(
            n::Policy::from_texts(&fs).unwrap_err().kind,
            ErrorKind::Unsupported
        );
        assert_eq!(p, before);
    }
    let mut duplicate = source.clone();
    duplicate.insert(
        "duplicate.nmconnection".into(),
        source["app.nmconnection"].clone(),
    );
    assert_eq!(
        n::Policy::from_texts(&duplicate).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    let mut large = BTreeMap::new();
    for i in 0..9 {
        large.insert(
            format!("{i}.nmconnection"),
            source["app.nmconnection"].clone(),
        );
    }
    assert_eq!(
        n::Policy::from_texts(&large).unwrap_err().kind,
        ErrorKind::Limit
    );
}
