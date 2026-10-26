//! Prospective revision9 checks from exact actually captured writer8 bytes.
//! New9 outcomes exist only when these tests explicitly execute the current VM.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell, workspace,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize},
};
#[path = "../examples/support/getopts_digest.rs"]
mod digest;
const ACTUAL8: &[u8] = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
const ACTUAL7: &[u8] = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
fn empty() -> Progress {
    Progress {
        schema_version: 1,
        ..Default::default()
    }
}
fn labs() -> Vec<pack::Scenario> {
    assert_eq!(shell::EVALUATOR_REVISION, 9);
    pack::bundled().unwrap()
}
fn raw8() -> Progress {
    serde_json::from_slice(ACTUAL8).unwrap()
}
fn snapshot(p: &Progress) -> serde_json::Value {
    serde_json::to_value(p).unwrap()
}
fn histories(p: &Progress) -> BTreeMap<String, Vec<shell::Record>> {
    p.attempts
        .iter()
        .map(|(id, a)| {
            (
                id.clone(),
                a.workspace
                    .as_ref()
                    .into_iter()
                    .flat_map(|w| {
                        w.script_history
                            .as_ref()
                            .into_iter()
                            .chain(&w.script_history_more)
                    })
                    .cloned()
                    .collect(),
            )
        })
        .collect()
}
fn with_main_records(p: &Progress) -> BTreeMap<String, Vec<shell::Record>> {
    let mut values = histories(p);
    for (id, a) in &p.attempts {
        if let State::Script {
            run: Some(record), ..
        } = &a.state
        {
            let target = values.get_mut(id).unwrap();
            if !target.contains(record) {
                target.push(record.clone());
            }
        }
    }
    values
}
fn slot(path: &Path) -> std::path::PathBuf {
    path.with_extension("before-shell-v9.json")
}
fn inspect(
    path: &Path,
    all: &[pack::Scenario],
    cancelled: bool,
) -> Result<backup::Candidate, String> {
    backup::inspect(path, all, &AtomicBool::new(cancelled), &AtomicUsize::new(0))
}
fn assert_drafts_and_unrelated(p: &Progress) {
    let before = raw8();
    assert_eq!(p.attempts.len(), 116);
    assert_eq!(p.favorites, before.favorites);
    for (id, old) in before.attempts {
        let now = &p.attempts[&id];
        if let (State::Script { source: a, .. }, State::Script { source: b, .. }) =
            (&now.state, &old.state)
        {
            assert_eq!(a, b, "assignment draft {id}");
            assert_eq!(
                now.workspace.as_ref().unwrap().command,
                old.workspace.as_ref().unwrap().command
            );
            let a = now
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap();
            let b = old
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap();
            assert_eq!(a.example_source, b.example_source, "separate draft {id}");
            assert_eq!(a.view, b.view);
            assert_eq!(now.scenario_version, old.scenario_version);
            assert_eq!(now.transcript, old.transcript);
            assert_eq!(now.visited, old.visited);
            assert_eq!(now.challenge, old.challenge);
        } else {
            assert_eq!(
                serde_json::to_value(now).unwrap(),
                serde_json::to_value(old).unwrap(),
                "unrelated {id}"
            );
        }
    }
}
#[test]
fn exact_genuine8_fixture_and_all40_definitions246_cases_are_pinned_without_new_credit() {
    assert_eq!(
        digest::hash(ACTUAL8),
        "88c07f20276ba1555eaad886ec2a58eff3d3b3cb3cef2bf3108be64a63bacd26"
    );
    assert_eq!(ACTUAL8.len(), 396_309);
    let provenance: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/shell-v8-synthetic-writer-provenance.json"
    ))
    .unwrap();
    assert_eq!(provenance["sha256"], digest::hash(ACTUAL8));
    let all = labs();
    assert_eq!(all.len(), 133);
    let baseline: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/getopts-before9-authored-inputs.json"
    ))
    .unwrap();
    let originals = baseline["scripts"].as_array().unwrap();
    assert_eq!(originals.len(), 40);
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.goal, Goal::Script { .. }))
            .count(),
        43
    );
    let mut case_count = 0;
    for old in originals {
        let s = all
            .iter()
            .find(|s| s.id == old["id"].as_str().unwrap())
            .unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(serde_json::to_value(cases).unwrap(), old["cases"]);
        assert_eq!(s.version, old["version"].as_u64().unwrap() as u32);
        assert_eq!(s.solution.join("\n"), old["source"].as_str().unwrap());
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(
            lesson.example.as_str(),
            old["example_source"].as_str().unwrap()
        );
        assert_eq!(
            serde_json::to_value(&lesson.fixture).unwrap(),
            old["example_case"]
        );
        case_count += cases.len();
    }
    assert_eq!(case_count, 246);
    let mut p = progress::decode(ACTUAL8, &all).unwrap();
    assert_eq!(p.attempts.len(), 116);
    let missing: BTreeSet<_> = all
        .iter()
        .filter(|s| !p.attempts.contains_key(&s.id))
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        missing,
        BTreeSet::from([
            "lpic2-dns-reverse-investigation",
            "lpic2-dns-reverse-ipv6",
            "lpic2-dns-reverse-handover",
            "shell-options-report",
            "shell-options-jobs",
            "shell-options-publication",
            "ccna-dhcp-held-options",
            "ccna-dhcp-relay-scope",
            "ccna-dhcp-lease-handover",
            "lpic1-package-install",
            "lpic1-package-metadata",
            "lpic1-package-payload",
            "lpic1-package-handover",
            "ccna-stp-root",
            "ccna-stp-local-cost",
            "ccna-stp-parallel",
            "ccna-stp-handover",
        ])
    ); // Later tasks remain absent and ungraded; no invented contexts.
    let original_histories = with_main_records(&p);
    for s in all
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }) && !missing.contains(s.id.as_str()))
    {
        let a = p.attempts.get_mut(&s.id).unwrap();
        assert!(!progress::current_solved(a, s));
        workspace::prepare(a);
        assert!(!progress::current_solved(a, s));
        assert!(matches!(a.state, State::Script { run: None, .. }));
    }
    assert_eq!(histories(&p), original_histories);
    let counts = original_histories
        .values()
        .flatten()
        .fold(BTreeMap::new(), |mut counts, r| {
            *counts.entry(r.engine_revision).or_insert(0) += 1;
            counts
        });
    assert_eq!(counts, BTreeMap::from([(6, 33), (7, 37), (8, 40)]));
    assert_drafts_and_unrelated(&p);
}
#[test]
fn actual8_target_gets_exact_before9_copy_keeps_before8_and_only_commits_actual_receipts() {
    let all = labs();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    std::fs::write(&path, ACTUAL8).unwrap();
    let earlier = path.with_extension("before-shell-v8.json");
    std::fs::write(&earlier, ACTUAL7).unwrap();
    let mut p = progress::load(&path, &all).unwrap();
    for a in p
        .attempts
        .values_mut()
        .filter(|a| matches!(a.state, State::Script { .. }))
    {
        workspace::prepare(a);
    }
    let caller_receipts = serde_json::to_value(&p.migration_receipts).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), ACTUAL8);
    assert_eq!(std::fs::read(&earlier).unwrap(), ACTUAL7);
    assert_eq!(
        serde_json::to_value(&p.migration_receipts).unwrap(),
        caller_receipts
    );
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9]
    );
    assert_eq!(histories(&loaded), with_main_records(&raw8()));
    assert_drafts_and_unrelated(&loaded);
    progress::save(&path, &loaded).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), ACTUAL8);
    assert_eq!(std::fs::read(&earlier).unwrap(), ACTUAL7);
}
#[test]
fn reset_first_and_practice_only8_cannot_skip_target_backup_and_conflicting_valid_slot_is_safe() {
    let all = labs();
    for practice_only in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        let raw = if practice_only {
            let mut value: serde_json::Value = serde_json::from_slice(ACTUAL8).unwrap();
            for a in value["attempts"].as_object_mut().unwrap().values_mut() {
                if a["state"]["kind"] == "Script" {
                    a["state"]["run"] = serde_json::Value::Null;
                }
            }
            serde_json::to_vec_pretty(&value).unwrap() // Disclosed derivative, original actual fixture untouched.
        } else {
            ACTUAL8.to_vec()
        };
        std::fs::write(&path, &raw).unwrap();
        let candidate = empty();
        let occupied = serde_json::to_vec(&empty()).unwrap(); // Valid different schema1, not merely broken JSON.
        std::fs::write(slot(&path), &occupied).unwrap();
        assert!(progress::save(&path, &candidate).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        assert_eq!(std::fs::read(slot(&path)).unwrap(), occupied);
        // New independent target proves reset-first still creates the exact copy.
        let reset = directory.path().join("reset.json");
        std::fs::write(&reset, &raw).unwrap();
        progress::save(&reset, &candidate).unwrap();
        assert_eq!(std::fs::read(slot(&reset)).unwrap(), raw);
        assert_eq!(
            progress::load(&reset, &all)
                .unwrap()
                .migration_receipts
                .unwrap()
                .shell,
            vec![7, 8, 9]
        );
    }
}
#[test]
fn imported_current_marker_never_skips_actual_old8_target_and_direct7_to9_does_not_invent8() {
    let all = labs();
    for (raw, receipts) in [(ACTUAL8, vec![7, 8, 9]), (ACTUAL7, vec![7, 9])] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        std::fs::write(&path, raw).unwrap();
        let mut candidate = empty();
        candidate.migration_receipts = Some(progress::MigrationReceipts {
            workspace: true,
            shell: vec![9],
        });
        progress::save(&path, &candidate).unwrap();
        assert_eq!(std::fs::read(slot(&path)).unwrap(), raw);
        assert_eq!(
            progress::load(&path, &all)
                .unwrap()
                .migration_receipts
                .unwrap()
                .shell,
            receipts
        );
    }
}
#[test]
fn export_inspect_cancel_confirmed_restore_rechecks_only_explicitly_and_retains6_7_8_exactly() {
    let all = labs();
    let original = raw8();
    let expected = with_main_records(&original);
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("export.json");
    backup::export(&input, &original, &all).unwrap();
    let export = std::fs::read(&input).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&export).unwrap();
    assert!(envelope["progress"].get("migration_receipts").is_none());
    let target = directory.path().join("progress.json");
    std::fs::write(&target, ACTUAL8).unwrap();
    assert!(
        inspect(&input, &all, true)
            .err()
            .unwrap()
            .contains("cancelled")
    );
    assert_eq!(std::fs::read(&target).unwrap(), ACTUAL8);
    assert!(!slot(&target).exists());
    let candidate = inspect(&input, &all, false).unwrap();
    assert_eq!(candidate.rechecked, 77); // 40 main +37 saved example outcomes; three examples had no result.
    assert!(candidate.progress.migration_receipts.is_none());
    assert_eq!(histories(&candidate.progress), expected);
    assert_drafts_and_unrelated(&candidate.progress);
    for (id, a) in &candidate.progress.attempts {
        if let State::Script {
            source,
            run: Some(run),
        } = &a.state
        {
            assert_eq!(run.engine_revision, 9);
            assert_eq!(&run.source, source);
            let s = all.iter().find(|s| &s.id == id).unwrap();
            let Goal::Script { cases } = &s.goal else {
                panic!()
            };
            let actual: Vec<_> = cases.iter().map(|c| shell::run(source, c)).collect();
            assert_eq!(run.results, actual); // Explicit VM runs, not fabricated fixture records.
        }
    }
    assert!(!progress::current_solved(
        &candidate.progress.attempts["shell-handover"],
        all.iter().find(|s| s.id == "shell-handover").unwrap()
    ));
    assert_eq!(std::fs::read(&target).unwrap(), ACTUAL8); // Inspection alone is not confirmation.
    let result = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(), ACTUAL8);
    assert_eq!(std::fs::read(slot(&target)).unwrap(), ACTUAL8);
    assert_eq!(std::fs::read(&input).unwrap(), export);
    let loaded = progress::load(&target, &all).unwrap();
    assert_eq!(snapshot(&loaded), snapshot(&candidate.progress));
    assert_eq!(loaded.migration_receipts.unwrap().shell, vec![7, 8, 9]);
}
#[test]
fn selected_reset_retains_all_other_attempts_favorites_histories_and_example_drafts() {
    let all = labs();
    let mut p = progress::decode(ACTUAL8, &all).unwrap();
    let before = snapshot(&p);
    let s = all.iter().find(|s| s.id == "shell-arguments").unwrap();
    p.attempts
        .insert(s.id.clone(), Attempt::new(s, p.clock + 1));
    let now = snapshot(&p);
    for (id, old) in before["attempts"].as_object().unwrap() {
        if id != &s.id {
            assert_eq!(&now["attempts"][id], old, "{id}");
        }
    }
    assert_eq!(now["favorites"], before["favorites"]);
    assert!(!progress::current_solved(&p.attempts[&s.id], s));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let restored = progress::load(&path, &all).unwrap();
    assert_eq!(snapshot(&restored), now);
}
#[test]
fn full_genuine_history_keeps_duplicate8_but_rejects_distinct33_without_mutation() {
    let all = labs();
    let original = raw8();
    let mut records: Vec<_> = original
        .attempts
        .values()
        .filter_map(|a| match &a.state {
            State::Script { run: Some(r), .. } => Some(r.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(records.len(), 40);
    let s = all.iter().find(|s| s.id == "shell-arguments").unwrap();
    let mut a = original.attempts[&s.id].clone();
    let selected = match &a.state {
        State::Script { run: Some(r), .. } => r.clone(),
        _ => panic!(),
    };
    records.retain(|r| r != &selected);
    let w = a.workspace.as_mut().unwrap();
    w.script_history = Some(records.remove(0));
    w.script_history_more = records.into_iter().take(31).collect();
    assert!(!workspace::can_preserve_script_history(&a));
    let before = serde_json::to_value(&a).unwrap();
    workspace::preserve_script_history(&mut a);
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
    let duplicate = a.workspace.as_ref().unwrap().script_history_more[0].clone();
    let State::Script { run, .. } = &mut a.state else {
        panic!()
    };
    *run = Some(duplicate);
    assert!(workspace::can_preserve_script_history(&a));
    workspace::preserve_script_history(&mut a);
    assert_eq!(a.workspace.as_ref().unwrap().script_history_more.len(), 31);
    assert!(matches!(a.state, State::Script { run: None, .. }));
}
