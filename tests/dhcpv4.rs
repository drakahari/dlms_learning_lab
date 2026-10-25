//! Pure prospective model tests; no OS DHCP, packet, clock or command execution.
use dlms_learning_lab::dhcpv4::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn net(s: &str, prefix: u8) -> Subnet {
    Subnet {
        network: ip(s),
        prefix,
    }
}
fn pool(subnet: Subnet, first: &str, last: &str) -> Pool {
    Pool {
        subnet,
        first: ip(first),
        last: ip(last),
        excluded: BTreeSet::new(),
        reservations: BTreeMap::new(),
        router: Some(Ipv4Addr::from(u32::from(subnet.network) + 1)),
        dns: vec![ip("203.0.113.53")],
        lease_seconds: 16,
    }
}
fn fixture() -> (Topology, Policy) {
    let segments = BTreeMap::from([
        (
            "alpha".into(),
            Segment {
                subnet: net("192.0.2.0", 24),
                server_interface: Some(ip("192.0.2.1")),
                relay_interface: None,
                relay_to_server: false,
                client_unicast_to_server: true,
                occupied: BTreeSet::new(),
            },
        ),
        (
            "beta".into(),
            Segment {
                subnet: net("198.51.100.0", 24),
                server_interface: None,
                relay_interface: Some(ip("198.51.100.1")),
                relay_to_server: true,
                client_unicast_to_server: true,
                occupied: BTreeSet::new(),
            },
        ),
    ]);
    let clients = [
        ("alice", "alpha"),
        ("bob", "alpha"),
        ("erin", "alpha"),
        ("frank", "alpha"),
        ("carol", "beta"),
        ("dave", "beta"),
    ]
    .into_iter()
    .map(|(c, s)| (c.into(), Client { segment: s.into() }))
    .collect();
    let topology = Topology {
        server: ip("192.0.2.1"),
        segments,
        clients,
    };
    let policy = Policy {
        server_enabled: true,
        pools: BTreeMap::from([
            (
                "alpha".into(),
                pool(net("192.0.2.0", 24), "192.0.2.10", "192.0.2.12"),
            ),
            (
                "beta".into(),
                pool(net("198.51.100.0", 24), "198.51.100.20", "198.51.100.22"),
            ),
        ]),
        relays: BTreeMap::from([(
            "beta".into(),
            Relay {
                enabled: true,
                target: topology.server,
                giaddr: ip("198.51.100.1"),
            },
        )]),
    };
    (topology, policy)
}
fn state() -> State {
    let (t, p) = fixture();
    State::new(t, p).unwrap()
}
fn offer(s: &mut State, c: &str) -> Offer {
    let Outcome::Offered(o) = s.discover(c).unwrap() else {
        panic!("expected offer")
    };
    o
}
fn acquire(s: &mut State, c: &str) -> Binding {
    let o = offer(s, c);
    let Outcome::Ack {
        binding,
        kind: AckKind::NewLease,
    } = s.request(c, o.token).unwrap()
    else {
        panic!("new ACK")
    };
    binding
}
fn policy(s: &mut State, f: impl FnOnce(&mut Policy)) {
    let mut p = s.policy().clone();
    f(&mut p);
    s.replace_policy(p).unwrap();
}
fn set_clock(s: &State, now: u64) -> State {
    let mut v = serde_json::to_value(s).unwrap();
    v["now"] = serde_json::json!(now);
    serde_json::from_value(v).unwrap()
}
#[test]
fn offer_is_not_a_lease_and_independent_clients_get_unique_current_addresses() {
    let mut s = state();
    let a = offer(&mut s, "alice");
    let b = offer(&mut s, "bob");
    assert_eq!(a.address, ip("192.0.2.10"));
    assert_eq!(b.address, ip("192.0.2.11"));
    assert!(s.bindings().is_empty() && s.leases().is_empty());
    assert_eq!(s.window("alice").unwrap(), Window::Selecting);
    let Outcome::Ack { binding, kind } = s.request("alice", a.token).unwrap() else {
        panic!()
    };
    assert_eq!(kind, AckKind::NewLease);
    assert_eq!(s.leases()["alice"], binding);
    assert_eq!(s.bindings()["alice"], binding);
    assert_eq!(binding.renew_at, 8);
    assert_eq!(binding.rebind_at, 14);
    assert_eq!(binding.expires_at, 16);
    assert_eq!(
        s.request("alice", a.token).unwrap(),
        Outcome::NoReply(Reason::NoOffer)
    );
    s.validate().unwrap();
}
#[test]
fn held_offers_exhaust_range_and_expire_without_configuring_any_client() {
    let mut s = state();
    for c in ["alice", "bob", "erin"] {
        offer(&mut s, c);
    }
    assert_eq!(
        s.discover("frank").unwrap(),
        Outcome::NoReply(Reason::Exhausted)
    );
    let Outcome::Advanced {
        offers,
        clients,
        bindings,
    } = s.advance(OFFER_HOLD).unwrap()
    else {
        panic!()
    };
    assert_eq!(offers, vec!["alice", "bob", "erin"]);
    assert!(clients.is_empty() && bindings.is_empty());
    assert_eq!(offer(&mut s, "frank").address, ip("192.0.2.10"));
}
#[test]
fn reservations_exclusions_and_infrastructure_are_deterministic_not_fallback_success() {
    let mut s = state();
    policy(&mut s, |p| {
        p.pools
            .get_mut("alpha")
            .unwrap()
            .reservations
            .insert("alice".into(), ip("192.0.2.50"));
    });
    assert_eq!(offer(&mut s, "alice").address, ip("192.0.2.50"));
    assert_eq!(offer(&mut s, "bob").address, ip("192.0.2.10"));
    policy(&mut s, |p| {
        p.pools
            .get_mut("alpha")
            .unwrap()
            .excluded
            .insert(ip("192.0.2.50"));
    });
    assert_eq!(
        s.discover("alice").unwrap(),
        Outcome::NoReply(Reason::ReservationUnavailable)
    );
    assert!(!s.offers().contains_key("alice"));
    let mut s = state();
    policy(&mut s, |p| {
        let a = p.pools.get_mut("alpha").unwrap();
        a.first = ip("192.0.2.1");
        a.last = ip("192.0.2.3");
        a.excluded.insert(ip("192.0.2.2"));
    });
    assert_eq!(offer(&mut s, "alice").address, ip("192.0.2.3"));
}
#[test]
fn current_options_change_only_on_a_successful_renewed_grant() {
    let mut s = state();
    let original = acquire(&mut s, "alice");
    policy(&mut s, |p| {
        let a = p.pools.get_mut("alpha").unwrap();
        a.router = Some(ip("192.0.2.254"));
        a.dns = vec![ip("203.0.113.99")];
        a.lease_seconds = 32;
    });
    assert_eq!(s.leases()["alice"], original);
    s.advance(4).unwrap();
    let Outcome::Ack { binding, kind } = s.renew("alice").unwrap() else {
        panic!()
    };
    assert_eq!(kind, AckKind::Renewed);
    assert_eq!(binding.parameters.router, Some(ip("192.0.2.254")));
    assert_eq!(binding.parameters.dns, vec![ip("203.0.113.99")]);
    assert_eq!(binding.expires_at, 36);
    assert_eq!(binding.acquired_at, 4);
}
#[test]
fn accepted_pending_offer_preserves_its_options_even_after_policy_edits_and_rename() {
    let mut s = state();
    let o = offer(&mut s, "alice");
    policy(&mut s, |p| {
        let mut a = p.pools.remove("alpha").unwrap();
        a.dns = vec![ip("203.0.113.99")];
        a.lease_seconds = 32;
        p.pools.insert("renamed".into(), a);
    });
    let Outcome::Ack { binding, .. } = s.request("alice", o.token).unwrap() else {
        panic!()
    };
    assert_eq!(binding.pool, "renamed");
    assert_eq!(binding.parameters, o.parameters);
    assert_eq!(binding.expires_at, 16);
    assert!(matches!(
        s.renew("alice").unwrap(),
        Outcome::Ack {
            kind: AckKind::Renewed,
            ..
        }
    ));
    assert_eq!(s.leases()["alice"].granted_seconds, 32);
}
#[test]
fn reachable_policy_decline_acknowledges_current_lease_without_applying_new_options() {
    let mut s = state();
    let b = acquire(&mut s, "alice");
    s.advance(2).unwrap();
    policy(&mut s, |p| {
        let a = p.pools.get_mut("alpha").unwrap();
        a.excluded.insert(b.address);
        a.dns = vec![ip("203.0.113.99")];
    });
    let before = s.clone();
    assert_eq!(
        s.renew("alice").unwrap(),
        Outcome::Ack {
            binding: b.clone(),
            kind: AckKind::CurrentLease
        }
    );
    assert_eq!(s, before);
    assert_eq!(b.remaining(s.now()), 14);
    s.advance(14).unwrap();
    assert!(!s.leases().contains_key("alice") && !s.bindings().contains_key("alice"));
    assert_eq!(offer(&mut s, "alice").address, ip("192.0.2.11"));
}
#[test]
fn relay_giaddr_selects_remote_scope_and_is_not_the_router_option() {
    let mut s = state();
    policy(&mut s, |p| {
        p.pools.get_mut("beta").unwrap().router = Some(ip("198.51.100.254"))
    });
    let o = offer(&mut s, "carol");
    assert_eq!(o.address, ip("198.51.100.20"));
    assert_eq!(
        o.delivery,
        Delivery::Relay {
            segment: "beta".into(),
            giaddr: ip("198.51.100.1")
        }
    );
    assert_eq!(o.parameters.router, Some(ip("198.51.100.254")));
    assert!(matches!(
        s.request("carol", o.token).unwrap(),
        Outcome::Ack {
            kind: AckKind::NewLease,
            ..
        }
    ));
}
#[test]
fn no_relay_bad_target_bad_giaddr_and_transport_failures_are_not_naks() {
    for (kind, reason) in [
        (0, Reason::NoRelay),
        (1, Reason::WrongRelayTarget),
        (2, Reason::WrongRelayAddress),
        (3, Reason::RelayUnreachable),
        (4, Reason::ServerDisabled),
        (5, Reason::NoPool),
    ] {
        let (t, mut p) = fixture();
        let mut t = t;
        match kind {
            0 => {
                p.relays.clear();
            }
            1 => p.relays.get_mut("beta").unwrap().target = ip("203.0.113.99"),
            2 => p.relays.get_mut("beta").unwrap().giaddr = ip("192.0.2.1"),
            3 => t.segments.get_mut("beta").unwrap().relay_to_server = false,
            4 => p.server_enabled = false,
            _ => {
                p.pools.remove("beta");
            }
        }
        let mut s = State::new(t, p).unwrap();
        assert_eq!(s.discover("carol").unwrap(), Outcome::NoReply(reason));
        assert!(s.leases().is_empty() && s.bindings().is_empty());
    }
    let mut s = state();
    policy(&mut s, |p| {
        p.relays.insert(
            "alpha".into(),
            Relay {
                enabled: true,
                target: ip("0.0.0.0"),
                giaddr: ip("255.255.255.255"),
            },
        );
    });
    assert!(
        matches!(s.discover("alice").unwrap(), Outcome::Offered(_)),
        "direct broadcast does not require a relay"
    );
}
#[test]
fn broken_relay_does_not_break_unicast_renewal_but_blocks_rebinding() {
    let mut s = state();
    acquire(&mut s, "carol");
    policy(&mut s, |p| {
        p.relays.get_mut("beta").unwrap().target = ip("203.0.113.99")
    });
    s.advance(8).unwrap();
    assert!(matches!(
        s.renew("carol").unwrap(),
        Outcome::Ack {
            kind: AckKind::Renewed,
            ..
        }
    ));
    s.advance(14).unwrap();
    assert_eq!(s.window("carol").unwrap(), Window::Rebinding);
    let before = s.clone();
    assert_eq!(
        s.rebind("carol").unwrap(),
        Outcome::NoReply(Reason::WrongRelayTarget)
    );
    assert_eq!(s, before);
}
#[test]
fn unreachable_unicast_retains_lease_and_valid_rebind_can_repair_reachability() {
    let (mut t, p) = fixture();
    t.segments.get_mut("beta").unwrap().client_unicast_to_server = false;
    let mut s = State::new(t, p).unwrap();
    let b = acquire(&mut s, "carol");
    s.advance(8).unwrap();
    let before = s.clone();
    assert_eq!(
        s.renew("carol").unwrap(),
        Outcome::NoReply(Reason::UnicastUnreachable)
    );
    assert_eq!(s, before);
    assert_eq!(s.leases()["carol"], b);
    s.advance(6).unwrap();
    assert!(matches!(
        s.rebind("carol").unwrap(),
        Outcome::Ack {
            kind: AckKind::Renewed,
            ..
        }
    ));
    assert_eq!(s.leases()["carol"].expires_at, 30);
}
#[test]
fn release_stops_client_use_even_when_server_notification_cannot_arrive() {
    let (mut t, p) = fixture();
    t.segments.get_mut("beta").unwrap().client_unicast_to_server = false;
    let mut s = State::new(t, p).unwrap();
    let b = acquire(&mut s, "carol");
    assert_eq!(
        s.release("carol").unwrap(),
        Outcome::Released {
            address: b.address,
            server_notified: false
        }
    );
    assert!(!s.leases().contains_key("carol"));
    assert_eq!(s.bindings()["carol"], b);
    assert_eq!(offer(&mut s, "dave").address, ip("198.51.100.21"));
    let o = offer(&mut s, "carol");
    assert!(o.existing_binding);
    assert_eq!(
        s.request("carol", o.token).unwrap(),
        Outcome::Ack {
            binding: b,
            kind: AckKind::CurrentLease
        }
    );
    s.validate().unwrap();
    let mut s = state();
    let b = acquire(&mut s, "alice");
    assert_eq!(
        s.release("alice").unwrap(),
        Outcome::Released {
            address: b.address,
            server_notified: true
        }
    );
    assert!(!s.bindings().contains_key("alice"));
    assert_eq!(offer(&mut s, "bob").address, b.address);
}
#[test]
fn stale_offer_is_rechecked_then_retry_works_without_duplicate_binding() {
    let mut s = state();
    let old = offer(&mut s, "alice");
    policy(&mut s, |p| {
        p.pools
            .get_mut("alpha")
            .unwrap()
            .excluded
            .insert(old.address);
    });
    assert_eq!(
        s.request("alice", old.token).unwrap(),
        Outcome::Nak {
            address: old.address
        }
    );
    assert!(s.leases().is_empty() && s.offers().is_empty());
    let new = offer(&mut s, "alice");
    assert_ne!(new.address, old.address);
    let before = s.clone();
    assert_eq!(
        s.request("alice", old.token).unwrap(),
        Outcome::NoReply(Reason::StaleOffer)
    );
    assert_eq!(s, before);
    assert!(matches!(
        s.request("alice", new.token).unwrap(),
        Outcome::Ack { .. }
    ));
    acquire(&mut s, "bob");
    assert_ne!(s.bindings()["alice"].address, s.bindings()["bob"].address);
    s.validate().unwrap();
}
#[test]
fn expired_offer_and_clock_windows_never_auto_send_or_fabricate_nak() {
    let mut s = state();
    let o = offer(&mut s, "alice");
    s.advance(30).unwrap();
    assert_eq!(
        s.request("alice", o.token).unwrap(),
        Outcome::NoReply(Reason::NoOffer)
    );
    let b = acquire(&mut s, "alice");
    s.advance(8).unwrap();
    assert_eq!(s.window("alice").unwrap(), Window::Renewal);
    assert_eq!(s.leases()["alice"], b);
    s.advance(6).unwrap();
    assert_eq!(s.window("alice").unwrap(), Window::Rebinding);
    s.advance(2).unwrap();
    assert_eq!(s.window("alice").unwrap(), Window::Unconfigured);
    assert!(!s.bindings().contains_key("alice"));
}
#[test]
fn full_ipv4_arithmetic_crosses_octets_without_scanning_the_whole_subnet() {
    let subnet = net("10.20.0.0", 16);
    let segment = Segment {
        subnet,
        server_interface: Some(ip("10.20.0.1")),
        relay_interface: None,
        relay_to_server: false,
        client_unicast_to_server: true,
        occupied: BTreeSet::new(),
    };
    let t = Topology {
        server: ip("10.20.0.1"),
        segments: BTreeMap::from([("wide".into(), segment)]),
        clients: (0..12)
            .map(|n| {
                (
                    format!("c{n}"),
                    Client {
                        segment: "wide".into(),
                    },
                )
            })
            .collect(),
    };
    let p = Policy {
        server_enabled: true,
        pools: BTreeMap::from([("wide".into(), pool(subnet, "10.20.0.250", "10.20.1.5"))]),
        relays: BTreeMap::new(),
    };
    let mut s = State::new(t, p).unwrap();
    let first = u32::from(ip("10.20.0.250"));
    for n in 0..12 {
        assert_eq!(
            acquire(&mut s, &format!("c{n}")).address,
            Ipv4Addr::from(first + n)
        );
    }
    assert_eq!(
        s.leases()["c6"].address,
        ip("10.20.1.0"),
        ".0 is a host in this /16"
    );
}
#[test]
fn well_formed_wrong_options_and_subnet_are_retained_not_rewritten_as_correct() {
    let mut s = state();
    policy(&mut s, |p| {
        let a = p.pools.get_mut("alpha").unwrap();
        a.subnet = net("192.0.0.0", 16);
        a.first = ip("192.0.3.10");
        a.last = ip("192.0.3.12");
        a.router = Some(ip("0.0.0.0"));
        a.dns = vec![ip("255.255.255.255")];
    });
    let b = acquire(&mut s, "alice");
    assert_ne!(b.parameters.subnet, s.topology().segments["alpha"].subnet);
    assert_eq!(b.address, ip("192.0.3.10"));
    assert_eq!(b.parameters.router, Some(ip("0.0.0.0")));
    assert_eq!(b.parameters.dns, vec![ip("255.255.255.255")]);
    s.validate().unwrap();
}
#[test]
fn invalid_policy_shape_and_action_errors_leave_exact_state() {
    let mut s = state();
    acquire(&mut s, "alice");
    for action in 0..5 {
        let before = s.clone();
        let r = match action {
            0 => s.discover("alice"),
            1 => s.release("bob"),
            2 => s.rebind("alice"),
            3 => s.renew("unknown"),
            _ => s.advance(MAX_ADVANCE + 1),
        };
        assert!(r.is_err());
        assert_eq!(s, before);
    }
    for kind in 0..7 {
        let mut p = s.policy().clone();
        let a = p.pools.get_mut("alpha").unwrap();
        match kind {
            0 => a.first = ip("192.0.2.0"),
            1 => a.last = ip("192.0.2.255"),
            2 => a.first = ip("192.0.2.100"),
            3 => a.lease_seconds = 7,
            4 => a.dns = vec![ip("203.0.113.1"); 5],
            5 => a.subnet.prefix = 31,
            _ => {
                a.reservations.insert("unknown".into(), ip("192.0.2.50"));
            }
        }
        let before = s.clone();
        assert!(s.replace_policy(p).is_err());
        assert_eq!(s, before);
    }
    let mut foreign = s.topology().clone();
    foreign.clients.get_mut("alice").unwrap().segment = "beta".into();
    assert!(s.validate_context(&foreign).is_err());
    s.validate_context(s.topology()).unwrap();
}
#[test]
fn clock_and_counter_overflow_reject_before_any_publication() {
    let s = state();
    let mut near = set_clock(&s, MAX_TIME - 10);
    let before = near.clone();
    assert!(near.advance(11).is_err());
    assert_eq!(near, before);
    assert!(near.discover("alice").is_err());
    assert_eq!(near, before);
    let mut near = set_clock(&s, MAX_TIME - 50);
    policy(&mut near, |p| {
        p.pools.get_mut("alpha").unwrap().lease_seconds = MAX_LEASE
    });
    let o = offer(&mut near, "alice");
    let before = near.clone();
    assert!(near.request("alice", o.token).is_err());
    assert_eq!(near, before);
    let mut v = serde_json::to_value(&s).unwrap();
    v["next_token"] = serde_json::json!(MAX_TIME);
    let mut full: State = serde_json::from_value(v).unwrap();
    let before = full.clone();
    assert!(full.discover("alice").is_err());
    assert_eq!(full, before);
}
#[test]
fn maximum_shapes_roundtrip_and_typed_readers_reject_duplicates_unknowns_and_overflow() {
    let mut t = Topology {
        server: ip("10.0.0.1"),
        segments: BTreeMap::new(),
        clients: BTreeMap::new(),
    };
    let mut p = Policy {
        server_enabled: true,
        pools: BTreeMap::new(),
        relays: BTreeMap::new(),
    };
    for n in 0..8 {
        let segment = format!("segment-{n}-{}", "s".repeat(38));
        let scope = net(&format!("10.{n}.0.0"), 16);
        t.segments.insert(
            segment.clone(),
            Segment {
                subnet: scope,
                server_interface: Some(ip(&format!("10.{n}.0.1"))),
                relay_interface: None,
                relay_to_server: false,
                client_unicast_to_server: false,
                occupied: BTreeSet::new(),
            },
        );
        let mut pool = pool(scope, &format!("10.{n}.10.1"), &format!("10.{n}.11.0"));
        pool.dns = vec![
            ip("223.255.255.254"),
            ip("203.0.113.254"),
            ip("198.51.100.254"),
            ip("192.0.2.254"),
        ];
        pool.lease_seconds = MAX_LEASE;
        let base = u32::from(ip(&format!("10.{n}.20.1")));
        pool.excluded = (0..256).map(|i| Ipv4Addr::from(base + i)).collect();
        for c in 0..4 {
            let client = format!("client-{n}-{c}-{}", "c".repeat(37));
            assert_eq!(client.len(), 48);
            t.clients.insert(
                client.clone(),
                Client {
                    segment: segment.clone(),
                },
            );
            pool.reservations
                .insert(client, ip(&format!("10.{n}.30.{}", c + 1)));
        }
        assert_eq!(segment.len(), 48);
        p.pools.insert(format!("pool-{n}-{}", "p".repeat(41)), pool);
    }
    let clients: Vec<_> = t.clients.keys().cloned().collect();
    let mut s = State::new(t, p).unwrap();
    for c in &clients {
        acquire(&mut s, c);
        s.release(c).unwrap();
    }
    for c in &clients {
        offer(&mut s, c);
    }
    assert_eq!(s.bindings().len(), MAX_CLIENTS);
    assert_eq!(s.offers().len(), MAX_CLIENTS);
    for pretty in [false, true] {
        let bytes = if pretty {
            serde_json::to_vec_pretty(&s)
        } else {
            serde_json::to_vec(&s)
        }
        .unwrap();
        assert!(bytes.len() < MAX_INPUT);
        assert_eq!(State::decode(&bytes).unwrap(), s);
    }
    assert!(State::decode(&vec![b' '; MAX_INPUT + 1]).is_err());
    let v = serde_json::to_value(state()).unwrap();
    for kind in 0..7 {
        let mut bad = v.clone();
        match kind {
            0 => {
                bad["extra"] = serde_json::json!(true);
            }
            1 => {
                bad["topology"]["clients"]["unknown"] = serde_json::json!({"segment":"missing"});
            }
            2 => {
                bad["policy"]["pools"]["alpha"]["dns"] = serde_json::json!(vec!["1.1.1.1"; 5]);
            }
            3 => {
                bad["policy"]["pools"]["alpha"]["excluded"] =
                    serde_json::json!(["192.0.2.10", "192.0.2.10"]);
            }
            4 => {
                bad["now"] = serde_json::json!(MAX_TIME + 1);
            }
            5 => {
                bad["topology"]["clients"]["alice"]["segment"] = serde_json::json!("x".repeat(49));
            }
            _ => {
                bad["policy"]["pools"]["alpha"]["dns"] = serde_json::json!(["1".repeat(16)]);
            }
        }
        let b = serde_json::to_vec(&bad).unwrap();
        let e = State::decode(&b).unwrap_err();
        assert!(e.message.len() <= MAX_ERROR && e.message.capacity() <= MAX_ERROR);
    }
    let raw = serde_json::to_string(&state()).unwrap().replacen(
        "\"server_enabled\":true",
        "\"server_enabled\":true,\"server_enabled\":false",
        1,
    );
    assert!(State::decode(raw.as_bytes()).is_err());
    let raw = serde_json::to_string(&state()).unwrap().replacen(
        "\"alice\":{\"segment\":\"alpha\"}",
        "\"alice\":{\"segment\":\"alpha\"},\"alice\":{\"segment\":\"alpha\"}",
        1,
    );
    assert!(State::decode(raw.as_bytes()).is_err());
}
#[test]
fn imported_duplicate_active_addresses_or_foreign_client_copies_are_rejected() {
    let mut s = state();
    acquire(&mut s, "alice");
    acquire(&mut s, "bob");
    let original = serde_json::to_value(&s).unwrap();
    let mut bad = original.clone();
    bad["bindings"]["bob"] = bad["bindings"]["alice"].clone();
    bad["leases"]["bob"] = bad["bindings"]["alice"].clone();
    assert!(serde_json::from_value::<State>(bad).is_err());
    let mut infra = original.clone();
    infra["bindings"]["alice"]["address"] = serde_json::json!("192.0.2.1");
    infra["leases"]["alice"]["address"] = serde_json::json!("192.0.2.1");
    assert!(serde_json::from_value::<State>(infra).is_err());
    let mut pending = state();
    offer(&mut pending, "alice");
    let mut infra = serde_json::to_value(pending).unwrap();
    infra["offers"]["alice"]["address"] = serde_json::json!("192.0.2.1");
    assert!(serde_json::from_value::<State>(infra).is_err());
    let mut bad = original;
    bad["leases"]["alice"]["parameters"]["dns"] = serde_json::json!(["203.0.113.99"]);
    assert!(serde_json::from_value::<State>(bad).is_err());
}

