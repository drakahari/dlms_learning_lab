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
    let progress = if value.get("format").is_some() {
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
    progress::decode(
        &serde_json::to_vec(&progress).map_err(|e| e.to_string())?,
        scenarios,
    )
}
fn run(source: &str, case: &shell::Case, cancel: &AtomicBool) -> Result<shell::Outcome, String> {
    match shell::Vm::new(source, case) {
        Ok(mut vm) => {
            while !vm.done() {
                if cancel.load(Ordering::Relaxed) {
                    return Err("Backup check cancelled; no changes made".into());
                }
                vm.tick(32);
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
                results.push(run(source, case, cancel)?);
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
            l.example_result = Some(run(&l.example_source, &lesson.fixture, cancel)?);
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
        let cancel = Arc::new(AtomicBool::new(false));
        let checked = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = mpsc::channel();
        let c = cancel.clone();
        let n = checked.clone();
        thread::spawn(move || {
            let result = inspect(&path, &scenarios, &c, &n);
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
    let bytes = serde_json::to_vec(&candidate.progress).map_err(|e| e.to_string())?;
    let _ = progress::decode(&bytes, scenarios)?;
    let backup = if path.exists() {
        let original = progress::read_bounded(path, 16_000_000)?;
        let parent = path.parent().ok_or("Missing progress folder")?;
        let mut chosen = None;
        for n in 1..=10_000 {
            let b = parent.join(format!("progress-before-restore-{n}.json"));
            if !b.exists() {
                if let Some(warning) = write_new(&b, &original)? {
                    return Err(format!("{warning}. Current profile was not replaced."));
                }
                chosen = Some(b);
                break;
            }
        }
        Some(chosen.ok_or("Too many recovery backups; move older backups first")?)
    } else {
        None
    };
    let saved = progress::write_atomic(path, &bytes)?;
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
