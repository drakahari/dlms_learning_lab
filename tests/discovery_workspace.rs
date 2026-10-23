use dlms_learning_lab::{
    discovery::{
        Check, Device, Endpoint, GlobalSettings, Link, MAX_TIME, Outcome, Port, Requirement, State,
        Topology,
    },
    discovery_teaching::{Learning, Lesson, MAX_LEARNING_BYTES, MAX_LESSON_BYTES, TryChange},
    discovery_workspace::{
        self as editor, Action, ActionResult, Draft, MAX_ACTION_BYTES, MAX_DRAFT_BYTES,
        MAX_DRAFT_TEXT,
    },
    script_teaching::{Concept, View},
};
use std::collections::BTreeMap;

fn e(device: &str, port: &str) -> Endpoint {
    Endpoint {
        device: device.into(),
        port: port.into(),
    }
}
fn device(id: &str, ports: &[&str]) -> Device {
    Device {
        chassis_id: format!("chassis-{id}"),
        system_name: Some(id.into()),
        enabled: true,
        ttl: 5,
        ports: ports
            .iter()
            .map(|p| {
                (
                    p.to_string(),
                    Port {
                        port_id: p.to_string(),
                        description: Some(format!("{id} {p}")),
                        up: true,
                        transmit: true,
                        receive: true,
                    },
                )
            })
            .collect(),
    }
}
fn fixture() -> State {
    let mut left = device("left", &["uplink", "unused"]);
    left.enabled = false;
    State::new(Topology {
        devices: BTreeMap::from([
            ("left".into(), left),
            ("right".into(), device("right", &["uplink"])),
        ]),
        links: vec![Link {
            id: "wire".into(),
            a: e("left", "uplink"),
            b: e("right", "uplink"),
            up: true,
        }],
    })
    .unwrap()
}
fn global(enabled: bool) -> Action {
    Action::GlobalConfiguration {
        device: "left".into(),
        settings: GlobalSettings {
            enabled,
            ttl: 5,
            system_name: Some("left".into()),
        },
    }
}
fn lesson() -> Lesson {
    let fixture = fixture();
    let mut configured = fixture.topology().clone();
    configured.devices.get_mut("left").unwrap().enabled = true;
    Lesson {
        prerequisite_note: "Know device and local port identities.".into(),
        concepts: vec![Concept {
            title: "Local reception".into(),
            text: "An advertisement reaches only the directly attached receiving port.".into(),
        }],
        example_description:
            "Enable the isolated practice sender and explicitly advertise in each direction.".into(),
        example_solution: vec![
            global(true),
            Action::Advertise {
                from: e("left", "uplink"),
            },
            Action::Advertise {
                from: e("right", "uplink"),
            },
        ],
        outcomes: vec![
            Requirement::CurrentNeighbor {
                local: e("right", "uplink"),
                peer: e("left", "uplink"),
            },
            Requirement::CurrentNeighbor {
                local: e("left", "uplink"),
                peer: e("right", "uplink"),
            },
            Requirement::Configuration {
                expected: configured.clone(),
            },
        ],
        try_change: TryChange {
            description: "Advance exactly one advertised lifetime and inspect both empty caches."
                .into(),
            actions: vec![Action::Advance { seconds: 5 }],
            outcomes: vec![
                Requirement::AbsentNeighbor {
                    local: e("left", "uplink"),
                },
                Requirement::AbsentNeighbor {
                    local: e("right", "uplink"),
                },
                Requirement::Configuration {
                    expected: configured,
                },
            ],
        },
        fixture,
    }
}