#[test]
fn range_limit_and_typed_collection_limits_reject_before_oversized_values() {
    let (t, mut p) = fixture();
    let mut wide = pool(net("10.20.0.0", 16), "10.20.0.1", "10.20.1.0");
    p.pools.clear();
    p.pools.insert("wide".into(), wide.clone());
    p.validate(&t).unwrap();
    wide.last = ip("10.20.1.1");
    p.pools.insert("wide".into(), wide);
    assert!(p.validate(&t).is_err());
    let single =
        serde_json::to_string(&pool(net("192.0.2.0", 24), "192.0.2.10", "192.0.2.12")).unwrap();
    let mut entries = (0..8)
        .map(|n| format!("\"p{n}\":{single}"))
        .collect::<Vec<_>>();
    entries.push("\"p8\":null".into());
    let source = format!(
        "{{\"server_enabled\":true,\"pools\":{{{}}},\"relays\":{{}}}}",
        entries.join(",")
    );
    let e = serde_json::from_str::<Policy>(&source).unwrap_err();
    assert!(e.to_string().contains("count/duplicate"));
    let mut bad = state().policy().clone();
    bad.pools.get_mut("alpha").unwrap().dns = vec![ip("1.1.1.1"); 5];
    let mut s = state();
    let before = s.clone();
    assert!(s.replace_policy(bad).is_err());
    assert_eq!(s, before);
}

