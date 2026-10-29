//! Prospective pure-model gates. No device, host command or network access.
use dlms_learning_lab::physical_link::*;
use std::collections::{BTreeMap, BTreeSet};
fn ability(rate: Rate, duplex: Duplex) -> Ability {
    Ability { rate, duplex }
}
fn abilities() -> BTreeSet<Ability> {
    BTreeSet::from([
        ability(Rate::M10, Duplex::Half),
        ability(Rate::M10, Duplex::Full),
        ability(Rate::M100, Duplex::Half),
        ability(Rate::M100, Duplex::Full),
        ability(Rate::M1000, Duplex::Full),
    ])
}
fn automatic() -> PortConfig {
    PortConfig {
        admin_up: true,
        profile: Profile::CopperRj45,
        negotiation: Negotiation::Auto {
            advertise: abilities(),
        },
    }
}
fn fixed(rate: Rate, duplex: Duplex) -> PortConfig {
    PortConfig {
        admin_up: true,
        profile: Profile::CopperRj45,
        negotiation: Negotiation::Fixed {
            ability: ability(rate, duplex),
        },
    }
}
fn optical(profile: Profile) -> PortConfig {
    PortConfig {
        profile,
        negotiation: Negotiation::Fixed {
            ability: ability(Rate::M1000, Duplex::Full),
        },
        ..automatic()
    }
}
fn base() -> (Topology, Configuration) {
    let ports = ["a", "b", "operations-a", "operations-b"]
        .into_iter()
        .map(|id| {
            (
                id.into(),
                PortSpec {
                    device: format!("device-{id}"),
                    label: format!("Port {id}"),
                    profiles: BTreeSet::from([
                        Profile::CopperRj45,
                        Profile::SxOm2,
                        Profile::LxLhOs2,
                    ]),
                    abilities: abilities(),
                    parallel_detection: true,
                },
            )
        })
        .collect();
    let cables = BTreeMap::from([
        (
            "copper".into(),
            Cable {
                label: "Supplied Cat5e100m".into(),
                kind: CableKind::Cat5e,
                length_m: 100,
                condition: Condition::Clean,
            },
        ),
        (
            "operations".into(),
            Cable {
                label: "Protected operations plant".into(),
                kind: CableKind::Cat5e,
                length_m: 5,
                condition: Condition::Clean,
            },
        ),
        (
            "om2".into(),
            Cable {
                label: "Supplied duplex OM2".into(),
                kind: CableKind::Om2Duplex,
                length_m: 550,
                condition: Condition::Clean,
            },
        ),
        (
            "os2".into(),
            Cable {
                label: "Supplied duplex OS2".into(),
                kind: CableKind::Os2Duplex,
                length_m: 10_000,
                condition: Condition::Clean,
            },
        ),
        (
            "noisy".into(),
            Cable {
                label: "Deterministic receive impairment".into(),
                kind: CableKind::Cat5e,
                length_m: 5,
                condition: Condition::ReceiveImpairment { every: 4 },
            },
        ),
    ]);
    let links = BTreeMap::from([
        (
            "uplink".into(),
            LinkSpec {
                a: "a".into(),
                b: "b".into(),
                required_span_m: 5,
                pattern: ProbePattern::DuplexMismatchContention,
            },
        ),
        (
            "operations-link".into(),
            LinkSpec {
                a: "operations-a".into(),
                b: "operations-b".into(),
                required_span_m: 5,
                pattern: ProbePattern::Serialized,
            },
        ),
    ]);
    let topology = Topology {
        version: 1,
        ports,
        cables,
        links,
    };
    let configuration = Configuration {
        ports: topology
            .ports
            .keys()
            .map(|id| (id.clone(), automatic()))
            .collect(),
        cables: BTreeMap::from([
            ("uplink".into(), "copper".into()),
            ("operations-link".into(), "operations".into()),
        ]),
    };
    (topology, configuration)
}
fn state() -> State {
    let (t, c) = base();
    State::new(t, c).unwrap()
}
fn configure(s: &mut State, p: &str, c: PortConfig) {
    s.perform(&Action::ConfigurePort {
        port: p.into(),
        configuration: c,
    })
    .unwrap();
}
fn select(s: &mut State, cable: &str) {
    s.perform(&Action::SelectCable {
        link: "uplink".into(),
        cable: cable.into(),
    })
    .unwrap();
}
fn probe(s: &mut State, a: u32, b: u32) -> ProbeReport {
    s.perform(&Action::Probe {
        link: "uplink".into(),
        a_frames: a,
        b_frames: b,
    })
    .unwrap()
    .unwrap()
}
#[test]
fn common_advertised_ability_selects_highest_rate_then_full_and_preserves_other_link() {
    let mut s = state();
    let before = s.configuration().ports["operations-a"].clone();
    let r = s.report("uplink").unwrap();
    assert_eq!(r.state, LinkState::Up);
    assert_eq!(
        r.a.unwrap(),
        Operational {
            ability: ability(Rate::M1000, Duplex::Full),
            resolution: Resolution::Advertised
        }
    );
    let mut b = automatic();
    b.negotiation = Negotiation::Auto {
        advertise: BTreeSet::from([
            ability(Rate::M100, Duplex::Half),
            ability(Rate::M100, Duplex::Full),
        ]),
    };
    configure(&mut s, "b", b);
    let r = s.report("uplink").unwrap();
    assert_eq!(r.a.unwrap().ability, ability(Rate::M100, Duplex::Full));
    assert_eq!(s.configuration().ports["operations-a"], before);
    assert_eq!(s.report("operations-link").unwrap().state, LinkState::Up);
}
#[test]
fn documented_legacy_parallel_detection_does_not_discover_forced_duplex() {
    let mut s = state();
    configure(&mut s, "b", fixed(Rate::M100, Duplex::Full));
    let r = s.report("uplink").unwrap();
    assert_eq!(r.state, LinkState::DegradedDuplex);
    assert_eq!(
        r.a.unwrap(),
        Operational {
            ability: ability(Rate::M100, Duplex::Half),
            resolution: Resolution::ParallelDetected
        }
    );
    assert_eq!(r.b.unwrap().ability, ability(Rate::M100, Duplex::Full));
    let report = probe(&mut s, 8, 8);
    assert_eq!(report.after["a"].late_collisions, 8);
    assert_eq!(report.after["b"].collisions, 0);
    assert_eq!(report.after["a"].crc_errors, 0);
    assert_eq!(report.after["b"].crc_errors, 0);
    // A cable is not automatically diagnosed from CRC; this mismatch has no CRC.
    for port in ["a", "b"] {
        assert_eq!(s.counters()[port].received, 0);
    }
}
#[test]
fn either_both_auto_or_matching_forced100full_is_a_real_repair() {
    for repair_auto in [false, true] {
        let mut s = state();
        configure(&mut s, "b", fixed(Rate::M100, Duplex::Full));
        probe(&mut s, 4, 4);
        if repair_auto {
            configure(&mut s, "b", automatic());
        } else {
            configure(&mut s, "a", fixed(Rate::M100, Duplex::Full));
        }
        let r = probe(&mut s, 8, 8);
        assert_eq!(r.link.state, LinkState::Up);
        assert_eq!(r.after["a"].received - r.before["a"].received, 8);
        assert_eq!(r.after["b"].received - r.before["b"].received, 8);
        assert_eq!(
            r.after["a"].late_collisions, r.before["a"].late_collisions,
            "Repairs do not erase cumulative history"
        );
    }
}
#[test]
fn speed_mismatch_and_no_common_advertisement_never_succeed() {
    let mut s = state();
    configure(&mut s, "a", fixed(Rate::M10, Duplex::Full));
    configure(&mut s, "b", fixed(Rate::M100, Duplex::Full));
    assert_eq!(
        s.report("uplink").unwrap().state,
        LinkState::NoCommonAbility
    );
    let before = s.clone();
    assert!(
        s.perform(&Action::Probe {
            link: "uplink".into(),
            a_frames: 4,
            b_frames: 4
        })
        .is_err()
    );
    assert_eq!(s, before);
    let mut ca = automatic();
    ca.negotiation = Negotiation::Auto {
        advertise: BTreeSet::from([ability(Rate::M10, Duplex::Full)]),
    };
    let mut cb = automatic();
    cb.negotiation = Negotiation::Auto {
        advertise: BTreeSet::from([ability(Rate::M100, Duplex::Full)]),
    };
    configure(&mut s, "a", ca);
    configure(&mut s, "b", cb);
    assert_eq!(
        s.report("uplink").unwrap().state,
        LinkState::NoCommonAbility
    );
}
#[test]
fn unsupported_1g_half_fixed_copper_and_optical_autoneg_are_honest_atomic_errors() {
    let mut s = state();
    let initial = s.clone();
    for c in [
        fixed(Rate::M1000, Duplex::Half),
        fixed(Rate::M1000, Duplex::Full),
        PortConfig {
            profile: Profile::SxOm2,
            ..automatic()
        },
    ] {
        assert!(
            s.perform(&Action::ConfigurePort {
                port: "a".into(),
                configuration: c
            })
            .is_err()
        );
        assert_eq!(s, initial);
    }
    let (t, mut c) = base();
    c.ports.get_mut("b").unwrap().negotiation = Negotiation::Fixed {
        ability: ability(Rate::M100, Duplex::Full),
    };
    let mut t = t;
    t.ports.get_mut("a").unwrap().parallel_detection = false;
    let s = State::new(t, c).unwrap();
    assert_eq!(s.report("uplink").unwrap_err().kind, ErrorKind::Unsupported);
}
#[test]
fn media_and_reach_are_qualified_against_named_profiles_not_fake_signal_measurements() {
    for (profile, cable, edge) in [
        (Profile::CopperRj45, "copper", 100),
        (Profile::SxOm2, "om2", 550),
        (Profile::LxLhOs2, "os2", 10_000),
    ] {
        for (distance, qualified) in [(edge, true), (edge + 1, false)] {
            let (mut t, mut c) = base();
            t.cables.get_mut(cable).unwrap().length_m = distance;
            c.cables.insert("uplink".into(), cable.into());
            if profile != Profile::CopperRj45 {
                for p in ["a", "b"] {
                    c.ports.insert(p.into(), optical(profile));
                }
            }
            let mut s = State::new(t, c).unwrap();
            let report = s.report("uplink").unwrap();
            assert_eq!(report.qualification == Qualification::Qualified, qualified);
            assert_eq!(
                report.state,
                if qualified {
                    LinkState::Up
                } else {
                    LinkState::OutsideModel
                }
            );
            if qualified {
                probe(&mut s, 2, 2);
            } else {
                let before = s.clone();
                assert!(
                    s.perform(&Action::Probe {
                        link: "uplink".into(),
                        a_frames: 2,
                        b_frames: 2
                    })
                    .is_err()
                );
                assert_eq!(s, before);
            }
        }
    }
    let mut s = state();
    configure(&mut s, "a", optical(Profile::SxOm2));
    configure(&mut s, "b", optical(Profile::LxLhOs2));
    assert_eq!(
        s.report("uplink").unwrap().qualification,
        Qualification::IncompatibleProfile
    );
    configure(&mut s, "b", optical(Profile::SxOm2));
    assert_eq!(
        s.report("uplink").unwrap().qualification,
        Qualification::IncompatibleCable
    );
    select(&mut s, "om2");
    assert_eq!(s.report("uplink").unwrap().state, LinkState::Up);
}
#[test]
fn admin_and_explicit_disconnection_are_distinct_from_unknown_physical_conformance() {
    let (mut t, c) = base();
    t.cables.get_mut("copper").unwrap().condition = Condition::Disconnected;
    let mut s = State::new(t, c).unwrap();
    assert_eq!(s.report("uplink").unwrap().state, LinkState::Disconnected);
    let mut a = automatic();
    a.admin_up = false;
    configure(&mut s, "a", a);
    assert_eq!(
        s.report("uplink").unwrap().state,
        LinkState::AdministrativelyDown
    );
    assert_eq!(s.counters()["a"], Counters::default());
}
#[test]
fn derived_crc_is_injected_receive_evidence_and_clearing_counters_is_not_a_repair() {
    let mut s = state();
    select(&mut s, "noisy");
    let first = probe(&mut s, 8, 8);
    assert_eq!(first.after["a"].crc_errors, 2);
    assert_eq!(first.after["a"].received, 6);
    assert_eq!(first.link.state, LinkState::Up);
    assert_eq!(first.after["a"].collisions, 0);
    s.perform(&Action::ClearCounters { port: "a".into() })
        .unwrap();
    let next = probe(&mut s, 4, 4);
    assert_eq!(next.after["a"].crc_errors, 1);
    select(&mut s, "copper");
    let clean = probe(&mut s, 4, 4);
    assert_eq!(
        clean.after["a"].crc_errors - clean.before["a"].crc_errors,
        0
    );
    assert_eq!(clean.after["a"].received - clean.before["a"].received, 4);
    let reading = SuppliedReading {
        origin: ReadingOrigin::AuthoredSample,
        note: "Authored earlier observation, not derived by the live model".into(),
        counters: ObservedCounters {
            input_packets: 700,
            output_packets: 800,
            crc_errors: 37,
            collisions: 0,
            late_collisions: 0,
        },
    };
    reading.validate().unwrap();
    assert_eq!(reading.counters.crc_errors, 37);
}
#[test]
fn half_duplex_serial_probe_works_but_unimplemented_contention_is_not_faked() {
    let (mut t, mut c) = base();
    for p in ["a", "b"] {
        c.ports.insert(p.into(), fixed(Rate::M100, Duplex::Half));
    }
    let mut s = State::new(t.clone(), c.clone()).unwrap();
    let before = s.clone();
    assert_eq!(
        s.perform(&Action::Probe {
            link: "uplink".into(),
            a_frames: 4,
            b_frames: 4
        })
        .unwrap_err()
        .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(s, before);
    t.links.get_mut("uplink").unwrap().pattern = ProbePattern::Serialized;
    let mut s = State::new(t, c).unwrap();
    assert_eq!(probe(&mut s, 4, 4).after["a"].received, 4);
}
#[test]
fn limits_invalid_ids_duplicate_wires_and_counter_overflow_preserve_full_state() {
    let mut s = state();
    let original = s.clone();
    for a in [
        Action::Probe {
            link: "uplink".into(),
            a_frames: u32::MAX,
            b_frames: 2,
        },
        Action::Probe {
            link: "uplink".into(),
            a_frames: 0,
            b_frames: 0,
        },
        Action::SelectCable {
            link: "uplink".into(),
            cable: "operations".into(),
        },
        Action::ClearCounters {
            port: "../../host".into(),
        },
    ] {
        assert!(s.perform(&a).is_err());
        assert_eq!(s, original);
    }
    let mut value = serde_json::to_value(&s).unwrap();
    value["sequences"]["a"] = MAX_COUNTER.into();
    let mut capped = State::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    let before = capped.clone();
    assert_eq!(
        capped
            .perform(&Action::Probe {
                link: "uplink".into(),
                a_frames: 1,
                b_frames: 1
            })
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(capped, before);
}
#[test]
fn parser_caps_duplicate_keys_unknown_actions_and_early_container_refusal() {
    assert_eq!(
        Topology::from_json(&vec![b' '; MAX_TOPOLOGY + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        Action::from_json(&vec![b' '; MAX_ACTION + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(Action::from_json(br#"{"kind":"HostShell","command":"whoami"}"#).is_err());
    assert!(Action::from_json(br#"{"kind":"ClearCounters","port":"a","port":"b"}"#).is_err());
    let (t, _) = base();
    let mut raw = serde_json::to_string(&t).unwrap();
    raw = raw.replacen(
        "\"ports\":{",
        &format!(
            "\"ports\":{{\"a\":{},",
            serde_json::to_string(&t.ports["a"]).unwrap()
        ),
        1,
    );
    assert!(
        Topology::from_json(raw.as_bytes())
            .unwrap_err()
            .message
            .contains("before value")
    );
    let port = serde_json::to_string(&t.ports["a"]).unwrap();
    let mut keys = (0..MAX_PORTS)
        .map(|n| format!("\"p{n}\":{port}"))
        .collect::<Vec<_>>();
    keys.push("\"overflow\":null".into());
    let raw = format!(
        "{{\"version\":1,\"ports\":{{{}}},\"links\":{{}},\"cables\":{{}}}}",
        keys.join(",")
    );
    let e = Topology::from_json(raw.as_bytes()).unwrap_err();
    assert!(e.message.contains("before value"), "{e}");
}
#[test]
fn reset_resume_and_probe_split_determinism_keep_configuration_and_both_directions() {
    let mut a = state();
    select(&mut a, "noisy");
    let mut b = a.clone();
    probe(&mut a, 8, 12);
    probe(&mut b, 3, 5);
    probe(&mut b, 5, 7);
    assert_eq!(a.counters(), b.counters());
    let bytes = serde_json::to_vec(&a).unwrap();
    assert_eq!(State::from_json(&bytes).unwrap(), a);
    let (t, c) = base();
    assert_eq!(State::new(t, c).unwrap(), state());
    let original = state();
    let mut bad = serde_json::to_value(original.clone()).unwrap();
    bad["topology"]["links"]["uplink"]["b"] = "operations-a".into();
    assert!(State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(original, state());
}

#[test]
fn route_span_is_immutable_and_short_spares_cannot_connect_a_long_required_route() {
    for (span, cable, profile, qualification) in [
        (100, "copper", Profile::CopperRj45, Qualification::Qualified),
        (
            101,
            "copper",
            Profile::CopperRj45,
            Qualification::TooShortForRoute,
        ),
        (550, "om2", Profile::SxOm2, Qualification::Qualified),
        (551, "om2", Profile::SxOm2, Qualification::TooShortForRoute),
        (10_000, "os2", Profile::LxLhOs2, Qualification::Qualified),
        (
            10_001,
            "os2",
            Profile::LxLhOs2,
            Qualification::TooShortForRoute,
        ),
    ] {
        let (mut t, mut c) = base();
        t.links.get_mut("uplink").unwrap().required_span_m = span;
        c.cables.insert("uplink".into(), cable.into());
        if profile != Profile::CopperRj45 {
            for port in ["a", "b"] {
                c.ports.insert(port.into(), optical(profile));
            }
        }
        let mut s = State::new(t, c).unwrap();
        let original_topology = s.topology().clone();
        assert_eq!(s.report("uplink").unwrap().qualification, qualification);
        if qualification != Qualification::Qualified {
            let before = s.clone();
            assert!(
                s.perform(&Action::Probe {
                    link: "uplink".into(),
                    a_frames: 8,
                    b_frames: 8
                })
                .is_err()
            );
            assert_eq!(s, before);
        }
        assert_eq!(s.topology(), &original_topology);
    }
    let (mut t, c) = base();
    t.links.get_mut("uplink").unwrap().required_span_m = 700;
    let mut s = State::new(t, c).unwrap();
    assert_eq!(
        s.report("uplink").unwrap().qualification,
        Qualification::TooShortForRoute
    );
    for port in ["a", "b"] {
        configure(&mut s, port, optical(Profile::LxLhOs2));
    }
    select(&mut s, "os2");
    assert_eq!(probe(&mut s, 8, 8).link.state, LinkState::Up);
    // Direct saved-state tampering is still checked by the later authored
    // Definition source pin; no Action changes this topology/span.
    assert_eq!(s.topology().links["uplink"].required_span_m, 700);
    let mut invalid = s.topology().clone();
    invalid.links.get_mut("uplink").unwrap().required_span_m = 0;
    assert!(invalid.validate().is_err());
}
#[test]
fn raw_reported_values_are_preserved_without_derived_counter_identities_or_mastery() {
    let raw=br#"{"origin":"ReportedMeasurement","note":"Provided prior platform reading; counter meanings are source specific","counters":{"input_packets":900,"output_packets":800,"crc_errors":37,"collisions":0,"late_collisions":7}}"#;
    let reading = SuppliedReading::from_json(raw).unwrap();
    assert_eq!(
        serde_json::to_value(&reading).unwrap(),
        serde_json::from_slice::<serde_json::Value>(raw).unwrap()
    );
    assert_eq!(reading.counters.late_collisions, 7);
    assert_eq!(reading.counters.collisions, 0);
    let mut state = state();
    let preserved = reading.clone();
    probe(&mut state, 8, 8);
    state
        .perform(&Action::ClearCounters { port: "a".into() })
        .unwrap();
    assert_eq!(reading, preserved);
    assert_eq!(
        state.counters()["a"].crc_errors,
        0,
        "A supplied raw reading was never installed as derived live state"
    );
    let mut excessive = serde_json::to_value(&reading).unwrap();
    excessive["counters"]["crc_errors"] = (MAX_COUNTER + 1).into();
    assert!(SuppliedReading::from_json(&serde_json::to_vec(&excessive).unwrap()).is_err());
    assert_eq!(
        SuppliedReading::from_json(&vec![b' '; MAX_ACTION + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut unknown = serde_json::to_value(&reading).unwrap();
    unknown["success"] = true.into();
    assert!(SuppliedReading::from_json(&serde_json::to_vec(&unknown).unwrap()).is_err());
}
