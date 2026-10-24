use dlms_learning_lab::{
    engine::{self, Goal, State},
    ipv6::{self, InterfaceCondition, Requirement},
    ipv6_workspace::{self, Draft},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
mod support;
use support::*;
fn scenario() -> pack::Scenario {
    let mut lab = pack::bundled().unwrap()[2].clone();
    // This fixture changes activity kind; do not inherit the IPv4 client's lesson.
    lab.client_teaching = None;
    lab.id = "test-ipv6".into();
    lab.unit.clear();
    lab.prerequisites.clear();
    lab.title = "Test IPv6 handover".into();
    let mut t = topology();
    t.devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .prefix = 32;
    lab.initial = State::Ipv6 { topology: t };
    lab.goal = Goal::Ipv6 {
        requirements: vec![
            Requirement::AddressIn {
                device: "workstation".into(),
                interface: "lan".into(),
                network: ipv6::Prefix::parse("2001:db8:10::/64").unwrap(),
                exclude: vec![ip("2001:db8:10::"), ip("2001:db8:10::1")],
                on_link: true,
            },
            Requirement::Reachable {
                label: "Normal operation".into(),
                from: "workstation".into(),
                target: ip("2001:db8:20::20"),
                scope: None,
                first_hop: None,
                when: vec![],
            },
        ],
    };
    lab.solution = vec!["address workstation lan 2001:db8:10::10/64".into()];
    lab
}
#[test]
fn drafts_apply_atomically_and_malformed_input_stays_editable_without_passing() {
    let original = topology();
    let mut t = original.clone();
    let mut draft = Draft::new(&t);
    assert!(draft.valid());
    assert!(draft.matches(&t));
    draft
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "2001:db8:10::99/64".into();
    draft.apply(&mut t).unwrap();
    assert!(draft.matches(&t));
    assert_eq!(
        t.devices["workstation"].interfaces["lan"]
            .address
            .as_ref()
            .unwrap()
            .address,
        ip("2001:db8:10::99")
    );
    let before = t.clone();
    for bad in [
        "not-an-address",
        "2001:::10/64",
        "2001:db8::1/129",
        "2001:db8::1/+64",
        "ff02::1/64",
        "2606:4700::1/64",
    ] {
        draft
            .devices
            .get_mut("workstation")
            .unwrap()
            .interfaces
            .get_mut("lan")
            .unwrap()
            .address = bad.into();
        assert!(draft.valid());
        assert!(draft.apply(&mut t).is_err(), "{bad}");
        assert_eq!(t, before);
        assert!(!draft.matches(&t));
    }
    draft = Draft::new(&original);
    draft
        .devices
        .get_mut("router")
        .unwrap()
        .interfaces
        .remove("left");
    assert!(draft.apply(&mut t).is_err());
    assert_eq!(t, before);
    draft = Draft::new(&original);
    draft.devices.get_mut("workstation").unwrap().routes[0].interface = "missing".into();
    assert!(draft.apply(&mut t).is_err());
    assert_eq!(t, before);
    draft = Draft::new(&original);
    draft
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "x".repeat(65);
    assert!(!draft.valid());
}
#[test]
fn declarative_checks_accept_free_addresses_and_alternate_routing_and_invalidate_after_edits() {
    let lab = scenario();
    let mut state = lab.initial.clone();
    assert!(!engine::solved(&state, &lab.goal));
    for address in [
        "2001:db8:10::99/64",
        "2001:0DB8:0010:0:0:0:0:1234/64",
        "2001:db8:10:0:ffff:ffff:ffff:ffff/64",
    ] {
        engine::execute(&mut state, &format!("address workstation lan {address}")).unwrap();
        assert!(engine::solved(&state, &lab.goal));
    }
    // A global on-link next hop is as valid here as the scoped link-local one.
    engine::execute(&mut state, "remove-route workstation 0").unwrap();
    engine::execute(&mut state, "route workstation ::/0 lan 2001:db8:10::1 10").unwrap();
    assert!(engine::solved(&state, &lab.goal));
    engine::execute(&mut state, "forward router off").unwrap();
    assert!(!engine::solved(&state, &lab.goal));
    engine::execute(&mut state, "forward router on").unwrap();
    assert!(engine::solved(&state, &lab.goal));
    for address in [
        "2001:db8:10::/64",
        "2001:db8:10::1/64",
        "2001:db8:99::10/64",
    ] {
        engine::execute(&mut state, &format!("address workstation lan {address}")).unwrap();
        assert!(!engine::solved(&state, &lab.goal), "{address}");
    }
}
#[test]
fn conditional_connectivity_checks_test_backup_routes_without_changing_current_state() {
    let mut t = topology();
    t.devices.get_mut("workstation").unwrap().interfaces.insert(
        "backup".into(),
        port("backup", "2001:db8:30::10", "fe80::10"),
    );
    t.devices
        .get_mut("router")
        .unwrap()
        .interfaces
        .insert("backup".into(), port("backup", "2001:db8:30::1", "fe80::1"));
    let requirement = Requirement::Reachable {
        label: "Primary interface unavailable".into(),
        from: "workstation".into(),
        target: ip("2001:db8:20::20"),
        scope: None,
        first_hop: None,
        when: vec![InterfaceCondition {
            device: "workstation".into(),
            interface: "lan".into(),
            up: false,
        }],
    };
    let before = t.clone();
    assert!(requirement.valid_for(&t));
    assert!(!requirement.check(&t).0);
    assert_eq!(t, before);
    t.devices
        .get_mut("workstation")
        .unwrap()
        .routes
        .push(route("::/0", "backup", "fe80::1", 10));
    let before = t.clone();
    assert!(requirement.check(&t).0);
    assert_eq!(t, before);
    t.devices.get_mut("server").unwrap().routes.clear();
    assert!(!requirement.check(&t).0);
}
#[test]
fn malformed_packs_adapters_and_changed_shapes_fail_honestly() {
    let lab = scenario();
    let mut serialized = serde_json::to_value(pack::Pack {
        schema_version: 1,
        scenarios: vec![lab.clone()],
    })
    .unwrap();
    assert!(pack::load(&serialized.to_string()).is_ok());
    let mut foreign_teaching = lab.clone();
    foreign_teaching.client_teaching = pack::bundled().unwrap()[2].client_teaching.clone();
    let incompatible = pack::Pack {
        schema_version: 1,
        scenarios: vec![foreign_teaching],
    };
    assert!(pack::load(&serde_json::to_string(&incompatible).unwrap()).is_err());
    serialized["scenarios"][0]["goal"]["requirements"][1]["from"] = "unknown".into();
    assert!(pack::load(&serialized.to_string()).is_err());
    let mut t = topology();
    for command in [
        "route workstation ::/0 missing fe80::1 1",
        "route workstation 2001:db8::1/64 lan fe80::1 1",
        "remove-route workstation 31",
        "address workstation lan bad/64",
        "link workstation lan maybe",
        "bash -c anything",
        "address workstation lan $(id)",
    ] {
        let before = t.clone();
        assert!(
            ipv6_workspace::adapter(&mut t, command).is_err(),
            "{command}"
        );
        assert_eq!(t, before);
    }
    // A valid but different saved graph cannot cause objective indexing to panic.
    t.devices.remove("workstation");
    let Goal::Ipv6 { requirements } = &lab.goal else {
        panic!()
    };
    for r in requirements {
        assert!(!r.check(&t).0);
    }
    let invalid = Requirement::Reachable {
        label: "Invalid link scope".into(),
        from: "server".into(),
        target: ip("fe80::1"),
        scope: None,
        first_hop: None,
        when: vec![],
    };
    assert!(!invalid.valid_for(&t));
}
#[test]
fn progress_preserves_both_invalid_drafts_and_previous_labs_but_never_shows_stale_success() {
    let lab = scenario();
    let mut labs = pack::bundled().unwrap();
    labs.push(lab.clone());
    let mut attempt = Attempt::new(&lab, 1);
    workspace::prepare(&mut attempt);
    assert!(!progress::current_solved(&attempt, &lab));
    let State::Ipv6 { topology } = &mut attempt.state else {
        panic!()
    };
    let draft = attempt
        .workspace
        .as_mut()
        .unwrap()
        .ipv6_draft
        .as_mut()
        .unwrap();
    draft
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "2001:db8:10::10/64".into();
    draft.apply(topology).unwrap();
    assert!(progress::current_solved(&attempt, &lab));
    attempt
        .workspace
        .as_mut()
        .unwrap()
        .ipv6_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "unfinished::value/".into();
    assert!(!progress::current_solved(&attempt, &lab));
    assert!(engine::solved(&attempt.state, &lab.goal));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), attempt);
    p.attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 2));
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(progress::load(&path, &labs).unwrap()).unwrap()
    );
}

