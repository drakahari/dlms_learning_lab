use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    switching::{Fabric, Mode, Requirement},
    switching_teaching::{self, Learning},
    switching_workspace::{self, Draft},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "vlan-data-foundations")
        .collect()
}
fn state(lab: &pack::Scenario) -> State {
    let mut s = lab.initial.clone();
    for action in &lab.solution {
        engine::execute(&mut s, action).unwrap();
    }
    s
}
fn group(h: &str) -> &str {
    h.rsplit_once('-').unwrap().0
}
fn fabric(s: &State) -> &Fabric {
    let State::Switching { fabric } = s else {
        panic!()
    };
    fabric
}
fn apply(s: &mut State, action: &str) {
    engine::execute(s, action).unwrap();
}
#[test]
fn each_authored_policy_accepts_equivalent_editor_repairs_and_rejects_real_misconceptions() {
    let tasks = labs();
    assert_eq!(tasks.len(), 4);
    for (n, lab) in tasks.iter().enumerate() {
        assert!(!engine::solved(&lab.initial, &lab.goal));
        let good = state(lab);
        assert!(engine::solved(&good, &lab.goal));
        let mut alt = fabric(&good).clone();
        let mut d = Draft::new(&alt);
        for s in d.switches.values_mut() {
            s.vlans.push_str(",0050");
            for p in s.ports.values_mut() {
                if p.trunk {
                    p.allowed = if n == 3 {
                        "30,020,010".into()
                    } else {
                        "020,010".into()
                    };
                }
            }
        }
        d.apply(&mut alt).unwrap();
        assert!(
            engine::solved(&State::Switching { fabric: alt }, &lab.goal),
            "{} alternate",
            lab.id
        );
        let wrongs: Vec<Vec<&str>> = match n {
            0 => vec![
                vec!["access office staff-b 20"],
                vec!["port office staff-b down"],
                vec!["access office staff-a 20"],
            ],
            1 => vec![
                vec!["trunk west uplink 99 10,20,99"],
                vec!["trunk west uplink 99 10,20,99", "vlan east 20 present"],
                vec!["vlan east 20 present", "trunk east uplink 99 10,20,99"],
            ],
            2 => vec![
                vec!["trunk west uplink 99 10,20,99"],
                vec!["trunk east uplink 99 10,20,99"],
                vec!["port west uplink down"],
            ],
            _ => vec![
                vec!["access west operations 30"],
                vec!["vlan east 20 present", "port east guest up"],
                vec![
                    "trunk west uplink 99 10,20,30,99",
                    "trunk east uplink 99 10,20,30,99",
                ],
            ],
        };
        for actions in wrongs {
            let mut wrong = lab.initial.clone();
            for action in actions {
                apply(&mut wrong, action);
            }
            assert!(!engine::solved(&wrong, &lab.goal));
            assert!(!workspace::issues(&wrong, &lab.goal).is_empty());
        }
        // Every requested group really delivers across its pair; all inter-group pairs stay separate,
        // including local pairs that are implied by the explicit access/native constraints.
        let f = fabric(&good);
        let hosts: Vec<_> = f.hosts.keys().collect();
        for (i, a) in hosts.iter().enumerate() {
            for b in hosts.iter().skip(i + 1) {
                let v = f.visibility(a, b).unwrap();
                assert_eq!(v.mutual(), group(a) == group(b), "{} {a}/{b}", lab.id);
                assert_eq!(v.isolated(), group(a) != group(b));
            }
        }
    }
}
#[test]
fn every_missing_repair_remains_partial_even_if_other_repairs_are_complete() {
    for lab in labs() {
        for omit in 0..lab.solution.len() {
            let mut partial = lab.initial.clone();
            for (i, action) in lab.solution.iter().enumerate() {
                if i != omit {
                    apply(&mut partial, action);
                }
            }
            let checks = engine::checks(&partial, &lab.goal);
            assert!(checks.iter().any(|c| !c.0), "{} omitted{omit}", lab.id);
            assert!(checks.iter().any(|c| c.0), "{} no retained policy", lab.id);
        }
    }
}
#[test]
fn examples_teach_before_assessment_and_the_exact_suggested_changes_have_explainable_effects() {
    let changes = [
        "access office studio-b 40",
        "trunk east uplink 90 30,90",
        "trunk east uplink 40 30,40,90",
        "vlan east 70 absent",
    ];
    for (n, lab) in labs().iter().enumerate() {
        let lesson = lab.switching_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.concepts.len() >= 3);
        assert!(!lesson.prerequisite_note.is_empty());
        let mut l = Learning::new(lesson);
        assert!(l.checks(lesson).is_empty());
        l.fill_example(lesson).unwrap();
        l.apply().unwrap();
        assert!(l.checks(lesson).iter().all(|c| c.0));
        let good = l.fabric.clone();
        switching_workspace::adapter(&mut l.fabric, changes[n]).unwrap();
        l.draft = Draft::new(&l.fabric);
        assert!(l.checks(lesson).iter().any(|c| !c.0));
        if n == 0 {
            assert!(
                l.fabric
                    .visibility("visitor-a", "visitor-b")
                    .unwrap()
                    .mutual()
            );
            assert!(
                !l.fabric
                    .visibility("studio-a", "studio-b")
                    .unwrap()
                    .mutual()
            );
        }
        if n == 1 {
            assert!(l.fabric.switches["east"].vlans.contains(&40));
            assert!(
                l.fabric
                    .visibility("studio-west", "studio-east")
                    .unwrap()
                    .mutual()
            );
            assert!(
                !l.fabric
                    .visibility("visitor-west", "visitor-east")
                    .unwrap()
                    .mutual()
            );
        }
        if n == 2 {
            let v = l.fabric.visibility("visitor-west", "visitor-east").unwrap();
            assert!(v.forward && !v.reverse);
            assert!(!v.mutual());
            assert!(!v.isolated());
        }
        if n == 3 {
            assert!(
                l.fabric
                    .visibility("design-west", "design-east")
                    .unwrap()
                    .mutual()
            );
            assert!(
                l.fabric
                    .visibility("visitor-west", "visitor-east")
                    .unwrap()
                    .mutual()
            );
            assert!(
                !l.fabric
                    .visibility("support-west", "support-east")
                    .unwrap()
                    .mutual()
            );
        }
        assert_ne!(l.fabric, good);
        assert!(lab.sources.iter().any(|s| s.contains("cisco.com")));
        assert!(lab.simplifications.contains("STP/PVST"));
        assert_eq!(
            lab.prerequisites,
            if n == 0 {
                vec![]
            } else {
                vec![labs()[n - 1].id.clone()]
            }
        );
    }
}
#[test]
fn each_individual_policy_can_be_broken_and_never_retains_current_success() {
    for lab in labs() {
        let good = state(&lab);
        let Goal::Switching { requirements } = &lab.goal else {
            panic!()
        };
        for req in requirements {
            let mut f = fabric(&good).clone();
            match req {
                Requirement::Visibility {
                    from, to, visible, ..
                } => {
                    if *visible {
                        let host = f.hosts.get_mut(from).unwrap();
                        host.up = false;
                    } else {
                        // Join the two endpoint groups on one switch, or use native translation across the trunk.
                        let local = |host: &str| {
                            f.cables
                                .iter()
                                .find_map(|c| match (&c.a, &c.b) {
                                    (
                                        dlms_learning_lab::switching::Jack::Host { host: h },
                                        dlms_learning_lab::switching::Jack::Switch { switch, port },
                                    ) if h == host => Some((switch.clone(), port.clone())),
                                    _ => None,
                                })
                                .unwrap()
                        };
                        let (s, p) = local(from);
                        let (ts, tp) = local(to);
                        let Mode::Access { vlan } = f.switches[&ts].ports[&tp].mode else {
                            panic!()
                        };
                        f.switches
                            .get_mut(&s)
                            .unwrap()
                            .ports
                            .get_mut(&p)
                            .unwrap()
                            .mode = Mode::Access { vlan };
                    }
                }
                Requirement::AccessVlan { switch, port, .. } => {
                    f.switches
                        .get_mut(switch)
                        .unwrap()
                        .ports
                        .get_mut(port)
                        .unwrap()
                        .up = false
                }
                Requirement::NativeVlan { switch, port, .. } => {
                    let p = f
                        .switches
                        .get_mut(switch)
                        .unwrap()
                        .ports
                        .get_mut(port)
                        .unwrap();
                    let Mode::Trunk { native, .. } = &mut p.mode else {
                        panic!()
                    };
                    *native = 1;
                }
                Requirement::ActiveVlan { switch, vlan } => {
                    f.switches.get_mut(switch).unwrap().vlans.remove(vlan);
                }
            }
            assert!(!req.check(&f).0, "{} {req:?}", lab.id);
            assert!(!engine::solved(&State::Switching { fabric: f }, &lab.goal));
        }
    }
}
#[test]
fn all_four_assignments_keep_example_drafts_separate_on_resume_and_only_reset_selected_work() {
    let tasks = labs();
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    // Preserve an original saved attempt alongside all new work.
    p.attempts
        .insert(all[0].id.clone(), Attempt::new(&all[0], 1));
    let old = serde_json::to_value(&p.attempts[&all[0].id]).unwrap();
    for lab in &tasks {
        let mut a = Attempt::new(lab, 2);
        workspace::prepare(&mut a);
        let lesson = lab.switching_teaching.as_ref().unwrap();
        switching_teaching::prepare(&mut a, lesson, true);
        let untouched = a.state.clone();
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .switching_learning
            .as_mut()
            .unwrap();
        l.fill_example(lesson).unwrap();
        l.apply().unwrap();
        assert_eq!(a.state, untouched);
        assert!(!progress::current_solved(&a, lab));
        a.state = state(lab);
        a.workspace.as_mut().unwrap().switching_draft = Some(Draft::new(fabric(&a.state)));
        assert!(progress::current_solved(&a, lab));
        p.attempts.insert(lab.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut q = progress::load(&path, &all).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(serde_json::to_value(&q.attempts[&all[0].id]).unwrap(), old);
    for lab in &tasks {
        let a = &q.attempts[&lab.id];
        assert!(progress::current_solved(a, lab));
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .switching_learning
                .as_ref()
                .unwrap()
                .checks(lab.switching_teaching.as_ref().unwrap())
                .iter()
                .all(|c| c.0)
        );
    }
    q.attempts
        .insert(tasks[1].id.clone(), Attempt::new(&tasks[1], 3));
    assert!(!progress::current_solved(
        &q.attempts[&tasks[1].id],
        &tasks[1]
    ));
    for lab in [&tasks[0], &tasks[2], &tasks[3]] {
        assert!(progress::current_solved(&q.attempts[&lab.id], lab));
    }
    assert_eq!(serde_json::to_value(&q.attempts[&all[0].id]).unwrap(), old);
}
