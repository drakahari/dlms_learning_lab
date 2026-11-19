//! Real pack/engine/progress admission using the two source fixtures, before catalog registration.
use dlms_learning_lab::{
    directory_policy_source as binding,
    engine::{self, Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
fn catalog() -> &'static Vec<Scenario> {
    static C: OnceLock<Vec<Scenario>> = OnceLock::new();
    C.get_or_init(|| {
        let base = pack::accepted290().unwrap();
        let fixture: Pack =
            serde_json::from_str(include_str!("fixtures/directory-policy-lessons-v1.json"))
                .unwrap();
        let byid: BTreeMap<_, _> = base.iter().map(|s| (s.id.clone(), s)).collect();
        let mut seen: BTreeSet<String> = fixture.scenarios.iter().map(|s| s.id.clone()).collect();
        let mut pending: Vec<String> = fixture
            .scenarios
            .iter()
            .flat_map(|s| s.prerequisites.iter().cloned())
            .collect();
        let mut subset = fixture.scenarios;
        while let Some(id) = pending.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let s = (*byid.get(&id).expect("Real retained prerequisite exists")).clone();
            pending.extend(s.prerequisites.clone());
            subset.push(s);
        }
        let validated = pack::load(
            &serde_json::to_string(&Pack {
                schema_version: 1,
                scenarios: subset,
            })
            .unwrap(),
        )
        .unwrap();
        let mut all = base;
        all.extend(
            validated
                .into_iter()
                .filter(|s| s.id.starts_with("lpic1-sticky-")),
        );
        all
    })
}
fn labs() -> Vec<&'static Scenario> {
    catalog()
        .iter()
        .filter(|s| s.id.starts_with("lpic1-sticky-"))
        .collect()
}
fn fresh(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    binding::context(&a, s).unwrap();
    a
}
fn solve(s: &Scenario) -> Attempt {
    let mut a = fresh(s);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn real_authored_sources_reach_outcomes_and_their_prerequisites_are_real() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        binding::source(s).unwrap();
        assert!(!progress::current_solved(&fresh(s), s));
        solve(s);
        for id in &s.prerequisites {
            assert!(catalog().iter().any(|other| other.id == *id));
        }
    }
}
#[test]
fn both_wrong_family_directions_are_refused_before_help_show_or_foreign_mutation() {
    for s in labs() {
        let d = binding::source(s).unwrap();
        let mut foreign = State::Filesystem {
            tree: d.initial.tree.clone(),
        };
        let before = foreign.clone();
        for command in [
            "help",
            "show",
            "chmod 0700 /work/shared",
            "touch /work/injected",
        ] {
            assert!(
                engine::execute_for_goal(&mut foreign, &s.goal, command).is_err(),
                "reverse family {command}"
            );
            assert_eq!(foreign, before);
        }
        let mut own = s.initial.clone();
        let old = own.clone();
        for command in ["help", "show", "chmod 1770 /work/shared"] {
            assert!(
                engine::execute_for_goal(
                    &mut own,
                    &Goal::Filesystem {
                        requirements: vec![]
                    },
                    command
                )
                .is_err()
            );
            assert_eq!(own, old);
            assert!(
                engine::execute(&mut own, command).is_err() || matches!(command, "help" | "show")
            );
            assert_eq!(own, old);
        }
    }
}
#[test]
fn catalog_aware_disk_save_resume_and_selected_reset_preserve_other_work_and_drafts() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    let old = &all[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    p.favorites.insert(old.id.clone());
    let old_json = serde_json::to_value(&p.attempts[&old.id]).unwrap();
    for s in labs() {
        let mut a = solve(s);
        let State::DirectoryPolicy { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "ls -ld /work ".into();
        simulation.practice.command = "chmod +t ".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    assert!(progress::save(&path, &p).is_err());
    assert!(!path.exists());
    progress::save_with_catalog(&path, &p, all).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut read = progress::load(&path, all).unwrap();
    assert_eq!(
        serde_json::to_value(&read).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for s in labs() {
        assert!(progress::current_solved(&read.attempts[&s.id], s));
    }
    let selected = labs()[0];
    read.attempts
        .insert(selected.id.clone(), Attempt::new(selected, 4));
    assert!(!progress::current_solved(
        &read.attempts[&selected.id],
        selected
    ));
    assert_eq!(
        serde_json::to_value(&read.attempts[&old.id]).unwrap(),
        old_json
    );
    let other = labs()[1];
    assert_eq!(
        serde_json::to_value(&read.attempts[&other.id]).unwrap(),
        serde_json::to_value(&p.attempts[&other.id]).unwrap()
    );
    assert_eq!(read.favorites, p.favorites);
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
#[test]
fn invalid_opaque_source_erasure_and_generic_extra_metadata_cannot_gain_success_or_save() {
    for s in labs() {
        let good = solve(s);
        let mut variants = vec![];
        let mut bad = good.clone();
        let State::DirectoryPolicy { simulation } = &mut bad.state else {
            panic!()
        };
        simulation.source_id = "foreign-source".into();
        variants.push(bad);
        let mut bad = good.clone();
        bad.state = State::Filesystem {
            tree: binding::source(s).unwrap().initial.tree.clone(),
        };
        variants.push(bad);
        let mut bad = good.clone();
        bad.step = s.steps.len();
        variants.push(bad);
        let mut bad = good.clone();
        bad.scenario_version += 1;
        variants.push(bad);
        let mut bad = good.clone();
        let mut extra = workspace::WorkspaceData::new();
        extra.command = "foreign legacy command".into();
        bad.workspace = Some(extra);
        variants.push(bad);
        for bad in variants {
            assert!(binding::context(&bad, s).is_err());
            assert!(!progress::current_solved(&bad, s));
            let mut p = Progress {
                schema_version: 1,
                ..Default::default()
            };
            p.attempts.insert(s.id.clone(), bad);
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("retained.json");
            std::fs::write(&path, b"original retained").unwrap();
            assert!(progress::save_with_catalog(&path, &p, catalog()).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), b"original retained");
        }
    }
}
#[test]
fn corrupted_disk_work_is_not_repaired_into_mastery_and_invalid_save_preserves_good_file() {
    let s = labs()[0];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), solve(s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let good = std::fs::read(&path).unwrap();
    let mut forged: serde_json::Value = serde_json::from_slice(&good).unwrap();
    forged["attempts"][&s.id]["state"]["simulation"]["assignment"]["history"] =
        serde_json::json!([]);
    let corrupt = dir.path().join("corrupt.json");
    std::fs::write(&corrupt, serde_json::to_vec(&forged).unwrap()).unwrap();
    let corrupt_bytes = std::fs::read(&corrupt).unwrap();
    assert!(progress::load(&corrupt, catalog()).is_err());
    assert_eq!(std::fs::read(&corrupt).unwrap(), corrupt_bytes);
    let State::DirectoryPolicy { simulation } = &mut p.attempts.get_mut(&s.id).unwrap().state
    else {
        panic!()
    };
    simulation.assignment.history.clear();
    assert!(progress::save_with_catalog(&path, &p, catalog()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), good);
}
