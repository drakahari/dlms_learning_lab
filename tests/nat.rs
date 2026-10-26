//! Prospective pure Basic NAT tests; no host references or executed evidence.
use dlms_learning_lab::nat::{
    self, Action, Addresses, Configuration, DynamicRule, Effect, FaultKind, Inventory, MappingKind,
    Pool, Prefix, ProbeResult, State,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn inventory() -> Inventory {
    Inventory {
        fixture: "nat-lab".into(),
        inside: BTreeMap::from([
            ("alpha".into(), ip("10.10.0.10")),
            ("beta".into(), ip("10.10.0.11")),
            ("gamma".into(), ip("10.10.0.12")),
        ]),
        outside: BTreeMap::from([
            ("peer-one".into(), ip("203.0.113.7")),
            ("peer-two".into(), ip("203.0.113.8")),
        ]),
        globals: [
            "198.51.100.250",
            "198.51.100.251",
            "198.51.100.252",
            "198.51.100.253",
            "198.51.100.254",
            "198.51.100.255",
            "198.51.101.0",
            "198.51.101.1",
        ]
        .into_iter()
        .map(ip)
        .collect(),
    }
}
fn configuration() -> Configuration {
    Configuration {
        statics: BTreeMap::from([("alpha".into(), ip("198.51.100.254"))]),
        dynamic: Some(DynamicRule {
            source: Prefix {
                network: ip("10.10.0.0"),
                length: 24,
            },
            pool: Pool {
                first: ip("198.51.100.254"),
                last: ip("198.51.101.1"),
            },
        }),
    }
}
fn state() -> State {
    State::new(&inventory(), &configuration()).unwrap()
}
fn out(s: &mut State, inside: &str, peer: &str) -> ProbeResult {
    let Effect::Probe { result } = s
        .apply(&Action::Outbound {
            inside: inside.into(),
            peer: peer.into(),
        })
        .unwrap()
    else {
        panic!()
    };
    result
}
fn inbound(s: &mut State, peer: &str, global: &str) -> ProbeResult {
    let Effect::Probe { result } = s
        .apply(&Action::Inbound {
            peer: peer.into(),
            global: ip(global),
        })
        .unwrap()
    else {
        panic!()
    };
    result
}
fn translated(p: ProbeResult) -> (Addresses, Addresses, MappingKind) {
    let ProbeResult::Translated {
        before,
        after,
        kind,
    } = p
    else {
        panic!("translation expected")
    };
    (before, after, kind)
}
fn atomic_error(s: &mut State, a: &Action, kind: FaultKind) {
    let before = serde_json::to_vec(&*s).unwrap();
    let error = s.apply(a).unwrap_err();
    assert_eq!(error.kind, kind);
    assert!(
        !error.message.is_empty() && error.message.len() <= 512 && error.message.capacity() <= 512
    );
    assert!(
        serde_json::to_vec(&*s).unwrap() == before,
        "failed action mutated full State"
    );
}
#[test]
fn independent_prefix_and_pool_arithmetic_vectors_use_all32_bits() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nat-address-vectors.json")).unwrap();
    for r in v["prefixes"].as_array().unwrap() {
        let p = Prefix {
            network: ip(r["network"].as_str().unwrap()),
            length: r["length"].as_u64().unwrap() as u8,
        };
        assert_eq!(p.valid(), r["valid"].as_bool().unwrap());
        assert_eq!(
            p.contains(ip(r["address"].as_str().unwrap())),
            r["contains"].as_bool().unwrap()
        );
    }
    for r in v["pools"].as_array().unwrap() {
        let p = Pool {
            first: ip(r["first"].as_str().unwrap()),
            last: ip(r["last"].as_str().unwrap()),
        };
        if let Some(n) = r["count"].as_u64() {
            assert_eq!(p.cardinality().unwrap(), n)
        } else {
            assert_eq!(
                format!("{:?}", p.cardinality().unwrap_err().kind),
                r["error"].as_str().unwrap()
            )
        }
    }
}
#[test]
fn independent_full_two_way_translation_vectors_preserve_other_address() {
    let mut s = state();
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nat-address-vectors.json")).unwrap();
    for r in v["translation"].as_array().unwrap() {
        let p = if r["direction"] == "out" {
            out(
                &mut s,
                r["inside"].as_str().unwrap(),
                r["peer"].as_str().unwrap(),
            )
        } else {
            inbound(
                &mut s,
                r["peer"].as_str().unwrap(),
                r["global"].as_str().unwrap(),
            )
        };
        let (before, after, kind) = translated(p);
        assert_eq!(
            [before.source.to_string(), before.destination.to_string()],
            [
                r["before"][0].as_str().unwrap(),
                r["before"][1].as_str().unwrap()
            ]
        );
        assert_eq!(
            [after.source.to_string(), after.destination.to_string()],
            [
                r["after"][0].as_str().unwrap(),
                r["after"][1].as_str().unwrap()
            ]
        );
        assert_eq!(format!("{kind:?}"), r["kind"].as_str().unwrap());
    }
}
#[test]
fn static_inbound_exists_before_outbound_and_survives_clear() {
    let mut s = state();
    assert!(s.bindings().is_empty());
    assert_eq!(
        translated(inbound(&mut s, "peer-one", "198.51.100.254"))
            .1
            .destination,
        ip("10.10.0.10")
    );
    out(&mut s, "beta", "peer-one");
    assert_eq!(
        s.apply(&Action::ClearDynamic {}).unwrap(),
        Effect::Cleared { removed: 1 }
    );
    assert!(s.bindings().is_empty());
    assert_eq!(
        translated(inbound(&mut s, "peer-two", "198.51.100.254")).2,
        MappingKind::Static
    );
}
#[test]
fn one_host_reuses_address_across_destinations_without_session_authority() {
    let mut s = state();
    let first = translated(out(&mut s, "beta", "peer-one"));
    let bytes = serde_json::to_vec(&s).unwrap();
    let next = translated(out(&mut s, "beta", "peer-two"));
    assert_eq!(first.1.source, next.1.source);
    assert_eq!(next.1.destination, ip("203.0.113.8"));
    assert!(serde_json::to_vec(&s).unwrap() == bytes);
    assert_eq!(
        translated(inbound(&mut s, "peer-two", "198.51.100.255"))
            .1
            .destination,
        ip("10.10.0.11")
    );
}
#[test]
fn finite_pool_exhaustion_and_unsolicited_inbound_never_allocate() {
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().pool.last = ip("198.51.100.255");
    let mut s = State::new(&inventory(), &c).unwrap();
    let before = serde_json::to_vec(&s).unwrap();
    assert!(matches!(
        inbound(&mut s, "peer-one", "198.51.100.255"),
        ProbeResult::NoMapping { .. }
    ));
    assert!(serde_json::to_vec(&s).unwrap() == before);
    out(&mut s, "beta", "peer-one");
    let before = serde_json::to_vec(&s).unwrap();
    assert!(matches!(
        out(&mut s, "gamma", "peer-two"),
        ProbeResult::PoolExhausted { .. }
    ));
    assert!(serde_json::to_vec(&s).unwrap() == before);
    assert!(matches!(
        inbound(&mut s, "peer-one", "192.0.2.200"),
        ProbeResult::NoMapping { .. }
    ));
    assert!(serde_json::to_vec(&s).unwrap() == before);
}
#[test]
fn static_globals_are_reserved_even_inside_dynamic_pool() {
    let mut s = state();
    assert_eq!(
        s.snapshot().unwrap().free_pool,
        vec![ip("198.51.100.255"), ip("198.51.101.0"), ip("198.51.101.1")]
    );
    assert_eq!(
        translated(out(&mut s, "beta", "peer-one")).1.source,
        ip("198.51.100.255")
    );
    assert_eq!(
        translated(out(&mut s, "gamma", "peer-one")).1.source,
        ip("198.51.101.0")
    );
}
#[test]
fn unmatched_outbound_is_typed_untranslated_not_connectivity_or_denial() {
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().source = Prefix {
        network: ip("10.10.0.11"),
        length: 32,
    };
    let mut s = State::new(&inventory(), &c).unwrap();
    let before = s.clone();
    assert_eq!(
        out(&mut s, "gamma", "peer-one"),
        ProbeResult::Untranslated {
            addresses: Addresses {
                source: ip("10.10.0.12"),
                destination: ip("203.0.113.7")
            }
        }
    );
    assert!(s == before);
    assert_eq!(
        translated(out(&mut s, "alpha", "peer-two")).2,
        MappingKind::Static
    );
}
#[test]
fn conflicting_policy_edits_require_explicit_clear_and_are_atomic() {
    let mut s = state();
    out(&mut s, "beta", "peer-one");
    let mut changes = vec![];
    let mut c = configuration();
    c.dynamic = None;
    changes.push(c);
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().source = Prefix {
        network: ip("10.10.0.10"),
        length: 32,
    };
    changes.push(c);
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().pool.first = ip("198.51.101.0");
    changes.push(c);
    let mut c = configuration();
    c.statics.insert("gamma".into(), ip("198.51.100.255"));
    changes.push(c);
    let mut c = configuration();
    c.statics.insert("beta".into(), ip("198.51.100.255"));
    changes.push(c);
    for c in changes {
        atomic_error(
            &mut s,
            &Action::Configure { configuration: c },
            FaultKind::ClearRequired,
        );
    }
}
#[test]
fn compatible_policy_edit_preserves_exact_bindings_and_clear_releases_capacity() {
    let mut s = state();
    out(&mut s, "beta", "peer-one");
    let binding = s.bindings().clone();
    let mut c = configuration();
    c.statics.insert("gamma".into(), ip("198.51.101.1"));
    assert_eq!(
        s.apply(&Action::Configure {
            configuration: c.clone()
        })
        .unwrap(),
        Effect::Configured
    );
    assert_eq!(s.bindings(), &binding);
    s.apply(&Action::ClearDynamic {}).unwrap();
    c.statics.remove("gamma");
    s.apply(&Action::Configure { configuration: c }).unwrap();
    assert_eq!(
        translated(out(&mut s, "gamma", "peer-one")).1.source,
        ip("198.51.100.255")
    );
}
#[test]
fn duplicate_static_globals_and_foreign_configuration_fail_without_mutation() {
    let mut s = state();
    let mut c = configuration();
    c.statics.insert("beta".into(), ip("198.51.100.254"));
    atomic_error(
        &mut s,
        &Action::Configure { configuration: c },
        FaultKind::Invalid,
    );
    let mut c = configuration();
    c.statics.insert("foreign".into(), ip("198.51.101.1"));
    atomic_error(
        &mut s,
        &Action::Configure { configuration: c },
        FaultKind::Context,
    );
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().pool.last = ip("198.51.101.2");
    atomic_error(
        &mut s,
        &Action::Configure { configuration: c },
        FaultKind::Context,
    );
}
#[test]
fn disjoint_inventory_rejects_duplicate_address_ids_and_global_aliases() {
    let c = configuration();
    let mut i = inventory();
    i.outside.insert("alpha".into(), ip("203.0.113.9"));
    assert_eq!(State::new(&i, &c).unwrap_err().kind, FaultKind::Unsupported);
    let mut i = inventory();
    i.outside.insert("peer-three".into(), ip("10.10.0.10"));
    assert_eq!(State::new(&i, &c).unwrap_err().kind, FaultKind::Unsupported);
    let mut i = inventory();
    i.globals.insert(ip("10.10.0.10"));
    assert_eq!(State::new(&i, &c).unwrap_err().kind, FaultKind::Unsupported);
}
#[test]
fn unknown_probe_id_and_oversized_direct_arguments_are_atomic_errors() {
    let mut s = state();
    atomic_error(
        &mut s,
        &Action::Outbound {
            inside: "foreign".into(),
            peer: "peer-one".into(),
        },
        FaultKind::Context,
    );
    atomic_error(
        &mut s,
        &Action::Inbound {
            peer: "foreign".into(),
            global: ip("198.51.101.1"),
        },
        FaultKind::Context,
    );
    atomic_error(
        &mut s,
        &Action::Outbound {
            inside: "a".repeat(100_000),
            peer: "peer-one".into(),
        },
        FaultKind::Invalid,
    );
    let mut c = configuration();
    c.statics.insert("a".repeat(100_000), ip("198.51.101.1"));
    atomic_error(
        &mut s,
        &Action::Configure { configuration: c },
        FaultKind::Invalid,
    );
}
#[test]
fn range_and_count_admission_precede_expansion_and_bounded_clone() {
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().pool = Pool {
        first: ip("0.0.0.0"),
        last: ip("255.255.255.255"),
    };
    assert_eq!(
        State::new(&inventory(), &c).unwrap_err().kind,
        FaultKind::Limit
    );
    let mut i = inventory();
    for n in 0..65 {
        i.inside
            .insert(format!("host{n}"), Ipv4Addr::from(0x0a010000_u32 + n));
    }
    assert_eq!(
        State::new(&i, &configuration()).unwrap_err().kind,
        FaultKind::Limit
    );
    let mut c = configuration();
    for n in 0..33 {
        c.statics.insert(format!("host{n}"), ip("198.51.101.1"));
    }
    assert_eq!(
        State::new(&inventory(), &c).unwrap_err().kind,
        FaultKind::Limit
    );
}
#[test]
fn bounded_typed_map_refuses_excess_key_before_its_value_is_typed() {
    let i = inventory();
    let mut value =
        serde_json::to_value(State::new(&i, &Configuration::default()).unwrap()).unwrap();
    value["bindings"] = serde_json::json!({});
    let mut text = serde_json::to_string(&value).unwrap();
    let members = (0..64)
        .map(|n| format!("\"host{n}\":\"198.51.100.1\""))
        .collect::<Vec<_>>()
        .join(",");
    text = text.replace(
        "\"bindings\":{}",
        &format!("\"bindings\":{{{members},\"excess\":{{\"not\":\"an address\"}}}}"),
    );
    let error = serde_json::from_str::<State>(&text)
        .unwrap_err()
        .to_string();
    assert!(error.contains("map count limit"));
    let raw = "{\"fixture\":\"nat-lab\",\"inside\":{\"alpha\":\"10.0.0.1\",\"alpha\":{}},\"outside\":{},\"globals\":[]}";
    assert!(
        serde_json::from_str::<Inventory>(raw)
            .unwrap_err()
            .to_string()
            .contains("Duplicate NAT identity")
    );
}
#[test]
fn raw_caps_unknown_fields_empty_struct_and_utf8_diagnostics_are_bounded() {
    let mut s = state();
    // All controls have no live-binding side effect in this static/empty state.
    let controls = [
        Action::Configure {
            configuration: configuration(),
        },
        Action::ClearDynamic {},
        Action::Outbound {
            inside: "alpha".into(),
            peer: "peer-one".into(),
        },
        Action::Inbound {
            peer: "peer-one".into(),
            global: ip("198.51.100.254"),
        },
    ];
    for control in controls {
        let raw = serde_json::to_vec(&control).unwrap();
        assert_eq!(Action::from_json(&raw).unwrap(), control);
        let before = serde_json::to_vec(&s).unwrap();
        s.apply_json(&raw).unwrap();
        assert!(
            serde_json::to_vec(&s).unwrap() == before,
            "valid static control changed State"
        );
        let value = serde_json::to_value(&control).unwrap();
        let mut unknown = value.clone();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), true.into());
        let unknown_raw = serde_json::to_vec(&unknown).unwrap();
        assert!(Action::from_json(&unknown_raw).is_err());
        assert!(s.apply_json(&unknown_raw).is_err());
        assert!(
            serde_json::to_vec(&s).unwrap() == before,
            "unknown action field changed State"
        );
        let text = String::from_utf8(raw).unwrap();
        let body = text.strip_suffix('}').unwrap();
        // Feed duplicate keys as raw bytes, never through a Value that erases them.
        for tag in [value["kind"].as_str().unwrap(), "ClearDynamic"] {
            let duplicate = format!("{body},\"kind\":\"{tag}\"}}");
            assert!(Action::from_json(duplicate.as_bytes()).is_err());
            assert!(s.apply_json(duplicate.as_bytes()).is_err());
            assert!(
                serde_json::to_vec(&s).unwrap() == before,
                "duplicate kind changed State"
            );
        }
    }
    let valid_state = serde_json::to_vec(&s).unwrap();
    assert!(State::from_json(&valid_state).unwrap() == s);
    let mut future = serde_json::to_value(&s).unwrap();
    future["schema_version"] = 2.into();
    let future_raw = serde_json::to_vec(&future).unwrap();
    assert!(State::from_json(&future_raw).is_err());
    assert!(State::from_json_for(&future_raw, &inventory()).is_err());
    assert!(
        serde_json::to_vec(&s).unwrap() == valid_state,
        "future version changed State"
    );
    // Malformed over-cap input still reports raw Limit before serde syntax errors.
    assert_eq!(
        State::from_json(&vec![b'!'; nat::MAX_STATE_JSON + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let invalid_action = vec![b'!'; nat::MAX_ACTION_JSON + 1];
    assert_eq!(
        Action::from_json(&invalid_action).unwrap_err().kind,
        FaultKind::Limit
    );
    assert_eq!(
        s.apply_json(&invalid_action).unwrap_err().kind,
        FaultKind::Limit
    );
    assert!(
        serde_json::to_vec(&s).unwrap() == valid_state,
        "oversized action changed State"
    );
    for raw in [
        b"{\"kind\":\"ClearDynamic\",\"extra\":true}".as_slice(),
        b"{\"kind\":\"Expire\"}".as_slice(),
        b"{\"kind\":\"Outbound\",\"inside\":\"bad_id\",\"peer\":\"peer-one\"}".as_slice(),
    ] {
        let before = s.clone();
        assert!(s.apply_json(raw).is_err());
        assert!(s == before);
    }
    let mut exact = serde_json::to_vec(&s).unwrap();
    exact.resize(nat::MAX_STATE_JSON, b' ');
    assert!(State::from_json(&exact).unwrap() == s);
    exact.push(b' ');
    assert_eq!(State::from_json(&exact).unwrap_err().kind, FaultKind::Limit);
    let mut action = serde_json::to_vec(&Action::ClearDynamic {}).unwrap();
    action.resize(nat::MAX_ACTION_JSON, b' ');
    assert_eq!(Action::from_json(&action).unwrap(), Action::ClearDynamic {});
    assert_eq!(
        Action::from_json(&vec![b' '; nat::MAX_ACTION_JSON + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let bad = format!("{{\"{}\":0}}", "é".repeat(20_000));
    let e = State::from_json(bad.as_bytes()).unwrap_err();
    assert!(e.message.len() <= 512 && e.message.capacity() <= 512);
    assert!(std::str::from_utf8(e.message.as_bytes()).is_ok());
}
#[test]
fn roundtrip_context_pin_and_serialization_only_snapshot_cannot_supply_authority() {
    let mut s = state();
    out(&mut s, "beta", "peer-one");
    let raw = serde_json::to_vec(&s).unwrap();
    assert!(State::from_json_for(&raw, &inventory()).unwrap() == s);
    let mut other = inventory();
    other.fixture = "other-initial".into();
    assert_eq!(
        State::from_json_for(&raw, &other).unwrap_err().kind,
        FaultKind::Context
    );
    let snapshot = serde_json::to_vec(&s.snapshot().unwrap()).unwrap();
    assert!(State::from_json(&snapshot).is_err());
    let mut bad: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    bad["bindings"]["beta"] = "198.51.100.254".into();
    assert!(State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
}
#[test]
fn multiple_fresh_host_orders_enforce_policy_not_current_allocation_history() {
    let orders = [
        (
            ["alpha", "beta", "gamma"],
            ["198.51.100.254", "198.51.100.255", "198.51.101.0"],
        ),
        (
            ["gamma", "beta", "alpha"],
            ["198.51.100.255", "198.51.101.0", "198.51.100.254"],
        ),
    ];
    for (order, expected) in orders {
        let mut s = state();
        for (host, global) in order.into_iter().zip(expected) {
            assert_eq!(
                translated(out(&mut s, host, "peer-one")).1.source,
                ip(global)
            );
        }
        assert_eq!(s.bindings().len(), 2);
    }
}
#[test]
fn maximum_legal_shape_and_allocation_fit_existing_raw_caps() {
    let inside = |n| format!("i{n:031}");
    let outside = |n| format!("o{n:031}");
    let i = Inventory {
        fixture: "f".repeat(32),
        inside: (0_u32..64)
            .map(|n| (inside(n), Ipv4Addr::from(0x0a000001_u32 + n)))
            .collect(),
        outside: (0_u32..64)
            .map(|n| (outside(n), Ipv4Addr::from(0xcb007101_u32 + n)))
            .collect(),
        globals: (0_u32..128)
            .map(|n| Ipv4Addr::from(0xc6336400_u32 + n))
            .collect(),
    };
    let c = Configuration {
        statics: (0_u32..32)
            .map(|n| (inside(n), Ipv4Addr::from(0xc6336400_u32 + n)))
            .collect(),
        dynamic: Some(DynamicRule {
            source: Prefix {
                network: ip("10.0.0.0"),
                length: 16,
            },
            pool: Pool {
                first: ip("198.51.100.0"),
                last: ip("198.51.100.63"),
            },
        }),
    };
    let mut s = State::new(&i, &c).unwrap();
    for n in 32_u32..64 {
        assert!(matches!(
            out(&mut s, &inside(n), &outside(0)),
            ProbeResult::Translated { .. }
        ));
    }
    let snapshot = s.snapshot().unwrap();
    assert_eq!(snapshot.mappings.len(), 64);
    assert!(snapshot.free_pool.is_empty());
    let raw = serde_json::to_vec_pretty(&s).unwrap();
    assert!(raw.len() < nat::MAX_STATE_JSON);
    assert!(State::from_json_for(&raw, &i).unwrap() == s);
    assert!(
        serde_json::to_vec(&Action::Configure { configuration: c })
            .unwrap()
            .len()
            < nat::MAX_ACTION_JSON
    );
}
#[test]
fn canonical_prefix_limits_and_global_count_shape_are_directly_checked() {
    let mut c = configuration();
    c.dynamic.as_mut().unwrap().source = Prefix {
        network: ip("10.10.0.1"),
        length: 24,
    };
    assert_eq!(
        State::new(&inventory(), &c).unwrap_err().kind,
        FaultKind::Invalid
    );
    let mut i = inventory();
    i.globals = (0..129)
        .map(|n| Ipv4Addr::from(0xc0000200_u32 + n))
        .collect();
    assert_eq!(
        State::new(&i, &Configuration::default()).unwrap_err().kind,
        FaultKind::Limit
    );
}
#[test]
fn empty_inventory_and_disabled_dynamic_state_are_not_fabricated_translation() {
    let i = Inventory {
        fixture: "empty".into(),
        inside: BTreeMap::new(),
        outside: BTreeMap::new(),
        globals: BTreeSet::new(),
    };
    let mut s = State::new(&i, &Configuration::default()).unwrap();
    assert!(s.snapshot().unwrap().mappings.is_empty());
    atomic_error(
        &mut s,
        &Action::Outbound {
            inside: "alpha".into(),
            peer: "peer-one".into(),
        },
        FaultKind::Context,
    );
    assert_eq!(
        s.apply(&Action::ClearDynamic {}).unwrap(),
        Effect::Cleared { removed: 0 }
    );
}
