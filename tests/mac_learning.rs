//! Proposed pure-model checks; uncompiled until source-review/sole-slot grant.
use dlms_learning_lab::{
    mac_learning::{Event, Mac, Reason, State},
    switching::{Fabric, Mode},
};
use std::collections::{BTreeMap, BTreeSet};
fn mac(n: u8) -> Mac {
    Mac([2, 0, 0, 0, 0, n])
}
fn fixture() -> State {
    let fabric:Fabric=serde_json::from_str(r#"{
  "switches":{
   "s1":{"label":"Desk switch","vlans":[10,20,99],"ports":{
    "a":{"up":true,"mode":{"kind":"access","vlan":10}},
    "b":{"up":true,"mode":{"kind":"access","vlan":10}},
    "uplink":{"up":true,"mode":{"kind":"trunk","native":99,"allowed":[10,20,99]}}}},
   "s2":{"label":"Room switch","vlans":[10,20,99],"ports":{
    "c":{"up":true,"mode":{"kind":"access","vlan":10}},
    "d":{"up":true,"mode":{"kind":"access","vlan":20}},
    "uplink":{"up":true,"mode":{"kind":"trunk","native":99,"allowed":[10,20,99]}}}}},
  "hosts":{"a":{"label":"Alex","up":true},"b":{"label":"Blair","up":true},"c":{"label":"Casey","up":true},"d":{"label":"Drew","up":true}},
  "cables":[
   {"id":"wire-a","a":{"kind":"host","host":"a"},"b":{"kind":"switch","switch":"s1","port":"a"},"up":true},
   {"id":"wire-b","a":{"kind":"host","host":"b"},"b":{"kind":"switch","switch":"s1","port":"b"},"up":true},
   {"id":"wire-c","a":{"kind":"host","host":"c"},"b":{"kind":"switch","switch":"s2","port":"c"},"up":true},
   {"id":"wire-d","a":{"kind":"host","host":"d"},"b":{"kind":"switch","switch":"s2","port":"d"},"up":true},
   {"id":"between","a":{"kind":"switch","switch":"s1","port":"uplink"},"b":{"kind":"switch","switch":"s2","port":"uplink"},"up":true}]}
 "#).unwrap();
    State::new(
        fabric,
        BTreeMap::from([
            ("a".into(), mac(1)),
            ("b".into(), mac(2)),
            ("c".into(), mac(3)),
            ("d".into(), mac(4)),
        ]),
        300,
    )
    .unwrap()
}
fn names(xs: &[&str]) -> BTreeSet<String> {
    xs.iter().map(|s| s.to_string()).collect()
}
#[test]
fn source_learning_unknown_flood_and_destination_nic_are_distinct() {
    let mut s = fixture();
    let o = s.send("a", mac(3)).unwrap();
    assert_eq!(o.arrivals, names(&["b", "c"]));
    assert_eq!(o.accepted_by, names(&["c"]));
    assert_eq!(s.entries().len(), 2);
    assert!(
        s.entries()
            .iter()
            .all(|e| e.address == mac(1) && e.vlan == 10)
    );
    assert_eq!(
        s.entries().iter().find(|e| e.switch == "s2").unwrap().port,
        "uplink"
    );
    assert!(o.events.iter().any(|e| matches!(
        e,
        Event::Forward {
            tag: Some(10),
            reason: Reason::UnknownUnicast,
            ..
        }
    )));
    let o = s.send("c", mac(1)).unwrap();
    assert_eq!(o.arrivals, names(&["a"]));
    assert_eq!(o.accepted_by, names(&["a"]));
    assert_eq!(s.entries().len(), 4);
    assert!(
        o.events
            .iter()
            .filter_map(|e| if let Event::Forward { reason, .. } = e {
                Some(reason)
            } else {
                None
            })
            .all(|r| *r == Reason::KnownUnicast)
    );
}
#[test]
fn unknown_destination_does_not_invent_a_recipient_and_broadcast_matches_existing_vlan_model() {
    let mut s = fixture();
    let missing = s.send("a", mac(99)).unwrap();
    assert_eq!(missing.arrivals, names(&["b", "c"]));
    assert!(missing.accepted_by.is_empty());
    for source in ["a", "b", "c", "d"] {
        let expected = s
            .fabric()
            .broadcast(source)
            .unwrap()
            .recipients
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let o = s.send(source, Mac::BROADCAST).unwrap();
        assert_eq!(o.arrivals, expected);
        assert_eq!(o.accepted_by, expected);
    }
}
#[test]
fn learning_is_vlan_scoped_and_neither_known_destination_nor_broadcast_can_cross_groups() {
    let mut s = fixture();
    s.send("a", mac(3)).unwrap();
    s.send("c", mac(1)).unwrap();
    let o = s.send("d", mac(1)).unwrap();
    assert!(o.accepted_by.is_empty());
    assert!(o.arrivals.is_empty());
    assert!(
        s.entries()
            .iter()
            .any(|e| e.address == mac(4) && e.vlan == 20)
    );
    assert!(o.events.iter().any(|e| matches!(
        e,
        Event::Forward {
            reason: Reason::UnknownUnicast,
            vlan: 20,
            ..
        }
    )));
    assert!(
        s.entries()
            .iter()
            .filter(|e| e.address == mac(1))
            .all(|e| e.vlan == 10)
    );
}
#[test]
fn destination_lookups_do_not_refresh_age_and_source_frames_do() {
    let mut s = fixture();
    s.send("c", mac(1)).unwrap();
    s.advance(299).unwrap();
    s.send("a", mac(3)).unwrap();
    s.advance(1).unwrap();
    assert!(
        s.entries()
            .iter()
            .all(|e| e.address == mac(1) && e.learned_at == 299)
    );
    let o = s.send("a", mac(3)).unwrap();
    assert!(o.events.iter().any(|e| matches!(
        e,
        Event::Forward {
            reason: Reason::UnknownUnicast,
            ..
        }
    )));
    s.advance(300).unwrap();
    assert!(s.entries().is_empty());
}
#[test]
fn clearing_is_selective_and_zero_aging_is_an_explicit_simulator_option() {
    let mut s = fixture();
    s.send("a", mac(3)).unwrap();
    s.send("d", mac(1)).unwrap();
    let mut f = s.fabric().clone();
    let before = s.entries().len();
    s.configure(f.clone(), 0).unwrap();
    s.advance(86_400).unwrap();
    assert_eq!(s.entries().len(), before);
    assert_eq!(s.clear_dynamic("s1", Some(10)).unwrap(), 1);
    assert!(s.entries().iter().any(|e| e.switch == "s1" && e.vlan == 20));
    assert_eq!(s.clear_dynamic("s2", None).unwrap(), 2);
    let keep = s.clone();
    assert!(s.clear_dynamic("s1", Some(40)).is_err());
    assert_eq!(s, keep);
    f.switches
        .get_mut("s1")
        .unwrap()
        .ports
        .get_mut("a")
        .unwrap()
        .up = false;
    s.configure(f, 0).unwrap();
    assert!(
        s.entries()
            .iter()
            .all(|e| !(e.switch == "s1" && e.port == "a"))
    );
}
#[test]
fn incoming_vlan_and_allowed_lists_gate_learning_before_mutation() {
    let mut s = fixture();
    let mut f = s.fabric().clone();
    let Mode::Trunk { allowed, .. } = &mut f
        .switches
        .get_mut("s2")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode
    else {
        panic!()
    };
    allowed.remove(&10);
    s.configure(f, 300).unwrap();
    let o = s.send("a", mac(3)).unwrap();
    assert_eq!(o.arrivals, names(&["b"]));
    assert!(o.accepted_by.is_empty());
    assert!(s.entries().iter().all(|e| e.switch == "s1"));
    assert!(
        o.events.iter().any(
            |e| matches!(e,Event::Drop{explanation,..} if explanation.contains("incoming VLAN"))
        )
    );
}
#[test]
fn same_ingress_is_filtered_and_a_supplied_stale_entry_is_relearned_on_the_real_source_port() {
    let s = fixture();
    let mut value = serde_json::to_value(&s).unwrap();
    value["entries"] =
        serde_json::json!([{"switch":"s1","vlan":10,"address":mac(1),"port":"b","learned_at":0}]);
    let mut s: State = serde_json::from_value(value).unwrap();
    let o = s.send("a", mac(1)).unwrap();
    assert!(o.arrivals.is_empty());
    assert!(
        o.events
            .iter()
            .any(|e| matches!(e,Event::Learn{previous_port:Some(p),port,..}if p=="b"&&port=="a"))
    );
    assert!(
        o.events
            .iter()
            .any(|e| matches!(e,Event::Filter{switch,..}if switch=="s1"))
    );
    assert_eq!(s.entries()[0].port, "a");
}
#[test]
fn unsupported_input_invalid_configuration_and_limits_leave_exact_state() {
    let mut s = fixture();
    let before = s.clone();
    assert!(s.send("unknown", mac(1)).is_err());
    assert_eq!(s, before);
    assert!(
        s.send("a", Mac([1, 0, 0, 0, 0, 1]))
            .unwrap_err()
            .contains("Unsupported")
    );
    assert_eq!(s, before);
    assert!(s.advance(86_401).is_err());
    assert_eq!(s, before);
    let mut f = s.fabric().clone();
    f.hosts.remove("a");
    assert!(s.configure(f, 300).is_err());
    assert_eq!(s, before);
    let mut f = s.fabric().clone();
    f.switches
        .get_mut("s2")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .mode = Mode::Access { vlan: 10 };
    s.configure(f, 300).unwrap();
    let before = s.clone();
    assert!(
        s.send("a", mac(3))
            .unwrap_err()
            .contains("Unsupported tagged")
    );
    assert_eq!(s, before);
}
#[test]
fn resume_is_exact_and_unknown_future_duplicate_expired_entries_are_rejected() {
    let mut s = fixture();
    s.send("a", mac(3)).unwrap();
    s.advance(17).unwrap();
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(State::decode(&json).unwrap(), s);
    for mutate in [0, 1, 2, 3] {
        let mut v = serde_json::to_value(&s).unwrap();
        match mutate {
            0 => v["schema_version"] = 2.into(),
            1 => {
                let e = v["entries"][0].clone();
                v["entries"].as_array_mut().unwrap().push(e);
            }
            2 => v["entries"][0]["learned_at"] = 99.into(),
            _ => v["now"] = 300.into(),
        }
        assert!(serde_json::from_value::<State>(v).is_err());
    }
    assert!(
        State::decode(&" ".repeat(1_048_577))
            .unwrap_err()
            .contains("1 MiB")
    );
}
#[test]
fn entry_reader_refuses_excess_before_decoding_its_invalid_fields() {
    let s = fixture();
    let mut v = serde_json::to_value(s).unwrap();
    let entries=(0..256).map(|n|serde_json::json!({"switch":"s1","vlan":10,"address":Mac([2,0,0,0,(n/256)as u8,n as u8]),"port":"a","learned_at":0})).collect::<Vec<_>>();
    v["entries"] = entries.into();
    let good = serde_json::to_string(&v).unwrap();
    assert!(State::decode(&good).is_ok());
    v["entries"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"unknown":true}));
    let error = serde_json::from_str::<State>(&serde_json::to_string(&v).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("Dynamic MAC table limit"), "{error}");
}

