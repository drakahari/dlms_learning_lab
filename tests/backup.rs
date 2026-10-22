use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching, shell, workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn profile(labs: &[dlms_learning_lab::pack::Scenario]) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let s = &labs[9];
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
    if let (State::Script { source, run }, Goal::Script { cases }) = (&mut a.state, &s.goal) {
        *source = s.solution[0].clone();
        *run = Some(shell::Record {
            engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(source, c)).collect(),
        });
    }
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    p
}
fn inspect(
    path: &std::path::Path,
    labs: &[dlms_learning_lab::pack::Scenario],
) -> Result<backup::Candidate, String> {
    backup::inspect(path, labs, &AtomicBool::new(false), &AtomicUsize::new(0))
}
#[test]
fn backup_preserves_drafts_favorites_and_earned_results_then_restores_exact_safety_copy() {
    let labs = pack::bundled().unwrap();
    let p = profile(&labs);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    backup::export(&path, &p, &labs).unwrap();
    assert!(backup::export(&path, &p, &labs).is_err());
    let candidate = inspect(&path, &labs).unwrap();
    assert_eq!(candidate.rechecked, 1);
    assert_eq!(
        serde_json::to_value(&candidate.progress).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let target = dir.path().join("progress.json");
    let previous = b"{ damaged previous profile }";
    std::fs::write(&target, previous).unwrap();
    let safety = backup::restore(&target, &candidate, &labs)
        .unwrap()
        .safety_copy
        .unwrap();
    assert_eq!(std::fs::read(safety).unwrap(), previous);
    assert_eq!(
        serde_json::to_value(progress::load(&target, &labs).unwrap()).unwrap(),
        serde_json::to_value(p).unwrap()
    );
}
#[test]
fn invalid_oversized_or_future_backups_leave_current_progress_untouched() {
    let labs = pack::bundled().unwrap();
    let p = profile(&labs);
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("progress.json");
    progress::save(&target, &p).unwrap();
    let original = std::fs::read(&target).unwrap();
    let input = dir.path().join("input.json");
    for bytes in [
        b"not json".to_vec(),
        b"{\"schema_version\":99,\"clock\":0,\"favorites\":[],\"attempts\":{}}".to_vec(),
        vec![b' '; 16_010_001],
    ] {
        std::fs::write(&input, bytes).unwrap();
        assert!(inspect(&input, &labs).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), original);
    }
    let mut data = serde_json::to_value(&p).unwrap();
    data["favorites"] = serde_json::json!(["unknown-lab"]);
    std::fs::write(&input, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(inspect(&input, &labs).is_err());
}
#[test]
fn forged_script_results_are_rechecked_and_changed_source_file_cannot_replace_reviewed_candidate() {
    let labs = pack::bundled().unwrap();
    let mut p = profile(&labs);
    let a = p.attempts.get_mut("shell-handover").unwrap();
    if let State::Script {
        source,
        run: Some(r),
    } = &mut a.state
    {
        *source = "printf 'wrong\\n'".into();
        r.source = source.clone();
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("backup.json");
    backup::export(&input, &p, &labs).unwrap();
    let candidate = inspect(&input, &labs).unwrap();
    assert!(!engine::solved(
        &candidate.progress.attempts["shell-handover"].state,
        &labs[9].goal
    ));
    std::fs::write(&input, b"changed after preview").unwrap();
    let target = dir.path().join("progress.json");
    backup::restore(&target, &candidate, &labs).unwrap();
    let restored = progress::load(&target, &labs).unwrap();
    assert!(!engine::solved(
        &restored.attempts["shell-handover"].state,
        &labs[9].goal
    ));
}
#[test]
fn reader_accepts_legacy_progress_export_and_cancel_makes_no_changes() {
    let labs = pack::bundled().unwrap();
    let p = profile(&labs);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    assert!(inspect(&path, &labs).is_ok());
    let original = std::fs::read(&path).unwrap();
    assert!(backup::inspect(&path, &labs, &AtomicBool::new(true), &AtomicUsize::new(0)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
#[test]
fn two_instances_cannot_own_the_same_profile_or_silently_replace_newer_drafts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let first = progress::lock_profile(&path).unwrap();
    assert!(progress::lock_profile(&path).is_err());
    let labs = pack::bundled().unwrap();
    let p = profile(&labs);
    progress::save(&path, &p).unwrap();
    drop(first);
    let _second = progress::lock_profile(&path).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &labs).unwrap()).unwrap(),
        serde_json::to_value(p).unwrap()
    );
    assert!(path.with_extension("lock").exists());
}
#[cfg(unix)]
#[test]
fn backup_reader_rejects_symbolic_links_and_directories() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("backup.json");
    backup::export(&target, &profile(&labs), &labs).unwrap();
    let link = dir.path().join("link.json");
    std::os::unix::fs::symlink(target, &link).unwrap();
    assert!(inspect(&link, &labs).is_err());
    assert!(inspect(dir.path(), &labs).is_err());
}

#[test]
fn export_cannot_occupy_missing_application_files_or_parent_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("progress-v1.json");
    for reserved in [
        &profile,
        &profile.with_extension("lock"),
        &profile.with_file_name("preferences-v1.json"),
        &profile.with_extension("before-workspace.json"),
        &profile.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION)),
        &profile.with_extension("before-shell-v999.json"),
    ] {
        assert!(!reserved.exists());
        assert!(backup::check_destination(reserved, &profile).is_err());
        let alias = dir.path().join(".").join(reserved.file_name().unwrap());
        assert!(backup::check_destination(&alias, &profile).is_err());
        assert!(!reserved.exists());
    }
    assert!(backup::check_destination(&dir.path().join("my-backup.json"), &profile).is_ok());
}
