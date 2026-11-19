use dlms_learning_lab::syslog::{self as log, *};
use std::collections::BTreeMap;
use std::net::Ipv4Addr;

fn ip(n: u8) -> Ipv4Addr {
    Ipv4Addr::new(192, 0, 2, n)
}
fn event(s: &str) -> EventId {
    EventId::new(s).unwrap()
}
fn facility(n: u8) -> Facility {
    Facility::new(n).unwrap()
}
fn fixture() -> State {
    let endpoint = |name: &str| Endpoint {
        name: name.into(),
        history_capacity: 2,
    };
    let environment = Environment {
        devices: [
            (ip(1), endpoint("Access switch")),
            (ip(2), endpoint("Distribution switch")),
        ]
        .into(),
        collectors: [
            (ip(100), endpoint("Operations")),
            (ip(101), endpoint("Archive")),
        ]
        .into(),
        events: [
            (
                event("port_down"),
                Event {
                    subsystem: "LINK".into(),
                    severity: Severity::Warning,
                    mnemonic: "CHANGED".into(),
                    text: "Supplied port changed state".into(),
                },
            ),
            (
                event("detail"),
                Event {
                    subsystem: "SYS".into(),
                    severity: Severity::Debug,
                    mnemonic: "DETAIL".into(),
                    text: "Supplied diagnostic detail".into(),
                },
            ),
        ]
        .into(),
        deliveries: vec![
            Delivery {
                device: ip(1),
                collector: ip(100),
                available: true,
            },
            Delivery {
                device: ip(1),
                collector: ip(101),
                available: true,
            },
            Delivery {
                device: ip(2),
                collector: ip(100),
                available: false,
            },
        ],
    };
    let policy = || DevicePolicy {
        process_enabled: true,
        buffer_threshold: Some(Severity::Warning),
        remote_threshold: Some(Severity::Warning),
        export_facility: facility(23),
        hosts: [ip(100), ip(101)].into(),
    };
    State::new(
        environment,
        [(ip(1), policy()), (ip(2), policy())].into(),
        [
            (
                ip(100),
                CollectorPolicy {
                    selectors: [(facility(23), Severity::Debug)].into(),
                },
            ),
            (
                ip(101),
                CollectorPolicy {
                    selectors: [(facility(23), Severity::Debug)].into(),
                },
            ),
        ]
        .into(),
    )
    .unwrap()
}
fn change_source(s: &mut State, f: impl FnOnce(&mut DevicePolicy)) {
    let mut p = s.device_policy(ip(1)).unwrap().clone();
    f(&mut p);
    s.set_device_policy(ip(1), p).unwrap();
}
fn requirement() -> Requirement {
    Requirement {
        device: ip(1),
        event: event("port_down"),
        local: Some(BufferDisposition::Accepted),
        remote: [(
            ip(100),
            RemoteDisposition::Accepted {
                facility: facility(23),
            },
        )]
        .into(),
    }
}
fn all_pass(s: &State) -> bool {
    s.checks(&[requirement()]).unwrap().iter().all(|c| c.pass)
}
fn raw(s: &State) -> serde_json::Value {
    serde_json::to_value(s).unwrap()
}
fn decode_value(v: serde_json::Value) -> log::Result<State> {
    State::decode(&serde_json::to_vec(&v).unwrap())
}

