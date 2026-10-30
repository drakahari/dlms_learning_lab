//! Explicit private10→current14 consumers. Genuine11 is separately retained. Old bytes/results never regenerated or relabeled.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
#[path = "support/parameter_arithmetic_amendment.rs"]
mod arithmetic_amendment;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};
#[path = "support/actual10_evidence.rs"]
mod evidence;
mod amendment_sha {
    pub(super) fn hash(bytes: &[u8]) -> String {
        super::evidence::sha(bytes)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authored {
    id: String,
    source: String,
    record: Record,
    cases: Vec<Case>,
    example_source: String,
    example_case: Case,
    example_outcome: Outcome,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    name: String,
    source: String,
    case: Case,
    actual10: Outcome,
}
fn snapshot(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).unwrap()
}
fn authored() -> Vec<Authored> {
    evidence::observed("actual10-authored.jsonl")
        .split(|b| *b == b'\n')
        .filter(|r| !r.is_empty())
        .map(|r| serde_json::from_slice(r).unwrap())
        .collect()
}
fn sliced(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap();
    for _ in 0..20_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Finite cooperative test bound exceeded");
}
fn records(p: &Progress) -> BTreeMap<String, Vec<Record>> {
    p.attempts
        .iter()
        .map(|(id, a)| {
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
            if let State::Script { run: Some(r), .. } = &a.state
                && !rows.contains(r)
            {
                rows.push(r.clone());
            }
            (id.clone(), rows)
        })
        .collect()
}
#[test]
#[ignore = "Requires immutable private actual10 capture, run through MAIN guard explicitly"]
fn all49_original_records350_cases49_examples_remain_exact_under12_at_four_slices() {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = pack::bundled().unwrap();
    let old = authored();
    assert_eq!(old.len(), 49);
    let mut ids = BTreeSet::new();
    let mut count = 0;
    for row in old {
        assert!(ids.insert(row.id.clone()));
        let s = all.iter().find(|s| s.id == row.id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(snapshot(cases), snapshot(&row.cases));
        assert_eq!(s.solution.join("\n"), row.source);
        assert_eq!(row.record.engine_revision, 10);
        assert!(!row.record.current());
        assert!(row.record.valid());
        assert_eq!(row.record.source, row.source);
        assert_eq!(row.record.results.len(), cases.len());
        count += cases.len();
        for (case, result) in cases.iter().zip(&row.record.results) {
            assert!(case.accepts(result));
            for slice in [1, 7, 32, 64] {
                assert_eq!(
                    sliced(&row.source, case, slice),
                    *result,
                    "{} / {} / {slice}",
                    row.id,
                    case.name
                );
            }
        }
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(lesson.example, row.example_source);
        assert_eq!(snapshot(&lesson.fixture), snapshot(&row.example_case));
        assert!(row.example_case.accepts(&row.example_outcome));
        for slice in [1, 7, 32, 64] {
            assert_eq!(
                sliced(&row.example_source, &row.example_case, slice),
                row.example_outcome
            );
        }
    }
    assert_eq!(count, 350);
}
#[test]
#[ignore = "Requires immutable private actual10 capture, run through MAIN guard explicitly"]
fn genuine35_keep_original10_results_classify33_new_supported_one_invalid_one_literal_control() {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let rows: Vec<Proposal> =
        serde_json::from_slice(&evidence::observed("actual10-parameters.json")).unwrap();
    assert_eq!(rows.len(), 35);
    let fixed: Value = serde_json::from_str(include_str!(
        "fixtures/bash-parameter-defaults-reference.json"
    ))
    .unwrap();
    let mut changed = 0;
    let mut same = 0;
    let mut invalid = 0;
    for (n, row) in rows.into_iter().enumerate() {
        assert_eq!(fixed[n]["name"], row.name);
        assert_eq!(fixed[n]["source"], row.source);
        assert_eq!(fixed[n]["case"], snapshot(&row.case));
        if arithmetic_amendment::invalid_proposal(&row.name, &row.source, &row.case) {
            assert_eq!(n, 32);
            assert_eq!(row.actual10.error.as_deref(), Some("Close ${parameter}"));
            assert_eq!(row.actual10.status, 2);
            arithmetic_amendment::assert_syntax_before_effects(&row.source, &row.case);
            invalid += 1;
            continue;
        }
        for slice in [1, 7, 32, 64] {
            let out = sliced(&row.source, &row.case, slice);
            assert!(row.case.accepts(&out), "{}: {out:?}", row.name);
            if row.actual10.error.is_none() {
                assert_eq!(out, row.actual10);
            } else {
                assert_ne!(out, row.actual10);
            }
        }
        if row.actual10.error.is_some() {
            assert_eq!(row.actual10.status, 2);
            changed += 1;
        } else {
            assert_eq!(row.name, "literal no reparsing");
            same += 1;
        }
    }
    assert_eq!((changed, same, invalid), (33, 1, 1));
}
#[test]
#[ignore = "Requires immutable private actual10 capture, run through MAIN guard explicitly"]
fn actual139_contexts_keep_drafts_all_histories_receipts_and_exact_raw10_backup_until_explicit_run()
{
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer10-original.json");
    let original: Progress = serde_json::from_slice(&raw).unwrap();
    let expected_records = records(&original);
    let mut p = progress::decode(&raw, &all).unwrap();
    assert_eq!(snapshot(&p), snapshot(&original));
    assert_eq!(p.attempts.len(), 139);
    assert_eq!(
        snapshot(&p.migration_receipts),
        json!({"workspace":true,"shell":[7,8,9,10]})
    );
    let mut scripts = 0;
    for (id, a) in &mut p.attempts {
        let old = &original.attempts[id];
        let s = all.iter().find(|s| &s.id == id).unwrap();
        if let State::Script {
            source,
            run: Some(record),
        } = &old.state
        {
            scripts += 1;
            assert_eq!(record.engine_revision, 10);
            assert!(!progress::current_solved(a, s));
            let mut expected = snapshot(old);
            expected["state"]["run"] = Value::Null;
            let w = &mut expected["workspace"];
            w["checked"] = Value::Bool(false);
            if w["script_history"].is_null() {
                w["script_history"] = snapshot(record);
            } else if !w["script_history_more"]
                .as_array()
                .into_iter()
                .flatten()
                .chain(std::iter::once(&w["script_history"]))
                .any(|r| r == &snapshot(record))
            {
                w.as_object_mut()
                    .unwrap()
                    .entry("script_history_more")
                    .or_insert_with(|| json!([]))
                    .as_array_mut()
                    .unwrap()
                    .push(snapshot(record));
            }
            workspace::prepare(a);
            assert_eq!(
                snapshot(a),
                expected,
                "Only explicit main-run→history fields change: {id}"
            );
            let State::Script { source: now, run } = &a.state else {
                panic!()
            };
            assert_eq!(now, source);
            assert!(run.is_none());
        } else {
            assert_eq!(snapshot(a), snapshot(old));
        }
    }
    assert_eq!(scripts, 49);
    assert_eq!(records(&p), expected_records);
    assert_eq!(p.favorites, original.favorites);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    progress::save(&path, &p).unwrap();
    let slot = path.with_extension("before-shell-v14.json");
    assert_eq!(std::fs::read(&slot).unwrap(), raw);
    let mut resumed = progress::load(&path, &all).unwrap();
    assert_eq!(snapshot(&resumed), snapshot(&p));
    assert_eq!(
        resumed.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 14]
    );
    let id = "shell-handover";
    let s = all.iter().find(|s| s.id == id).unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let a = resumed.attempts.get_mut(id).unwrap();
    let source = s.solution.join("\n");
    a.state = State::Script {
        source: source.clone(),
        run: Some(Record {
            engine_revision: 14,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(&source, c)).collect(),
        }),
    };
    assert!(progress::current_solved(a, s));
    assert_eq!(
        records(&resumed)[id][..expected_records[id].len()],
        expected_records[id]
    );
    let State::Script { source, .. } = &mut resumed.attempts.get_mut(id).unwrap().state else {
        panic!()
    };
    source.push_str("printf wrong\n");
    assert!(!progress::current_solved(&resumed.attempts[id], s));
    let before = snapshot(&resumed);
    resumed
        .attempts
        .insert(id.into(), Attempt::new(s, resumed.clock + 1));
    let after = snapshot(&resumed);
    for (other, a) in before["attempts"].as_object().unwrap() {
        if other != id {
            assert_eq!(&after["attempts"][other], a);
        }
    }
    assert_eq!(after["favorites"], before["favorites"]);
    progress::save(&path, &resumed).unwrap();
    assert_eq!(std::fs::read(slot).unwrap(), raw);
    assert_eq!(snapshot(&progress::load(&path, &all).unwrap()), after);
}
#[test]
#[ignore = "Requires immutable private actual10 capture, run through MAIN guard explicitly"]
fn actual_unfinished10_drafts_survive_and_backup_rechecks_only_after_explicit_inspection_restore() {
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer10-drafts.json");
    let original: Progress = serde_json::from_slice(&raw).unwrap();
    let old_records = records(&original);
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("export.json");
    backup::export(&source, &original, &all).unwrap();
    let target = dir.path().join("progress.json");
    std::fs::write(&target, &raw).unwrap();
    assert!(backup::inspect(&source, &all, &AtomicBool::new(true), &AtomicUsize::new(0)).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), raw);
    let candidate =
        backup::inspect(&source, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let scripts: Vec<_> = original
        .attempts
        .iter()
        .filter(|(_, a)| matches!(a.state, State::Script { .. }))
        .collect();
    assert_eq!(scripts.len(), 49);
    let absent_main: BTreeSet<_> = scripts
        .iter()
        .filter_map(|(id, a)| {
            matches!(a.state, State::Script { run: None, .. }).then_some(id.as_str())
        })
        .collect();
    let absent_example: BTreeSet<_> = scripts
        .iter()
        .filter_map(|(id, a)| {
            a.workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_result
                .is_none()
                .then_some(id.as_str())
        })
        .collect();
    assert_eq!(
        absent_main,
        BTreeSet::from(["shell-command-plan", "shell-handover"])
    );
    assert_eq!(absent_example, absent_main);
    assert_eq!(candidate.rechecked, 94); //47 existing assignment records +47 existing example results; two explicit resets remain absent.
    assert!(candidate.progress.migration_receipts.is_none());
    for (id, a) in &candidate.progress.attempts {
        let old = &original.attempts[id];
        let s = all.iter().find(|s| &s.id == id).unwrap();
        if let State::Script {
            source,
            run: Some(r),
        } = &a.state
        {
            let Goal::Script { cases } = &s.goal else {
                panic!()
            };
            assert_eq!(r.engine_revision, 14);
            assert_eq!(&r.source, source);
            assert_eq!(
                r.results,
                cases
                    .iter()
                    .map(|c| shell::run(source, c))
                    .collect::<Vec<_>>()
            );
            let State::Script { source: before, .. } = &old.state else {
                panic!()
            };
            assert_eq!(source, before);
            let now = a.workspace.as_ref().unwrap();
            let before = old.workspace.as_ref().unwrap();
            let l = now.script_learning.as_ref().unwrap();
            let prior = before.script_learning.as_ref().unwrap();
            assert_eq!(l.example_source, prior.example_source);
            assert_eq!(l.view, prior.view);
            assert_eq!(l.example_revision, 14);
            assert_eq!(
                l.example_result,
                Some(shell::run(
                    &l.example_source,
                    &s.teaching.as_ref().unwrap().fixture
                ))
            );
            assert_eq!(
                now.script_history
                    .as_ref()
                    .into_iter()
                    .chain(&now.script_history_more)
                    .cloned()
                    .collect::<Vec<_>>(),
                old_records[id]
            );
        } else {
            assert_eq!(snapshot(a), snapshot(old));
        }
    }
    assert_eq!(
        std::fs::read(&target).unwrap(),
        raw,
        "Inspection is not restore confirmation"
    );
    let result = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(), raw);
    assert_eq!(
        std::fs::read(target.with_extension("before-shell-v14.json")).unwrap(),
        raw
    );
    let restored = progress::load(&target, &all).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&candidate.progress));
    assert_eq!(
        restored.migration_receipts.unwrap().shell,
        vec![7, 8, 9, 10, 14]
    );
}

#[test]
#[ignore = "Requires immutable private actual10 capture, run through MAIN guard explicitly"]
fn occupied12_backup_refuses_changed10_bytes_without_stamping_caller_or_replacing_target() {
    let all = pack::bundled().unwrap();
    let original = evidence::observed("writer10-original.json");
    let other = evidence::observed("writer10-drafts.json");
    assert_ne!(original, other);
    let mut p = progress::decode(&original, &all).unwrap();
    workspace::prepare(p.attempts.get_mut("shell-handover").unwrap());
    let before = snapshot(&p);
    let receipts = snapshot(&p.migration_receipts);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let slot = path.with_extension("before-shell-v14.json");
    std::fs::write(&path, &original).unwrap();
    std::fs::write(&slot, &other).unwrap();
    let error =
        progress::save(&path, &p).expect_err("An occupied different raw backup cannot be replaced");
    assert!(error.contains("backup differs"), "{error}");
    assert_eq!(std::fs::read(path).unwrap(), original);
    assert_eq!(std::fs::read(slot).unwrap(), other);
    assert_eq!(snapshot(&p), before);
    assert_eq!(snapshot(&p.migration_receipts), receipts);
}
