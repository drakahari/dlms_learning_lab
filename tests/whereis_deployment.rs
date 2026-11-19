#[path = "support/whereis_deployment_fixture.rs"]
mod fixture;
#[path = "support/command_location_fixture.rs"]
mod previous;
use dlms_learning_lab::{
    command_location_workspace as work,
    engine::{self, Goal, State},
    filesystem::{Requirement, discovery_policy as policy, whereis_deployment::Definition},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::prepare(&mut a, s, true).unwrap();
    a
}
fn run(a: &mut Attempt, s: &pack::Scenario, c: &str) -> String {
    engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap()
}
fn solve(a: &mut Attempt, s: &pack::Scenario) {
    for c in &s.solution {
        run(a, s, c);
    }
}
fn tree(a: &mut Attempt) -> &mut dlms_learning_lab::filesystem::Tree {
    let State::Filesystem { tree } = &mut a.state else {
        panic!()
    };
    tree
}
#[test]
fn source_validates_and_queries_are_real_readonly_category_results() {
    let s = fixture::scenario();
    let d = fixture::definition(&s);
    d.validate().unwrap();
    assert!(d.valid_for(fixture::tree(&s)));
    assert!(!d.check(&d.initial).0);
    let mut a = fresh(&s);
    let original = a.state.clone();
    assert_eq!(run(&mut a, &s, "which reportctl"), "/apps/bin/reportctl\n");
    assert_eq!(
        run(&mut a, &s, "whereis -b -B /apps/bin -f reportctl"),
        "reportctl: /apps/bin/reportctl\n"
    );
    assert_eq!(
        run(&mut a, &s, "whereis -m -M /apps/man -f reportctl"),
        "reportctl:\n"
    );
    assert_eq!(
        run(&mut a, &s, "whereis -s -S /apps/source -f reportctl"),
        "reportctl: /apps/source/reportctl.c\n"
    );
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn meaningful_partial_repair_and_three_equivalent_methods_grade_resulting_files() {
    let s = fixture::scenario();
    for method in [
        s.solution.clone(),
        vec![
            "cd /apps/stage".into(),
            "cp reportctl.1 ../man/reportctl.1".into(),
            "cp reportctl.c ../source/reportctl.c".into(),
            "rm ../man/reportctl-old.1".into(),
        ],
        vec![
            "cp /apps/stage/reportctl.c /work/support.tmp".into(),
            "mv /work/support.tmp /apps/source/reportctl.c".into(),
            "cp /apps/stage/reportctl.1 /apps/man/reportctl.1".into(),
            "rm /apps/man/reportctl-old.1".into(),
        ],
    ] {
        let mut a = fresh(&s);
        for (i, c) in method.iter().enumerate() {
            run(&mut a, &s, c);
            if i == 0 {
                assert!(!progress::current_solved(&a, &s));
            }
        }
        assert!(progress::current_solved(&a, &s));
        assert!(engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
    }
    let mut a = fresh(&s);
    run(&mut a, &s, &s.solution[0]);
    let r = engine::checks(&a.state, &s.goal);
    assert!(r.iter().any(|r| r.0) && r.iter().any(|r| !r.0));
}
#[test]
fn found_name_with_archived_bytes_or_wrong_mode_is_not_success_and_edits_invalidate() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    solve(&mut a, &s);
    assert!(progress::current_solved(&a, &s));
    run(
        &mut a,
        &s,
        "cp /apps/archive/reportctl.c /apps/source/reportctl.c",
    );
    assert_eq!(
        run(&mut a, &s, "whereis -s -S /apps/source -f reportctl"),
        "reportctl: /apps/source/reportctl.c\n"
    );
    assert!(!progress::current_solved(&a, &s));
    assert!(
        engine::checks(&a.state, &s.goal)
            .iter()
            .any(|(ok, m)| !ok && m.contains("release contents"))
    );
    run(
        &mut a,
        &s,
        "cp /apps/stage/reportctl.c /apps/source/reportctl.c",
    );
    assert!(progress::current_solved(&a, &s));
    run(&mut a, &s, "chmod 0600 /apps/source/reportctl.c");
    assert!(!progress::current_solved(&a, &s));
    run(&mut a, &s, "chmod 0644 /apps/source/reportctl.c");
    assert!(progress::current_solved(&a, &s));
    run(&mut a, &s, "rm /apps/man/reportctl.1");
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn protected_nodes_contexts_and_unsupported_commands_refuse_atomically() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    for c in [
        "chmod 0777 /apps/bin/reportctl",
        "rm /apps/man/auditctl.1",
        "cp /apps/stage/reportctl.c /work/unapproved",
        "rm /apps/archive/reportctl.c",
        "umask 000",
        "mkdir /apps/man/reportctl.1",
        "whereis reportctl",
        "whereis -m -M /home/private -f reportctl",
        "whereis -m -M /apps/man -f unknown",
        "reportctl",
        "/apps/bin/reportctl",
        "cp /apps/stage/reportctl.c /apps/source/reportctl.c; id",
        "cat /etc/passwd",
    ] {
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, c).is_err(),
            "{c}"
        );
        assert_eq!(a.state, before, "{c}");
    }
    let before = a.state.clone();
    let State::Filesystem { tree } = &mut a.state else {
        panic!()
    };
    assert!(
        tree.execute("cp /apps/stage/reportctl.c /apps/source/reportctl.c")
            .is_err()
    );
    assert_eq!(a.state, before);
}
#[test]
fn imported_source_erasure_wrong_family_and_held_reference_cannot_authorize_actions() {
    let s = fixture::scenario();
    for which in 0..7 {
        let mut a = fresh(&s);
        match which {
            0 => tree(&mut a).command_location = None,
            1 => tree(&mut a)
                .command_location
                .as_mut()
                .unwrap()
                .source_id
                .push_str("-forged"),
            2 => tree(&mut a)
                .users
                .get_mut("alex")
                .unwrap()
                .push("root".into()),
            3 => tree(&mut a)
                .files
                .insert("/work/notes.txt".into(), "forged".into())
                .map(|_| ())
                .unwrap(),
            4 => {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .terminal_learning
                    .as_mut()
                    .unwrap()
                    .tree
                    .command_location = None
            }
            5 => a.workspace.as_mut().unwrap().terminal_learning = None,
            _ => a.scenario_version += 1,
        }
        assert!(work::context(&a, &s).is_err());
        assert!(!progress::current_solved(&a, &s));
        let before = serde_json::to_value(&a).unwrap();
        assert!(work::prepare(&mut a, &s, false).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
    let mut state = s.initial.clone();
    let prior = state.clone();
    assert!(
        engine::execute_for_goal(
            &mut state,
            &Goal::Filesystem {
                requirements: vec![]
            },
            "help"
        )
        .is_err()
    );
    assert_eq!(state, prior);
    let mut other = State::Script {
        source: "keep this draft".into(),
        run: None,
    };
    let prior = other.clone();
    assert!(engine::execute_for_goal(&mut other, &s.goal, "help").is_err());
    assert_eq!(other, prior);
}
#[test]
fn separate_worked_practice_stays_ungraded_and_source_free_or_crossed_practice_keeps_draft() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    let original = a.state.clone();
    let lesson = s.terminal_teaching.as_ref().unwrap();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    for c in &lesson.example {
        learn.command = c.clone();
        learn.execute_for(lesson).unwrap();
    }
    assert!(lesson.outcomes.iter().all(|r| r.check(&learn.tree).0));
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, &s));
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    learn.command = "whereis -m -M /gallery/man -f democtl".into();
    let before = serde_json::to_value(&learn).unwrap();
    learn.execute();
    assert_eq!(serde_json::to_value(&learn).unwrap(), before);
    let mut foreign = lesson.clone();
    foreign.fixture = fixture::tree(&s).clone();
    assert!(learn.execute_for(&foreign).is_err());
    assert_eq!(serde_json::to_value(&learn).unwrap(), before);
}
#[test]
fn invalid_source_versions_reserved_names_paths_bounds_and_duplicate_families_are_rejected() {
    let s = fixture::scenario();
    let d = fixture::definition(&s);
    for kind in 0..9 {
        let mut bad = d.clone();
        match kind {
            0 => bad.version = 0,
            1 => bad.context.source_id.push_str("-wrong"),
            2 => bad.selected.pop().map(|_| ()).unwrap(),
            3 => bad.mutable_files.push(bad.mutable_files[0].clone()),
            4 => bad.mutable_files[0] = "/apps/bin/reportctl".into(),
            5 => bad.mutable_files[0] = "/apps/man/../bin/reportctl".into(),
            6 => {
                bad.context.names = vec!["printf".into()];
                for t in [&mut bad.initial, &mut bad.expected] {
                    t.command_location.as_mut().unwrap().names = vec!["printf".into()];
                }
            }
            7 => bad.initial.nodes.get_mut("/apps/man").unwrap().mode = 0o000,
            _ => {
                bad.initial
                    .files
                    .insert("/work/notes.txt".into(), "a".repeat(256 * 1024 + 1));
            }
        }
        assert!(bad.validate().is_err(), "kind {kind}");
    }
    let row = Requirement::WhereisDeployment {
        definition: Box::new(d.clone()),
    };
    assert!(policy::unique(&[row.clone(), row.clone()]).is_err());
    let old = serde_json::from_str::<serde_json::Value>(include_str!(
        "../scenarios/command-location-lessons-v1.json"
    ))
    .unwrap();
    let old: pack::Scenario = serde_json::from_value(old["scenarios"][0].clone()).unwrap();
    let Goal::Filesystem { requirements } = &old.goal else {
        panic!()
    };
    assert!(
        policy::unique(&[
            row,
            requirements
                .iter()
                .find(|r| matches!(r, Requirement::CommandLocation { .. }))
                .unwrap()
                .clone()
        ])
        .is_err()
    );
    assert!(Definition::decode(&vec![b' '; 1024 * 1024 + 1]).is_err());
}
#[test]
fn validated_save_resume_both_drafts_raw_save_refusal_and_reset_preserve_other_attempts() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    solve(&mut a, &s);
    a.workspace.as_mut().unwrap().command = "cat /apps/source/reportctl.c".into();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    l.view = dlms_learning_lab::script_teaching::View::Example;
    l.command = "cat /gallery/stage/release.txt".into();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a.clone());
    p.favorites.insert(s.id.clone());
    let other = previous::scenario();
    p.attempts.insert(other.id.clone(), fresh(&other));
    let untouched = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let catalog = vec![s.clone(), other.clone()];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
        untouched
    );
    let reset = fresh(&s);
    assert!(!progress::current_solved(&reset, &s));
    assert_eq!(reset.state, s.initial);
    assert!(loaded.favorites.contains(&s.id));
}

