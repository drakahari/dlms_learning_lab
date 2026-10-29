use dlms_learning_lab::hsrp::*;
use std::net::Ipv4Addr;
fn ip(n: u8) -> Ipv4Addr {
    Ipv4Addr::new(192, 0, 2, n)
}
fn config(priority: u8, preempt: bool, decrement: Option<u8>) -> RouterConfig {
    RouterConfig {
        participating: true,
        priority,
        preempt,
        track_wan_decrement: decrement,
    }
}
fn router(name: &str, n: u8, priority: u8) -> Router {
    Router {
        label: name.into(),
        address: ip(n),
        config: config(priority, false, None),
        facts: RouterFacts {
            power_on: true,
            lan_link_up: true,
            wan_link_up: true,
            route_available: true,
        },
    }
}
fn clients(gateway: Ipv4Addr) -> Vec<Client> {
    vec![
        Client {
            id: 1,
            label: "desk".into(),
            address: ip(20),
            gateway,
            link_up: true,
        },
        Client {
            id: 2,
            label: "counter".into(),
            address: ip(21),
            gateway,
            link_up: true,
        },
    ]
}
fn fixture() -> State {
    State::new(
        7,
        24,
        ip(254),
        [router("north", 1, 110), router("south", 2, 100)],
        clients(ip(254)),
    )
    .unwrap()
}
fn outcome(s: &State, id: u8) -> Connectivity {
    s.client_status(id).unwrap().outcome
}
fn both_available(s: &State) -> bool {
    s.clients()
        .iter()
        .all(|c| s.client_status(c.id).unwrap().forwarding_available())
}
fn raw(s: &State) -> serde_json::Value {
    serde_json::from_slice(&s.to_json().unwrap()).unwrap()
}
fn decode(v: &serde_json::Value) -> Result<State> {
    State::from_json(&serde_json::to_vec(v).unwrap())
}

