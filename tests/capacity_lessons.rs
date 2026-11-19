use dlms_learning_lab::{
    capacity_workspace as w, course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/capacity-lessons-v1.json")).unwrap()
}
fn parts(s: &pack::Scenario) -> (&w::Definition, &w::Session) {
    let (Goal::Capacity { definition }, State::Capacity { simulation }) = (&s.goal, &s.initial)
    else {
        panic!()
    };
    (definition, simulation)
}
fn solved(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for a in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, a).unwrap();
    }
    state
}
#[test]
fn capacity_all_four_real_references_and_separate_worked_examples() {
    for s in labs() {
        let (d, init) = parts(&s);
        assert!(!engine::solved(&s.initial, &s.goal));
        let done = solved(&s);
        assert!(
            engine::solved(&done, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&done, &s.goal)
        );
        let mut example = init.clone();
        for a in &d.example_actions {
            match a {
                w::Action::Put { path, bytes } => {
                    example.practice.path = path.clone();
                    example.practice.bytes = bytes.to_string();
                    example.practice.dirty = true;
                    example.practice.put().unwrap();
                }
                w::Action::Remove { names } => {
                    example.practice.selected = names.clone();
                    example.practice.preview_remove().unwrap();
                    example.practice.apply_remove().unwrap();
                }
            }
        }
        assert!(
            d.checks(&example, true).iter().all(|r| r.0),
            "{} {:?}",
            s.id,
            d.checks(&example, true)
        );
        assert_eq!(example.assignment, init.assignment);
        assert!(!d.checks(&example, false).iter().all(|r| r.0));
    }
}
#[test]
fn capacity_ten_different_inode_cleanups_pass_but_shrinking_does_not() {
    let s = labs().remove(0);
    let (d, init) = parts(&s);
    let names: Vec<_> = (1..=5).map(|n| format!("/spool-{n}")).collect();
    let mut count = 0;
    for a in 0..5 {
        for b in a + 1..5 {
            for c in b + 1..5 {
                let mut x = init.clone();
                x.assignment.selected = vec![names[a].clone(), names[b].clone(), names[c].clone()];
                x.assignment.preview_remove().unwrap();
                x.assignment.apply_remove().unwrap();
                x.assignment.path = "/result".into();
                x.assignment.bytes = "4096".into();
                x.assignment.put().unwrap();
                assert!(d.checks(&x, false).iter().all(|r| r.0));
                count += 1;
            }
        }
    }
    assert_eq!(count, 10);
    let mut x = init.clone();
    x.assignment.path = "/spool-1".into();
    x.assignment.bytes = "0".into();
    x.assignment.put().unwrap();
    x.assignment.path = "/result".into();
    assert!(x.assignment.put().unwrap_err().contains("inode"));
    assert!(!d.checks(&x, false).iter().all(|r| r.0));
}
#[test]
fn capacity_block_shrink_is_a_legitimate_alternate_to_removal() {
    let s = labs().remove(1);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    x.assignment.path = "/cache-large".into();
    x.assignment.bytes = "4096".into();
    x.assignment.put().unwrap();
    x.assignment.path = "/report".into();
    x.assignment.bytes = "32769".into();
    x.assignment.put().unwrap();
    assert!(d.checks(&x, false).iter().all(|r| r.0));
    assert!(
        x.assignment
            .world
            .files()
            .unwrap()
            .contains_key("/cache-large")
    );
    x.assignment.path = "/report".into();
    x.assignment.bytes = "32768".into();
    x.assignment.dirty = true;
    x.assignment.put().unwrap();
    let rows = d.checks(&x, false);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("/report")));
    assert!(
        rows.iter()
            .any(|r| r.0 && !r.1.contains("headroom") && r.1.contains("ordinary-available"))
    );
}
#[test]
fn capacity_partial_handover_and_wrong_length_cannot_false_pass() {
    for s in labs() {
        let (d, init) = parts(&s);
        let mut x = init.clone();
        let first = &s.solution[0];
        let mut state = State::Capacity {
            simulation: Box::new(x.clone()),
        };
        engine::execute_for_goal(&mut state, &s.goal, first).unwrap();
        assert!(!engine::solved(&state, &s.goal), "{}", s.id);
        let State::Capacity { simulation } = solved(&s) else {
            panic!()
        };
        x = *simulation;
        let requirement = d
            .requirements
            .iter()
            .find_map(|r| match r {
                w::Requirement::File { path, bytes }
                    if !d.initial.profile().unwrap().retained.contains_key(path) =>
                {
                    Some((path, bytes))
                }
                _ => None,
            })
            .unwrap();
        x.assignment.path = requirement.0.clone();
        x.assignment.bytes = (requirement.1 + 1).to_string();
        x.assignment.dirty = true;
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
        x.assignment.put().unwrap();
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
    }
}
#[test]
fn capacity_stale_preview_refuses_atomically_and_a_new_preview_recovers() {
    let s = labs().remove(2);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    x.assignment.selected = vec!["/cache".into()];
    x.assignment.preview_remove().unwrap();
    let old = x.assignment.preview.clone();
    x.assignment.path = "/active".into();
    x.assignment.bytes = "24576".into();
    x.assignment.put().unwrap();
    let before = x.assignment.world.clone();
    assert!(x.assignment.apply_remove().unwrap_err().contains("changed"));
    assert_eq!(before, x.assignment.world);
    assert_eq!(old, x.assignment.preview);
    x.assignment.preview_remove().unwrap();
    x.assignment.apply_remove().unwrap();
    x.assignment.path = "/manifest".into();
    x.assignment.bytes = "4096".into();
    x.assignment.put().unwrap();
    assert!(d.checks(&x, false).iter().all(|r| r.0));
}
#[test]
fn capacity_invalid_text_paths_and_protected_mutations_are_atomic() {
    let s = labs().remove(0);
    let (_, init) = parts(&s);
    for (path, bytes) in [
        ("/../host", "1"),
        ("/tmp/escape", "1"),
        ("/audit", "0"),
        ("/x", "-1"),
        ("/x", "1KiB"),
        ("/x", ""),
        ("/x", "18446744073709551616"),
    ] {
        let mut x = init.assignment.clone();
        x.path = path.into();
        x.bytes = bytes.into();
        let before = x.world.clone();
        assert!(x.put().is_err(), "{path} {bytes}");
        assert_eq!(before, x.world);
    }
    let mut x = init.assignment.clone();
    x.selected = vec!["/audit".into()];
    assert!(x.preview_remove().is_err());
    assert_eq!(x.world, init.assignment.world);
    for cmd in [
        "truncate -s 0 /audit",
        "rm /audit",
        "capacity-action {\"kind\":\"Put\",\"path\":\"/x\",\"bytes\":1,\"extra\":true}",
    ] {
        let mut st = s.initial.clone();
        assert!(engine::execute_for_goal(&mut st, &s.goal, cmd).is_err());
        assert_eq!(st, s.initial);
    }
}
#[test]
fn capacity_save_reload_preserves_unfinished_draft_selection_and_stale_preview() {
    let all = pack::ccna_pilot().unwrap();
    let s = labs().remove(2);
    let mut a = Attempt::new(&s, 1);
    let State::Capacity { simulation } = &mut a.state else {
        panic!()
    };
    simulation.view = View::Example;
    simulation.assignment.selected = vec!["/cache".into()];
    simulation.assignment.preview_remove().unwrap();
    simulation.assignment.path = "/active".into();
    simulation.assignment.bytes = "24576".into();
    simulation.assignment.put().unwrap();
    simulation.assignment.bytes = "24x".into();
    simulation.assignment.dirty = true;
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(!progress::current_solved(&got.attempts[&s.id], &s));
    assert!(progress::save(&path, &p).is_err());
}
#[test]
fn capacity_source_forgery_and_foreign_metadata_cannot_be_saved_or_graded() {
    let s = labs().remove(0);
    let mut a = Attempt::new(&s, 1);
    a.state = solved(&s);
    assert!(progress::current_solved(&a, &s));
    let other = labs().remove(1);
    let State::Capacity { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.world = parts(&other).0.initial.clone();
    assert!(w::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    let mut a = Attempt::new(&s, 1);
    a.workspace = Some(dlms_learning_lab::workspace::WorkspaceData::new());
    assert!(w::context(&a, &s).is_ok());
    a.workspace.as_mut().unwrap().command = "foreign command".into();
    assert!(w::context(&a, &s).is_err());
}
#[test]
fn capacity_current_success_is_cleared_by_pending_and_applied_damage_and_reset_is_local() {
    let all = pack::ccna_pilot().unwrap();
    let s = labs().remove(1);
    let mut a = Attempt::new(&s, 1);
    a.state = solved(&s);
    assert!(progress::current_solved(&a, &s));
    let State::Capacity { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.path = "/report".into();
    simulation.assignment.bytes = "1".into();
    simulation.assignment.dirty = true;
    assert!(!progress::current_solved(&a, &s));
    let State::Capacity { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.put().unwrap();
    assert!(!progress::current_solved(&a, &s));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let untouched = &all[0];
    p.attempts
        .insert(untouched.id.clone(), Attempt::new(untouched, 2));
    let before = serde_json::to_value(&p.attempts[&untouched.id]).unwrap();
    p.attempts.insert(s.id.clone(), Attempt::new(&s, 3));
    assert_eq!(
        before,
        serde_json::to_value(&p.attempts[&untouched.id]).unwrap()
    );
    assert_eq!(p.attempts[&s.id].state, s.initial);
}
#[test]
fn capacity_catalog_preserves_prior_217_and_orders_one_real_unit() {
    let current = pack::accepted221().unwrap();
    assert_eq!(current.len(), 221);
    let units = course::accepted221(&current).unwrap();
    assert_eq!(units.len(), 59);
    let u = units
        .iter()
        .find(|u| u.id == "linux-capacity-handover")
        .unwrap();
    assert_eq!(
        u.labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    assert_eq!(u.prerequisites, ["persistent-mounts"]);
    let accepted = pack::accepted213().unwrap();
    assert_eq!(
        serde_json::to_value(&current[..213]).unwrap(),
        serde_json::to_value(&accepted).unwrap()
    );
    let syslog =
        serde_json::from_str::<pack::Pack>(include_str!("../scenarios/syslog-policy-v1.json"))
            .unwrap()
            .scenarios;
    assert_eq!(
        serde_json::to_value(&current[213..217]).unwrap(),
        serde_json::to_value(&syslog).unwrap()
    );
}

#[test]
fn capacity_invalid_packs_and_corrupt_preview_cannot_be_admitted() {
    let text = include_str!("../scenarios/capacity-lessons-v1.json");
    for mutate in 0..5 {
        let mut v: serde_json::Value = serde_json::from_str(text).unwrap();
        let first = &mut v["scenarios"][0];
        match mutate {
            0 => first["goal"]["definition"]["requirements"] = serde_json::json!([]),
            1 => {
                first["goal"]["definition"]["example"] =
                    first["goal"]["definition"]["initial"].clone()
            }
            2 => {
                first["goal"]["definition"]["requirements"][0]["path"] =
                    serde_json::json!("/../escape")
            }
            3 => {
                first["initial"]["simulation"]["assignment"]["world"]["profile"]["total_blocks"] =
                    serde_json::json!(999)
            }
            _ => {
                first["goal"]["definition"]["example_actions"][0]["unknown"] =
                    serde_json::json!(true)
            }
        }
        assert!(pack::load(&v.to_string()).is_err(), "invalid case {mutate}");
    }
    let all = pack::ccna_pilot().unwrap();
    let s = labs().remove(0);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), Attempt::new(&s, 1));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let before = std::fs::read(&path).unwrap();
    let State::Capacity { simulation } = &mut p.attempts.get_mut(&s.id).unwrap().state else {
        panic!()
    };
    simulation.assignment.preview = Some(w::Preview {
        before: simulation.practice.world.clone(),
        names: vec!["/sample-q1".into()],
    });
    assert!(progress::save_with_catalog(&path, &p, &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&raw, &all).is_err());
}