#[test]
fn draft_snapshot_roundtrips_unsaved_text_and_applies_configuration_without_events() {
    let mut s = fixture();
    let mut draft = Draft::new(&s);
    assert!(draft.valid() && draft.same_shape(&s) && draft.matches(&s));
    draft.selected_device = "right".into();
    draft.devices.get_mut("left").unwrap().enabled = true;
    draft.devices.get_mut("left").unwrap().ttl = "0005".into();
    draft.devices.get_mut("left").unwrap().system_name = None;
    draft
        .devices
        .get_mut("left")
        .unwrap()
        .ports
        .get_mut("unused")
        .unwrap()
        .description = None;
    assert!(draft.valid() && !draft.matches(&s));
    let before = s.clone();
    let snapshot = draft.snapshot(&s).unwrap();
    assert_eq!(snapshot.devices["left"].ttl, 5);
    assert_eq!(s, before);
    for text in [
        serde_json::to_string(&draft).unwrap(),
        serde_json::to_string_pretty(&draft).unwrap(),
    ] {
        assert_eq!(Draft::from_json(&text, &s).unwrap(), draft);
    }
    draft.apply(&mut s).unwrap();
    assert!(draft.matches(&s));
    assert_eq!(s.now(), 0);
    assert!(s.neighbors("left").unwrap().is_empty());
    assert!(s.neighbors("right").unwrap().is_empty());
    assert_eq!(s.topology().links, before.topology().links);
}

#[test]
fn invalid_bounded_ttl_and_labels_remain_editable_but_leave_every_simulation_field_untouched() {
    let mut s = fixture();
    global(true).apply(&mut s).unwrap();
    Action::Advertise {
        from: e("left", "uplink"),
    }
    .apply(&mut s)
    .unwrap();
    s.advance(2).unwrap();
    let before = s.clone();
    for ttl in ["", "0", "+5", "-1", "65536", "5.0", " 5", "1e2", "bad"] {
        let mut d = Draft::new(&s);
        d.devices.get_mut("left").unwrap().ttl = ttl.into();
        assert!(d.valid());
        let restored = Draft::from_json(&serde_json::to_string(&d).unwrap(), &s).unwrap();
        assert_eq!(restored, d);
        assert!(d.apply(&mut s).is_err());
        assert!(!d.matches(&s));
        assert_eq!(s, before);
    }
    for text in ["", " ", "has\nnewline", "nonasciié", &"x".repeat(81)] {
        for name in [false, true] {
            let mut d = Draft::new(&s);
            if name {
                d.devices.get_mut("left").unwrap().system_name = Some(text.into());
            } else {
                d.devices
                    .get_mut("left")
                    .unwrap()
                    .ports
                    .get_mut("uplink")
                    .unwrap()
                    .description = Some(text.into());
            }
            assert!(d.valid());
            assert!(d.apply(&mut s).is_err());
            assert_eq!(s, before);
        }
    }
    let mut d = Draft::new(&s);
    d.devices.get_mut("left").unwrap().ttl = "x".repeat(41);
    assert!(!d.valid());
    let mut d = Draft::new(&s);
    d.devices.get_mut("left").unwrap().system_name = Some("x".repeat(MAX_DRAFT_TEXT + 1));
    assert!(!d.valid());
}

#[test]
fn draft_shape_and_identity_pins_reject_deletion_renaming_rewiring_and_cross_fixture_use() {
    let mut s = fixture();
    let draft = Draft::new(&s);
    for kind in 0..4 {
        let mut t = s.topology().clone();
        match kind {
            0 => t.devices.get_mut("left").unwrap().chassis_id = "changed-chassis".into(),
            1 => {
                t.devices
                    .get_mut("left")
                    .unwrap()
                    .ports
                    .get_mut("uplink")
                    .unwrap()
                    .port_id = "changed-port".into()
            }
            2 => t.links[0].a = e("left", "unused"),
            _ => t.links[0].id = "changed-link".into(),
        }
        let mut other = State::new(t).unwrap();
        let before = other.clone();
        assert!(!draft.same_shape(&other));
        assert!(draft.apply(&mut other).is_err());
        assert_eq!(other, before);
        assert!(Draft::from_json(&serde_json::to_string(&draft).unwrap(), &other).is_err());
    }
    let before = s.clone();
    let mut deleted = draft.clone();
    deleted
        .devices
        .get_mut("left")
        .unwrap()
        .ports
        .remove("unused");
    assert!(!deleted.valid());
    assert!(deleted.apply(&mut s).is_err());
    assert_eq!(s, before);
    let mut selected = draft.clone();
    selected.selected_device = "missing".into();
    assert!(!selected.valid());
}

