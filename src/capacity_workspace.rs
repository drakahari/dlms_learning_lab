//! Native capacity lessons; supplied virtual accounting only, never host I/O.
use crate::{fs_capacity as core, pack::Scenario, progress::Attempt, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const HELP: &str = "Virtual capacity model: a file uses one inode and rounds its logical byte length up to whole supplied blocks. Zero-length files use an inode but no data blocks. Fixed allocation and reserved blocks are supplied facts, not measured ext4 overhead. Ordinary files cannot spend the reserve. Flat virtual /names only: no real files, contents, directories, sparse allocation, links, mounts, quotas, df, fsck or disk repair. The native file-size editor changes logical size; it does not execute truncate or a shell. Preview removal never removes data; Apply removal refuses a preview if any intervening write changed the snapshot.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Headroom {
        blocks: u64,
        inodes: u64,
    },
    File {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        path: String,
        bytes: u64,
    },
}
impl Requirement {
    fn check(&self, w: &core::State) -> (bool, String) {
        match self {
            Self::Headroom { blocks, inodes } => match w.usage() {
                Ok(u) => (
                    u.available_blocks >= *blocks && u.free_inodes >= *inodes,
                    format!(
                        "Leave at least {blocks} ordinary-available blocks and {inodes} free inodes. Current: {} blocks, {} inodes. Shrinking can release blocks; only removing file names releases their inodes.",
                        u.available_blocks, u.free_inodes
                    ),
                ),
                Err(e) => (false, e.to_string()),
            },
            Self::File { path, bytes } => {
                let actual = w.files().ok().and_then(|f| f.get(path)).copied();
                (
                    actual == Some(*bytes),
                    format!(
                        "{path} must exist with {bytes} logical bytes. Current: {}. A new file needs both an inode and enough ordinary-available blocks.",
                        actual
                            .map(|n| format!("{n} bytes"))
                            .unwrap_or_else(|| "missing".into())
                    ),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Put {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        path: String,
        bytes: u64,
    },
    Remove {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,128>")]
        names: Vec<String>,
    },
}
fn apply(w: &mut core::State, a: &Action) -> Result<(), String> {
    match a {
        Action::Put { path, bytes } => w.put_file(path, *bytes),
        Action::Remove { names } => {
            let ns: Vec<_> = names.iter().map(String::as_str).collect();
            let plan = w.preview_remove(&ns).map_err(|e| e.to_string())?;
            w.apply_remove(&plan)
        }
    }
    .map_err(|e| e.to_string())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: core::State,
    pub example: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,12>")]
    pub example_requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub example_actions: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.initial == self.example
            || self.requirements.is_empty()
            || self.example_requirements.is_empty()
            || self.example_actions.is_empty()
            || self.requirements.len() > 12
            || self.example_requirements.len() > 12
            || self.example_actions.len() > 24
        {
            return Err(
                "Capacity lesson needs distinct sample inputs, outcomes and a worked example"
                    .into(),
            );
        }
        for (w, rs) in [
            (&self.initial, &self.requirements),
            (&self.example, &self.example_requirements),
        ] {
            w.validate().map_err(|e| e.to_string())?;
            if w.generation().map_err(|e| e.to_string())? != 0 {
                return Err("Capacity supplied files must start at generation zero".into());
            }
            let p = w.profile().map_err(|e| e.to_string())?;
            for r in rs {
                match r {
                    Requirement::Headroom { blocks, inodes }
                        if *blocks > p.total_blocks || *inodes > p.total_inodes =>
                    {
                        return Err("Impossible capacity target".into());
                    }
                    Requirement::File { path, bytes }
                        if *bytes > core::MAX_LOGICAL_BYTES || !valid_path(path) =>
                    {
                        return Err("Invalid required virtual file".into());
                    }
                    _ => {}
                }
            }
        }
        let mut w = self.example.clone();
        for a in &self.example_actions {
            apply(&mut w, a)?;
        }
        if !self.example_requirements.iter().all(|r| r.check(&w).0) {
            return Err("Worked capacity example does not meet its outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        if s.example_action >= self.example_actions.len() {
            return Err("Unknown worked action".into());
        }
        for (scope, initial) in [(&s.assignment, &self.initial), (&s.practice, &self.example)] {
            if !scope.world.same_environment(initial)
                || scope
                    .preview
                    .as_ref()
                    .is_some_and(|p| !p.before.same_environment(initial))
            {
                return Err("Capacity work differs from its supplied filesystem facts; previous work retained".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (scope, rs) = if practice {
            (&s.practice, &self.example_requirements)
        } else {
            (&s.assignment, &self.requirements)
        };
        let mut rows=vec![(!scope.dirty,"Apply or discard the edited file-size draft before checking. Unapplied changes are not proof of the current file state.".into())];
        rows.extend(rs.iter().map(|r| r.check(&scope.world)));
        rows
    }
}
fn valid_path(p: &str) -> bool {
    p.len() <= 48
        && p.strip_prefix('/').is_some_and(|s| {
            !s.is_empty()
                && !matches!(s, "." | "..")
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub before: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,128>")]
    pub names: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,128>")]
    pub selected: Vec<String>,
    pub preview: Option<Preview>,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub bytes: String,
    pub dirty: bool,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub message: String,
}
impl Scope {
    pub fn new(w: &core::State) -> Self {
        Self {
            world: w.clone(),
            selected: vec![],
            preview: None,
            path: String::new(),
            bytes: String::new(),
            dirty: false,
            checked: false,
            message: String::new(),
        }
    }
    pub fn put(&mut self) -> Result<(), String> {
        let bytes = if self.bytes.is_empty() {
            return Err("Enter a logical file length in bytes, using digits only.".into());
        } else if !self.bytes.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Use a whole number of bytes, without units or decimal points.".into());
        } else {
            self.bytes
                .parse()
                .map_err(|_| "File length exceeds the supported whole-number range")?
        };
        self.world.put_file(&self.path,bytes).map_err(|e|match e{
            core::Error::NoInodes=>"No free inode: a new file needs its own inode, even when it is empty. Remove an expendable file rather than merely shrinking it.".into(),
            core::Error::NoBlocks=>"Not enough ordinary-available blocks. File lengths round up to whole blocks; reserved blocks cannot be spent by this operation.".into(),
            core::Error::Protected=>"This file must be retained unchanged. Choose an expendable file instead.".into(),
            _=>format!("{e}. Use one flat virtual path, for example /trial, and a bounded length.")})?;
        self.dirty = false;
        self.checked = false;
        self.message = "Applied logical length. Inspect blocks and inodes before checking.".into();
        Ok(())
    }
    pub fn preview_remove(&mut self) -> Result<(), String> {
        let names: Vec<_> = self.selected.iter().map(String::as_str).collect();
        self.world
            .preview_remove(&names)
            .map_err(|e| format!("{e}. Select one or more existing expendable files."))?;
        self.preview = Some(Preview {
            before: self.world.clone(),
            names: self.selected.clone(),
        });
        self.message = "Preview only: no files have been removed.".into();
        Ok(())
    }
    pub fn apply_remove(&mut self) -> Result<(), String> {
        let p = self.preview.as_ref().ok_or("Preview a removal first")?;
        let names: Vec<_> = p.names.iter().map(String::as_str).collect();
        let plan = p.before.preview_remove(&names).map_err(|e| e.to_string())?;
        self.world.apply_remove(&plan).map_err(|e|if e==core::Error::Stale{"The files changed after this preview. Review current usage and preview the removal again; nothing was removed.".into()}else{e.to_string()})?;
        self.preview = None;
        self.selected.clear();
        self.checked = false;
        self.message = "Removed only the previewed virtual files.".into();
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub example_action: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            example_action: 0,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.example_action >= 24
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 192 * 1024
        {
            return Err("Capacity workspace exceeds its version or size limit".into());
        }
        for s in [&self.assignment, &self.practice] {
            s.world.validate().map_err(|e| e.to_string())?;
            if s.path.len() > 48
                || s.bytes.len() > 24
                || s.message.len() > 1024
                || s.selected.len() > 128
                || s.selected.iter().collect::<BTreeSet<_>>().len() != s.selected.len()
                || s.selected.iter().any(|p| !valid_path(p))
            {
                return Err("Invalid capacity editor/selection".into());
            }
            if let Some(p) = &s.preview {
                let names: Vec<_> = p.names.iter().map(String::as_str).collect();
                p.before.preview_remove(&names).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, text: &str) -> Result<String, String> {
        d.bound(self)?;
        let raw = text
            .strip_prefix("capacity-action ")
            .ok_or("Use the native file table and editor. No shell or disk utility runs here.")?;
        if raw.len() > 8192 {
            return Err("Capacity reference exceeds limit".into());
        }
        let action: Action =
            serde_json::from_str(raw).map_err(|_| "Invalid simulator-only capacity reference")?;
        let mut next = self.clone();
        apply(&mut next.assignment.world, &action)?;
        next.assignment.checked = false;
        d.bound(&next)?;
        *self = next;
        Ok("Applied simulated capacity operation".into())
    }
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::Capacity { simulation },
        crate::engine::Goal::Capacity { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Capacity task and state must agree".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err("Capacity task must start from its separate supplied sample inputs".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::Capacity { simulation } = &a.state else {
        return Err("Saved work belongs to a different capacity task".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible capacity attempt metadata; previous work retained".into());
    }
    d.bound(simulation)
}
