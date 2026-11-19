//! Pure bounded demand-mount/idle-check model. Reuses actual virtual filesystem
//! identity and data; no host clock, daemon, process, filesystem or network I/O.
use super::maps::{self, Entry, Error, ErrorKind, Root, Style};
use crate::{cron_workspace as bounded, lvm};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_CLOCK: u32 = 86400;
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        line: None,
        message: message.into(),
    }
}
fn context_error(message: impl Into<String>) -> Error {
    error(ErrorKind::Context, message)
}
fn lookup_error(message: impl Into<String>) -> Error {
    error(ErrorKind::Lookup, message)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text(#[serde(deserialize_with = "crate::cron::text::<_,4096>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,128>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapFile {
    pub name: Name,
    pub text: Text,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub inventory: lvm::mounts::Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Name,3>")]
    pub map_files: Vec<Name>,
}
impl Context {
    fn names(&self) -> Vec<String> {
        self.map_files.iter().map(|n| n.0.clone()).collect()
    }
    fn points(&self) -> BTreeSet<String> {
        self.inventory
            .mountpoints
            .iter()
            .map(|p| p.0.clone())
            .collect()
    }
    fn shape(&self) -> Result<()> {
        if self.map_files.is_empty() || self.map_files.len() > 3 || !bounded::within(self, 8192) {
            return Err(context_error(
                "AutoFS context exceeds its supplied file roster",
            ));
        }
        maps::master("", &self.names())?;
        if self
            .inventory
            .mountpoints
            .iter()
            .any(|p| !maps::absolute(&p.0) || p.0 == "/-")
        {
            return Err(context_error(
                "Supply ordinary absolute virtual trigger directories",
            ));
        }
        Ok(())
    }
    pub fn validate(&self, world: &lvm::State) -> Result<()> {
        self.shape()?;
        self.inventory.validate(world).map_err(context_error)?;
        for fs in &self.inventory.filesystems {
            let n = world
                .identity
                .as_ref()
                .unwrap()
                .volumes
                .iter()
                .find(|(_, id)| **id == fs.original_volume)
                .map(|(n, _)| n)
                .ok_or_else(|| context_error("Missing supplied original filesystem"))?;
            if world
                .volume(n)
                .map_err(|e| context_error(e.to_string()))?
                .1
                .filesystem
                .as_ref()
                .unwrap()
                .files
                .values()
                .any(|f| f.contents.len() > 4096)
            {
                return Err(context_error(
                    "This unit supplies virtual files of at most 4096 bytes",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub master: Text,
    #[serde(deserialize_with = "bounded::sequence::<_,MapFile,3>")]
    pub maps: Vec<MapFile>,
}
impl Configuration {
    pub fn validate(&self, c: &Context) -> Result<()> {
        c.shape()?;
        if self.maps.len() != c.map_files.len()
            || self.maps.len() > 3
            || !bounded::within(self, 16 * 1024)
        {
            return Err(context_error(
                "Keep the supplied master and map file roster",
            ));
        }
        maps::text(&self.master.0)?;
        let mut seen = BTreeSet::new();
        for f in &self.maps {
            if !seen.insert(&f.name.0) || !c.map_files.contains(&f.name) {
                return Err(context_error("Use each supplied map file once"));
            }
            maps::text(&f.text.0)?;
        }
        Ok(())
    }
    fn file(&self, name: &str) -> Result<&str> {
        self.maps
            .iter()
            .find(|f| f.name.0 == name)
            .map(|f| f.text.0.as_str())
            .ok_or_else(|| context_error("No supplied virtual map file by that name"))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loaded {
    pub master: Text,
    #[serde(deserialize_with = "bounded::sequence::<_,MapFile,3>")]
    pub direct: Vec<MapFile>,
}
fn entries(text: &str, r: &Root, c: &Context) -> Result<Vec<Entry>> {
    maps::entries(text, r, &c.points()).map_err(|mut e| {
        e.message = format!("{}: {}", r.file, e.message);
        e
    })
}
impl Loaded {
    pub fn from_saved(c: &Context, saved: &Configuration) -> Result<Self> {
        saved.validate(c)?;
        let roots = maps::master(&saved.master.0, &c.names())?;
        let mut direct = vec![];
        for r in roots.iter().filter(|r| r.style() == Style::Direct) {
            let text = saved.file(&r.file)?;
            entries(text, r, c)?;
            direct.push(MapFile {
                name: Name(r.file.clone()),
                text: Text(text.into()),
            });
        }
        let result = Self {
            master: saved.master.clone(),
            direct,
        };
        result.roots(c)?;
        Ok(result)
    }
    pub fn roots(&self, c: &Context) -> Result<Vec<Root>> {
        c.shape()?;
        if self.direct.len() > 2 || !bounded::within(self, 12 * 1024) {
            return Err(context_error(
                "Loaded direct-map snapshot exceeds its bounds",
            ));
        }
        let roots = maps::master(&self.master.0, &c.names())?;
        let expected = roots
            .iter()
            .filter(|r| r.style() == Style::Direct)
            .map(|r| r.file.as_str())
            .collect::<BTreeSet<_>>();
        let got = self
            .direct
            .iter()
            .map(|m| m.name.0.as_str())
            .collect::<BTreeSet<_>>();
        if got.len() != self.direct.len() || expected != got {
            return Err(context_error(
                "Loaded direct maps must match the active master roster",
            ));
        }
        let mut targets = BTreeSet::new();
        for r in roots.iter().filter(|r| r.style() == Style::Direct) {
            for e in entries(self.file(&r.file)?, r, c)? {
                if !targets.insert(e.target.clone())
                    || roots
                        .iter()
                        .filter(|r| r.style() == Style::Indirect)
                        .any(|r| {
                            e.target == r.point
                                || e.target.starts_with(&format!("{}/", r.point))
                                || r.point.starts_with(&format!("{}/", e.target))
                        })
                {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "Overlapping direct and indirect trigger namespaces are outside this exercise",
                    ));
                }
            }
        }
        Ok(roots)
    }
    fn file(&self, name: &str) -> Result<&str> {
        self.direct
            .iter()
            .find(|f| f.name.0 == name)
            .map(|f| f.text.0.as_str())
            .ok_or_else(|| context_error("Loaded direct-map snapshot is missing"))
    }
    fn direct_targets(&self, c: &Context) -> Result<BTreeSet<String>> {
        let mut result = BTreeSet::new();
        for r in self.roots(c)?.iter().filter(|r| r.style() == Style::Direct) {
            for e in entries(self.file(&r.file)?, r, c)? {
                result.insert(e.target);
            }
        }
        Ok(result)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lease {
    pub target: Name,
    pub root: Name,
    pub map: Name,
    pub last_used: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub saved: Configuration,
    pub loaded: Loaded,
    pub mounted: lvm::mounts::Mounted,
    pub clock: u32,
    #[serde(deserialize_with = "bounded::sequence::<_,Lease,4>")]
    pub leases: Vec<Lease>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    pub target: String,
    pub original_volume: u32,
    pub content: String,
}
impl State {
    /// The source explicitly supplies active master/direct snapshots separately
    /// from saved edits. No real service is started or inferred from those facts.
    pub fn new(
        c: &Context,
        world: &lvm::State,
        active: &Configuration,
        saved: Configuration,
    ) -> Result<Self> {
        c.validate(world)?;
        active.validate(c)?;
        saved.validate(c)?;
        let s = Self {
            version: 1,
            saved,
            loaded: Loaded::from_saved(c, active)?,
            mounted: lvm::mounts::Mounted::new(&c.inventory, world, String::new(), BTreeMap::new())
                .map_err(context_error)?,
            clock: 0,
            leases: vec![],
        };
        s.validate(c, world)?;
        Ok(s)
    }
    pub fn validate(&self, c: &Context, world: &lvm::State) -> Result<()> {
        c.validate(world)?;
        self.saved.validate(c)?;
        let roots = self.loaded.roots(c)?;
        if self.version != 1
            || self.clock > MAX_CLOCK
            || self.leases.len() > 4
            || !bounded::within(self, 160 * 1024)
        {
            return Err(context_error("Unsupported or oversized saved AutoFS model"));
        }
        self.mounted
            .validate(&c.inventory, world)
            .map_err(context_error)?;
        let mut seen = BTreeSet::new();
        for l in &self.leases {
            if !seen.insert(l.target.0.as_str())
                || l.last_used > self.clock
                || !roots.iter().any(|r| {
                    r.point == l.root.0
                        && r.file == l.map.0
                        && match r.style() {
                            Style::Direct => entries(self.loaded.file(&r.file).unwrap_or(""), r, c)
                                .is_ok_and(|v| v.iter().any(|e| e.target == l.target.0)),
                            Style::Indirect => l
                                .target
                                .0
                                .strip_prefix(&format!("{}/", r.point))
                                .is_some_and(|key| !key.is_empty() && !key.contains('/')),
                        }
                })
            {
                return Err(context_error(
                    "Current mount activity differs from its loaded roots or bounded clock",
                ));
            }
        }
        if seen
            != self
                .mounted
                .storage
                .live()
                .keys()
                .map(String::as_str)
                .collect()
        {
            return Err(context_error(
                "Every current backing mount needs exactly one supplied activity record",
            ));
        }
        Ok(())
    }
    /// Save bounded but possibly malformed work. Indirect maps are consulted on
    /// new lookups; master/direct snapshots change only on explicit refresh.
    pub fn save(
        &mut self,
        c: &Context,
        world: &lvm::State,
        file: &str,
        text: String,
    ) -> Result<()> {
        self.validate(c, world)?;
        maps::text(&text)?;
        let mut next = self.clone();
        if file == "/etc/auto.master" {
            next.saved.master = Text(text);
        } else {
            let f = next
                .saved
                .maps
                .iter_mut()
                .find(|f| f.name.0 == file)
                .ok_or_else(|| context_error("Choose a supplied virtual configuration file"))?;
            f.text = Text(text);
        }
        next.validate(c, world)?;
        *self = next;
        Ok(())
    }
    pub fn reload(&mut self, c: &Context, world: &lvm::State) -> Result<()> {
        self.validate(c, world)?;
        let next = Loaded::from_saved(c, &self.saved)?;
        if !self.leases.is_empty()
            && (next.roots(c)? != self.loaded.roots(c)?
                || next.direct_targets(c)? != self.loaded.direct_targets(c)?)
        {
            return Err(error(
                ErrorKind::Unsupported,
                "Changing managed roots, timeouts or direct-key sets while backing filesystems are mounted is outside this simulator subset. Let the provided nonbusy mounts expire first; this is not a universal Linux reload rule",
            ));
        }
        let mut result = self.clone();
        result.loaded = next;
        result.validate(c, world)?;
        *self = result;
        Ok(())
    }
    fn route(&self, c: &Context, target: &str) -> Result<(Root, Entry)> {
        for r in self.loaded.roots(c)? {
            if r.style() == Style::Direct {
                if let Some(e) = entries(self.loaded.file(&r.file)?, &r, c)?
                    .into_iter()
                    .find(|e| e.target == target)
                {
                    return Ok((r, e));
                }
            } else if target
                .strip_prefix(&format!("{}/", r.point))
                .is_some_and(|key| !key.contains('/'))
                && let Some(e) = entries(self.saved.file(&r.file)?, &r, c)?
                    .into_iter()
                    .find(|e| e.target == target)
            {
                return Ok((r, e));
            }
        }
        Err(lookup_error(format!(
            "No entry in the active master/map selects {target}. Check the managed directory and map key together"
        )))
    }
    /// A successful demand mount remains even if the requested file is absent.
    /// Callers must not pretend every read error rolls back a real path lookup.
    pub fn read(&mut self, c: &Context, world: &lvm::State, path: &str) -> Result<Read> {
        self.validate(c, world)?;
        if path.len() > 256
            || !path.starts_with('/')
            || !path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'_' | b'-' | b'.'))
            || path
                .split('/')
                .skip(1)
                .any(|p| p.is_empty() || matches!(p, "." | ".."))
        {
            return Err(error(
                ErrorKind::Unsupported,
                "Read a supplied absolute virtual file path without traversal, quoting or control characters",
            ));
        }
        let target = c
            .inventory
            .mountpoints
            .iter()
            .find(|p| path.strip_prefix(&p.0).is_some_and(|s| s.starts_with('/')))
            .map(|p| p.0.clone())
            .ok_or_else(|| {
                error(
                    ErrorKind::Unsupported,
                    "This file is outside the supplied virtual trigger mountpoints",
                )
            })?;
        if !self.mounted.storage.live().contains_key(&target) {
            let (r, e) = self.route(c, &target)?;
            let mut next = self.mounted.clone();
            next.save(
                &c.inventory,
                world,
                format!(
                    "{} {} ext4 {} 0 2\n",
                    e.device,
                    e.target,
                    if e.read_only { "ro" } else { "rw" }
                ),
            )
            .map_err(context_error)?;
            next.mount(&c.inventory, world, &target)
                .map_err(lookup_error)?;
            self.mounted = next;
            self.leases.push(Lease {
                target: Name(target.clone()),
                root: Name(r.point),
                map: Name(r.file),
                last_used: self.clock,
            });
        }
        let lease = self
            .leases
            .iter_mut()
            .find(|l| l.target.0 == target)
            .ok_or_else(|| context_error("Current mount lacks its activity record"))?;
        lease.last_used = self.clock;
        let key = &self.mounted.storage.live()[&target].volume;
        let original_volume = c
            .inventory
            .filesystems
            .iter()
            .find(|f| lvm::mounts::key(f.original_volume) == *key)
            .ok_or_else(|| context_error("Missing original filesystem binding"))?
            .original_volume;
        let result = self
            .mounted
            .read_path(&c.inventory, world, path)
            .map_err(lookup_error)
            .map(|s| s.to_owned());
        self.validate(c, world)?;
        result.map(|content| Read {
            target,
            original_volume,
            content,
        })
    }
    /// Explicit settled idle check, not wall-clock sleep or exact kernel timing.
    /// The provided world has no modeled busy processes or open-file handles.
    pub fn advance(
        &mut self,
        c: &Context,
        world: &lvm::State,
        seconds: u32,
    ) -> Result<Vec<String>> {
        self.validate(c, world)?;
        let clock=self.clock.checked_add(seconds).filter(|n|*n<=MAX_CLOCK&&seconds>0&&seconds<=3600).ok_or_else(||error(ErrorKind::Limit,"Advance 1–3600 simulated seconds at a time, within this exercise's 86400-second clock"))?;
        let roots = self.loaded.roots(c)?;
        let mut expired = vec![];
        for l in &self.leases {
            let r = roots
                .iter()
                .find(|r| r.point == l.root.0 && r.file == l.map.0)
                .ok_or_else(|| context_error("Missing active root for idle check"))?;
            if clock - l.last_used >= u32::from(r.timeout) {
                expired.push(l.target.0.clone());
            }
        }
        let mut next = self.clone();
        next.clock = clock;
        for target in &expired {
            next.mounted
                .umount(&c.inventory, world, target)
                .map_err(context_error)?;
        }
        next.leases.retain(|l| !expired.contains(&l.target.0));
        next.validate(c, world)?;
        *self = next;
        Ok(expired)
    }
}
