//! Source-bound, separate DKMS investigation workspaces. All values stay in memory.
use crate::{dkms as core, script_teaching::View};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
pub const MAX_TASK_BYTES: usize = 32768;

fn identity(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}
fn text(s: &str, cap: usize) -> bool {
    s.len() <= cap
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t' | '\r'))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Required {
    Built,
    Installed,
    NotInstalled,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub source: usize,
    pub kernel: usize,
    pub state: Required,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    CommandDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Run {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        input: String,
    },
    Check,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    pub facts: core::Facts,
    pub initial: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,8>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,usize,4>")]
    pub preserve_kernels: Vec<usize>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,16>")]
    pub reference: Vec<Action>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
    pub ok: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub state: core::State,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command_draft: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Line,96>")]
    pub log: Vec<Line>,
    pub checked: bool,
}
impl Work {
    pub fn new(t: &Task) -> Self {
        Self {
            state: t.initial.clone(),
            command_draft: String::new(),
            log: vec![],
            checked: false,
        }
    }
}
impl Task {
    pub fn validate_work(&self, w: &Work) -> Result<()> {
        w.state.validate(&self.facts)?;
        if !text(&w.command_draft, 2048)
            || w.log.len() > 96
            || w.log
                .iter()
                .any(|l| !text(&l.input, 2048) || !text(&l.output, 8192))
        {
            return Err("Keep the command draft and actual console history within their bounded plain-text limits.".into());
        }
        Ok(())
    }
    pub fn apply(&self, w: &mut Work, a: &Action) -> Result<String> {
        self.validate_work(w)?;
        match a {
            Action::CommandDraft { text: value } => {
                if !text(value, 2048) {
                    return Err("Keep one command draft within 2048 text bytes.".into());
                }
                w.command_draft = value.clone();
                Ok("Draft retained; it has not run.".into())
            }
            Action::Run { input } => {
                if !text(input, 2048) {
                    return Err("Keep one console operation within 2048 text bytes.".into());
                }
                let before = w.state.clone();
                let result = core::execute(&self.facts, &mut w.state, input);
                if w.state != before {
                    w.checked = false;
                }
                let (ok, output) = match &result {
                    Ok(out) => (true, out.clone()),
                    Err(e) => (false, e.clone()),
                };
                if output.len() > 8192 {
                    return Err(
                        "The bounded operation produced an unexpected oversized diagnostic.".into(),
                    );
                }
                if w.log.len() == 96 {
                    w.log.remove(0);
                }
                w.log.push(Line {
                    input: input.clone(),
                    output,
                    ok,
                });
                result
            }
            Action::Check => {
                w.checked = true;
                Ok("Checked the actual per-kernel registry and preserved instances.".into())
            }
            Action::Reset => {
                *w = Work::new(self);
                Ok("Reset this activity only; its other workspace remains unchanged.".into())
            }
        }
    }
    pub fn checks(&self, w: &Work) -> Vec<Check> {
        if self
            .requirements
            .iter()
            .any(|r| r.source >= self.facts.sources.len() || r.kernel >= self.facts.kernels.len())
            || self
                .preserve_kernels
                .iter()
                .any(|k| *k >= self.facts.kernels.len())
        {
            return vec![Check {
                passed: false,
                label: "Authored requirements".into(),
                detail: "A required instance refers to unavailable source/kernel facts.".into(),
            }];
        }
        if let Err(e) = self.validate_work(w) {
            return vec![Check {
                passed: false,
                label: "Saved workspace".into(),
                detail: e,
            }];
        }
        let mut checks=self.requirements.iter().map(|r|{
            let source=&self.facts.sources[r.source];let release=&self.facts.kernels[r.kernel].release;
            let record=w.state.records.iter().find(|p|p.source==r.source&&p.kernel==r.kernel);
            let actual=if record.is_some_and(|p|p.installed){"installed"}else if record.is_some(){"built, not installed"}else if w.state.registered.contains(&r.source){"added, not built for this kernel"}else{"not registered"};
            let passed=match r.state {Required::Built=>record.is_some(),Required::Installed=>record.is_some_and(|p|p.installed),Required::NotInstalled=>!record.is_some_and(|p|p.installed)};
            let wanted=match r.state {Required::Built=>"built (installation is also acceptable)",Required::Installed=>"installed",Required::NotInstalled=>"not installed"};
            let detail=if passed {format!("Actual: {actual}; required: {wanted}. Installed is not a loaded-driver test.")}
                else if matches!(r.state,Required::Installed)&&record.is_some(){format!("Actual: {actual}. A successful build has not placed this version in the selected kernel’s module tree. Install it for {release}, then check again.")}
                else {format!("Actual: {actual}; required: {wanted}. Inspect dkms status for this exact module/version and kernel; omitting -k selects the supplied running kernel.")};
            Check{passed,label:format!("{}/{} · {}",source.name,source.version,release),detail}
        }).collect::<Vec<_>>();
        for &kernel in &self.preserve_kernels {
            let original = self
                .initial
                .records
                .iter()
                .filter(|r| r.kernel == kernel)
                .collect::<Vec<_>>();
            let current = w
                .state
                .records
                .iter()
                .filter(|r| r.kernel == kernel)
                .collect::<Vec<_>>();
            let equal = original.iter().all(|r| current.contains(r))
                && current
                    .iter()
                    .filter(|r| r.installed)
                    .all(|r| original.contains(r));
            checks.push(Check{passed:equal,label:format!("Preserve {}",self.facts.kernels[kernel].release),detail:if equal {"All supplied records and installed versions for this kernel remain unchanged; extra uninstalled builds are harmless.".into()}else{"A protected kernel’s supplied record or installed version changed. Restore its supplied instances; target the maintenance kernel with -k.".into()}});
        }
        checks
    }
    pub fn solved(&self, w: &Work) -> bool {
        w.checked && self.checks(w).iter().all(|c| c.passed)
    }
    pub fn validate(&self) -> Result<()> {
        self.facts.validate()?;
        self.initial.validate(&self.facts)?;
        if !identity(&self.id)
            || self.instructions.trim().is_empty()
            || !text(&self.instructions, 1600)
            || self.requirements.is_empty()
            || self.requirements.len() > 8
            || self.preserve_kernels.len() > 4
            || self.reference.is_empty()
            || self.reference.len() > 16
            || !matches!(self.reference.last(), Some(Action::Check))
        {
            return Err("Supply a named bounded DKMS task, actual requirements and a supported reference ending in Check.".into());
        }
        let mut pairs = std::collections::BTreeSet::new();
        for r in &self.requirements {
            if r.source >= self.facts.sources.len()
                || r.kernel >= self.facts.kernels.len()
                || !pairs.insert((r.source, r.kernel))
            {
                return Err("Use distinct supplied source/kernel requirements; never duplicate or contradict a pair.".into());
            }
        }
        let mut kept = std::collections::BTreeSet::new();
        for &k in &self.preserve_kernels {
            if k >= self.facts.kernels.len()
                || !kept.insert(k)
                || !self
                    .initial
                    .records
                    .iter()
                    .any(|r| r.kernel == k && r.installed)
            {
                return Err(
                    "A protected kernel needs distinct supplied installed records to preserve."
                        .into(),
                );
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_TASK_BYTES {
            return Err("DKMS task exceeds its persisted source limit.".into());
        }
        let mut w = Work::new(self);
        w.checked = true;
        if self.solved(&w) {
            return Err("The starter must leave meaningful required work unfinished.".into());
        }
        w.checked = false;
        for a in &self.reference {
            if matches!(a, Action::Reset | Action::CommandDraft { .. }) {
                return Err("A worked reference contains supported operations and a final check, not draft/reset actions.".into());
            }
            self.apply(&mut w, a)?;
        }
        if !self.solved(&w) {
            return Err("The authored reference does not reach every actual DKMS outcome.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.source_id != self.assignment.id
            || !identity(&self.source_id)
            || self.assignment.id == self.example.id
            || self.assignment.facts == self.example.facts
        {
            return Err("Keep a versioned assignment and a separate example with different supplied sources/kernels.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub source_task: String,
    pub work: Work,
}
fn canonical(t: &Task) -> Result<String> {
    let s = serde_json::to_string(t).map_err(|e| e.to_string())?;
    if s.len() > MAX_TASK_BYTES {
        return Err("DKMS task exceeds its persisted source limit.".into());
    }
    Ok(s)
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            source_task: canonical(t)?,
            work: Work::new(t),
        })
    }
    pub fn bind(&self, t: &Task) -> Result<()> {
        if self.source_task != canonical(t)? {
            return Err(
                "Saved DKMS facts or requirements differ from this lesson; keep the original work."
                    .into(),
            );
        }
        t.validate_work(&self.work)
    }
    pub fn validate_shape(&self) -> Result<()> {
        if self.source_task.len() > MAX_TASK_BYTES {
            return Err("Saved DKMS source exceeds its limit.".into());
        }
        let t: Task = serde_json::from_str(&self.source_task)
            .map_err(|e| format!("Malformed saved DKMS task: {e}"))?;
        t.validate()?;
        self.bind(&t)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            example: Scope::new(&d.example)?,
        })
    }
    pub fn bind(&self, d: &Definition) -> Result<()> {
        d.validate()?;
        if self.version != 1 || self.source_id != d.source_id {
            return Err(
                "Saved DKMS session belongs to a different lesson/version; keep the original work."
                    .into(),
            );
        }
        self.assignment.bind(&d.assignment)?;
        self.example.bind(&d.example)
    }
    pub fn validate_shape(&self) -> Result<()> {
        if self.version != 1 || !identity(&self.source_id) {
            return Err("Use a known named DKMS session version.".into());
        }
        self.assignment.validate_shape()?;
        self.example.validate_shape()?;
        let a: Task =
            serde_json::from_str(&self.assignment.source_task).map_err(|e| e.to_string())?;
        let e: Task = serde_json::from_str(&self.example.source_task).map_err(|e| e.to_string())?;
        self.bind(&Definition {
            version: 1,
            source_id: self.source_id.clone(),
            assignment: a,
            example: e,
        })
    }
    pub fn apply(&mut self, d: &Definition, practice: bool, a: &Action) -> Result<String> {
        self.bind(d)?;
        let (t, w) = if practice {
            (&d.example, &mut self.example.work)
        } else {
            (&d.assignment, &mut self.assignment.work)
        };
        t.apply(w, a)
    }
}
