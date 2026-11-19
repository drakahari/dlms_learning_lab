//! Source-held task behavior, history admission and isolated draft regressions.
use dlms_learning_lab::directory_policy_workspace::{
    Action, Definition, Expected, MAX_SESSION, Session, Task,
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Fixture {
    schema_version: u32,
    tasks: Vec<FixtureTask>,
}
#[derive(Deserialize)]
struct FixtureTask {
    id: String,
    definition: Definition,
    solution: Vec<String>,
}
fn tasks() -> Vec<FixtureTask> {
    let v: Fixture =
        serde_json::from_str(include_str!("fixtures/directory-policy-workspace-v1.json")).unwrap();
    assert_eq!(v.schema_version, 1);
    assert_eq!(v.tasks.len(), 2);
    v.tasks
}
fn passed(d: &Definition, s: &Session, practice: bool) -> bool {
    d.checks(s, practice).iter().all(|r| r.0)
}
fn solve(task: &FixtureTask) -> Session {
    let mut s = Session::new(&task.definition);
    for c in &task.solution {
        s.run(&task.definition, c, false).unwrap();
    }
    assert!(
        passed(&task.definition, &s, false),
        "{}: {:?}",
        task.id,
        task.definition.checks(&s, false)
    );
    s
}
#[test]
fn source_tasks_validate_without_solving_and_authored_methods_reach_complete_outcomes() {
    for t in tasks() {
        t.definition.validate().unwrap();
        let s = Session::new(&t.definition);
        assert!(s.assignment.history.is_empty());
        assert!(s.practice.history.is_empty());
        assert_eq!(s.assignment.world, t.definition.initial);
        assert_eq!(s.practice.world, t.definition.example);
        assert!(!passed(&t.definition, &s, false));
        assert!(!passed(&t.definition, &s, true));
        solve(&t);
    }
}
#[test]
fn complete_examples_never_change_assignment_or_earn_its_outcomes() {
    for t in tasks() {
        let d = &t.definition;
        let mut s = Session::new(d);
        s.assignment.command = "unfinished assignment command".into();
        s.practice.command = "unfinished example command".into();
        let before = s.assignment.clone();
        for c in &d.example_commands {
            s.run(d, c, true).unwrap();
        }
        assert!(passed(d, &s, true), "{}: {:?}", t.id, d.checks(&s, true));
        assert_eq!(s.assignment, before);
        assert_eq!(s.practice.command, "unfinished example command");
        assert!(!passed(d, &s, false));
    }
}
#[test]
fn numerical_and_symbolic_policy_repairs_pass_but_lost_collaboration_and_cleared_sticky_fail() {
    let t = tasks().remove(0);
    let d = &t.definition;
    let Task::Collaboration { directory, .. } = &d.goal else {
        panic!()
    };
    for commands in [
        vec![format!("chmod 1770 '{directory}'")],
        vec![
            format!("chmod u=rwx,g=rwx,o= '{directory}'"),
            format!("chmod +t '{directory}'"),
        ],
    ] {
        let mut s = Session::new(d);
        for c in commands {
            s.run(d, &c, false).unwrap();
        }
        assert!(passed(d, &s, false));
    }
    let mut s = solve(&t);
    s.run(d, &format!("chmod 1700 '{directory}'"), false)
        .unwrap();
    assert!(!passed(d, &s, false));
    let failures = d.checks(&s, false);
    assert!(
        failures
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("needs to work"))
    );
    s.run(d, &format!("chmod 1770 '{directory}'"), false)
        .unwrap();
    assert!(passed(d, &s, false));
    s.run(d, &format!("chmod a=rwx '{directory}'"), false)
        .unwrap();
    assert!(!passed(d, &s, false));
    assert!(
        d.checks(&s, false)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("needs to be blocked"))
    );
}
#[test]
fn missing_foreign_file_is_not_permission_denial_credit() {
    let t = tasks().remove(0);
    let d = &t.definition;
    let mut s = solve(&t);
    s.run(d, "rm /work/shared/readonly-note", false).unwrap();
    let rows = d.checks(&s, false);
    assert!(!passed(d, &s, false));
    assert!(
        rows.iter()
            .any(|(ok, msg)| !ok && msg.contains("missing file or another operation error"))
    );
}
#[test]
fn collision_is_blocked_atomically_and_partial_handover_remains_incomplete() {
    let t = tasks().remove(1);
    let d = &t.definition;
    let Task::Handover {
        source, collision, ..
    } = &d.goal
    else {
        panic!()
    };
    let mut s = Session::new(d);
    let before = s.clone();
    let error = s
        .run(d, &format!("mv '{source}' '{collision}'"), false)
        .unwrap_err();
    assert!(error.contains("Restricted deletion"));
    assert_eq!(s, before);
    s.run(d, &t.solution[0], false).unwrap();
    assert!(!passed(d, &s, false));
    s.run(d, &t.solution[1], false).unwrap();
    assert!(passed(d, &s, false));
}
#[test]
fn equivalent_copy_remove_handover_passes_and_bad_content_has_an_actual_recovery() {
    let t = tasks().remove(1);
    let d = &t.definition;
    let Task::Handover {
        source,
        destination,
        retire,
        collision,
        ..
    } = &d.goal
    else {
        panic!()
    };
    let mut s = Session::new(d);
    for c in [
        format!("cp '{source}' '{destination}'"),
        format!("rm '{source}'"),
        format!("rm '{retire}'"),
    ] {
        s.run(d, &c, false).unwrap();
    }
    assert!(passed(d, &s, false));
    s.run(d, &format!("cp '{destination}' '{collision}'"), false)
        .unwrap();
    assert!(!passed(d, &s, false));
    s.run(
        d,
        &format!("cp /work/retained-release.txt '{collision}'"),
        false,
    )
    .unwrap();
    assert!(passed(d, &s, false));
    s.run(d, &format!("rm '{destination}'"), false).unwrap();
    assert!(!passed(d, &s, false));
    s.run(
        d,
        &format!("cp /work/approved-copy.txt '{destination}'"),
        false,
    )
    .unwrap();
    assert!(passed(d, &s, false));
}
#[test]
fn own_and_foreign_role_probes_use_discarded_models_not_completion_strings() {
    for t in tasks() {
        let s = solve(&t);
        let before = s.clone();
        for (i, c) in t.definition.goal.cases().iter().enumerate() {
            let (ok, feedback) = s.participant(&t.definition, false, i).unwrap();
            assert!(ok, "{} {:?}: {feedback}", t.id, c.expected);
            if c.expected == Expected::PermissionDenied {
                assert!(
                    feedback.contains("Restricted deletion")
                        || feedback.contains("Permission denied")
                );
            }
        }
        assert_eq!(s, before);
        assert!(s.participant(&t.definition, false, 99).is_err());
    }
}
#[test]
fn forged_world_roles_erased_history_and_swapped_practice_are_refused() {
    for t in tasks() {
        let d = &t.definition;
        let good = solve(&t);
        let mut variants = vec![];
        let mut bad = good.clone();
        bad.source_id = "other-task".into();
        variants.push(bad);
        let mut bad = good.clone();
        bad.assignment.history.clear();
        variants.push(bad);
        let mut bad = good.clone();
        bad.assignment.world.tree.user = "root".into();
        variants.push(bad);
        let mut bad = good.clone();
        bad.assignment
            .world
            .tree
            .users
            .insert("extra".into(), vec!["team".into()]);
        variants.push(bad);
        let mut bad = good.clone();
        std::mem::swap(&mut bad.assignment, &mut bad.practice);
        variants.push(bad);
        for bad in variants {
            assert!(d.bound(&bad).is_err());
            assert!(!passed(d, &bad, false));
        }
    }
}
#[test]
fn typed_decode_and_both_drafts_roundtrip_and_fresh_constructor_resets_only_this_session() {
    for t in tasks() {
        let d = &t.definition;
        let mut s = solve(&t);
        s.assignment.command = "ls -l /work ".into();
        s.practice.command = "chmod +t ".into();
        let raw = serde_json::to_vec(&s).unwrap();
        let read = Session::decode(&raw).unwrap();
        assert_eq!(read, s);
        d.bound(&read).unwrap();
        assert!(passed(d, &read, false));
        let fresh = Session::new(d);
        assert_eq!(fresh.assignment.world, d.initial);
        assert!(fresh.assignment.command.is_empty());
        assert!(fresh.practice.command.is_empty());
        assert!(!passed(d, &fresh, false));
        assert_eq!(read.assignment.command, "ls -l /work ");
    }
}
#[test]
fn malformed_bounds_reject_before_serialization_or_reference_replay() {
    let t = tasks().remove(0);
    let d = &t.definition;
    let mut bad = Session::new(d);
    bad.assignment.history = vec![Action::Command {
        input: "x".repeat(4097),
    }];
    assert!(bad.validate().is_err());
    let mut bad = Session::new(d);
    bad.assignment.transcript = vec!["x".repeat(8193)];
    assert!(bad.validate().is_err());
    let mut bad = Session::new(d);
    bad.assignment.history = vec![
        Action::Command {
            input: "pwd".into()
        };
        129
    ];
    assert!(bad.validate().is_err());
    assert!(Session::decode(&vec![b' '; MAX_SESSION + 1]).is_err());
    let mut v = serde_json::to_value(Session::new(d)).unwrap();
    v.as_object_mut().unwrap().remove("source_id");
    assert!(Session::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = serde_json::to_value(Session::new(d)).unwrap();
    v["foreign"] = serde_json::json!(true);
    assert!(Session::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn invalid_authored_roles_collision_and_clock_only_examples_are_not_admitted() {
    for t in tasks() {
        let mut d = t.definition.clone();
        d.example = d.initial.clone();
        d.example.tree.clock = 1;
        assert!(d.validate().is_err());
        let mut d = t.definition.clone();
        match &mut d.goal {
            Task::Collaboration { cases, .. } | Task::Handover { cases, .. } => {
                cases[0].actor = "root".into()
            }
        };
        assert!(d.validate().is_err());
    }
    let t = tasks().remove(1);
    let mut d = t.definition;
    let Task::Handover { collision, .. } = &d.goal else {
        panic!()
    };
    d.initial.tree.nodes.get_mut(collision).unwrap().owner = d.initial.tree.user.clone();
    assert!(d.validate().is_err());
}