#[test]
fn typed_actions_and_bounded_json_adapter_report_actual_directional_results() {
    let mut s = fixture();
    let before = s.clone();
    let advertise = Action::Advertise {
        from: e("left", "uplink"),
    };
    assert!(
        matches!(advertise.apply(&mut s).unwrap(), ActionResult::Transmission(t) if t.outcome == Outcome::TransmitDisabled)
    );
    assert_eq!(s, before);
    let input = format!(
        "discovery {}",
        serde_json::to_string(&global(true)).unwrap()
    );
    editor::adapter(&mut s, &input).unwrap();
    assert_eq!(s.now(), 0);
    assert!(s.neighbors("right").unwrap().is_empty());
    let input = format!("discovery {}", serde_json::to_string(&advertise).unwrap());
    assert!(
        editor::adapter(&mut s, &input)
            .unwrap()
            .contains("recorded the advertisement")
    );
    Action::Advance { seconds: 4 }.apply(&mut s).unwrap();
    assert_eq!(
        s.neighbor(&e("right", "uplink"))
            .unwrap()
            .unwrap()
            .remaining_seconds,
        1
    );
    assert!(
        matches!(Action::Withdraw { from: e("left", "uplink") }.apply(&mut s).unwrap(), ActionResult::Transmission(t) if t.outcome == Outcome::Withdrawn)
    );
    assert!(s.neighbors("right").unwrap().is_empty());
    Action::PortConfiguration {
        local: e("left", "uplink"),
        up: true,
        transmit: false,
        receive: true,
        description: None,
    }
    .apply(&mut s)
    .unwrap();
    Action::Advertise {
        from: e("right", "uplink"),
    }
    .apply(&mut s)
    .unwrap();
    assert!(s.neighbor(&e("left", "uplink")).unwrap().is_some());
    assert!(
        matches!(advertise.apply(&mut s).unwrap(), ActionResult::Transmission(t) if t.outcome == Outcome::TransmitDisabled)
    );
}

#[test]
fn unsupported_and_malformed_adapter_actions_fail_atomically_without_a_command_interpreter() {
    let mut s = fixture();
    let before = s.clone();
    for input in [
        "show lldp neighbors",
        "lldp run",
        "discovery ",
        "discovery {}",
        "discovery {\"kind\":\"shell\",\"command\":\"touch /tmp/no\"}",
        "discovery {\"kind\":\"advance\",\"seconds\":1}; run",
        "discovery {\"kind\":\"advance\",\"seconds\":-1}",
        "discovery {\"kind\":\"advance\",\"seconds\":0,\"cdp\":true}",
    ] {
        assert!(editor::adapter(&mut s, input).is_err());
        assert_eq!(s, before);
    }
    assert!(editor::adapter(&mut s, &"x".repeat(MAX_ACTION_BYTES + 1)).is_err());
    let bad_actions = [
        Action::GlobalConfiguration {
            device: "left".into(),
            settings: GlobalSettings {
                enabled: true,
                ttl: 0,
                system_name: None,
            },
        },
        Action::PortConfiguration {
            local: e("left", "uplink"),
            up: true,
            transmit: true,
            receive: true,
            description: Some(" ".into()),
        },
        Action::Advertise {
            from: e("missing", "port"),
        },
        Action::Advance { seconds: u64::MAX },
    ];
    for action in bad_actions {
        assert!(action.apply(&mut s).is_err());
        assert_eq!(s, before);
    }
    Action::Advance { seconds: MAX_TIME }.apply(&mut s).unwrap();
    let before = s.clone();
    assert!(Action::Advance { seconds: 1 }.apply(&mut s).is_err());
    assert_eq!(s, before);
}