#[test]
fn independent_current_state_checks_accept_relearning_alternatives_and_invalidate_after_clear() {
    use dlms_learning_lab::mac_learning::{Requirement, valid_requirements};
    let mut s = fixture();
    let requirements = vec![
        Requirement::Learned {
            switch: "s1".into(),
            vlan: 10,
            address: mac(3),
            port: "uplink".into(),
        },
        Requirement::Frame {
            label: "Alex to Casey without unknown flooding".into(),
            from: "a".into(),
            destination: mac(3),
            after_seconds: 0,
            arrivals: names(&["c"]),
            accepted_by: names(&["c"]),
            forwarding: Some(Reason::KnownUnicast),
        },
    ];
    assert!(valid_requirements(&requirements, &s));
    assert!(requirements.iter().all(|r| !r.check(&s).0));
    let before = s.clone();
    for r in &requirements {
        r.check(&s);
    }
    assert_eq!(s, before, "Checks cannot teach/mutate the table");
    s.send("c", mac(1)).unwrap();
    assert!(requirements.iter().all(|r| r.check(&s).0));
    s.clear_dynamic("s1", Some(10)).unwrap();
    assert!(requirements.iter().all(|r| !r.check(&s).0));
    s.send("c", Mac::BROADCAST).unwrap();
    assert!(
        requirements.iter().all(|r| r.check(&s).0),
        "A broadcast from the right source is also valid learning"
    );
    let isolation = Requirement::Frame {
        label: "Separate group".into(),
        from: "d".into(),
        destination: mac(1),
        after_seconds: 0,
        arrivals: BTreeSet::new(),
        accepted_by: BTreeSet::new(),
        forwarding: None,
    };
    assert!(
        !valid_requirements(&[isolation], &s),
        "No all-negative activity can pass by disconnecting everything"
    );
}
#[test]
fn future_idle_checks_do_not_mutate_clock_and_reject_expired_destination_even_after_prior_success()
{
    use dlms_learning_lab::mac_learning::Requirement;
    let mut s = fixture();
    s.send("c", mac(1)).unwrap();
    let probe = Requirement::Frame {
        label: "Quiet destination".into(),
        from: "a".into(),
        destination: mac(3),
        after_seconds: 301,
        arrivals: names(&["c"]),
        accepted_by: names(&["c"]),
        forwarding: Some(Reason::KnownUnicast),
    };
    assert!(!probe.check(&s).0);
    assert_eq!(s.now(), 0);
    s.configure(s.fabric().clone(), 900).unwrap();
    assert!(probe.check(&s).0);
    assert_eq!(s.now(), 0);
    s.configure(s.fabric().clone(), 300).unwrap();
    assert!(!probe.check(&s).0);
    let before = s.clone();
    let mut v = serde_json::to_value(&s).unwrap();
    v["now"] = u64::MAX.into();
    v["entries"] = serde_json::json!([]);
    let mut end: State = serde_json::from_value(v).unwrap();
    let frozen = end.clone();
    assert!(end.advance(1).is_err());
    assert_eq!(end, frozen);
    assert_eq!(s, before);
}