#[test]
fn simultaneous_initial_election_uses_priority_then_real_ip_not_array_order() {
    let s = fixture();
    assert_eq!(
        (s.active(), s.standby()),
        (Some(RouterId::A), Some(RouterId::B))
    );
    let a = router("north", 1, 100);
    let b = router("south", 2, 100);
    let tie = State::new(7, 24, ip(254), [a.clone(), b.clone()], clients(ip(254))).unwrap();
    let reversed = State::new(7, 24, ip(254), [b, a], clients(ip(254))).unwrap();
    assert_eq!(tie.active(), Some(RouterId::B));
    assert_eq!(reversed.active(), Some(RouterId::A));
    assert_eq!(reversed.router(reversed.active().unwrap()).address, ip(2));
    assert!(both_available(&tie));
    let mut a = router("north", 1, 110);
    let mut b = router("south", 2, 100);
    a.facts.power_on = false;
    let single = State::new(7, 24, ip(254), [a.clone(), b.clone()], clients(ip(254))).unwrap();
    assert_eq!(
        (single.active(), single.standby()),
        (Some(RouterId::B), None)
    );
    b.facts.lan_link_up = false;
    let mut unavailable = State::new(7, 24, ip(254), [a, b], clients(ip(254))).unwrap();
    assert_eq!((unavailable.active(), unavailable.standby()), (None, None));
    assert_eq!(outcome(&unavailable, 1), Connectivity::NoActiveRouter);
    unavailable
        .fault(FaultEvent::RouterPower {
            router: RouterId::A,
            up: true,
        })
        .unwrap();
    assert_eq!(unavailable.active(), Some(RouterId::A));
    assert!(both_available(&unavailable));
}
#[test]
fn incumbent_history_survives_return_without_preempt_and_resume() {
    let mut s = fixture();
    s.fault(FaultEvent::RouterPower {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
    assert_eq!(s.standby(), None);
    s.fault(FaultEvent::RouterPower {
        router: RouterId::A,
        up: true,
    })
    .unwrap();
    assert_eq!(
        (s.active(), s.standby()),
        (Some(RouterId::B), Some(RouterId::A))
    );
    assert!(s.router(RouterId::A).config.priority > s.router(RouterId::B).config.priority);
    let mut reopened = State::from_json(&s.to_json().unwrap()).unwrap();
    assert_eq!(reopened, s);
    reopened.configure_gateway(1, ip(254)).unwrap();
    assert_eq!(reopened.active(), Some(RouterId::B));
    reopened
        .configure_router(RouterId::A, config(110, true, None))
        .unwrap();
    assert_eq!(reopened.active(), Some(RouterId::A));
    assert!(both_available(&reopened));
}
#[test]
fn equal_priority_higher_ip_does_not_preempt_an_existing_active() {
    let mut s = State::new(
        7,
        24,
        ip(254),
        [router("north", 1, 100), router("south", 2, 100)],
        clients(ip(254)),
    )
    .unwrap();
    s.fault(FaultEvent::LanLink {
        router: RouterId::B,
        up: false,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    s.configure_router(RouterId::B, config(100, true, None))
        .unwrap();
    s.fault(FaultEvent::LanLink {
        router: RouterId::B,
        up: true,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    assert_eq!(s.standby(), Some(RouterId::B));
    s.configure_router(RouterId::B, config(101, true, None))
        .unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
}
#[test]
fn active_priority_reduction_needs_the_other_routers_preempt_not_its_own() {
    let mut s = fixture();
    s.configure_router(RouterId::A, config(90, true, None))
        .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    s.configure_router(RouterId::B, config(100, true, None))
        .unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
    s.configure_router(RouterId::B, config(80, false, None))
        .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
}
#[test]
fn failover_without_preemption_preserves_virtual_gateway_but_not_a_real_gateway() {
    let mut s = fixture();
    s.configure_gateway(2, ip(1)).unwrap();
    assert!(both_available(&s));
    s.fault(FaultEvent::RouterPower {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
    assert!(s.client_status(1).unwrap().uses_virtual_ip);
    assert_eq!(s.client_status(1).unwrap().via, Some(RouterId::B));
    assert_eq!(outcome(&s, 1), Connectivity::ForwardingAvailable);
    assert_eq!(outcome(&s, 2), Connectivity::RouterPoweredOff);
    assert!(!s.client_status(2).unwrap().uses_virtual_ip);
    assert_eq!(s.client_status(2).unwrap().via, Some(RouterId::A));
    assert!(!both_available(&s));
    s.configure_gateway(2, ip(254)).unwrap();
    assert!(both_available(&s));
}
#[test]
fn lan_failure_participation_and_real_address_forwarding_are_distinct() {
    let mut s = fixture();
    s.configure_gateway(2, ip(1)).unwrap();
    let mut disabled = config(110, false, None);
    disabled.participating = false;
    s.configure_router(RouterId::A, disabled).unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
    assert_eq!(s.router_status(RouterId::A).unwrap().role, Role::Disabled);
    assert!(both_available(&s)); // Disabling HSRP is not disabling IPv4 forwarding.
    s.fault(FaultEvent::LanLink {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    assert_eq!(
        s.router_status(RouterId::A).unwrap().role,
        Role::Unavailable
    );
    assert_eq!(outcome(&s, 2), Connectivity::RouterLanDown);
    s.fault(FaultEvent::LanLink {
        router: RouterId::B,
        up: false,
    })
    .unwrap();
    assert_eq!((s.active(), s.standby()), (None, None));
    assert_eq!(outcome(&s, 1), Connectivity::NoActiveRouter);
}
#[test]
fn untracked_wan_failure_does_not_magically_elect_the_healthy_router() {
    let mut s = fixture();
    s.configure_router(RouterId::B, config(100, true, None))
        .unwrap();
    s.fault(FaultEvent::WanLink {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    assert_eq!(
        s.router_status(RouterId::A).unwrap().effective_priority,
        110
    );
    assert_eq!(outcome(&s, 1), Connectivity::WanLinkDown);
    assert!(!both_available(&s));
    s.fault(FaultEvent::WanLink {
        router: RouterId::A,
        up: true,
    })
    .unwrap();
    assert!(both_available(&s));
}
#[test]
fn tracked_wan_decrement_and_strict_preempt_cause_settled_takeover_and_return() {
    let mut s = fixture();
    s.configure_router(RouterId::A, config(110, false, Some(20)))
        .unwrap();
    s.fault(FaultEvent::WanLink {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    assert_eq!(s.router_status(RouterId::A).unwrap().effective_priority, 90);
    assert_eq!(outcome(&s, 1), Connectivity::WanLinkDown);
    s.configure_router(RouterId::B, config(100, true, None))
        .unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
    assert!(both_available(&s));
    s.fault(FaultEvent::WanLink {
        router: RouterId::A,
        up: true,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::B));
    s.configure_router(RouterId::A, config(110, true, Some(20)))
        .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    assert!(both_available(&s));
}
#[test]
fn tracking_just_the_link_does_not_detect_route_loss_or_equal_priority() {
    let mut s = fixture();
    s.configure_router(RouterId::A, config(110, true, Some(10)))
        .unwrap();
    s.configure_router(RouterId::B, config(100, true, None))
        .unwrap();
    s.fault(FaultEvent::UpstreamRoute {
        router: RouterId::A,
        available: false,
    })
    .unwrap();
    assert_eq!(s.active(), Some(RouterId::A));
    assert_eq!(
        s.router_status(RouterId::A).unwrap().effective_priority,
        110
    );
    assert_eq!(outcome(&s, 1), Connectivity::NoUpstreamRoute);
    s.fault(FaultEvent::WanLink {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    assert_eq!(
        s.router_status(RouterId::A).unwrap().effective_priority,
        100
    );
    assert_eq!(s.active(), Some(RouterId::A));
    assert_eq!(outcome(&s, 1), Connectivity::WanLinkDown);
}
#[test]
fn wrong_gateway_diagnostics_client_faults_and_partial_repair_derive_from_state() {
    let mut s = fixture();
    for (gateway, expected) in [
        (
            Ipv4Addr::new(198, 51, 100, 1),
            Connectivity::GatewayOutsideLan,
        ),
        (ip(0), Connectivity::GatewayNotUsableHost),
        (ip(255), Connectivity::GatewayNotUsableHost),
        (ip(99), Connectivity::UnknownGateway),
        (ip(20), Connectivity::UnknownGateway),
    ] {
        s.configure_gateway(1, gateway).unwrap();
        assert_eq!(outcome(&s, 1), expected);
        assert_eq!(outcome(&s, 2), Connectivity::ForwardingAvailable);
        assert!(!both_available(&s));
    }
    s.configure_gateway(1, ip(254)).unwrap();
    assert!(both_available(&s));
    s.fault(FaultEvent::ClientLink {
        client: 2,
        up: false,
    })
    .unwrap();
    assert_eq!(outcome(&s, 2), Connectivity::ClientLinkDown);
    assert!(!both_available(&s));
    s.fault(FaultEvent::ClientLink {
        client: 2,
        up: true,
    })
    .unwrap();
    assert!(both_available(&s));
}
#[test]
fn valid_alternate_priorities_and_active_choices_pass_without_canned_answers() {
    for (a, b) in [(150, 120), (80, 190), (42, 42), (1, 255)] {
        let mut s = fixture();
        s.configure_router(RouterId::A, config(a, true, None))
            .unwrap();
        s.configure_router(RouterId::B, config(b, true, None))
            .unwrap();
        assert!(both_available(&s));
        let active = s.active().unwrap();
        s.fault(FaultEvent::RouterPower {
            router: active,
            up: false,
        })
        .unwrap();
        assert_eq!(s.active(), Some(active.other_for_test()));
        assert!(both_available(&s));
        s.fault(FaultEvent::RouterPower {
            router: active.other_for_test(),
            up: false,
        })
        .unwrap();
        assert!(!both_available(&s));
        assert_eq!(outcome(&s, 1), Connectivity::NoActiveRouter);
    }
}
trait Other {
    fn other_for_test(self) -> Self;
}
impl Other for RouterId {
    fn other_for_test(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}
#[test]
fn unsupported_partition_and_priority_forms_are_atomic_not_split_brain_success() {
    let mut s = fixture();
    let before = s.clone();
    assert_eq!(
        s.fault(FaultEvent::PeerCommunicationLost).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(s, before);
    for c in [
        config(0, true, None),
        config(100, true, Some(0)),
        config(100, true, Some(100)),
        config(100, false, Some(200)),
    ] {
        assert_eq!(
            s.configure_router(RouterId::A, c).unwrap_err().kind,
            ErrorKind::Unsupported
        );
        assert_eq!(s, before);
    }
    assert_eq!(
        s.configure_gateway(9, ip(254)).unwrap_err().kind,
        ErrorKind::Unknown
    );
    assert_eq!(
        s.fault(FaultEvent::ClientLink {
            client: 9,
            up: false
        })
        .unwrap_err()
        .kind,
        ErrorKind::Unknown
    );
    assert_eq!(s.client_status(9).unwrap_err().kind, ErrorKind::Unknown);
    assert_eq!(s, before);
}
#[test]
fn restore_rejects_competing_or_unavailable_roles_and_unsettled_preempt_but_not_valid_history() {
    let s = fixture();
    for change in 0..5 {
        let mut v = raw(&s);
        match change {
            0 => v["standby"] = serde_json::json!("A"),
            1 => v["active"] = serde_json::json!(null),
            2 => v["routers"][0]["facts"]["power_on"] = serde_json::json!(false),
            3 => {
                v["routers"][1]["config"]["priority"] = serde_json::json!(120);
                v["routers"][1]["config"]["preempt"] = serde_json::json!(true);
            }
            _ => v["active"] = serde_json::json!("foreign"),
        }
        assert!(decode(&v).is_err(), "change {change}");
    }
    let mut historic = s;
    historic
        .fault(FaultEvent::RouterPower {
            router: RouterId::A,
            up: false,
        })
        .unwrap();
    historic
        .fault(FaultEvent::RouterPower {
            router: RouterId::A,
            up: true,
        })
        .unwrap();
    assert_eq!(historic.active(), Some(RouterId::B));
    assert_eq!(
        State::from_json(&historic.to_json().unwrap()).unwrap(),
        historic
    );
}
#[test]
fn endpoint_identity_and_lan_validation_reject_duplicates_reserved_addresses_and_foreign_worlds() {
    let s = fixture();
    for change in 0..9 {
        let mut v = raw(&s);
        match change {
            0 => v["routers"][1]["address"] = serde_json::json!("192.0.2.1"),
            1 => v["clients"][0]["address"] = serde_json::json!("192.0.2.254"),
            2 => v["clients"][1]["id"] = serde_json::json!(1),
            3 => v["clients"][0]["label"] = serde_json::json!("../personal"),
            4 => v["virtual_ip"] = serde_json::json!("192.0.2.0"),
            5 => v["routers"][1]["address"] = serde_json::json!("192.0.3.2"),
            6 => v["clients"][1]["address"] = serde_json::json!("192.0.2.255"),
            7 => v["clients"][1]["label"] = serde_json::json!("north"),
            _ => v["routers"][1]["label"] = serde_json::json!("x".repeat(33)),
        }
        assert!(decode(&v).is_err(), "change {change}");
    }
    let mut v = raw(&s);
    v["routers"][0]["address"] = serde_json::json!("192.0.2.3");
    let foreign = decode(&v).unwrap();
    assert!(!foreign.environment_matches(&s).unwrap());
    let mut changed = s.clone();
    changed
        .configure_router(RouterId::B, config(140, true, Some(5)))
        .unwrap();
    changed.configure_gateway(1, ip(2)).unwrap();
    changed
        .fault(FaultEvent::WanLink {
            router: RouterId::A,
            up: false,
        })
        .unwrap();
    assert!(changed.environment_matches(&s).unwrap()); // This is identity, not goal/fault authorization.
}
#[test]
fn subnet_math_supports_octet_crossings_and_excludes_31_32_without_last_octet_shortcuts() {
    for (prefix, vip, a, b, c) in [
        (
            25,
            "192.0.2.254",
            "192.0.2.129",
            "192.0.2.130",
            "192.0.2.131",
        ),
        (23, "192.0.3.254", "192.0.2.1", "192.0.3.1", "192.0.2.255"),
        (
            16,
            "198.18.255.254",
            "198.18.0.1",
            "198.18.1.1",
            "198.18.3.255",
        ),
        (30, "192.0.2.1", "192.0.2.2", "192.0.2.1", "192.0.2.2"),
    ] {
        let mut ra = router("north", 1, 100);
        ra.address = a.parse().unwrap();
        let mut rb = router("south", 2, 90);
        rb.address = b.parse().unwrap();
        let cs = vec![Client {
            id: 1,
            label: "desk".into(),
            address: c.parse().unwrap(),
            gateway: vip.parse().unwrap(),
            link_up: true,
        }];
        let state = State::new(7, prefix, vip.parse().unwrap(), [ra, rb], cs);
        if prefix == 30 {
            assert!(state.is_err());
        }
        // Only two hosts cannot fit VIP+two routers+client.
        else {
            assert!(
                state
                    .unwrap()
                    .client_status(1)
                    .unwrap()
                    .forwarding_available()
            );
        }
    }
    for prefix in [0, 31, 32, 33, 255] {
        let s = fixture();
        let mut v = raw(&s);
        v["prefix"] = serde_json::json!(prefix);
        assert_eq!(decode(&v).unwrap_err().kind, ErrorKind::Unsupported);
    }
}
#[test]
fn raw_codec_bounds_unknown_duplicate_foreign_fields_and_resources() {
    let s = fixture();
    let bytes = s.to_json().unwrap();
    assert_eq!(State::from_json(&bytes).unwrap(), s);
    assert_eq!(
        State::from_json(&vec![b' '; MAX_STATE_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        State::from_json(
            text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        State::from_json(
            text.replacen("\"priority\":110", "\"priority\":110,\"priority\":110", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(State::from_json(b"{malformed").is_err());
    for (key, val) in [
        ("host_command", serde_json::json!("/bin/sh")),
        ("version", serde_json::json!(99)),
        ("group", serde_json::json!(256)),
    ] {
        let mut v = raw(&s);
        v[key] = val;
        assert!(decode(&v).is_err());
    }
    let mut v = raw(&s);
    v["routers"][0]["facts"]["host_path"] = serde_json::json!("/home/personal");
    assert!(decode(&v).is_err());
    let mut v = raw(&s);
    let third = v["routers"][0].clone();
    v["routers"].as_array_mut().unwrap().push(third);
    assert!(decode(&v).is_err());
    let mut v = raw(&s);
    v["clients"] = serde_json::json!([]);
    assert!(decode(&v).is_err());
    let mut v = raw(&s);
    v["clients"] = serde_json::json!(
        (0..9)
            .map(|_| raw(&s)["clients"][0].clone())
            .collect::<Vec<_>>()
    );
    assert!(decode(&v).is_err());
}
#[test]
fn event_counter_exhaustion_and_failed_actions_preserve_the_full_state() {
    let mut v = raw(&fixture());
    v["revision"] = serde_json::json!(u64::MAX);
    let mut s = decode(&v).unwrap();
    let before = s.clone();
    assert_eq!(
        s.configure_gateway(1, ip(1)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        s.fault(FaultEvent::RouterPower {
            router: RouterId::A,
            up: false
        })
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
    assert!(both_available(&s));
}
#[test]
fn selected_status_reads_do_not_mutate_state_and_reset_is_source_clone_not_re_election_on_resume() {
    let initial = fixture();
    let mut s = initial.clone();
    s.fault(FaultEvent::RouterPower {
        router: RouterId::A,
        up: false,
    })
    .unwrap();
    s.fault(FaultEvent::RouterPower {
        router: RouterId::A,
        up: true,
    })
    .unwrap();
    let before = s.clone();
    for id in RouterId::ALL {
        s.router_status(id).unwrap();
    }
    for c in s.clients() {
        s.client_status(c.id).unwrap();
    }
    s.environment_matches(&initial).unwrap();
    s.to_json().unwrap();
    assert_eq!(s, before);
    assert_eq!(s.active(), Some(RouterId::B));
    s = initial.clone();
    assert_eq!(s, initial);
    assert_eq!(s.active(), Some(RouterId::A));
    assert_eq!(s.revision(), 0);
}
