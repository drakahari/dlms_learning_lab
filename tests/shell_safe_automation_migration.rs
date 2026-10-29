//! Current11 boundaries use actual9 evidence ONLY in explicit private replay.
//! The default tests never fabricate historical9 outcomes or implied revisions.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell, workspace,
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize},
};
#[path = "support/actual9_evidence.rs"]
mod evidence;
fn empty() -> Progress {
    Progress {
        schema_version: 1,
        ..Default::default()
    }
}
fn slot(path: &Path) -> std::path::PathBuf {
    path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION))
}
fn history(profile: &Progress, include_main: bool) -> BTreeMap<String, Vec<shell::Record>> {
    profile
        .attempts
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
            if include_main
                && let State::Script { run: Some(r), .. } = &a.state
                && !rows.contains(r)
            {
                rows.push(r.clone());
            }
            (id.clone(), rows)
        })
        .collect()
}
fn private() -> (Vec<u8>, Progress, Vec<pack::Scenario>) {
    assert_eq!(shell::EVALUATOR_REVISION, 11);
    let raw = evidence::original9();
    let original: Progress = serde_json::from_slice(&raw).unwrap();
    let labs = pack::bundled().unwrap();
    assert_eq!(labs.len(), 168);
    let later = labs
        .iter()
        .filter(|s| !original.attempts.contains_key(&s.id))
        .map(|s| s.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        later,
        std::collections::BTreeSet::from([
            "lpic2-backup-selective-recovery",
            "lpic2-backup-incremental-chain",
            "lpic2-backup-snapshot-branch",
            "lpic2-backup-recovery-handover",
            "ccna-lacp-initiation",
            "ccna-lacp-member-service",
            "ccna-lacp-resilient-handover",
            "shell-preserve-lines",
            "shell-pipeline-status",
            "shell-safe-handover",
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
            "shell-command-options",
            "shell-command-dispatch",
            "shell-command-plan",
            "shell-optional-policy",
            "shell-optional-report",
            "shell-optional-handover",
            "lpic2-nfs-reference",
            "lpic2-nfs-publication",
            "lpic2-nfs-handover",
            "lpic2-nfs-capstone",
            "lpic1-cron-weekday-report",
            "lpic1-cron-environment-output",
            "lpic1-cron-access-retirement",
            "lpic1-cron-calendar-handover",
        ])
    );
    assert_eq!(original.attempts.len(), 133);
    for (id, scripting) in [
        ("shell-command-options", true),
        ("shell-command-dispatch", true),
        ("shell-command-plan", true),
        ("shell-optional-policy", true),
        ("shell-optional-report", true),
        ("shell-optional-handover", true),
        ("lpic2-nfs-reference", false),
        ("lpic2-nfs-publication", false),
        ("lpic2-nfs-handover", false),
        ("lpic2-nfs-capstone", false),
    ] {
        let source = labs.iter().find(|s| s.id == id).unwrap();
        if scripting {
            assert!(matches!(source.initial, State::Script { .. }));
            assert!(matches!(source.goal, Goal::Script { .. }));
        } else {
            assert!(matches!(source.initial, State::Nfs { .. }));
            assert!(matches!(source.goal, Goal::Nfs { .. }));
        }
        assert!(
            !original.attempts.contains_key(id) && !original.favorites.contains(id),
            "Original actual9 must retain absent later attempts and credit"
        );
    }
    for id in [
        "lpic1-cron-weekday-report",
        "lpic1-cron-environment-output",
        "lpic1-cron-access-retirement",
        "lpic1-cron-calendar-handover",
    ] {
        let source = labs.iter().find(|s| s.id == id).unwrap();
        assert!(
            matches!(source.initial, State::Cron { .. })
                && matches!(source.goal, Goal::Cron { .. })
        );
        assert!(!original.attempts.contains_key(id) && !original.favorites.contains(id));
    }
    (raw, original, labs)
}
// Progress's portable Serialize intentionally omits main-only provenance.
// Full replay assertions restore that field explicitly, including None, instead
// of accidentally comparing only a portable projection.
fn full(p: &Progress) -> serde_json::Value {
    let mut value = serde_json::to_value(p).unwrap();
    value["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    value
}
fn expected_after_actual9_target_save(
    candidate: &Progress,
    target: &Progress,
) -> serde_json::Value {
    let mut value = full(candidate);
    let mut receipts = target
        .migration_receipts
        .clone()
        .expect("Pinned actual9 target receipts");
    assert!(receipts.workspace);
    assert_eq!(receipts.shell, vec![7, 8, 9]);
    receipts.shell.push(shell::EVALUATOR_REVISION);
    value["migration_receipts"] = serde_json::to_value(receipts).unwrap();
    value
}
fn count_history(p: &Progress) -> BTreeMap<u32, usize> {
    let mut counts = BTreeMap::new();
    for record in history(p, false).values().flatten() {
        *counts.entry(record.engine_revision).or_insert(0) += 1;
    }
    counts
}
fn unchanged_except_execution_and_history(now: &Progress, old: &Progress) {
    let mut a = full(now);
    let b = full(old);
    // Explicit known normalization only: current-invalid assignment runs become
    // exact history; no draft/transcript/favorite/visited/state field is omitted.
    for (id, previous) in b["attempts"].as_object().unwrap() {
        if previous["state"]["kind"] == "Script" {
            let current = &mut a["attempts"][id];
            current["state"]["run"] = previous["state"]["run"].clone();
            if let Some(w) = current["workspace"].as_object_mut() {
                for field in ["script_history", "script_history_more"] {
                    if let Some(v) = previous["workspace"].get(field) {
                        w.insert(field.into(), v.clone());
                    } else {
                        w.remove(field);
                    }
                }
            }
        }
    }
    evidence::exact(&a, &b);
}
#[test]
fn empty_legacy9_record_shape_is_explicitly_ungraded_not_a_fabricated_execution() {
    assert_eq!(shell::EVALUATOR_REVISION, 11);
    let labs = pack::bundled().unwrap();
    let lab = labs.iter().find(|s| s.id == "shell-arguments").unwrap();
    let mut attempt = Attempt::new(lab, 1);
    let State::Script { source, run } = &mut attempt.state else {
        panic!()
    };
    *run = Some(shell::Record {
        engine_revision: 9,
        source: source.clone(),
        results: vec![],
    });
    assert!(!progress::current_solved(&attempt, lab));
    workspace::prepare(&mut attempt);
    assert!(!progress::current_solved(&attempt, lab));
    assert!(matches!(attempt.state, State::Script { run: None, .. }));
    let retained = attempt
        .workspace
        .as_ref()
        .unwrap()
        .script_history
        .as_ref()
        .unwrap();
    assert_eq!(retained.engine_revision, 9);
    assert!(retained.results.is_empty());
}
#[test]
fn direct7_to_current_retains_exact_source7_and_before8_without_inventing_intermediate_receipts() {
    let raw = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    let labs = pack::bundled().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    std::fs::write(&path, raw).unwrap();
    let before8 = path.with_extension("before-shell-v8.json");
    std::fs::write(&before8, raw).unwrap();
    progress::save(&path, &empty()).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), raw);
    assert_eq!(std::fs::read(&before8).unwrap(), raw);
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
#[ignore = "Requires exact retained private actual9 COMPLETE; run only through explicit guarded replay gate"]
fn actual9_open_prepare_preserves_all6_7_8_9_histories_drafts_and_unrelated_contexts() {
    let (raw, original, labs) = private();
    let mut loaded = progress::decode(&raw, &labs).unwrap();
    evidence::exact(&history(&loaded, false), &history(&original, false));
    // These are the exact loaded133 contexts. All35 later tasks stay absent; opening
    // an older profile must not fabricate Attempts or historical answers.
    for lab in labs
        .iter()
        .filter(|lab| original.attempts.contains_key(&lab.id))
    {
        let a = loaded.attempts.get_mut(&lab.id).unwrap();
        if matches!(a.state, State::Script { .. }) {
            assert!(!progress::current_solved(a, lab));
            workspace::prepare(a);
            assert!(!progress::current_solved(a, lab));
            assert!(matches!(a.state, State::Script { run: None, .. }));
        }
    }
    evidence::exact(&history(&loaded, false), &history(&original, true));
    assert_eq!(
        count_history(&loaded),
        BTreeMap::from([(6, 33), (7, 37), (8, 40), (9, 43)])
    );
    unchanged_except_execution_and_history(&loaded, &original);
}
#[test]
#[ignore = "Requires exact retained private actual9 COMPLETE; explicit guarded replay only"]
fn actual9_target_before10_exact_bytes_old_slots_and_receipts_are_preserved() {
    let (raw, original, labs) = private();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    let before8 = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    let before9 = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    std::fs::write(path.with_extension("before-shell-v8.json"), before8).unwrap();
    std::fs::write(path.with_extension("before-shell-v9.json"), before9).unwrap();
    let mut loaded = progress::load(&path, &labs).unwrap();
    for a in loaded.attempts.values_mut() {
        if matches!(a.state, State::Script { .. }) {
            workspace::prepare(a);
        }
    }
    let caller = full(&loaded);
    progress::save(&path, &loaded).unwrap();
    evidence::exact(&full(&loaded), &caller);
    assert_eq!(std::fs::read(slot(&path)).unwrap(), raw);
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        before8
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v9.json")).unwrap(),
        before9
    );
    let disk = progress::load(&path, &labs).unwrap();
    assert_eq!(
        disk.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, shell::EVALUATOR_REVISION]
    );
    evidence::exact(&history(&disk, false), &history(&original, true));
    progress::save(&path, &disk).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), raw);
}
#[test]
#[ignore = "Requires exact retained private actual9 COMPLETE; explicit guarded replay only"]
fn actual9_reset_first_practice_only_and_conflicting_valid_slot_never_skip_raw_target() {
    let (raw, _, labs) = private();
    for practice_only in [false, true] {
        let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        if practice_only {
            for a in value["attempts"].as_object_mut().unwrap().values_mut() {
                if a["state"]["kind"] == "Script" {
                    a["state"]["run"] = serde_json::Value::Null;
                }
            }
        }
        let bytes = if practice_only {
            serde_json::to_vec_pretty(&value).unwrap()
        } else {
            raw.clone()
        };
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        std::fs::write(&path, &bytes).unwrap();
        let occupied = serde_json::to_vec(&empty()).unwrap();
        std::fs::write(slot(&path), &occupied).unwrap();
        assert!(progress::save(&path, &empty()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(slot(&path)).unwrap(), occupied);
        let reset = directory.path().join("reset.json");
        std::fs::write(&reset, &bytes).unwrap();
        progress::save(&reset, &empty()).unwrap();
        assert_eq!(std::fs::read(slot(&reset)).unwrap(), bytes);
        assert_eq!(
            progress::load(&reset, &labs)
                .unwrap()
                .migration_receipts
                .unwrap()
                .shell,
            vec![7, 8, 9, shell::EVALUATOR_REVISION]
        );
    }
}
#[test]
#[ignore = "Requires exact retained private actual9 COMPLETE; explicit guarded replay only"]
fn actual9_export_preview_cancel_confirm_restore_runs_current_only_explicitly_and_keeps_histories()
{
    let (raw, original, labs) = private();
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("export.json");
    let target = directory.path().join("progress.json");
    std::fs::write(&target, &raw).unwrap();
    backup::export(&input, &original, &labs).unwrap();
    let export = std::fs::read(&input).unwrap();
    assert!(
        backup::inspect(&input, &labs, &AtomicBool::new(true), &AtomicUsize::new(0))
            .err()
            .unwrap()
            .contains("cancelled")
    );
    assert_eq!(std::fs::read(&target).unwrap(), raw);
    assert!(!slot(&target).exists());
    let candidate =
        backup::inspect(&input, &labs, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(candidate.rechecked, 80);
    assert!(candidate.progress.migration_receipts.is_none());
    evidence::exact(
        &history(&candidate.progress, false),
        &history(&original, true),
    );
    for (id, a) in &candidate.progress.attempts {
        if let State::Script {
            source,
            run: Some(run),
        } = &a.state
        {
            assert_eq!(run.engine_revision, shell::EVALUATOR_REVISION);
            assert_eq!(&run.source, source);
            let lab = labs.iter().find(|s| &s.id == id).unwrap();
            let Goal::Script { cases } = &lab.goal else {
                panic!()
            };
            let expected = cases
                .iter()
                .map(|c| shell::run(source, c))
                .collect::<Vec<_>>();
            evidence::exact(&run.results, &expected);
        }
    }
    assert_eq!(std::fs::read(&target).unwrap(), raw);
    let restored = backup::restore(&target, &candidate, &labs).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), raw);
    assert_eq!(std::fs::read(slot(&target)).unwrap(), raw);
    assert_eq!(std::fs::read(&input).unwrap(), export);
    let disk = progress::load(&target, &labs).unwrap();
    let expected_full = expected_after_actual9_target_save(&candidate.progress, &original);
    evidence::exact(&full(&disk), &expected_full);
    assert_eq!(
        disk.migration_receipts.unwrap().shell,
        vec![7, 8, 9, shell::EVALUATOR_REVISION]
    );
}
#[test]
#[ignore = "Requires exact retained private actual9 COMPLETE; explicit guarded replay only"]
fn actual9_selected_reset_retains_other_attempts_favorites_and_all_their_records() {
    let (raw, original, labs) = private();
    let mut loaded = progress::decode(&raw, &labs).unwrap();
    let old = full(&loaded);
    let lab = labs.iter().find(|s| s.id == "shell-arguments").unwrap();
    loaded
        .attempts
        .insert(lab.id.clone(), Attempt::new(lab, loaded.clock + 1));
    let now = full(&loaded);
    for (id, a) in old["attempts"].as_object().unwrap() {
        if id != &lab.id {
            evidence::exact(&now["attempts"][id], a);
        }
    }
    evidence::exact(&now["favorites"], &old["favorites"]);
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], lab));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    progress::save(&path, &loaded).unwrap();
    assert_eq!(std::fs::read(slot(&path)).unwrap(), raw);
    let disk = progress::load(&path, &labs).unwrap();
    let expected_full = expected_after_actual9_target_save(&loaded, &original);
    evidence::exact(&full(&disk), &expected_full);
    // Save stamps disk only; the in-memory caller retains actual9 provenance.
    evidence::exact(&full(&loaded), &now);
}
