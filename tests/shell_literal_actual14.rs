//! Genuine14-before15 consumers. Source proposal; every group needs future
//! real capture/seals. No old producer invocation or invented historical result.
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as disk_registration;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Progress},
    shell::{self, Case, Outcome, Record},
    workspace,
};
use serde::Deserialize;
use serde_json::Value;
#[path = "support/actual14_evidence.rs"]
mod evidence;
#[derive(Deserialize)]
struct Authored {
    id: String,
    source: String,
    cases: Vec<Case>,
    record: Record,
    example_source: String,
    example_case: Case,
    example_outcome: Outcome,
}
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn rows() -> Vec<Authored> {
    evidence::observed("actual14-authored.jsonl")
        .split(|b| *b == b'\n')
        .filter(|r| !r.is_empty())
        .map(|r| serde_json::from_slice(r).unwrap())
        .collect()
}
fn histories(a: &progress::Attempt, primary: bool) -> std::collections::BTreeMap<String, usize> {
    let mut m = std::collections::BTreeMap::new();
    if primary && let State::Script { run: Some(r), .. } = &a.state {
        *m.entry(serde_json::to_string(r).unwrap()).or_insert(0) += 1;
    }
    if let Some(w) = &a.workspace {
        for r in w.script_history.iter().chain(&w.script_history_more) {
            *m.entry(serde_json::to_string(r).unwrap()).or_insert(0) += 1;
        }
    }
    m
}
#[test]
#[ignore = "Requires independently sealed genuine14; Root guarded explicit run only"]
fn genuine60_records477_cases60_examples_remain_real_source_bound_at_current14() {
    assert_eq!(shell::EVALUATOR_REVISION, 15);
    let all = pack::bundled().unwrap();
    let observed = rows();
    assert_eq!(observed.len(), 60);
    let mut count = 0;
    for row in observed {
        let s = all.iter().find(|s| s.id == row.id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(cases, &row.cases);
        assert_eq!(s.solution.join("\n"), row.source);
        assert_eq!(row.record.engine_revision, 14);
        assert!(!row.record.current());
        assert_eq!(row.record.source, row.source);
        assert_eq!(row.record.results.len(), cases.len());
        for (c, old) in cases.iter().zip(&row.record.results) {
            assert!(c.accepts(old));
            assert_eq!(&shell::run(&row.source, c), old);
            count += 1;
        }
        let l = s.teaching.as_ref().unwrap();
        assert_eq!(l.example, row.example_source);
        assert_eq!(l.fixture, row.example_case);
        assert!(l.fixture.accepts(&row.example_outcome));
        assert_eq!(shell::run(&l.example, &l.fixture), row.example_outcome);
    }
    assert_eq!(count, 477);
}
#[test]
#[ignore = "Requires independently sealed genuine14 original/draft profiles"]
fn all_prior_records_multiplicity_two_unfinished_contexts_and_90_nonscript_attempts_stay_exact() {
    assert_eq!(shell::EVALUATOR_REVISION, 15);
    let all = pack::bundled().unwrap();
    for (role, total) in [("original", 426usize), ("drafts", 416usize)] {
        let raw = evidence::observed(&format!("writer14-{role}.json"));
        let old: Progress = serde_json::from_slice(&raw).unwrap();
        assert_eq!(old.attempts.len(), 150);
        let previous_raw = evidence::observed(&format!("input13-{role}.json"));
        let previous: Progress = serde_json::from_slice(&previous_raw).unwrap();
        assert_eq!(previous.attempts.len(), 147);
        let prior_count: usize = previous
            .attempts
            .values()
            .map(|a| histories(a, true).values().sum::<usize>())
            .sum();
        assert_eq!(prior_count, if role == "drafts" { 358 } else { 366 });
        for (id, before) in &previous.attempts {
            let after = &old.attempts[id];
            let unchanged = !matches!(before.state, State::Script { .. })
                || (role == "drafts"
                    && ["shell-command-plan", "shell-handover"].contains(&id.as_str()));
            if unchanged {
                assert_eq!(value(after), value(before));
            } else {
                assert_eq!(histories(after, false), histories(before, true));
            }
        }

        let mut current = progress::decode(&raw, &all).unwrap();
        assert_eq!(value(&current), value(&old));
        assert_eq!(
            value(&current.migration_receipts),
            value(&old.migration_receipts)
        );
        let mut count = 0;
        let mut non = 0;
        for (id, before) in &old.attempts {
            count += histories(before, true).values().sum::<usize>();
            let after = current.attempts.get_mut(id).unwrap();
            let untouched =
                role == "drafts" && ["shell-command-plan", "shell-handover"].contains(&id.as_str());
            if !matches!(before.state, State::Script { .. }) || untouched {
                assert_eq!(value(after), value(before));
                if !matches!(before.state, State::Script { .. }) {
                    non += 1;
                }
                continue;
            }
            let prior = histories(before, true);
            workspace::prepare(after);
            assert_eq!(histories(after, true), prior);
        }
        assert_eq!(count, total);
        assert_eq!(non, 90);
        for id in [
            "shell-ere-event-view",
            "shell-ere-inventory-review",
            "shell-ere-incident-report",
        ] {
            let s = all.iter().find(|s| s.id == id).unwrap();
            let a = &old.attempts[id];
            assert!(matches!(a.state, State::Script { run: Some(_), .. }));
            assert_eq!(
                progress::current_solved(a, s),
                shell::EVALUATOR_REVISION == 14
            );
        }
    }
}

#[test]
#[ignore = "Requires genuine sealed14 original/draft profiles; no replay"]
fn four_new_normal_disk_sources_do_not_invent_work_in_the_genuine14_profiles() {
    let all = pack::bundled().unwrap();
    assert_eq!(disk_registration::before_scenarios(&all).len(), 193);
    assert_eq!(all.len(), 197);
    for role in ["original", "drafts"] {
        let raw = evidence::observed(&format!("writer14-{role}.json"));
        let old: Progress = serde_json::from_slice(&raw).unwrap();
        assert_eq!(old.attempts.len(), 150);
        let current = progress::decode(&raw, &all).unwrap();
        assert_eq!(value(&current.attempts), value(&old.attempts));
        assert_eq!(current.favorites, old.favorites);
        for id in disk_registration::IDS {
            let s = all.iter().find(|s| s.id == id).unwrap();
            assert!(
                matches!(s.initial, State::PartitionFs { .. })
                    && matches!(s.goal, Goal::PartitionFs { .. })
            );
            assert!(!old.attempts.contains_key(id) && !current.attempts.contains_key(id));
            assert!(!old.favorites.contains(id) && !current.favorites.contains(id));
            assert!(!progress::current_solved(&progress::Attempt::new(s, 1), s));
        }
    }
}

#[test]
#[ignore = "Requires sealed genuine14; actual15 rerun, private save and explicit historical preservation"]
fn genuine14_remains_legacy_until_actual15_rerun_and_save_keeps_every_other_attempt() {
    assert_eq!(shell::EVALUATOR_REVISION, 15);
    let all = pack::bundled().unwrap();
    let id = "shell-ere-event-view";
    let s = all.iter().find(|s| s.id == id).unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    for role in ["original", "drafts"] {
        let raw = evidence::observed(&format!("writer14-{role}.json"));
        let old: Progress = serde_json::from_slice(&raw).unwrap();
        let mut p = progress::decode(&raw, &all).unwrap();
        assert_eq!(value(&p), value(&old));
        assert!(!progress::current_solved(&p.attempts[id], s));
        let prior = histories(&p.attempts[id], true);
        let a = p.attempts.get_mut(id).unwrap();
        workspace::prepare(a);
        assert_eq!(histories(a, true), prior);
        let source = s.solution.join("\n");
        let results = cases
            .iter()
            .map(|c| shell::run(&source, c))
            .collect::<Vec<_>>();
        assert!(cases.iter().zip(&results).all(|(c, r)| c.accepts(r)));
        let actual = Record {
            engine_revision: 15,
            source: source.clone(),
            results,
        };
        a.state = State::Script {
            source,
            run: Some(actual.clone()),
        };
        assert!(progress::current_solved(a, s));
        let mut expected = prior;
        *expected
            .entry(serde_json::to_string(&actual).unwrap())
            .or_insert(0) += 1;
        assert_eq!(histories(a, true), expected);
        for (other, before) in &old.attempts {
            if other != id {
                assert_eq!(value(&p.attempts[other]), value(before));
            }
        }
        let mut before = value(&old);
        let mut after = value(&p);
        before["attempts"].as_object_mut().unwrap().remove(id);
        after["attempts"].as_object_mut().unwrap().remove(id);
        assert_eq!(before, after);
        assert_eq!(value(&p.migration_receipts), value(&old.migration_receipts));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        std::fs::write(&path, &raw).unwrap();
        let caller = value(&p);
        let receipt = value(&p.migration_receipts);
        progress::save_with_catalog(&path, &p, &all).unwrap();
        assert_eq!(value(&p), caller);
        assert_eq!(value(&p.migration_receipts), receipt);
        assert_eq!(
            std::fs::read(path.with_extension("before-shell-v15.json")).unwrap(),
            raw
        );
        let resumed = progress::load(&path, &all).unwrap();
        assert_eq!(value(&resumed), caller);
        let mut expected_receipt = receipt;
        expected_receipt["shell"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(15));
        assert_eq!(value(&resumed.migration_receipts), expected_receipt);
        assert!(progress::current_solved(&resumed.attempts[id], s));
    }
}
