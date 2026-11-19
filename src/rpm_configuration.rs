//! Opt-in virtual regular-configuration updates. The legacy RPM API stays closed.
//! No filesystem/process/network API. Source-owned new tasks choose this transition.
use crate::rpm_packages::{self as rpm, Action, Archive, File, FileKind, State};
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, rpm::Error>;
fn refuse(message: &str) -> rpm::Error {
    rpm::Error {
        code: rpm::Code::UnsupportedSubset,
        message: message.into(),
    }
}
fn attributes(a: &File, b: &File) -> bool {
    a.mode == b.mode && a.owner == b.owner && a.group == b.group
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    InstallIncoming,
    KeepLocal,
    StageIncoming,
    SaveLocal,
    Remove,
    AlreadyMissing,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect {
    pub path: String,
    pub disposition: Disposition,
    pub sidecar: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pub state: State,
    pub plan: rpm::Plan,
    pub configurations: Vec<Effect>,
}
struct Entry<'a> {
    path: &'a str,
    kind: FileKind,
    recorded: &'a File,
    current: Option<&'a File>,
    incoming: Option<&'a File>,
}
/// Compute a new state atomically; this never changes the supplied state.
/// Configuration manifests/flags and permission/ownership attributes must remain
/// the same. Timestamps may change; decisions compare contents, not timestamps.
pub fn transition(state: &State, action: &Action) -> Result<Transition> {
    state.validate()?;
    // The existing action decoder/planner supplies all count/identity/version,
    // local-availability, final ownership and dependency constraints. Validate the
    // action's ordinary shape before allocating any normalized candidate.
    let bounded = match action {
        Action::InstallSet { archives } | Action::UpgradeSet { archives } => {
            !archives.is_empty() && archives.len() <= 8 && archives.iter().all(|s| s.len() <= 32)
        }
        Action::ReinstallExact { archive } => archive.len() <= 32,
        Action::EraseSet { packages } => {
            !packages.is_empty()
                && packages.len() <= 8
                && packages.iter().all(|p| p.name.len() <= 32)
        }
    };
    if !bounded {
        return Err(refuse(
            "Keep the transaction within eight bounded package identities or archive names.",
        ));
    }
    let encoded = serde_json::to_vec(action).map_err(|_| refuse("Invalid RPM action."))?;
    Action::decode(&encoded)?;
    let catalog = state.catalog();
    let runtime = state.runtime();
    let mut changes: Vec<(&Archive, Option<&Archive>)> = vec![];
    match action {
        Action::InstallSet { archives } | Action::UpgradeSet { archives } => {
            for id in archives {
                let new = state.local_archive(id)?;
                if let Some(old) = runtime
                    .installed
                    .iter()
                    .find(|p| p.identity.name == new.identity.name)
                {
                    changes.push((&catalog.archives[&old.archive], Some(new)));
                }
            }
        }
        Action::ReinstallExact { archive } => {
            let new = state.local_archive(archive)?;
            if let Some(old) = runtime
                .installed
                .iter()
                .find(|p| p.archive == *archive && p.identity == new.identity)
            {
                changes.push((&catalog.archives[&old.archive], Some(new)));
            }
        }
        Action::EraseSet { packages } => {
            for identity in packages {
                if let Some(old) = runtime.installed.iter().find(|p| &p.identity == identity) {
                    changes.push((&catalog.archives[&old.archive], None));
                }
            }
        }
    }
    let mut entries = vec![];
    for (old, new) in changes {
        let manifest = |a: &Archive| {
            a.files
                .iter()
                .filter(|(_, p)| p.kind != FileKind::Ordinary)
                .map(|(p, f)| (p.clone(), f.kind))
                .collect::<BTreeMap<_, _>>()
        };
        if new.is_some_and(|a| manifest(old) != manifest(a)) {
            return Err(refuse(
                "Configuration path/flag changes are outside this update subset; no files were changed.",
            ));
        }
        for (path, payload) in &old.files {
            if payload.kind == FileKind::Ordinary {
                continue;
            }
            if entries.iter().any(|e: &Entry<'_>| e.path == path) {
                return Err(refuse(
                    "Choose one archive per package name in this transaction.",
                ));
            }
            let current = runtime.files.get(path);
            let incoming = new.and_then(|a| a.files.get(path)).map(|p| &p.file);
            if current.is_some_and(|c| !attributes(c, &payload.file))
                || incoming.is_some_and(|n| !attributes(n, &payload.file))
            {
                return Err(refuse(
                    "Configuration content updates do not yet handle changed mode, owner or group. Restore the recorded attributes before this operation; modification times may differ. No files were changed.",
                ));
            }
            entries.push(Entry {
                path,
                kind: payload.kind,
                recorded: &payload.file,
                current,
                incoming,
            });
        }
    }
    // The normalization is private intermediate data, never published or saved.
    // It admits exactly the declared configuration-content cases to the existing
    // planner, leaving every non-configuration value and eligibility rule intact.
    let mut normalized = runtime.clone();
    for e in &entries {
        normalized.files.insert(e.path.into(), e.recorded.clone());
    }
    let mut candidate = State::new(catalog.clone(), normalized)?;
    let plan = candidate.apply(action)?;
    let mut result = candidate.runtime().clone();
    let mut effects = vec![];
    let mut sidecars = std::collections::BTreeSet::new();
    for e in entries {
        let (mut chosen, mut sidecar) = (None, None);
        let disposition = match (e.current, e.incoming) {
            (None, Some(new)) => {
                chosen = Some(new);
                Disposition::InstallIncoming
            }
            (Some(current), Some(new))
                if current.contents == e.recorded.contents || current.contents == new.contents =>
            {
                chosen = Some(new);
                Disposition::InstallIncoming
            }
            (Some(current), Some(new)) if new.contents == e.recorded.contents => {
                chosen = Some(current);
                Disposition::KeepLocal
            }
            (Some(current), Some(new)) if e.kind == FileKind::ConfigNoreplace => {
                chosen = Some(current);
                sidecar = Some((format!("{}.rpmnew", e.path), new));
                Disposition::StageIncoming
            }
            (Some(current), Some(new)) => {
                chosen = Some(new);
                sidecar = Some((format!("{}.rpmsave", e.path), current));
                Disposition::SaveLocal
            }
            (Some(current), None) if current.contents != e.recorded.contents => {
                sidecar = Some((format!("{}.rpmsave", e.path), current));
                Disposition::SaveLocal
            }
            (Some(_), None) => Disposition::Remove,
            (None, None) => Disposition::AlreadyMissing,
        };
        let sidecar_name = sidecar.as_ref().map(|(name, _)| name.clone());
        if let Some((name, file)) = sidecar.take() {
            if name.len() > 160 || !catalog.allowed_paths.contains(&name) {
                return Err(refuse(
                    "The required configuration sidecar is outside this task's supplied virtual paths; no files were changed.",
                ));
            }
            if runtime.files.contains_key(&name)
                || result.files.contains_key(&name)
                || candidate
                    .runtime()
                    .installed
                    .iter()
                    .any(|p| catalog.archives[&p.archive].files.contains_key(&name))
                || !sidecars.insert(name.clone())
            {
                return Err(refuse(
                    "A configuration sidecar already exists or collides with this transaction. This subset will not overwrite it; no files were changed.",
                ));
            }
            result.files.insert(name, file.clone());
        }
        match chosen {
            Some(file) => {
                result.files.insert(e.path.into(), file.clone());
            }
            None => {
                result.files.remove(e.path);
            }
        }
        effects.push(Effect {
            path: e.path.into(),
            disposition,
            sidecar: sidecar_name,
        });
    }
    // Existing total file/string/token/raw budgets remain the final authority.
    let state = State::new(catalog.clone(), result)?;
    Ok(Transition {
        state,
        plan,
        configurations: effects,
    })
}
