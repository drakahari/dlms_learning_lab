//! Exact replay of independently sealed18; never reruns a historical producer.
#[path = "support/actual18_evidence.rs"]
mod actual18;
use dlms_learning_lab::shell::{self, Case, Record};
#[test]
#[ignore = "Requires independently sealed private actual18 artifacts; read-only retained source and ELF verification"]
fn all_679_prior_supported_outcomes_remain_byte_exact_under_current_evaluator() {
    let raw = actual18::observed("actual18-authored.jsonl");
    let mut count = 0;
    for row in raw.split(|b| *b == b'\n').filter(|r| !r.is_empty()) {
        let row: serde_json::Value = serde_json::from_slice(row).unwrap();
        let record: Record = serde_json::from_value(row["record"].clone()).unwrap();
        assert_eq!(record.engine_revision, 18);
        assert!(!record.current());
        let cases: Vec<Case> = serde_json::from_value(row["cases"].clone()).unwrap();
        for (c, old) in cases.iter().zip(&record.results) {
            assert_eq!(
                shell::run(&record.source, c),
                *old,
                "{} / {}",
                row["id"],
                c.name
            );
            count += 1;
        }
        let example: Case = serde_json::from_value(row["example_case"].clone()).unwrap();
        let old: shell::Outcome = serde_json::from_value(row["example_outcome"].clone()).unwrap();
        assert_eq!(
            shell::run(row["example_source"].as_str().unwrap(), &example),
            old,
            "{} practice",
            row["id"]
        );
        count += 1;
    }
    assert_eq!(count, 679);
}

use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
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
#[ignore = "Requires independently sealed private18 progress; current in-process reruns only"]
fn inherited_and_new_profiles_retain_drafts_history_and_backup_before_actual19_credit() {
    let catalog = pack::ccna_pilot().unwrap();
    let rows = actual18::observed("actual18-authored.jsonl")
        .split(|b| *b == b'\n')
        .filter(|b| !b.is_empty())
        .map(|b| serde_json::from_slice::<Value>(b).unwrap())
        .collect::<Vec<_>>();
    for new_profile in [false, true] {
        for drafts in [false, true] {
            let name = match (new_profile, drafts) {
                (false, false) => "writer18-original.json",
                (false, true) => "writer18-drafts.json",
                (true, false) => "writer18-new-passed.json",
                (true, true) => "writer18-new-drafts.json",
            };
            let raw = actual18::observed(name);
            let old: Progress = serde_json::from_slice(&raw).unwrap();
            let mut p = progress::decode(&raw, &catalog).unwrap();
            assert_eq!(value(&p), value(&old));
            assert_eq!(p.attempts.len(), if new_profile { 3 } else { 158 });
            let mut expected = records(&old);
            let mut ran = 0;
            for row in &rows {
                let id = row["id"].as_str().unwrap();
                let Some(before) = old.attempts.get(id) else {
                    continue;
                };
                let sc = catalog.iter().find(|s| s.id == id).unwrap();
                assert!(!progress::current_solved(&p.attempts[id], sc));
                if matches!(before.state, State::Script { run: None, .. }) {
                    assert!(drafts);
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
                assert!(progress::current_solved(a, sc));
                assert_eq!(metadata(a), metadata(before));
                ran += 1;
            }
            assert_eq!(
                ran,
                match (new_profile, drafts) {
                    (false, false) => 68,
                    (false, true) => 66,
                    (true, false) => 3,
                    (true, true) => 0,
                }
            );
            assert_eq!(p.attempts.len(), old.attempts.len());
            assert_eq!(p.favorites, old.favorites);
            assert_eq!(records(&p), expected);
            for (id, before) in &old.attempts {
                if !matches!(before.state, State::Script { run: Some(_), .. }) {
                    assert_eq!(value(&p.attempts[id]), value(before));
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
                if new_profile {
                    vec![18, 19]
                } else {
                    (7..=19).collect()
                }
            );
            let saved = std::fs::read(&path).unwrap();
            progress::save(&path, &loaded).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), saved);
        }
    }
}

#[test]
#[ignore = "Requires sealed actual18 typed fixtures; re-executes only current19 in-process code"]
fn typed18_progress_has_no_imported_current_proof_and_replays_without_losing_drafts() {
    use dlms_learning_lab::environment_workspace as e;
    let catalog = pack::ccna_pilot().unwrap();
    let observed: Value =
        serde_json::from_slice(&actual18::observed("TYPED-OBSERVATIONS.json")).unwrap();
    assert_eq!(observed["observations"].as_array().unwrap().len(), 32);
    for drafts in [false, true] {
        let raw = actual18::observed(if drafts {
            "writer18-typed-drafts.json"
        } else {
            "writer18-typed-passed.json"
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
                        actual["state"]["simulation"][role]["result"]["revision"] = json!(18);
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