#[test]
fn filling_example_changes_only_practice_draft_and_explicit_actions_reach_current_outcomes() {
    let lesson = lesson();
    lesson.validate().unwrap();
    let assignment = fixture();
    let assignment_before = assignment.clone();
    let fixture_before = lesson.fixture.clone();
    let mut learning = Learning::new(&lesson);
    assert_eq!(learning.view, View::Introduction);
    assert!(learning.checks(&lesson).is_empty());
    assert!(learning.check(&lesson).unwrap().iter().any(|r| !r.passed()));
    learning.draft.selected_device = "right".into();
    learning.draft.devices.get_mut("left").unwrap().ttl = "bad".into();
    let simulation_before = learning.simulation.clone();
    learning.fill_example(&lesson).unwrap();
    assert_eq!(learning.simulation, simulation_before);
    assert_eq!(learning.draft.selected_device, "right");
    assert!(!learning.checked);
    assert!(!learning.draft.matches(&learning.simulation));
    learning.apply().unwrap();
    assert!(learning.simulation.neighbors("right").unwrap().is_empty());
    assert_eq!(learning.simulation.now(), 0);
    assert!(!learning.checked);
    learning
        .execute(&Action::Advertise {
            from: e("left", "uplink"),
        })
        .unwrap();
    learning
        .execute(&Action::Advertise {
            from: e("right", "uplink"),
        })
        .unwrap();
    assert_eq!(learning.draft.selected_device, "right");
    assert!(learning.check(&lesson).unwrap().iter().all(|r| r.passed()));
    assert_eq!(assignment, assignment_before);
    assert_eq!(lesson.fixture, fixture_before);
    learning.execute(&Action::Advance { seconds: 5 }).unwrap();
    assert!(!learning.checked);
    assert!(learning.check(&lesson).unwrap().iter().any(|r| !r.passed()));
}

#[test]
fn fill_does_not_refresh_existing_cache_or_reset_practice_clock() {
    let lesson = lesson();
    let mut learning = Learning::new(&lesson);
    for action in &lesson.example_solution {
        learning.execute(action).unwrap();
    }
    learning.execute(&Action::Advance { seconds: 3 }).unwrap();
    learning.check(&lesson).unwrap();
    let before = learning.simulation.clone();
    learning.fill_example(&lesson).unwrap();
    assert_eq!(learning.simulation, before);
    assert_eq!(
        learning
            .simulation
            .neighbor(&e("right", "uplink"))
            .unwrap()
            .unwrap()
            .remaining_seconds,
        2
    );
    assert!(!learning.checked);
}

#[test]
fn learning_preserves_unapplied_edits_and_current_checks_never_retain_stale_success() {
    let lesson = lesson();
    let mut learning = Learning::new(&lesson);
    for action in &lesson.example_solution {
        learning.execute(action).unwrap();
    }
    assert!(learning.check(&lesson).unwrap().iter().all(|r| r.passed()));
    learning.draft.devices.get_mut("left").unwrap().system_name = Some("unapplied name".into());
    let draft = learning.draft.clone();
    let state = learning.simulation.clone();
    assert!(
        learning
            .checks(&lesson)
            .iter()
            .any(|(passed, message)| !passed && message.contains("unapplied"))
    );
    assert!(learning.execute(&Action::Advance { seconds: 1 }).is_err());
    assert_eq!(learning.simulation, state);
    assert_eq!(learning.draft, draft);
    learning.apply().unwrap();
    assert!(!learning.checked);
    assert!(
        learning
            .check(&lesson)
            .unwrap()
            .contains(&Check::HistoricalNeighbor)
    );
    learning
        .execute(&Action::Advertise {
            from: e("left", "uplink"),
        })
        .unwrap();
    // The advertisement now matches current remote fields, while exact authored configuration still fails.
    assert!(
        learning
            .check(&lesson)
            .unwrap()
            .contains(&Check::ConfigurationMismatch)
    );
    learning.draft.devices.get_mut("left").unwrap().ttl = "wrong".into();
    let before = learning.simulation.clone();
    assert!(learning.apply().is_err());
    assert_eq!(learning.simulation, before);
    assert_eq!(learning.draft.devices["left"].ttl, "wrong");
    assert!(learning.notice.is_some());
    assert!(!learning.checked);
}

