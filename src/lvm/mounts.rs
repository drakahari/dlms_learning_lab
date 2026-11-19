//! Pure bounded LVM↔existing virtual-mount adapter prerequisite. Not a new
//! normal command, source Goal, session or UI route. Never accesses host storage.
use super::State as LvmState;
use crate::{cron_workspace as bounded, storage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_FILESYSTEMS: usize = 4;
pub const MAX_CONFIG: usize = 4096;
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filesystem {
    pub original_volume: u32,
    /// Explicit supplied synthetic filesystem metadata, NOT an internal LV ID,
    /// real block-device probe, PVID or automatically generated learner answer.
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub uuid: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mountpoint(#[serde(deserialize_with = "crate::cron::text::<_,128>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    #[serde(deserialize_with = "bounded::sequence::<_,Filesystem,4>")]
    pub filesystems: Vec<Filesystem>,
    #[serde(deserialize_with = "bounded::sequence::<_,Mountpoint,4>")]
    pub mountpoints: Vec<Mountpoint>,
}
pub fn key(original: u32) -> String {
    format!("lvm-{original}")
}
fn name(s: &LvmState, original: u32) -> Result<&str> {
    s.identity
        .as_ref()
        .and_then(|ids| {
            ids.volumes
                .iter()
                .find(|(_, id)| **id == original)
                .map(|(name, _)| name.as_str())
        })
        .ok_or_else(|| {
            "The supplied original filesystem volume is missing; no replacement is inferred".into()
        })
}
fn config(text: &str) -> Result<()> {
    if text.len() > MAX_CONFIG
        || text.lines().count() > 64
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(
            "Use at most 4096 bytes/64 lines with ordinary whitespace; previous mount work retained"
                .into(),
        );
    }
    Ok(())
}
impl Context {
    pub fn inventory(&self, s: &LvmState) -> Result<storage::Inventory> {
        if self.filesystems.is_empty()
            || self.filesystems.len() > MAX_FILESYSTEMS
            || self.mountpoints.is_empty()
            || self.mountpoints.len() > 4
            || !bounded::within(self, 8192)
            || !bounded::within(s, 64 * 1024)
        {
            return Err(
                "Mount context needs 1–4 supplied filesystems/points and bounded virtual storage"
                    .into(),
            );
        }
        s.validate().map_err(|e| e.to_string())?;
        if s.identity.is_none()
            || s.activation.is_none()
            || s.snapshots.is_some()
            || s.pv_capacity.is_some()
        {
            return Err("Mount adapter needs original-object identity/availability and ordinary filesystem LVs; other profiles stay separate".into());
        }
        let mut originals = BTreeSet::new();
        let mut volumes = BTreeMap::new();
        let mut devices = BTreeMap::new();
        for f in &self.filesystems {
            if !originals.insert(f.original_volume) {
                return Err("Supply each original filesystem once".into());
            }
            let current = name(s, f.original_volume)?;
            let (_, lv) = s.volume(current).map_err(|e| e.to_string())?;
            if lv.filesystem.is_none() {
                return Err("A raw logical volume has no modeled filesystem to mount".into());
            }
            let (vg, lv) = current
                .split_once('/')
                .ok_or("Invalid current volume name")?;
            let device = super::device_path::canonical(vg, lv).map_err(|e| e.to_string())?;
            let id = key(f.original_volume);
            volumes.insert(
                id.clone(),
                storage::Volume {
                    uuid: f.uuid.clone(),
                    filesystem: storage::Filesystem::Ext4,
                },
            );
            devices.insert(device, id);
        }
        storage::Inventory::new(
            volumes,
            devices,
            self.mountpoints.iter().map(|p| p.0.clone()).collect(),
        )
        .map_err(|e| e.to_string())
    }
    pub fn validate(&self, s: &LvmState) -> Result<()> {
        self.inventory(s).map(|_| ())
    }
    fn original(&self, key_value: &str) -> Result<u32> {
        self.filesystems
            .iter()
            .find(|f| key(f.original_volume) == key_value)
            .map(|f| f.original_volume)
            .ok_or_else(|| {
                "Mounted filesystem is outside the supplied original-object bindings".into()
            })
    }
}
/// Owns saved policy and actual live mount associations. Files remain in LVM's
/// real virtual filesystems; there is no copied/canned application response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mounted {
    pub version: u32,
    pub storage: storage::State,
}
impl Mounted {
    pub fn new(
        c: &Context,
        s: &LvmState,
        text: String,
        live: BTreeMap<String, storage::Mount>,
    ) -> Result<Self> {
        config(&text)?;
        let result = Self {
            version: 1,
            storage: storage::State::new(c.inventory(s)?, text, live).map_err(|e| e.to_string())?,
        };
        result.validate(c, s)?;
        Ok(result)
    }
    pub fn validate(&self, c: &Context, s: &LvmState) -> Result<()> {
        if self.version != 1 || !bounded::within(self, 64 * 1024) {
            return Err("Unsupported or oversized virtual mount work".into());
        }
        config(self.storage.configuration())?;
        self.storage.validate().map_err(|e| e.to_string())?;
        if !self.storage.active_swap().is_empty() || self.storage.inventory() != &c.inventory(s)? {
            return Err(
                "Mount policy/inventory differs from current supplied filesystem names or metadata"
                    .into(),
            );
        }
        for mount in self.storage.live().values() {
            let id = c.original(&mount.volume)?;
            if !super::activation::is_active(s, name(s, id)?).map_err(|e| e.to_string())? {
                return Err("A mounted filesystem must remain available; unmount it before deactivating its group".into());
            }
        }
        Ok(())
    }
    pub fn save(&mut self, c: &Context, s: &LvmState, text: String) -> Result<()> {
        self.validate(c, s)?;
        config(&text)?;
        let mut next = self.clone();
        next.storage
            .save_configuration(text)
            .map_err(|e| e.to_string())?;
        next.validate(c, s)?;
        *self = next;
        Ok(())
    }
    pub fn mount(
        &mut self,
        c: &Context,
        s: &LvmState,
        target: &str,
    ) -> Result<storage::MountOutcome> {
        self.validate(c, s)?;
        let entries = self.storage.entries().map_err(|e| e.to_string())?;
        let entry = entries
            .iter()
            .find(|e| e.target == target)
            .ok_or("No saved entry names this supplied mountpoint")?;
        let id = self
            .storage
            .inventory()
            .resolve(&entry.source)
            .map_err(|e| e.to_string())?;
        let original = c.original(id)?;
        if !super::activation::is_active(s, name(s, original)?).map_err(|e| e.to_string())? {
            return Err("This filesystem's LV is inactive. Activate its supplied group before mounting; its data is retained".into());
        }
        let mut next = self.clone();
        let output = next.storage.mount(target).map_err(|e| e.to_string())?;
        next.validate(c, s)?;
        *self = next;
        Ok(output)
    }
    pub fn umount(&mut self, c: &Context, s: &LvmState, target: &str) -> Result<()> {
        self.validate(c, s)?;
        let mut next = self.clone();
        next.storage.umount(target).map_err(|e| e.to_string())?;
        next.validate(c, s)?;
        *self = next;
        Ok(())
    }
    pub fn read_path<'a>(&self, c: &Context, s: &'a LvmState, path: &str) -> Result<&'a str> {
        self.validate(c, s)?;
        if !path.starts_with('/')
            || path.len() > 256
            || path.bytes().any(|b| b.is_ascii_control())
            || path
                .split('/')
                .skip(1)
                .any(|p| p.is_empty() || matches!(p, "." | ".."))
        {
            return Err("Use an absolute supplied virtual file path without traversal; no host path is accessed".into());
        }
        let point = c
            .mountpoints
            .iter()
            .find(|p| {
                path.strip_prefix(&p.0)
                    .is_some_and(|suffix| suffix.starts_with('/'))
            })
            .ok_or("This file is outside the supplied mountpoints")?;
        let mount=self.storage.live().get(&point.0).ok_or("No filesystem is mounted here; the underlying directory contents are outside this exercise")?;
        let original = c.original(&mount.volume)?;
        s.read_file(name(s, original)?, &path[point.0.len()..])
            .map_err(|e| e.to_string())
    }
    /// Restricted existing in-process LVM actions, atomically coupled to current
    /// mounts. Never passes text to a host shell, device tool or filesystem API.
    pub fn execute_lvm(&mut self, c: &Context, s: &mut LvmState, text: &str) -> Result<String> {
        self.validate(c, s)?;
        if text.len() > 2048
            || text.contains(['\0', '\n', '\r'])
            || !matches!(
                text.split_ascii_whitespace().next(),
                Some("pvs" | "vgs" | "lvs" | "vgchange" | "vgrename" | "lvrename")
            )
        {
            return Err("This mount profile supports supplied LVM inspection, named-group availability and existing name changes only; no command ran".into());
        }
        super::validate_command(text).map_err(|e| e.to_string())?;
        let mut next = s.clone();
        let output = next.execute(text).map_err(|e| e.to_string())?;
        // Reproject current aliases while retaining live filesystem identity and
        // saved text. This is not storage::fresh_session's startup permutation.
        let projection = Self::new(
            c,
            &next,
            self.storage.configuration().into(),
            self.storage.live().clone(),
        )?;
        *s = next;
        *self = projection;
        Ok(output)
    }
}
