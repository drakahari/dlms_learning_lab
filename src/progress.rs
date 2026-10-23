use crate::{engine::State, pack::Scenario};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
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
/// Earlier script completion remains distinguishable from current evaluator acceptance.
pub fn legacy_script_credit(a: &Attempt, s: &Scenario) -> bool {
    let crate::engine::Goal::Script { cases } = &s.goal else {
        return false;
    };
    let State::Script { run, .. } = &a.state else {
        return false;
    };
    run.as_ref()
        .into_iter()
        .chain(a.workspace.as_ref().and_then(|w| w.script_history.as_ref()))
        .chain(
            a.workspace
                .as_ref()
                .into_iter()
                .flat_map(|w| &w.script_history_more),
        )
        .any(|r| {
            r.engine_revision < crate::shell::EVALUATOR_REVISION
                && r.results.len() == cases.len()
                && r.results
                    .iter()
                    .zip(cases)
                    .all(|(result, case)| case.accepts(result))
        })
}
pub fn current_solved(a: &Attempt, s: &Scenario) -> bool {
    if let State::Discovery { simulation } = &a.state {
        let State::Discovery {
            simulation: initial,
        } = &s.initial
        else {
            return false;
        };
        if !simulation.topology().same_identity(initial.topology())
            || a.workspace
                .as_ref()
                .and_then(|w| w.discovery_draft.as_ref())
                .is_some_and(|d| !d.matches(simulation))
        {
            return false;
        }
    }
    if let State::Processes { table } = &a.state {
        let State::Processes { table: initial } = &s.initial else {
            return false;
        };
        if !table.same_problem(initial) {
            return false;
        }
    }
    if let State::AuthoritativeDns { server } = &a.state {
        let State::AuthoritativeDns { server: initial } = &s.initial else {
            return false;
        };
        if !crate::dns_workspace::same_problem(server, initial)
            || a.workspace
                .as_ref()
                .and_then(|w| w.dns_draft.as_ref())
                .is_some_and(|d| !d.matches(server))
        {
            return false;
        }
    }
    if crate::workspace::pending_service_draft(a) {
        return false;
    }
    if crate::address_teaching::goal(&s.initial).is_some()
        && !crate::address_teaching::same_assignment(&a.state, &s.initial)
    {
        return false;
    }

    if let State::Switching { fabric } = &a.state
        && a.workspace
            .as_ref()
            .and_then(|w| w.switching_draft.as_ref())
            .is_some_and(|d| !d.matches(fabric))
    {
        return false;
    }
    if let State::Ipv6 { topology } = &a.state
        && a.workspace
            .as_ref()
            .and_then(|w| w.ipv6_draft.as_ref())
            .is_some_and(|d| !d.matches(topology))
    {
        return false;
    }
    if let State::Ipv4Routing { topology } = &a.state
        && a.workspace
            .as_ref()
            .and_then(|w| w.routing_v4_draft.as_ref())
            .is_some_and(|d| !d.matches(topology))
    {
        return false;
    }
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
    decode(&read_bounded(path, 16_000_000)?, scenarios)
}
pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    if !std::fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_file()
    {
        return Err(
            "Choose an ordinary file; symbolic links and special files are not supported".into(),
        );
    }
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Choose an ordinary file".into());
    }
    let mut bytes = Vec::new();
    f.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("File exceeds the supported size; original retained".into());
    }
    Ok(bytes)
}
pub fn decode(bytes: &[u8], scenarios: &[Scenario]) -> Result<Progress, String> {
    if bytes.len() > 16_000_000 {
        return Err("Progress exceeds 16 MB; original retained".into());
    }
    let p: Progress = serde_json::from_slice(bytes)
        .map_err(|e| format!("Progress unreadable; original retained: {e}"))?;
    if p.schema_version != 1 {
        return Err("Unsupported progress version; original retained".into());
    }
    if p.attempts.len() > scenarios.len()
        || p.favorites.len() > scenarios.len()
        || p.favorites
            .iter()
            .any(|id| !scenarios.iter().any(|s| &s.id == id))
    {
        return Err("Unknown or excessive saved labs; original retained".into());
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
        if let Some(learning) = a
            .workspace
            .as_ref()
            .and_then(|w| w.process_learning.as_ref())
            && (!matches!(a.state, State::Processes { .. })
                || s.process_teaching.as_ref().is_none_or(|lesson| {
                    !learning.table.same_problem(&lesson.fixture)
                        || learning.example_command >= lesson.example.len()
                }))
        {
            return Err("Saved process example differs from this lesson; original retained".into());
        }
        if let (State::Processes { table }, State::Processes { table: initial }) =
            (&a.state, &s.initial)
            && !table.same_problem(initial)
        {
            return Err(
                "Saved process identities differ from this assignment; original retained".into(),
            );
        }
        if let (State::Switching { fabric }, State::Switching { fabric: initial }) =
            (&a.state, &s.initial)
            && (!fabric.same_shape(initial)
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.switching_draft.as_ref())
                    .is_some_and(|d| !d.same_shape(fabric)))
        {
            return Err("Saved switch/workstation/cable structure differs from this activity; original retained".into());
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.switching_learning.as_ref())
            && s.switching_teaching
                .as_ref()
                .is_none_or(|lesson| !l.fabric.same_shape(&lesson.fixture))
        {
            return Err(
                "Saved switching example structure differs from this lesson; original retained"
                    .into(),
            );
        }
        if let (State::Ipv6 { topology }, State::Ipv6 { topology: initial }) =
            (&a.state, &s.initial)
            && (!topology.same_shape(initial)
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.ipv6_draft.as_ref())
                    .is_some_and(|d| !d.same_shape(topology)))
        {
            return Err("Saved IPv6 device/interface/link structure differs from this scenario; original retained".into());
        }
        if let State::Ipv4Routing { topology } = &a.state {
            let State::Ipv4Routing { topology: initial } = &s.initial else {
                unreachable!()
            };
            if !topology.same_shape(initial) {
                return Err("Saved IPv4 device/interface/link structure differs from this activity; original retained".into());
            }
        }
        if crate::address_teaching::goal(&s.initial).is_some()
            && !crate::address_teaching::same_assignment(&a.state, &s.initial)
        {
            return Err(
                "Saved addressing assignment inputs differ from this lab; original retained".into(),
            );
        }
        validate_service_workspace(a)?;
        validate_discovery_workspace(a)?;
        if let (
            State::Discovery { simulation },
            State::Discovery {
                simulation: initial,
            },
        ) = (&a.state, &s.initial)
            && !simulation.topology().same_identity(initial.topology())
        {
            return Err("Saved discovery identities or attachments differ from this assignment; original retained".into());
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.discovery_learning.as_ref())
            && s.discovery_teaching.as_ref().is_none_or(|lesson| {
                !l.simulation
                    .topology()
                    .same_identity(lesson.fixture.topology())
            })
        {
            return Err(
                "Saved discovery practice differs from this lesson; original retained".into(),
            );
        }
        validate_process_workspace(a)?;
        validate_dns_workspace(a)?;
        if let (State::AuthoritativeDns { server }, State::AuthoritativeDns { server: initial }) =
            (&a.state, &s.initial)
            && !crate::dns_workspace::same_problem(server, initial)
        {
            return Err(
                "Saved DNS zone identity differs from this assignment; original retained".into(),
            );
        }
        if let Some(l) = a.workspace.as_ref().and_then(|w| w.dns_learning.as_ref())
            && s.dns_teaching.as_ref().is_none_or(|lesson| {
                !crate::dns_workspace::same_problem(&l.server, &lesson.fixture)
            })
        {
            return Err("Saved DNS example differs from this lesson; original retained".into());
        }
        validate_address_workspace(a)?;
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.address_learning.as_ref())
            && s.address_teaching.as_ref().is_none_or(|lesson| {
                !crate::address_teaching::same_problem(&l.state, &lesson.fixture)
            })
        {
            return Err(
                "Saved addressing example differs from this lesson; original retained".into(),
            );
        }
        if a.workspace
            .as_ref()
            .is_some_and(|w| w.service_learning.is_some())
            && s.service_teaching.is_none()
        {
            return Err(
                "Service practice metadata without authored teaching; original retained".into(),
            );
        }
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.service_learning.as_ref())
            && s.service_teaching
                .as_ref()
                .is_none_or(|lesson| l.command_index >= lesson.commands.len())
        {
            return Err("Saved service example selection is incompatible with this lesson; original retained".into());
        }
        validate_routing_workspace(a)?;
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.routing_v4_learning.as_ref())
            && s.ipv4_routing_teaching
                .as_ref()
                .is_none_or(|lesson| !l.topology.same_shape(&lesson.fixture))
        {
            return Err(
                "Saved IPv4 practice structure differs from this lesson; original retained".into(),
            );
        }
        crate::pack::validate_state(&a.state)?;
        if !crate::workspace::can_preserve_script_history(a) {
            return Err("Saved script history is full; original retained. Keep a copy of the progress file; this profile needs a history migration.".into());
        }
        if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
            return Err("Invalid workspace data; original retained".into());
        }
        if let Some(learning) = a.workspace.as_ref().and_then(|w| w.ipv6_learning.as_ref())
            && s.ipv6_teaching
                .as_ref()
                .is_none_or(|lesson| !learning.topology.same_shape(&lesson.fixture))
        {
            return Err(
                "Saved IPv6 practice structure differs from this lesson; original retained".into(),
            );
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
fn validate_discovery_workspace(a: &Attempt) -> Result<(), String> {
    if let Some(w) = &a.workspace {
        match &a.state {
            State::Discovery { simulation } => {
                if w.discovery_draft
                    .as_ref()
                    .is_some_and(|d| !d.same_shape(simulation))
                {
                    return Err("Invalid discovery draft identities; original retained".into());
                }
            }
            _ if w.discovery_draft.is_some() || w.discovery_learning.is_some() => {
                return Err("Discovery metadata on another activity; original retained".into());
            }
            _ => {}
        }
    }
    Ok(())
}
fn validate_process_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.process_learning.is_some())
        && !matches!(a.state, State::Processes { .. })
    {
        return Err("Process example metadata on a different activity; original retained".into());
    }
    Ok(())
}
fn validate_dns_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.dns_learning.is_some() || w.dns_draft.is_some())
        && !matches!(a.state, State::AuthoritativeDns { .. })
    {
        return Err("DNS editor/practice metadata on another activity; original retained".into());
    }
    Ok(())
}
fn validate_service_workspace(a: &Attempt) -> Result<(), String> {
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.service_learning.is_some())
        && !matches!(a.state, State::Service { .. })
    {
        return Err("Service practice metadata on another activity; original retained".into());
    }
    Ok(())
}
fn validate_address_workspace(a: &Attempt) -> Result<(), String> {
    if let Some(l) = a
        .workspace
        .as_ref()
        .and_then(|w| w.address_learning.as_ref())
        && (crate::address_teaching::goal(&a.state).is_none()
            || std::mem::discriminant(&a.state) != std::mem::discriminant(&l.state))
    {
        return Err(
            "Addressing example metadata on an incompatible activity; original retained".into(),
        );
    }
    Ok(())
}
fn validate_routing_workspace(a: &Attempt) -> Result<(), String> {
    let Some(w) = &a.workspace else {
        return Ok(());
    };
    if w.routing_v4_draft.is_none() && w.routing_v4_learning.is_none() {
        return Ok(());
    }
    let State::Ipv4Routing { topology } = &a.state else {
        return Err("IPv4 routing metadata on a different activity; original retained".into());
    };
    if w.routing_v4_draft
        .as_ref()
        .is_some_and(|d| !d.same_shape(topology))
    {
        return Err("Invalid IPv4 routing draft structure; original retained".into());
    }
    Ok(())
}
#[derive(Debug)]
pub struct Saved {
    pub durability_warning: Option<String>,
}
pub fn save(path: &Path, p: &Progress) -> Result<Saved, String> {
    for a in p.attempts.values() {
        crate::pack::validate_state(&a.state)?;
        validate_discovery_workspace(a)?;
        validate_process_workspace(a)?;
        validate_dns_workspace(a)?;
        validate_service_workspace(a)?;
        validate_address_workspace(a)?;
        validate_routing_workspace(a)?;
        if let State::Switching { fabric } = &a.state
            && a.workspace
                .as_ref()
                .and_then(|w| w.switching_draft.as_ref())
                .is_some_and(|d| !d.same_shape(fabric))
        {
            return Err("Invalid switching draft structure; previous progress retained".into());
        }

        if let State::Ipv6 { topology } = &a.state
            && a.workspace
                .as_ref()
                .and_then(|w| w.ipv6_draft.as_ref())
                .is_some_and(|draft| !draft.same_shape(topology))
        {
            return Err("Invalid IPv6 draft structure; previous progress retained".into());
        }
        if !crate::workspace::can_preserve_script_history(a) {
            return Err("Saved script history is full; original retained. Keep a copy of the progress file; this profile needs a history migration.".into());
        }
        if a.workspace.as_ref().is_some_and(|w| !w.valid()) {
            return Err("Invalid draft data; previous progress retained".into());
        }
    }
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let encoded = serde_json::to_vec(p).map_err(|e| e.to_string())?;
    if encoded.len() > 16_000_000 {
        return Err("Progress exceeds 16 MB; original retained".into());
    }
    // Read-time migration never rewrites the original. Preserve exact bytes before an upgraded write.
    let workspace_backup = path.with_extension("before-workspace.json");
    let evaluator_backup = path.with_extension(format!(
        "before-shell-v{}.json",
        crate::shell::EVALUATOR_REVISION
    ));
    // Inspect the on-disk version even when Reset removed the last legacy run from memory.
    if path.exists() {
        let original = read_bounded(path, 16_000_000)?;
        let old: Progress = serde_json::from_slice(&original)
            .map_err(|e| format!("Original retained; cannot back up: {e}"))?;
        if p.attempts.values().any(|a| a.workspace.is_some())
            && old.attempts.values().all(|a| a.workspace.is_none())
        {
            preserve_copy(&workspace_backup, &original)?;
        }
        if old.attempts.values().any(|a| {
            matches!(&a.state,State::Script{run:Some(r),..}if !r.current())
                || a.workspace
                    .as_ref()
                    .and_then(|w| w.script_learning.as_ref())
                    .is_some_and(|l| {
                        l.example_result.is_some()
                            && l.example_revision < crate::shell::EVALUATOR_REVISION
                    })
        }) {
            preserve_copy(&evaluator_backup, &original)?;
        }
    }
    write_atomic(path, &encoded)
}
fn validate_saved_copy(path: &Path) -> Result<(), String> {
    let bytes = read_bounded(path, 16_000_000)?;
    let previous:Progress=serde_json::from_slice(&bytes).map_err(|_|"Original retained; existing migration-backup filename is occupied by incompatible data")?;
    if previous.schema_version != 1 {
        return Err("Original retained; existing migration backup has an unknown version".into());
    }
    Ok(())
}
fn preserve_copy(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if metadata.file_type().is_file() {
            return validate_saved_copy(path);
        }
        return Err("Original retained; migration backup path is not an ordinary file".into());
    }
    let mut copy =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing backup directory")?)
            .map_err(|e| e.to_string())?;
    copy.write_all(bytes)
        .and_then(|_| copy.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    match copy.persist_noclobber(path) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            validate_saved_copy(path)
        }
        Err(error) => Err(format!(
            "Original retained; migration backup failed: {}",
            error.error
        )),
    }
}
pub(crate) fn write_atomic(path: &Path, encoded: &[u8]) -> Result<Saved, String> {
    write_atomic_with_sync(path, encoded, |parent| {
        #[cfg(unix)]
        {
            std::fs::File::open(parent)?.sync_all()
        }
        #[cfg(not(unix))]
        {
            let _ = parent;
            Ok(())
        }
    })
}
fn write_atomic_with_sync(
    path: &Path,
    encoded: &[u8],
    sync: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<Saved, String> {
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(encoded).map_err(|e| e.to_string())?;
    file.flush()
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    let durability_warning=sync(parent).err().map(|e|format!("The file was replaced, but storage durability could not be confirmed: {e}. Keep an additional backup before closing."));
    Ok(Saved { durability_warning })
}

/// Held for the app lifetime. OS locks are released on exit/crash; the lock file is never deleted.
/// Deleting it would let a second process lock another inode while the first still owns its lock.
pub struct ProfileLock {
    _file: std::fs::File,
}
pub fn lock_profile(path: &Path) -> Result<ProfileLock, String> {
    let parent = path.parent().ok_or("Missing storage directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let file = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|e|format!("Progress is read-only because another app instance owns it or this storage cannot lock safely: {e}. Close other copies and reopen this app."))?;
    Ok(ProfileLock { _file: file })
}

#[cfg(test)]
mod durability_tests {
    #[test]
    fn committed_write_reports_sync_warning_instead_of_claiming_original_was_retained() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");
        std::fs::write(&path, b"before").unwrap();
        let saved = super::write_atomic_with_sync(&path, b"after", |_| {
            Err(std::io::Error::other("controlled sync failure"))
        })
        .unwrap();
        assert!(
            saved
                .durability_warning
                .unwrap()
                .contains("file was replaced")
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"after");
    }
}