#[test]
fn rediscovered_held_binding_advertises_remaining_seconds_without_inventing_new_timers() {
    let (mut t, p) = fixture();
    t.segments.get_mut("beta").unwrap().client_unicast_to_server = false;
    let mut s = State::new(t, p).unwrap();
    let original = acquire(&mut s, "carol");
    s.release("carol").unwrap();
    s.advance(15).unwrap();
    let o = offer(&mut s, "carol");
    assert!(o.existing_binding);
    assert_eq!(o.lease_seconds, 1);
    assert_eq!(o.expires_at, 16);
    assert_eq!(
        s.request("carol", o.token).unwrap(),
        Outcome::Ack {
            binding: original,
            kind: AckKind::CurrentLease
        }
    );
    assert_eq!(s.leases()["carol"].remaining(s.now()), 1);
    s.advance(1).unwrap();
    assert!(s.leases().is_empty() && s.bindings().is_empty());
}
#[test]
fn changed_scope_silently_diagnoses_stale_exchange_and_new_discovery_uses_current_policy() {
    let mut s = state();
    let old = offer(&mut s, "alice");
    policy(&mut s, |p| {
        let a = p.pools.get_mut("alpha").unwrap();
        a.subnet = net("192.0.0.0", 16);
        a.first = ip("192.0.3.10");
        a.last = ip("192.0.3.12");
    });
    let before = s.clone();
    assert_eq!(
        s.request("alice", old.token).unwrap(),
        Outcome::NoReply(Reason::PoolChanged)
    );
    assert_eq!(s, before);
    let o = offer(&mut s, "alice");
    assert_eq!(o.address, ip("192.0.3.10"));
    assert_ne!(o.parameters.subnet, old.parameters.subnet);
    assert!(matches!(
        s.request("alice", o.token).unwrap(),
        Outcome::Ack {
            kind: AckKind::NewLease,
            ..
        }
    ));
    s.validate().unwrap();
}

