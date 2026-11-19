#[path = "support/command_location_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    command_location_workspace as work,
    engine::{self, State},
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
fn finish(a: &mut Attempt, s: &pack::Scenario) {
    for cmd in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, cmd).unwrap();
    }
    assert!(progress::current_solved(a, s));
}
#[test]
fn source_pairing_initialization_is_unsolved_atomic_and_never_recreates_missing_resumed_work() {
    let s = fixture::scenario();
    let a = fresh(&s);
    assert_eq!(a.state, s.initial);
    assert!(!progress::current_solved(&a, &s));
    work::context(&a, &s).unwrap();
    for kind in 0..4 {
        let mut wrong = Attempt::new(&s, 2);
        workspace::prepare(&mut wrong);
        match kind {
            0 => wrong.scenario_version += 1,
            1 => wrong.workspace.as_mut().unwrap().version = 0,
            2 => wrong.workspace.as_mut().unwrap().service_port = Some("foreign".into()),
            _ => {
                wrong.state = State::Script {
                    source: "keep my script".into(),
                    run: None,
                }
            }
        }
        let before = serde_json::to_value(&wrong).unwrap();
        assert!(work::prepare(&mut wrong, &s, true).is_err());
        assert_eq!(serde_json::to_value(wrong).unwrap(), before);
    }
    let mut missing = fresh(&s);
    missing.workspace.as_mut().unwrap().terminal_learning = None;
    let before = serde_json::to_value(&missing).unwrap();
    assert!(work::prepare(&mut missing, &s, false).is_err());
    assert_eq!(serde_json::to_value(missing).unwrap(), before);
}
#[test]
fn assignment_partial_alternate_modes_recheck_live_state_and_invalidate_success() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "chmod 0755 /tools/preferred/report").unwrap();
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "chmod 0700 /tools/preferred/report").unwrap();
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "chmod u-x /tools/preferred/report").unwrap();
    assert!(!progress::current_solved(&a, &s));
    finish(&mut a, &s);
    let untouched = a.state.clone();
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "chmod 0777 /work/keep.txt").is_err());
    assert_eq!(a.state, untouched);
}
#[test]
fn practice_actual_commands_are_source_bound_and_independent_from_assignment() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    let original = a.state.clone();
    let l = s.terminal_teaching.as_ref().unwrap();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    for cmd in &l.example {
        learn.command = cmd.clone();
        learn.execute_for(l).unwrap();
    }
    assert!(l.outcomes.iter().all(|r| r.check(&learn.tree).0));
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, &s));
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    learn.command = "chmod u-x /gallery/preferred/demo-report".into();
    learn.execute_for(l).unwrap();
    assert!(l.outcomes.iter().any(|r| !r.check(&learn.tree).0));
    learn.command = "which demo-report".into();
    learn.execute_for(l).unwrap();
    assert!(
        learn
            .transcript
            .last()
            .unwrap()
            .ends_with("/gallery/fallback/demo-report\n")
    );
    let mut bad = l.clone();
    bad.fixture.command_location = None;
    learn.command = "preserved command draft".into();
    assert!(learn.execute_for(&bad).is_err());
    assert_eq!(learn.command, "preserved command draft");
}
#[test]
fn separate_drafts_state_favorites_and_other_attempts_roundtrip_and_reset_only_selected() {
    let s = fixture::scenario();
    let mut other_value = fixture::value();
    other_value["id"] = "other-location-fixture".into();
    fn role_ids(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::String(id)) = map.get_mut("source_id") {
                    *id = id.replace("command-location-fixture-", "other-location-fixture-");
                }
                for value in map.values_mut() {
                    role_ids(value);
                }
            }
            serde_json::Value::Array(a) => {
                for value in a {
                    role_ids(value)
                }
            }
            _ => {}
        }
    }
    role_ids(&mut other_value);
    let other = pack::load(
        &serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[other_value]}))
            .unwrap(),
    )
    .unwrap()
    .remove(0);
    let catalog = vec![s.clone(), other.clone()];
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(other.id.clone(), fresh(&other));
    let untouched = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let mut a = fresh(&s);
    finish(&mut a, &s);
    let w = a.workspace.as_mut().unwrap();
    w.command = "ls -l /tools/preferred/report".into();
    w.terminal_learning.as_mut().unwrap().command = "which demo-audit".into();
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    p.attempts.insert(s.id.clone(), fresh(&s));
    assert_eq!(
        serde_json::to_value(&p.attempts[&other.id]).unwrap(),
        untouched
    );
    assert!(p.favorites.contains(&s.id));
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
}
#[test]
fn erasure_source_role_and_foreign_workspace_forgery_refuse_before_disk_replacement() {
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
    for kind in 0..7 {
        let mut wrong: Progress = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        let a = wrong.attempts.get_mut(&s.id).unwrap();
        match kind {
            0 => {
                let State::Filesystem { tree } = &mut a.state else {
                    panic!()
                };
                tree.command_location = None;
            }
            1 => a.workspace.as_mut().unwrap().terminal_learning = None,
            2 => {
                let State::Filesystem { tree } = &mut a.state else {
                    panic!()
                };
                tree.command_location = None;
                a.workspace.as_mut().unwrap().terminal_learning = None;
            }
            3 => {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .terminal_learning
                    .as_mut()
                    .unwrap()
                    .tree = match &s.initial {
                    State::Filesystem { tree } => tree.clone(),
                    _ => panic!(),
                }
            }
            4 => {
                let State::Filesystem { tree } = &mut a.state else {
                    panic!()
                };
                tree.command_location.as_mut().unwrap().source_id = "forged".into();
            }
            5 => a.workspace.as_mut().unwrap().service_port = Some("foreign".into()),
            _ => {
                a.state = State::Script {
                    source: "keep this draft".into(),
                    run: None,
                }
            }
        }
        assert!(!progress::current_solved(a, &s));
        assert!(work::context(a, &s).is_err());
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
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(path).unwrap(), original);
}
#[test]
fn packs_reject_mismatched_original_reference_and_reused_practice_identity() {
    for kind in 0..3 {
        let mut s = fixture::value();
        match kind {
            0 => {
                s["goal"]["requirements"][1]["definition"]["initial"]["nodes"]["/tools/preferred/report"]
                    ["mode"] = serde_json::json!(0o644)
            }
            1 => {
                let id = s["initial"]["tree"]["command_location"]["source_id"].clone();
                s["terminal_teaching"]["fixture"]["command_location"]["source_id"] = id;
            }
            _ => {
                s["goal"]["requirements"] = serde_json::json!([{"kind":"Mode","path":"/tools/preferred/report","mode":0o700}])
            }
        }
        assert!(
            pack::load(
                &serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[s]}))
                    .unwrap()
            )
            .is_err()
        );
    }
}
