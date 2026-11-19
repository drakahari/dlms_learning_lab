//! Source-bound local AutoFS activities. Commands only enter the pure runtime;
//! exact replay validates saved effects, never supplies grading authority.
use crate::{
    autofs::{maps, runtime as core},
    cron_workspace as bounded, lvm,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_HISTORY: usize = 64;
pub const MAX_SESSION: usize = 1024 * 1024;
pub const CHECK: &str = "@autofs-check";
pub const HELP: &str = "Edit only the supplied virtual master/map files and choose Save file. Commands: cat supplied-file; findmnt -t ext4 [provided-mountpoint]; systemctl reload autofs. Only local ext4 FILE maps, supplied devices and nonbusy idle checks are modeled. Idle checks are simulator controls, not Linux commands. No real daemon, files, shell, devices or network is accessed.";
type Result<T> = std::result::Result<T, String>;
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty()
        && s.len() <= n
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub point: core::Name,
    pub original_volume: u32,
    pub file: core::Name,
    pub read_only: bool,
    pub style: maps::Style,
    pub timeout: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub world: lvm::State,
    pub context: core::Context,
    pub active: core::Configuration,
    pub saved: core::Configuration,
    #[serde(deserialize_with = "bounded::sequence::<_,core::Name,4>")]
    pub warm: Vec<core::Name>,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,4>")]
    pub targets: Vec<Target>,
}
fn route(
    c: &core::Context,
    loaded: &core::Loaded,
    saved: &core::Configuration,
    point: &str,
) -> Result<(maps::Root, maps::Entry)> {
    let points = c
        .inventory
        .mountpoints
        .iter()
        .map(|p| p.0.clone())
        .collect::<BTreeSet<_>>();
    for root in loaded.roots(c).map_err(|e| e.to_string())? {
        let files = if root.style() == maps::Style::Direct {
            &loaded.direct
        } else {
            &saved.maps
        };
        let text = &files
            .iter()
            .find(|f| f.name.0 == root.file)
            .ok_or("Missing supplied map")?
            .text
            .0;
        if let Some(entry) = maps::entries(text, &root, &points)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|e| e.target == point)
        {
            return Ok((root, entry));
        }
    }
    Err(format!(
        "No active key selects {point}; check master directory and map key together"
    ))
}
impl Activity {
    pub fn validate(&self) -> Result<()> {
        if !bounded::within(self, 128 * 1024)
            || self.warm.len() > 4
            || self.targets.is_empty()
            || self.targets.len() > 4
        {
            return Err(
                "AutoFS activity exceeds supplied storage, startup reads or requirements".into(),
            );
        }
        self.context
            .validate(&self.world)
            .map_err(|e| e.to_string())?;
        self.active
            .validate(&self.context)
            .and_then(|_| self.saved.validate(&self.context))
            .map_err(|e| e.to_string())?;
        let mut seen = BTreeSet::new();
        for t in &self.targets {
            let name = self
                .world
                .identity
                .as_ref()
                .unwrap()
                .volumes
                .iter()
                .find(|(_, id)| **id == t.original_volume)
                .map(|(n, _)| n)
                .ok_or("Requirement needs an original supplied filesystem")?;
            if !seen.insert(&t.point.0)
                || !self
                    .context
                    .inventory
                    .mountpoints
                    .iter()
                    .any(|p| p.0 == t.point.0)
                || !self
                    .context
                    .inventory
                    .filesystems
                    .iter()
                    .any(|f| f.original_volume == t.original_volume)
                || !maps::absolute(&t.file.0)
                || !(1..=3600).contains(&t.timeout)
                || self.world.read_file(name, &t.file.0).is_err()
            {
                return Err("Every requirement needs a distinct supplied trigger point, original data file and explicit supported timeout".into());
            }
        }
        if seen
            != self
                .context
                .inventory
                .mountpoints
                .iter()
                .map(|p| &p.0)
                .collect()
        {
            return Err("Every provided trigger point needs a meaningful requirement".into());
        }
        self.initial()?;
        Ok(())
    }
    pub fn initial(&self) -> Result<core::State> {
        let mut r = core::State::new(&self.context, &self.world, &self.active, self.saved.clone())
            .map_err(|e| e.to_string())?;
        for path in &self.warm {
            r.read(&self.context, &self.world, &path.0)
                .map_err(|e| format!("Supplied startup read: {e}"))?;
        }
        Ok(r)
    }
    fn entry_correct(
        &self,
        t: &Target,
        root: &maps::Root,
        e: &maps::Entry,
        r: &core::State,
    ) -> bool {
        root.style() == t.style
            && root.timeout == t.timeout
            && e.read_only == t.read_only
            && r.mounted
                .storage
                .inventory()
                .resolve(&crate::storage::Source::Device(e.device.clone()))
                .ok()
                == Some(lvm::mounts::key(t.original_volume).as_str())
    }
    pub fn checks(&self, r: &core::State) -> Vec<(bool, String)> {
        if let Err(e) = r.validate(&self.context, &self.world) {
            return vec![(false, e.to_string())];
        }
        let saved = core::Loaded::from_saved(&self.context, &r.saved);
        let mut out = vec![];
        for t in &self.targets {
            for (label, loaded) in [
                (
                    "Saved configuration",
                    saved.as_ref().map_err(ToString::to_string),
                ),
                ("Active lookup policy", Ok(&r.loaded)),
            ] {
                let result = loaded.and_then(|l| route(&self.context, l, &r.saved, &t.point.0));
                let okay = result
                    .as_ref()
                    .is_ok_and(|(root, e)| self.entry_correct(t, root, e, r));
                out.push((okay,format!("{} · {label}: {}",t.point.0,match result{Err(e)=>e,Ok(_)if !okay=>format!("needs the intended original filesystem, {} access, {} map and {}-second timeout; saving a direct/master edit does not refresh it",if t.read_only{"read-only"}else{"read/write"},if t.style==maps::Style::Direct{"direct"}else{"indirect"},t.timeout),_=>"selects the required filesystem and policy".into()})));
            }
            let key = lvm::mounts::key(t.original_volume);
            let actual = r.mounted.storage.live().get(&t.point.0);
            let path = format!("{}{}", t.point.0, t.file.0);
            let expected = self
                .world
                .identity
                .as_ref()
                .unwrap()
                .volumes
                .iter()
                .find(|(_, id)| **id == t.original_volume)
                .and_then(|(n, _)| self.world.read_file(n, &t.file.0).ok());
            let current = r
                .mounted
                .read_path(&self.context.inventory, &self.world, &path);
            let good = actual.is_some_and(|m| m.volume == key && m.read_only == t.read_only)
                && current.as_ref().ok().copied() == expected;
            out.push((good,format!("{path} · Current mount: {}",if good{"reads its intended original data with the required access"}else if actual.is_none(){"not mounted; read this file to trigger a lookup"}else{"uses another filesystem or access policy; saved edits do not replace a current mount. Inspect it, run the nonbusy idle check, then retry the file"})));
            let mut future = core::State {
                version: 1,
                saved: r.saved.clone(),
                loaded: r.loaded.clone(),
                mounted: lvm::mounts::Mounted::new(
                    &self.context.inventory,
                    &self.world,
                    String::new(),
                    BTreeMap::new(),
                )
                .unwrap(),
                clock: 0,
                leases: vec![],
            };
            let fresh = future.read(&self.context, &self.world, &path);
            let good = fresh.as_ref().is_ok_and(|v| {
                v.original_volume == t.original_volume && Some(v.content.as_str()) == expected
            }) && future
                .mounted
                .storage
                .live()
                .get(&t.point.0)
                .is_some_and(|m| m.read_only == t.read_only);
            out.push((good,format!("{path} · New lookup after expiry: {}",match fresh{Err(e)=>e.to_string(),_ if !good=>"would still choose different data or access options; fix the policy for future lookups too".into(),_=>"returns the intended original data and access options".into()})));
        }
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Save {
        file: core::Name,
        text: core::Text,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Idle {
        seconds: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub status: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
}
impl Outcome {
    pub fn display(&self) -> String {
        if self.status == 0 {
            if self.output.is_empty() {
                "[Succeeded · exit status 0]".into()
            } else {
                self.output.clone()
            }
        } else {
            format!("[Failed · exit status {}]\n{}", self.status, self.output)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub action: String,
    pub result: Outcome,
}
#[derive(Debug)]
enum Command<'a> {
    Read(&'a str),
    Mounts(Option<&'a str>),
    Reload,
    Help,
}
fn command<'a>(a: &Activity, text: &'a str) -> Result<Command<'a>> {
    if text.len() > 2048
        || !text.is_ascii()
        || text.bytes().any(|b| b.is_ascii_control() && b != b'\t')
    {
        return Err("Use one supported simulated command of at most 2048 bytes".into());
    }
    let words = text.split_ascii_whitespace().collect::<Vec<_>>();
    let provided = |p: &str| a.context.inventory.mountpoints.iter().any(|v| v.0 == p);
    match words.as_slice() {
        ["help"] => Ok(Command::Help),
        ["systemctl", "reload", "autofs"] => Ok(Command::Reload),
        ["findmnt", "-t", "ext4"] => Ok(Command::Mounts(None)),
        ["findmnt", "-t", "ext4", p] if provided(p) => Ok(Command::Mounts(Some(p))),
        ["cat", p]
            if *p == "/etc/auto.master"
                || a.context.map_files.iter().any(|f| f.0 == *p)
                || (maps::absolute(p)
                    && a.context
                        .inventory
                        .mountpoints
                        .iter()
                        .any(|m| p.strip_prefix(&m.0).is_some_and(|v| v.starts_with('/')))) =>
        {
            Ok(Command::Read(p))
        }
        _ => Err(format!("Unsupported command or path. {HELP}")),
    }
}
impl Operation {
    pub fn label(&self) -> String {
        match self {
            Self::Command { text } => text.clone(),
            Self::Save { file, .. } => format!("Save {}", file.0),
            Self::Idle { seconds } => format!("Simulator idle check · {seconds}s"),
        }
    }
    fn shape(&self, a: &Activity) -> Result<()> {
        match self {
            Self::Save { file, text } => {
                if file.0 != "/etc/auto.master" && !a.context.map_files.contains(file) {
                    return Err("Choose one supplied virtual configuration file".into());
                }
                maps::text(&text.0).map_err(|e| e.to_string())
            }
            Self::Command { text } => command(a, text).map(|_| ()),
            Self::Idle { seconds } if (1..=3600).contains(seconds) => Ok(()),
            Self::Idle { .. } => Err("Idle checks need 1–3600 simulated seconds".into()),
        }
    }
    fn apply(&self, a: &Activity, r: &mut core::State) -> Result<Outcome> {
        self.shape(a)?;
        let result: std::result::Result<String, maps::Error> = match self {
            Self::Save { file, text } => r
                .save(&a.context, &a.world, &file.0, text.0.clone())
                .map(|_| "Saved virtual file. Existing backing mounts are unchanged.".into()),
            Self::Idle { seconds } => r.advance(&a.context, &a.world, *seconds).map(|v| {
                if v.is_empty() {
                    "Idle check complete · no backing mounts expired".into()
                } else {
                    format!("Expired backing mounts: {}", v.join(", "))
                }
            }),
            Self::Command { text } => match command(a, text)? {
                Command::Help => Ok(HELP.into()),
                Command::Reload => r.reload(&a.context, &a.world).map(|_| String::new()),
                Command::Read("/etc/auto.master") => Ok(r.saved.master.0.clone()),
                Command::Read(p) if a.context.map_files.iter().any(|f| f.0 == p) => Ok(r
                    .saved
                    .maps
                    .iter()
                    .find(|f| f.name.0 == p)
                    .unwrap()
                    .text
                    .0
                    .clone()),
                Command::Read(p) => r.read(&a.context, &a.world, p).map(|v| v.content),
                Command::Mounts(target) => {
                    let mut rows = vec!["TARGET SOURCE FSTYPE OPTIONS".into()];
                    for (p, m) in r.mounted.storage.live() {
                        if target.is_none_or(|t| t == p) {
                            rows.push(format!(
                                "{} {} ext4 {}",
                                p,
                                r.mounted.storage.inventory().device_for(&m.volume).unwrap(),
                                if m.read_only { "ro" } else { "rw" }
                            ));
                        }
                    }
                    if rows.len() == 1 && target.is_some() {
                        return Ok(Outcome{status:1,output:"No ext4 backing mount at that provided point. Reading a file there triggers a lookup.".into()});
                    }
                    Ok(rows.join("\n"))
                }
            },
        };
        r.validate(&a.context, &a.world)
            .map_err(|e| e.to_string())?;
        Ok(match result {
            Ok(output) => Outcome { status: 0, output },
            Err(e) => Outcome {
                status: 1,
                output: format!(
                    "{}: {e}",
                    match e.kind {
                        maps::ErrorKind::Syntax => "Map syntax error",
                        maps::ErrorKind::Unsupported => "Unsupported map or operation",
                        maps::ErrorKind::Limit => "Simulation limit",
                        maps::ErrorKind::Context => "Invalid supplied state",
                        maps::ErrorKind::Lookup => "Lookup failed",
                    }
                ),
            },
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: Operation,
    #[serde(deserialize_with = "crate::cron::text::<_,640>")]
    pub explanation: String,
    pub status: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Activity,
    pub example: Activity,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,20>")]
    pub reference: Vec<Step>,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,16>")]
    pub example_steps: Vec<Step>,
    pub experiment: Step,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,8>")]
    pub repair: Vec<Step>,
}
fn id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !id(&self.source_id)
            || !bounded::within(self, 384 * 1024)
            || self.reference.is_empty()
            || self.reference.len() > 20
            || self.example_steps.is_empty()
            || self.example_steps.len() > 16
            || self.repair.is_empty()
            || self.repair.len() > 8
        {
            return Err("Unknown or oversized AutoFS teaching definition".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self
            .assignment
            .world
            .devices
            .keys()
            .any(|k| self.example.world.devices.contains_key(k))
            || self
                .assignment
                .world
                .groups
                .keys()
                .any(|k| self.example.world.groups.contains_key(k))
        {
            return Err("AutoFS example needs its own separate virtual storage".into());
        }
        for (steps, a) in [
            (&self.reference, &self.assignment),
            (&self.example_steps, &self.example),
            (&self.repair, &self.example),
        ] {
            for s in steps {
                if s.status > 1 || !prose(&s.explanation, 640) {
                    return Err(
                        "Every teaching action needs a short explanation and expected result"
                            .into(),
                    );
                }
                s.operation.shape(a)?;
            }
        }
        self.experiment.operation.shape(&self.example)?;
        if self.experiment.status > 1 || !prose(&self.experiment.explanation, 640) {
            return Err("Provide one meaningful practice change with an explanation".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        if s.version != 1
            || s.origin != self.source_id
            || s.example_step >= self.example_steps.len()
            || !bounded::within(s, MAX_SESSION)
        {
            return Err(
                "AutoFS work has incompatible source/version/selection/size; prior data retained"
                    .into(),
            );
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.assignment)
    }
    pub fn validate_reachability(&self) -> Result<()> {
        self.validate()?;
        for (practice, steps) in [(false, &self.reference), (true, &self.example_steps)] {
            let a = if practice {
                &self.example
            } else {
                &self.assignment
            };
            let mut w = Work::new(a)?;
            if a.checks(&w.runtime).iter().all(|r| r.0) {
                return Err("AutoFS initial work must require a real repair".into());
            }
            for s in steps {
                let out = w.apply(a, &s.operation)?;
                if out.status != s.status {
                    return Err(format!(
                        "{}: expected{}, actual{}: {}",
                        s.operation.label(),
                        s.status,
                        out.status,
                        out.output
                    ));
                }
            }
            w.checked = true;
            if w.checks(a).iter().any(|r| !r.0) {
                return Err(format!(
                    "AutoFS reference fails meaningful outcomes: {:?}",
                    w.checks(a)
                ));
            }
            if practice {
                let out = w.apply(a, &self.experiment.operation)?;
                if out.status != self.experiment.status {
                    return Err("Practice change has an unexpected result".into());
                }
                if a.checks(&w.runtime).iter().all(|r| r.0) {
                    return Err("Practice change must affect an assessed outcome".into());
                }
                for s in &self.repair {
                    if w.apply(a, &s.operation)?.status != s.status {
                        return Err("Practice retry did not run as documented".into());
                    }
                }
                w.checked = true;
                if w.checks(a).iter().any(|r| !r.0) {
                    return Err("Practice retry does not repair the outcome".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub runtime: core::State,
    pub drafts: core::Configuration,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,64>")]
    pub history: Vec<Operation>,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,32>")]
    pub transcript: Vec<Row>,
    pub selected: core::Name,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command: String,
    pub checked: bool,
}
fn record(rows: &mut Vec<Row>, op: &Operation, result: Outcome) {
    if rows.len() == 32 {
        rows.remove(0);
    }
    rows.push(Row {
        action: op.label(),
        result,
    });
}
impl Work {
    pub fn new(a: &Activity) -> Result<Self> {
        Ok(Self {
            runtime: a.initial()?,
            drafts: a.saved.clone(),
            history: vec![],
            transcript: vec![],
            selected: core::Name("/etc/auto.master".into()),
            command: String::new(),
            checked: false,
        })
    }
    pub fn bound(&self, a: &Activity) -> Result<()> {
        if !bounded::within(self, 512 * 1024)
            || self.history.len() > MAX_HISTORY
            || self.transcript.len() > 32
            || self.command.len() > 2048
            || self
                .command
                .bytes()
                .any(|b| b.is_ascii_control() && b != b'\t')
            || self.selected.0 != "/etc/auto.master"
                && !a.context.map_files.contains(&self.selected)
        {
            return Err(
                "Saved AutoFS work exceeds its fields/history limits or supplied selection".into(),
            );
        }
        self.drafts
            .validate(&a.context)
            .map_err(|e| e.to_string())?;
        self.runtime
            .validate(&a.context, &a.world)
            .map_err(|e| e.to_string())?;
        let mut actual = a.initial()?;
        let mut rows = vec![];
        for op in &self.history {
            let result = op.apply(a, &mut actual)?;
            record(&mut rows, op, result);
        }
        if actual != self.runtime || rows != self.transcript {
            return Err("Saved AutoFS effects or results differ from their supplied activity and actions; prior work retained".into());
        }
        Ok(())
    }
    pub fn draft(&self) -> &str {
        if self.selected.0 == "/etc/auto.master" {
            &self.drafts.master.0
        } else {
            self.drafts
                .maps
                .iter()
                .find(|m| m.name == self.selected)
                .map(|m| m.text.0.as_str())
                .unwrap_or("")
        }
    }
    pub fn edit(&mut self, a: &Activity, text: String) -> Result<()> {
        self.bound(a)?;
        maps::text(&text).map_err(|e| e.to_string())?;
        if self.selected.0 == "/etc/auto.master" {
            self.drafts.master = core::Text(text);
        } else {
            self.drafts
                .maps
                .iter_mut()
                .find(|m| m.name == self.selected)
                .ok_or("No supplied map editor")?
                .text = core::Text(text);
        }
        self.checked = false;
        Ok(())
    }
    pub fn apply(&mut self, a: &Activity, op: &Operation) -> Result<Outcome> {
        self.bound(a)?;
        if self.history.len() == MAX_HISTORY {
            return Err("This attempt reached 64 actions. Save/export your work if needed, then reset this selected attempt to start a new investigation".into());
        }
        let mut next = self.clone();
        let result = op.apply(a, &mut next.runtime)?;
        if let Operation::Save { file, text } = op
            && result.status == 0
        {
            if file.0 == "/etc/auto.master" {
                next.drafts.master = text.clone();
            } else {
                next.drafts
                    .maps
                    .iter_mut()
                    .find(|m| m.name == *file)
                    .ok_or("No supplied map editor")?
                    .text = text.clone();
            }
        }
        next.history.push(op.clone());
        record(&mut next.transcript, op, result.clone());
        next.checked = false;
        next.bound(a)?;
        *self = next;
        Ok(result)
    }
    pub fn checks(&self, a: &Activity) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![(
                false,
                "Not checked · choose Check work to inspect the saved, active and mounted state"
                    .into(),
            )];
        }
        let mut rows=vec![(self.drafts==self.runtime.saved,"Save or revert each edited file before checking. Drafts, saved maps and current mounts are different states".into())];
        rows.extend(a.checks(&self.runtime));
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub example_step: usize,
    pub assignment: Work,
    pub practice: Work,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.source_id.clone(),
            view: View::Introduction,
            example_step: 0,
            assignment: Work::new(&d.assignment)?,
            practice: Work::new(&d.example)?,
        })
    }
    pub fn shape(&self) -> bool {
        self.version == 1
            && id(&self.origin)
            && self.example_step < 16
            && bounded::within(self, MAX_SESSION)
    }
    pub fn work(&self, practice: bool) -> &Work {
        if practice {
            &self.practice
        } else {
            &self.assignment
        }
    }
    pub fn work_mut(&mut self, practice: bool) -> &mut Work {
        if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        }
    }
    pub fn activity(d: &Definition, practice: bool) -> &Activity {
        if practice { &d.example } else { &d.assignment }
    }
    pub fn apply(&mut self, d: &Definition, op: &Operation, practice: bool) -> Result<Outcome> {
        d.bound(self)?;
        let out = self
            .work_mut(practice)
            .apply(Self::activity(d, practice), op)?;
        d.bound(self)?;
        Ok(out)
    }
    pub fn run(&mut self, d: &Definition, input: &str, practice: bool) -> Result<String> {
        d.bound(self)?;
        if input == CHECK {
            self.work_mut(practice).checked = true;
            return Ok(if self
                .work(practice)
                .checks(Self::activity(d, practice))
                .iter()
                .all(|r| r.0)
            {
                "Current work meets every requirement"
            } else {
                "Work needs changes; inspect the objective results and retry"
            }
            .into());
        }
        self.apply(d, &Operation::Command { text: input.into() }, practice)
            .map(|v| v.display())
    }
    pub fn edit(&mut self, d: &Definition, text: String, practice: bool) -> Result<()> {
        d.bound(self)?;
        self.work_mut(practice)
            .edit(Self::activity(d, practice), text)
    }
    pub fn save(&mut self, d: &Definition, practice: bool) -> Result<Outcome> {
        d.bound(self)?;
        let w = self.work(practice);
        let op = Operation::Save {
            file: w.selected.clone(),
            text: core::Text(w.draft().into()),
        };
        self.apply(d, &op, practice)
    }
}
