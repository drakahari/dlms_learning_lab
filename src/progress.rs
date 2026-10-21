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
}
impl Attempt {
    pub fn new(s: &Scenario, visited: u64) -> Self {
        Self {
            scenario_version: s.version,
            state: s.initial.clone(),
            step: 0,
            challenge: false,
            hints: 0,
            transcript: vec!["Simulation ready. Type help to see the command subset.".into()],
            visited,
            ipv4: None,
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
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 5_000_000 {
        return Err("Progress exceeds 5 MB; original retained".into());
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
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, p).map_err(|e| e.to_string())?;
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
