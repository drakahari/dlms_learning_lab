//! Current storage outcomes, saved drafts and in-memory investigation commands.
use crate::storage::{self, Inventory, Mount, MountOutcome, State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_DRAFT_BYTES: usize = storage::MAX_CONFIG_BYTES * 4;
pub const MAX_OUTPUT: usize = 32_768;
pub const HELP: &str = "blkid [DEVICE] • cat /etc/fstab • mount • mount TARGET • umount TARGET • help. Literal paths from this activity only; no shell expansion, pipelines, redirection, remount, -a or host access. Edit saved configuration in the native editor. Validate configuration and Test fresh session are simulator actions, not Linux commands.";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Configured {
        target: String,
        volume: String,
        read_only: bool,
        automatic: bool,
        dump: u32,
        pass: u32,
    },
    Unconfigured {
        target: String,
    },
    Mounted {
        target: String,
        volume: String,
        read_only: bool,
    },
    Unmounted {
        target: String,
    },
    FreshSession {
        #[serde(deserialize_with = "storage::map")]
        devices: BTreeMap<String, String>,
        #[serde(deserialize_with = "storage::map")]
        expected: BTreeMap<String, Option<Mount>>,
    },
}
impl Requirement {
    fn check(&self, state: &State, requirements: &[Requirement]) -> (bool, String) {
        match self {
            Self::Configured {
                target,
                volume,
                read_only,
                automatic,
                dump,
                pass,
            } => {
                let entries = match storage::parse(state.configuration()) {
                    Ok(e) => e,
                    Err(e) => return (false, format!("Saved configuration: {e}")),
                };
                let Some(entry) = entries.iter().find(|e| &e.target == target) else {
                    return (
                        false,
                        format!(
                            "Saved configuration: {target} has no entry. Keep the required volume available in the saved table."
                        ),
                    );
                };
                let id = match state.inventory().resolve(&entry.source) {
                    Ok(id) => id,
                    Err(e) => return (false, format!("Saved {target}: {e}")),
                };
                if id != volume {
                    return (
                        false,
                        format!(
                            "Saved {target} selects a different volume. Compare its source with the activity's volume identities and the blkid inventory."
                        ),
                    );
                }
                if state.inventory().volumes()[id].filesystem != entry.filesystem {
                    return (
                        false,
                        format!(
                            "Saved {target}: the filesystem type differs from TYPE in the inventory."
                        ),
                    );
                }
                if entry.read_only != *read_only {
                    return (
                        false,
                        format!(
                            "Saved {target} has the wrong access policy. {}",
                            if *read_only {
                                "This volume must remain read-only (ro)."
                            } else {
                                "This work volume must allow writes (rw)."
                            }
                        ),
                    );
                }
                if entry.automatic != *automatic {
                    return (
                        false,
                        format!(
                            "Saved {target} has the wrong automatic-mount policy. {}",
                            if *automatic {
                                "It must be selected for the fresh-session test (auto)."
                            } else {
                                "Keep it available for an explicit mount, but skip automatic selection (noauto)."
                            }
                        ),
                    );
                }
                if entry.dump != *dump || entry.pass != *pass {
                    return (
                        false,
                        format!(
                            "Saved {target}: keep the supplied dump/check fields. This activity stores those values but does not run backups or filesystem checks."
                        ),
                    );
                }
                (
                    true,
                    format!("Saved {target}: intended volume and access/automatic policy"),
                )
            }
            Self::Unconfigured { target } => {
                let entries = match storage::parse(state.configuration()) {
                    Ok(e) => e,
                    Err(e) => return (false, format!("Saved configuration: {e}")),
                };
                (
                    !entries.iter().any(|e| &e.target == target),
                    format!("Saved table must have no entry for {target}"),
                )
            }
            Self::Mounted {
                target,
                volume,
                read_only,
            } => {
                let Some(m) = state.live().get(target) else {
                    return (
                        false,
                        format!(
                            "Live {target}: nothing is mounted. Save the intended entry, then explicitly mount it."
                        ),
                    );
                };
                if &m.volume != volume {
                    return (
                        false,
                        format!(
                            "Live {target} still uses a different volume. Saving configuration does not replace a live mount; inspect it before unmounting and mounting the intended entry."
                        ),
                    );
                }
                if m.read_only != *read_only {
                    return (
                        false,
                        format!(
                            "Live {target} is {}, but the task needs {}. Saved edits do not change the live policy; this subset uses unmount, then mount.",
                            mode(m.read_only),
                            mode(*read_only)
                        ),
                    );
                }
                (
                    true,
                    format!("Live {target}: intended volume, {}", mode(*read_only)),
                )
            }
            Self::Unmounted { target } => (
                !state.live().contains_key(target),
                format!("Live {target} must remain unmounted"),
            ),
            Self::FreshSession { devices, expected } => {
                if !state.validate_configuration().is_empty() {
                    return(false,"Fresh-session test needs a valid saved configuration. Fix the reported source/type/format issues first.".into());
                }
                let fresh = match state.fresh_session(devices.clone()) {
                    Ok(f) => f,
                    Err(e) => return (false, format!("Fresh-session test: {e}")),
                };
                if let Some(issue) = fresh.state.validate_configuration().first() {
                    return (false, format!("Fresh-session saved configuration: {issue}"));
                }
                for saved in requirements
                    .iter()
                    .filter(|r| matches!(r, Self::Configured { .. } | Self::Unconfigured { .. }))
                {
                    let (valid, detail) = saved.check(&fresh.state, &[]);
                    if !valid {
                        return (
                            false,
                            format!("Fresh-session saved identity/policy: {detail}"),
                        );
                    }
                }
                for (target, want) in expected {
                    if fresh.state.live().get(target) != want.as_ref() {
                        return (
                            false,
                            format!(
                                "Fresh-session test at {target}: {} The test changes device names; check stable volume identity and saved auto/noauto and ro/rw options.",
                                if want.is_none() {
                                    "this mount should be skipped automatically."
                                } else {
                                    "the intended volume and access policy did not return."
                                }
                            ),
                        );
                    }
                }
                (true,"Fresh-session test: intended automatic mounts survive the controlled device-name change; manual mounts stay skipped".into())
            }
        }
    }
}
fn mode(read_only: bool) -> &'static str {
    if read_only { "read-only" } else { "writable" }
}