#[test]
fn negative_delivery_cannot_pass_by_disabling_its_attachment_while_another_pair_works() {
    use dlms_learning_lab::mac_learning::{Requirement, valid_requirements};
    let mut s = fixture();
    s.send("c", mac(1)).unwrap();
    let positive = Requirement::Frame {
        label: "Healthy group".into(),
        from: "a".into(),
        destination: mac(3),
        after_seconds: 0,
        arrivals: names(&["c"]),
        accepted_by: names(&["c"]),
        forwarding: Some(Reason::KnownUnicast),
    };
    let negative = Requirement::Frame {
        label: "Separate groups".into(),
        from: "d".into(),
        destination: mac(1),
        after_seconds: 0,
        arrivals: BTreeSet::new(),
        accepted_by: BTreeSet::new(),
        forwarding: None,
    };
    assert!(valid_requirements(
        &[positive.clone(), negative.clone()],
        &s
    ));
    assert!(positive.check(&s).0 && negative.check(&s).0);
    let mut f = s.fabric().clone();
    f.switches
        .get_mut("s2")
        .unwrap()
        .ports
        .get_mut("d")
        .unwrap()
        .up = false;
    s.configure(f, 300).unwrap();
    assert!(positive.check(&s).0);
    let (pass, why) = negative.check(&s);
    assert!(!pass && why.contains("port is down"), "{why}");
}
#[test]
fn tagged_copies_arrive_even_when_the_untagged_endpoint_cannot_accept_them() {
    let mut s = fixture();
    let mut f = s.fabric().clone();
    f.switches
        .get_mut("s1")
        .unwrap()
        .ports
        .get_mut("b")
        .unwrap()
        .mode = Mode::Trunk {
        native: 20,
        allowed: BTreeSet::from([10, 20]),
    };
    s.configure(f, 300).unwrap();
    let result = s.send("a", Mac::BROADCAST).unwrap();
    assert_eq!(result.arrivals, names(&["b", "c"]));
    assert_eq!(result.accepted_by, names(&["c"]));
    assert!(result.events.iter().any(|e|matches!(e,Event::Drop{location,explanation}if location=="b"&&explanation.contains("untagged"))));
}
#[test]
fn malformed_unicode_field_diagnostics_have_a_retained_utf8_byte_bound() {
    use dlms_learning_lab::mac_learning::MAX_ERROR_BYTES;
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["é".repeat(100_000)] = true.into();
    let text = serde_json::to_string(&value).unwrap();
    let error = State::decode(&text).unwrap_err();
    assert!(
        error.len() <= MAX_ERROR_BYTES && error.capacity() <= MAX_ERROR_BYTES,
        "len{} capacity{}",
        error.len(),
        error.capacity()
    );
    assert!(error.ends_with("..."));
    assert!(error.contains("unknown field"));
}
#[test]
fn maximum_supported_shape_roundtrips_pretty_json_beyond_the_compact_payload_limit() {
    use dlms_learning_lab::{
        mac_learning::{Entry, MAX_INPUT_BYTES, MAX_STATE_BYTES},
        switching::{Cable, Host, Jack, Port, Switch},
    };
    fn id(prefix: char, n: usize) -> String {
        format!("{prefix}{n:02}{}", "x".repeat(61))
    }
    let mut fabric = Fabric {
        switches: BTreeMap::new(),
        hosts: BTreeMap::new(),
        cables: vec![],
    };
    let mut addresses = BTreeMap::new();
    for n in 0..8 {
        let ports = (0..16)
            .map(|p| {
                (
                    id('p', p),
                    Port {
                        up: true,
                        // Exercise the maximum accepted field shape, not a recommended
                        // workstation configuration: native VLAN1 remains untagged.
                        mode: Mode::Trunk {
                            native: 1,
                            allowed: (1..=64).collect(),
                        },
                    },
                )
            })
            .collect();
        fabric.switches.insert(
            id('s', n),
            Switch {
                label: "\\".repeat(80),
                vlans: (1..=64).collect(),
                ports,
            },
        );
    }
    for n in 0..32 {
        fabric.hosts.insert(
            id('h', n),
            Host {
                label: "\\".repeat(80),
                up: true,
            },
        );
        addresses.insert(id('h', n), mac(n as u8));
        fabric.cables.push(Cable {
            id: id('c', n),
            a: Jack::Host { host: id('h', n) },
            b: Jack::Switch {
                switch: id('s', n / 4),
                port: id('p', n % 4),
            },
            up: true,
        });
    }
    for n in 0..7 {
        fabric.cables.push(Cable {
            id: id('c', n + 32),
            a: Jack::Switch {
                switch: id('s', n),
                port: id('p', 5),
            },
            b: Jack::Switch {
                switch: id('s', n + 1),
                port: id('p', 4),
            },
            up: true,
        });
    }
    let state = State::new(fabric, addresses, 300).unwrap();
    let mut value = serde_json::to_value(state).unwrap();
    let entries = (0..8)
        .flat_map(|sw| {
            (0..32).map(move |host| Entry {
                switch: id('s', sw),
                vlan: 1,
                address: mac(host as u8),
                port: id(
                    'p',
                    if host / 4 == sw {
                        host % 4
                    } else if host / 4 < sw {
                        4
                    } else {
                        5
                    },
                ),
                learned_at: 0,
            })
        })
        .collect::<Vec<_>>();
    value["entries"] = serde_json::to_value(entries).unwrap();
    let state: State = serde_json::from_value(value).unwrap();
    let compact = serde_json::to_string(&state).unwrap();
    let pretty = serde_json::to_string_pretty(&state).unwrap();
    assert!(compact.len() <= MAX_STATE_BYTES);
    assert!(
        pretty.len() > MAX_STATE_BYTES && pretty.len() < MAX_INPUT_BYTES,
        "compact{} pretty{}",
        compact.len(),
        pretty.len()
    );
    assert_eq!(State::decode(&pretty).unwrap(), state);
}