#[test]
fn imports_reject_changed_physical_structure_without_rejecting_editable_wrong_answers() {
    let lab = scenario();
    let mut labs = pack::bundled().unwrap();
    labs.push(lab.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(&lab, 1);
    workspace::prepare(&mut a);
    p.attempts.insert(lab.id.clone(), a);
    let original = serde_json::to_value(&p).unwrap();
    for mutation in 0..3 {
        let mut value = original.clone();
        match mutation {
            0 => {
                value["attempts"]["test-ipv6"]["state"]["topology"]["devices"]["workstation"]["interfaces"]
                    ["lan"]["link"] = "invented-link".into()
            }
            1 => {
                value["attempts"]["test-ipv6"]["state"]["topology"]["devices"]
                    .as_object_mut()
                    .unwrap()
                    .remove("server");
            }
            _ => {
                value["attempts"]["test-ipv6"]["workspace"]["ipv6_draft"]["devices"]
                    .as_object_mut()
                    .unwrap()
                    .remove("server");
            }
        }
        assert!(
            progress::decode(&serde_json::to_vec(&value).unwrap(), &labs)
                .err()
                .unwrap()
                .contains("structure differs")
        );
    }
    let mut value = original;
    value["attempts"]["test-ipv6"]["workspace"]["ipv6_draft"]["devices"]["workstation"]["interfaces"]
        ["lan"]["address"] = "incomplete answer".into();
    assert!(progress::decode(&serde_json::to_vec(&value).unwrap(), &labs).is_ok());
}

fn teaching_lesson() -> dlms_learning_lab::ipv6_teaching::Lesson {
    use dlms_learning_lab::{ipv6_teaching::Lesson, script_teaching::Concept};
    let mut fixture = topology();
    for device in fixture.devices.values_mut() {
        for iface in device.interfaces.values_mut() {
            let assignment = iface.address.as_mut().unwrap();
            assignment.address = ip(&assignment
                .address
                .to_string()
                .replace("2001:db8", "fd77:55"));
        }
    }
    fixture
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .prefix = 32;
    Lesson{start_with_routes:false,prerequisite_note:"Use the native address and route fields. No terminal setup is needed.".into(),concepts:vec![Concept{title:"Keep the prefix correct".into(),text:"The manually connected prefix determines which destinations are treated as on the local link.".into()}],example_description:"Repair the practice workstation prefix, then test its path and reply.".into(),fixture,example_solution:vec!["address workstation lan fd77:55:10::10/64".into()],outcomes:vec![Requirement::Reachable{label:"Practice path".into(),from:"workstation".into(),target:ip("fd77:55:20::20"),scope:None,first_hop:None,when:vec![]}],try_change:"Choose another free address in the same client /64, then check again.".into()}
}
#[test]
fn native_example_fill_and_apply_never_touch_the_assignment_or_award_credit() {
    use dlms_learning_lab::{
        ipv6_teaching::{self, Learning},
        script_teaching::View,
    };
    let lesson = teaching_lesson();
    lesson.validate().unwrap();
    let mut lab = scenario();
    lab.ipv6_teaching = Some(lesson.clone());
    let mut a = Attempt::new(&lab, 1);
    workspace::prepare(&mut a);
    ipv6_teaching::prepare(&mut a, &lesson, true);
    let assignment = a.state.clone();
    let original_draft = a.workspace.as_ref().unwrap().ipv6_draft.clone();
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .ipv6_learning
        .as_mut()
        .unwrap();
    assert_eq!(learning.view, View::Introduction);
    let before = learning.topology.clone();
    learning.fill_example(&lesson).unwrap();
    assert_eq!(learning.topology, before);
    assert!(!learning.draft.matches(&learning.topology));
    assert!(learning.checks(&lesson).is_empty());
    learning.apply().unwrap();
    assert!(learning.checks(&lesson).iter().all(|r| r.0));
    learning
        .draft
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "fd77:55:10::99/64".into();
    learning.apply().unwrap();
    assert!(learning.checks(&lesson).iter().all(|r| r.0));
    learning
        .draft
        .devices
        .get_mut("workstation")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "not finished".into();
    assert!(learning.checks(&lesson).is_empty());
    assert!(learning.apply().is_err());
    assert!(learning.valid());
    assert_eq!(a.state, assignment);
    assert_eq!(a.workspace.as_ref().unwrap().ipv6_draft, original_draft);
    assert!(!progress::current_solved(&a, &lab));
    let mut labs = pack::bundled().unwrap();
    labs.push(lab.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(progress::load(&path, &labs).unwrap()).unwrap()
    );
    let mut bad = lesson.clone();
    bad.example_solution = vec!["address workstation lan fd77:55:10::10/32".into()];
    assert!(bad.validate().is_err());
    let mut bad = Learning::new(&lesson);
    bad.topology.devices.remove("server");
    assert!(!bad.valid());
}

#[test]
fn wrong_shape_draft_cannot_replace_a_known_good_progress_file() {
    let lab = scenario();
    let mut attempt = Attempt::new(&lab, 1);
    workspace::prepare(&mut attempt);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), attempt);
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    let draft = p
        .attempts
        .get_mut(&lab.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .ipv6_draft
        .as_mut()
        .unwrap();
    draft.devices.remove("server");
    assert!(draft.valid());
    assert!(
        progress::save(&path, &p)
            .err()
            .unwrap()
            .contains("draft structure")
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn ordinary_workstation_choices_exclude_reserved_anycast_but_keep_all_one_host_bits() {
    let lab = scenario();
    let mut state = lab.initial.clone();
    for address in [
        "2001:db8:10::/64",
        "2001:db8:10:0:fdff:ffff:ffff:ff80/64",
        "2001:db8:10:0:fdff:ffff:ffff:ffff/64",
    ] {
        engine::execute(&mut state, &format!("address workstation lan {address}")).unwrap();
        assert!(!engine::solved(&state, &lab.goal));
        assert!(engine::checks(&state, &lab.goal)[0].1.contains("reserved"));
    }
    engine::execute(
        &mut state,
        "address workstation lan 2001:db8:10:0:ffff:ffff:ffff:ffff/64",
    )
    .unwrap();
    assert!(engine::solved(&state, &lab.goal));
}