#[test]
fn teaching_roundtrips_independent_example_state_and_bounded_invalid_drafts() {
    let lesson = lesson();
    let loaded = Lesson::from_json(&serde_json::to_string_pretty(&lesson).unwrap()).unwrap();
    loaded.validate().unwrap();
    let mut learning = Learning::new(&loaded);
    learning.view = View::Example;
    learning.draft.devices.get_mut("left").unwrap().ttl = "unfinished".into();
    learning.notice = Some("Correct the draft before applying it.".into());
    let text = serde_json::to_string(&learning).unwrap();
    let resumed = Learning::from_json(&text, &loaded).unwrap();
    assert_eq!(resumed.view, View::Example);
    assert_eq!(resumed.draft, learning.draft);
    assert_eq!(resumed.simulation, learning.simulation);
    assert!(resumed.valid_for(&loaded));
    let mut other = lesson.clone();
    let mut topology = other.fixture.topology().clone();
    topology.devices.get_mut("left").unwrap().chassis_id = "different".into();
    other.fixture = State::new(topology).unwrap();
    assert!(Learning::from_json(&text, &other).is_err());
}

#[test]
fn unreachable_stale_noop_duplicate_or_oversized_teaching_is_rejected() {
    for kind in 0..7 {
        let mut l = lesson();
        match kind {
            0 => {
                l.example_solution.remove(0);
            }
            1 => l.example_solution.push(Action::GlobalConfiguration {
                device: "left".into(),
                settings: GlobalSettings {
                    enabled: true,
                    ttl: 4,
                    system_name: Some("left".into()),
                },
            }),
            2 => l.try_change.actions = vec![Action::Advance { seconds: 0 }],
            3 => l.outcomes.push(l.outcomes[0].clone()),
            4 => l.example_solution = vec![global(true); 17],
            5 => l.concepts[0].text = "x".repeat(1201),
            _ => {
                l.outcomes[0] = Requirement::CurrentNeighbor {
                    local: e("right", "uplink"),
                    peer: e("left", "unused"),
                }
            }
        }
        assert!(l.validate().is_err(), "case {kind}");
        assert!(Lesson::from_json(&serde_json::to_string(&l).unwrap()).is_err());
    }
    assert!(Lesson::from_json(&" ".repeat(MAX_LESSON_BYTES + 1)).is_err());
    assert!(Learning::from_json(&" ".repeat(MAX_LEARNING_BYTES + 1), &lesson()).is_err());
}

#[test]
fn draft_and_learning_imports_reject_duplicate_unknown_and_over_limit_shapes() {
    let s = fixture();
    let d = Draft::new(&s);
    let mut value = serde_json::to_value(&d).unwrap();
    value["host_command"] = serde_json::json!("ignored");
    assert!(Draft::from_json(&value.to_string(), &s).is_err());
    let mut value = serde_json::to_value(&d).unwrap();
    value["devices"]["left"]["system_name"] = serde_json::json!("x".repeat(MAX_DRAFT_TEXT + 1));
    assert!(Draft::from_json(&value.to_string(), &s).is_err());
    let raw = serde_json::to_value(&d).unwrap();
    let duplicated = format!(
        "{{\"version\":1,\"selected_device\":\"left\",\"identity\":{},\"devices\":{{\"left\":{},\"left\":{}}}}}",
        raw["identity"], raw["devices"]["left"], raw["devices"]["left"]
    );
    assert!(Draft::from_json(&duplicated, &s).is_err());
    assert!(Draft::from_json(&" ".repeat(MAX_DRAFT_BYTES + 1), &s).is_err());
    let mut value = serde_json::to_value(Learning::new(&lesson())).unwrap();
    value["notice"] = serde_json::json!("x".repeat(16001));
    assert!(Learning::from_json(&value.to_string(), &lesson()).is_err());
}

