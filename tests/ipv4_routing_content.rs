use dlms_learning_lab::{
    engine::{self, Goal, State},
    ipv4_routing::Requirement,
    ipv4_routing_teaching::{self, Learning},
    ipv4_routing_workspace::Draft,
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "ipv4-static-foundations")
        .collect()
}
fn topology(s: &State) -> &dlms_learning_lab::ipv4_routing::Topology {
    let State::Ipv4Routing { topology } = s else {
        panic!()
    };
    topology
}
fn topology_mut(s: &mut State) -> &mut dlms_learning_lab::ipv4_routing::Topology {
    let State::Ipv4Routing { topology } = s else {
        panic!()
    };
    topology
}
fn actions(s: &mut State, commands: &[&str]) {
    for action in commands {
        engine::execute(s, action).unwrap();
    }
}
fn solved(lab: &pack::Scenario) -> State {
    let mut s = lab.initial.clone();
    for a in &lab.solution {
        engine::execute(&mut s, a).unwrap();
    }
    assert!(
        engine::solved(&s, &lab.goal),
        "{} {:?}",
        lab.id,
        engine::checks(&s, &lab.goal)
    );
    s
}
#[test]
fn all_six_distinct_tasks_accept_alternate_configuration_and_reject_plausible_wrong_repairs() {
    let alternatives: Vec<Vec<&str>> = vec![
        vec!["route R1 10.40.1.0/24 transit 10.0.0.2 37"],
        vec!["route R2 192.168.10.0/24 transit 10.0.0.1 17"],
        vec![
            "remove-route R1 1",
            "route R1 10.40.1.0/24 transit 10.0.0.2 210",
        ],
        vec!["route R1 10.40.1.20/32 backup 10.0.0.6 200"],
        vec![
            "remove-route R1 0",
            "remove-route R1 0",
            "route R1 10.40.1.0/24 backup 10.0.0.6 200",
            "route R1 10.40.1.0/24 transit 10.0.0.2 30",
        ],
        vec![
            "address Client2 lan 192.168.10.197/24",
            "forward R1 on",
            "remove-route R1 0",
            "remove-route R1 0",
            "route R1 10.40.1.0/24 backup 10.0.0.6 200",
            "route R1 10.40.1.0/24 transit 10.0.0.2 30",
            "remove-route R2 0",
            "route R2 192.168.10.0/24 transit 10.0.0.1 5",
        ],
    ];
    let mistakes: Vec<Vec<&str>> = vec![
        vec!["route R1 10.40.1.20/32 transit 10.0.0.2 1"],
        vec!["route R2 192.168.10.10/32 transit 10.0.0.1 1"],
        vec!["remove-route R1 1"], // Working default cannot replace the explicitly requested network row.
        vec![
            "remove-route R1 0",
            "route R1 10.40.1.0/24 backup 10.0.0.6 1",
        ],
        vec![
            "remove-route R1 0",
            "remove-route R1 0",
            "route R1 10.40.1.0/24 transit 10.0.0.2 1",
        ],
        vec!["address Client2 lan 192.168.10.22/24", "forward R1 on"],
    ];
    let labs = labs();
    assert_eq!(labs.len(), 6);
    for ((lab, alternate), wrong) in labs.iter().zip(alternatives).zip(mistakes) {
        assert!(!engine::solved(&lab.initial, &lab.goal));
        let mut s = lab.initial.clone();
        actions(&mut s, &alternate);
        assert!(
            engine::solved(&s, &lab.goal),
            "{} {:?}",
            lab.id,
            engine::checks(&s, &lab.goal)
        );
        actions(&mut s, &["forward R1 off"]);
        assert!(!engine::solved(&s, &lab.goal), "{} stale success", lab.id);
        let mut s = lab.initial.clone();
        actions(&mut s, &wrong);
        assert!(!engine::solved(&s, &lab.goal), "{} plausible wrong", lab.id);
        let result = engine::checks(&s, &lab.goal);
        assert!(result.iter().any(|r| r.0));
        assert!(result.iter().any(|r| !r.0));
    }
}
#[test]
fn every_declared_example_and_exact_suggested_experiment_runs_without_assignment_credit() {
    for (i, lab) in labs().iter().enumerate() {
        let lesson = lab.ipv4_routing_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let mut l = Learning::new(lesson);
        l.fill_example(lesson).unwrap();
        assert!(l.checks(lesson).is_empty());
        l.apply().unwrap();
        assert!(l.checks(lesson).iter().all(|r| r.0));
        match i {
            0 => {
                l.draft.devices.get_mut("R1").unwrap().routes[0].distance = "25".into();
                l.apply().unwrap();
                assert!(l.checks(lesson).iter().all(|r| r.0));
                l.draft.devices.get_mut("R1").unwrap().routes[0].next_hop = "172.22.1.20".into();
            }
            1 => {
                l.draft.devices.get_mut("R2").unwrap().routes[0].destination =
                    "172.20.10.10/32".into()
            }
            2 => {
                l.draft.devices.get_mut("R1").unwrap().routes[1].distance = "150".into();
                l.apply().unwrap();
                assert!(l.checks(lesson).iter().all(|r| r.0));
                let r = &mut l.draft.devices.get_mut("R1").unwrap().routes[1];
                r.interface = "backup".into();
                r.next_hop = "172.21.0.6".into();
                r.distance = "1".into();
            }
            3 => {
                l.draft.devices.get_mut("R1").unwrap().routes[1].destination =
                    "172.22.1.0/24".into()
            }
            4 => l.draft.devices.get_mut("R1").unwrap().routes[1].distance = "255".into(),
            5 => {
                l.draft
                    .devices
                    .get_mut("Client2")
                    .unwrap()
                    .interfaces
                    .get_mut("lan")
                    .unwrap()
                    .address = "172.20.10.77/24".into();
                l.apply().unwrap();
                assert!(l.checks(lesson).iter().all(|r| r.0));
                l.draft.devices.get_mut("R1").unwrap().forwarding = false;
            }
            _ => unreachable!(),
        }
        assert!(l.checks(lesson).is_empty());
        l.apply().unwrap();
        assert!(
            !l.checks(lesson).iter().all(|r| r.0),
            "{} experiment",
            lab.id
        );
        let mut a = Attempt::new(lab, 1);
        workspace::prepare(&mut a);
        ipv4_routing_teaching::prepare(&mut a, lesson, true);
        let assignment = a.state.clone();
        let draft = a.workspace.as_ref().unwrap().routing_v4_draft.clone();
        a.workspace.as_mut().unwrap().routing_v4_learning = Some(l);
        assert_eq!(a.state, assignment);
        assert_eq!(a.workspace.as_ref().unwrap().routing_v4_draft, draft);
        assert!(!progress::current_solved(&a, lab));
    }
}
#[test]
fn floating_backup_requires_the_actual_network_prefix_and_capstone_requires_primary_return_repair()
{
    let labs = labs();
    for lab in labs.iter().skip(4) {
        for replacement in ["0.0.0.0/0", "10.40.0.0/16", "10.40.1.20/32"] {
            let mut s = solved(lab);
            let backup = topology_mut(&mut s)
                .devices
                .get_mut("R1")
                .unwrap()
                .routes
                .iter_mut()
                .find(|r| r.interface == "backup")
                .unwrap();
            backup.destination =
                dlms_learning_lab::ipv4_routing::Prefix::parse(replacement).unwrap();
            assert!(
                !engine::solved(&s, &lab.goal),
                "{} backup {replacement}",
                lab.id
            );
        }
    }
    let lab = &labs[5];
    let mut s = lab.initial.clone();
    for action in lab.solution.iter().take(lab.solution.len() - 2) {
        engine::execute(&mut s, action).unwrap();
    }
    assert!(!engine::solved(&s, &lab.goal));
    assert!(
        engine::checks(&s, &lab.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("normally returns"))
    );
    let mut s = solved(lab);
    for target in ["192.168.10.10/32", "192.168.10.22/32"] {
        let r = &mut topology_mut(&mut s).devices.get_mut("R2").unwrap().routes[1];
        r.destination = dlms_learning_lab::ipv4_routing::Prefix::parse(target).unwrap();
        assert!(!engine::solved(&s, &lab.goal));
    }
}
#[test]
fn fixed_endpoints_cannot_be_moved_and_controlled_checks_do_not_repair_saved_state() {
    for lab in labs() {
        let solved = solved(&lab);
        let Goal::Ipv4Routing { requirements } = &lab.goal else {
            panic!()
        };
        let baseline = topology(&solved).clone();
        for r in requirements {
            let before = topology(&solved).clone();
            assert!(r.check(&before).0);
            assert_eq!(before, baseline);
        }
        let mut s = solved.clone();
        let target = topology_mut(&mut s)
            .devices
            .get_mut("Server")
            .unwrap()
            .interfaces
            .get_mut("lan")
            .unwrap();
        target.address = None;
        assert!(!engine::solved(&s, &lab.goal));
        let mut s = solved.clone();
        let old = topology(&s).devices["Server"].interfaces["lan"]
            .address
            .clone();
        topology_mut(&mut s)
            .devices
            .get_mut("Server")
            .unwrap()
            .interfaces
            .get_mut("lan")
            .unwrap()
            .address = None;
        topology_mut(&mut s)
            .devices
            .get_mut("R2")
            .unwrap()
            .interfaces
            .get_mut("branch")
            .unwrap()
            .address = old;
        assert!(!engine::solved(&s, &lab.goal));
        if lab.id.ends_with("floating-route") || lab.id.ends_with("branch-handover") {
            let mut s = solved;
            topology_mut(&mut s)
                .devices
                .get_mut("R1")
                .unwrap()
                .interfaces
                .get_mut("transit")
                .unwrap()
                .up = false;
            assert!(!engine::solved(&s, &lab.goal));
        }
    }
}
#[test]
fn current_unit_resumes_each_view_and_invalid_draft_while_original_58_attempts_remain_exact() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (i, lab) in all.iter().take(58).enumerate() {
        let mut a = Attempt::new(lab, i as u64);
        workspace::prepare(&mut a);
        p.attempts.insert(lab.id.clone(), a);
    }
    let old = serde_json::to_value(&p).unwrap();
    for (i, lab) in labs().iter().enumerate() {
        let mut a = Attempt::new(lab, 100 + i as u64);
        workspace::prepare(&mut a);
        ipv4_routing_teaching::prepare(&mut a, lab.ipv4_routing_teaching.as_ref().unwrap(), true);
        a.state = solved(lab);
        let w = a.workspace.as_mut().unwrap();
        w.routing_v4_draft = Some(Draft::new(topology(&a.state)));
        let learning = w.routing_v4_learning.as_mut().unwrap();
        learning.view = match i % 3 {
            0 => dlms_learning_lab::script_teaching::View::Introduction,
            1 => dlms_learning_lab::script_teaching::View::Example,
            _ => dlms_learning_lab::script_teaching::View::Assignment,
        };
        learning
            .fill_example(lab.ipv4_routing_teaching.as_ref().unwrap())
            .unwrap();
        learning.apply().unwrap();
        assert!(progress::current_solved(&a, lab));
        a.workspace
            .as_mut()
            .unwrap()
            .routing_v4_draft
            .as_mut()
            .unwrap()
            .devices
            .get_mut("R1")
            .unwrap()
            .routes[0]
            .next_hop = "unfinished".into();
        assert!(!progress::current_solved(&a, lab));
        p.attempts.insert(lab.id.clone(), a);
    }
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for lab in all.iter().take(58) {
        assert_eq!(
            serde_json::to_value(&loaded).unwrap()["attempts"][&lab.id],
            old["attempts"][&lab.id]
        );
    }
    let selected = &all[63];
    loaded
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 101));
    progress::save(&path, &loaded).unwrap();
    let after = progress::load(&path, &all).unwrap();
    for lab in all.iter().take(63) {
        assert_eq!(
            serde_json::to_value(&after.attempts[&lab.id]).unwrap(),
            serde_json::to_value(&p.attempts[&lab.id]).unwrap()
        );
    }
}
#[test]
fn conditional_static_requirements_reject_invalid_refs_duplicates_and_never_mutate_on_failure() {
    let lab = labs().remove(4);
    let solved = solved(&lab);
    let t = topology(&solved);
    let Goal::Ipv4Routing { requirements } = lab.goal else {
        panic!()
    };
    let conditional = requirements
        .iter()
        .find(|r| matches!(r,Requirement::StaticRouteFor{when,..} if !when.is_empty()))
        .unwrap();
    let before = t.clone();
    assert!(conditional.check(t).0);
    assert_eq!(*t, before);
    for mutation in [0, 1, 2] {
        let mut r = conditional.clone();
        if let Requirement::StaticRouteFor { when, .. } = &mut r {
            match mutation {
                0 => when[0].device = "unknown".into(),
                1 => when.push(when[0].clone()),
                _ => when[0].interface = "unknown".into(),
            }
        }
        assert!(!r.valid_for(t));
        assert!(!r.check(t).0);
        assert_eq!(*t, before);
    }
    // Equivalent route row order and unequal eligible distances remain legitimate.
    let mut draft = Draft::new(t);
    let routes = &mut draft.devices.get_mut("R1").unwrap().routes;
    routes.reverse();
    for r in routes {
        r.distance = if r.interface == "backup" { "150" } else { "25" }.into();
    }
    let mut t = t.clone();
    draft.apply(&mut t).unwrap();
    assert!(conditional.check(&t).0);
}