/// Every fixed mountpoint needs explicit saved and live outcomes; a full fresh
/// table must include at least one positive mount. Absence alone cannot pass.
pub fn valid_requirements(requirements: &[Requirement], inventory: &Inventory) -> bool {
    if requirements.is_empty() || requirements.len() > 32 {
        return false;
    }
    let mut saved = BTreeSet::new();
    let mut live = BTreeSet::new();
    let mut sessions = BTreeSet::new();
    for r in requirements {
        match r {
            Requirement::Configured { target, volume, .. } => {
                if !inventory.mountpoints().contains(target)
                    || !inventory.volumes().contains_key(volume)
                    || !saved.insert(target.clone())
                {
                    return false;
                }
            }
            Requirement::Unconfigured { target } => {
                if !inventory.mountpoints().contains(target) || !saved.insert(target.clone()) {
                    return false;
                }
            }
            Requirement::Mounted { target, volume, .. } => {
                if !inventory.mountpoints().contains(target)
                    || !inventory.volumes().contains_key(volume)
                    || !live.insert(target.clone())
                {
                    return false;
                }
            }
            Requirement::Unmounted { target } => {
                if !inventory.mountpoints().contains(target) || !live.insert(target.clone()) {
                    return false;
                }
            }
            Requirement::FreshSession { devices, expected } => {
                if devices.keys().ne(inventory.devices().keys())
                    || devices
                        .values()
                        .any(|id| !inventory.volumes().contains_key(id))
                    || expected.keys().ne(inventory.mountpoints().iter())
                    || Inventory::new(
                        inventory.volumes().clone(),
                        devices.clone(),
                        inventory.mountpoints().iter().cloned().collect(),
                    )
                    .is_err()
                    || expected
                        .values()
                        .flatten()
                        .any(|m| !inventory.volumes().contains_key(&m.volume))
                    || !expected.values().any(Option::is_some)
                    || !sessions.insert(format!("{devices:?}"))
                {
                    return false;
                }
                let ids: BTreeSet<_> = expected.values().flatten().map(|m| &m.volume).collect();
                if ids.len() != expected.values().flatten().count() {
                    return false;
                }
            }
        }
    }
    saved == *inventory.mountpoints()
        && live == *inventory.mountpoints()
        && !sessions.is_empty()
        && requirements
            .iter()
            .any(|r| matches!(r, Requirement::Mounted { .. }))
        && requirements.iter().any(
            |r| matches!(r,Requirement::FreshSession{devices,..} if devices!=inventory.devices()),
        )
}

pub fn checks(state: &State, requirements: &[Requirement]) -> Vec<(bool, String)> {
    let mut checks = state
        .validate_configuration()
        .into_iter()
        .map(|e| (false, format!("Configuration needs attention: {e}")))
        .collect::<Vec<_>>();
    checks.extend(requirements.iter().map(|r| r.check(state, requirements)));
    checks
}

