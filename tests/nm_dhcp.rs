//! Supplied-lease projection, not daemon/runtime conformance.
#[path = "support/nm_workspace.rs"]
mod fixture;
use dlms_learning_lab::{
    ipv4_routing::Assignment,
    nm_dhcp::Outcome,
    nm_policy as policy,
    nm_profiles::{self as parser, Addressing, ErrorKind},
    nm_workspace as work,
};
use std::net::Ipv4Addr;
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn lease(last: u8, router: Option<&str>) -> Outcome {
    Outcome::Bound {
        address: Assignment {
            address: ip(&format!("10.20.0.{last}")),
            prefix: 24,
        },
        router: router.map(ip),
    }
}
fn auto(text: &str) -> String {
    text.lines()
        .filter(|l| !l.starts_with("address1=") && !l.starts_with("gateway="))
        .map(|l| {
            if l == "method=manual" {
                "method=auto"
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
fn automatic_files() -> work::Files {
    let mut fs = fixture::files();
    fs.insert("app.nmconnection".into(), auto(&fs["app.nmconnection"]));
    fs
}
const APP: &str = "00000000-0000-4000-8000-000000000001";

#[test]
fn typed_saved_method_has_no_fake_address_and_preserves_the_static_only_api() {
    let fs = automatic_files();
    let text = &fs["app.nmconnection"];
    assert_eq!(
        parser::parse_settings(text).unwrap().addressing,
        Addressing::Automatic
    );
    assert_eq!(
        parser::parse(text).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        parser::parse_settings(&format!(
            "{text}\n[ipv4]\nmethod=manual\naddress1=10.20.0.80/24\n"
        ))
        .unwrap()
        .addressing,
        Addressing::Manual {
            address: Assignment {
                address: ip("10.20.0.80"),
                prefix: 24
            },
            gateway: None
        }
    );
    for extra in [
        "address1=10.20.0.60/24",
        "gateway=10.20.0.1",
        "dns=10.20.0.53",
        "ignore-auto-routes=true",
        "dhcp-timeout=5",
    ] {
        assert_eq!(
            parser::parse_settings(&format!("{text}\n[ipv4]\n{extra}\n"))
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported,
            "{extra}"
        );
    }
    assert_eq!(
        parser::parse_settings(&"x".repeat(parser::MAX_TEXT + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(parser::parse_settings(&format!("{text}#\0")).is_err());
    assert!(parser::parse_settings(&format!("{text}# non-ASCII é")).is_err());
    let manual = &fixture::files()["app.nmconnection"];
    let Addressing::Manual { address, gateway } =
        parser::parse_settings(manual).unwrap().addressing
    else {
        panic!()
    };
    let active = parser::parse(manual).unwrap();
    assert_eq!((address, gateway), (active.address, active.gateway));
}

#[test]
fn each_supplied_lease_derives_the_real_address_router_and_profile_route_metrics() {
    let fs = automatic_files();
    let loaded = policy::Policy::from_texts(&fs).unwrap();
    for (last, gateway) in [(60, "10.20.0.1"), (97, "10.20.0.254"), (126, "10.20.0.1")] {
        let mut env = fixture::environment();
        env.dhcp.insert("enp1s0".into(), lease(last, Some(gateway)));
        let (active, _) = policy::Choices::empty().automatic(&env, &loaded).unwrap();
        assert!(active.uses_dhcp("enp1s0"));
        assert!(!active.uses_dhcp("enp2s0"));
        assert_eq!(
            active.for_device("enp1s0").unwrap().address.address,
            ip(&format!("10.20.0.{last}"))
        );
        let routes = active.route_table(&env).unwrap();
        assert!(routes.iter().any(|r| r.device == "enp1s0"
            && r.destination.length == 24
            && r.metric == 100
            && r.via.is_none()));
        assert!(routes.iter().any(|r| r.device == "enp1s0"
            && r.destination.length == 0
            && r.metric == 100
            && r.via == Some(ip(gateway))));
    }
    let mut env = fixture::environment();
    env.dhcp
        .insert("enp1s0".into(), lease(60, Some("10.20.0.1")));
    let active = policy::Choices::empty().manual(&env, &loaded, APP).unwrap();
    assert!(active.probe(&env, ip("172.16.5.10")).pass);
    env.dhcp
        .insert("enp1s0".into(), lease(50, Some("10.20.0.1")));
    let collision = policy::Choices::empty().manual(&env, &loaded, APP).unwrap();
    assert!(!collision.probe(&env, ip("172.16.5.10")).pass);
    assert!(
        collision
            .probe(&env, ip("172.16.5.10"))
            .reason
            .contains("conflicts")
    );
}

#[test]
fn never_default_retains_the_lease_and_connected_route_and_allows_the_other_default() {
    let mut fs = automatic_files();
    let text = fs["app.nmconnection"].replace("[ipv4]", "[ipv4]\nnever-default=true");
    fs.insert("app.nmconnection".into(), text);
    let loaded = policy::Policy::from_texts(&fs).unwrap();
    let mut env = fixture::environment();
    env.dhcp
        .insert("enp1s0".into(), lease(60, Some("10.20.0.1")));
    let (active, _) = policy::Choices::empty().automatic(&env, &loaded).unwrap();
    assert!(active.uses_dhcp("enp1s0"));
    assert!(active.for_device("enp1s0").unwrap().gateway.is_none());
    let rows = active.route_table(&env).unwrap();
    assert!(
        !rows
            .iter()
            .any(|r| r.destination.length == 0 && r.device == "enp1s0")
    );
    assert_eq!(
        active.select(&env, ip("172.16.5.10")).unwrap().route.device,
        "enp2s0"
    );
    assert_eq!(
        active.select(&env, ip("10.20.0.60")).unwrap().route.device,
        "enp1s0"
    );
    env.dhcp.insert("enp1s0".into(), lease(60, None));
    assert!(
        policy::Choices::empty()
            .automatic(&env, &loaded)
            .unwrap()
            .0
            .for_device("enp1s0")
            .unwrap()
            .gateway
            .is_none()
    );
}

#[test]
fn current_snapshots_are_not_reacquired_and_unavailable_inputs_never_invent_connectivity() {
    let loaded = policy::Policy::from_texts(&automatic_files()).unwrap();
    let mut env = fixture::environment();
    env.dhcp
        .insert("enp1s0".into(), lease(60, Some("10.20.0.1")));
    let current = policy::Choices::empty().manual(&env, &loaded, APP).unwrap();
    env.dhcp
        .insert("enp1s0".into(), lease(97, Some("10.20.0.254")));
    let (warm, _) = current.automatic(&env, &loaded).unwrap();
    assert_eq!(warm.for_device("enp1s0"), current.for_device("enp1s0"));
    assert!(warm.uses_dhcp("enp1s0"));
    assert_eq!(
        policy::Choices::empty()
            .automatic(&env, &loaded)
            .unwrap()
            .0
            .for_device("enp1s0")
            .unwrap()
            .address
            .address,
        ip("10.20.0.97")
    );
    env.dhcp.insert("enp1s0".into(), Outcome::Unavailable {});
    let (future, decisions) = policy::Choices::empty().automatic(&env, &loaded).unwrap();
    assert!(future.for_device("enp1s0").is_none());
    assert!(!future.uses_dhcp("enp1s0"));
    assert!(future.for_device("enp2s0").is_some());
    assert!(
        decisions
            .iter()
            .any(|d| d.device == "enp1s0" && d.uuid.is_none() && d.reason.contains("unavailable"))
    );
    assert!(current.manual(&env, &loaded, APP).is_err());
    assert_eq!(
        current.for_device("enp1s0").unwrap().address.address,
        ip("10.20.0.60")
    );
    env.dhcp.clear();
    assert_eq!(
        policy::Choices::empty()
            .automatic(&env, &loaded)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}

#[test]
fn outcome_validation_is_bounded_and_old_environment_wire_remains_exact() {
    let env = fixture::environment();
    let wire = serde_json::to_value(&env).unwrap();
    assert!(wire.get("dhcp").is_none());
    assert_eq!(
        serde_json::from_value::<policy::Environment>(wire.clone()).unwrap(),
        env
    );
    for bad in [
        Outcome::Bound {
            address: Assignment {
                address: ip("10.20.0.0"),
                prefix: 24,
            },
            router: None,
        },
        Outcome::Bound {
            address: Assignment {
                address: ip("10.20.0.60"),
                prefix: 20,
            },
            router: None,
        },
        Outcome::Bound {
            address: Assignment {
                address: ip("10.21.0.50"),
                prefix: 24,
            },
            router: None,
        },
        lease(60, Some("10.21.0.1")),
        lease(60, Some("10.20.0.255")),
        lease(60, Some("10.20.0.60")),
    ] {
        let mut e = env.clone();
        e.dhcp.insert("enp1s0".into(), bad);
        assert!(e.validate().is_err());
    }
    let mut e = env.clone();
    e.dhcp.insert("foreign".into(), Outcome::Unavailable {});
    assert!(e.validate().is_err());
    let raw = serde_json::to_string(&wire).unwrap();
    for map in [
        r#"{"enp1s0":{"kind":"Unavailable"},"enp1s0":{"kind":"Unavailable"}}"#,
        r#"{"a":{"kind":"Unavailable"},"b":{"kind":"Unavailable"},"c":{"kind":"Unavailable"}}"#,
        r#"{"enp1s0":{"kind":"Unavailable","host_path":"/etc/secret"}}"#,
    ] {
        let with = format!("{},\"dhcp\":{map}}}", &raw[..raw.len() - 1]);
        assert!(
            serde_json::from_str::<policy::Environment>(&with).is_err(),
            "{map}"
        );
    }
}

#[test]
fn behavior_cases_reject_hardcoded_addresses_wrong_method_and_stale_or_unfinished_credit() {
    let mut d = fixture::definition();
    for (n, c) in d.initial.cases.iter_mut().enumerate() {
        let last = if n == 0 { 60 } else { 97 };
        c.environment
            .dhcp
            .insert("enp1s0".into(), lease(last, Some("10.20.0.1")));
        for r in &mut c.requirements {
            if let work::Requirement::FutureAddress { address, .. } = r {
                address.address = ip(&format!("10.20.0.{last}"));
            }
        }
        c.requirements.push(work::Requirement::FutureDhcp {
            interface: "enp1s0".into(),
        });
    }
    d.initial.protected_profiles = vec!["management.nmconnection".into()];
    d.validate().unwrap();
    let mut fs = d.initial.starter.clone();
    fs.insert("app.nmconnection".into(), auto(&fs["app.nmconnection"]));
    let mut equivalent = fs.clone();
    equivalent
        .get_mut("management.nmconnection")
        .unwrap()
        .push_str("# explanatory note\n");
    assert!(
        d.initial
            .run(&equivalent)
            .unwrap()
            .iter()
            .all(|r| r.passed())
    );
    let mut damaged = fs.clone();
    let text = damaged["management.nmconnection"].replace("route-metric=200", "route-metric=201");
    damaged.insert("management.nmconnection".into(), text);
    assert!(d.initial.run(&damaged).unwrap().iter().all(|r| {
        r.checks
            .iter()
            .any(|(ok, why)| !ok && why.contains("protected profile"))
    }));
    let results = d.initial.run(&fs).unwrap();
    assert!(results.iter().all(|r| r.passed()), "{results:?}");
    let mut s = work::Session::new(&d).unwrap();
    s.assignment.replace(&fs).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let raw = serde_json::to_string(&s).unwrap();
    let restored = work::Session::from_json(&raw, &d).unwrap();
    assert!(d.solved(&restored));
    let mut wrong = fs.clone();
    wrong.insert(
        "app.nmconnection".into(),
        fixture::solved(&d.initial)["app.nmconnection"].replace("10.20.0.20/24", "10.20.0.60/24"),
    );
    let failed = d.initial.run(&wrong).unwrap();
    assert!(failed.iter().all(|r| !r.passed()));
    assert!(
        failed[0]
            .checks
            .iter()
            .any(|(ok, why)| !ok && why.contains("manual address"))
    );
    s.assignment.replace(&wrong).unwrap();
    assert!(!d.solved(&s));
    assert!(s.assignment.tested.is_none());
    s.assignment.replace(&fs).unwrap();
    let mut running = s.assignment.begin(&d.initial).unwrap();
    running.tick();
    running.cancel();
    assert!(s.assignment.finish(&d.initial, &running).is_err());
    assert!(!d.solved(&s));
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
}

#[test]
fn local_only_outcome_rejects_a_less_preferred_default_but_accepts_no_default() {
    let mut task = fixture::definition().initial;
    for case in &mut task.cases {
        case.environment
            .dhcp
            .insert("enp1s0".into(), lease(60, Some("10.20.0.1")));
        case.requirements = vec![
            work::Requirement::FutureDhcp {
                interface: "enp1s0".into(),
            },
            work::Requirement::FutureNoDefault {
                interface: "enp1s0".into(),
            },
            work::Requirement::DefaultOwner {
                interface: "enp2s0".into(),
            },
        ];
    }
    let mut files = task.starter.clone();
    let text = auto(&files["app.nmconnection"]);
    files.insert(
        "app.nmconnection".into(),
        text.replace("route-metric=100", "route-metric=400"),
    );
    let results = task.run(&files).unwrap();
    assert!(results.iter().all(|r| {
        !r.passed()
            && r.checks
                .iter()
                .any(|(ok, why)| !ok && why.contains("Raising a metric"))
    }));
    files.insert(
        "app.nmconnection".into(),
        text.replace("[ipv4]", "[ipv4]\nnever-default=true"),
    );
    assert!(task.run(&files).unwrap().iter().all(|r| r.passed()));
}
