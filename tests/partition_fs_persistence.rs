//! Prospective source-aware persistence boundaries; private fixtures are not curriculum.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack, partition_fs_workspace as work,
    progress::{self, Attempt, Progress},
};
#[path = "support/partition_fs_fixture.rs"]
mod fixture;
use fixture::*;
fn value<T: serde::Serialize + ?Sized>(x: &T) -> serde_json::Value {
    serde_json::to_value(x).unwrap()
}
fn profile(s: &pack::Scenario) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 3);
    work::binding::prepare(&mut a, s).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p
}
fn files(dir: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read(p).unwrap(),
            )
        })
        .collect()
}
#[test]
fn warm_cold_export_inspect_restore_and_reset_preserve_distinct_worlds_drafts_and_receipts() {
    let s = scenario();
    let all = vec![s.clone()];
    let mut p = profile(&s);
    let a = p.attempts.get_mut(&s.id).unwrap();
    for op in operations("/dev/vdc", "reports", "paging", 12, 3, "reports") {
        work::binding::apply(a, &s, &op, false).unwrap();
    }
    let mut draft = session(a).assignment_editor.drafts["/dev/vda"].clone();
    draft.start = "unfinished".into();
    work::binding::set_draft(a, &s, "/dev/vda", draft, false).unwrap();
    work::binding::insert(a, &s, 1).unwrap();
    work::binding::request_hint(a, &s).unwrap();
    work::binding::check(a, &s, false).unwrap();
    assert!(progress::current_solved(a, &s));
    let portable = value(&p);
    let caller_receipts = value(&p.migration_receipts);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(value(&p), portable);
    assert_eq!(value(&p.migration_receipts), caller_receipts);
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(value(&loaded.attempts), value(&p.attempts));
    assert!(loaded.migration_receipts.is_some());
    let export = dir.path().join("portable.json");
    backup::export(&export, &p, &all).unwrap();
    let candidate = backup::inspect(
        &export,
        &all,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(value(&candidate.progress.attempts), value(&p.attempts));
    assert!(candidate.summary(&all).contains("1"));
    let restore = dir.path().join("restored.json");
    backup::restore(&restore, &candidate, &all).unwrap();
    let cold = progress::load(&restore, &all).unwrap();
    assert_eq!(value(&cold.attempts), value(&p.attempts));
    let a = &cold.attempts[&s.id];
    assert!(progress::current_solved(a, &s));
    assert_eq!(session(a).example, definition().lesson.fixture);
    assert_eq!(
        session(a).assignment_editor.drafts["/dev/vda"].start,
        "unfinished"
    );
    // Reset is a fresh supplied attempt; it never silently edits existing content.
    let reset = Attempt::new(&s, a.visited);
    assert!(!progress::current_solved(&reset, &s));
    assert_eq!(session(&reset).current, definition().initial);
    assert_eq!(value(&cold.attempts), value(&p.attempts));
}
#[test]
fn every_nonselected_attempt_and_held_source_is_checked_before_any_write_or_export() {
    let s = scenario();
    let mut second = s.clone();
    second.id = "private-other-disk-task".into();
    let all = vec![s.clone(), second.clone()];
    let mut good = profile(&s);
    let mut other = Attempt::new(&second, 2);
    work::binding::prepare(&mut other, &second).unwrap();
    good.attempts.insert(second.id.clone(), other);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &good, &all).unwrap();
    let original = files(dir.path());
    for mode in 0..7 {
        let mut p: Progress = serde_json::from_value(value(&good)).unwrap();
        let mut held = all.clone();
        let a = p.attempts.get_mut(&second.id).unwrap();
        match mode {
            0 => {
                held.pop();
            }
            1 => {
                a.state = s.initial.clone();
                a.scenario_version += 1;
            }
            2 => {
                a.state = pack::bundled().unwrap()[0].initial.clone();
            }
            3 => {
                a.workspace.as_mut().unwrap().command = "foreign generic terminal".into();
            }
            4 => {
                a.workspace.as_mut().unwrap().checked = true;
            }
            5 => {
                let State::PartitionFs { simulation } = &mut a.state else {
                    panic!()
                };
                simulation.origin.plan.targets[0].minimum_bytes = 4 * MIB;
                simulation.validate().unwrap();
                let Goal::PartitionFs { definition } = &mut held[1].goal else {
                    panic!()
                };
                definition.plan.targets[0].minimum_bytes = 4 * MIB; // A carried Goal alone does not update the trusted initial source.
            }
            _ => {
                let State::PartitionFs { simulation } = &mut a.state else {
                    panic!()
                };
                simulation
                    .current
                    .disks
                    .get_mut("/dev/vdb")
                    .unwrap()
                    .sectors += 2048;
                simulation.current.validate().unwrap();
            }
        }
        let before = value(&p);
        let receipts = value(&p.migration_receipts);
        assert!(progress::save_with_catalog(&path, &p, &held).is_err());
        assert_eq!(files(dir.path()), original);
        assert_eq!(value(&p), before);
        assert_eq!(value(&p.migration_receipts), receipts);
        assert!(backup::export(&dir.path().join("refused-export.json"), &p, &held).is_err());
        assert_eq!(files(dir.path()), original);
        assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &held).is_err());
    }
    assert!(progress::save(&path, &good).is_err());
    assert_eq!(files(dir.path()), original);
}
#[test]
fn jointly_forged_witness_and_initial_and_goal_cannot_replace_application_catalog() {
    let s = scenario();
    let mut p = profile(&s);
    for op in operations("/dev/vda", "reports", "paging", 8, 2, "reports") {
        work::binding::apply(p.attempts.get_mut(&s.id).unwrap(), &s, &op, false).unwrap();
    }
    let mut carried = s.clone();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let State::PartitionFs { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin.plan.targets[0].minimum_bytes = 4 * MIB;
    let State::PartitionFs { simulation } = &mut carried.initial else {
        panic!()
    };
    simulation.origin.plan.targets[0].minimum_bytes = 4 * MIB;
    let Goal::PartitionFs { definition } = &mut carried.goal else {
        panic!()
    };
    definition.plan.targets[0].minimum_bytes = 4 * MIB;
    assert!(progress::current_solved(a, &carried));
    assert!(!progress::current_solved(a, &s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let original = profile(&s);
    progress::save_with_catalog(&path, &original, std::slice::from_ref(&s)).unwrap();
    let bytes = files(dir.path());
    let before = value(&p);
    assert!(progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).is_err());
    assert_eq!(files(dir.path()), bytes);
    assert_eq!(value(&p), before);
    // An untrusted portable profile contains no Scenario catalog replacement.
    let envelope = serde_json::json!({"format":"dlms-learning-lab-backup","version":1,"app_version":env!("CARGO_PKG_VERSION"),"progress":p});
    let forged = dir.path().join("forged.json");
    std::fs::write(&forged, serde_json::to_vec(&envelope).unwrap()).unwrap();
    assert!(
        backup::inspect(
            &forged,
            std::slice::from_ref(&s),
            &std::sync::atomic::AtomicBool::new(false),
            &std::sync::atomic::AtomicUsize::new(0)
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes["progress.json"]);
}
#[test]
fn escaped_model_and_context_budgets_and_aggregate16mb_refusal_retain_original_bytes() {
    let s = scenario();
    let mut p = profile(&s);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let old = files(dir.path());
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.transcript = vec!["\"".repeat(8192); 120];
    assert!(a.transcript.iter().all(|s| s.len() <= 8192));
    let before = value(&p);
    assert!(progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).is_err());
    assert_eq!(files(dir.path()), old);
    assert_eq!(value(&p), before);
    // Individually valid near-bound sessions need not all fit the unchanged global cap.
    let mut huge = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut sources = Vec::new();
    for n in 0..17 {
        let mut source = s.clone();
        source.id = format!("private-bounded-{n}");
        let mut a = Attempt::new(&source, 1);
        work::binding::prepare(&mut a, &source).unwrap();
        a.transcript = vec!["x".repeat(8100); 120];
        work::binding::context(&a, &source).unwrap();
        huge.attempts.insert(source.id.clone(), a);
        sources.push(source);
    }
    assert!(serde_json::to_vec(&huge).unwrap().len() > 16_000_000);
    let caller = value(&huge);
    assert!(
        progress::save_with_catalog(&path, &huge, &sources)
            .unwrap_err()
            .contains("16")
    );
    assert_eq!(files(dir.path()), old);
    assert_eq!(value(&huge), caller);
    let mut bad = definition();
    let t = bad
        .initial
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap();
    let dlms_learning_lab::partition_fs::Content::Ext4 { files, .. } = &mut t.partitions[0].content
    else {
        panic!()
    };
    files.clear();
    for n in 0..3 {
        files.insert(
            format!("/escaped-{n}"),
            dlms_learning_lab::partition_fs::File {
                text: "\"".repeat(16000),
            },
        );
    }
    bad.initial.validate().unwrap();
    assert!(bad.shape().is_err());
}
#[test]
fn invalid_packs_and_duplicate_or_excess_drafts_refuse_without_inventing_old_progress() {
    let s = scenario();
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[&s]}).to_string()).is_ok()
    );
    let mut impossible = value(&s);
    impossible["goal"]["definition"]["reference"][2]["label"] = serde_json::json!("wrong_label");
    let error =
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[impossible]}).to_string())
            .unwrap_err();
    assert!(error.contains("reference") && error.contains("assignment"));
    let mut v = value(&s);
    v["goal"]["definition"]["plan"]["targets"][0]["minimum_bytes"] = serde_json::json!(0);
    let wrap = |v: serde_json::Value| serde_json::json!({"schema_version":1,"scenarios":[v]});
    assert!(pack::load(&wrap(v).to_string()).is_err());
    let mut v = value(&s);
    v["initial"]["simulation"]["example"] = v["initial"]["simulation"]["current"].clone();
    assert!(pack::load(&wrap(v).to_string()).is_err());
    let State::PartitionFs { simulation } = &s.initial else {
        panic!()
    };
    let raw = serde_json::to_string(simulation).unwrap();
    let draft = serde_json::to_string(&simulation.assignment_editor.drafts["/dev/vda"]).unwrap();
    let needle = format!("\"/dev/vda\":{draft}");
    assert!(raw.contains(&needle));
    let duplicate = raw.replacen(&needle, &format!("{needle},{needle}"), 1);
    assert!(serde_json::from_str::<work::Session>(&duplicate).is_err());
    let mut oversized = Vec::new();
    for n in 0..12 {
        let mut next = s.clone();
        next.id = format!("private-pack-bound-{n}");
        next.solution = vec!["z".repeat(4000); 50];
        oversized.push(next);
    }
    let too_large =
        serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":oversized}))
            .unwrap();
    assert!(too_large.len() > pack::MAX_PACK_BYTES);
    assert!(pack::load(&too_large).unwrap_err().contains("2 MB"));
    let old = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let mut all = pack::bundled().unwrap();
    all.push(s.clone());
    let decoded = progress::decode(old, &all).unwrap();
    let original: Progress = serde_json::from_slice(old).unwrap();
    assert_eq!(value(&decoded.attempts), value(&original.attempts));
    assert!(!decoded.attempts.contains_key(&s.id));
    assert!(!decoded.favorites.contains(&s.id));
    assert!(!progress::current_solved(&Attempt::new(&s, 0), &s));
}
