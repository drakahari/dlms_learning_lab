use dlms_learning_lab::{
    engine::Node,
    file_backup::{Entry, Tree},
    file_backup_terminal::Store,
    file_backup_workspace::{CheckRun, Definition, RecoveryProbe, Requirement},
};
use std::collections::BTreeMap;
fn node(directory: bool) -> Node {
    Node {
        directory,
        mode: if directory { 0o750 } else { 0o640 },
        owner: "alex".into(),
        group: "staff".into(),
    }
}
fn empty() -> Tree {
    Tree {
        clock: 0,
        entries: BTreeMap::from([(
            "".into(),
            Entry {
                node: node(true),
                text: None,
                modified: 0,
                changed: 0,
            },
        )]),
    }
}
fn initial() -> Store {
    let mut source = empty();
    for (name, text) in [
        ("stable.txt", "retained\n"),
        ("report.txt", "yesterday\n"),
        ("obsolete.txt", "remove later\n"),
    ] {
        source.add(name, node(false), Some(text)).unwrap();
    }
    let mut unrelated = empty();
    unrelated
        .add("notes.txt", node(false), Some("personal simulated note\n"))
        .unwrap();
    Store {
        trees: BTreeMap::from([
            ("/srv/team".into(), source),
            ("/restore/team".into(), empty()),
            ("/srv/unrelated".into(), unrelated),
        ]),
        storage: "/backups".into(),
        archives: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        cwd: "/srv".into(),
    }
}
fn chain() -> (Store, Store) {
    let initial = initial();
    let mut state = initial.clone();
    state
        .execute("tar -cf /backups/base.tar -g /backups/current.snar team")
        .unwrap();
    let source = state.trees.get_mut("/srv/team").unwrap();
    source.write("report.txt", "today\n").unwrap();
    source.remove("obsolete.txt").unwrap();
    source
        .add("new.txt", node(false), Some("new record\n"))
        .unwrap();
    state
        .execute("tar -cf /backups/change.tar -g /backups/current.snar team")
        .unwrap();
    (initial, state)
}
fn recovery_definition(state: &Store) -> Definition {
    let mut dirty = empty();
    dirty
        .add("orphan.txt", node(false), Some("stale\n"))
        .unwrap();
    Definition {
        version: 1,
        initial: state.clone(),
        protected: vec!["/srv/team".into(), "/srv/unrelated".into()],
        requirements: vec![
            Requirement::Recovery {
                name: "Recover on independent sample inputs".into(),
                root: "team".into(),
                archives: vec!["/backups/base.tar".into(), "/backups/change.tar".into()],
                probes: vec![
                    RecoveryProbe {
                        name: "Empty destination".into(),
                        initial: empty(),
                        expected: state.trees["/srv/team"].clone(),
                    },
                    RecoveryProbe {
                        name: "Destination with obsolete data".into(),
                        initial: dirty,
                        expected: state.trees["/srv/team"].clone(),
                    },
                ],
            },
            Requirement::Payloads {
                name: "Incremental payload".into(),
                archive: "/backups/change.tar".into(),
                root: "team".into(),
                files: vec!["new.txt".into(), "report.txt".into()],
                incremental: true,
            },
            Requirement::Snapshot {
                name: "Current snapshot".into(),
                path: "/backups/current.snar".into(),
                source: "/srv/team".into(),
            },
        ],
    }
}
fn passes(def: &Definition, state: &Store) -> bool {
    let rows = def.checks(state);
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
#[test]
fn short_and_long_real_operations_pass_outcomes_without_command_history_credit() {
    let initial = initial();
    let expected = initial.trees["/srv/team"].clone();
    let def = Definition {
        version: 1,
        initial: initial.clone(),
        protected: vec!["/srv/team".into(), "/srv/unrelated".into()],
        requirements: vec![Requirement::Tree {
            name: "Restore the team".into(),
            path: "/restore/team".into(),
            expected,
        }],
    };
    assert!(!passes(&def, &initial));
    for commands in [
        vec![
            "tar -cf /backups/full.tar team",
            "tar -xf /backups/full.tar -C /restore",
        ],
        vec![
            "tar --create --file=/backups/alternate.tar --directory=/srv team",
            "tar --extract --file /backups/alternate.tar --directory /restore",
        ],
    ] {
        let mut state = initial.clone();
        for command in commands {
            state.execute(command).unwrap();
        }
        assert!(passes(&def, &state));
    }
}
#[test]
fn independent_recovery_requires_both_levels_and_purges_stale_sample_data() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    assert!(passes(&def, &state));
    assert!(
        def.checks(&state)
            .iter()
            .any(|r| r.1.contains("obsolete data"))
    );
    let mut missing = state.clone();
    missing.archives.remove("/backups/base.tar");
    assert!(!passes(&def, &missing));
    assert!(
        def.checks(&missing)
            .iter()
            .any(|r| !r.0 && r.1.contains("base.tar is missing"))
    );
    let mut delta_only = def.clone();
    if let Requirement::Recovery { archives, .. } = &mut delta_only.requirements[0] {
        archives.remove(0);
    }
    assert!(!passes(&delta_only, &state));
    let mut reversed = def.clone();
    if let Requirement::Recovery { archives, .. } = &mut reversed.requirements[0] {
        archives.reverse();
    }
    assert!(!passes(&reversed, &state));
}
#[test]
fn a_filename_and_final_clean_tree_do_not_substitute_for_incremental_payloads() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let mut wrong = state.clone();
    wrong.execute("tar -cf /backups/change.tar team").unwrap();
    assert!(!passes(&def, &wrong));
    assert!(
        def.checks(&wrong)
            .iter()
            .any(|r| !r.0 && r.1.contains("backup type differ"))
    );
    wrong.archives.insert(
        "/backups/change.tar".into(),
        state.archives["/backups/base.tar"].clone(),
    );
    assert!(!passes(&def, &wrong));
    wrong.archives.get_mut("/backups/change.tar").unwrap().root = "renamed".into();
    assert!(!passes(&def, &wrong));
}
#[test]
fn preservation_checks_allow_logical_time_changes_but_reject_data_and_permission_damage() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let mut altered = state.clone();
    let tree = altered.trees.get_mut("/srv/unrelated").unwrap();
    tree.write("notes.txt", "personal simulated note\n")
        .unwrap();
    assert!(passes(&def, &altered));
    tree_damage(&mut altered);
    assert!(!passes(&def, &altered));
    let rows = def.checks(&altered);
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("1 wrong contents"))
    );
    let mut permissions = state.clone();
    permissions
        .execute("chmod 777 /srv/unrelated/notes.txt")
        .unwrap();
    assert!(!passes(&def, &permissions));
}
fn tree_damage(state: &mut Store) {
    state
        .trees
        .get_mut("/srv/unrelated")
        .unwrap()
        .write("notes.txt", "damaged\n")
        .unwrap();
}
#[test]
fn a_current_snapshot_is_not_the_same_as_a_snapshot_filename() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let mut altered = state.clone();
    altered
        .trees
        .get_mut("/srv/team")
        .unwrap()
        .write("report.txt", "later\n")
        .unwrap();
    assert!(!passes(&def, &altered));
    assert!(
        def.checks(&altered)
            .iter()
            .any(|r| !r.0 && r.1.contains("predates"))
    );
    let mut other = state.clone();
    other
        .snapshots
        .get_mut("/backups/current.snar")
        .unwrap()
        .source = "/srv/unrelated".into();
    assert!(!passes(&def, &other));
}
#[test]
fn partial_success_reports_exact_missing_extra_contents_and_metadata_without_dumping_answers() {
    let initial = initial();
    let mut expected = initial.trees["/srv/team"].clone();
    expected
        .write("report.txt", "confidential assignment answer\n")
        .unwrap();
    let def = Definition {
        version: 1,
        initial: initial.clone(),
        protected: vec![],
        requirements: vec![Requirement::Tree {
            name: "Restore".into(),
            path: "/restore/team".into(),
            expected,
        }],
    };
    let mut partial = initial.clone();
    partial
        .trees
        .insert("/restore/team".into(), initial.trees["/srv/team"].clone());
    let target = partial.trees.get_mut("/restore/team").unwrap();
    target.remove("obsolete.txt").unwrap();
    target
        .add("extra.txt", node(false), Some("extra\n"))
        .unwrap();
    target.chmod("report.txt", 0o777).unwrap();
    let rows = def.checks(&partial);
    assert!(!rows[0].0);
    for part in [
        "1 missing",
        "1 extra",
        "1 wrong contents",
        "1 wrong file types/permissions/owners",
    ] {
        assert!(rows[0].1.contains(part), "{}", rows[0].1);
    }
    assert!(!rows[0].1.contains("confidential assignment answer"));
}
#[test]
fn check_run_yields_between_each_archive_and_keeps_learner_state_unchanged() {
    let (_, state) = chain();
    let original = state.clone();
    let def = recovery_definition(&state);
    let mut run = CheckRun::new(&def, &state).unwrap();
    assert!(!run.done());
    assert!(!run.success());
    let mut ticks = 0;
    while !run.tick() {
        ticks += 1;
        assert!(ticks < 20);
    }
    ticks += 1;
    assert_eq!(ticks, 8);
    assert_eq!(run.rows().len(), 6);
    assert!(run.success());
    assert_eq!(state, original);
    assert!(run.tick());
    assert_eq!(run.rows().len(), 6);
}
#[test]
fn cancelling_and_restarting_a_check_cannot_publish_partial_success_or_modify_data() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let mut run = CheckRun::new(&def, &state).unwrap();
    run.tick();
    assert!(!run.success());
    drop(run);
    let mut changed = state.clone();
    changed.archives.remove("/backups/change.tar");
    let mut retry = CheckRun::new(&def, &changed).unwrap();
    while !retry.tick() {}
    assert!(!retry.success());
    assert_eq!(state.archives.len(), 2);
    assert!(passes(&def, &state));
}
#[test]
fn admitted_run_is_an_immutable_snapshot_and_a_new_check_invalidates_broken_work() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let mut run = CheckRun::new(&def, &state).unwrap();
    let mut changed = state.clone();
    tree_damage(&mut changed);
    while !run.tick() {}
    assert!(run.success());
    assert!(!passes(&def, &changed));
}
#[test]
fn context_escape_and_wrong_roots_fail_before_source_cloning() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    for mutate in [0, 1, 2] {
        let mut changed = state.clone();
        match mutate {
            0 => changed.storage = "/different".into(),
            1 => {
                let t = changed.trees.remove("/srv/team").unwrap();
                changed.trees.insert("/srv/team2".into(), t);
            }
            _ => changed.cwd = "/host/does-not-exist".into(),
        };
        assert!(CheckRun::new(&def, &changed).is_err());
        assert!(!passes(&def, &changed));
    }
}
#[test]
fn bounded_decoder_rejects_duplicates_extra_fields_bad_paths_and_oversized_rows() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let bytes = serde_json::to_vec(&def).unwrap();
    assert_eq!(Definition::from_json(&bytes).unwrap(), def);
    let valid = String::from_utf8(bytes).unwrap();
    for raw in [
        valid.replacen("\"version\":1", "\"version\":1,\"version\":1", 1),
        valid.replacen("\"version\":1", "\"version\":1,\"extra\":false", 1),
        valid.replacen("new.txt", "../new.txt", 1),
        valid.replacen("Current snapshot", &"n".repeat(121), 1),
    ] {
        assert!(Definition::from_json(raw.as_bytes()).is_err());
    }
    let mut duplicate = def.clone();
    duplicate
        .requirements
        .push(duplicate.requirements[0].clone());
    assert!(duplicate.validate().is_err());
    let mut duplicate = def.clone();
    duplicate.protected.push("/srv/team".into());
    assert!(duplicate.validate().is_err());
    assert!(Definition::from_json(&vec![b' '; 2 * 1024 * 1024 + 1]).is_err());
    let mut raw = serde_json::to_value(&def).unwrap();
    let row =
        serde_json::json!({"kind":"Snapshot","name":"x","path":"/backups/x","source":"/srv/team"});
    raw["requirements"] = serde_json::Value::Array(vec![row; 13]);
    assert!(Definition::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
}
#[test]
fn invalid_authored_expectations_are_not_accepted_as_automatic_success() {
    let (_, state) = chain();
    let mut def = recovery_definition(&state);
    if let Requirement::Recovery { archives, .. } = &mut def.requirements[0] {
        archives.clear();
    }
    assert!(def.validate().is_err());
    def = recovery_definition(&state);
    if let Requirement::Payloads { files, .. } = &mut def.requirements[1] {
        files.push("/host/path".into());
    }
    assert!(def.validate().is_err());
    def = recovery_definition(&state);
    if let Requirement::Payloads { root, .. } = &mut def.requirements[1] {
        *root = "team/child".into();
    }
    assert!(def.validate().is_err());
    def = recovery_definition(&state);
    def.initial.storage = format!("/{}", ["abcdefghijklmno"; 62].join("/"));
    assert!(def.initial.storage.len() < 1024);
    def.initial.archives.clear();
    def.initial.snapshots.clear();
    assert!(def.initial.valid());
    def.requirements = vec![Requirement::Snapshot {
        name: "Too deep".into(),
        path: format!("{}/{}", def.initial.storage, "x".repeat(32)),
        source: "/srv/team".into(),
    }];
    assert!(def.validate().is_err());
    def = recovery_definition(&state);
    def.version = 2;
    assert!(def.validate().is_err());
}
#[test]
fn recovery_checks_the_named_root_even_without_a_payload_requirement() {
    let (_, state) = chain();
    let mut def = recovery_definition(&state);
    def.requirements.truncate(1);
    assert!(passes(&def, &state));
    let mut wrong = state.clone();
    for archive in wrong.archives.values_mut() {
        archive.root = "another-team".into();
    }
    assert!(wrong.valid());
    assert!(!passes(&def, &wrong));
    assert!(
        def.checks(&wrong)
            .iter()
            .any(|r| !r.0 && r.1.contains("different top-level folder"))
    );
}
#[test]
fn archive_and_snapshot_preservation_distinguishes_a_copy_from_overwriting_history() {
    let (_, state) = chain();
    let mut def = recovery_definition(&state);
    def.requirements.push(Requirement::PreserveArtifacts {
        name: "Preserve recovery history".into(),
        archives: vec!["/backups/base.tar".into()],
        snapshots: vec!["/backups/current.snar".into()],
    });
    assert!(passes(&def, &state));
    let mut branch = state.clone();
    branch
        .execute("cp /backups/current.snar /backups/branch.snar")
        .unwrap();
    branch
        .execute("cp /backups/base.tar /backups/copied.tar")
        .unwrap();
    assert!(passes(&def, &branch));
    branch
        .execute("cp /backups/change.tar /backups/base.tar")
        .unwrap();
    assert!(!passes(&def, &branch));
    assert!(
        def.checks(&branch)
            .iter()
            .any(|r| !r.0 && r.1.contains("1 changed or missing backup archives"))
    );
    let mut removed = state.clone();
    removed.execute("rm /backups/current.snar").unwrap();
    assert!(!passes(&def, &removed));
    let mut bad = def.clone();
    if let Requirement::PreserveArtifacts { archives, .. } = bad.requirements.last_mut().unwrap() {
        archives.push("/backups/not-supplied.tar".into());
    }
    assert!(bad.validate().is_err());
}
#[test]
fn excess_definition_rows_are_rejected_before_their_bodies_are_decoded() {
    let (_, state) = chain();
    let def = recovery_definition(&state);
    let mut value = serde_json::to_value(&def).unwrap();
    let row =
        serde_json::json!({"kind":"Snapshot","name":"x","path":"/backups/s","source":"/srv/team"});
    let mut rows = vec![row; 12];
    rows.push(serde_json::json!("not an objective; body must not be read"));
    value["requirements"] = serde_json::Value::Array(rows);
    let error = Definition::from_json(&serde_json::to_vec(&value).unwrap()).unwrap_err();
    assert!(error.contains("capacity exceeded before body"), "{error}");
}
