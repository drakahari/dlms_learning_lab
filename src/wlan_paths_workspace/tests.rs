use super::*;
use crate::{switching::Mode, wlan_paths::fixture::world};
fn task() -> Task {
    let mut initial = world();
    initial.mode = ApMode::FlexConnect;
    let switch = initial.fabric.switches.get_mut("access").unwrap();
    switch.ports.get_mut("ap").unwrap().mode = Mode::Access { vlan: 10 };
    switch.ports.get_mut("controller").unwrap().mode = Mode::Trunk {
        native: 10,
        allowed: BTreeSet::from([10, 30]),
    };
    let mut reference = initial.clone();
    reference.mode = ApMode::Local;
    reference
        .fabric
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("controller")
        .unwrap()
        .mode = Mode::Trunk {
        native: 10,
        allowed: BTreeSet::from([10, 20, 30]),
    };
    Task {
        id: "campus-path".into(),
        initial,
        edit_mode: true,
        edit_native: false,
        edit_wlans: vec![WlanEdit {
            id: "staff-wlan".into(),
            forwarding: true,
            local_vlan: false,
            controller_vlan: false,
        }],
        edit_ports: vec![PortEdit {
            switch: "access".into(),
            port: "controller".into(),
            up: false,
            mode: true,
        }],
        cases: vec![
            Case {
                id: "staff-service".into(),
                label: "Staff service".into(),
                wlan: "staff-wlan".into(),
                destination: "staff".into(),
                expected: Expected::Path {
                    forwarding: Forwarding::Central,
                    vlan: 20,
                },
            },
            Case {
                id: "guest-service".into(),
                label: "Guest service".into(),
                wlan: "guest-wlan".into(),
                destination: "guest".into(),
                expected: Expected::Path {
                    forwarding: Forwarding::Central,
                    vlan: 30,
                },
            },
            Case {
                id: "staff-isolation".into(),
                label: "Staff isolation from guest service".into(),
                wlan: "staff-wlan".into(),
                destination: "guest".into(),
                expected: Expected::Isolated,
            },
        ],
        reference: Fields::new(&reference),
    }
}
#[test]
fn source_reference_works_and_alternate_mode_and_forwarding_choice_passes() {
    let t = task();
    t.validate().unwrap();
    assert!(!t.checks(&t.initial_fields()).iter().all(|c| c.correct));
    assert!(
        t.checks(&t.reference)
            .iter()
            .all(|c| c.correct && c.evaluated)
    );
    let mut alternate = t.reference.clone();
    alternate.mode = ApMode::FlexConnect;
    alternate.wlans[0].forwarding = Forwarding::Central;
    assert_ne!(alternate, t.reference);
    assert!(
        t.checks(&alternate)
            .iter()
            .all(|c| c.correct && c.evaluated)
    );
}
#[test]
fn mode_only_repair_and_data_only_repair_cannot_bypass_both_required_paths() {
    let t = task();
    let mut fields = t.initial_fields();
    fields.mode = ApMode::Local;
    let checks = t.checks(&fields);
    assert!(checks[0].correct);
    assert!(
        !checks
            .iter()
            .find(|c| c.id == "staff-service")
            .unwrap()
            .correct
    );
    fields = t.initial_fields();
    fields.ports = t.reference.ports.clone();
    assert!(!t.checks(&fields).iter().all(|c| c.correct));
}
#[test]
fn malformed_blank_protected_and_foreign_drafts_never_pass_or_earn_negative_credit() {
    let t = task();
    for text in ["", "abc", "0", "4095"] {
        let mut f = t.reference.clone();
        f.wlans[0].local_vlan = text.into();
        let c = t.checks(&f);
        assert!(!c.iter().all(|c| c.correct));
        assert!(
            !c.iter()
                .find(|c| c.id == "staff-isolation")
                .unwrap()
                .evaluated
        );
    }
    let mut f = t.reference.clone();
    f.wlans[1].forwarding = Forwarding::Local;
    assert!(t.world(&f).unwrap_err().contains("read-only"));
    let mut f = t.reference.clone();
    f.ports
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("guest")
        .unwrap()
        .up = false;
    assert!(t.world(&f).is_err());
    let mut f = t.reference.clone();
    f.wlans[0].id = "foreign".into();
    assert!(t.world(&f).is_err());
}
#[test]
fn broken_control_is_deferred_not_successful_isolation() {
    let mut t = task();
    t.edit_native = true;
    let mut f = t.reference.clone();
    f.native = "11".into();
    let checks = t.checks(&f);
    assert!(!checks[0].correct);
    assert!(checks[0].feedback.contains("differs"));
    assert!(checks[1..].iter().all(|c| !c.evaluated && !c.correct));
}
#[test]
fn duplicate_missing_unreachable_initially_solved_and_negative_only_definitions_refuse() {
    let t = task();
    let mut n = t.clone();
    n.cases.push(n.cases[0].clone());
    assert!(n.validate().is_err());
    let mut n = t.clone();
    n.cases.retain(|c| c.wlan != "guest-wlan");
    assert!(n.validate().is_err());
    let mut n = t.clone();
    n.reference = n.initial_fields();
    assert!(n.validate().is_err());
    let mut n = t.clone();
    n.initial = n.world(&n.reference).unwrap();
    assert!(n.validate().is_err());
    let mut n = t.clone();
    for c in &mut n.cases {
        c.expected = Expected::Isolated;
    }
    assert!(n.validate().is_err());
}
fn definition() -> Definition {
    let t = task();
    let mut example = t.clone();
    example.id = "example-campus-path".into();
    let mut reference = example.world(&example.reference).unwrap();
    fn shift(w: &mut World) {
        w.ap_native += 100;
        w.controller_native += 100;
        for m in &mut w.wlans {
            m.local_vlan += 100;
            m.controller_vlan += 100;
        }
        for sw in w.fabric.switches.values_mut() {
            sw.vlans = sw.vlans.iter().map(|n| n + 100).collect();
            for p in sw.ports.values_mut() {
                p.mode = match &p.mode {
                    Mode::Access { vlan } => Mode::Access { vlan: vlan + 100 },
                    Mode::Trunk { native, allowed } => Mode::Trunk {
                        native: native + 100,
                        allowed: allowed.iter().map(|n| n + 100).collect(),
                    },
                };
            }
        }
    }
    shift(&mut example.initial);
    shift(&mut reference);
    example.reference = Fields::new(&reference);
    for c in &mut example.cases {
        if let Expected::Path { vlan, .. } = &mut c.expected {
            *vlan += 100;
        }
    }
    let def = Definition {
        initial: t,
        example,
    };
    def.validate().unwrap();
    def
}

