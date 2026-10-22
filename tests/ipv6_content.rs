use dlms_learning_lab::{
    engine::{self, Goal, State},
    ipv6::Requirement,
    ipv6_teaching::Learning,
    ipv6_workspace::{Draft, RouteDraft},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.ipv6_teaching.is_some())
        .collect()
}
fn actions(state: &mut State, commands: &[&str]) {
    for cmd in commands {
        engine::execute(state, cmd).unwrap();
    }
}
#[test]
fn every_ipv6_assignment_accepts_meaningful_alternatives_and_rejects_plausible_partial_work() {
    let alternatives: Vec<Vec<&str>> = vec![
        vec!["address NewPC lan 2001:0DB8:0044:0:0:0:0:37/64"],
        vec![
            "address Client lan 2001:db8:10::10/64",
            "remove-route Client 0",
            "route Client ::/0 lan 2001:db8:10::1 12",
        ],
        vec![
            "remove-route R1 0",
            "route R1 2001:db8:20::1/128 transit fe80::2 3",
            "route R1 2001:db8:20::20/128 transit fe80::2 4",
        ],
        vec![
            "remove-route Client 1",
            "route Client 2001:db8:20::/64 lan fe80::1 200",
        ],
        vec![
            "remove-route Client 0",
            "remove-route Client 0",
            "route Client ::/0 backup 2001:db8:30::1 50",
            "route Client ::/0 primary 2001:db8:10::1 5",
        ],
        vec![
            "address Client lan 2001:db8:10::10/64",
            "forward R1 on",
            "remove-route R1 0",
            "route R1 2001:db8:20::1/128 transit fe80::2 3",
            "route R1 2001:db8:20::20/128 transit fe80::2 4",
            "route Server 2001:db8:10::/64 lan fe80::1 8",
        ],
    ];
    let mistakes: Vec<Vec<&str>> = vec![
        vec!["address NewPC lan 2001:db8:44::10/64"],
        vec!["address Client lan 2001:db8:10::10/64"],
        vec!["remove-route R1 0", "route R1 2001:db8:20::/64 transit - 1"],
        vec![
            "remove-route Client 1",
            "route Client 2001:db8:20::/64 lan fe80::99 254",
        ],
        vec![
            "remove-route Client 0",
            "remove-route Client 0",
            "route Client ::/0 primary fe80::1 50",
            "route Client ::/0 backup fe80::1 1",
        ],
        vec![
            "address Client lan 2001:db8:10::10/64",
            "forward R1 on",
            "remove-route R1 0",
            "route R1 2001:db8:20::/64 transit fe80::2 1",
        ],
    ];
    let labs = labs();
    assert_eq!(labs.len(), 6);
    for ((lab, alt), wrong) in labs.iter().zip(alternatives).zip(mistakes) {
        let mut state = lab.initial.clone();
        actions(&mut state, &alt);
        assert!(
            engine::solved(&state, &lab.goal),
            "{} {:?}",
            lab.id,
            engine::checks(&state, &lab.goal)
        );
        let mut state = lab.initial.clone();
        actions(&mut state, &wrong);
        assert!(
            !engine::solved(&state, &lab.goal),
            "{} plausible wrong",
            lab.id
        );
    }
}
#[test]
fn wrong_prefixed_neighbor_reserved_and_unsupported_choices_explain_the_actual_mistake() {
    let lab = labs().remove(0);
    let mut state = lab.initial.clone();
    let checks = [
        ("2001:db8:44::20/32", "prefix is /32"),
        ("2001:db8:99::20/64", "outside"),
        ("2001:db8:44::/64", "reserved"),
        ("2001:db8:44::10/64", "already assigned"),
    ];
    for (address, message) in checks {
        engine::execute(&mut state, &format!("address NewPC lan {address}")).unwrap();
        assert!(engine::checks(&state, &lab.goal)[0].1.contains(message));
        assert!(!engine::solved(&state, &lab.goal));
    }
    let State::Ipv6 { topology } = &lab.initial else {
        panic!()
    };
    let mut draft = Draft::new(topology);
    draft
        .devices
        .get_mut("NewPC")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "2001:::99/64".into();
    let before = topology.clone();
    let mut current = before.clone();
    assert!(
        draft
            .apply(&mut current)
            .unwrap_err()
            .contains("Malformed IPv6")
    );
    assert_eq!(current, before);
    draft
        .devices
        .get_mut("NewPC")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = String::new();
    draft.apply(&mut current).unwrap();
    let Goal::Ipv6 { requirements } = &lab.goal else {
        panic!()
    };
    assert!(requirements[0].check(&current).1.contains("No address"));
}
#[test]
fn every_worked_example_and_suggested_experiment_has_verified_results() {
    for (index, lab) in labs().into_iter().enumerate() {
        let lesson = lab.ipv6_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let mut learning = Learning::new(lesson);
        let initial = learning.topology.clone();
        let selected = learning.draft.selected_device.clone();
        learning.draft.show_routes = index != 0;
        learning.fill_example(lesson).unwrap();
        assert_eq!(learning.topology, initial);
        assert_eq!(learning.draft.selected_device, selected);
        assert_eq!(learning.draft.show_routes, index != 0);
        learning.apply().unwrap();
        assert!(learning.checks(lesson).iter().all(|r| r.0));
        match index {
            0 => {
                learning
                    .draft
                    .devices
                    .get_mut("NewPC")
                    .unwrap()
                    .interfaces
                    .get_mut("lan")
                    .unwrap()
                    .address = "fd71:55:44::37/64".into();
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().all(|r| r.0));
            }
            1 => {
                learning.draft.devices.get_mut("Client").unwrap().routes[0].next_hop =
                    "fe80::88".into();
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().any(|r| !r.0));
                learning.draft.devices.get_mut("Client").unwrap().routes[0].next_hop =
                    "fe80::1".into();
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().all(|r| r.0));
            }
            2 => {
                learning.draft.devices.get_mut("R1").unwrap().routes[0].interface = "office".into();
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().any(|r| !r.0));
                learning.draft.devices.get_mut("R1").unwrap().routes[0].interface =
                    "transit".into();
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().all(|r| r.0));
            }
            3 => {
                learning
                    .draft
                    .devices
                    .get_mut("Client")
                    .unwrap()
                    .routes
                    .push(RouteDraft {
                        destination: "fd71:55:20::/64".into(),
                        interface: "lan".into(),
                        next_hop: "fe80::1".into(),
                        distance: "200".into(),
                    });
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().all(|r| r.0));
            }
            4 => {
                let routes = &mut learning.draft.devices.get_mut("Client").unwrap().routes;
                routes[0].distance = "5".into();
                routes[1].distance = "50".into();
                learning.apply().unwrap();
                assert!(learning.checks(lesson).iter().all(|r| r.0));
                let routes = &mut learning.draft.devices.get_mut("Client").unwrap().routes;
                routes[0].distance = "50".into();
                routes[1].distance = "5".into();
                learning.apply().unwrap();
                assert!(
                    learning
                        .checks(lesson)
                        .iter()
                        .any(|r| r.1.contains("first interface"))
                );
            }
            _ => {
                learning
                    .draft
                    .devices
                    .get_mut("Server")
                    .unwrap()
                    .routes
                    .clear();
                learning.apply().unwrap();
                let checks = learning.checks(lesson);
                assert!(
                    checks
                        .iter()
                        .any(|r| r.0 && r.1.contains("Office reaches branch router"))
                );
                assert!(checks.iter().any(|r| !r.0 && r.1.contains("reply fails")));
            }
        }
        assert!(!engine::solved(&lab.initial, &lab.goal));
    }
}
#[test]
fn completed_networks_invalidate_on_current_changes_and_both_drafts_resume() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let breaks = [
        "address NewPC lan 2001:db8:44::10/64",
        "link Client lan down",
        "forward R1 off",
        "route Client 2001:db8:20::/64 lan fe80::99 200",
        "remove-route Client 1",
        "remove-route Server 0",
    ];
    for (lab, change) in labs().iter().zip(breaks) {
        let mut a = Attempt::new(lab, 1);
        workspace::prepare(&mut a);
        let lesson = lab.ipv6_teaching.as_ref().unwrap();
        dlms_learning_lab::ipv6_teaching::prepare(&mut a, lesson, true);
        for cmd in &lab.solution {
            engine::execute(&mut a.state, cmd).unwrap();
        }
        assert!(engine::solved(&a.state, &lab.goal));
        let State::Ipv6 { topology } = &a.state else {
            panic!()
        };
        a.workspace.as_mut().unwrap().ipv6_draft = Some(Draft::new(topology));
        assert!(progress::current_solved(&a, lab));
        engine::execute(&mut a.state, change).unwrap();
        assert!(!engine::solved(&a.state, &lab.goal), "{}", lab.id);
        assert!(!progress::current_solved(&a, lab));
        p.attempts.insert(lab.id.clone(), a);
    }
    p.attempts
        .insert(all[0].id.clone(), Attempt::new(&all[0], 5));
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap()
    );
}
#[test]
fn failover_preference_is_observed_not_forced_by_the_probe_and_invalid_targets_are_rejected() {
    let lab = labs().remove(4);
    let State::Ipv6 { mut topology } = lab.initial.clone() else {
        panic!()
    };
    topology.devices.get_mut("Client").unwrap().routes[1].via = Some("fe80::1".parse().unwrap());
    topology.devices.get_mut("Client").unwrap().routes[1].distance = 1;
    topology.devices.get_mut("Client").unwrap().routes[0].distance = 50;
    assert!(
        topology
            .ping("Client", "2001:db8:20::20".parse().unwrap(), None)
            .is_ok()
    );
    let Goal::Ipv6 { requirements } = &lab.goal else {
        panic!()
    };
    assert!(
        requirements
            .iter()
            .any(|r| !r.check(&topology).0 && r.check(&topology).1.contains("first interface"))
    );
    let mut data = serde_json::to_value(requirements.last().unwrap()).unwrap();
    data["first_hop"] = "missing".into();
    let invalid: Requirement = serde_json::from_value(data).unwrap();
    assert!(!invalid.valid_for(&topology));
}
