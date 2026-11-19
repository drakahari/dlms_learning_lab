//! Exact delivery-set outcomes; no host execution or command-history matching.
use dlms_learning_lab::{
    engine::Node,
    file_backup::{Entry, Tree},
    file_backup_terminal::Store,
    file_backup_workspace::{Definition, RecoveryProbe, Requirement},
};
use std::collections::BTreeMap;
fn empty() -> Tree {
    Tree {
        clock: 0,
        entries: BTreeMap::from([(
            "".into(),
            Entry {
                node: Node {
                    directory: true,
                    mode: 0o750,
                    owner: "alex".into(),
                    group: "staff".into(),
                },
                text: None,
                modified: 0,
                changed: 0,
            },
        )]),
    }
}
fn initial() -> Store {
    let mut source = empty();
    source
        .add(
            "records.txt",
            Node {
                directory: false,
                mode: 0o640,
                owner: "alex".into(),
                group: "staff".into(),
            },
            Some("Durable application records\n"),
        )
        .unwrap();
    let state = Store {
        trees: BTreeMap::from([
            ("/srv/demo".into(), source),
            ("/restore/demo".into(), empty()),
        ]),
        storage: "/backup".into(),
        archives: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        cwd: "/srv".into(),
    };
    assert!(state.valid());
    state
}
fn definition() -> Definition {
    let state = initial();
    let d = Definition {
        version: 1,
        initial: state.clone(),
        protected: vec!["/srv/demo".into()],
        requirements: vec![
            Requirement::ArtifactSet {
                name: "Deliver only the requested backup files".into(),
                archives: vec!["/backup/demo.tar".into()],
                snapshots: vec![],
            },
            Requirement::Recovery {
                name: "Recover actual data separately".into(),
                archives: vec!["/backup/demo.tar".into()],
                root: "demo".into(),
                probes: vec![RecoveryProbe {
                    name: "Independent empty input".into(),
                    initial: empty(),
                    expected: state.trees["/srv/demo"].clone(),
                }],
            },
        ],
    };
    d.validate().unwrap();
    d
}
fn solved() -> Store {
    let mut state = initial();
    state
        .execute("tar -cf /backup/demo.tar -C /srv demo")
        .unwrap();
    state
}
#[test]
fn exact_delivery_set_distinguishes_missing_extra_and_required_files_without_command_matching() {
    let d = definition();
    let rows = d.checks(&d.initial);
    assert!(!rows[1].0);
    assert!(rows[1].1.contains("1 missing and 0 extra archives"));
    for line in [
        "tar -cf /backup/demo.tar -C /srv demo",
        "tar --directory /srv --file /backup/demo.tar --create -- demo",
    ] {
        let mut state = initial();
        state.execute(line).unwrap();
        assert!(d.checks(&state).iter().all(|r| r.0));
    }
    let mut state = solved();
    state
        .execute("cp /backup/demo.tar /backup/unrequested.tar")
        .unwrap();
    let rows = d.checks(&state);
    assert!(!rows[1].0);
    assert!(rows[1].1.contains("0 missing and 1 extra archives"));
    assert!(rows[2].0);
    state.execute("rm /backup/unrequested.tar").unwrap();
    assert!(d.checks(&state).iter().all(|r| r.0));
    state.execute("rm /backup/demo.tar").unwrap();
    assert!(!d.checks(&state).iter().all(|r| r.0));
}
#[test]
fn right_names_do_not_claim_right_data_and_breaking_data_invalidates_recovery() {
    let d = definition();
    let mut state = solved();
    assert!(d.checks(&state).iter().all(|r| r.0));
    state
        .archives
        .get_mut("/backup/demo.tar")
        .unwrap()
        .entries
        .get_mut("records.txt")
        .unwrap()
        .text = Some("Wrong recovery data\n".into());
    assert!(state.valid());
    let rows = d.checks(&state);
    assert!(rows[1].0, "Filename-only row remains truthful");
    assert!(!rows[2].0, "Fresh actual recovery detects damaged contents");
    assert!(!rows.iter().all(|r| r.0));
}
#[test]
fn snapshots_are_separate_and_empty_delivery_sets_are_explicit() {
    let mut d = definition();
    let mut state = initial();
    state
        .execute("tar -cf /backup/demo.tar -g /backup/current.snar -C /srv demo")
        .unwrap();
    let rows = d.checks(&state);
    assert!(!rows[1].0);
    assert!(rows[1].1.contains("0 missing and 1 extra snapshots"));
    if let Requirement::ArtifactSet { snapshots, .. } = &mut d.requirements[0] {
        snapshots.push("/backup/current.snar".into())
    } else {
        panic!()
    }
    d.validate().unwrap();
    assert!(d.checks(&state).iter().all(|r| r.0));
    let empty_goal = Definition {
        version: 1,
        initial: initial(),
        protected: vec![],
        requirements: vec![Requirement::ArtifactSet {
            name: "Leave no backup files".into(),
            archives: vec![],
            snapshots: vec![],
        }],
    };
    empty_goal.validate().unwrap();
    assert!(empty_goal.checks(&empty_goal.initial).iter().all(|r| r.0));
    assert!(!empty_goal.checks(&state)[0].0);
}
#[test]
fn invalid_and_ambiguous_delivery_paths_are_rejected_without_changing_the_source() {
    let baseline = definition();
    let original = serde_json::to_value(&baseline).unwrap();
    for paths in [
        vec!["/outside/x"],
        vec!["/backup/../escape"],
        vec!["/backup/a", "/backup/a"],
        vec!["/backup/a/b"],
        vec!["/backup/demo.tar"; 9],
    ] {
        let mut d = baseline.clone();
        if let Requirement::ArtifactSet { archives, .. } = &mut d.requirements[0] {
            *archives = paths.into_iter().map(str::to_owned).collect()
        } else {
            panic!()
        }
        assert!(d.validate().is_err());
        assert_eq!(serde_json::to_value(&baseline).unwrap(), original);
    }
    let mut d = baseline.clone();
    if let Requirement::ArtifactSet { snapshots, .. } = &mut d.requirements[0] {
        snapshots.push("/backup/demo.tar".into())
    } else {
        panic!()
    }
    assert!(d.validate().is_err());
    let mut value = original.clone();
    value["requirements"][0]["unknown"] = true.into();
    assert!(Definition::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn complete_current_results_are_recomputed_after_roundtrip_not_stamped_as_success() {
    let d = definition();
    let good = solved();
    let db = serde_json::to_vec(&d).unwrap();
    let sb = serde_json::to_vec(&good).unwrap();
    let loaded = Definition::from_json(&db).unwrap();
    let mut state: Store = serde_json::from_slice(&sb).unwrap();
    assert_eq!(state, good);
    assert!(loaded.checks(&state).iter().all(|r| r.0));
    state
        .execute("cp /backup/demo.tar /backup/extra.tar")
        .unwrap();
    assert!(!loaded.checks(&state).iter().all(|r| r.0));
    assert!(loaded.checks(&good).iter().all(|r| r.0));
    assert_eq!(serde_json::to_vec(&d).unwrap(), db);
}
#[test]
fn existing_authored_goal_variants_roundtrip_without_new_requirements() {
    let pack: dlms_learning_lab::pack::Pack =
        serde_json::from_str(include_str!("../scenarios/file-backup-v1.json")).unwrap();
    assert_eq!(pack.scenarios.len(), 4);
    for s in pack.scenarios {
        let raw = serde_json::to_vec(&s.goal).unwrap();
        let goal: dlms_learning_lab::engine::Goal = serde_json::from_slice(&raw).unwrap();
        assert_eq!(serde_json::to_vec(&goal).unwrap(), raw);
        let dlms_learning_lab::engine::Goal::FileBackup { definition } = goal else {
            panic!()
        };
        assert!(
            !definition
                .requirements
                .iter()
                .any(|r| matches!(r, Requirement::ArtifactSet { .. }))
        );
        definition.validate().unwrap();
    }
}
