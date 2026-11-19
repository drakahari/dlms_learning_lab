//! In-process progress/transaction fixtures; no learner host commands or native acceptance.
#[path = "support/backup_selection.rs"]
mod fixture;
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    file_backup_teaching as teach,
    file_backup_workspace::{self as work, Scope},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use fixture::*;
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn full(p: &Progress) -> serde_json::Value {
    let mut v = serde_json::to_value(p).unwrap();
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
fn fresh(s: &pack::Scenario) -> Attempt {
    if s.id == ID {
        assert_eq!(s.solution, commands(&review()["reference"]["commands"]));
    }
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    teach::prepare(&mut a, s.file_backup_teaching.as_ref().unwrap(), true).unwrap();
    a
}
fn finish(a: &mut Attempt, s: &pack::Scenario) {
    for c in &s.solution {
        work::perform(a, s, Scope::Assignment, c).unwrap();
    }
}
#[test]
fn separate_practice_success_never_changes_assignment_work_or_earns_credit() {
    let s = source();
    let mut a = fresh(&s);
    a.workspace.as_mut().unwrap().command = "unfinished assignment selection".into();
    let state = a.state.clone();
    let lesson = s.file_backup_teaching.as_ref().unwrap();
    for c in &lesson.example_commands {
        work::perform(&mut a, &s, Scope::Practice, &c.command).unwrap();
    }
    let l = a
        .workspace
        .as_ref()
        .unwrap()
        .file_backup_learning
        .as_ref()
        .unwrap();
    assert!(lesson.definition.checks(&l.state).iter().all(|x| x.0));
    assert_eq!(a.state, state);
    assert!(a.transcript.is_empty());
    assert_eq!(
        a.workspace.as_ref().unwrap().command,
        "unfinished assignment selection"
    );
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn both_drafts_restore_outputs_and_current_success_survive_safe_save_export_and_restore() {
    let s = source();
    let all = pack::ccna_pilot().unwrap();
    let mut a = fresh(&s);
    finish(&mut a, &s);
    assert!(progress::current_solved(&a, &s));
    assert_eq!(flags(&a.state, &s), vec![true; 13]);
    for c in s
        .file_backup_teaching
        .as_ref()
        .unwrap()
        .example_commands
        .iter()
        .take(3)
    {
        work::perform(&mut a, &s, Scope::Practice, &c.command).unwrap();
    }
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .file_backup_learning
        .as_mut()
        .unwrap();
    l.view = View::Example;
    l.command = "tar -tf /example-backups/unfinished.tar".into();
    a.workspace.as_mut().unwrap().command = "tar -tf /backups/unfinished.tar".into();
    let mut p = Progress {
        schema_version: 1,
        clock: 70,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let before = full(&p);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(full(&p), before);
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert!(progress::current_solved(&loaded.attempts[ID], &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    assert!(backup::inspect(&export, &all, &AtomicBool::new(true), &AtomicUsize::new(0)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(
        serde_json::to_value(&candidate.progress).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(candidate.progress.migration_receipts.is_none());
    let result = backup::restore(&path, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(), raw);
    assert_eq!(full(&progress::load(&path, &all).unwrap()), full(&loaded));
}
#[test]
fn selected_typed_reset_preserves_all_four_prior_backups_favorites_receipts_and_clock() {
    let all = pack::ccna_pilot().unwrap();
    let s = source();
    let mut p = Progress {
        schema_version: 1,
        clock: 90,
        ..Default::default()
    };
    for old in &all[164..168] {
        let mut a = fresh(old);
        finish(&mut a, old);
        p.favorites.insert(old.id.clone());
        p.attempts.insert(old.id.clone(), a);
    }
    let mut a = fresh(&s);
    finish(&mut a, &s);
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let before = full(&p);
    let visited = p.attempts[ID].visited;
    let mut reset = Attempt::new(&s, visited);
    workspace::prepare(&mut reset);
    teach::prepare(&mut reset, s.file_backup_teaching.as_ref().unwrap(), true).unwrap();
    p.attempts.insert(ID.into(), reset);
    let mut expected = before;
    expected["attempts"][ID] = serde_json::to_value(&p.attempts[ID]).unwrap();
    assert_eq!(full(&p), expected);
    assert!(!progress::current_solved(&p.attempts[ID], &s));
    for old in &all[164..168] {
        assert!(progress::current_solved(&p.attempts[&old.id], old));
    }
}
#[test]
fn origin_binding_versions_foreign_fields_and_rejected_commands_retain_whole_attempt() {
    let s = source();
    let mut a = fresh(&s);
    finish(&mut a, &s);
    for text in [
        "sh -c host-command",
        "tar -xf /backups/orchard-config.tar -C /personal",
        "tar --to-command=sh -xf /backups/orchard-config.tar",
        "cat /etc/orchard/../../personal",
    ] {
        let before = serde_json::to_vec(&a).unwrap();
        assert!(work::perform(&mut a, &s, Scope::Assignment, text).is_err());
        assert_eq!(serde_json::to_vec(&a).unwrap(), before);
        assert!(progress::current_solved(&a, &s));
    }
    let mut changed = s.clone();
    changed.version += 1;
    let before = serde_json::to_vec(&a).unwrap();
    assert!(work::perform(&mut a, &changed, Scope::Assignment, "pwd").is_err());
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    for old in &pack::ccna_pilot().unwrap()[164..168] {
        let mut other = fresh(old);
        other.workspace.as_mut().unwrap().file_backup_learning =
            a.workspace.as_ref().unwrap().file_backup_learning.clone();
        let before = serde_json::to_vec(&other).unwrap();
        assert!(work::context(&other, old).is_err());
        assert!(work::perform(&mut other, old, Scope::Assignment, "pwd").is_err());
        assert_eq!(serde_json::to_vec(&other).unwrap(), before);
    }
    a.workspace.as_mut().unwrap().service_port = Some("foreign".into());
    let before = serde_json::to_vec(&a).unwrap();
    assert!(work::perform(&mut a, &s, Scope::Assignment, "pwd").is_err());
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    assert!(!progress::current_solved(&a, &s));
    let mut wrong = s.clone();
    wrong.goal = Goal::Subnet;
    assert!(!progress::current_solved(&a, &wrong));
    assert!(matches!(s.initial, State::FileBackup { .. }));
}
#[test]
fn full_attempt_capacity_rejection_is_atomic_for_new_selection() {
    fn fill_history(a: &mut Attempt) {
        a.transcript = vec![String::new(); 120];
        let cap = work::MAX_ATTEMPT;
        let base = work::size(a, cap).unwrap();
        let fill = cap - base - 8;
        for (i, text) in a.transcript.iter_mut().enumerate() {
            *text = "x".repeat(fill / 120 + usize::from(i < fill % 120));
        }
        assert!(work::within(a, cap));
    }
    let s = source();
    let practice_command = "tar -cf /example-backups/willow-config.tar -C /etc willow";
    // First prove the exact command is supported in this exact Willow fixture.
    let mut practice_control = fresh(&s);
    let original_assignment = practice_control.state.clone();
    work::perform(&mut practice_control, &s, Scope::Practice, practice_command).unwrap();
    assert_eq!(practice_control.state, original_assignment);
    assert!(
        practice_control
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .state
            .archives
            .contains_key("/example-backups/willow-config.tar")
    );
    assert!(!progress::current_solved(&practice_control, &s));

    // Practice retains assignment history. Its real archive growth must be
    // rejected by the complete saved-work limit, retaining every caller byte.
    let mut a = fresh(&s);
    fill_history(&mut a);
    let before = serde_json::to_vec(&a).unwrap();
    let error = work::perform(&mut a, &s, Scope::Practice, practice_command).unwrap_err();
    assert!(error.contains("saved backup workspace limit"), "{error}");
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);

    // Assignment commands legitimately prune old assignment history before the
    // bound check. That supported behavior must succeed, not be called refusal.
    let assignment_command = "tar -cf /backups/orchard-config.tar -C /etc orchard";
    let mut assignment_control = fresh(&s);
    let expected_output = work::perform(
        &mut assignment_control,
        &s,
        Scope::Assignment,
        assignment_command,
    )
    .unwrap();
    let mut with_history = fresh(&s);
    fill_history(&mut with_history);
    let previous_history_bytes: usize = with_history.transcript.iter().map(String::len).sum();
    let output =
        work::perform(&mut with_history, &s, Scope::Assignment, assignment_command).unwrap();
    assert_eq!(output, expected_output);
    assert_eq!(with_history.state, assignment_control.state);
    assert_eq!(
        serde_json::to_value(&with_history.workspace).unwrap(),
        serde_json::to_value(&assignment_control.workspace).unwrap()
    );
    let retained_history_bytes: usize = with_history.transcript.iter().map(String::len).sum();
    assert!(retained_history_bytes < previous_history_bytes);
    assert!(with_history.transcript.len() <= 120 && retained_history_bytes <= 262144);
    assert_eq!(
        with_history.transcript.last(),
        assignment_control.transcript.last()
    );
    assert!(work::within(&with_history, work::MAX_ATTEMPT));
    let mut expected = assignment_control;
    expected.transcript = with_history.transcript.clone();
    assert_eq!(
        serde_json::to_vec(&with_history).unwrap(),
        serde_json::to_vec(&expected).unwrap(),
        "Pruning only old history retains the fresh command's intended state, workspace, drafts and metadata"
    );
}
