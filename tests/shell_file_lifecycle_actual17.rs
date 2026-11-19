//! Current19 comparisons read sealed17 bytes; never execute an old producer.
#[path = "support/actual17_evidence.rs"]
mod actual17;
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell::{self, Case, Record},
    workspace,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn rows() -> Vec<Value> {
    actual17::observed("actual17-authored.jsonl")
        .split(|b| *b == b'\n')
        .filter(|r| !r.is_empty())
        .map(|r| serde_json::from_slice(r).unwrap())
        .collect()
}
fn metadata(a: &Attempt) -> Value {
    let mut v = value(a);
    v.as_object_mut().unwrap().remove("state");
    if let Some(w) = v.get_mut("workspace").and_then(Value::as_object_mut) {
        for key in ["script_history", "script_history_more", "checked"] {
            w.remove(key);
        }
        if let Some(l) = w.get_mut("script_learning").and_then(Value::as_object_mut) {
            l.remove("example_revision");
            l.remove("example_result");
        }
    }
    v
}
fn records(p: &Progress) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for a in p.attempts.values() {
        let current = if let State::Script { run, .. } = &a.state {
            run.iter().collect::<Vec<_>>()
        } else {
            vec![]
        };
        for r in current.into_iter().chain(
            a.workspace
                .iter()
                .flat_map(|w| w.script_history.iter().chain(&w.script_history_more)),
        ) {
            *map.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
        }
    }
    map
}
#[test]
fn current_revision_and_historical_target_remain_distinct() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
}
#[test]
#[ignore = "Requires independently sealed private actual17 artifacts; read-only source/ELF verification"]
fn all641_sealed17_outcomes_match_actual19_execution() {
    let mut count = 0;
    let rows = rows();
    assert_eq!(rows.len(), 68);
    for row in rows {
        let cases: Vec<Case> = serde_json::from_value(row["cases"].clone()).unwrap();
        for (n, c) in cases.iter().enumerate() {
            let out = shell::run(row["source"].as_str().unwrap(), c);
            assert!(c.accepts(&out), "{} case{n}: {out:?}", row["id"]);
            assert_eq!(value(&out), row["record"]["results"][n]);
            count += 1;
        }
        let c: Case = serde_json::from_value(row["example_case"].clone()).unwrap();
        let out = shell::run(row["example_source"].as_str().unwrap(), &c);
        assert!(c.accepts(&out));
        assert_eq!(value(&out), row["example_outcome"]);
        count += 1;
    }
    assert_eq!(count, 641);
}
#[test]
#[ignore = "Requires sealed actual17 synthetic profile pair, never personal progress"]
fn inherited_profiles_preserve_all_contexts_histories_unfinished_work_and_raw17_backup() {
    let catalog = pack::ccna_pilot().unwrap();
    let rows = rows();
    for drafts in [false, true] {
        let raw = actual17::observed(if drafts {
            "writer17-drafts.json"
        } else {
            "writer17-original.json"
        });
        let old: Progress = serde_json::from_slice(&raw).unwrap();
        let mut p = progress::decode(&raw, &catalog).unwrap();
        assert_eq!(value(&old), value(&p));
        assert_eq!(old.attempts.len(), 158);
        assert_eq!(
            old.migration_receipts.as_ref().unwrap().shell,
            vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
            "The sealed fixture was actually saved through17, not18"
        );
        let mut expected = records(&old);
        let mut rerun = 0;
        for row in &rows {
            let id = row["id"].as_str().unwrap();
            let s = catalog.iter().find(|s| s.id == id).unwrap();
            let before = &old.attempts[id];
            if drafts && matches!(before.state, State::Script { run: None, .. }) {
                assert!(["shell-command-plan", "shell-handover"].contains(&id));
                continue;
            }
            let a = p.attempts.get_mut(id).unwrap();
            workspace::prepare(a);
            let cases: Vec<Case> = serde_json::from_value(row["cases"].clone()).unwrap();
            let source = row["source"].as_str().unwrap();
            let results = cases
                .iter()
                .map(|c| shell::run(source, c))
                .collect::<Vec<_>>();
            assert!(cases.iter().zip(&results).all(|(c, r)| c.accepts(r)));
            let record = Record {
                engine_revision: 19,
                source: source.into(),
                results,
            };
            *expected
                .entry(serde_json::to_string(&record).unwrap())
                .or_default() += 1;
            a.state = State::Script {
                source: source.into(),
                run: Some(record),
            };
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            let c: Case = serde_json::from_value(row["example_case"].clone()).unwrap();
            let example = shell::run(&l.example_source, &c);
            assert!(c.accepts(&example));
            l.example_revision = 19;
            l.example_result = Some(example);
            assert!(progress::current_solved(a, s));
            assert_eq!(metadata(a), metadata(before));
            rerun += 1;
        }
        assert_eq!(rerun, if drafts { 66 } else { 68 });
        assert_eq!(p.attempts.len(), 158);
        assert_eq!(p.favorites, old.favorites);
        assert_eq!(records(&p), expected);
        for (id, a) in &old.attempts {
            if !matches!(a.state, State::Script { .. })
                || drafts && ["shell-command-plan", "shell-handover"].contains(&id.as_str())
            {
                assert_eq!(value(&p.attempts[id]), value(a));
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress-v1.json");
        std::fs::write(&path, &raw).unwrap();
        progress::save(&path, &p).unwrap();
        let loaded = progress::load(&path, &catalog).unwrap();
        assert_eq!(value(&loaded), value(&p));
        assert_eq!(
            std::fs::read(path.with_extension("before-shell-v19.json")).unwrap(),
            raw
        );
        assert_eq!(
            loaded.migration_receipts.as_ref().unwrap().shell,
            vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19],
            "Current19 adds its own receipt; it must not invent an18 writer"
        );
        let saved = std::fs::read(&path).unwrap();
        progress::save(&path, &loaded).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), saved);
    }
}
#[test]
#[ignore = "Requires sealed actual17 typed fixtures; re-executes only current19 in-process code"]
fn typed17_progress_has_no_imported_current_proof_and_replays_without_losing_drafts() {
    use dlms_learning_lab::environment_workspace as e;
    let catalog = pack::ccna_pilot().unwrap();
    let observed: Value =
        serde_json::from_slice(&actual17::observed("TYPED-OBSERVATIONS.json")).unwrap();
    assert_eq!(observed["observations"].as_array().unwrap().len(), 32);
    for drafts in [false, true] {
        let raw = actual17::observed(if drafts {
            "writer17-typed-drafts.json"
        } else {
            "writer17-typed-passed.json"
        });
        let old: Progress = serde_json::from_slice(&raw).unwrap();
        let mut p = progress::decode(&raw, &catalog).unwrap();
        assert_eq!(value(&old), value(&p));
        assert_eq!(p.attempts.len(), 4);
        for (id, a) in &mut p.attempts {
            let s = catalog.iter().find(|s| s.id == *id).unwrap();
            assert!(!progress::current_solved(a, s));
            let before = value(a);
            if drafts {
                continue;
            }
            match (&mut a.state, &s.goal) {
                (State::Environment { simulation }, Goal::Environment { .. }) => {
                    let d = &simulation.definition;
                    let assignment = e::run(&d.assignment, &simulation.assignment.source).unwrap();
                    let example = e::run(&d.example, &simulation.example.source).unwrap();
                    assert!(simulation.assignment.store(assignment, &d.assignment));
                    assert!(simulation.example.store(example, &d.example));
                    let mut actual = value(a);
                    for role in ["assignment", "example"] {
                        actual["state"]["simulation"][role]["result"]["revision"] = json!(17);
                    }
                    assert_eq!(actual, before);
                }
                (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) => {
                    simulation.assignment.run(&definition.initial).unwrap();
                    simulation.practice.run(&definition.example).unwrap();
                    assert_eq!(value(a), before);
                }
                _ => panic!(),
            };
            assert!(progress::current_solved(a, s));
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("typed.json");
        progress::save_with_catalog(&path, &p, &catalog).unwrap();
        let loaded = progress::load(&path, &catalog).unwrap();
        assert_eq!(value(&loaded), value(&p));
        if drafts {
            assert_eq!(value(&loaded), value(&old));
        }
    }
}