#[test]
fn separate_examples_edits_roundtrip_and_selected_reset_cannot_fabricate_completion() {
    let def = definition();
    let mut s = Session::new(&def);
    let untouched = serde_json::to_vec(&s.assignment).unwrap();
    s.practice.fill(&def.example).unwrap();
    assert!(!s.practice.checked());
    s.practice.check(&def.example).unwrap();
    assert!(s.practice.solved(&def.example));
    assert!(!s.solved(&def));
    assert_eq!(serde_json::to_vec(&s.assignment).unwrap(), untouched);
    s.assignment.fill(&def.initial).unwrap();
    s.assignment.check(&def.initial).unwrap();
    assert!(s.solved(&def));
    s.assignment.fields.mode = ApMode::FlexConnect;
    s.assignment.edited();
    assert!(!s.solved(&def));
    let saved = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&saved).unwrap();
    assert!(restored.valid(&def));
    assert!(!restored.solved(&def));
    let practice = serde_json::to_vec(&restored.practice).unwrap();
    restored.assignment.reset(&def.initial);
    assert_eq!(serde_json::to_vec(&restored.practice).unwrap(), practice);
    assert!(!restored.solved(&def));
    let mut foreign = def.clone();
    foreign.initial.id = "other-task".into();
    assert!(!restored.valid(&foreign));
}

#[test]
fn disconnecting_a_negative_case_destination_does_not_prove_vlan_isolation() {
    let mut t = task();
    t.edit_ports.push(PortEdit {
        switch: "access".into(),
        port: "guest".into(),
        up: true,
        mode: false,
    });
    t.validate().unwrap();
    let mut fields = t.reference.clone();
    fields
        .ports
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("guest")
        .unwrap()
        .up = false;
    let checks = t.checks(&fields);
    let isolation = checks.iter().find(|c| c.id == "staff-isolation").unwrap();
    assert!(
        !isolation.correct,
        "A disconnected guest endpoint is not verified VLAN separation"
    );
    assert!(!isolation.evaluated);
    assert!(isolation.feedback.contains("disconnection"));
    fields
        .ports
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("guest")
        .unwrap()
        .up = true;
    assert!(t.checks(&fields).iter().all(|c| c.correct && c.evaluated));
}

