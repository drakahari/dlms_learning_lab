//! Genuine evaluator12 baseline comparisons; never synthesize historical runs.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Progress},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
use serde_json::Value;
#[path = "support/ntp_registration.rs"]
mod ntp_registration;
use std::collections::BTreeMap;
#[path = "support/actual12_evidence.rs"]
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
fn v(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).unwrap()
}
fn sliced(source: &str, c: &Case, n: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(n)
    }
    panic!("Current comparison exceeded bounded polling")
}
fn histories(p: &Progress) -> BTreeMap<String, Vec<Record>> {
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
                rows.push(r.clone())
            }
            (id.clone(), rows)
        })
        .collect()
}
fn same_metadata(mut a: Value, mut b: Value) {
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
#[ignore = "Requires immutable private genuine12 before13 data and seals; explicit guarded replay"]
fn all55_actual12_records420_cases55_examples_match_at_four_cooperative_slices() {
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("actual12-authored.jsonl");
    let rows: Vec<Authored> = raw
        .split(|b| *b == b'\n')
        .filter(|r| !r.is_empty())
        .map(|r| serde_json::from_slice(r).unwrap())
        .collect();
    assert_eq!(rows.len(), 55);
    let mut count = 0;
    for r in rows {
        let s = all.iter().find(|s| s.id == r.id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(v(cases), v(&r.cases));
        assert_eq!(s.solution.join("\n"), r.source);
        assert_eq!(r.record.source, r.source);
        assert_eq!(r.record.engine_revision, 12);
        assert!(r.record.valid());
        assert!(!r.record.current());
        assert_eq!(r.record.results.len(), cases.len());
        for (c, old) in cases.iter().zip(&r.record.results) {
            assert!(c.accepts(old));
            for n in [1, 7, 32, 64] {
                assert_eq!(
                    sliced(&r.source, c, n),
                    *old,
                    "{}/{} slice{n}",
                    r.id,
                    c.name
                )
            }
        }
        count += cases.len();
        let t = s.teaching.as_ref().unwrap();
        assert_eq!(t.example, r.example_source);
        assert_eq!(v(&t.fixture), v(&r.example_case));
        assert_eq!(r.example_revision, 12);
        assert!(r.example_case.accepts(&r.example_outcome));
        for n in [1, 7, 32, 64] {
            assert_eq!(
                sliced(&r.example_source, &r.example_case, n),
                r.example_outcome
            )
        }
    }
    assert_eq!(count, 420);
}
#[test]
#[ignore = "Requires immutable private genuine12 before13 data and seals; explicit guarded replay"]
fn actual145_contexts_keep_all_records_receipts_and_non_script_attempts_without_inventing_answers()
{
    assert_eq!(shell::EVALUATOR_REVISION, 14);
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer12-original.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    let mut p = progress::decode(&raw, &all).unwrap();
    assert_eq!(v(&p), v(&old));
    assert_eq!(p.attempts.len(), 145);
    assert_eq!(
        p.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 11, 12]
    );
    let prior = histories(&p);
    let (mut scripts, mut other) = (0, 0);
    for (id, a) in &mut p.attempts {
        let b = &old.attempts[id];
        let s = all.iter().find(|s| &s.id == id).unwrap();
        if let State::Script { run: Some(r), .. } = &b.state {
            scripts += 1;
            assert_eq!(r.engine_revision, 12);
            assert!(!progress::current_solved(a, s));
            workspace::prepare(a);
            assert!(matches!(a.state, State::Script { run: None, .. }));
            assert!(!progress::current_solved(a, s));
            assert_eq!(a.visited, b.visited);
            assert_eq!(a.step, b.step);
            assert_eq!(a.hints, b.hints);
            same_metadata(v(a), v(b));
        } else {
            other += 1;
            assert_eq!(v(a), v(b));
        }
    }
    assert_eq!((scripts, other), (55, 90));
    assert_eq!(histories(&p), prior);
    assert_eq!(p.favorites, old.favorites);
    assert_eq!(p.clock, old.clock);
    for s in &all {
        if !old.attempts.contains_key(&s.id) {
            assert!(!p.attempts.contains_key(&s.id));
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    let caller = v(&p);
    let receipts = v(&p.migration_receipts);
    progress::save(&path, &p).unwrap();
    assert_eq!(v(&p), caller);
    assert_eq!(v(&p.migration_receipts), receipts);
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v14.json")).unwrap(),
        raw
    );
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(v(&loaded), caller);
    assert_eq!(
        loaded.migration_receipts.unwrap().shell,
        vec![7, 8, 9, 10, 11, 12, 14]
    );
}
#[test]
#[ignore = "Requires immutable private genuine12 before13 data and seals; explicit guarded replay"]
fn two_genuine_unfinished_assignment_and_practice_drafts_remain_entirely_unchanged() {
    let all = pack::bundled().unwrap();
    let raw = evidence::observed("writer12-drafts.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    let mut p = progress::decode(&raw, &all).unwrap();
    assert_eq!(v(&old), v(&p));
    let before = histories(&p);
    for id in ["shell-command-plan", "shell-handover"] {
        let a = p.attempts.get_mut(id).unwrap();
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert!(matches!(a.state, State::Script { run: None, .. }));
        workspace::prepare(a);
        assert_eq!(v(a), v(&old.attempts[id]));
        assert!(!progress::current_solved(a, s));
    }
    assert_eq!(histories(&p), before);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    progress::save(&path, &p).unwrap();
    let resumed = progress::load(&path, &all).unwrap();
    for id in ["shell-command-plan", "shell-handover"] {
        assert_eq!(v(&resumed.attempts[id]), v(&old.attempts[id]));
    }
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v14.json")).unwrap(),
        raw
    );
    assert_eq!(
        resumed.migration_receipts.unwrap().shell,
        vec![7, 8, 9, 10, 11, 12, 14]
    );
}

#[test]
#[ignore = "Requires genuine12 sealed profile; explicit guarded current14 migration"]
fn nine_real_new_non_script_and_two_loop_assignments_have_no_invented_answers() {
    let all = pack::bundled().unwrap();
    assert_eq!(ntp_registration::before_scenarios(&all).len(), 186);
    assert_eq!(all.len(), 190);
    let raw = evidence::observed("writer12-original.json");
    let old: Progress = serde_json::from_slice(&raw).unwrap();
    let current = progress::decode(&raw, &all).unwrap();
    let nine = [
        "ccna-physical-media-span",
        "ccna-physical-duplex-diagnosis",
        "ccna-physical-counter-baseline",
        "ccna-physical-uplink-handover",
        "ccna-private-office-plan",
        "lpic1-at-report",
        "lpic1-at-capture-repair",
        "lpic1-at-owner-queue",
        "lpic1-at-handover",
    ];
    assert_eq!(
        all[171..180]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        nine
    );
    for s in &all[171..] {
        assert!(!old.attempts.contains_key(&s.id));
        assert!(!current.attempts.contains_key(&s.id));
        assert!(!old.favorites.contains(&s.id));
        assert!(!current.favorites.contains(&s.id));
    }
    for id in [
        "ccna-ntp-client-chain",
        "ccna-ntp-cycle-repair",
        "ccna-ntp-asymmetric-delivery",
        "ccna-ntp-policy-handover",
    ] {
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert!(matches!(s.initial, State::Ntp { .. }));
        assert!(!current.attempts.contains_key(id) && !current.favorites.contains(id));
        assert!(!old.attempts.contains_key(id) && !old.favorites.contains(id));
        assert!(!dlms_learning_lab::progress::current_solved(
            &dlms_learning_lab::progress::Attempt::new(s, 0),
            s
        ));
    }
    for id in [
        "lpic2-lvm-free-space",
        "lpic2-lvm-add-pv",
        "lpic2-lvm-filesystem-gap",
        "lpic2-lvm-handover",
    ] {
        assert!(!old.attempts.contains_key(id) && !current.attempts.contains_key(id));
        assert!(!old.favorites.contains(id) && !current.favorites.contains(id));
        let source = all.iter().find(|s| s.id == id).unwrap();
        assert!(matches!(source.initial, State::Lvm { .. }));
        assert!(!progress::current_solved(
            &progress::Attempt::new(source, 1),
            source
        ));
    }
    assert_eq!(current.attempts.len(), 145);
    assert_eq!(v(&current), v(&old));
    assert_eq!(v(&current.migration_receipts), v(&old.migration_receipts));
    for (s, id) in all[180..182]
        .iter()
        .zip(["shell-loop-intake", "shell-loop-fallback"])
    {
        assert_eq!(s.id, id);
        assert!(matches!(s.initial, State::Script { .. }));
        assert!(matches!(s.goal, Goal::Script { .. }));
    }
}
