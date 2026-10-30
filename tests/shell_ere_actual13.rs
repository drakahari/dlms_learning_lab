//! Genuine evaluator13 observations, independently rerun by current14 when gated.
//! No producer invocation, constructed historical outcomes or learner mastery.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
#[path = "support/actual13_evidence.rs"]
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
    example_revision: u32,
}
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn rows() -> Vec<Authored> {
    let raw = evidence::observed("actual13-authored.jsonl");
    assert!(!raw.contains(&b'\r') && raw.ends_with(b"\n"));
    raw.split(|b| *b == b'\n')
        .filter(|r| !r.is_empty())
        .map(|r| serde_json::from_slice(r).unwrap())
        .collect()
}
fn sliced(source: &str, c: &Case, n: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(n);
    }
    panic!("Current14 replay exceeded fixed polling bound")
}
fn records(a: &Attempt) -> Vec<Record> {
    let mut out = Vec::new();
    if let Some(w) = &a.workspace {
        out.extend(w.script_history.iter().cloned());
        out.extend(w.script_history_more.iter().cloned());
    }
    if let State::Script { run: Some(r), .. } = &a.state {
        out.push(r.clone());
    }
    out
}
/// Sorting preserves every occurrence; no set/dedup can hide a missing duplicate.
fn multisets(p: &Progress) -> BTreeMap<String, Vec<String>> {
    p.attempts
        .iter()
        .map(|(id, a)| {
            let mut out = records(a)
                .iter()
                .map(|r| serde_json::to_string(r).unwrap())
                .collect::<Vec<_>>();
            out.sort();
            (id.clone(), out)
        })
        .collect()
}
fn count(p: &Progress) -> usize {
    p.attempts.values().map(|a| records(a).len()).sum()
}
fn revision_counts(p: &Progress) -> BTreeMap<u32, usize> {
    let mut out = BTreeMap::new();
    for a in p.attempts.values() {
        for r in records(a) {
            *out.entry(r.engine_revision).or_default() += 1;
        }
    }
    out
}
fn preserved_metadata(mut a: Value, mut b: Value) {
    for v in [&mut a, &mut b] {
        v.as_object_mut().unwrap().remove("state");
        let w = v["workspace"].as_object_mut().unwrap();
        for k in ["checked", "script_history", "script_history_more"] {
            w.remove(k);
        }
    }
    assert_eq!(a, b);
}
fn all() -> Vec<pack::Scenario> {
    let out = pack::bundled().unwrap();
    assert_eq!(out.len(), 190);
    out
}
fn save(raw: &[u8], p: &Progress, all: &[pack::Scenario]) -> Progress {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, raw).unwrap();
    let caller = value(p);
    let receipts = value(&p.migration_receipts);
    progress::save_with_catalog(&path, p, all).unwrap();
    assert_eq!(value(p), caller);
    assert_eq!(value(&p.migration_receipts), receipts);
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v14.json")).unwrap(),
        raw
    );
    let loaded = progress::load(&path, all).unwrap();
    assert_eq!(value(&loaded), caller);
    // Portable Progress serialization intentionally omits writer receipts.
    // Compare them explicitly rather than fabricating an invisible Value field.
    let mut expected_receipts = receipts;
    expected_receipts["shell"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!(14));
    assert_eq!(value(&loaded.migration_receipts), expected_receipts);
    loaded
}
#[test]
#[ignore = "Requires fixed sealed genuine13 observations; explicit guarded current14 replay"]
fn all57_actual13_records442_cases57_examples_match_at_four_cooperative_slices() {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = all();
    let rows = rows();
    assert_eq!(rows.len(), 57);
    let mut ids = std::collections::BTreeSet::new();
    let mut cases_count = 0;
    for row in rows {
        assert!(ids.insert(row.id.clone()));
        let s = all.iter().find(|s| s.id == row.id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(value(cases), value(&row.cases));
        assert_eq!(s.solution.join("\n"), row.source);
        assert_eq!(row.record.source, row.source);
        assert_eq!(row.record.engine_revision, 13);
        assert!(row.record.valid() && !row.record.current());
        assert_eq!(row.record.results.len(), cases.len());
        for (c, old) in cases.iter().zip(&row.record.results) {
            assert!(c.accepts(old));
            for slice in [1, 7, 32, 64] {
                assert_eq!(
                    sliced(&row.source, c, slice),
                    *old,
                    "{}/{} slice{slice}",
                    row.id,
                    c.name
                );
            }
        }
        cases_count += cases.len();
        let t = s.teaching.as_ref().unwrap();
        assert_eq!(t.example, row.example_source);
        assert_eq!(value(&t.fixture), value(&row.example_case));
        assert_eq!(row.example_revision, 13);
        assert!(row.example_case.accepts(&row.example_outcome));
        for slice in [1, 7, 32, 64] {
            assert_eq!(
                sliced(&row.example_source, &row.example_case, slice),
                row.example_outcome,
                "{} example slice{slice}",
                row.id
            );
        }
    }
    assert_eq!(cases_count, 442);
}
#[test]
#[ignore = "Requires fixed genuine13 original147 context profile and all retained seals"]
fn original147_contexts366_record_occurrences_and90_non_script_attempts_are_preserved_until_real_rerun()
 {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = all();
    let raw = evidence::observed("writer13-original.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    assert_eq!(old.attempts.len(), 147);
    assert_eq!(count(&old), 366);
    assert_eq!(
        revision_counts(&old),
        BTreeMap::from([
            (6, 33),
            (7, 37),
            (8, 40),
            (9, 43),
            (10, 49),
            (11, 52),
            (12, 55),
            (13, 57)
        ])
    );
    let mut p = progress::decode(&raw, &all).unwrap();
    assert_eq!(value(&p), value(&old));
    assert_eq!(
        p.migration_receipts.as_ref().unwrap().shell,
        [7, 8, 9, 10, 11, 12, 13]
    );
    let before = multisets(&old);
    let (mut scripts, mut non_script) = (0, 0);
    for (id, a) in &mut p.attempts {
        let previous = &old.attempts[id];
        let s = all.iter().find(|s| &s.id == id).unwrap();
        if let State::Script {
            source,
            run: Some(record),
        } = &previous.state
        {
            scripts += 1;
            assert_eq!(record.engine_revision, 13);
            assert_eq!(&record.source, source);
            assert!(!progress::current_solved(a, s));
            let learning = previous
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap();
            assert_eq!(learning.example_revision, 13);
            workspace::prepare(a);
            let State::Script {
                source: prepared_source,
                run: None,
            } = &a.state
            else {
                panic!("Legacy run must move to history without changing its draft")
            };
            assert_eq!(prepared_source, source);
            assert!(!progress::current_solved(a, s));
            preserved_metadata(value(a), value(previous));
            assert_eq!(
                value(&a.workspace.as_ref().unwrap().script_learning),
                value(&previous.workspace.as_ref().unwrap().script_learning)
            );
        } else {
            non_script += 1;
            assert_eq!(value(a), value(previous));
        }
    }
    assert_eq!((scripts, non_script), (57, 90));
    assert_eq!(multisets(&p), before);
    assert_eq!(count(&p), 366);
    assert_eq!(p.clock, old.clock);
    assert_eq!(p.favorites, old.favorites);
    let resumed = save(&raw, &p, &all);
    assert_eq!(multisets(&resumed), before);
    assert_eq!(
        resumed.migration_receipts.unwrap().shell,
        [7, 8, 9, 10, 11, 12, 13, 14]
    );
}
#[test]
#[ignore = "Requires fixed genuine13 draft147 profile; whole unfinished Attempts are authoritative"]
fn genuine_unfinished_assignment_and_practice_drafts_keep_all358_records_and_entire_attempts() {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = all();
    let raw = evidence::observed("writer13-drafts.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    let mut p = progress::decode(&raw, &all).unwrap();
    assert_eq!(value(&p), value(&old));
    assert_eq!(p.attempts.len(), 147);
    assert_eq!(count(&p), 358);
    assert_eq!(
        revision_counts(&p),
        BTreeMap::from([
            (6, 33),
            (7, 37),
            (8, 40),
            (9, 43),
            (10, 47),
            (11, 50),
            (12, 53),
            (13, 55)
        ])
    );
    let before = multisets(&p);
    for id in ["shell-command-plan", "shell-handover"] {
        let a = p.attempts.get_mut(id).unwrap();
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert!(matches!(a.state, State::Script { run: None, .. }));
        let l = a
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap();
        assert!(l.example_result.is_none());
        workspace::prepare(a);
        assert_eq!(value(a), value(&old.attempts[id]));
        assert!(!progress::current_solved(a, s));
    }
    assert_eq!(multisets(&p), before);
    let resumed = save(&raw, &p, &all);
    assert_eq!(multisets(&resumed), before);
    assert_eq!(count(&resumed), 358);
    for id in ["shell-command-plan", "shell-handover"] {
        assert_eq!(value(&resumed.attempts[id]), value(&old.attempts[id]));
    }
}
#[test]
#[ignore = "Requires fixed genuine13 profiles and genuine12 inputs, never regenerated history"]
fn both_captured13_profiles_retain_every_prior12_record_and_exact_non_script_and_unfinished_work() {
    for (input, output, prior_count, new_count, executed) in [
        (
            "input12-original.json",
            "writer13-original.json",
            309,
            366,
            57,
        ),
        ("input12-drafts.json", "writer13-drafts.json", 303, 358, 55),
    ] {
        let old: Progress = serde_json::from_slice(&evidence::observed(input)).unwrap();
        let captured: Progress = serde_json::from_slice(&evidence::observed(output)).unwrap();
        assert_eq!(count(&old), prior_count);
        assert_eq!(count(&captured), new_count);
        assert_eq!(old.attempts.len(), 145);
        assert_eq!(captured.attempts.len(), 147);
        let oldrows = multisets(&old);
        let currentrows = multisets(&captured);
        let mut added = 0;
        for (id, original) in oldrows {
            let mut remaining = currentrows[&id].clone();
            for record in original {
                let at = remaining
                    .iter()
                    .position(|r| r == &record)
                    .expect("Every old record occurrence survives");
                remaining.remove(at);
            }
            added += remaining.len();
            for record in remaining {
                let record: Record = serde_json::from_str(&record).unwrap();
                assert_eq!(record.engine_revision, 13);
                assert_eq!(
                    record.source,
                    match &captured.attempts[&id].state {
                        State::Script { source, .. } => source,
                        _ => panic!(),
                    }
                    .as_str()
                );
            }
            if !matches!(old.attempts[&id].state, State::Script { .. }) {
                assert_eq!(value(&old.attempts[&id]), value(&captured.attempts[&id]));
            }
        }
        for id in ["shell-loop-intake", "shell-loop-fallback"] {
            assert!(!old.attempts.contains_key(id));
            let a = &captured.attempts[id];
            assert!(matches!(a.state, State::Script { .. }));
            let rr = records(a);
            assert_eq!(rr.len(), 1);
            assert_eq!(rr[0].engine_revision, 13);
            added += rr.len();
        }
        assert_eq!(added, executed);
        if input.contains("drafts") {
            for id in ["shell-command-plan", "shell-handover"] {
                assert_eq!(value(&captured.attempts[id]), value(&old.attempts[id]));
            }
        }
    }
}
#[test]
#[ignore = "Requires fixed genuine13 profile; all43 current missing nonScript contexts remain absent"]
fn current190_absent_non_script_identities_never_gain_answers_favorites_or_mastery() {
    let all = all();
    let raw = evidence::observed("writer13-original.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    let loaded = progress::decode(&raw, &all).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    let missing = [
        "lpic2-web-routing",
        "lpic2-web-pages",
        "lpic2-web-access",
        "lpic2-web-handover",
        "ccna-nat-static",
        "ccna-nat-pool",
        "ccna-nat-handover",
        "lpic1-rpm-provider",
        "lpic1-rpm-epoch",
        "lpic1-rpm-payload",
        "lpic1-rpm-handover",
        "lpic2-nfs-reference",
        "lpic2-nfs-publication",
        "lpic2-nfs-handover",
        "lpic2-nfs-capstone",
        "ccna-lacp-initiation",
        "ccna-lacp-member-service",
        "ccna-lacp-resilient-handover",
        "lpic1-cron-weekday-report",
        "lpic1-cron-environment-output",
        "lpic1-cron-access-retirement",
        "lpic1-cron-calendar-handover",
        "lpic2-backup-selective-recovery",
        "lpic2-backup-incremental-chain",
        "lpic2-backup-snapshot-branch",
        "lpic2-backup-recovery-handover",
        "ccna-physical-media-span",
        "ccna-physical-duplex-diagnosis",
        "ccna-physical-counter-baseline",
        "ccna-physical-uplink-handover",
        "ccna-private-office-plan",
        "lpic1-at-report",
        "lpic1-at-capture-repair",
        "lpic1-at-owner-queue",
        "lpic1-at-handover",
        "lpic2-lvm-free-space",
        "lpic2-lvm-add-pv",
        "lpic2-lvm-filesystem-gap",
        "lpic2-lvm-handover",
        "ccna-ntp-client-chain",
        "ccna-ntp-cycle-repair",
        "ccna-ntp-asymmetric-delivery",
        "ccna-ntp-policy-handover",
    ];
    for id in missing {
        assert!(ids.insert(id));
        let s = all
            .iter()
            .find(|s| s.id == id)
            .expect("Exact current authored identity");
        assert!(!matches!(s.initial, State::Script { .. }));
        assert!(!matches!(s.goal, Goal::Script { .. }));
        assert!(!old.attempts.contains_key(id) && !loaded.attempts.contains_key(id));
        assert!(!old.favorites.contains(id) && !loaded.favorites.contains(id));
        assert!(!progress::current_solved(&Attempt::new(s, 1), s));
    }
    assert_eq!(ids.len(), 43);
    assert_eq!(
        all.iter()
            .filter(|s| !old.attempts.contains_key(&s.id))
            .map(|s| s.id.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        ids
    );
    assert_eq!(value(&old), value(&loaded));
    assert_eq!(
        value(&old.migration_receipts),
        value(&loaded.migration_receipts)
    );
}
#[test]
#[ignore = "Requires genuine13 data; all new14 records are obtained from actual current VM execution"]
fn only_real_current_run_and_separate_practice_refresh_credit_then_edits_break_it_and_reset_is_selected()
 {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = all();
    let raw = evidence::observed("writer13-original.json");
    let mut p = progress::decode(&raw, &all).unwrap();
    let id = "shell-handover";
    let s = all.iter().find(|s| s.id == id).unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let lesson = s.teaching.as_ref().unwrap();
    let before = value(&p);
    let clock = p.clock;
    let keep_receipts = value(&p.migration_receipts);
    let keep_favorites = p.favorites.clone();
    let a = p.attempts.get_mut(id).unwrap();
    assert!(!progress::current_solved(a, s));
    workspace::prepare(a);
    let saved_history = records(a);
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    assert_eq!(l.example_revision, 13);
    let example = lesson.example.clone();
    l.example_source = example.clone();
    l.example_revision = 14;
    l.example_result = Some(sliced(&example, &lesson.fixture, 7));
    assert!(lesson.fixture.accepts(l.example_result.as_ref().unwrap()));
    l.view = script_teaching::View::Example;
    assert!(!progress::current_solved(a, s));
    assert_eq!(records(a), saved_history);
    let source = s.solution.join("\n");
    let record = Record {
        engine_revision: 14,
        source: source.clone(),
        results: cases.iter().map(|c| sliced(&source, c, 7)).collect(),
    };
    assert!(record.current() && record.valid());
    a.state = State::Script {
        source: source.clone(),
        run: Some(record),
    };
    assert!(progress::current_solved(a, s));
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("\nprintf wrong\\n\n");
    assert!(!progress::current_solved(a, s));
    let wrong = "printf '%s\\n' wrong";
    a.state = State::Script {
        source: wrong.into(),
        run: Some(Record {
            engine_revision: 14,
            source: wrong.into(),
            results: cases.iter().map(|c| sliced(wrong, c, 1)).collect(),
        }),
    };
    assert!(!progress::current_solved(a, s));
    let source = s.solution.join("\n");
    a.state = State::Script {
        source: source.clone(),
        run: Some(Record {
            engine_revision: 14,
            source: source.clone(),
            results: cases.iter().map(|c| sliced(&source, c, 32)).collect(),
        }),
    };
    assert!(progress::current_solved(a, s));
    let resumed = save(&raw, &p, &all);
    assert!(progress::current_solved(&resumed.attempts[id], s));
    assert_eq!(value(&p.migration_receipts), keep_receipts);
    assert_eq!(p.clock, clock);
    assert_eq!(p.favorites, keep_favorites);
    for (other, attempt) in before["attempts"].as_object().unwrap() {
        if other != id {
            assert_eq!(value(&p.attempts[other]), *attempt);
        }
    }
    let prior = value(&p);
    let visited = p.attempts[id].visited;
    let mut reset = Attempt::new(s, visited);
    workspace::prepare(&mut reset);
    script_teaching::prepare(&mut reset, lesson, true);
    p.attempts.insert(id.into(), reset);
    assert_eq!(p.attempts[id].visited, visited);
    let l = p.attempts[id]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.view, script_teaching::View::Introduction);
    assert_eq!(l.example_revision, 14);
    assert!(l.example_result.is_none());
    assert!(!progress::current_solved(&p.attempts[id], s));
    assert_eq!(p.clock, clock);
    assert_eq!(p.favorites, keep_favorites);
    assert_eq!(value(&p.migration_receipts), keep_receipts);
    for (other, attempt) in prior["attempts"].as_object().unwrap() {
        if other != id {
            assert_eq!(value(&p.attempts[other]), *attempt);
        }
    }
    let reset_resumed = save(&raw, &p, &all);
    assert!(!progress::current_solved(&reset_resumed.attempts[id], s));
    assert_eq!(value(&reset_resumed.attempts[id]), value(&p.attempts[id]));
}