#[test]
fn editable_numeric_errors_explain_blank_format_and_range_and_preserve_raw_drafts() {
    let mut t = task();
    t.edit_wlans[0].local_vlan = true;
    t.validate().unwrap();
    for (raw, message) in [
        ("", "enter a VLAN"),
        ("abc", "whole VLAN"),
        ("4095", "1–4094"),
    ] {
        let mut draft = Draft::new(&t);
        draft.fields = t.reference.clone();
        draft.fields.wlans[0].local_vlan = raw.into();
        assert!(draft.valid(&t));
        let checked = draft.check(&t).unwrap();
        assert!(!draft.solved(&t));
        assert!(
            checked[0].feedback.contains(message),
            "{raw:?}: {:?}",
            checked[0]
        );
        assert!(checked[0].feedback.contains("Staff WLAN local VLAN"));
        let resumed: Draft = serde_json::from_slice(&serde_json::to_vec(&draft).unwrap()).unwrap();
        assert_eq!(resumed.fields.wlans[0].local_vlan, raw);
        assert!(!resumed.solved(&t));
    }
}

#[test]
fn switching_the_viewed_switch_keeps_credit_but_breaking_a_port_clears_current_success() {
    let mut t = task();
    let mut reference = t.world(&t.reference).unwrap();
    let spare = crate::switching::Switch {
        label: "Unconnected spare".into(),
        vlans: BTreeSet::from([10]),
        ports: std::collections::BTreeMap::from([(
            "spare-port".into(),
            crate::switching::Port {
                up: true,
                mode: Mode::Access { vlan: 10 },
            },
        )]),
    };
    t.initial
        .fabric
        .switches
        .insert("spare".into(), spare.clone());
    reference.fabric.switches.insert("spare".into(), spare);
    t.reference = Fields::new(&reference);
    t.validate().unwrap();
    let mut d = Draft::new(&t);
    d.fill(&t).unwrap();
    d.check(&t).unwrap();
    assert!(d.solved(&t));
    d.fields.ports.selected_switch = "spare".into();
    assert!(
        d.checked() && d.solved(&t),
        "Navigation is not a changed answer"
    );
    d.fields
        .ports
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("controller")
        .unwrap()
        .allowed = "10,30".into();
    assert!(
        !d.checked() && !d.solved(&t),
        "A current breaking edit cannot retain prior success"
    );
}

#[test]
fn native_vlan_translation_cannot_masquerade_as_the_required_vlan_path() {
    use crate::switching::{Cable, Jack, Port, Switch};
    use std::collections::BTreeMap;
    fn change(w: &mut World) {
        w.fabric
            .cables
            .iter_mut()
            .find(|c| c.id == "cable-staff")
            .unwrap()
            .b = Jack::Switch {
            switch: "remote".into(),
            port: "staff".into(),
        };
        let allowed = BTreeSet::from([10, 20, 30]);
        w.fabric.switches.get_mut("access").unwrap().ports.insert(
            "uplink".into(),
            Port {
                up: true,
                mode: Mode::Trunk {
                    native: 20,
                    allowed: allowed.clone(),
                },
            },
        );
        w.fabric.switches.insert(
            "remote".into(),
            Switch {
                label: "Remote switch".into(),
                vlans: allowed.clone(),
                ports: BTreeMap::from([
                    (
                        "uplink".into(),
                        Port {
                            up: true,
                            mode: Mode::Trunk {
                                native: 30,
                                allowed,
                            },
                        },
                    ),
                    (
                        "staff".into(),
                        Port {
                            up: true,
                            mode: Mode::Access { vlan: 30 },
                        },
                    ),
                ]),
            },
        );
        w.fabric.cables.push(Cable {
            id: "inter-switch".into(),
            a: Jack::Switch {
                switch: "access".into(),
                port: "uplink".into(),
            },
            b: Jack::Switch {
                switch: "remote".into(),
                port: "uplink".into(),
            },
            up: true,
        });
    }
    let mut t = task();
    let mut reference = t.world(&t.reference).unwrap();
    change(&mut t.initial);
    change(&mut reference);
    t.reference = Fields::new(&reference);
    let world = t.world(&t.reference).unwrap();
    let Outcome::Data(actual) = world.data_path("staff-wlan", "staff").unwrap() else {
        panic!("Management transport remains up")
    };
    assert!(
        actual.bidirectional(),
        "The old bounded fabric really exposes this native-VLAN leak"
    );
    assert_eq!(actual.vlan, 20);
    assert!(
        actual
            .forward
            .as_ref()
            .unwrap()
            .iter()
            .any(|hop| hop.vlan == 30)
    );
    let rows = t.checks(&t.reference);
    let staff = rows.iter().find(|r| r.id == "staff-service").unwrap();
    assert!(
        !staff.correct,
        "Reaching VLAN30 is not the required VLAN20 path"
    );
    assert!(staff.feedback.contains("changes VLAN"));
    assert!(
        t.validate().is_err(),
        "An accidentally translated reference must not be authored as valid"
    );
}