#[test]
fn all_severities_facilities_and_priority_codes_are_mathematically_invertible() {
    for f in 0..24 {
        for n in 0..8 {
            let severity = Severity::from_number(n).unwrap();
            let wire = facility(f).priority(severity);
            assert_eq!(u16::from(wire), u16::from(f) * 8 + u16::from(n));
            assert_eq!(
                split_priority(u16::from(wire)).unwrap(),
                (facility(f), severity)
            );
        }
    }
    assert_eq!(facility(16).name(), "local0");
    assert_eq!(facility(23).name(), "local7");
    assert!(Facility::new(24).is_err());
    assert!(Severity::from_number(8).is_err());
    assert!(split_priority(192).is_err());
}
#[test]
fn thresholds_include_every_more_urgent_level_and_not_less_urgent_levels() {
    for limit in 0..8 {
        for message in 0..8 {
            assert_eq!(
                Severity::from_number(limit)
                    .unwrap()
                    .includes(Severity::from_number(message).unwrap()),
                message <= limit
            );
        }
    }
    let mut s = fixture();
    assert_eq!(
        s.current_probe(ip(1), &event("detail")).unwrap().local,
        BufferDisposition::SeverityFiltered
    );
    change_source(&mut s, |p| p.buffer_threshold = Some(Severity::Debug));
    let p = s.current_probe(ip(1), &event("detail")).unwrap();
    assert_eq!(p.local, BufferDisposition::Accepted);
    assert!(
        p.remote
            .values()
            .all(|r| *r == RemoteDisposition::SeverityFiltered)
    );
    s.emit(ip(1), &event("detail")).unwrap();
    assert_eq!(s.local_history(ip(1)).unwrap().len(), 1);
    assert!(s.collector_history(ip(100)).unwrap().is_empty());
}
#[test]
fn diagnostic_reasons_distinguish_configuration_process_threshold_host_delivery_and_selector() {
    let mut s = fixture();
    let e = event("port_down");
    assert_eq!(
        s.current_remote(ip(1), &e, ip(102)).unwrap(),
        RemoteDisposition::NotConfigured
    );
    change_source(&mut s, |p| {
        p.hosts.insert(ip(102));
    });
    assert_eq!(
        s.current_remote(ip(1), &e, ip(102)).unwrap(),
        RemoteDisposition::UnknownCollector
    );
    change_source(&mut s, |p| p.process_enabled = false);
    assert_eq!(
        s.current_probe(ip(1), &e).unwrap().local,
        BufferDisposition::ProcessDisabled
    );
    assert_eq!(
        s.current_remote(ip(1), &e, ip(100)).unwrap(),
        RemoteDisposition::ProcessDisabled
    );
    change_source(&mut s, |p| {
        p.process_enabled = true;
        p.remote_threshold = None;
    });
    assert_eq!(
        s.current_remote(ip(1), &e, ip(100)).unwrap(),
        RemoteDisposition::RemoteDisabled
    );
    change_source(&mut s, |p| p.remote_threshold = Some(Severity::Error));
    assert_eq!(
        s.current_remote(ip(1), &e, ip(100)).unwrap(),
        RemoteDisposition::SeverityFiltered
    );
    change_source(&mut s, |p| p.remote_threshold = Some(Severity::Warning));
    assert_eq!(
        s.current_remote(ip(2), &e, ip(100)).unwrap(),
        RemoteDisposition::DeliveryUnavailable
    );
    assert_eq!(
        s.current_remote(ip(2), &e, ip(101)).unwrap(),
        RemoteDisposition::DeliveryUnavailable
    );
    s.set_collector_policy(
        ip(100),
        CollectorPolicy {
            selectors: BTreeMap::new(),
        },
    )
    .unwrap();
    assert_eq!(
        s.current_remote(ip(1), &e, ip(100)).unwrap(),
        RemoteDisposition::CollectorFiltered
    );
    s.set_collector_policy(
        ip(100),
        CollectorPolicy {
            selectors: [(facility(23), Severity::Error)].into(),
        },
    )
    .unwrap();
    assert_eq!(
        s.current_remote(ip(1), &e, ip(100)).unwrap(),
        RemoteDisposition::CollectorFiltered
    );
}
#[test]
fn wire_facility_and_message_subsystem_are_independent_and_history_captures_wire_facility() {
    let mut s = fixture();
    let e = event("port_down");
    s.emit(ip(1), &e).unwrap();
    assert!(s.show_device(ip(1)).unwrap().contains("%LINK-4-CHANGED"));
    assert!(
        s.show_collector(ip(100))
            .unwrap()
            .contains("PRI=188 local7 | %LINK-4-CHANGED")
    );
    change_source(&mut s, |p| p.export_facility = facility(16));
    assert_eq!(
        s.current_remote(ip(1), &e, ip(100)).unwrap(),
        RemoteDisposition::CollectorFiltered
    );
    assert_eq!(
        s.collector_history(ip(100)).unwrap()[0].facility,
        facility(23)
    );
    s.set_collector_policy(
        ip(100),
        CollectorPolicy {
            selectors: [(facility(16), Severity::Warning)].into(),
        },
    )
    .unwrap();
    s.emit(ip(1), &e).unwrap();
    assert_eq!(
        s.collector_history(ip(100)).unwrap()[1].facility,
        facility(16)
    );
    assert!(
        s.show_collector(ip(100))
            .unwrap()
            .contains("PRI=132 local0 | %LINK-4-CHANGED")
    );
}
#[test]
fn local_and_multiple_remote_destinations_are_independent() {
    let mut s = fixture();
    change_source(&mut s, |p| p.buffer_threshold = None);
    let p = s.emit(ip(1), &event("port_down")).unwrap();
    assert_eq!(p.local, BufferDisposition::BufferDisabled);
    assert!(s.local_history(ip(1)).unwrap().is_empty());
    assert_eq!(s.collector_history(ip(100)).unwrap().len(), 1);
    assert_eq!(s.collector_history(ip(101)).unwrap().len(), 1);
    change_source(&mut s, |p| {
        p.hosts.remove(&ip(100));
    });
    s.emit(ip(1), &event("port_down")).unwrap();
    assert_eq!(s.collector_history(ip(100)).unwrap().len(), 1);
    assert_eq!(s.collector_history(ip(101)).unwrap().len(), 2);
    change_source(&mut s, |p| p.process_enabled = false);
    let before = s.clone();
    s.emit(ip(1), &event("port_down")).unwrap();
    assert_eq!(
        s.collector_history(ip(101)).unwrap(),
        before.collector_history(ip(101)).unwrap()
    );
}
#[test]
fn bounded_circular_histories_drop_oldest_and_keep_emission_identity_across_destinations() {
    let mut s = fixture();
    for _ in 0..4 {
        s.emit(ip(1), &event("port_down")).unwrap();
    }
    let local = s.local_history(ip(1)).unwrap();
    let remote = s.collector_history(ip(100)).unwrap();
    assert_eq!(local.iter().map(|r| r.sequence).collect::<Vec<_>>(), [3, 4]);
    assert_eq!(
        remote.iter().map(|r| r.sequence).collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(remote, s.collector_history(ip(101)).unwrap());
    s.validate().unwrap();
}
#[test]
fn checks_are_fresh_nonmutating_and_damaged_policy_invalidates_despite_old_receipts() {
    let mut s = fixture();
    let before = s.clone();
    assert!(all_pass(&s));
    assert_eq!(s, before);
    s.emit(ip(1), &event("port_down")).unwrap();
    let delivered = s.collector_history(ip(100)).unwrap().to_vec();
    change_source(&mut s, |p| p.remote_threshold = Some(Severity::Error));
    let results = s.checks(&[requirement()]).unwrap();
    assert_eq!(
        results.iter().map(|r| r.pass).collect::<Vec<_>>(),
        [true, false]
    );
    assert!(results[1].explanation.contains("less urgent"));
    assert!(!all_pass(&s));
    assert_eq!(s.collector_history(ip(100)).unwrap(), delivered);
    change_source(&mut s, |p| p.remote_threshold = Some(Severity::Warning));
    assert!(all_pass(&s));
    s.clear_buffer(ip(1)).unwrap();
    s.clear_collector_history(ip(100)).unwrap();
    assert!(all_pass(&s));
}
#[test]
fn behavior_accepts_alternate_thresholds_and_unused_selectors_without_exact_policy_matching() {
    let mut s = fixture();
    assert!(all_pass(&s));
    change_source(&mut s, |p| {
        p.buffer_threshold = Some(Severity::Informational);
        p.remote_threshold = Some(Severity::Debug);
    });
    s.set_collector_policy(
        ip(100),
        CollectorPolicy {
            selectors: [
                (facility(23), Severity::Notice),
                (facility(17), Severity::Debug),
            ]
            .into(),
        },
    )
    .unwrap();
    assert!(all_pass(&s));
    let debug_forbidden = Requirement {
        device: ip(1),
        event: event("detail"),
        local: Some(BufferDisposition::SeverityFiltered),
        remote: [(ip(100), RemoteDisposition::CollectorFiltered)].into(),
    };
    assert!(s.checks(&[debug_forbidden]).unwrap().iter().all(|c| c.pass));
}
#[test]
fn policy_failures_are_atomic_and_invalid_foreign_targets_are_honest() {
    let mut s = fixture();
    let before = s.clone();
    let mut p = s.device_policy(ip(1)).unwrap().clone();
    p.hosts = (10..19).map(ip).collect();
    assert!(s.set_device_policy(ip(1), p).is_err());
    assert_eq!(s, before);
    let p = s.device_policy(ip(1)).unwrap().clone();
    assert!(s.set_device_policy(ip(3), p).is_err());
    assert_eq!(s, before);
    assert!(
        s.set_collector_policy(
            ip(102),
            CollectorPolicy {
                selectors: BTreeMap::new()
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert!(s.emit(ip(1), &event("not_supplied")).is_err());
    assert_eq!(s, before);
    assert!(s.emit(ip(3), &event("port_down")).is_err());
    assert_eq!(s, before);
    assert!(
        s.current_remote(ip(1), &event("port_down"), Ipv4Addr::BROADCAST)
            .is_err()
    );
    assert!(s.clear_buffer(ip(3)).is_err());
    assert!(s.clear_collector_history(ip(102)).is_err());
    assert_eq!(s, before);
}
#[test]
fn overflow_refuses_whole_emission_without_partial_local_or_remote_publish() {
    let mut value = raw(&fixture());
    value["next_sequence"] = u64::MAX.into();
    let mut s = decode_value(value).unwrap();
    let before = s.clone();
    let e = s.emit(ip(1), &event("port_down")).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Limit);
    assert_eq!(s, before);
}
#[test]
fn encode_decode_restores_policy_history_and_same_next_emission_exactly() {
    let mut s = fixture();
    s.emit(ip(1), &event("port_down")).unwrap();
    change_source(&mut s, |p| p.buffer_threshold = Some(Severity::Debug));
    let bytes = s.encode().unwrap();
    let mut restored = State::decode(&bytes).unwrap();
    assert_eq!(restored, s);
    s.emit(ip(1), &event("detail")).unwrap();
    restored.emit(ip(1), &event("detail")).unwrap();
    assert_eq!(restored, s);
    assert!(
        restored
            .environment_matches(s.environment().unwrap())
            .unwrap()
    );
    let mut foreign = s.environment().unwrap().clone();
    foreign.events.get_mut(&event("detail")).unwrap().text = "Other supplied fixture".into();
    assert!(!restored.environment_matches(&foreign).unwrap());
    foreign.devices.clear();
    assert!(restored.environment_matches(&foreign).is_err());
}
#[test]
fn raw_versions_unknown_fields_foreign_keys_and_duplicate_map_values_are_rejected() {
    let s = fixture();
    let mut v = raw(&s);
    v["version"] = 2.into();
    assert!(decode_value(v).is_err());
    let mut v = raw(&s);
    v["extra"] = true.into();
    assert!(decode_value(v).is_err());
    let mut v = raw(&s);
    v["next_sequence"] = 0.into();
    assert!(decode_value(v).is_err());
    let mut v = raw(&s);
    v["devices"].as_object_mut().unwrap().remove("192.0.2.1");
    assert!(decode_value(v).is_err());
    let text = String::from_utf8(s.encode().unwrap()).unwrap();
    let needle = "\"devices\":{\"192.0.2.1\":";
    let bad = text.replacen(needle, "\"devices\":{\"192.0.2.1\":null,\"192.0.2.1\":", 1);
    assert!(State::decode(bad.as_bytes()).is_err());
    assert!(State::decode(&vec![b' '; MAX_STATE_BYTES + 1]).is_err());
}
#[test]
fn duplicate_and_excess_collections_refuse_poison_bodies_before_value_decode() {
    let prefix =
        "{\"devices\":{\"192.0.2.1\":{\"name\":\"One\",\"history_capacity\":2},\"192.0.2.1\":";
    let err =
        serde_json::from_str::<Environment>(&format!("{prefix}{{\"name\":false}}}}")).unwrap_err();
    assert!(err.to_string().contains("duplicate key before value"));
    let entries = (1..=8)
        .map(|n| format!("\"192.0.2.{n}\":{{\"name\":\"One\",\"history_capacity\":2}}"))
        .collect::<Vec<_>>()
        .join(",");
    let err = serde_json::from_str::<Environment>(&format!(
        "{{\"devices\":{{{entries},\"192.0.2.9\":{{\"name\":false}}}}"
    ))
    .unwrap_err();
    assert!(err.to_string().contains("Map bound"));
    let mut v = raw(&fixture());
    v["local"]["192.0.2.1"] = serde_json::Value::Array(
        (1..=33)
            .map(|n| serde_json::json!({"sequence":n,"device":"192.0.2.1","event":"port_down"}))
            .collect(),
    );
    let err = serde_json::from_value::<State>(v.clone()).unwrap_err();
    assert!(err.to_string().contains("Sequence excess"));
    assert!(decode_value(v).is_err());
}
#[test]
fn restored_history_requires_order_known_event_same_emission_identity_and_fixed_delivery() {
    let mut s = fixture();
    s.emit(ip(1), &event("port_down")).unwrap();
    s.emit(ip(1), &event("port_down")).unwrap();
    for field in ["event", "device", "sequence"] {
        let mut v = raw(&s);
        v["remote"]["192.0.2.100"][0][field] = match field {
            "event" => "not_supplied".into(),
            "device" => "192.0.2.2".into(),
            _ => 0.into(),
        };
        assert!(decode_value(v).is_err());
    }
    let mut v = raw(&s);
    v["remote"]["192.0.2.101"][0]["facility"] = 16.into();
    assert!(decode_value(v).is_err());
    let mut v = raw(&s);
    v["remote"]["192.0.2.101"][0]["event"] = "detail".into();
    assert!(decode_value(v).is_err());
    let mut v = raw(&s);
    v["local"]["192.0.2.1"][1]["sequence"] = 1.into();
    assert!(decode_value(v).is_err());
}
#[test]
fn source_admission_rejects_ambiguous_identity_invalid_text_and_unbounded_facts() {
    let s = fixture();
    let original = s.environment().unwrap();
    for n in 0..6 {
        let mut e = original.clone();
        match n {
            0 => {
                e.collectors.insert(
                    ip(1),
                    Endpoint {
                        name: "Collision".into(),
                        history_capacity: 2,
                    },
                );
            }
            1 => e.events.get_mut(&event("detail")).unwrap().subsystem = "local7".into(),
            2 => e.events.get_mut(&event("detail")).unwrap().text = "injected\nnew record".into(),
            3 => e.deliveries.push(e.deliveries[0].clone()),
            4 => e.devices.get_mut(&ip(1)).unwrap().history_capacity = 33,
            _ => e.deliveries[0].device = ip(3),
        }
        assert!(e.validate().is_err());
    }
    assert!(EventId::new("../host").is_err());
    assert!(EventId::new(&"a".repeat(33)).is_err());
}
#[test]
fn requirement_admission_rejects_empty_duplicates_and_foreign_cases() {
    let s = fixture();
    assert!(s.checks(&[]).is_err());
    assert!(s.checks(&[requirement(), requirement()]).is_err());
    let mut r = requirement();
    r.local = None;
    r.remote.clear();
    assert!(s.checks(&[r]).is_err());
    let mut r = requirement();
    r.remote.insert(ip(102), RemoteDisposition::NotConfigured);
    assert!(s.checks(&[r]).is_err());
    let mut r = requirement();
    r.device = ip(3);
    assert!(s.checks(&[r]).is_err());
    let mut r = requirement();
    r.event = event("missing");
    assert!(s.checks(&[r]).is_err());
    let mut r = requirement();
    r.local = None;
    r.remote = [(
        ip(101),
        RemoteDisposition::Accepted {
            facility: facility(23),
        },
    )]
    .into();
    assert!(s.checks(&[r]).unwrap()[0].pass);
}
#[test]
fn literal_shell_looking_event_is_data_and_reports_remain_bounded_and_nontechnical() {
    let old = fixture();
    let mut env = old.environment().unwrap().clone();
    env.events.get_mut(&event("port_down")).unwrap().text =
        "$(touch /private) ; curl example.invalid".into();
    let mut s = State::new(
        env,
        [
            (ip(1), old.device_policy(ip(1)).unwrap().clone()),
            (ip(2), old.device_policy(ip(2)).unwrap().clone()),
        ]
        .into(),
        [
            (ip(100), old.collector_policy(ip(100)).unwrap().clone()),
            (ip(101), old.collector_policy(ip(101)).unwrap().clone()),
        ]
        .into(),
    )
    .unwrap();
    s.emit(ip(1), &event("port_down")).unwrap();
    assert!(s.show_device(ip(1)).unwrap().contains("$(touch /private)"));
    assert!(s.show_device(ip(1)).unwrap().len() < MAX_OUTPUT);
    assert!(s.show_collector(ip(100)).unwrap().len() < MAX_OUTPUT);
    change_source(&mut s, |p| p.process_enabled = false);
    let c = s.checks(&[requirement()]).unwrap();
    assert!(
        c.iter()
            .all(|r| !r.pass && !r.explanation.contains("ProcessDisabled"))
    );
}
#[test]
fn worst_escaped_valid_state_full_histories_and_large_sequences_fit_raw_cap() {
    let name = "\\\"".repeat(32);
    let text = "\\\"".repeat(128);
    let endpoint = || Endpoint {
        name: name.clone(),
        history_capacity: 32,
    };
    let devices: BTreeMap<_, _> = (1..=8).map(|n| (ip(n), endpoint())).collect();
    let collectors: BTreeMap<_, _> = (100..108).map(|n| (ip(n), endpoint())).collect();
    let events: BTreeMap<_, _> = (0..32)
        .map(|n| {
            (
                event(&format!("event_{n:026}")),
                Event {
                    subsystem: "X".repeat(16),
                    severity: Severity::Debug,
                    mnemonic: "X".repeat(24),
                    text: text.clone(),
                },
            )
        })
        .collect();
    let deliveries = devices
        .keys()
        .flat_map(|d| {
            collectors.keys().map(move |c| Delivery {
                device: *d,
                collector: *c,
                available: true,
            })
        })
        .collect();
    let policies = devices
        .keys()
        .map(|d| {
            (
                *d,
                DevicePolicy {
                    process_enabled: true,
                    buffer_threshold: Some(Severity::Debug),
                    remote_threshold: Some(Severity::Debug),
                    export_facility: facility(23),
                    hosts: collectors.keys().copied().collect(),
                },
            )
        })
        .collect();
    let collector_policies = collectors
        .keys()
        .map(|c| {
            (
                *c,
                CollectorPolicy {
                    selectors: (0..24).map(|n| (facility(n), Severity::Debug)).collect(),
                },
            )
        })
        .collect();
    let mut s = State::new(
        Environment {
            devices,
            collectors,
            events,
            deliveries,
        },
        policies,
        collector_policies,
    )
    .unwrap();
    for d in 1..=8 {
        for n in 0..32 {
            s.emit(ip(d), &event(&format!("event_{n:026}"))).unwrap();
        }
    }
    let mut value = raw(&s);
    value["next_sequence"] = u64::MAX.into();
    for family in ["local", "remote"] {
        for rows in value[family].as_object_mut().unwrap().values_mut() {
            for row in rows.as_array_mut().unwrap() {
                let old = row["sequence"].as_u64().unwrap();
                row["sequence"] = (u64::MAX - 1024 + old).into();
            }
        }
    }
    let s = decode_value(value).unwrap();
    let bytes = s.encode().unwrap();
    assert!(
        bytes.len() <= 180_480,
        "conservative proof envelope, actual {}",
        bytes.len()
    );
    assert!(bytes.len() < MAX_STATE_BYTES);
    assert_eq!(State::decode(&bytes).unwrap(), s);
    assert_eq!(s.local_history(ip(1)).unwrap().len(), 32);
    assert_eq!(s.collector_history(ip(100)).unwrap().len(), 32);
    for d in 1..=8 {
        assert!(s.show_device(ip(d)).unwrap().len() < MAX_OUTPUT);
    }
    for c in 100..108 {
        assert!(s.show_collector(ip(c)).unwrap().len() < MAX_OUTPUT);
    }
}

#[test]
fn public_state_boundaries_refuse_invalid_direct_serde_state_without_mutation() {
    let valid = fixture();
    let mut value = raw(&valid);
    value["version"] = 99.into();
    let mut invalid: State = serde_json::from_value(value).unwrap();
    let before = invalid.clone();
    assert!(invalid.validate().is_err());
    assert!(invalid.encode().is_err());
    assert!(invalid.environment().is_err());
    assert!(
        invalid
            .environment_matches(valid.environment().unwrap())
            .is_err()
    );
    assert!(invalid.device_policy(ip(1)).is_err());
    assert!(invalid.collector_policy(ip(100)).is_err());
    assert!(invalid.local_history(ip(1)).is_err());
    assert!(invalid.collector_history(ip(100)).is_err());
    assert!(invalid.current_probe(ip(1), &event("port_down")).is_err());
    assert!(
        invalid
            .current_remote(ip(1), &event("port_down"), ip(100))
            .is_err()
    );
    assert!(invalid.checks(&[requirement()]).is_err());
    assert!(invalid.show_device(ip(1)).is_err());
    assert!(invalid.show_collector(ip(100)).is_err());
    assert!(
        invalid
            .set_device_policy(ip(1), valid.device_policy(ip(1)).unwrap().clone())
            .is_err()
    );
    assert!(
        invalid
            .set_collector_policy(ip(100), valid.collector_policy(ip(100)).unwrap().clone())
            .is_err()
    );
    assert!(invalid.emit(ip(1), &event("port_down")).is_err());
    assert!(invalid.clear_buffer(ip(1)).is_err());
    assert!(invalid.clear_collector_history(ip(100)).is_err());
    assert_eq!(invalid, before);
}
