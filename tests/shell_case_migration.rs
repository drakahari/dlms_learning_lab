//! Current-revision migration checks from genuine frozen7/6 snapshots.
//! Original8 test source is retained in ignored review evidence; old records stay exact.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    shell, workspace,
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize},
};
const ORIGINAL: &[u8] = include_bytes!("fixtures/shell-v7-synthetic-progress-original.json");
const WRITER7: &[u8] = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
#[derive(Deserialize)]
struct Old {
    id: String,
    scenario_version: u32,
    cases: Vec<shell::Case>,
    record: shell::Record,
}
fn records() -> Vec<Old> {
    include_str!("fixtures/shell-v7-authored-records.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn labs() -> Vec<Scenario> {
    const {
        assert!(
            shell::EVALUATOR_REVISION >= 8,
            "current migration never relabels captured7"
        );
    }
    pack::bundled().unwrap()
}
fn empty() -> Progress {
    Progress {
        schema_version: 1,
        ..Default::default()
    }
}
fn snapshot(p: &Progress) -> serde_json::Value {
    serde_json::to_value(p).unwrap()
}
fn profile(labs: &[Scenario]) -> Progress {
    progress::decode(ORIGINAL, labs).unwrap()
}
fn slot(path: &Path) -> std::path::PathBuf {
    path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION))
}
fn inspect(path: &Path, labs: &[Scenario]) -> Result<backup::Candidate, String> {
    backup::inspect(path, labs, &AtomicBool::new(false), &AtomicUsize::new(0))
}
fn assert_contexts(p: &Progress) {
    let before: serde_json::Value = serde_json::from_slice(ORIGINAL).unwrap();
    assert_eq!(p.attempts.len(), 102);
    assert_eq!(snapshot(p)["favorites"], before["favorites"]);
    for (id, a) in &p.attempts {
        let now = serde_json::to_value(a).unwrap();
        let old = &before["attempts"][id];
        if matches!(a.state, State::Script { .. }) {
            assert_eq!(
                now["state"]["source"], old["state"]["source"],
                "assignment draft {id}"
            );
            for field in [
                "scenario_version",
                "step",
                "challenge",
                "hints",
                "transcript",
                "visited",
            ] {
                assert_eq!(now[field], old[field], "{id}/{field}");
            }
            assert_eq!(now["workspace"]["command"], old["workspace"]["command"]);
            assert_eq!(
                now["workspace"]["script_learning"]["example_source"],
                old["workspace"]["script_learning"]["example_source"],
                "separate example draft {id}"
            );
        } else {
            assert_eq!(&now, old, "unrelated context {id}");
        }
    }
}
fn assert_histories(p: &Progress, prepared: bool) {
    let before: Progress = serde_json::from_slice(ORIGINAL).unwrap();
    let mut six = 0;
    let mut seven = 0;
    for old in records() {
        assert_eq!(old.record.engine_revision, 7);
        let a = &p.attempts[&old.id];
        let w = a.workspace.as_ref().unwrap();
        let original = before.attempts[&old.id].workspace.as_ref().unwrap();
        let mut expected_first = original.script_history.clone();
        let mut expected_more = original.script_history_more.clone();
        if prepared {
            if expected_first.is_none() {
                expected_first = Some(old.record.clone());
            } else if !expected_first
                .as_ref()
                .into_iter()
                .chain(&expected_more)
                .any(|r| r == &old.record)
            {
                expected_more.push(old.record.clone());
            }
        } else {
            let State::Script { run: Some(r), .. } = &a.state else {
                panic!()
            };
            assert_eq!(r, &old.record);
        }
        assert_eq!(
            w.script_history, expected_first,
            "exact first historical slot {}",
            old.id
        );
        assert_eq!(
            w.script_history_more, expected_more,
            "exact additional historical slots {}",
            old.id
        );
        six += w
            .script_history
            .as_ref()
            .into_iter()
            .chain(&w.script_history_more)
            .filter(|r| r.engine_revision == 6)
            .count();
        seven += w
            .script_history
            .as_ref()
            .into_iter()
            .chain(&w.script_history_more)
            .filter(|r| r.engine_revision == 7)
            .count();
    }
    assert_eq!(six, 33);
    assert_eq!(seven, if prepared { 37 } else { 0 });
}