#[test]
fn negative_physical_guards_cover_destination_transit_and_native_input_failures() {
    use dlms_learning_lab::mac_learning::Requirement;
    for fault in 0..4 {
        let mut s = fixture();
        let positive = if fault == 1 {
            s.send("b", mac(1)).unwrap();
            Requirement::Frame {
                label: "Local healthy pair".into(),
                from: "a".into(),
                destination: mac(2),
                after_seconds: 0,
                arrivals: names(&["b"]),
                accepted_by: names(&["b"]),
                forwarding: Some(Reason::KnownUnicast),
            }
        } else {
            s.send("c", mac(2)).unwrap();
            Requirement::Frame {
                label: "Other healthy pair".into(),
                from: "b".into(),
                destination: mac(3),
                after_seconds: 0,
                arrivals: names(&["c"]),
                accepted_by: names(&["c"]),
                forwarding: Some(Reason::KnownUnicast),
            }
        };
        let negative = Requirement::Frame {
            label: "Separate groups".into(),
            from: "d".into(),
            destination: mac(1),
            after_seconds: 0,
            arrivals: BTreeSet::new(),
            accepted_by: BTreeSet::new(),
            forwarding: None,
        };
        assert!(positive.check(&s).0 && negative.check(&s).0);
        let mut f = s.fabric().clone();
        match fault {
            0 => {
                f.switches
                    .get_mut("s1")
                    .unwrap()
                    .ports
                    .get_mut("a")
                    .unwrap()
                    .up = false
            }
            1 => {
                f.switches
                    .get_mut("s2")
                    .unwrap()
                    .ports
                    .get_mut("uplink")
                    .unwrap()
                    .up = false
            }
            2 => {
                f.switches
                    .get_mut("s2")
                    .unwrap()
                    .ports
                    .get_mut("d")
                    .unwrap()
                    .mode = Mode::Trunk {
                    native: 20,
                    allowed: BTreeSet::from([10]),
                }
            }
            _ => {
                f.switches.get_mut("s2").unwrap().vlans.remove(&20);
            }
        }
        s.configure(f, 300).unwrap();
        assert!(positive.check(&s).0, "unrelated positive must stay healthy");
        let (pass, why) = negative.check(&s);
        assert!(!pass, "fault{fault}: {why}");
        assert!(
            why.contains("down")
                || why.contains("physical")
                || why.contains("not allowed")
                || why.contains("inactive"),
            "{why}"
        );
    }
}
