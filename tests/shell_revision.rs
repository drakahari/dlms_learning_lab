use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell, workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn earlier(lab: &pack::Scenario, unsupported: bool) -> Attempt {
    let Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let original = &lab.solution[0];
    let source = if unsupported {
        format!("IFS=,\n{original}")
    } else {
        original.clone()
    };
    let mut a = Attempt::new(lab, 1);
    a.state = State::Script {
        source: source.clone(),
        run: Some(shell::Record {
            engine_revision: 0,
            source,
            results: cases.iter().map(|c| shell::run(original, c)).collect(),
        }),
    };
    a
}
#[test]
fn old_results_retain_earlier_credit_and_drafts_but_cannot_award_current_success() {
    let labs = pack::bundled().unwrap();
    let lab = &labs[9];
    for unsupported in [false, true] {
        let mut a = earlier(lab, unsupported);
        let old = a.state.clone();
        assert!(!progress::current_solved(&a, lab));
        assert!(progress::legacy_script_credit(&a, lab));
        workspace::prepare(&mut a);
        let State::Script { source, run } = &mut a.state else {
            panic!()
        };
        let State::Script {
            source: old_source,
            run: old_record,
        } = &old
        else {
            panic!()
        };
        assert_eq!(source, old_source);
        assert!(run.is_none());
        assert_eq!(
            a.workspace.as_ref().unwrap().script_history.as_ref(),
            old_record.as_ref()
        );
        let Goal::Script { cases } = &lab.goal else {
            panic!()
        };
        *run = Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(source, c)).collect(),
        });
        assert_eq!(progress::current_solved(&a, lab), !unsupported);
        assert!(progress::legacy_script_credit(&a, lab));
        let State::Script { source, run } = &mut a.state else {
            panic!()
        };
        *source = "printf retry".into();
        *run = None;
        assert!(!progress::current_solved(&a, lab));
        assert!(progress::legacy_script_credit(&a, lab));
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(lab.id.clone(), a);
        let encoded = serde_json::to_vec(&p).unwrap();
        let decoded = progress::decode(&encoded, &labs).unwrap();
        assert_eq!(serde_json::to_vec(&decoded).unwrap(), encoded);
    }
}
#[test]
fn omitted_revision_is_legacy_and_future_revision_is_rejected_without_rewriting() {
    let labs = pack::bundled().unwrap();
    let lab = &labs[9];
    let a = earlier(lab, false);
    let json = serde_json::to_value(&a).unwrap();
    assert!(json["state"]["run"].get("engine_revision").is_none());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    let encoded = serde_json::to_vec(&p).unwrap();
    let decoded = progress::decode(&encoded, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), encoded);
    let mut bad = serde_json::to_value(&p).unwrap();
    bad["attempts"][&lab.id]["state"]["run"]["engine_revision"] =
        serde_json::json!(shell::EVALUATOR_REVISION + 1);
    assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &labs).is_err());
}
#[test]
fn restore_rechecks_with_current_engine_while_preserving_earlier_output_and_credit() {
    let labs = pack::bundled().unwrap();
    let lab = &labs[9];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), earlier(lab, true));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    backup::export(&path, &p, &labs).unwrap();
    let candidate =
        backup::inspect(&path, &labs, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let a = &candidate.progress.attempts[&lab.id];
    assert_eq!(candidate.rechecked, 1);
    assert!(!progress::current_solved(a, lab));
    assert!(progress::legacy_script_credit(a, lab));
    assert!(a.workspace.as_ref().unwrap().script_history.is_some());
    let State::Script {
        run: Some(record), ..
    } = &a.state
    else {
        panic!()
    };
    assert!(record.current());
    assert!(record.results.iter().all(|r| r.error.is_some()));
}

#[test]
fn first_evaluator_upgrade_write_keeps_exact_backup_and_never_replaces_it() {
    let labs = pack::bundled().unwrap();
    let lab = &labs[9];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), earlier(lab, true));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let original = serde_json::to_vec_pretty(&p).unwrap();
    std::fs::write(&path, &original).unwrap();
    workspace::prepare(p.attempts.get_mut(&lab.id).unwrap());
    progress::save(&path, &p).unwrap();
    let backup = path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION));
    assert_eq!(std::fs::read(&backup).unwrap(), original);
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(&backup).unwrap(), original);
}

#[test]
fn occupied_migration_slots_cannot_silently_replace_the_exact_backup() {
    let labs = pack::bundled().unwrap();
    let lab = &labs[9];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), earlier(lab, true));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let original = serde_json::to_vec_pretty(&p).unwrap();
    std::fs::write(&path, &original).unwrap();
    workspace::prepare(p.attempts.get_mut(&lab.id).unwrap());
    let slot = path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION));
    std::fs::create_dir(&slot).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    std::fs::remove_dir(&slot).unwrap();
    std::fs::write(&slot, b"unrelated existing data").unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read(&slot).unwrap(), b"unrelated existing data");
}

#[test]
fn reset_as_first_upgrade_action_still_validates_and_preserves_migration_backup() {
    let labs = pack::bundled().unwrap();
    let lab = &labs[9];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), earlier(lab, true));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let original = serde_json::to_vec_pretty(&p).unwrap();
    std::fs::write(&path, &original).unwrap();
    // Both slots already exist as ordinary files: neither existence nor in-memory
    // history can substitute for checking the profile about to be replaced.
    std::fs::write(path.with_extension("before-workspace.json"), &original).unwrap();
    let slot = path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION));
    std::fs::write(&slot, b"unrelated existing data").unwrap();
    p.attempts.insert(lab.id.clone(), Attempt::new(lab, 2));
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read(&slot).unwrap(), b"unrelated existing data");
    std::fs::remove_file(&slot).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(&slot).unwrap(), original);
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(&slot).unwrap(), original);
}