#[test]
fn genuine7_capture_all102_contexts_and_6_7_records_load_without_execution_or_new_credit() {
    let labs = labs();
    let mut p = profile(&labs);
    assert_contexts(&p);
    assert_histories(&p, false);
    assert_eq!(records().len(), 37);
    for old in records() {
        let s = labs.iter().find(|s| s.id == old.id).unwrap();
        assert_eq!(s.version, old.scenario_version);
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(
            serde_json::to_value(cases).unwrap(),
            serde_json::to_value(&old.cases).unwrap()
        );
        let a = p.attempts.get_mut(&old.id).unwrap();
        assert!(!progress::current_solved(a, s));
        assert!(progress::legacy_script_credit(a, s));
        workspace::prepare(a);
        assert!(!progress::current_solved(a, s));
    }
    assert_contexts(&p);
    assert_histories(&p, true);
    for a in p
        .attempts
        .values()
        .filter(|a| matches!(a.state, State::Script { .. }))
    {
        let State::Script { run, .. } = &a.state else {
            unreachable!()
        };
        assert!(run.is_none());
    }
}
#[test]
fn exact_actual_writer7_bytes_are_backed_up_once_before_current_and_repeat_saves_keep_history() {
    let labs = labs();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, WRITER7).unwrap();
    let mut p = progress::load(&path, &labs).unwrap();
    assert_eq!(p.migration_receipts.as_ref().unwrap().shell, vec![7]);
    for a in p
        .attempts
        .values_mut()
        .filter(|a| matches!(a.state, State::Script { .. }))
    {
        workspace::prepare(a);
    }
    let caller = serde_json::to_value(p.migration_receipts.as_ref().unwrap()).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), WRITER7);
    assert_eq!(
        serde_json::to_value(p.migration_receipts.as_ref().unwrap()).unwrap(),
        caller
    );
    let read = progress::load(&path, &labs).unwrap();
    assert_eq!(
        read.migration_receipts.as_ref().unwrap().shell,
        vec![7, shell::EVALUATOR_REVISION]
    );
    assert_contexts(&read);
    assert_histories(&read, true);
    progress::save(&path, &read).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), WRITER7);
}
#[test]
fn reset_first_and_practice_only7_cannot_skip_exact_target_backup() {
    let labs = labs();
    for practice_only in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        let raw = if practice_only {
            let mut p = profile(&labs);
            p.attempts.retain(|id, _| id == "shell-arguments");
            let a = p.attempts.get_mut("shell-arguments").unwrap();
            let State::Script { run, .. } = &mut a.state else {
                panic!()
            };
            *run = None;
            a.workspace.as_mut().unwrap().script_history = None;
            let mut value = snapshot(&p);
            value["migration_receipts"] = serde_json::json!({"workspace":true,"shell":[7]});
            serde_json::to_vec_pretty(&value).unwrap()
        } else {
            WRITER7.to_vec()
        };
        std::fs::write(&path, &raw).unwrap();
        progress::save(&path, &empty()).unwrap();
        assert_eq!(std::fs::read(slot(&path)).unwrap(), raw);
        let read = progress::load(&path, &labs).unwrap();
        assert_eq!(
            read.migration_receipts.as_ref().unwrap().shell,
            vec![7, shell::EVALUATOR_REVISION]
        );
    }
}
#[test]
fn valid_different_and_whitespace_only_occupied_before_current_slots_block_without_writes() {
    let labs = labs();
    let mut whitespace_only = WRITER7.to_vec();
    whitespace_only.push(b'\n');
    assert_ne!(whitespace_only, WRITER7);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&whitespace_only).unwrap(),
        serde_json::from_slice::<serde_json::Value>(WRITER7).unwrap()
    );
    for occupied in [
        serde_json::to_vec(&empty()).unwrap(),
        whitespace_only,
        WRITER7.to_vec(),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        std::fs::write(&path, WRITER7).unwrap();
        std::fs::write(slot(&path), &occupied).unwrap();
        let p = profile(&labs);
        let before = snapshot(&p);
        let result = progress::save(&path, &p);
        assert_eq!(result.is_ok(), occupied == WRITER7);
        assert_eq!(snapshot(&p), before);
        assert_eq!(std::fs::read(slot(&path)).unwrap(), occupied);
        if result.is_err() {
            assert_eq!(std::fs::read(&path).unwrap(), WRITER7);
        }
    }
}
#[test]
fn forged_candidate_receipts_never_authorize_skipping_actual_old7_target() {
    let labs = labs();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, WRITER7).unwrap();
    let mut p = empty();
    p.migration_receipts = Some(progress::MigrationReceipts {
        workspace: true,
        shell: vec![shell::EVALUATOR_REVISION],
    });
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), WRITER7);
    assert_eq!(
        progress::load(&path, &labs)
            .unwrap()
            .migration_receipts
            .unwrap()
            .shell,
        vec![7, shell::EVALUATOR_REVISION]
    );
}
#[test]
fn explicit_import_recheck_adds_real_current8_only_from_edited_sources_and_retains_6_7() {
    let labs = labs();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    std::fs::write(&input, WRITER7).unwrap();
    let candidate = inspect(&input, &labs).unwrap();
    assert_eq!(candidate.rechecked, 74);
    assert!(candidate.progress.migration_receipts.is_none());
    assert_contexts(&candidate.progress);
    assert_histories(&candidate.progress, true);
    for old in records() {
        let a = &candidate.progress.attempts[&old.id];
        let State::Script {
            source,
            run: Some(run),
        } = &a.state
        else {
            panic!()
        };
        assert_eq!(run.engine_revision, shell::EVALUATOR_REVISION);
        assert_eq!(&run.source, source);
    }
    for id in ["shell-handover", "shell-capture-handover"] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        assert!(
            !progress::current_solved(&candidate.progress.attempts[id], s),
            "unfinished source must not earn current credit"
        );
    }
    let target = dir.path().join("target.json");
    std::fs::write(&target, WRITER7).unwrap();
    let result = backup::restore(&target, &candidate, &labs).unwrap();
    assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(), WRITER7);
    assert_eq!(std::fs::read(slot(&target)).unwrap(), WRITER7);
    let p = progress::load(&target, &labs).unwrap();
    assert_contexts(&p);
    assert_histories(&p, true);
    let current_bytes = std::fs::read(&target).unwrap();
    let again = backup::restore(&target, &candidate, &labs).unwrap();
    assert_eq!(
        std::fs::read(again.safety_copy.unwrap()).unwrap(),
        current_bytes
    );
    assert_eq!(std::fs::read(slot(&target)).unwrap(), WRITER7);
    assert_eq!(
        progress::load(&target, &labs)
            .unwrap()
            .migration_receipts
            .unwrap()
            .shell,
        vec![7, shell::EVALUATOR_REVISION]
    );
    let new_target = dir.path().join("new.json");
    let created = backup::restore(&new_target, &candidate, &labs).unwrap();
    assert!(created.safety_copy.is_none());
    assert!(!slot(&new_target).exists());
    let new_read = progress::load(&new_target, &labs).unwrap();
    assert_eq!(
        new_read.migration_receipts.as_ref().unwrap().shell,
        vec![shell::EVALUATOR_REVISION]
    );
    assert_contexts(&new_read);
    assert_histories(&new_read, true);
    let output = dir.path().join("portable.json");
    backup::export(&output, &p, &labs).unwrap();
    let envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert!(envelope["progress"].get("migration_receipts").is_none());
}
#[test]
fn cancelled_preview_and_restore_conflict_leave_target_input_and_receipts_exact() {
    let labs = labs();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    std::fs::write(&input, WRITER7).unwrap();
    let target = dir.path().join("target.json");
    std::fs::write(&target, WRITER7).unwrap();
    assert!(backup::inspect(&input, &labs, &AtomicBool::new(true), &AtomicUsize::new(0)).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), WRITER7);
    assert!(!slot(&target).exists());
    let candidate = backup::Candidate {
        progress: empty(),
        rechecked: 0,
    };
    let occupied = serde_json::to_vec(&empty()).unwrap();
    std::fs::write(slot(&target), &occupied).unwrap();
    assert!(backup::restore(&target, &candidate, &labs).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), WRITER7);
    assert_eq!(std::fs::read(&input).unwrap(), WRITER7);
    assert_eq!(std::fs::read(slot(&target)).unwrap(), occupied);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}