/// Diagnose configuration root causes first without repeating them for every
/// dependent outcome. The complete objective list above still controls grading.
pub fn issues(state: &State, requirements: &[Requirement]) -> Vec<String> {
    let errors = state.validate_configuration();
    if !errors.is_empty() {
        return errors.iter().map(ToString::to_string).collect();
    }
    let mut seen = BTreeSet::new();
    checks(state, requirements)
        .into_iter()
        .filter_map(|(ok, message)| (!ok && seen.insert(message.clone())).then_some(message))
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub text: String,
    pub command: String,
    pub output: String,
    pub notice: Option<String>,
}
impl Draft {
    pub fn new(state: &State) -> Self {
        Self {
            version: 1,
            text: state.configuration().into(),
            command: String::new(),
            output: String::new(),
            notice: None,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.text.len() <= MAX_DRAFT_BYTES
            && self.command.len() <= 4096
            && self.output.len() <= MAX_OUTPUT
            && self.notice.as_ref().is_none_or(|s| s.len() <= 4096)
    }
    pub fn matches(&self, state: &State) -> bool {
        self.text == state.configuration()
    }
    pub fn save(&mut self, state: &mut State) -> bool {
        match state.save_configuration(self.text.clone()) {
            Ok(()) => {
                self.notice=Some("Configuration saved. Existing live mounts keep their current volumes and policies.".into());
                true
            }
            Err(e) => {
                self.notice = Some(format!("Not saved: {e}"));
                false
            }
        }
    }
    pub fn append_output(&mut self, text: &str) {
        let text = if text.len() > MAX_OUTPUT - 64 {
            let mut cut = text.len() - (MAX_OUTPUT - 64);
            while !text.is_char_boundary(cut) {
                cut += 1;
            }
            self.output.clear();
            self.output.push_str("[Earlier simulated output omitted]\n");
            &text[cut..]
        } else {
            text
        };
        self.output.push_str(text);
        if self.output.len() > MAX_OUTPUT {
            let mut cut = self.output.len() - (MAX_OUTPUT - 64);
            while !self.output.is_char_boundary(cut) {
                cut += 1;
            }
            self.output = self.output[cut..].to_owned();
            self.output
                .insert_str(0, "[Earlier simulated output omitted]\n");
        }
    }
    pub fn run_command(&mut self, state: &mut State) {
        let input = std::mem::take(&mut self.command);
        if input.len() > 4096 {
            self.append_output("Command exceeds this simulator's 4KiB input limit.\n");
            return;
        }
        let result = command(state, &input).unwrap_or_else(|e| format!("{e}\n"));
        self.append_output(&format!(
            "$ {input}\n{result}{}",
            if result.ends_with('\n') || result.is_empty() {
                ""
            } else {
                "\n"
            }
        ));
    }
}

/// The learner terminal accepts this literal subset only, never pack JSON actions.
pub fn command(state: &mut State, input: &str) -> Result<String, String> {
    if input.len() > 4096 {
        return Err("Command exceeds this simulator's 4KiB input limit.".into());
    }
    let parts = input.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["help"] => Ok(HELP.into()),
        ["blkid"] => Ok(state.inventory().blkid()),
        ["blkid", device] => {
            let Some(id) = state.inventory().devices().get(*device) else {
                return Err("blkid: no matching virtual inventory device. Use blkid to inspect the available devices.".into());
            };
            let v = &state.inventory().volumes()[id];
            Ok(format!(
                "{device}: UUID=\"{}\" TYPE=\"{}\"",
                v.uuid,
                v.filesystem.name()
            ))
        }
        ["cat", "/etc/fstab"] => Ok(state.configuration().into()),
        ["mount"] => Ok(state.mount_listing()),
        ["mount", target] if !target.starts_with('-') => state
            .mount(target)
            .map(|outcome| {
                if outcome == MountOutcome::AlreadyMounted {
                    format!("mount: {target}: already mounted with the saved volume and policy.")
                } else {
                    String::new()
                }
            })
            .map_err(|e| format!("mount: {e}")),
        ["umount", target] if !target.starts_with('-') => state
            .umount(target)
            .map(|()| String::new())
            .map_err(|e| format!("umount: {e}")),
        _ => Err(format!(
            "Unsupported command or arguments in this storage simulator. {HELP}"
        )),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Save { text: String },
    Mount { target: String },
    Unmount { target: String },
}
impl Action {
    pub fn apply(&self, state: &mut State) -> Result<String, String> {
        match self {
            Self::Save { text } => state
                .save_configuration(text.clone())
                .map(|()| "Saved simulated mount configuration; live mounts unchanged.".into())
                .map_err(|e| e.to_string()),
            Self::Mount { target } => state
                .mount(target)
                .map(|_| String::new())
                .map_err(|e| e.to_string()),
            Self::Unmount { target } => state
                .umount(target)
                .map(|()| String::new())
                .map_err(|e| e.to_string()),
        }
    }
}
/// Pack-only reachability adapter. The native terminal calls command instead.
pub fn adapter(state: &mut State, input: &str) -> Result<String, String> {
    if input.len() > MAX_DRAFT_BYTES + 256 {
        return Err("Storage action is too large.".into());
    }
    if let Some(json) = input.strip_prefix("storage ") {
        let action: Action = serde_json::from_str(json)
            .map_err(|_| "Invalid declarative storage action".to_owned())?;
        action.apply(state)
    } else {
        command(state, input)
    }
}