fn scenario() -> crate::pack::Scenario {
    let def = definition();
    let solution = vec![
        "@ap-edit {\"kind\":\"mode\",\"value\":\"local\"}".to_string(),
        format!(
            "@ap-edit {}",
            serde_json::json!({"kind":"port","switch":"access","port":"controller","settings":def.initial.reference.ports.switches["access"].ports["controller"]})
        ),
        "@ap-check".into(),
    ];
    serde_json::from_value(serde_json::json!({
        "id":def.initial.id,"version":1,"track":"CCNA","topic":"Wireless architecture","title":"Synthetic AP-path binding fixture",
        "summary":"Verify supplied controller and client VLAN paths.","objective":"Restore the staff path and preserve guest service and separation.",
        "difficulty":"Intermediate","minutes":20,"steps":["Inspect the supplied topology and configure its forwarding paths."],
        "hints":["Separate management and client data."],"sources":["https://www.cisco.com/"],"simplifications":"Supplied VLAN frames only; no real controller join or radio.",
        "initial":crate::engine::State::WlanPaths{simulation:Box::new(Session::new(&def))},
        "goal":crate::engine::Goal::WlanPaths{definition:Box::new(def)},"solution":solution
    })).unwrap()
}
#[test]
fn free_shapes_keep_wrong_drafts_but_reject_unknown_versions_ids_and_snapshots() {
    let def = definition();
    let s = Session::new(&def);
    s.shape().unwrap();
    let mut n = s.clone();
    n.version = 2;
    assert!(n.shape().is_err());
    let mut n = s.clone();
    n.practice.task_id = n.assignment.task_id.clone();
    assert!(n.shape().is_err());
    let mut n = s.clone();
    n.assignment.fields.wlans.clear();
    assert!(n.shape().is_err());
    let mut n = s.clone();
    n.assignment.fields.native = "abc".into();
    assert!(n.shape().is_ok());
    let mut n = s.clone();
    n.assignment.fields.native = "\n".into();
    assert!(n.shape().is_err());
    let mut n = s.clone();
    let mut snapshot = n.assignment.fields.clone();
    snapshot.wlans[0].id = "foreign".into();
    n.assignment.checked_fields = Some(snapshot);
    assert!(n.shape().is_err());
    let mut bad = def.initial.clone();
    bad.cases.clear();
    let mut d = Draft::new(&def.initial);
    d.fields = def.initial.reference.clone();
    d.checked_fields = Some(d.fields.clone());
    assert!(!d.solved(&bad));
    assert!(d.check(&bad).is_err());
    assert!(d.fill(&bad).is_err());
}
#[test]
fn source_engine_actions_grade_actual_state_and_refuse_host_and_unsupported_input_atomically() {
    let s = scenario();
    source(&s).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    let original = serde_json::to_value(&a.state).unwrap();
    for raw in [
        "bash /home/private/script",
        "curl https://example.invalid",
        "@ap-edit {}",
        "@ap-edit {\"kind\":\"mode\",\"value\":\"monitor\"}",
        "@ap-edit {\"kind\":\"native\",\"value\":\"99999\"}",
    ] {
        assert!(crate::engine::execute_for_goal(&mut a.state, &s.goal, raw).is_err());
        assert_eq!(serde_json::to_value(&a.state).unwrap(), original);
    }
    for raw in &s.solution {
        crate::engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    assert!(crate::progress::current_solved(&a, &s));
    assert!(crate::engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
    crate::engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "@ap-edit {\"kind\":\"mode\",\"value\":\"flex_connect\"}",
    )
    .unwrap();
    assert!(!crate::progress::current_solved(&a, &s));
    crate::engine::execute_for_goal(&mut a.state, &s.goal, "@ap-check").unwrap();
    assert!(!crate::progress::current_solved(&a, &s));
    a.scenario_version += 1;
    assert!(context(&a, &s).is_err());
    assert!(!crate::progress::current_solved(&a, &s));
}
#[test]
fn packs_reject_mismatched_source_prechecked_work_and_unreachable_references() {
    let s = scenario();
    let text =
        serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[s.clone()]}))
            .unwrap();
    assert_eq!(crate::pack::load_catalog(&[&text]).unwrap().len(), 1);
    let mut bad = s.clone();
    if let crate::engine::Goal::WlanPaths { definition } = &mut bad.goal {
        definition.initial.id = "foreign".into();
    }
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    if let crate::engine::State::WlanPaths { simulation } = &mut bad.initial {
        simulation.assignment.checked_fields = Some(simulation.assignment.fields.clone());
    }
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    if let crate::engine::Goal::WlanPaths { definition } = &mut bad.goal {
        definition.initial.reference = definition.initial.initial_fields();
    }
    assert!(source(&bad).is_err());
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::WlanPaths { simulation } = &mut a.state {
        simulation.assignment.task_id = "foreign".into();
    }
    assert!(context(&a, &s).is_err());
}
#[test]
fn actual_disk_roundtrip_retains_all_424_old_attempts_and_independent_new_drafts() {
    let mut sources = crate::pack::accepted424().unwrap();
    assert_eq!(sources.len(), 424);
    assert_eq!(crate::course::accepted424(&sources).unwrap().len(), 163);
    let s = scenario();
    assert!(!sources.iter().any(|x| x.id == s.id));
    sources.push(s.clone());
    let mut p = crate::progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &sources[..424] {
        let mut a = crate::progress::Attempt::new(source, 1);
        crate::workspace::prepare(&mut a);
        if crate::command_location_workspace::has_source(source) {
            crate::command_location_workspace::prepare(&mut a, source, true).unwrap();
        }
        if crate::accounts_template_workspace::has_source(source) {
            crate::accounts_template_workspace::prepare(&mut a, source, true).unwrap();
        }
        p.attempts.insert(source.id.clone(), a);
    }
    let before = serde_json::to_value(&p.attempts).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::WlanPaths { simulation } = &mut a.state {
        simulation
            .assignment
            .fields
            .ports
            .switches
            .get_mut("access")
            .unwrap()
            .ports
            .get_mut("controller")
            .unwrap()
            .native = "abc".into();
        simulation.practice.fill(&definition().example).unwrap();
        simulation.practice.check(&definition().example).unwrap();
    }
    p.attempts.insert(s.id.clone(), a.clone());
    p.favorites.insert(s.id.clone());
    p.favorites.insert(sources[0].id.clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261111/next-wlc/progress-tests");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    crate::progress::save_with_catalog(&path, &p, &sources).unwrap();
    let original = std::fs::read(&path).unwrap();
    let restored = crate::progress::load(&path, &sources).unwrap();
    assert_eq!(restored.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
    let mut old = restored.attempts.clone();
    old.remove(&s.id);
    assert_eq!(serde_json::to_value(old).unwrap(), before);
    assert!(crate::progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut forged: crate::progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    if let crate::engine::State::WlanPaths { simulation } =
        &mut forged.attempts.get_mut(&s.id).unwrap().state
    {
        simulation.assignment.fields.native = "11".into();
    }
    assert!(crate::progress::save_with_catalog(&path, &forged, &sources).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    p.attempts.get_mut(&s.id).unwrap().scenario_version += 1;
    assert!(crate::progress::save_with_catalog(&path, &p, &sources).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