#[test]
fn reserved_lookup_names_and_slots_in_any_path_folder_are_refused_at_admission() {
    let s = fixture::scenario();
    let d = fixture::definition(&s);
    let raw = serde_json::to_string(d)
        .unwrap()
        .replace("reportctl", "printf");
    let reserved: Definition = serde_json::from_str(&raw).unwrap();
    reserved.context.validate_for(&reserved.initial).unwrap();
    assert!(
        reserved
            .initial
            .command_location
            .as_ref()
            .unwrap()
            .validate_for(&reserved.initial)
            .is_err()
    );
    assert!(reserved.validate().is_err());
    let mut extra = d.clone();
    for t in [&mut extra.initial, &mut extra.expected] {
        t.nodes
            .insert("/apps/fallback".into(), t.nodes["/apps/bin"].clone());
        t.command_location
            .as_mut()
            .unwrap()
            .path
            .push_str(":/apps/fallback");
    }
    extra.mutable_files[3] = "/apps/fallback/reportctl".into();
    assert!(extra.initial.valid());
    assert!(extra.validate().is_err());
}

#[test]
fn erased_or_crossed_saved_contexts_refuse_before_replacing_good_bytes() {
    let s = fixture::scenario();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), fresh(&s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let original = std::fs::read(&path).unwrap();
    for kind in 0..5 {
        let mut wrong: Progress = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        let a = wrong.attempts.get_mut(&s.id).unwrap();
        match kind {
            0 => tree(a).command_location = None,
            1 => a.workspace.as_mut().unwrap().terminal_learning = None,
            2 => {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .terminal_learning
                    .as_mut()
                    .unwrap()
                    .tree = fixture::tree(&s).clone()
            }
            3 => {
                tree(a).command_location = None;
                a.workspace.as_mut().unwrap().terminal_learning = None;
            }
            _ => a.workspace.as_mut().unwrap().service_port = Some("foreign".into()),
        }
        assert!(progress::save_with_catalog(&path, &wrong, std::slice::from_ref(&s)).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(
            progress::decode(
                &serde_json::to_vec(&wrong).unwrap(),
                std::slice::from_ref(&s)
            )
            .is_err()
        );
    }
}
