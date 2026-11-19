//! Current interpreter versus real sealed16; never reruns a historical producer.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Progress},
    shell, workspace,
};
use serde::Deserialize;
use serde_json::Value;
#[path = "support/actual16_evidence.rs"]
mod evidence;
#[derive(Deserialize)]
struct Authored {
    id: String,
    source: String,
    cases: Vec<shell::Case>,
    record: shell::Record,
    example_source: String,
    example_case: shell::Case,
    example_outcome: shell::Outcome,
}
fn rows() -> Vec<Authored> {
    evidence::observed("actual16-authored.jsonl")
        .split(|b| *b == b'\n')
        .filter(|x| !x.is_empty())
        .map(|x| serde_json::from_slice(x).unwrap())
        .collect()
}
fn value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn histories(p: &Progress) -> std::collections::BTreeMap<String, usize> {
    let mut out = std::collections::BTreeMap::new();
    for a in p.attempts.values() {
        let current = match &a.state {
            State::Script { run, .. } => run.as_ref(),
            _ => None,
        };
        for r in current.into_iter().chain(
            a.workspace
                .as_ref()
                .into_iter()
                .flat_map(|w| w.script_history.iter().chain(&w.script_history_more)),
        ) {
            *out.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
        }
    }
    out
}
#[test]
#[ignore = "Requires fixed private independently sealed genuine16; run with the guarded runner"]
fn all573_actual16_assignment_outcomes_and68_examples_remain_exact() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let catalog = pack::ccna_pilot().unwrap();
    let rows = rows();
    assert_eq!(rows.len(), 68);
    let mut count = 0;
    for row in rows {
        let s = catalog.iter().find(|s| s.id == row.id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(cases, &row.cases);
        assert_eq!(s.solution[0], row.source);
        assert_eq!(row.record.engine_revision, 16);
        assert!(!row.record.current());
        assert_eq!(row.record.source, row.source);
        assert_eq!(row.record.results.len(), cases.len());
        for (c, old) in cases.iter().zip(&row.record.results) {
            assert!(c.accepts(old));
            assert_eq!(shell::run(&row.source, c), *old);
            count += 1;
        }
        let l = s.teaching.as_ref().unwrap();
        assert_eq!(l.example, row.example_source);
        assert_eq!(l.fixture, row.example_case);
        assert!(l.fixture.accepts(&row.example_outcome));
        assert_eq!(shell::run(&l.example, &l.fixture), row.example_outcome);
    }
    assert_eq!(count, 573);
}
#[test]
#[ignore = "Requires fixed private independently sealed genuine16; run with the guarded runner"]
fn actual16_progress_stays_legacy_until_rerun_and_saves_exact_before17_backups() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let catalog = pack::ccna_pilot().unwrap();
    for name in ["writer16-original.json", "writer16-drafts.json"] {
        let raw = evidence::observed(name);
        let old: Progress = serde_json::from_slice(&raw).unwrap();
        let mut p = progress::decode(&raw, &catalog).unwrap();
        assert_eq!(value(&p), value(&old));
        assert_eq!(p.attempts.len(), 158);
        let before = histories(&p);
        for (id, a) in &mut p.attempts {
            let s = catalog.iter().find(|s| &s.id == id).unwrap();
            if !matches!(a.state, State::Script { .. }) {
                assert_eq!(value(a), value(&old.attempts[id]));
                continue;
            }
            let completed = matches!(a.state, State::Script { run: Some(_), .. });
            if !completed {
                assert_eq!(value(a), value(&old.attempts[id]));
                continue;
            }
            assert!(progress::legacy_script_credit(a, s));
            assert!(!progress::current_solved(a, s));
            workspace::prepare(a);
            assert!(!progress::current_solved(a, s));
            assert!(progress::legacy_script_credit(a, s));
        }
        assert_eq!(histories(&p), before);
        assert_eq!(p.favorites, old.favorites);
        for id in ["shell-command-plan", "shell-handover"] {
            if name.contains("drafts") {
                assert_eq!(value(&p.attempts[id]), value(&old.attempts[id]));
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress-v1.json");
        std::fs::write(&path, &raw).unwrap();
        progress::save_with_catalog(&path, &p, &catalog).unwrap();
        assert_eq!(
            std::fs::read(path.with_extension("before-shell-v19.json")).unwrap(),
            raw
        );
        let loaded = progress::load(&path, &catalog).unwrap();
        assert_eq!(value(&loaded), value(&p));
        assert_eq!(histories(&loaded), before);
        assert_eq!(
            loaded.migration_receipts.as_ref().unwrap().shell,
            vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 19]
        );
        let saved = std::fs::read(&path).unwrap();
        progress::save_with_catalog(&path, &loaded, &catalog).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(
            std::fs::read(path.with_extension("before-shell-v19.json")).unwrap(),
            raw
        );
    }
}
#[test]
#[ignore = "Requires fixed private independently sealed genuine16; run with the guarded runner"]
fn rerun_awards_only_observed_current_results_and_edits_invalidate_them() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let catalog = pack::ccna_pilot().unwrap();
    let raw = evidence::observed("writer16-original.json");
    let mut p = progress::decode(&raw, &catalog).unwrap();
    let row = rows()
        .into_iter()
        .find(|r| r.id == "shell-debug-status")
        .unwrap();
    let s = catalog.iter().find(|s| s.id == row.id).unwrap();
    let a = p.attempts.get_mut(&row.id).unwrap();
    workspace::prepare(a);
    let before = a.workspace.as_ref().unwrap().script_history_more.clone();
    let results = row
        .cases
        .iter()
        .map(|c| shell::run(&row.source, c))
        .collect();
    a.state = State::Script {
        source: row.source.clone(),
        run: Some(shell::Record {
            engine_revision: 19,
            source: row.source.clone(),
            results,
        }),
    };
    assert!(progress::current_solved(a, s));
    assert_eq!(a.workspace.as_ref().unwrap().script_history_more, before);
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("\nprintf 'broken'\n");
    assert!(!progress::current_solved(a, s));
    assert!(progress::legacy_script_credit(a, s));
}