#[test]
fn a_new_reservation_cannot_steal_a_live_binding_from_another_client() {
    let mut s = state();
    let original = acquire(&mut s, "alice");
    policy(&mut s, |p| {
        p.pools
            .get_mut("alpha")
            .unwrap()
            .reservations
            .insert("bob".into(), original.address);
    });
    assert_eq!(
        s.discover("bob").unwrap(),
        Outcome::NoReply(Reason::ReservationUnavailable)
    );
    s.advance(2).unwrap();
    assert_eq!(
        s.renew("alice").unwrap(),
        Outcome::Ack {
            binding: original.clone(),
            kind: AckKind::CurrentLease
        }
    );
    assert_eq!(s.leases()["alice"], original);
    s.release("alice").unwrap();
    assert_eq!(acquire(&mut s, "bob").address, original.address);
    assert_ne!(acquire(&mut s, "alice").address, original.address);
    s.validate().unwrap();
}
#[test]
fn direct_collection_overflow_is_rejected_before_derived_context_sets_or_publication() {
    let (t, p) = fixture();
    let mut too_many = t.clone();
    let example = t.segments["alpha"].clone();
    for n in 0..9 {
        too_many
            .segments
            .insert(format!("extra-{n}"), example.clone());
    }
    assert_eq!(
        State::new(too_many, p.clone()).unwrap_err().kind,
        ErrorKind::Limit
    );
    let mut s = State::new(t, p).unwrap();
    let before = s.clone();
    let mut too_many = s.policy().clone();
    let example = too_many.pools["alpha"].clone();
    for n in 0..9 {
        too_many.pools.insert(format!("extra-{n}"), example.clone());
    }
    assert_eq!(
        s.replace_policy(too_many).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
}

#[test]
fn removed_original_binding_and_offer_cannot_be_moved_to_a_foreign_segment_selector() {
    let mut s = state();
    acquire(&mut s, "alice");
    let original = serde_json::to_value(&s).unwrap();
    assert_eq!(
        serde_json::from_value::<State>(original.clone()).unwrap(),
        s
    );
    let mut moved = original;
    let binding = moved["bindings"]
        .as_object_mut()
        .unwrap()
        .remove("alice")
        .unwrap();
    let lease = moved["leases"]
        .as_object_mut()
        .unwrap()
        .remove("alice")
        .unwrap();
    moved["bindings"]["carol"] = binding;
    moved["leases"]["carol"] = lease;
    assert_eq!(moved["bindings"].as_object().unwrap().len(), 1);
    assert_eq!(moved["leases"].as_object().unwrap().len(), 1);
    assert!(
        serde_json::from_value::<State>(moved).is_err(),
        "known client and unique address alone do not pin segment context"
    );
    let mut s = state();
    offer(&mut s, "alice");
    let original = serde_json::to_value(&s).unwrap();
    assert_eq!(
        serde_json::from_value::<State>(original.clone()).unwrap(),
        s
    );
    let mut moved = original;
    let mut pending = moved["offers"]
        .as_object_mut()
        .unwrap()
        .remove("alice")
        .unwrap();
    pending["delivery"] =
        serde_json::json!({"kind":"Relay","segment":"beta","giaddr":"198.51.100.1"});
    moved["offers"]["carol"] = pending;
    assert_eq!(moved["offers"].as_object().unwrap().len(), 1);
    assert!(
        serde_json::from_value::<State>(moved).is_err(),
        "even a matching new delivery record must retain a compatible copied subnet"
    );
}
