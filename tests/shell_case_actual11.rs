//! Explicit private11→12 comparisons. Reads genuine sealed before-change data;
//! creates only named current executions in disposable project-disk fixtures.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};
#[path = "support/actual11_evidence.rs"]
mod evidence;
fn value(v: &impl serde::Serialize) -> Value {
    serde_json::to_value(v).unwrap()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authored {
    id: String,
    source: String,
    cases: Vec<Case>,
    record: Record,
    example_source: String,
    example_case: Case,
    example_outcome: Outcome,
    example_revision: u32,
}
fn authored() -> Vec<Authored> {
    evidence::observed("actual11-authored.jsonl")
        .split(|b| *b == b'\n')
        .filter(|r| !r.is_empty())
        .map(|r| serde_json::from_slice(r).unwrap())
        .collect()
}
fn sliced(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Finite current12 comparison exceeded tick bound")
}
fn histories(a: &Attempt, primary: bool) -> Vec<Record> {
    let mut rows = a
        .workspace
        .as_ref()
        .map(|w| {
            w.script_history
                .iter()
                .chain(&w.script_history_more)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if primary
        && let State::Script { run: Some(r), .. } = &a.state
        && !rows.contains(r)
    {
        rows.push(r.clone());
    }
    rows
}
fn all_histories(p: &Progress) -> BTreeMap<String, Vec<Record>> {
    p.attempts
        .iter()
        .map(|(id, a)| (id.clone(), histories(a, true)))
        .collect()
}
fn same_old_metadata(mut a: Value, mut b: Value) {
    for v in [&mut a, &mut b] {
        v.as_object_mut().unwrap().remove("state");
        let w = v["workspace"].as_object_mut().unwrap();
        for k in ["checked", "script_history", "script_history_more"] {
            w.remove(k);
        }
    }
    assert_eq!(a, b);
}

#[test]
#[ignore = "Requires immutable private actual11 COMPLETE and attestation; explicit guarded replay"]
fn all52_genuine11_records384_cases52_examples_remain_exact_at_four_slices_under12() {
    assert_eq!(shell::EVALUATOR_REVISION, 12);
    let all = pack::bundled().unwrap();
    let rows = authored();
    assert_eq!(rows.len(), 52);
    let mut ids = BTreeSet::new();
    let mut count = 0;
    for row in rows {
        assert!(ids.insert(row.id.clone()));
        let s = all.iter().find(|s| s.id == row.id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(value(cases), value(&row.cases));
        assert_eq!(s.solution.join("\n"), row.source);
        assert_eq!(row.record.engine_revision, 11);
        assert!(!row.record.current());
        assert!(row.record.valid());
        assert_eq!(row.record.source, row.source);
        assert_eq!(row.record.results.len(), cases.len());
        for (case, old) in cases.iter().zip(&row.record.results) {
            assert!(case.accepts(old));
            for slice in [1, 7, 32, 64] {
                assert_eq!(
                    sliced(&row.source, case, slice),
                    *old,
                    "{}/{} slice{slice}",
                    row.id,
                    case.name
                );
            }
        }
        count += cases.len();
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(lesson.example, row.example_source);
        assert_eq!(value(&lesson.fixture), value(&row.example_case));
        assert_eq!(row.example_revision, 11);
        assert!(row.example_case.accepts(&row.example_outcome));
        for slice in [1, 7, 32, 64] {
            assert_eq!(
                sliced(&row.example_source, &row.example_case, slice),
                row.example_outcome
            );
        }
    }
    assert_eq!(count, 384);
}

#[test]
#[ignore = "Requires immutable private actual11 COMPLETE and attestation; explicit guarded replay"]
fn actual142_contexts_keep_all6_through11_records_metadata_raw_backup_and_receipts_until_explicit_run()
 {
    assert_eq!(shell::EVALUATOR_REVISION, 12);
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer11-original.json");
    let original: Progress = serde_json::from_slice(&raw).unwrap();
    let before_histories = all_histories(&original);
    let mut p = progress::decode(&raw, &all).unwrap();
    assert_eq!(value(&p), value(&original));
    assert_eq!(p.attempts.len(), 142);
    assert_eq!(
        p.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 11]
    );
    let mut scripts = 0;
    let mut non_scripts = 0;
    for (id, a) in &mut p.attempts {
        let before = &original.attempts[id];
        let s = all.iter().find(|s| &s.id == id).unwrap();
        if let State::Script { run: Some(r), .. } = &before.state {
            scripts += 1;
            assert_eq!(r.engine_revision, 11);
            assert!(!progress::current_solved(a, s));
            workspace::prepare(a);
            assert!(matches!(a.state, State::Script { run: None, .. }));
            assert_eq!(histories(a, false), before_histories[id]);
            same_old_metadata(value(a), value(before));
            assert!(!progress::current_solved(a, s));
        } else {
            non_scripts += 1;
            assert_eq!(value(a), value(before));
        }
    }
    assert_eq!((scripts, non_scripts), (52, 90));
    assert_eq!(all_histories(&p), before_histories);
    assert_eq!(p.favorites, original.favorites);
    for s in &all {
        if !original.attempts.contains_key(&s.id) {
            assert!(
                !p.attempts.contains_key(&s.id),
                "Never invent new-catalog answers: {}",
                s.id
            );
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    let caller_receipts = value(&p.migration_receipts);
    progress::save(&path, &p).unwrap();
    assert_eq!(value(&p.migration_receipts), caller_receipts);
    let slot = path.with_extension("before-shell-v12.json");
    assert_eq!(std::fs::read(&slot).unwrap(), raw);
    let mut resumed = progress::load(&path, &all).unwrap();
    assert_eq!(value(&resumed), value(&p));
    assert_eq!(
        resumed.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 11, 12]
    );
    let s = all.iter().find(|s| s.id == "shell-handover").unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let a = resumed.attempts.get_mut(&s.id).unwrap();
    let source = s.solution.join("\n");
    a.state = State::Script {
        source: source.clone(),
        run: Some(Record {
            engine_revision: 12,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(&source, c)).collect(),
        }),
    };
    assert!(progress::current_solved(a, s));
    assert_eq!(histories(a, false), before_histories[&s.id]);
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("printf wrong\n");
    assert!(!progress::current_solved(a, s));
    let State::Script { source, run } = &mut a.state else {
        panic!()
    };
    *run = Some(Record {
        engine_revision: 12,
        source: source.clone(),
        results: cases.iter().map(|c| shell::run(source, c)).collect(),
    });
    assert!(!progress::current_solved(a, s));
    let before = value(&resumed);
    resumed
        .attempts
        .insert(s.id.clone(), Attempt::new(s, resumed.clock + 1));
    let after = value(&resumed);
    for (id, old) in before["attempts"].as_object().unwrap() {
        if id != &s.id {
            assert_eq!(&after["attempts"][id], old);
        }
    }
    assert_eq!(before["favorites"], after["favorites"]);
    progress::save(&path, &resumed).unwrap();
    assert_eq!(value(&progress::load(&path, &all).unwrap()), after);
    assert_eq!(std::fs::read(slot).unwrap(), raw);
}

#[test]
#[ignore = "Requires immutable private actual11 COMPLETE and attestation; explicit guarded replay"]
fn authentic_two_assignment_and_practice_drafts_stay_exact_and_only_saved_work_is_rechecked_on_restore()
 {
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer11-drafts.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    let histories_before = all_histories(&old);
    let dir = tempfile::tempdir().unwrap();
    let exported = dir.path().join("export.json");
    backup::export(&exported, &old, &all).unwrap();
    let target = dir.path().join("progress.json");
    std::fs::write(&target, &raw).unwrap();
    let cancelled = AtomicUsize::new(0);
    assert!(backup::inspect(&exported, &all, &AtomicBool::new(true), &cancelled).is_err());
    assert_eq!(cancelled.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(std::fs::read(&target).unwrap(), raw);
    let checked = AtomicUsize::new(0);
    let candidate = backup::inspect(&exported, &all, &AtomicBool::new(false), &checked).unwrap();
    assert_eq!(candidate.rechecked, 100);
    assert_eq!(checked.load(std::sync::atomic::Ordering::Relaxed), 100);
    assert!(candidate.progress.migration_receipts.is_none());
    let mut scripts = 0;
    let mut whole_drafts = 0;
    for (id, a) in &candidate.progress.attempts {
        let before = &old.attempts[id];
        let s = all.iter().find(|s| &s.id == id).unwrap();
        if ["shell-handover", "shell-command-plan"].contains(&id.as_str()) {
            whole_drafts += 1;
            assert_eq!(value(a), value(before));
            assert!(!progress::current_solved(a, s));
            continue;
        }
        if let State::Script {
            source,
            run: Some(r),
        } = &a.state
        {
            scripts += 1;
            assert_eq!(r.engine_revision, 12);
            assert_eq!(&r.source, source);
            assert!(progress::current_solved(a, s));
            assert_eq!(histories(a, false), histories_before[id]);
            let l = a
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap();
            let prior = before
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap();
            assert_eq!(l.example_source, prior.example_source);
            assert_eq!(l.view, prior.view);
            assert_eq!(l.example_revision, 12);
            assert_eq!(
                l.example_result,
                Some(shell::run(
                    &l.example_source,
                    &s.teaching.as_ref().unwrap().fixture
                ))
            );
        } else {
            assert_eq!(value(a), value(before));
        }
    }
    assert_eq!((scripts, whole_drafts), (50, 2));
    assert_eq!(
        std::fs::read(&target).unwrap(),
        raw,
        "Review never silently restores"
    );
    let restored = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), raw);
    assert_eq!(
        std::fs::read(target.with_extension("before-shell-v12.json")).unwrap(),
        raw
    );
    let loaded = progress::load(&target, &all).unwrap();
    assert_eq!(value(&loaded), value(&candidate.progress));
    assert_eq!(
        loaded.migration_receipts.unwrap().shell,
        vec![7, 8, 9, 10, 11, 12]
    );
}

#[test]
#[ignore = "Requires immutable private actual11 COMPLETE and attestation; explicit guarded replay"]
fn occupied12_backup_retains_actual11_main_raw_slot_and_unstamped_caller() {
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer11-original.json");
    let other = evidence::observed("writer11-drafts.json");
    let mut p = progress::decode(&raw, &all).unwrap();
    workspace::prepare(p.attempts.get_mut("shell-handover").unwrap());
    let before = value(&p);
    let receipts = value(&p.migration_receipts);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let slot = path.with_extension("before-shell-v12.json");
    std::fs::write(&path, &raw).unwrap();
    std::fs::write(&slot, &other).unwrap();
    let error = progress::save(&path, &p).expect_err("Never overwrite another raw backup");
    assert!(error.contains("backup differs"), "{error}");
    assert_eq!(std::fs::read(path).unwrap(), raw);
    assert_eq!(std::fs::read(slot).unwrap(), other);
    assert_eq!(value(&p), before);
    assert_eq!(value(&p.migration_receipts), receipts);
}

#[test]
#[ignore = "Requires immutable private actual11 COMPLETE and attestation; explicit guarded replay"]
fn future_malformed_or_unknown_record_data_is_refused_without_writing_original_evidence() {
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer11-original.json");
    let original: Value = serde_json::from_slice(&raw).unwrap();
    for change in [
        "future_record",
        "future_example",
        "future_receipt",
        "unknown_context",
    ] {
        let mut input = original.clone();
        match change {
            "future_record" => {
                input["attempts"]["shell-handover"]["state"]["run"]["engine_revision"] = json!(13)
            }
            "future_example" => {
                input["attempts"]["shell-handover"]["workspace"]["script_learning"]["example_revision"] =
                    json!(13)
            }
            "future_receipt" => input["migration_receipts"]["shell"]
                .as_array_mut()
                .unwrap()
                .push(json!(13)),
            _ => {
                input["attempts"]["unknown-synthetic-test-context"] =
                    input["attempts"]["shell-handover"].clone();
            }
        }
        let synthetic = serde_json::to_vec(&input).unwrap();
        assert!(progress::decode(&synthetic, &all).is_err(), "{change}");
        assert_eq!(evidence::observed("writer11-original.json"), raw);
    }
}
