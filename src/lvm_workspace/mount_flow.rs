//! Source-bound mount drafts, real virtual file readers and state-based goals.
//! Capability prerequisite only until explicitly admitted by Definition/Session.
use super::{bounded, lvm, rename};
use crate::storage;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialMount {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub target: String,
    pub original_volume: u32,
    pub read_only: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub target: String,
    pub original_volume: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub file: String,
    pub read_only: bool,
    pub automatic: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub inventory: lvm::mounts::Context,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial_policy: String,
    #[serde(deserialize_with = "bounded::sequence::<_,InitialMount,4>")]
    pub initial_live: Vec<InitialMount>,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,4>")]
    pub targets: Vec<Target>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renaming: Option<rename::Plan>,
}
fn current_name(s: &lvm::State, id: u32) -> Option<&str> {
    s.identity
        .as_ref()?
        .volumes
        .iter()
        .find(|(_, i)| **i == id)
        .map(|(n, _)| n.as_str())
}
/// Stronger than legacy growth admission: this mount profile never creates,
/// removes, resizes or rewrites an original object. Names/availability may change.
fn same_storage(a: &lvm::State, b: &lvm::State) -> bool {
    if !rename::same_problem(a, b) {
        return false;
    }
    let x = a.identity.as_ref().unwrap();
    let y = b.identity.as_ref().unwrap();
    if x.next != y.next || x.groups.len() != y.groups.len() || x.volumes.len() != y.volumes.len() {
        return false;
    }
    for (old, id) in &x.volumes {
        let Some(now) = current_name(b, *id) else {
            return false;
        };
        if a.volume(old).ok().map(|(_, v)| v) != b.volume(now).ok().map(|(_, v)| v) {
            return false;
        }
    }
    for (old, id) in &x.groups {
        let Some((now, _)) = y.groups.iter().find(|(_, v)| *v == id) else {
            return false;
        };
        if a.groups[old].extent_bytes != b.groups[now].extent_bytes {
            return false;
        }
    }
    a.devices.iter().all(|(p, d)| {
        let n = &b.devices[p];
        d.bytes == n.bytes
            && d.pv_bytes == n.pv_bytes
            && d.role == n.role
            && if d.group.is_empty() {
                n.group.is_empty()
            } else {
                x.groups.get(&d.group) == y.groups.get(&n.group)
            }
    })
}
impl Context {
    pub fn initial(&self, s: &lvm::State) -> Result<lvm::mounts::Mounted> {
        let mut live = BTreeMap::new();
        for m in &self.initial_live {
            if live
                .insert(
                    m.target.clone(),
                    storage::Mount {
                        volume: lvm::mounts::key(m.original_volume),
                        read_only: m.read_only,
                    },
                )
                .is_some()
            {
                return Err("Initial mountpoints must be distinct".into());
            }
        }
        lvm::mounts::Mounted::new(&self.inventory, s, self.initial_policy.clone(), live)
    }
    pub fn shape(&self, s: &lvm::State) -> Result<()> {
        self.inventory.validate(s)?;
        if !bounded::within(self, 16 * 1024)
            || self.initial_live.len() > 4
            || self.targets.is_empty()
            || self.targets.len() > 4
            || s.activation.as_ref().is_none_or(|a| a.open.len() > 1)
            || self.renaming.as_ref().is_some_and(|p| !p.valid(s))
        {
            return Err("Mount practice needs 1–4 supplied requirements, bounded saved work and optional valid name changes".into());
        }
        self.initial(s)?;
        let mut seen = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for t in &self.targets {
            let Some(n) = current_name(s, t.original_volume) else {
                return Err("Mount requirement names a missing original volume".into());
            };
            if !seen.insert(t.target.as_str())
                || !ids.insert(t.original_volume)
                || !self
                    .inventory
                    .filesystems
                    .iter()
                    .any(|f| f.original_volume == t.original_volume)
                || !self.inventory.mountpoints.iter().any(|p| p.0 == t.target)
                || !t.file.starts_with('/')
                || t.file.len() > 128
                || t.file
                    .split('/')
                    .skip(1)
                    .any(|p| p.is_empty() || matches!(p, "." | ".."))
                || s.volume(n)
                    .ok()
                    .and_then(|(_, v)| v.filesystem.as_ref())
                    .is_none_or(|f| !f.files.contains_key(&t.file))
            {
                return Err("Each mountpoint needs one supplied original filesystem, distinct target and existing virtual data file".into());
            }
        }
        if seen
            != self
                .inventory
                .mountpoints
                .iter()
                .map(|p| p.0.as_str())
                .collect()
        {
            return Err("Every supplied mountpoint needs a requirement; unrelated mounts are not silently reassessed".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    SavePolicy {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Live {
    pub mounted: lvm::mounts::Mounted,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub draft: String,
}
#[derive(Debug)]
enum Command<'a> {
    Lvm,
    Mount(&'a str),
    Unmount(&'a str),
    Read(&'a str),
    Mounts(Option<&'a str>),
    Inventory,
}
fn command<'a>(c: &Context, text: &'a str) -> Result<Command<'a>> {
    if text.len() > 2048
        || text.contains(['\0', '\n', '\r'])
        || text.chars().any(|ch| ch.is_control() && ch != '\t')
    {
        return Err("Use one supported simulated command, at most 2048 bytes".into());
    }
    let args = text.split_ascii_whitespace().collect::<Vec<_>>();
    let point = |p: &str| c.inventory.mountpoints.iter().any(|v| v.0 == p);
    match args.as_slice(){
        ["mount",p] if point(p)=>Ok(Command::Mount(p)),
        ["umount",p] if point(p)=>Ok(Command::Unmount(p)),
        ["cat",p] if c.inventory.mountpoints.iter().any(|m|p.strip_prefix(&m.0).is_some_and(|v|v.starts_with('/')))=>Ok(Command::Read(p)),
        ["findmnt"]=>Ok(Command::Mounts(None)),
        ["findmnt",p] if point(p)=>Ok(Command::Mounts(Some(p))),
        ["blkid"]=>Ok(Command::Inventory),
        ["pvs"|"vgs"|"lvs"|"vgchange"|"vgrename"|"lvrename",..]=>{lvm::validate_command(text).map_err(|e|e.to_string())?;Ok(Command::Lvm)},
        _=>Err("Supported here: pvs, vgs, lvs; supplied vgchange/vgrename/lvrename forms; blkid; findmnt [supplied mountpoint]; mount/umount supplied-mountpoint; cat supplied-mountpoint/file. No pipes, redirects, shell, arbitrary flags or host paths run".into()),
    }
}
impl Operation {
    pub fn shape(&self, c: &Context) -> Result<()> {
        match self {
            Self::Command { text } => command(c, text).map(|_| ()),
            Self::SavePolicy { text } => {
                if text.len() > lvm::mounts::MAX_CONFIG
                    || text.lines().count() > 64
                    || text
                        .chars()
                        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
                {
                    Err("Use a saved mount configuration of at most 4096 bytes/64 lines".into())
                } else {
                    Ok(())
                }
            }
        }
    }
    pub fn description(&self) -> String {
        match self {
            Self::Command { text } => text.clone(),
            Self::SavePolicy { .. } => "Save simulated mount configuration".into(),
        }
    }
}
impl Live {
    pub fn new(c: &Context, initial: &lvm::State) -> Result<Self> {
        c.shape(initial)?;
        Ok(Self {
            mounted: c.initial(initial)?,
            draft: c.initial_policy.clone(),
        })
    }
    pub fn validate(&self, c: &Context, initial: &lvm::State, current: &lvm::State) -> Result<()> {
        c.shape(initial)?;
        if !bounded::within(self, 80 * 1024) || !same_storage(initial, current) {
            return Err(
                "Mount work differs from its supplied original storage; previous work retained"
                    .into(),
            );
        }
        Operation::SavePolicy {
            text: self.draft.clone(),
        }
        .shape(c)?;
        self.mounted.validate(&c.inventory, current)
    }
    pub fn set_draft(
        &mut self,
        c: &Context,
        initial: &lvm::State,
        current: &lvm::State,
        text: String,
    ) -> Result<()> {
        self.validate(c, initial, current)?;
        Operation::SavePolicy { text: text.clone() }.shape(c)?;
        self.draft = text;
        Ok(())
    }
    pub fn checks(
        &self,
        c: &Context,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Vec<(bool, String)> {
        if let Err(e) = self.validate(c, initial, current) {
            return vec![(false, e)];
        }
        let synced = self.draft == self.mounted.storage.configuration();
        let mut rows = vec![(
            synced,
            if synced {
                "The editor matches the saved mount configuration".into()
            } else {
                "Unsaved configuration: choose Save configuration, then check again; typing does not change current mounts".into()
            },
        )];
        let errors = self.mounted.storage.validate_configuration();
        rows.push((
            errors.is_empty(),
            if errors.is_empty() {
                "Saved mount configuration refers to supplied filesystems in the supported format"
                    .into()
            } else {
                format!(
                    "Saved mount configuration: {}",
                    errors
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            },
        ));
        let entries = self.mounted.storage.entries().unwrap_or_default();
        for t in &c.targets {
            let key = lvm::mounts::key(t.original_volume);
            let saved = entries.iter().find(|e| e.target == t.target);
            let correct = saved.is_some_and(|e| {
                self.mounted.storage.inventory().resolve(&e.source).ok() == Some(key.as_str())
                    && e.read_only == t.read_only
                    && e.automatic == t.automatic
            });
            rows.push((correct,format!("{}: saved entry must select its intended original filesystem, {} access and {} mounting; saving alone does not replace a live mount",t.target,if t.read_only{"read-only"}else{"read/write"},if t.automatic{"automatic"}else{"manual"})));
            let live = self.mounted.storage.live().get(&t.target);
            rows.push((live.is_some_and(|m|m.volume==key&&m.read_only==t.read_only),format!("{}: current mount {}",t.target,match live{None=>"is absent; mount the saved entry after checking it",Some(m)if m.volume!=key=>"uses another filesystem; inspect, unmount it, then mount the corrected saved entry",Some(m)if m.read_only!=t.read_only=>"has different access options; saved edits do not change the current mount",_=>"uses the intended original filesystem and access options"})));
            let want = current_name(initial, t.original_volume)
                .and_then(|n| initial.volume(n).ok())
                .and_then(|(_, v)| v.filesystem.as_ref())
                .and_then(|f| f.files.get(&t.file))
                .map(|f| f.contents.as_str());
            let actual =
                self.mounted
                    .read_path(&c.inventory, current, &format!("{}{}", t.target, t.file));
            rows.push((
                actual.as_ref().ok().copied() == want,
                match actual {
                    Err(e) => format!("{}{}: {e}", t.target, t.file),
                    Ok(_) if actual.as_ref().ok().copied() != want => format!(
                        "{}{} reads different data; inspect which original filesystem is mounted",
                        t.target, t.file
                    ),
                    Ok(_) => format!("{}{} reads its original virtual data", t.target, t.file),
                },
            ));
        }
        let before = initial.identity.as_ref().unwrap();
        let preserved = before.volumes.iter().all(|(name, id)| {
            c.targets.iter().any(|t| t.original_volume == *id)
                || current_name(current, *id).is_some_and(|n| {
                    initial.activation.as_ref().unwrap().active[name]
                        == current.activation.as_ref().unwrap().active[n]
                })
        });
        rows.push((
            preserved,
            "Keep the active/inactive state of unrelated original volumes unchanged".into(),
        ));
        if let Some(p) = &c.renaming {
            rows.extend(p.outcomes(initial, current));
        } else {
            rows.push((
                initial.identity.as_ref().unwrap() == current.identity.as_ref().unwrap(),
                "Keep supplied group and volume names unchanged".into(),
            ));
        }
        rows
    }
    pub fn apply(
        &mut self,
        c: &Context,
        initial: &lvm::State,
        current: &mut lvm::State,
        op: &Operation,
    ) -> Result<String> {
        self.validate(c, initial, current)?;
        op.shape(c)?;
        let mut next = self.clone();
        let mut world = current.clone();
        let out = match op {
            Operation::SavePolicy { text } => {
                next.mounted.save(&c.inventory, &world, text.clone())?;
                next.draft = text.clone();
                "Saved simulated mount configuration. Current mounts are unchanged.".into()
            }
            Operation::Command { text } => match command(c, text)? {
                Command::Lvm => next.mounted.execute_lvm(&c.inventory, &mut world, text)?,
                Command::Mount(p) => {
                    let result = next.mounted.mount(&c.inventory, &world, p)?;
                    format!(
                        "{}: {}",
                        p,
                        if result == storage::MountOutcome::AlreadyMounted {
                            "already mounted with this saved configuration"
                        } else {
                            "saved filesystem now mounted"
                        }
                    )
                }
                Command::Unmount(p) => {
                    next.mounted.umount(&c.inventory, &world, p)?;
                    format!("{p}: unmounted")
                }
                Command::Read(p) => next.mounted.read_path(&c.inventory, &world, p)?.into(),
                Command::Inventory => next.mounted.storage.inventory().blkid(),
                Command::Mounts(target) => {
                    let mut rows = vec!["TARGET SOURCE FSTYPE OPTIONS".into()];
                    for (p, m) in next.mounted.storage.live() {
                        if target.is_none_or(|t| t == p) {
                            rows.push(format!(
                                "{} {} ext4 {}",
                                p,
                                next.mounted
                                    .storage
                                    .inventory()
                                    .device_for(&m.volume)
                                    .ok_or("Missing current device name")?,
                                if m.read_only { "ro" } else { "rw" }
                            ));
                        }
                    }
                    if target.is_some() && rows.len() == 1 {
                        return Err(
                            "No filesystem is currently mounted at that supplied mountpoint".into(),
                        );
                    }
                    rows.join("\n")
                }
            },
        };
        next.validate(c, initial, &world)?;
        *current = world;
        *self = next;
        Ok(out)
    }
    pub fn save_draft(
        &mut self,
        c: &Context,
        initial: &lvm::State,
        current: &mut lvm::State,
    ) -> Result<String> {
        self.apply(
            c,
            initial,
            current,
            &Operation::SavePolicy {
                text: self.draft.clone(),
            },
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: Operation,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,24>")]
    pub reference: Vec<Operation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Practice {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,12>")]
    pub worked: Vec<Step>,
    pub experiment: Operation,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,12>")]
    pub repair: Vec<Operation>,
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
impl Task {
    pub fn shape(&self, s: &lvm::State) -> Result<()> {
        self.context.shape(s)?;
        if self.reference.is_empty() || self.reference.len() > 24 {
            return Err("Mount assignment needs 1–24 bounded reference actions".into());
        }
        for op in &self.reference {
            op.shape(&self.context)?;
        }
        Ok(())
    }
    pub fn validate(&self, s: &lvm::State) -> Result<()> {
        self.shape(s)?;
        let mut current = s.clone();
        let mut live = Live::new(&self.context, s)?;
        if all(&live.checks(&self.context, s, &current)) {
            return Err("Mount assignment must require real work".into());
        }
        for op in &self.reference {
            live.apply(&self.context, s, &mut current, op)?;
        }
        if !all(&live.checks(&self.context, s, &current)) {
            return Err("Mount reference misses actual saved/current/filesystem outcomes".into());
        }
        Ok(())
    }
}
impl Practice {
    pub fn shape(&self, s: &lvm::State) -> Result<()> {
        self.context.shape(s)?;
        if self.worked.is_empty()
            || self.worked.len() > 12
            || self.repair.is_empty()
            || self.repair.len() > 12
        {
            return Err(
                "Mount practice needs bounded worked actions, a real changed outcome and repair"
                    .into(),
            );
        }
        for step in &self.worked {
            if step.explanation.trim().is_empty()
                || step.explanation.len() > 800
                || step
                    .explanation
                    .chars()
                    .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
            {
                return Err("Explain every mount practice action".into());
            }
            step.operation.shape(&self.context)?;
        }
        self.experiment.shape(&self.context)?;
        for op in &self.repair {
            op.shape(&self.context)?;
        }
        Ok(())
    }
    pub fn validate(&self, s: &lvm::State) -> Result<()> {
        self.shape(s)?;
        let mut current = s.clone();
        let mut live = Live::new(&self.context, s)?;
        if all(&live.checks(&self.context, s, &current)) {
            return Err("Mount practice must start incomplete".into());
        }
        for step in &self.worked {
            live.apply(&self.context, s, &mut current, &step.operation)?;
        }
        if !all(&live.checks(&self.context, s, &current)) {
            return Err("Worked mount practice misses saved/current/data requirements".into());
        }
        let before = (current.clone(), live.clone());
        live.apply(&self.context, s, &mut current, &self.experiment)?;
        if before == (current.clone(), live.clone())
            || all(&live.checks(&self.context, s, &current))
        {
            return Err("Mount exploration must expose a changed incomplete outcome".into());
        }
        for op in &self.repair {
            live.apply(&self.context, s, &mut current, op)?;
        }
        if !all(&live.checks(&self.context, s, &current)) {
            return Err("Mount repair does not restore actual outcomes".into());
        }
        Ok(())
    }
}