#[test]
fn maximum_draft_and_learning_text_escape_roundtrips_fit_separate_document_bounds() {
    fn key(prefix: &str, n: usize) -> String {
        let start = format!("{prefix}{n:03}");
        format!("{start}{}", "x".repeat(64 - start.len()))
    }
    let mut t = Topology {
        devices: BTreeMap::new(),
        links: Vec::new(),
    };
    for d in 0..12 {
        let ports = (0..16)
            .map(|p| {
                (
                    key("p", p),
                    Port {
                        port_id: format!("{}p{p:02}", "\\".repeat(61)),
                        description: Some("\\".repeat(80)),
                        up: true,
                        transmit: true,
                        receive: true,
                    },
                )
            })
            .collect();
        t.devices.insert(
            key("d", d),
            Device {
                chassis_id: format!("{}d{d:02}", "\\".repeat(61)),
                system_name: Some("\\".repeat(80)),
                enabled: true,
                ttl: u16::MAX,
                ports,
            },
        );
    }
    for d in (0..12).step_by(2) {
        for p in 0..16 {
            t.links.push(Link {
                id: key("l", d * 16 + p),
                a: e(&key("d", d), &key("p", p)),
                b: e(&key("d", d + 1), &key("p", p)),
                up: true,
            });
        }
    }
    let ends: Vec<_> = t
        .devices
        .iter()
        .flat_map(|(id, d)| d.ports.keys().map(move |p| e(id, p)))
        .collect();
    let mut state = State::new(t).unwrap();
    for end in &ends {
        state.advertise(end).unwrap();
    }
    let mut l = lesson();
    l.fixture = state;
    l.example_solution = vec![Action::Advance { seconds: 1 }];
    l.outcomes = vec![Requirement::CurrentNeighbor {
        local: ends[0].clone(),
        peer: ends[16].clone(),
    }];
    l.try_change.actions = vec![Action::Advance { seconds: 1 }];
    l.try_change.outcomes = l.outcomes.clone();
    l.validate().unwrap();
    let mut learning = Learning::new(&l);
    for device in learning.draft.devices.values_mut() {
        device.ttl = "\0".repeat(40);
        device.system_name = Some("\0".repeat(MAX_DRAFT_TEXT));
        for port in device.ports.values_mut() {
            port.description = Some("\0".repeat(MAX_DRAFT_TEXT));
        }
    }
    learning.notice = Some("\0".repeat(16_000));
    assert!(learning.valid_for(&l));
    for pretty in [false, true] {
        let draft = if pretty {
            serde_json::to_string_pretty(&learning.draft).unwrap()
        } else {
            serde_json::to_string(&learning.draft).unwrap()
        };
        assert!(draft.len() <= MAX_DRAFT_BYTES);
        assert_eq!(
            Draft::from_json(&draft, &learning.simulation).unwrap(),
            learning.draft
        );
        let text = if pretty {
            serde_json::to_string_pretty(&learning).unwrap()
        } else {
            serde_json::to_string(&learning).unwrap()
        };
        assert!(text.len() <= MAX_LEARNING_BYTES);
        let restored = Learning::from_json(&text, &l).unwrap();
        assert_eq!(restored.draft, learning.draft);
        assert_eq!(restored.simulation, learning.simulation);
    }
}

