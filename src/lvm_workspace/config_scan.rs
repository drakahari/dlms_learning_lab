//! Standalone source-bound configuration practice prerequisite. Not admitted to
//! normal goals, sessions, commands or native UI. No host paths or execution.
use crate::{cron_workspace as bounded, lvm, lvm::config_scan as core};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const MAX_CONTEXT: usize = 192 * 1024;
const MAX_LIVE: usize = 24 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text(#[serde(deserialize_with = "crate::cron::text::<_,4096>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceName(#[serde(deserialize_with = "crate::cron::text::<_,37>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    #[serde(deserialize_with = "bounded::sequence::<_,DeviceName,8>")]
    pub entries: Vec<DeviceName>,
}
impl Selection {
    fn file(&self) -> Result<core::DevicesFile, String> {
        core::DevicesFile::new(&self.entries.iter().map(|v| v.0.clone()).collect::<Vec<_>>())
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    #[serde(deserialize_with = "crate::cron::text::<_,37>")]
    pub path: String,
    pub allowed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub label: String,
    pub inventory: lvm::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Expected,8>")]
    pub expected: Vec<Expected>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredMode {
    Filters,
    DevicesFile,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub initial_config: Text,
    pub initial_file: Option<Selection>,
    pub selection_editable: bool,
    pub required_mode: Option<RequiredMode>,
    #[serde(deserialize_with = "bounded::sequence::<_,Case,3>")]
    pub cases: Vec<Case>,
}
fn text(s: &str) -> bool {
    s.len() <= core::MAX_CONFIG
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
impl Context {
    pub fn shape(&self) -> Result<(), String> {
        if !bounded::within(self, MAX_CONTEXT)
            || !(2..=3).contains(&self.cases.len())
            || !text(&self.initial_config.0)
            || (self.selection_editable && self.initial_file.is_none())
        {
            return Err("Scan teaching needs two or three bounded supplied inventories and explicit editable-file scope".into());
        }
        // Intentional policy mistakes are allowed, but the supplied fragment must
        // be syntactically in the declared subset. Learner saves may be malformed.
        core::parse(&self.initial_config.0).map_err(|e| e.to_string())?;
        if let Some(file) = &self.initial_file {
            file.file()?;
        }
        let mut labels = BTreeSet::new();
        for case in &self.cases {
            let expected: BTreeSet<_> = case.expected.iter().map(|e| &e.path).collect();
            if case.label.trim().is_empty()
                || case.label.len() > 100
                || case.label.chars().any(char::is_control)
                || !labels.insert(&case.label)
                || case.inventory.devices.is_empty()
                || !bounded::within(&case.inventory, 64 * 1024)
                || expected.len() != case.expected.len()
                || expected != case.inventory.devices.keys().collect()
            {
                return Err("Each scan case needs a distinct label and exactly one requirement for every supplied device".into());
            }
            case.inventory.validate().map_err(|e| e.to_string())?;
        }
        if self.required_mode == Some(RequiredMode::DevicesFile) && self.initial_file.is_none() {
            return Err("A devices-file assignment needs a provided virtual file".into());
        }
        Ok(())
    }
    fn selection_shape(&self, file: &Option<Selection>) -> Result<(), String> {
        if file.is_some() != self.initial_file.is_some()
            || (!self.selection_editable && file != &self.initial_file)
        {
            return Err("The provided devices file cannot be created, removed or changed outside this task's scope".into());
        }
        if let Some(file) = file {
            file.file()?;
            let supplied: BTreeSet<&str> = self
                .cases
                .iter()
                .flat_map(|c| c.inventory.devices.keys().map(String::as_str))
                .chain(
                    self.initial_file
                        .iter()
                        .flat_map(|f| f.entries.iter().map(|e| e.0.as_str())),
                )
                .collect();
            if file
                .entries
                .iter()
                .any(|e| !supplied.contains(e.0.as_str()))
            {
                return Err("Choose only device names supplied by this simulated task".into());
            }
        }
        Ok(())
    }
    fn scan(&self, snapshot: &Snapshot) -> Result<Vec<core::Report>, String> {
        let policy = core::parse(&snapshot.config.0).map_err(|e| e.to_string())?;
        let file = snapshot.file.as_ref().map(Selection::file).transpose()?;
        let mut remaining = core::MAX_MATCH_WORK;
        let mut reports = Vec::new();
        for case in &self.cases {
            let report = core::scan_limited(&policy, &case.inventory, file.as_ref(), remaining)
                .map_err(|e| format!("{}: {e}", case.label))?;
            remaining -= report.matching_work;
            reports.push(report);
        }
        Ok(reports)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub config: Text,
    pub file: Option<Selection>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Live {
    pub draft: Text,
    pub saved: Snapshot,
    pub draft_file: Option<Selection>,
    pub last_run: Option<Snapshot>,
}
impl Live {
    pub fn new(c: &Context) -> Result<Self, String> {
        c.shape()?;
        Ok(Self {
            draft: c.initial_config.clone(),
            saved: Snapshot {
                config: c.initial_config.clone(),
                file: c.initial_file.clone(),
            },
            draft_file: c.initial_file.clone(),
            last_run: None,
        })
    }
    pub fn shape(&self) -> Result<(), String> {
        if !bounded::within(self, MAX_LIVE)
            || !text(&self.draft.0)
            || !text(&self.saved.config.0)
            || self.last_run.as_ref().is_some_and(|s| !text(&s.config.0))
        {
            return Err(
                "Configuration draft or retained scan exceeds its bounds; previous work retained"
                    .into(),
            );
        }
        for file in [&self.saved.file, &self.draft_file].into_iter().flatten() {
            file.file()?;
        }
        if let Some(file) = self.last_run.as_ref().and_then(|s| s.file.as_ref()) {
            file.file()?;
        }
        Ok(())
    }
    pub fn validate(&self, c: &Context) -> Result<(), String> {
        c.shape()?;
        self.shape()?;
        c.selection_shape(&self.saved.file)?;
        c.selection_shape(&self.draft_file)?;
        if let Some(s) = &self.last_run {
            c.selection_shape(&s.file)?;
        }
        Ok(())
    }
    pub fn edit(&mut self, c: &Context, value: String) -> Result<(), String> {
        self.validate(c)?;
        if !text(&value) {
            return Err("Keep configuration within4096bytes and use ordinary line breaks; previous draft retained".into());
        }
        self.draft = Text(value);
        Ok(())
    }
    /// This edits an explicitly simulated devname selection, not the real
    /// system.devices hardware-ID serialization or a host lvmdevices command.
    pub fn edit_selection(&mut self, c: &Context, file: Selection) -> Result<(), String> {
        self.validate(c)?;
        if !c.selection_editable {
            return Err("This task's provided devices file is read-only".into());
        }
        let next = Some(file);
        c.selection_shape(&next)?;
        self.draft_file = next;
        Ok(())
    }
    pub fn save_config(&mut self, c: &Context) -> Result<(), String> {
        self.validate(c)?;
        self.saved.config = self.draft.clone();
        Ok(())
    }
    pub fn save_selection(&mut self, c: &Context) -> Result<(), String> {
        self.validate(c)?;
        if !c.selection_editable {
            return Err("This task's provided devices file is read-only".into());
        }
        self.saved.file = self.draft_file.clone();
        Ok(())
    }
    /// Record what actually ran, including a bad saved configuration. No cached
    /// report or success bit is trusted; reading results recomputes actual scans.
    pub fn run(&mut self, c: &Context) -> Result<Vec<core::Report>, String> {
        self.validate(c)?;
        self.last_run = Some(self.saved.clone());
        c.scan(&self.saved)
    }
    pub fn last_results(
        &self,
        c: &Context,
    ) -> Result<Option<Result<Vec<core::Report>, String>>, String> {
        self.validate(c)?;
        Ok(self.last_run.as_ref().map(|s| c.scan(s)))
    }
    pub fn changed_since_run(&self) -> bool {
        self.last_run.as_ref() != Some(&self.saved)
            || self.draft != self.saved.config
            || self.draft_file != self.saved.file
    }
    /// Re-evaluate saved behavior across every case. Run-position and draft rows
    /// are additional requirements; neither can replace actual outcome checks.
    pub fn checks(&self, c: &Context) -> Vec<(bool, String)> {
        if let Err(e) = self.validate(c) {
            return vec![(false, e)];
        }
        let mut rows = vec![
            (
                self.draft == self.saved.config && self.draft_file == self.saved.file,
                "Save edited configuration and device selection before checking".into(),
            ),
            (
                self.last_run.as_ref() == Some(&self.saved),
                "Run a scan of the current saved configuration before checking".into(),
            ),
        ];
        let reports = match c.scan(&self.saved) {
            Ok(r) => r,
            Err(e) => {
                rows.push((false, e));
                return rows;
            }
        };
        for (case, report) in c.cases.iter().zip(&reports) {
            if let Some(mode) = c.required_mode {
                let correct = match mode {
                    RequiredMode::Filters => report.mode != core::Mode::DevicesFile,
                    RequiredMode::DevicesFile => report.mode == core::Mode::DevicesFile,
                };
                rows.push((correct,format!("{}: {}",case.label,if correct { "The intended selection mechanism is active" } else if mode == RequiredMode::DevicesFile { "Use the provided devices file; filters must not replace this task's selection mechanism" } else { "Use the filter policy; the enabled provided devices file currently takes precedence" })));
            }
            for e in &case.expected {
                let r = report
                    .records
                    .iter()
                    .find(|r| r.path == e.path)
                    .expect("validated complete inventory scan");
                let why = match &r.reason {
                    core::Reason::FilterRule { index, action } => format!(
                        "filter rule{} {} first",
                        index + 1,
                        if *action == core::Action::Accept {
                            "accepts"
                        } else {
                            "rejects"
                        }
                    ),
                    core::Reason::NoFilterMatched => {
                        "no filter matched, so the default allows it".into()
                    }
                    core::Reason::ListedInDevicesFile => {
                        "listed in the enabled devices file; filters are ignored".into()
                    }
                    core::Reason::NotInDevicesFile => {
                        "not listed in the enabled devices file; filters are ignored".into()
                    }
                };
                rows.push((
                    r.allowed == e.allowed,
                    format!(
                        "{} · {}: {}. {}.",
                        case.label,
                        e.path,
                        if r.allowed == e.allowed {
                            if e.allowed {
                                "Allowed as required"
                            } else {
                                "Excluded as required"
                            }
                        } else if e.allowed {
                            "Needs to be allowed"
                        } else {
                            "Needs to be excluded"
                        },
                        why
                    ),
                ));
            }
            rows.push((
                report.unresolved_file_entries.is_empty(),
                format!(
                    "{}: {}",
                    case.label,
                    if report.unresolved_file_entries.is_empty() {
                        "No missing devices-file entries"
                    } else {
                        "Remove devices-file entries absent from this supplied inventory"
                    }
                ),
            ));
        }
        rows
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    SaveConfig { text: Text },
    SaveSelection { selection: Selection },
    Scan,
}
impl Operation {
    pub fn shape(&self, c: &Context) -> Result<(), String> {
        match self {
            Self::SaveConfig { text: value } if !text(&value.0) => {
                Err("Configuration action exceeds its text bounds".into())
            }
            Self::SaveSelection { selection } => {
                if !c.selection_editable {
                    return Err("Selection is read-only in this task".into());
                }
                c.selection_shape(&Some(selection.clone()))
            }
            _ => Ok(()),
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::SaveConfig { .. } => "Save simulated LVM configuration",
            Self::SaveSelection { .. } => "Save simulated devices-file selection",
            Self::Scan => "Run simulated device scan",
        }
    }
}
impl Live {
    pub fn apply(&mut self, c: &Context, op: &Operation) -> Result<(), String> {
        self.validate(c)?;
        op.shape(c)?;
        if matches!(op, Operation::Scan) {
            // A diagnostic still records what ran. No partial scan or successful
            // flag is stored, and current checks derive the actual failure again.
            return self.run(c).map(|_| ());
        }
        let mut next = self.clone();
        match op {
            Operation::SaveConfig { text } => {
                next.edit(c, text.0.clone())?;
                next.save_config(c)?;
            }
            Operation::SaveSelection { selection } => {
                next.edit_selection(c, selection.clone())?;
                next.save_selection(c)?;
            }
            Operation::Scan => unreachable!(),
        }
        next.validate(c)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,16>")]
    pub reference: Vec<Operation>,
}
impl Task {
    /// Shape validation never executes the worked reference on idle grading.
    pub fn shape(&self) -> Result<(), String> {
        self.context.shape()?;
        if self.reference.is_empty()
            || self.reference.len() > 16
            || !matches!(self.reference.last(), Some(Operation::Scan))
            || !bounded::within(self, 256 * 1024)
        {
            return Err(
                "A scan task needs a bounded reference finishing with an actual scan".into(),
            );
        }
        for op in &self.reference {
            op.shape(&self.context)?;
        }
        Ok(())
    }
    /// Authoring-only reachability check over the actual parser and inventories.
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let mut initial = Live::new(&self.context)?;
        let _ = initial.run(&self.context);
        if initial.checks(&self.context).iter().all(|r| r.0) {
            return Err("Assignment must require a real configuration or selection change".into());
        }
        let mut live = Live::new(&self.context)?;
        for op in &self.reference {
            live.apply(&self.context, op)?;
        }
        if live.checks(&self.context).iter().any(|r| !r.0) {
            return Err(
                "The authored reference does not meet every actual scan requirement".into(),
            );
        }
        Ok(())
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
pub struct Practice {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,12>")]
    pub worked: Vec<Step>,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,4>")]
    pub experiment: Vec<Operation>,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,8>")]
    pub repair: Vec<Operation>,
}
impl Practice {
    pub fn shape(&self) -> Result<(), String> {
        self.context.shape()?;
        if self.worked.is_empty()
            || self.worked.len() > 12
            || self.experiment.is_empty()
            || self.experiment.len() > 4
            || self.repair.is_empty()
            || self.repair.len() > 8
            || !bounded::within(self, 256 * 1024)
            || !self
                .worked
                .last()
                .is_some_and(|s| matches!(s.operation, Operation::Scan))
            || !matches!(self.experiment.last(), Some(Operation::Scan))
            || !matches!(self.repair.last(), Some(Operation::Scan))
        {
            return Err("Guided scan practice needs bounded worked, change and repair actions, each ending with a real scan".into());
        }
        for step in &self.worked {
            if step.explanation.trim().is_empty()
                || step.explanation.len() > 800
                || step
                    .explanation
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
            {
                return Err(
                    "Every worked action needs a short explanation of what to observe".into(),
                );
            }
            step.operation.shape(&self.context)?;
        }
        for op in self.experiment.iter().chain(&self.repair) {
            op.shape(&self.context)?;
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let c = &self.context;
        let mut live = Live::new(c)?;
        let _ = live.run(c);
        if live.checks(c).iter().all(|r| r.0) {
            return Err("Practice must require a real change before its worked scan".into());
        }
        for step in &self.worked {
            live.apply(c, &step.operation)?;
        }
        if live.checks(c).iter().any(|r| !r.0) {
            return Err("The worked scan does not satisfy its actual requirements".into());
        }
        let before = live.saved.clone();
        for op in &self.experiment {
            if let Err(e) = live.apply(c, op)
                && !matches!(op, Operation::Scan)
            {
                return Err(e);
            }
        }
        if live.saved == before || live.checks(c).iter().all(|r| r.0) {
            return Err("Practice change must expose a real saved-policy or device-selection mistake, not just clear a checked flag".into());
        }
        for op in &self.repair {
            live.apply(c, op)?;
        }
        if live.checks(c).iter().any(|r| !r.0) {
            return Err("Practice repair does not restore every actual scan outcome".into());
        }
        Ok(())
    }
}
/// Assignment and example snapshots are separate. No version migration, goal
/// kind or normal Session field is admitted by this standalone type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub version: u32,
    pub assignment: Live,
    pub example: Live,
}
fn separate(task: &Task, practice: &Practice) -> bool {
    let paths: BTreeSet<_> = task
        .context
        .cases
        .iter()
        .flat_map(|c| c.inventory.devices.keys())
        .collect();
    practice
        .context
        .cases
        .iter()
        .flat_map(|c| c.inventory.devices.keys())
        .all(|p| !paths.contains(p))
}
impl Runtime {
    pub fn new(task: &Task, practice: &Practice) -> Result<Self, String> {
        task.shape()?;
        practice.shape()?;
        if !separate(task, practice) {
            return Err(
                "Worked practice must use different supplied device names from the assignment"
                    .into(),
            );
        }
        Ok(Self {
            version: 1,
            assignment: Live::new(&task.context)?,
            example: Live::new(&practice.context)?,
        })
    }
    pub fn shape(&self) -> Result<(), String> {
        if self.version != 1 || !bounded::within(self, 64 * 1024) {
            return Err("Unsupported scan-work version or size".into());
        }
        self.assignment.shape()?;
        self.example.shape()
    }
    pub fn validate(&self, task: &Task, practice: &Practice) -> Result<(), String> {
        task.shape()?;
        practice.shape()?;
        if self.version != 1 || !separate(task, practice) || !bounded::within(self, 64 * 1024) {
            return Err("Scan work belongs to another version or exceeds its separate assignment/example bounds".into());
        }
        self.assignment.validate(&task.context)?;
        self.example.validate(&practice.context)
    }
    pub fn reset_assignment(&mut self, task: &Task, practice: &Practice) -> Result<(), String> {
        self.validate(task, practice)?;
        self.assignment = Live::new(&task.context)?;
        Ok(())
    }
    pub fn reset_example(&mut self, task: &Task, practice: &Practice) -> Result<(), String> {
        self.validate(task, practice)?;
        self.example = Live::new(&practice.context)?;
        Ok(())
    }
}