#[test]
fn confirmed_raw_recovery_does_not_misclassify_corrupt_future_bytes_as_old7() {
    let labs = labs();
    for raw in [b"{ corrupt synthetic target }".to_vec(),br#"{"schema_version":99,"clock":0,"favorites":[],"attempts":{}}"#.to_vec(),serde_json::to_vec(&serde_json::json!({"schema_version":1,"clock":0,"favorites":[],"attempts":{},"migration_receipts":{"shell":[shell::EVALUATOR_REVISION + 1]}})).unwrap()] {
        let dir=tempfile::tempdir().unwrap();let target=dir.path().join("progress.json");std::fs::write(&target,&raw).unwrap();std::fs::write(slot(&target),b"unrelated fixed slot").unwrap();let result=backup::restore(&target,&backup::Candidate{progress:empty(),rechecked:0},&labs).unwrap();assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(),raw);assert_eq!(std::fs::read(slot(&target)).unwrap(),b"unrelated fixed slot");assert_eq!(progress::load(&target,&labs).unwrap().migration_receipts.unwrap().shell,vec![shell::EVALUATOR_REVISION]);
    }
}
#[test]
fn genuine_history_capacity_allows_identical32_but_rejects_distinct33_without_loss() {
    let labs = labs();
    let old = records();
    let s = labs.iter().find(|s| s.id == old[0].id).unwrap();
    let mut a = Attempt::new(s, 1);
    let w = a
        .workspace
        .get_or_insert_with(workspace::WorkspaceData::new);
    // Synthetic capacity pool assembled from genuine frozen observations across
    // activities; this is not a captured learner's chronological history.
    let original = profile(&labs);
    w.script_history = original.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .script_history
        .clone();
    assert_eq!(w.script_history.as_ref().unwrap().engine_revision, 6);
    w.script_history_more = old[..30].iter().map(|r| r.record.clone()).collect();
    a.state = State::Script {
        source: old[30].record.source.clone(),
        run: Some(old[30].record.clone()),
    };
    assert!(workspace::can_preserve_script_history(&a));
    workspace::prepare(&mut a);
    assert_eq!(a.workspace.as_ref().unwrap().script_history_more.len(), 31);
    // At combined32, reuse a genuinely identical saved7 run without duplication.

    a.state = State::Script {
        source: old[0].record.source.clone(),
        run: Some(old[0].record.clone()),
    };
    assert!(workspace::can_preserve_script_history(&a));
    workspace::prepare(&mut a);
    let w = a.workspace.as_ref().unwrap();
    assert_eq!(w.script_history_more.len(), 31);
    a.state = State::Script {
        source: old[32].record.source.clone(),
        run: Some(old[32].record.clone()),
    };
    assert!(!workspace::can_preserve_script_history(&a));
    let mut p = empty();
    p.attempts.insert(s.id.clone(), a);
    let bytes = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&bytes, &labs).is_err());
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("progress.json");
    std::fs::write(&target, WRITER7).unwrap();
    assert!(progress::save(&target, &p).is_err());
    assert_eq!(std::fs::read(target).unwrap(), WRITER7);
}
#[test]
fn explicit_assignment8_rerun_preserves_history_then_selected_reset_affects_only_one_attempt() {
    let labs = labs();
    let mut p = profile(&labs);
    for a in p
        .attempts
        .values_mut()
        .filter(|a| matches!(a.state, State::Script { .. }))
    {
        workspace::prepare(a);
    }
    let s = labs.iter().find(|s| s.id == "shell-handover").unwrap();
    let before_other = p
        .attempts
        .iter()
        .filter(|(id, _)| *id != &s.id)
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect::<BTreeMap<_, _>>();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let State::Script { source, run } = &mut a.state else {
        panic!()
    };
    *source = s.solution[0].clone();
    *run = Some(shell::Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.clone(),
        results: cases.iter().map(|c| shell::run(source, c)).collect(),
    });
    assert!(progress::current_solved(a, s));
    assert!(progress::legacy_script_credit(a, s));
    let earlier = a.workspace.as_ref().unwrap().script_history_more.clone();
    assert!(earlier.iter().any(|r| r.engine_revision == 7));
    let State::Script { source, run } = &mut a.state else {
        panic!()
    };
    source.push_str("\nprintf broken");
    *run = None;
    assert!(!progress::current_solved(a, s));
    assert_eq!(a.workspace.as_ref().unwrap().script_history_more, earlier);
    p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("progress.json");
    std::fs::write(&target, WRITER7).unwrap();
    progress::save(&target, &p).unwrap();
    assert_eq!(std::fs::read(slot(&target)).unwrap(), WRITER7);
    let read = progress::load(&target, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&read.attempts[&s.id]).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap(),
        "selected reset is exact after reload"
    );
    for (id, before) in before_other {
        assert_eq!(serde_json::to_value(&p.attempts[&id]).unwrap(), before);
        assert_eq!(serde_json::to_value(&read.attempts[&id]).unwrap(), before);
    }
    assert!(!progress::current_solved(&p.attempts[&s.id], s));
}

#[test]
fn malformed_future_histories_or_receipts_cannot_change_original_target_bytes() {
    let labs = labs();
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("progress.json");
    std::fs::write(&target, WRITER7).unwrap();
    for future in [false, true] {
        let mut value: serde_json::Value = serde_json::from_slice(WRITER7).unwrap();
        let history = &mut value["attempts"]["shell-handover"]["workspace"]["script_history"];
        if future {
            history["engine_revision"] = serde_json::json!(shell::EVALUATOR_REVISION + 1);
        } else {
            history["results"] = serde_json::json!("malformed results, not observations");
        }
        let input = dir.path().join("invalid-history.json");
        std::fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(inspect(&input, &labs).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), WRITER7);
        assert!(!slot(&target).exists());
    }
    for receipts in [
        serde_json::json!({"shell":[shell::EVALUATOR_REVISION + 1]}),
        serde_json::json!({"shell":[8,7]}),
        serde_json::json!({"shell":[7,7]}),
        serde_json::json!({"shell":vec![1;33]}),
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(WRITER7).unwrap();
        value["migration_receipts"] = receipts;
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(progress::decode(&bytes, &labs).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), WRITER7);
    }
}
