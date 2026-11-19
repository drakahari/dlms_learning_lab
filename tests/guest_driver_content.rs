use dlms_learning_lab::{
    engine::{self, State},
    guest_driver_source as binding, guest_driver_workspace as work, pack, rpm_packages as rpm,
};
fn authored() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/guest-driver-lessons-v1.json")).unwrap()
}
fn encode(a: &work::Action) -> String {
    format!("{}{}", binding::PREFIX, serde_json::to_string(a).unwrap())
}
fn scope(s: &mut State) -> &mut work::Session {
    let State::GuestDrivers { simulation } = s else {
        panic!()
    };
    simulation
}
fn action(s: &mut State, g: &engine::Goal, a: work::Action) {
    engine::execute_for_goal(s, g, &encode(&a)).unwrap();
}
fn run(s: &mut State, g: &engine::Goal, input: &str) {
    action(
        s,
        g,
        work::Action::Run {
            input: input.into(),
        },
    );
}
fn package(s: &mut State, g: &engine::Goal, action_: rpm::Action) {
    action(s, g, work::Action::Package { action: action_ });
}

#[test]
fn all_three_authored_tasks_and_their_different_worked_images_reach_every_outcome() {
    let labs = authored();
    assert_eq!(labs.len(), 3);
    for s in &labs {
        let d = binding::source(s).unwrap();
        d.validate_reachability().unwrap();
        assert_ne!(d.assignment.facts, d.example.facts);
        assert_ne!(d.assignment.initial, d.example.initial);
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for step in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, step).unwrap();
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let mut session = work::Session::new(d).unwrap();
        let before = session.assignment.clone();
        for step in &d.example_steps {
            session.action(d, step, true).unwrap();
        }
        assert!(
            d.example
                .checks(&session.practice.runtime)
                .iter()
                .all(|r| r.passed)
        );
        assert_eq!(session.assignment, before);
        assert!(!d.current(&session));
        assert_eq!(s.guide.len(), 3);
        assert!(
            s.guide
                .iter()
                .all(|g| !g.why.is_empty() && !g.action.is_empty() && !g.commands.is_empty())
        );
        assert!(
            s.guide
                .iter()
                .flat_map(|g| g.commands.iter())
                .all(|c| !c.simulator_only && !c.command.starts_with(binding::PREFIX))
        );
        assert!(
            s.simplifications.contains("not actual RPM")
                && s.simplifications.contains("not real startup")
        );
    }
}
#[test]
fn plausible_partial_and_current_only_repairs_fail_and_saved_flags_cannot_hide_broken_state() {
    for (case, s) in authored().into_iter().enumerate() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        if case == 0 {
            // Removing the incompatible record and loading a valid OTHER driver
            // does not activate the actual presented network controller.
            package(
                &mut state,
                &s.goal,
                rpm::Action::EraseSet {
                    packages: vec![
                        d.assignment.facts.packages.archives["guest-support-old"]
                            .identity
                            .clone(),
                    ],
                },
            );
            package(
                &mut state,
                &s.goal,
                rpm::Action::InstallSet {
                    archives: vec!["guest-emulated-support".into()],
                },
            );
            run(&mut state, &s.goal, "depmod -a");
            run(&mut state, &s.goal, "modprobe guest_emulated");
        } else if case == 1 {
            run(&mut state, &s.goal, "modprobe guest_net");
            assert!(
                scope(&mut state)
                    .assignment
                    .runtime
                    .modules
                    .loaded
                    .contains_key("guest_net")
            );
            package(
                &mut state,
                &s.goal,
                rpm::Action::ReinstallExact {
                    archive: "guest-support-new".into(),
                },
            );
            assert!(
                !scope(&mut state)
                    .assignment
                    .runtime
                    .modules
                    .index
                    .contains("guest_net")
            );
        } else {
            package(
                &mut state,
                &s.goal,
                rpm::Action::UpgradeSet {
                    archives: vec!["guest-support-new".into()],
                },
            );
            run(&mut state, &s.goal, "depmod -a");
            assert!(
                scope(&mut state)
                    .assignment
                    .runtime
                    .fresh_request(&d.assignment.facts, "guest_net")
                    .unwrap()
                    .status
                    == 0
            );
            assert!(
                scope(&mut state)
                    .assignment
                    .runtime
                    .fresh_request(&d.assignment.facts, "volume_io")
                    .unwrap()
                    .status
                    != 0
            );
        }
        action(&mut state, &s.goal, work::Action::Check);
        assert!(!engine::solved(&state, &s.goal), "{}", s.id);
        let rows = d.assignment.checks(&scope(&mut state).assignment.runtime);
        assert!(rows.iter().any(|r| !r.passed && !r.detail.is_empty()));
        scope(&mut state).assignment.checked = true;
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn supported_noncanonical_package_and_module_repairs_pass_by_result_not_history() {
    for (case, s) in authored().into_iter().enumerate() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        if case != 1 {
            package(
                &mut state,
                &s.goal,
                rpm::Action::EraseSet {
                    packages: vec![
                        d.assignment.facts.packages.archives["guest-support-old"]
                            .identity
                            .clone(),
                    ],
                },
            );
            if case == 2 {
                package(
                    &mut state,
                    &s.goal,
                    rpm::Action::ReinstallExact {
                        archive: "kernel-core".into(),
                    },
                );
            }
            package(
                &mut state,
                &s.goal,
                rpm::Action::InstallSet {
                    archives: vec!["guest-support-new".into()],
                },
            );
        } else {
            package(
                &mut state,
                &s.goal,
                rpm::Action::ReinstallExact {
                    archive: "guest-support-new".into(),
                },
            );
            run(&mut state, &s.goal, "rmmod guest_net");
        }
        run(&mut state, &s.goal, "depmod -a");
        if case == 0 {
            let path = &d.assignment.facts.modules.modules["guest_bus"].path;
            run(&mut state, &s.goal, &format!("insmod {path}"));
        }
        if case != 2 {
            let path = &d.assignment.facts.modules.modules["guest_net"].path;
            run(&mut state, &s.goal, &format!("insmod {path}"));
        }
        action(&mut state, &s.goal, work::Action::Check);
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        scope(&mut state)
            .assignment
            .runtime
            .remove_object(&d.assignment.facts, "guest_net")
            .unwrap();
        assert!(
            !engine::solved(&state, &s.goal),
            "A stored pass cannot survive missing support"
        );
    }
}
#[test]
fn introductory_recovery_and_capstone_are_distinct_operational_faults_not_number_variants() {
    let labs = authored();
    let tasks: Vec<_> = labs
        .iter()
        .map(|s| &binding::source(s).unwrap().assignment)
        .collect();
    assert_eq!(tasks[0].devices.len(), 1);
    assert_eq!(tasks[1].devices.len(), 1);
    assert_eq!(tasks[2].devices.len(), 2);
    assert!(!tasks[0].initial.modules.loaded.contains_key("guest_net"));
    assert!(tasks[1].initial.modules.loaded.contains_key("guest_net"));
    assert!(
        !tasks[1]
            .initial
            .present(&tasks[1].facts)
            .unwrap()
            .contains("guest_net")
    );
    assert!(tasks[2].initial.modules.loaded.contains_key("volume_io"));
    assert!(
        !tasks[2]
            .initial
            .present(&tasks[2].facts)
            .unwrap()
            .contains("volume_io")
    );
    assert_eq!(
        tasks[0].facts.modules.kernel_release,
        tasks[1].facts.modules.kernel_release
    );
    assert_eq!(
        tasks[1].facts.modules.kernel_release,
        tasks[2].facts.modules.kernel_release
    );
}