#[test]
fn native_feedback_names_local_context_and_distinguishes_missing_history_and_configuration() {
    let initial = fixture();
    let positive = Requirement::CurrentNeighbor {
        local: e("right", "uplink"),
        peer: e("left", "uplink"),
    };
    let absent = Requirement::AbsentNeighbor {
        local: e("right", "uplink"),
    };
    let mut s = initial.clone();
    let missing = editor::checks(initial.topology(), &s, std::slice::from_ref(&positive));
    assert!(
        !missing[0].0
            && missing[0].1.starts_with("right/uplink:")
            && missing[0].1.contains("no unexpired")
    );
    global(true).apply(&mut s).unwrap();
    Action::Advertise {
        from: e("left", "uplink"),
    }
    .apply(&mut s)
    .unwrap();
    assert!(editor::checks(initial.topology(), &s, std::slice::from_ref(&positive))[0].0);
    let unexpected = editor::checks(initial.topology(), &s, std::slice::from_ref(&absent));
    assert!(!unexpected[0].0 && unexpected[0].1.contains("remains cached"));
    Action::GlobalConfiguration {
        device: "left".into(),
        settings: GlobalSettings {
            enabled: true,
            ttl: 5,
            system_name: Some("new name".into()),
        },
    }
    .apply(&mut s)
    .unwrap();
    let historical = editor::checks(initial.topology(), &s, std::slice::from_ref(&positive));
    assert!(!historical[0].0 && historical[0].1.contains("historical"));
    let config = editor::checks(
        initial.topology(),
        &s,
        &[Requirement::GlobalConfiguration {
            device: "left".into(),
            expected: GlobalSettings::from_device(&initial.topology().devices["left"]),
        }],
    );
    assert!(!config[0].0 && config[0].1.contains("configuration differs"));
    s.advance(5).unwrap();
    assert!(editor::checks(initial.topology(), &s, std::slice::from_ref(&absent))[0].0);
    let aged = editor::checks(initial.topology(), &s, std::slice::from_ref(&positive));
    assert!(!aged[0].0 && aged[0].1.contains("aged-out"));
    assert!(!editor::valid_requirements(
        &[positive.clone(), positive.clone()],
        initial.topology()
    ));
    assert!(!editor::valid_requirements(
        &vec![positive; editor::MAX_REQUIREMENTS + 1],
        initial.topology()
    ));
}

#[test]
fn editable_clock_request_is_saved_but_never_applied_or_graded_as_configuration() {
    let simulation = fixture();
    let mut draft = Draft::new(&simulation);
    assert_eq!(draft.advance_seconds, "30");
    draft.advance_seconds = "6".into();
    assert!(draft.valid() && draft.matches(&simulation));
    let text = serde_json::to_string(&draft).unwrap();
    let loaded = Draft::from_json(&text, &simulation).unwrap();
    assert_eq!(loaded.advance_seconds, "6");
    let mut applied = simulation.clone();
    loaded.apply(&mut applied).unwrap();
    assert_eq!(applied, simulation);
    let mut old = serde_json::to_value(&draft).unwrap();
    old.as_object_mut().unwrap().remove("advance_seconds");
    let restored = Draft::from_json(&old.to_string(), &simulation).unwrap();
    assert_eq!(restored.advance_seconds, "30");
    draft.advance_seconds = "not a time".into();
    assert!(draft.valid() && draft.matches(&simulation));
    draft.advance_seconds = "9".repeat(41);
    assert!(!draft.valid());
}

#[test]
fn invalid_editable_labels_receive_contextual_feedback_without_discarding_drafts() {
    let mut simulation = fixture();
    let mut d = Draft::new(&simulation);
    let initial = simulation.clone();
    d.devices.get_mut("left").unwrap().system_name = Some("   ".into());
    let err = d.apply(&mut simulation).unwrap_err();
    assert!(err.contains("left: advertised name"), "{err}");
    assert_eq!(simulation, initial);
    d = Draft::new(&simulation);
    d.devices
        .get_mut("left")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .description = Some("é".into());
    let err = d.apply(&mut simulation).unwrap_err();
    assert!(err.contains("left/uplink: port description"), "{err}");
    assert_eq!(simulation, initial);
    assert!(d.valid());
}
