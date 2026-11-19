//! Authored caller-policy tasks must require actual simulated operations.
use dlms_learning_lab::{
    engine::{self, State},
    pack, user_mounts_workspace as w,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/removable-mount-lessons-v1.json"),
        include_str!("../scenarios/user-mount-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "linux-removable-callers")
    .collect()
}
fn session(s: &pack::Scenario) -> w::Session {
    let State::UserMounts { simulation } = &s.initial else {
        panic!()
    };
    (**simulation).clone()
}
fn check(t: &w::Task, x: &mut w::Work) -> bool {
    let r = w::run(t, x).unwrap();
    assert!(x.store(r, t));
    x.solved(t)
}
#[test]
fn authored_references_examples_and_alternates_reach_real_state() {
    let labs = labs();
    assert_eq!(labs.len(), 2);
    for s in labs {
        let mut x = session(&s);
        let untouched = x.assignment.clone();
        for a in &x.definition.example_actions {
            x.example.apply(&x.definition.example, a.clone()).unwrap();
        }
        assert!(
            check(&x.definition.example, &mut x.example),
            "{} example",
            s.id
        );
        assert_eq!(untouched, x.assignment);
        assert!(!engine::solved(
            &State::UserMounts {
                simulation: Box::new(x.clone())
            },
            &s.goal
        ));
        let mut via_engine = State::UserMounts {
            simulation: Box::new(x.clone()),
        };
        for cmd in &s.solution {
            engine::execute_for_goal(&mut via_engine, &s.goal, cmd).unwrap();
        }
        let State::UserMounts {
            simulation: mut executed,
        } = via_engine
        else {
            unreachable!()
        };
        assert!(check(
            &executed.definition.assignment,
            &mut executed.assignment
        ));
        assert!(engine::solved(
            &State::UserMounts {
                simulation: executed
            },
            &s.goal
        ));
        for cmd in &s.solution {
            let a: w::Action = serde_json::from_str(cmd.strip_prefix(w::PREFIX).unwrap()).unwrap();
            x.assignment.apply(&x.definition.assignment, a).unwrap();
        }
        assert!(
            check(&x.definition.assignment, &mut x.assignment),
            "{} reference: {:?}",
            s.id,
            x.assignment.result
        );
        let changed = x
            .assignment
            .draft
            .replace("ro,noauto,", "noauto,ro,nofail,");
        x.assignment
            .apply(
                &x.definition.assignment,
                w::Action::Save {
                    configuration: changed,
                },
            )
            .unwrap();
        assert!(!x.assignment.solved(&x.definition.assignment));
        assert!(
            check(&x.definition.assignment, &mut x.assignment),
            "{} valid alternate",
            s.id
        );
    }
}
#[test]
fn saved_answers_alone_and_wrong_options_do_not_pass() {
    for s in labs() {
        let x = session(&s);
        let t = &x.definition.assignment;
        let mut a = x.assignment.clone();
        assert!(!check(t, &mut a));
        let save: w::Action =
            serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap();
        a.apply(t, save).unwrap();
        assert!(
            !check(t, &mut a),
            "saving cannot invent actual mounting/handover"
        );
        for cmd in &s.solution[1..] {
            a.apply(
                t,
                serde_json::from_str(cmd.strip_prefix(w::PREFIX).unwrap()).unwrap(),
            )
            .unwrap();
        }
        assert!(check(t, &mut a));
        for wrong in [
            a.draft.replace("ro,noauto,", "rw,noauto,"),
            a.draft.replace("noauto,", "auto,"),
            a.draft.replace("user 0", "users 0"),
            a.draft.replace("users 0", "user 0"),
        ] {
            if wrong == a.draft {
                continue;
            }
            let mut b = a.clone();
            b.apply(
                t,
                w::Action::Save {
                    configuration: wrong,
                },
            )
            .unwrap();
            assert!(!check(t, &mut b), "wrong policy passed: {}", b.draft);
        }
        let mut b = a.clone();
        b.edit(format!("{}# note\n", b.draft)).unwrap();
        assert!(!b.solved(t));
        assert!(!check(t, &mut b), "unsaved edit must remain visible");
        let raw = serde_json::to_vec(&w::Session {
            assignment: a,
            ..x.clone()
        })
        .unwrap();
        let mut loaded = w::Session::decode(&raw, &x.definition).unwrap();
        assert!(!loaded.assignment.solved(t));
        assert!(loaded.assignment.needs_verification(t));
        assert!(check(t, &mut loaded.assignment));
        assert!(!w::Work::new(t).solved(t));
    }
}

#[test]
fn preceding_catalog_and_course_remain_exact_with_new_unit_after_its_prerequisite() {
    use dlms_learning_lab::course;
    let old = pack::accepted259().unwrap();
    let now = pack::accepted261().unwrap();
    assert_eq!((old.len(), now.len()), (259, 261));
    for s in &old {
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(now.iter().find(|n| n.id == s.id).unwrap()).unwrap(),
            "{} changed",
            s.id
        );
    }
    let prior = course::accepted259(&old).unwrap();
    let units = course::accepted261(&now).unwrap();
    assert_eq!((prior.len(), units.len()), (75, 76));
    let retained = units
        .iter()
        .filter(|u| u.id != "linux-removable-callers")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(prior).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-removable-callers")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-removable-startup");
}
#[test]
fn source_bound_progress_rejects_foreign_state_and_keeps_independent_drafts() {
    use dlms_learning_lab::{
        progress::{self, Attempt, Progress},
        workspace,
    };
    let labs = labs();
    let s = &labs[0];
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    let State::UserMounts { simulation } = &mut a.state else {
        panic!()
    };
    for cmd in &s.solution {
        simulation
            .assignment
            .apply(
                &simulation.definition.assignment,
                serde_json::from_str(cmd.strip_prefix(w::PREFIX).unwrap()).unwrap(),
            )
            .unwrap();
    }
    assert!(check(
        &simulation.definition.assignment,
        &mut simulation.assignment
    ));
    simulation
        .example
        .edit("# unfinished example\n".into())
        .unwrap();
    assert!(progress::current_solved(&a, s));
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private-progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let loaded = progress::load(&path, &labs).unwrap();
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    let State::UserMounts { simulation } = &loaded.attempts[&s.id].state else {
        panic!()
    };
    assert_eq!(simulation.example.draft, "# unfinished example\n");
    assert!(
        simulation
            .assignment
            .needs_verification(&simulation.definition.assignment)
    );
    let mut foreign = a;
    let State::UserMounts { simulation } = &mut foreign.state else {
        panic!()
    };
    simulation.definition.assignment.first_actor = "Impostor".into();
    assert!(!progress::current_solved(&foreign, s));
    assert!(w::context(&foreign, s).is_err());
}
#[test]
fn feedback_distinguishes_absent_mount_wrong_mounter_and_old_applied_options() {
    let labs = labs();
    let first = session(&labs[0]);
    let r = w::run(&first.definition.assignment, &first.assignment).unwrap();
    assert!(
        r.current
            .iter()
            .any(|r| !r.0 && r.1.contains(": not mounted."))
    );
    let mut x = session(&labs[1]);
    let t = &x.definition.assignment;
    let r = w::run(t, &x.assignment).unwrap();
    assert!(
        r.current
            .iter()
            .any(|r| !r.0 && r.1.contains("actual mounter is different"))
    );
    let save: w::Action =
        serde_json::from_str(labs[1].solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap();
    let w::Action::Save { configuration } = save else {
        panic!()
    };
    x.assignment
        .apply(
            t,
            w::Action::Save {
                configuration: configuration.replace("users 0", "users,exec 0"),
            },
        )
        .unwrap();
    for cmd in &labs[1].solution[1..] {
        x.assignment
            .apply(
                t,
                serde_json::from_str(cmd.strip_prefix(w::PREFIX).unwrap()).unwrap(),
            )
            .unwrap();
    }
    let r = w::run(t, &x.assignment).unwrap();
    assert!(
        r.current
            .iter()
            .any(|r| !r.0 && r.1.contains("applied options differ"))
    );
    assert!(
        r.current
            .iter()
            .any(|r| !r.0 && r.1.contains("Later options can override"))
    );
}
