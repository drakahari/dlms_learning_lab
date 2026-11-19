//! Source-bound behavioral cases and cooperative checking over virtual graphs.
use crate::{
    fs_integrity,
    fs_integrity_terminal::{self as terminal, System},
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
pub const MAX_TASK_BYTES: usize = 131072;
pub const MAX_CASE_BYTES: usize = 32768;
pub const MAX_HISTORY: usize = 64;
pub const MAX_HISTORY_BYTES: usize = 32768;
pub const MAX_TICKS: usize = 3 * (MAX_HISTORY + 2);
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn commands<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,2048>")] String);
    fs_integrity::sequence::<D, Text, MAX_HISTORY>(d).map(|v| v.into_iter().map(|s| s.0).collect())
}
fn history_valid(history: &[String]) -> bool {
    history.len() <= MAX_HISTORY
        && crate::cron_workspace::within(history, MAX_HISTORY_BYTES)
        && history.iter().all(|s| terminal::parse(s).is_ok())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Healthy {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        device: String,
    },
    Mounted {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        device: String,
    },
    Unchanged {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        device: String,
    },
    Preserved {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        device: String,
        #[serde(deserialize_with = "crate::cron::text::<_,80>")]
        name: String,
    },
    Recovered {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        device: String,
        inode: u32,
    },
}
impl Requirement {
    fn device(&self) -> &str {
        match self {
            Self::Healthy { device }
            | Self::Mounted { device }
            | Self::Unchanged { device }
            | Self::Preserved { device, .. }
            | Self::Recovered { device, .. } => device,
        }
    }
    fn check(&self, initial: &System, current: &System) -> (bool, String) {
        let old = initial.disks.iter().find(|d| d.device == self.device());
        let got = current.disks.iter().find(|d| d.device == self.device());
        let (Some(old), Some(got)) = (old, got) else {
            return (
                false,
                "Keep the supplied virtual volume identities; no replacement device is accepted."
                    .into(),
            );
        };
        match self {
            Self::Healthy { device } => match got.volume.findings() {
                Err(e) => (
                    false,
                    format!("{device}: unsupported or invalid metadata: {e}"),
                ),
                Ok(findings) => (
                    findings.is_empty(),
                    if let Some(first) = findings.first() {
                        format!(
                            "{device}: {} consistency finding(s) remain. {}",
                            findings.len(),
                            terminal::finding_text(first)
                        )
                    } else {
                        format!("{device}: file references, allocation maps and counts agree.")
                    },
                ),
            },
            Self::Mounted { device } => (
                got.volume.mounted,
                format!(
                    "{device}: {}. Finish by restoring the supplied mount after checking the repaired graph.",
                    if got.volume.mounted {
                        "mounted"
                    } else {
                        "still unmounted"
                    }
                ),
            ),
            Self::Unchanged { device } => (
                old == got,
                format!(
                    "{device}: preserve the separate volume, its mount and all supplied data; it is outside this repair task."
                ),
            ),
            Self::Preserved { device, name } => {
                let before = old
                    .volume
                    .entries
                    .get(name)
                    .and_then(|id| old.volume.files.get(id).map(|file| (*id, file)));
                let after = got
                    .volume
                    .entries
                    .get(name)
                    .and_then(|id| got.volume.files.get(id).map(|file| (*id, file)));
                let pass = before.zip(after).is_some_and(|((id, a), (other, b))| {
                    id == other && a.content == b.content && a.blocks == b.blocks
                });
                (
                    pass,
                    format!(
                        "{device}/{name}: retain the original inode, file contents and data-block references. Repair accounting instead of discarding data."
                    ),
                )
            }
            Self::Recovered { device, inode } => {
                let preserved = old
                    .volume
                    .files
                    .get(inode)
                    .zip(got.volume.files.get(inode))
                    .is_some_and(|(a, b)| a.content == b.content && a.blocks == b.blocks);
                let named = got
                    .volume
                    .entries
                    .iter()
                    .any(|(n, i)| *i == *inode && n.starts_with("lost+found/"));
                (
                    preserved && named,
                    format!(
                        "{device}: inode {inode} must retain its bytes and gain a directory name inside lost+found. A nonempty unattached inode requires an explicit recovery decision; preen alone leaves it unresolved."
                    ),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub initial: System,
    #[serde(deserialize_with = "crate::fs_integrity::sequence::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
impl Case {
    fn checks(&self, current: &System) -> Vec<(bool, String)> {
        self.requirements
            .iter()
            .map(|r| r.check(&self.initial, current))
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::fs_integrity::sequence::<_,Case,3>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || !(2..=3).contains(&self.cases.len())
            || !crate::cron_workspace::within(self, MAX_TASK_BYTES)
        {
            return Err("Invalid bounded integrity task".into());
        }
        let mut ids = BTreeSet::new();
        let mut associations = None;
        for case in &self.cases {
            case.initial.validate().map_err(|e| e.to_string())?;
            if !identifier(&case.id)
                || !ids.insert(&case.id)
                || case.label.trim().is_empty()
                || case.label.len() > 120
                || case.label.chars().any(char::is_control)
                || !crate::cron_workspace::within(&case.initial, MAX_CASE_BYTES)
                || case.requirements.is_empty()
                || case.requirements.len() > 12
            {
                return Err("Invalid or oversized integrity sample".into());
            }
            let pairs: BTreeSet<_> = case
                .initial
                .disks
                .iter()
                .map(|d| (d.device.clone(), d.mountpoint.clone()))
                .collect();
            if associations.as_ref().is_some_and(|a| *a != pairs) {
                return Err(
                    "Samples must retain the task's supplied virtual device/target associations"
                        .into(),
                );
            }
            associations = Some(pairs);
            let mut goals = BTreeSet::new();
            for goal in &case.requirements {
                if !goals.insert(serde_json::to_string(goal).map_err(|e| e.to_string())?) {
                    return Err("Duplicate integrity objective".into());
                }
                let disk = case
                    .initial
                    .disks
                    .iter()
                    .find(|d| d.device == goal.device())
                    .ok_or("Integrity objective refers to an unknown device")?;
                match goal {
                    Requirement::Preserved { name, .. } => {
                        if !disk
                            .volume
                            .entries
                            .get(name)
                            .is_some_and(|i| disk.volume.files.contains_key(i))
                        {
                            return Err("Preservation needs a supplied named file".into());
                        }
                    }
                    Requirement::Recovered { inode, .. } => {
                        let file = disk
                            .volume
                            .files
                            .get(inode)
                            .ok_or("Recovery needs supplied retained inode data")?;
                        if file.content.is_empty()
                            || disk.volume.entries.values().any(|i| i == inode)
                        {
                            return Err("Recovery must start with a nonempty unnamed inode".into());
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn run(&self, history: &[String]) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, history)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.results);
            }
        }
        Err("Integrity check reached its bounded phase limit".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub checks: Vec<(bool, String)>,
    pub diagnostic: Option<String>,
    pub actual: Option<System>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.diagnostic.is_none() && !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
pub struct Run {
    task: Task,
    history: Vec<String>,
    case: usize,
    step: usize,
    current: Option<System>,
    results: Vec<CaseResult>,
    cancelled: bool,
}
impl Run {
    pub fn new(task: &Task, history: &[String]) -> Result<Self, String> {
        task.validate()?;
        if !history_valid(history) {
            return Err(
                "Integrity action history exceeds its limits or contains unsupported commands"
                    .into(),
            );
        }
        Ok(Self {
            task: task.clone(),
            history: history.into(),
            case: 0,
            step: 0,
            current: None,
            results: vec![],
            cancelled: false,
        })
    }
    pub fn done(&self) -> bool {
        !self.cancelled && self.case == self.task.cases.len()
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.current = None;
        self.results.clear();
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.results)
    }
    pub fn history(&self) -> &[String] {
        &self.history
    }
    pub fn task(&self) -> &Task {
        &self.task
    }
    pub fn tick(&mut self) -> bool {
        if self.cancelled {
            return false;
        }
        if self.done() {
            return true;
        }
        let sample = &self.task.cases[self.case];
        if self.current.is_none() {
            self.current = Some(sample.initial.clone());
            return false;
        }
        if self.step < self.history.len() {
            let command = &self.history[self.step];
            if let Err(e) = self.current.as_mut().unwrap().execute(command) {
                let reason = format!("{}: {}. No host command was run.", sample.label, e.message);
                self.results.push(CaseResult {
                    id: sample.id.clone(),
                    label: sample.label.clone(),
                    checks: vec![(false, reason.clone())],
                    diagnostic: Some(reason),
                    actual: self.current.take(),
                });
                self.case += 1;
                self.step = 0;
            } else {
                self.step += 1
            }
            return self.done();
        }
        let current = self.current.take().unwrap();
        let checks = sample.checks(&current);
        self.results.push(CaseResult {
            id: sample.id.clone(),
            label: sample.label.clone(),
            checks,
            diagnostic: None,
            actual: Some(current),
        });
        self.case += 1;
        self.step = 0;
        self.done()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "commands")]
    pub example_commands: Vec<String>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.id == self.example.id
            || !crate::cron_workspace::within(self, 2 * MAX_TASK_BYTES)
        {
            return Err("Practice requires a separate bounded identity".into());
        }
        let a = &self.assignment.cases[0].initial.disks;
        let b = &self.example.cases[0].initial.disks;
        if a.iter().any(|x| {
            b.iter()
                .any(|y| x.device == y.device || x.mountpoint == y.mountpoint)
        }) {
            return Err("Practice needs its own virtual devices and mountpoints".into());
        }
        for task in [&self.assignment, &self.example] {
            if task
                .cases
                .iter()
                .all(|c| c.checks(&c.initial).iter().all(|r| r.0))
            {
                return Err("Retain meaningful unfinished work in each integrity starter".into());
            }
        }
        if self.example_commands.is_empty()
            || self.example_commands.len() > 12
            || !self
                .example
                .run(&self.example_commands)?
                .iter()
                .all(CaseResult::passed)
        {
            return Err(
                "Worked integrity commands must reach all separate practice outcomes".into(),
            );
        }
        Ok(())
    }
}

pub const MAX_SESSION_BYTES: usize = 524288;
pub const MAX_TRANSCRIPT_BYTES: usize = 65536;
pub const PREFIX: &str = "FS INTEGRITY\n";
fn checked_history<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<String>>, D::Error> {
    #[derive(Deserialize)]
    struct History(#[serde(deserialize_with = "commands")] Vec<String>);
    Option::<History>::deserialize(d).map(|value| value.map(|h| h.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub text: String,
    pub status: u8,
    pub diagnostic: Option<terminal::DiagnosticKind>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Proof {
    task: Task,
    history: Vec<String>,
    rows: Vec<CaseResult>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "commands")]
    history: Vec<String>,
    #[serde(default, deserialize_with = "checked_history")]
    checked: Option<Vec<String>>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command: String,
    #[serde(deserialize_with = "crate::fs_integrity::sequence::<_,Message,40>")]
    pub transcript: Vec<Message>,
    #[serde(skip)]
    current: Option<System>,
    #[serde(skip)]
    proof: Option<Proof>,
}
impl Scope {
    pub fn new(task: &Task) -> Result<Self, String> {
        task.validate()?;
        Ok(Self {
            history: vec![],
            checked: None,
            command: String::new(),
            transcript: vec![],
            current: Some(task.cases[0].initial.clone()),
            proof: None,
        })
    }
    pub fn history(&self) -> &[String] {
        &self.history
    }
    pub fn current(&self) -> Option<&System> {
        self.current.as_ref()
    }
    pub fn pending_recheck(&self) -> bool {
        self.checked.as_ref() == Some(&self.history) && self.proof.is_none()
    }
    pub fn validate(&self, task: &Task) -> Result<(), String> {
        task.validate()?;
        if !history_valid(&self.history)
            || self.command.len() > terminal::MAX_COMMAND
            || self.transcript.len() > 40
            || !crate::cron_workspace::within(&self.transcript, MAX_TRANSCRIPT_BYTES)
            || self
                .checked
                .as_ref()
                .is_some_and(|history| history != &self.history)
            || self.transcript.iter().any(|m| {
                m.input.len() > 2048
                    || m.text.len() > 32768
                    || !crate::cron_workspace::within(m, 32768)
            })
        {
            return Err(
                "Invalid or oversized saved integrity work; raw drafts are not executed".into(),
            );
        }
        if self.current.is_none() {
            let mut restored = task.cases[0].initial.clone();
            for command in &self.history {
                restored
                    .execute(command)
                    .map_err(|e| format!("Invalid saved action for this supplied volume: {e}"))?;
            }
        }
        Ok(())
    }
    /// Reconstruct only the interactive sample; this does not grant a grade receipt.
    pub fn restore_current(&mut self, task: &Task) -> Result<(), String> {
        self.validate(task)?;
        if self.current.is_some() {
            return Ok(());
        }
        let mut current = task.cases[0].initial.clone();
        for command in &self.history {
            current
                .execute(command)
                .map_err(|e| format!("Saved virtual action cannot be replayed: {e}"))?;
        }
        self.current = Some(current);
        Ok(())
    }
    fn record(&mut self, mut message: Message) {
        if message.input.len() > 2048 {
            let mut end = 2048;
            while !message.input.is_char_boundary(end) {
                end -= 1;
            }
            message.input.truncate(end);
        }
        let full = message.text.clone();
        let mut length = full.len();
        while !crate::cron_workspace::within(&message, 32768) {
            length /= 2;
            while !full.is_char_boundary(length) {
                length -= 1;
            }
            message.text = format!(
                "{}\n[Output shortened to fit saved-console limits.]",
                &full[..length]
            );
        }
        self.transcript.push(message);
        while self.transcript.len() > 40
            || !crate::cron_workspace::within(&self.transcript, MAX_TRANSCRIPT_BYTES)
        {
            self.transcript.remove(0);
        }
    }
    pub fn execute_recorded(
        &mut self,
        task: &Task,
        input: &str,
    ) -> Result<terminal::Output, terminal::Diagnostic> {
        self.restore_current(task)
            .map_err(|message| terminal::Diagnostic {
                kind: terminal::DiagnosticKind::Invalid,
                status: 1,
                message,
            })?;
        let mut next = self.current.as_ref().unwrap().clone();
        let result=next.execute(input).and_then(|output|{
            if output.changed {
                let mut history=self.history.clone();history.push(input.into());
                if !history_valid(&history){return Err(terminal::Diagnostic{kind:terminal::DiagnosticKind::Limit,status:1,message:"Saved action limit reached; this command was not applied. Reset only this attempt if you want to start again.".into()})}
                self.history=history;self.current=Some(next);self.checked=None;self.proof=None;
            }
            Ok(output)
        });
        let message = match &result {
            Ok(output) => Message {
                input: input.chars().take(2048).collect(),
                text: output.stdout.clone(),
                status: output.status,
                diagnostic: None,
            },
            Err(error) => Message {
                input: input.chars().take(2048).collect(),
                text: error.message.clone(),
                status: error.status,
                diagnostic: Some(error.kind),
            },
        };
        self.record(message);
        result
    }
    pub fn submit(&mut self, task: &Task) -> Result<terminal::Output, terminal::Diagnostic> {
        let input = self.command.clone();
        let result = self.execute_recorded(task, &input);
        if result.is_ok() {
            self.command.clear();
        }
        result
    }
    pub fn begin(&mut self, task: &Task) -> Result<Run, String> {
        self.validate(task)?;
        // Retain the previous checked-history stamp across a stopped unchanged
        // recheck. It grants no credit without a new in-process proof.
        self.proof = None;
        Run::new(task, &self.history)
    }
    pub fn finish(&mut self, task: &Task, run: &Run) -> Result<(), String> {
        self.validate(task)?;
        if run.task() != task || run.history() != self.history || !run.done() {
            return Err(
                "Integrity checks are unfinished or no longer match the current actions".into(),
            );
        }
        self.checked = Some(self.history.clone());
        if self.current.is_none() {
            self.current = run
                .results()
                .and_then(|rows| rows.first())
                .filter(|row| row.diagnostic.is_none())
                .and_then(|row| row.actual.clone());
        }
        self.proof = Some(Proof {
            task: task.clone(),
            history: self.history.clone(),
            rows: run.results().unwrap().into(),
        });
        Ok(())
    }
    pub fn run_checked(&mut self, task: &Task) -> Result<Vec<CaseResult>, String> {
        let mut run = self.begin(task)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                self.finish(task, &run)?;
                return Ok(run.results);
            }
        }
        Err("Integrity checks reached their bounded phase limit".into())
    }
    pub fn results(&self, task: &Task) -> Option<&[CaseResult]> {
        let proof = self.proof.as_ref()?;
        (self.checked.as_ref() == Some(&self.history)
            && proof.history == self.history
            && proof.task == *task)
            .then_some(proof.rows.as_slice())
    }
    pub fn passed(&self, task: &Task) -> bool {
        self.results(task).is_some_and(|rows| {
            rows.len() == task.cases.len()
                && rows
                    .iter()
                    .zip(&task.cases)
                    .all(|(row, case)| row.id == case.id && row.passed())
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub definition: Definition,
    pub view: crate::script_teaching::View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    pub fn new(definition: Definition) -> Result<Self, String> {
        definition.validate()?;
        Ok(Self {
            version: 1,
            assignment: Scope::new(&definition.assignment)?,
            example: Scope::new(&definition.example)?,
            definition,
            view: crate::script_teaching::View::Introduction,
        })
    }
    pub fn validate(&self, expected: &Definition) -> Result<(), String> {
        if self.version != 1
            || self.definition != *expected
            || !crate::cron_workspace::within(self, MAX_SESSION_BYTES)
        {
            return Err("Integrity source/version/size mismatch; supplied data must remain bound to this lesson".into());
        }
        expected.validate()?;
        self.assignment.validate(&expected.assignment)?;
        self.example.validate(&expected.example)
    }
    pub fn decode(raw: &str, expected: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION_BYTES {
            return Err("Integrity progress exceeds512KiB".into());
        }
        let session: Self =
            serde_json::from_str(raw).map_err(|e| format!("Invalid integrity progress: {e}"))?;
        session.validate(expected)?;
        Ok(session)
    }
    pub fn execute_reference(
        &mut self,
        expected: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.validate(expected)?;
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native virtual console; this is an internal reference-action codec")?;
        if raw.len() > 4096 {
            return Err("Reference integrity action exceeds4KiB".into());
        }
        match serde_json::from_str(raw).map_err(|e| format!("Invalid reference action: {e}"))? {
            Action::Command { input } => self
                .assignment
                .execute_recorded(&expected.assignment, &input)
                .map(|output| output.stdout)
                .map_err(|e| e.to_string()),
            Action::Check {} => {
                let rows = self.assignment.run_checked(&expected.assignment)?;
                Ok(format!(
                    "Actual virtual consistency outcomes: {}/{} samples pass",
                    rows.iter().filter(|r| r.passed()).count(),
                    rows.len()
                ))
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        input: String,
    },
    Check {},
}

impl Definition {
    pub fn solved(&self, session: &Session) -> bool {
        session.validate(self).is_ok() && session.assignment.passed(&self.assignment)
    }
    pub fn checks(&self, session: &Session) -> Vec<(bool, String)> {
        if let Err(error) = session.validate(self) {
            return vec![(false, error)];
        }
        let Some(rows) = session.assignment.results(&self.assignment) else {
            return vec![(
                false,
                if session.assignment.pending_recheck() {
                    "Saved work needs a fresh verification against the supplied samples."
                } else {
                    "Not checked. Choose Check work to verify the current simulated actions."
                }
                .into(),
            )];
        };
        rows.iter()
            .map(|row| {
                (
                    row.passed(),
                    format!(
                        "{}: {}",
                        row.label,
                        row.checks.iter().find(|r| !r.0).map_or(
                            "Every required metadata, file and mount outcome agrees.",
                            |r| r.1.as_str()
                        )
                    ),
                )
            })
            .collect()
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::FsIntegrity { simulation },
        crate::engine::Goal::FsIntegrity { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Integrity source/state mismatch".into());
    };
    definition.validate()?;
    if !matches!(s.track.as_str(), "LPIC-1" | "LPIC-2")
        || definition.assignment.id != s.id
        || definition.assignment.instructions != s.objective
        || serde_json::to_value(&**simulation).map_err(|e| e.to_string())?
            != serde_json::to_value(Session::new((**definition).clone())?)
                .map_err(|e| e.to_string())?
    {
        return Err("Integrity source identity, task and untouched initial work must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let definition = source(s)?;
    let crate::engine::State::FsIntegrity { simulation } = &a.state else {
        return Err("Saved integrity work belongs to another activity".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible integrity-work metadata; original retained".into());
    }
    simulation.validate(definition)
}
