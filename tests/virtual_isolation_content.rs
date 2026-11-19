use dlms_learning_lab::{
    engine::{self, State},
    pack,
    switching::{Mode, Visibility},
    virtual_isolation_source as binding, virtual_isolation_workspace as work,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!(
        "../scenarios/virtual-isolation-lessons-v1.json"
    ))
    .unwrap()
}
#[test]
fn all_three_authored_assignments_are_reachable_and_initially_incomplete() {
    let labs = labs();
    assert_eq!(labs.len(), 3);
    for s in labs {
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for action in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        match (&mut state, &s.goal) {
            (
                State::VirtualIsolation { simulation },
                engine::Goal::VirtualIsolation { definition },
            ) => {
                simulation
                    .assignment
                    .edit(&definition.assignment, definition.assignment.initial())
                    .unwrap();
                assert!(!engine::solved(&state, &s.goal));
            }
            (State::Switching { fabric }, _) => {
                fabric
                    .switches
                    .get_mut("south-vswitch")
                    .unwrap()
                    .ports
                    .get_mut("uplink")
                    .unwrap()
                    .up = false;
                assert!(!engine::solved(&state, &s.goal));
            }
            _ => panic!(),
        }
    }
}
#[test]
fn taught_guide_methods_work_without_assignment_credit() {
    for s in labs() {
        if let State::VirtualIsolation { simulation } = s.initial.clone() {
            let d = binding::source(&s).unwrap();
            let mut session = *simulation;
            let mut config = d.example.initial();
            match &mut config {
                work::Draft::Workloads(c) => {
                    for p in &mut c.placements {
                        p.host = match p.workload.as_str() {
                            "archive-a" | "status-a" => "alpha",
                            "archive-b" => "spare",
                            "status-b" => "beta",
                            _ => panic!(),
                        }
                        .into();
                    }
                }
                work::Draft::Routing(c) => {
                    for b in &mut c.bindings {
                        b.context = if b.interface.starts_with("studio-") {
                            "cedar"
                        } else {
                            "birch"
                        }
                        .into();
                    }
                }
            }
            session.practice.edit(&d.example, config).unwrap();
            session.practice.run(&d.example).unwrap();
            session.practice.check(&d.example).unwrap();
            assert!(session.practice.solved(&d.example));
            assert!(!session.assignment.solved(&d.assignment));
        } else {
            let lesson = s.switching_teaching.as_ref().unwrap();
            let mut learning = dlms_learning_lab::switching_teaching::Learning::new(lesson);
            learning.fill_example(lesson).unwrap();
            assert!(learning.checks(lesson).iter().all(|c| c.0));
            assert!(!engine::solved(&s.initial, &s.goal));
        }
    }
}
#[test]
fn legitimate_noncanonical_placements_context_labels_and_unused_native_lists_pass() {
    for s in labs() {
        let mut state = s.initial.clone();
        for action in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
        }
        match (&mut state, &s.goal) {
            (
                State::VirtualIsolation { simulation },
                engine::Goal::VirtualIsolation { definition },
            ) => {
                let mut config = simulation.assignment.configuration.clone();
                match &mut config {
                    work::Draft::Workloads(c) => {
                        for p in &mut c.placements {
                            p.host = match p.workload.as_str() {
                                "ledger-a" | "portal-b" => "south",
                                "ledger-b" | "portal-a" => "north",
                                _ => panic!(),
                            }
                            .into();
                        }
                    }
                    work::Draft::Routing(c) => {
                        for b in &mut c.bindings {
                            b.context = if b.context == "blue" { "green" } else { "blue" }.into();
                        }
                    }
                }
                simulation
                    .assignment
                    .edit(&definition.assignment, config)
                    .unwrap();
                simulation.assignment.check(&definition.assignment).unwrap();
            }
            (State::Switching { fabric }, _) => {
                for sw in fabric.switches.values_mut() {
                    for p in sw.ports.values_mut() {
                        if let Mode::Trunk { allowed, .. } = &mut p.mode {
                            allowed.remove(&99);
                        }
                    }
                }
                fabric.validate().unwrap();
            }
            _ => panic!(),
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
    }
}
#[test]
fn partial_shortcuts_do_not_pass_or_merely_hide_investigation() {
    let labs = labs();
    for s in &labs {
        let mut state = s.initial.clone();
        engine::execute_for_goal(&mut state, &s.goal, &s.solution[0]).unwrap();
        if matches!(state, State::VirtualIsolation { .. }) {
            assert!(!engine::solved(&state, &s.goal));
        }
    }
    let s = &labs[1];
    let mut state = s.initial.clone();
    engine::execute_for_goal(&mut state, &s.goal, "access south-vswitch accounting 10").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    let State::Switching { fabric } = &mut state else {
        panic!()
    };
    assert_eq!(
        fabric.visibility("visitor-north", "visitor-south").unwrap(),
        Visibility {
            forward: true,
            reverse: true
        }
    );
    for sw in fabric.switches.values_mut() {
        for p in sw.ports.values_mut() {
            if let Mode::Access { vlan } = &mut p.mode {
                *vlan = 10;
            }
        }
    }
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn assignment_does_not_embed_reference_config_and_teaching_has_real_actions() {
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/ccna.json")).unwrap();
    for s in labs() {
        let e = entries.iter().find(|e| e["id"] == s.id).unwrap();
        assert_eq!(e["concepts"].as_array().unwrap().len(), 3);
        assert_eq!(e["practice"].as_array().unwrap().len(), 3);
        assert_eq!(e["worked"].as_array().unwrap().len(), 3);
        assert!(!e["requirements"].as_array().unwrap().is_empty());
        for p in e["practice"].as_array().unwrap() {
            assert!(
                p["action"].as_str().unwrap().len() > 40
                    && p["observe"].as_str().unwrap().len() > 40
            );
        }
        assert!(!s.objective.contains("virtual-net"));
        if let State::VirtualIsolation { simulation } = &s.initial {
            assert!(
                !simulation.assignment.checked
                    && simulation.assignment.actual.is_none()
                    && simulation.practice.actual.is_none()
            );
            let d = binding::source(&s).unwrap();
            assert_ne!(
                simulation.assignment.configuration,
                d.assignment.reference()
            );
            assert_ne!(simulation.practice.configuration, d.example.reference());
        }
    }
}
#[test]
fn integration_preserves_every458_old_scenario182_unit_and_order() {
    use dlms_learning_lab::course;
    let before = pack::accepted458().unwrap();
    let now = pack::accepted461().unwrap();
    assert_eq!((before.len(), now.len()), (458, 461));
    let ids = labs()
        .iter()
        .map(|s| s.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        serde_json::to_value(
            now.iter()
                .filter(|s| !ids.contains(&s.id))
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let previous_units = course::accepted458(&before).unwrap();
    let units = course::accepted461(&now).unwrap();
    assert_eq!((previous_units.len(), units.len()), (182, 183));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ccna-virtual-isolation")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&previous_units).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ccna-virtual-isolation")
        .unwrap();
    assert_eq!(units[at - 1].id, "ccna-leaf-spine-fabrics");
    assert_eq!(now.iter().filter(|s| s.track == "CCNA").count(), 113);
    let current: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/ccna.json")).unwrap();
    assert_eq!(current.len(), 113);
}
