use crate::{engine::State, pack::Scenario};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub scenario_version: u32,
    pub state: State,
    pub step: usize,
    pub challenge: bool,
    pub hints: usize,
    pub transcript: Vec<String>,
    pub visited: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<crate::ipv4::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<crate::workspace::WorkspaceData>,
}
impl Attempt {
    pub fn new(s: &Scenario, visited: u64) -> Self {
        Self {
            scenario_version: s.version,
            state: s.initial.clone(),
            step: 0,
            challenge: false,
            hints: 0,
            transcript: Vec::new(),
            visited,
            ipv4: None,
            workspace: None,
        }
    }
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub schema_version: u32,
    pub clock: u64,
    pub favorites: BTreeSet<String>,
    pub attempts: BTreeMap<String, Attempt>,
}
/// Original analysis credit is visible, but never stands in for a new requirement.
pub fn legacy_credit(a: &Attempt) -> bool {
    a.workspace.as_ref().is_some_and(|w| w.migrated_host)
        || matches!(&a.state, State::Subnet { second: None, .. })
            && crate::ipv4::Exercise::from_state(&a.state).is_some_and(|e| e.solved())
}
pub fn current_solved(a: &Attempt, s: &Scenario) -> bool {
    !matches!(&a.state, State::Subnet { second: None, .. })
        && a.workspace
            .as_ref()
            .is_none_or(|w| !w.migrated_host || w.legacy_host_reviewed)
        && crate::engine::solved(&a.state, &s.goal)
}
pub fn default_path() -> Result<PathBuf, String> {
    directories::ProjectDirs::from("org", "DLMS", "DLMS Learning Lab")
        .map(|d| d.data_local_dir().join("progress-v1.json"))
        .ok_or("No local application data directory".into())
}
pub fn load(path: &Path, scenarios: &[Scenario]) -> Result<Progress, String> {
    if !path.exists() {
        return Ok(Progress {
            schema_version: 1,
            ..Default::default()
        });
    }
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 16_000_000 {
        return Err("Progress exceeds 16 MB; original retained".into());
    }
    let p: Progress = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Progress unreadable; original retained: {e}"))?;
    if p.schema_version != 1 {
        return Err("Unsupported progress version; original retained".into());
    }
    for (id, a) in &p.attempts {
        let s = scenarios
            .iter()
            .find(|s| &s.id == id)
            .ok_or("Unknown saved scenario; original retained")?;
        if a.scenario_version != s.version
            || std::mem::discriminant(&a.state) != std::mem::discriminant(&s.initial)
            || a.step >= s.steps.len()
            || a.hints > s.hints.len()
            || a.transcript.len() > 120
        {
            return Err("Incompatible saved attempt; original retained".into());
        }
        crate::pack::validate_state(&a.state)?;
        if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
            return Err("Invalid workspace data; original retained".into());
        }
        if let Some(l) = &a.ipv4 {
            let original = crate::ipv4::Exercise::from_state(&a.state)
                .ok_or("IPv4 lesson metadata on a different state")?;
            if !crate::ipv4::validate_lesson(l, &original) {
                return Err("Invalid saved IPv4 lesson; original retained".into());
            }
        }
    }
    Ok(p)
}
pub fn save(path: &Path, p: &Progress) -> Result<(), String> {
    for a in p.attempts.values() {
        crate::pack::validate_state(&a.state)?;
        if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
            return Err("Invalid draft data; previous progress retained".into());
        }
    }
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    // Preserve a byte-for-byte pre-workspace save before the first upgraded write.
    // Reading an old save never migrates or rewrites it.
    if p.attempts.values().any(|a| a.workspace.is_some()) && path.exists() {
        let backup = path.with_extension("before-workspace.json");
        if !backup.exists() {
            let original = std::fs::read(path).map_err(|e| e.to_string())?;
            let old: Progress = serde_json::from_slice(&original)
                .map_err(|e| format!("Original retained; cannot back up: {e}"))?;
            if old.attempts.values().all(|a| a.workspace.is_none()) {
                let mut copy =
                    tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
                copy.write_all(&original)
                    .and_then(|_| copy.as_file().sync_all())
                    .map_err(|e| e.to_string())?;
                if let Err(e) = copy.persist_noclobber(&backup)
                    && e.error.kind() != std::io::ErrorKind::AlreadyExists
                {
                    return Err(format!("Original retained; backup failed: {e}"));
                }
            }
        }
    }
    let encoded = serde_json::to_vec(p).map_err(|e| e.to_string())?;
    if encoded.len() > 16_000_000 {
        return Err("Progress exceeds 16 MB; original retained".into());
    }
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(&encoded).map_err(|e| e.to_string())?;
    file.flush()
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
