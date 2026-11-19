//! Authored RAID1 learning, grading and progress; no real storage operation.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    raid1 as core, raid1_workspace as work, workspace,
};
use std::sync::OnceLock;
fn labs() -> &'static Vec<pack::Scenario> {
    static LABS: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    LABS.get_or_init(|| pack::load(include_str!("../scenarios/raid1-lessons-v1.json")).unwrap())
}
fn d(s: &pack::Scenario) -> &work::Definition {
    let Goal::Raid1 { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &mut State) -> &mut work::Session {
    let State::Raid1 { simulation } = s else {
        panic!()
    };
    simulation
}
#[test]
fn every_taught_method_and_different_example_reaches_meaningful_realized_outcomes() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        d(s).validate().unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for c in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        let mut practice = work::Session::new(d(s)).unwrap();
        for step in &d(s).lesson.worked {
            if let Some(c) = &step.command {
                assert_eq!(practice.execute(d(s), c, true).unwrap().status, 0);
            } else {
                practice
                    .action(d(s), step.action.as_ref().unwrap(), true)
                    .unwrap();
            }
        }
        assert!(practice.example_checks(d(s)).iter().all(|r| r.0));
        assert!(!d(s).checks(&practice).iter().all(|r| r.0));
    }
}
#[test]
fn alternate_operation_order_and_eligible_media_are_accepted_without_exact_command_matching() {
    let s = &labs()[0];
    let mut state = s.initial.clone();
    engine::execute_for_goal(
        &mut state,
        &s.goal,
        "mdadm --manage /dev/md10 --add /dev/vdc1",
    )
    .unwrap();
    engine::execute_for_goal(&mut state, &s.goal, s.solution.last().unwrap()).unwrap();
    assert!(
        !engine::solved(&state, &s.goal),
        "still-attached failed member is not retired"
    );
    engine::execute_for_goal(&mut state, &s.goal, "mdadm /dev/md10 --remove /dev/vdb1").unwrap();
    assert!(engine::solved(&state, &s.goal));
    let s = &labs()[1];
    for device in ["/dev/sdc1", "/dev/sdf1"] {
        let mut state = s.initial.clone();
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            &format!("mdadm /dev/md12 --add {device}"),
        )
        .unwrap();
        engine::execute_for_goal(&mut state, &s.goal, s.solution.last().unwrap()).unwrap();
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn wrong_candidate_partial_recovery_and_later_failure_cannot_claim_completion() {
    for s in labs() {
        let target = &d(s).plan.array;
        let mut state = s.initial.clone();
        let small = if s.id.ends_with("replace") {
            "/dev/vdd1"
        } else {
            "/dev/sdd1"
        };
        let old = state.clone();
        let err = engine::execute_for_goal(
            &mut state,
            &s.goal,
            &format!("mdadm {target} --add {small}"),
        )
        .unwrap_err();
        assert!(err.contains("usable MD capacity"));
        assert_eq!(
            session(&mut state).current,
            session(&mut old.clone()).current
        );
        for c in s.solution.iter().take(s.solution.len() - 1) {
            engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
        }
        assert!(!engine::solved(&state, &s.goal));
        session(&mut state)
            .action(
                d(s),
                &core::Action::Advance {
                    array: target.clone(),
                    units: 1,
                },
                false,
            )
            .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        engine::execute_for_goal(&mut state, &s.goal, s.solution.last().unwrap()).unwrap();
        assert!(engine::solved(&state, &s.goal));
        let replacement = if s.id.ends_with("replace") {
            "/dev/vdc1"
        } else {
            "/dev/sdc1"
        };
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            &format!("mdadm {target} --fail {replacement}"),
        )
        .unwrap();
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn partial_copy_and_drafts_save_resume_retry_and_selected_reset_preserve_other_progress() {
    let catalog = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    for (n, s) in catalog.iter().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        workspace::prepare(&mut a);
        p.attempts.insert(s.id.clone(), a);
    }
    let s = &catalog[0];
    let a = p.attempts.get_mut(&s.id).unwrap();
    for c in s.solution.iter().take(s.solution.len() - 1) {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    let sim = session(&mut a.state);
    sim.action(
        d(s),
        &core::Action::Advance {
            array: d(s).plan.array.clone(),
            units: 3,
        },
        false,
    )
    .unwrap();
    sim.set_command(d(s), "mdadm --detail /dev/md10".into(), false)
        .unwrap();
    for step in &d(s).lesson.worked {
        if let Some(c) = &step.command {
            sim.execute(d(s), c, true).unwrap();
        } else {
            sim.action(d(s), step.action.as_ref().unwrap(), true)
                .unwrap();
        }
    }
    assert!(sim.example_checks(d(s)).iter().all(|r| r.0));
    assert!(!progress::current_solved(a, s));
    p.favorites.insert(catalog[1].id.clone());
    let untouched = serde_json::to_value(&p.attempts[&catalog[1].id]).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, catalog).unwrap();
    let mut loaded = progress::load(&path, catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let a = loaded.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, s));
    let sim = session(&mut a.state);
    assert_eq!(
        sim.current.health(&d(s).plan.array).unwrap(),
        core::Health::Rebuilding {
            copied: 3,
            total: 8
        }
    );
    assert_eq!(sim.command, "mdadm --detail /dev/md10");
    engine::execute_for_goal(&mut a.state, &s.goal, s.solution.last().unwrap()).unwrap();
    assert!(progress::current_solved(a, s));
    progress::save_with_catalog(&path, &loaded, catalog).unwrap();
    let mut loaded = progress::load(&path, catalog).unwrap();
    let a = loaded.attempts.get_mut(&s.id).unwrap();
    assert!(progress::current_solved(a, s));
    let visited = a.visited;
    *a = Attempt::new(s, visited);
    workspace::prepare(a);
    assert!(!progress::current_solved(a, s));
    assert_eq!(a.state, s.initial);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&catalog[1].id]).unwrap(),
        untouched
    );
    assert!(loaded.favorites.contains(&catalog[1].id));
}
#[test]
fn source_authority_rejects_foreign_progress_and_plain_unsupported_commands_without_fake_success() {
    let s = &labs()[0];
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::context(&a, s).unwrap();
    let previous = a.state.clone();
    assert!(
        engine::execute_for_goal(&mut a.state, &s.goal, "mdadm --detail --test /dev/md10")
            .unwrap_err()
            .contains("--test option is not implemented")
    );
    assert_eq!(a.state, previous);
    session(&mut a.state)
        .initial
        .devices
        .get_mut("/dev/vdc1")
        .unwrap()
        .raw_sectors += 1;
    assert!(work::context(&a, s).is_err());
    assert!(!progress::current_solved(&a, s));
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().service_port = Some("22".into());
    assert!(work::context(&a, s).is_err());
}
#[test]
fn prior275_scenarios_and82_units_remain_exact_and_the_new_unit_follows_real_prerequisites() {
    let old = pack::accepted275().unwrap();
    let old_units = dlms_learning_lab::course::accepted275(&old).unwrap();
    let current = pack::accepted277().unwrap();
    let units = dlms_learning_lab::course::accepted277(&current).unwrap();
    assert_eq!(
        (old.len(), current.len(), old_units.len(), units.len()),
        (275, 277, 82, 83)
    );
    let ids = old
        .iter()
        .map(|s| s.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            current
                .iter()
                .filter(|s| ids.contains(s.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let oldids = old_units
        .iter()
        .map(|s| s.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|s| oldids.contains(s.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let pos = |id: &str| units.iter().position(|u| u.id == id).unwrap();
    assert!(pos("linux-raid1-recovery") > pos("linux-disk-preparation"));
    assert!(pos("linux-raid1-recovery") > pos("persistent-mounts"));
    let mut counts = std::collections::BTreeMap::new();
    for s in &current {
        *counts.entry(s.track.as_str()).or_insert(0) += 1;
    }
    assert_eq!(
        (
            counts["CCNA"],
            counts["LPIC-1"],
            counts["LPIC-2"],
            counts["Bash/Shell"]
        ),
        (72, 89, 45, 71)
    );
}

#[test]
fn missing_slot_feedback_omits_a_vacuous_detach_objective_but_still_requires_real_redundancy() {
    let s = &labs()[1];
    let rows = engine::checks(&s.initial, &s.goal);
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        [false, true, true]
    );
    assert!(rows[0].1.contains("two fully synchronized copies"));
    assert!(!rows.iter().any(|r| r.1.contains("Detach")));
    let mut state = s.initial.clone();
    for command in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
}

#[test]
fn retirement_feedback_is_relevant_and_no_missing_sibling_claim_inflates_the_first_task() {
    let s = &labs()[0];
    let rows = engine::checks(&s.initial, &s.goal);
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        [false, false, true]
    );
    assert!(rows[0].1.contains("Detach"));
    assert!(!rows.iter().any(|r| r.1.contains("unrelated mirror")));
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
}
