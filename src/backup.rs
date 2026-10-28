//! Explicit user-requested host data management. No simulator can call these functions.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::{self, Progress},
    shell,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};
const LIMIT: usize = 16_010_000;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    app_version: String,
    progress: Progress,
}
pub struct Candidate {
    pub progress: Progress,
    pub rechecked: usize,
}
impl Candidate {
    pub fn summary(&self, scenarios: &[Scenario]) -> String {
        let complete = self
            .progress
            .attempts
            .iter()
            .filter(|(id, a)| {
                scenarios
                    .iter()
                    .find(|s| &s.id == *id)
                    .is_some_and(|s| progress::current_solved(a, s))
            })
            .count();
        let legacy = self
            .progress
            .attempts
            .iter()
            .filter(|(id, a)| {
                progress::legacy_credit(a)
                    || scenarios
                        .iter()
                        .find(|s| &s.id == *id)
                        .is_some_and(|s| progress::legacy_script_credit(a, s))
            })
            .count();
        format!(
            "{} saved labs · {complete} currently complete · {legacy} with earlier completion credit · {} favorites · {} script/example results rechecked",
            self.progress.attempts.len(),
            self.progress.favorites.len(),
            self.rechecked
        )
    }
}
/// New files only: backups can never replace one another or the main profile by accident.
fn write_new(path: &Path, bytes: &[u8]) -> Result<Option<String>, String> {
    let parent = path.parent().ok_or("Choose a backup folder")?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist_noclobber(path)
        .map_err(|e| format!("Backup not written; choose an unused filename: {}", e.error))?;
    #[cfg(unix)]
    let warning = std::fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .err()
        .map(|e| format!("Backup created, but storage durability was not confirmed: {e}"));
    #[cfg(not(unix))]
    let warning = None;
    Ok(warning)
}
pub fn export(path: &Path, p: &Progress, scenarios: &[Scenario]) -> Result<Option<String>, String> {
    // Validate provenance before portable serialization deliberately removes it.
    progress::validate_provenance(p)?;
    // Validate through the same versioned reader before writing anything.
    let encoded = serde_json::to_vec(p).map_err(|e| e.to_string())?;
    let _: Progress = progress::decode(&encoded, scenarios)?;
    let owned: Progress = serde_json::from_slice(&encoded).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&Envelope {
        format: "dlms-learning-lab-backup".into(),
        version: 1,
        app_version: env!("CARGO_PKG_VERSION").into(),
        progress: owned,
    })
    .map_err(|e| e.to_string())?;
    if bytes.len() > LIMIT {
        return Err("Backup exceeds size limit".into());
    }
    write_new(path, &bytes)
}
fn parse(path: &Path, scenarios: &[Scenario]) -> Result<Progress, String> {
    let bytes = progress::read_bounded(path, LIMIT)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let mut progress = if value.get("format").is_some() {
        let envelope: Envelope = serde_json::from_value(value).map_err(|e| e.to_string())?;
        if envelope.format != "dlms-learning-lab-backup"
            || envelope.version != 1
            || envelope.app_version.len() > 80
        {
            return Err("Unsupported backup format/version; no changes made".into());
        }
        envelope.progress
    } else {
        serde_json::from_value(value).map_err(|e| e.to_string())?
    };
    progress::validate_provenance(&progress)?;
    progress.migration_receipts = None;
    progress::decode(
        &serde_json::to_vec(&progress).map_err(|e| e.to_string())?,
        scenarios,
    )
}
fn run(
    source: &str,
    case: &shell::Case,
    cancel: &AtomicBool,
    observer: &mut impl FnMut(&shell::Vm),
) -> Result<shell::Outcome, String> {
    match shell::Vm::new(source, case) {
        Ok(mut vm) => {
            while !vm.done() {
                if cancel.load(Ordering::Relaxed) {
                    return Err("Backup check cancelled; no changes made".into());
                }
                vm.tick(32);
                observer(&vm);
            }
            Ok(vm.outcome)
        }
        Err(_) => Ok(shell::run(source, case)),
    }
}
pub fn inspect(
    path: &Path,
    scenarios: &[Scenario],
    cancel: &AtomicBool,
    checked: &AtomicUsize,
) -> Result<Candidate, String> {
    inspect_observing(path, scenarios, cancel, checked, &mut |_| {})
}
fn inspect_observing(
    path: &Path,
    scenarios: &[Scenario],
    cancel: &AtomicBool,
    checked: &AtomicUsize,
    observer: &mut impl FnMut(&shell::Vm),
) -> Result<Candidate, String> {
    let mut p = parse(path, scenarios)?;
    let mut count = 0;
    for (id, a) in &mut p.attempts {
        if cancel.load(Ordering::Relaxed) {
            return Err("Backup check cancelled; no changes made".into());
        }
        let s = scenarios
            .iter()
            .find(|s| &s.id == id)
            .ok_or("Unknown backup lab")?;
        let had_saved_run = matches!(&a.state, State::Script { run: Some(_), .. });
        crate::workspace::preserve_script_history(a);
        if let (
            State::Script {
                source,
                run: record,
            },
            Goal::Script { cases },
        ) = (&mut a.state, &s.goal)
            && had_saved_run
        {
            let mut results = Vec::new();
            for case in cases {
                results.push(run(source, case, cancel, observer)?);
            }
            *record = Some(shell::Record {
                engine_revision: crate::shell::EVALUATOR_REVISION,
                source: source.clone(),
                results,
            });
            count += 1;
        }
        if let Some(l) = a
            .workspace
            .as_mut()
            .and_then(|w| w.script_learning.as_mut())
            && l.example_result.is_some()
        {
            let lesson = s
                .teaching
                .as_ref()
                .ok_or("Example data does not belong to this lab")?;
            l.example_result = Some(run(&l.example_source, &lesson.fixture, cancel, observer)?);
            l.example_revision = shell::EVALUATOR_REVISION;
            count += 1;
        }
        checked.store(count, Ordering::Relaxed);
    }
    Ok(Candidate {
        progress: p,
        rechecked: count,
    })
}
pub struct Inspection {
    cancel: Arc<AtomicBool>,
    pub checked: Arc<AtomicUsize>,
    rx: mpsc::Receiver<Result<Candidate, String>>,
}
impl Inspection {
    pub fn start(path: PathBuf, scenarios: Vec<Scenario>) -> Self {
        Self::start_observing(path, scenarios, |_| {})
    }
    fn start_observing(
        path: PathBuf,
        scenarios: Vec<Scenario>,
        mut observer: impl FnMut(&shell::Vm) + Send + 'static,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let checked = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = mpsc::channel();
        let c = cancel.clone();
        let n = checked.clone();
        thread::spawn(move || {
            let result = inspect_observing(&path, &scenarios, &c, &n, &mut observer);
            let _ = tx.send(result);
        });
        Self {
            cancel,
            checked,
            rx,
        }
    }
    pub fn poll(&self) -> Option<Result<Candidate, String>> {
        match self.rx.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(
                "Backup checking stopped unexpectedly; no changes made".into(),
            )),
        }
    }
}
impl Drop for Inspection {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
/// Caller holds the profile lock. Preserve exact current bytes (including malformed data) first.
/// Restore the already inspected candidate, never re-read the import path after confirmation.
pub struct Restored {
    pub safety_copy: Option<PathBuf>,
    pub durability_warning: Option<String>,
}
pub fn restore(
    path: &Path,
    candidate: &Candidate,
    scenarios: &[Scenario],
) -> Result<Restored, String> {
    progress::validate_provenance(&candidate.progress)?;
    let bytes = serde_json::to_vec(&candidate.progress).map_err(|e| e.to_string())?;
    let _ = progress::decode(&bytes, scenarios)?;
    let mut backup = None;
    let saved = progress::save_for_restore(path, &candidate.progress, scenarios, |original| {
        if let Some(original) = original {
            let parent = path.parent().ok_or("Missing progress folder")?;
            let mut chosen = None;
            for n in 1..=10_000 {
                let b = parent.join(format!("progress-before-restore-{n}.json"));
                if !b.exists() {
                    if let Some(warning) = write_new(&b, original)? {
                        return Err(format!("{warning}. Current profile was not replaced."));
                    }
                    chosen = Some(b);
                    break;
                }
            }
            backup = Some(chosen.ok_or("Too many recovery backups; move older backups first")?);
        }
        Ok(())
    })?;
    Ok(Restored {
        safety_copy: backup,
        durability_warning: saved.durability_warning,
    })
}

/// Explicit user exports cannot occupy application files, even before those files exist.
pub fn check_destination(target: &Path, profile: &Path) -> Result<(), String> {
    fn normalized(path: &Path) -> Result<PathBuf, String> {
        Ok(path
            .parent()
            .ok_or("Missing folder")?
            .canonicalize()
            .map_err(|e| e.to_string())?
            .join(path.file_name().ok_or("Missing filename")?))
    }
    let target = normalized(target)?;
    let profile = normalized(profile)?;
    let prefix = profile
        .with_extension("before-shell-v")
        .to_string_lossy()
        .into_owned();
    let candidate = target.to_string_lossy().into_owned();
    let (prefix, candidate) = if cfg!(windows) {
        (prefix.to_ascii_lowercase(), candidate.to_ascii_lowercase())
    } else {
        (prefix, candidate)
    };
    if candidate
        .strip_prefix(&prefix)
        .and_then(|tail| tail.strip_suffix(".json"))
        .is_some_and(|version| !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(
            "Choose a backup filename outside the application's evaluator-migration filenames"
                .into(),
        );
    }
    for reserved in [
        profile.to_path_buf(),
        profile.with_extension("lock"),
        profile.with_file_name("preferences-v1.json"),
        profile.with_extension("before-workspace.json"),
        profile.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION)),
    ] {
        let reserved = normalized(&reserved)?;
        #[cfg(windows)]
        let equal = target
            .to_string_lossy()
            .eq_ignore_ascii_case(&reserved.to_string_lossy());
        #[cfg(not(windows))]
        let equal = target == reserved;
        if equal {
            return Err("Choose a backup filename outside the application's progress, preferences and lock filenames".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[test]
    fn worker_cancellation_after_child_capture_starts_never_returns_a_graded_candidate() {
        let labs = crate::pack::bundled().unwrap();
        let evidence: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/shell-v6-authored-records.json"
        ))
        .unwrap();
        let old = &evidence[0];
        let lab = labs
            .iter()
            .find(|s| s.id == old["id"].as_str().unwrap())
            .unwrap();
        let record: shell::Record = serde_json::from_value(old["record"].clone()).unwrap();
        assert_eq!(record.engine_revision, 6);
        let mut a = crate::progress::Attempt::new(lab, 1);
        a.state = State::Script {
            source: "value=$(while true; do :; done); printf parent-must-not-run".into(),
            run: Some(record),
        };
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(lab.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("import.json");
        let original = serde_json::to_vec_pretty(&p).unwrap();
        std::fs::write(&input, &original).unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let child_barrier = barrier.clone();
        let mut stopped = false;
        let task = Inspection::start_observing(input.clone(), labs, move |vm| {
            if !stopped && vm.active_capture_depth() > 0 {
                stopped = true;
                entered_tx.send(()).unwrap();
                child_barrier.wait();
            }
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        // This is exactly the flag set by dropping the application's preview worker.
        task.cancel.store(true, Ordering::Relaxed);
        barrier.wait();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(result) = task.poll() {
                assert!(result.err().unwrap().contains("cancelled"));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(task.checked.load(Ordering::Relaxed), 0);
        assert_eq!(std::fs::read(input).unwrap(), original);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}

#[cfg(test)]
mod current_revision_capture_cancellation_acceptance {
    use super::*;
    #[test]
    fn actual7_profile_preview_cancelled_inside_child_retains_every_input_target_and_history_byte()
    {
        const {
            assert!(shell::EVALUATOR_REVISION >= 8);
        }
        let labs = crate::pack::bundled().unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../tests/fixtures/shell-v7-synthetic-writer-progress.json"
        ))
        .unwrap();
        value["attempts"]["shell-arguments"]["state"]["source"] =
            serde_json::json!("pending=$(while true; do :; done); printf parent-must-not-run");
        let bytes = serde_json::to_vec_pretty(&value).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.json");
        let target = dir.path().join("progress.json");
        std::fs::write(&input, &bytes).unwrap();
        let original: &[u8] =
            include_bytes!("../tests/fixtures/shell-v7-synthetic-writer-progress.json");
        std::fs::write(&target, original).unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let child_barrier = barrier.clone();
        let mut stopped = false;
        let task = Inspection::start_observing(input.clone(), labs, move |vm| {
            if !stopped && vm.active_capture_depth() > 0 {
                stopped = true;
                entered_tx.send(()).unwrap();
                child_barrier.wait();
            }
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        task.cancel.store(true, Ordering::Relaxed);
        barrier.wait();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(result) = task.poll() {
                assert!(result.err().unwrap().contains("cancelled"));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(std::fs::read(input).unwrap(), bytes);
        assert_eq!(std::fs::read(target).unwrap(), original);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        assert_eq!(task.checked.load(Ordering::Relaxed), 0);
    }
}

#[cfg(test)]
mod genuine8_getopts_child_cancellation {
    use super::*;
    #[test]
    fn confirmed_preview_cancel_with_active_getopts_child_preserves_raw8_and_all_histories() {
        const {
            assert!(shell::EVALUATOR_REVISION >= 9);
        }
        const RAW: &[u8] =
            include_bytes!("../tests/fixtures/shell-v8-synthetic-writer-progress.json");
        let mut value: serde_json::Value = serde_json::from_slice(RAW).unwrap();
        value["attempts"]["shell-arguments"]["state"]["source"] = serde_json::json!(
            "pending=$(getopts ab choice -ab; while true; do :; done); printf parent-must-not-run"
        );
        let bytes = serde_json::to_vec_pretty(&value).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.json");
        let target = directory.path().join("progress.json");
        std::fs::write(&input, &bytes).unwrap();
        std::fs::write(&target, RAW).unwrap();
        let (tx, rx) = mpsc::channel();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let child_barrier = barrier.clone();
        let mut observed = false;
        let task = Inspection::start_observing(
            input.clone(),
            crate::pack::bundled().unwrap(),
            move |vm| {
                if !observed && vm.active_capture_depth() > 0 && vm.active_getopts_scan() {
                    observed = true;
                    tx.send(()).unwrap();
                    child_barrier.wait();
                }
            },
        );
        rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        task.cancel.store(true, Ordering::Relaxed);
        barrier.wait();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(result) = task.poll() {
                assert!(result.err().unwrap().contains("cancelled"));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(task.checked.load(Ordering::Relaxed), 0);
        assert_eq!(std::fs::read(input).unwrap(), bytes);
        assert_eq!(std::fs::read(target).unwrap(), RAW);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }
}

#[cfg(test)]
mod actual9_safe_automation_child_cancellation {
    use super::*;
    use crate::actual9_test_evidence as evidence;
    #[test]
    #[ignore = "Requires fixed private actual9 COMPLETE; explicit guarded replay only"]
    fn actual9_preview_cancel_with_active_child_preserves_exact_input_target_and_all_history() {
        assert_eq!(shell::EVALUATOR_REVISION, 11);
        let raw = evidence::original9();
        let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        value["attempts"]["shell-arguments"]["state"]["source"] = serde_json::json!(
            "pending=$(set -o pipefail; false | true; while true; do :; done); printf must-not-run"
        );
        let bytes = serde_json::to_vec(&value).unwrap();
        evidence::exact(
            &value,
            &serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        );
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.json");
        let target = dir.path().join("progress.json");
        std::fs::write(&input, &bytes).unwrap();
        std::fs::write(&target, &raw).unwrap();
        let (tx, rx) = mpsc::channel();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let child_barrier = barrier.clone();
        let mut observed = false;
        let task = Inspection::start_observing(
            input.clone(),
            crate::pack::bundled().unwrap(),
            move |vm| {
                if !observed && vm.active_capture_depth() > 0 {
                    observed = true;
                    tx.send(()).unwrap();
                    child_barrier.wait();
                }
            },
        );
        rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        task.cancel.store(true, Ordering::Relaxed);
        barrier.wait();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(result) = task.poll() {
                assert!(result.err().unwrap().contains("cancelled"));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(task.checked.load(Ordering::Relaxed), 0);
        assert_eq!(std::fs::read(input).unwrap(), bytes);
        assert_eq!(std::fs::read(target).unwrap(), raw);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}