#[test]
fn retained464_integration_keeps_every461_scenario183_unit_and_prior_relative_order() {
    use dlms_learning_lab::course;
    let before = pack::accepted461().unwrap();
    let now = pack::accepted464().unwrap();
    assert_eq!((before.len(), now.len()), (461, 464));
    let ids: std::collections::BTreeSet<_> = authored().into_iter().map(|s| s.id).collect();
    assert_eq!(
        serde_json::to_value(
            now.iter()
                .filter(|s| !ids.contains(&s.id))
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let old = course::accepted461(&before).unwrap();
    let units = course::accepted464(&now).unwrap();
    assert_eq!((old.len(), units.len()), (183, 184));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-guest-image-support")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-guest-image-support")
        .unwrap();
    for prerequisite in &units[at].prerequisites {
        assert!(units.iter().position(|u| u.id == *prerequisite).unwrap() < at);
    }
    assert_eq!(now.iter().filter(|s| s.track == "LPIC-1").count(), 154);
    let prior: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v44.json")).unwrap();
    let consolidated: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v45.json")).unwrap();
    assert_eq!(prior["schema_version"], consolidated["schema_version"]);
    let prior_rows = prior["scenarios"].as_array().unwrap();
    let rows = consolidated["scenarios"].as_array().unwrap();
    assert_eq!((prior_rows.len(), rows.len()), (88, 91));
    assert_eq!(&rows[..88], prior_rows);
    assert_eq!(
        rows[88..]
            .iter()
            .map(|s| s["id"].as_str().unwrap().to_owned())
            .collect::<std::collections::BTreeSet<_>>(),
        ids
    );
}
