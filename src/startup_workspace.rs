//! Source-bound startup configuration assignments; all execution is in memory.
use crate::{shell, shell::startup as model};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_TICKS: usize = 32768;
pub const PREFIX: &str = "STARTUP FILES\n";
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
}
fn paths<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,32>")] String);
    crate::fs_integrity::sequence::<D, Text, 6>(d).map(|v| v.into_iter().map(|s| s.0).collect())
}
fn optional_source<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,2048>")] String);
    Option::<Text>::deserialize(d).map(|v| v.map(|s| s.0))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub invocation: model::Invocation,
    #[serde(deserialize_with = "paths")]
    pub present: Vec<String>,
    #[serde(default, deserialize_with = "optional_source")]
    pub system_profile: Option<String>,
    pub input: shell::Case,
    pub parent: BTreeMap<String, shell::EnvironmentVariable>,
    pub child: shell::Case,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub instructions: String,
    pub starter: model::Files,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub child_source: String,
    #[serde(deserialize_with = "crate::fs_integrity::sequence::<_,Case,6>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.starter.validate()?;
        if !identifier(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 4096
            || self.child_source.trim().is_empty()
            || self.child_source.len() > 2048
            || self.starter.0.is_empty()
            || self.starter.0.contains_key("/etc/profile")
            || !(2..=6).contains(&self.cases.len())
            || !crate::cron_workspace::within(self, 262144)
        {
            return Err("Invalid bounded startup assignment".into());
        }
        let mut ids = BTreeSet::new();
        for case in &self.cases {
            if !identifier(&case.id)
                || !ids.insert(&case.id)
                || case.label.trim().is_empty()
                || case.label.len() > 120
                || case.label.chars().any(char::is_control)
                || !case.input.valid()
                || !case.child.valid()
                || !case.child.env.is_empty()
                || case.input.after != case.input.files
                || case.child.after != case.child.files
                || case.child.files != case.input.files
                || case.parent.is_empty()
                || case.parent.len() > 8
                || case.parent.keys().any(|n| !name(n))
                || case.parent.values().any(|v| {
                    v.value
                        .as_ref()
                        .is_some_and(|s| s.len() > 4096 || s.contains('\0'))
                })
                || !crate::cron_workspace::within(case, 65536)
            {
                return Err("Invalid startup sample inputs or expected settings".into());
            }
            let present: BTreeSet<_> = case.present.iter().collect();
            if present.len() != case.present.len()
                || present.iter().any(|p| !self.starter.0.contains_key(*p))
                || case
                    .system_profile
                    .as_ref()
                    .is_some_and(|s| s.len() > 2048 || s.contains('\0'))
            {
                return Err(
                    "Startup sample file inventory is not bound to the supplied templates".into(),
                );
            }
            let files = self.files_for(&self.starter, case)?;
            model::select(&case.invocation, &files)?;
        }
        Ok(())
    }
    pub fn files_for(&self, draft: &model::Files, case: &Case) -> Result<model::Files, String> {
        self.validate_files(draft)?;
        let mut files = model::Files(
            case.present
                .iter()
                .map(|p| (p.clone(), draft.0[p].clone()))
                .collect(),
        );
        if let Some(source) = &case.system_profile {
            files.0.insert(
                "/etc/profile".into(),
                model::File {
                    text: source.clone(),
                },
            );
        }
        files.validate()?;
        Ok(files)
    }
    pub fn validate_files(&self, draft: &model::Files) -> Result<(), String> {
        draft.validate()?;
        if draft.0.keys().ne(self.starter.0.keys()) {
            return Err(
                "Keep the supplied startup filenames; edit their contents in this activity".into(),
            );
        }
        Ok(())
    }
    pub fn run(&self, draft: &model::Files) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, draft)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.rows);
            }
        }
        Err("Startup checks reached their bounded execution limit".into())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParentObservation {
    pub selection: Vec<String>,
    pub output: shell::Outcome,
    pub variables: shell::EnvironmentSnapshot,
    pub command_finished: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub checks: Vec<(bool, String)>,
    pub parent: Option<ParentObservation>,
    pub child: Option<shell::Outcome>,
    pub error: Option<String>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.error.is_none() && !self.checks.is_empty() && self.checks.iter().all(|c| c.0)
    }
}
fn setting_value(value: &Option<String>) -> String {
    value
        .as_ref()
        .map_or_else(|| "not set".into(), |s| format!("{s:?}"))
}
fn result(
    case: &Case,
    parent: Option<ParentObservation>,
    child: Option<shell::Outcome>,
    error: Option<String>,
) -> CaseResult {
    let mut checks = vec![];
    if let Some(p) = &parent {
        checks.push((
            p.command_finished,
            "Startup must finish without closing the session before verification.".into(),
        ));
        checks.push((
            p.output.status == case.input.status,
            format!(
                "Startup status: expected {}, actual {}.",
                case.input.status, p.output.status
            ),
        ));
        checks.push((p.output.stdout==case.input.stdout,if p.output.stdout==case.input.stdout{"Startup output matches the requested behavior.".into()}else{format!("Startup output differs: expected {:?}, actual {:?}. Keep startup messages in the requested session type.",case.input.stdout,p.output.stdout)}));
        checks.push((
            p.output.stderr == case.input.stderr,
            "Startup must not emit unexpected error output.".into(),
        ));
        checks.push((
            p.output.files == case.input.files,
            "Startup must preserve the supplied simulated files.".into(),
        ));
        for (n, wanted) in &case.parent {
            let actual = p
                .variables
                .parent
                .get(n)
                .cloned()
                .unwrap_or(shell::EnvironmentVariable {
                    value: None,
                    exported: false,
                });
            checks.push((actual.value==wanted.value,if actual.value==wanted.value{format!("{n}: requested value is set.")}else{format!("{n}: expected {}, actual {}. Inspect the file actually selected for this session.",setting_value(&wanted.value),setting_value(&actual.value))}));
            checks.push((actual.exported==wanted.exported,if actual.exported==wanted.exported{format!("{n}: sharing with new programs is correct.")}else if wanted.exported{format!("{n} is not shared with new programs. Setting a value in this shell and exporting it are separate actions.")}else{format!("{n} is shared with new programs; keep this setting only in the current shell.")}));
        }
    } else {
        checks.push((
            false,
            "Startup did not finish successfully; read the diagnostic and retry.".into(),
        ));
    }
    if let Some(c) = &child {
        checks.push((
            c.error.is_none(),
            c.error.clone().unwrap_or_else(|| {
                "The supplied verification program ran in the simulator.".into()
            }),
        ));
        checks.push((c.stdout==case.child.stdout,if c.stdout==case.child.stdout{"New-program output matches.".into()}else{format!("New-program output differs: expected {:?}, actual {:?}. Check the values and export attributes passed from startup.",case.child.stdout,c.stdout)}));
        checks.push((
            c.stderr == case.child.stderr,
            "New-program error output matches.".into(),
        ));
        checks.push((
            c.status == case.child.status,
            format!(
                "New-program status: expected {}, actual {}.",
                case.child.status, c.status
            ),
        ));
        checks.push((
            c.files == case.child.files,
            "The verification program preserves its simulated sample files.".into(),
        ));
    } else {
        checks.push((
            false,
            "The supplied verification program could not run after startup.".into(),
        ));
    }
    if let Some(why) = &error {
        checks.insert(0, (false, why.clone()));
    }
    CaseResult {
        id: case.id.clone(),
        label: case.label.clone(),
        checks,
        parent,
        child,
        error,
    }
}
enum Stage {
    Prepare,
    Startup(model::Run),
    Child(ParentObservation, shell::Vm),
}
pub struct Run {
    task: Task,
    draft: model::Files,
    at: usize,
    stage: Option<Stage>,
    rows: Vec<CaseResult>,
    stopped: bool,
    ticks: usize,
}
impl Run {
    pub fn new(task: &Task, draft: &model::Files) -> Result<Self, String> {
        task.validate()?;
        task.validate_files(draft)?;
        Ok(Self {
            task: task.clone(),
            draft: draft.clone(),
            at: 0,
            stage: Some(Stage::Prepare),
            rows: vec![],
            stopped: false,
            ticks: 0,
        })
    }
    pub fn task(&self) -> &Task {
        &self.task
    }
    pub fn draft(&self) -> &model::Files {
        &self.draft
    }
    pub fn done(&self) -> bool {
        !self.stopped && self.at == self.task.cases.len()
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.rows)
    }
    pub fn stop(&mut self) {
        self.stopped = true;
        self.stage = None;
        self.rows.clear();
    }
    fn push(&mut self, row: CaseResult) {
        self.rows.push(row);
        self.at += 1;
        self.stage = Some(Stage::Prepare);
    }
    pub fn tick(&mut self) -> bool {
        if self.stopped {
            return false;
        }
        if self.done() {
            return true;
        }
        self.ticks += 1;
        if self.ticks >= MAX_TICKS - 4 {
            while self.at < self.task.cases.len() {
                let c = &self.task.cases[self.at];
                self.push(result(c,None,None,Some("Simulation execution limit reached; simplify the startup settings and retry.".into())));
            }
            return true;
        }
        let case = self.task.cases[self.at].clone();
        match self.stage.take().unwrap_or(Stage::Prepare) {
            Stage::Prepare => {
                let files = self.task.files_for(&self.draft, &case);
                match files.and_then(|f| model::Run::new(&case.invocation, &f, &case.input, ":")) {
                    Ok(run) => self.stage = Some(Stage::Startup(run)),
                    Err(e) => self.push(result(&case, None, None, Some(e))),
                }
            }
            Stage::Startup(mut run) => {
                run.tick();
                if !run.done() {
                    self.stage = Some(Stage::Startup(run));
                    return false;
                }
                let output = run.outcome().unwrap().clone();
                if let Some(e) = &output.error {
                    self.push(result(&case, None, None, Some(e.clone())));
                    return self.done();
                }
                match run.environment() {
                    Err(e) => self.push(result(&case, None, None, Some(e))),
                    Ok(variables) => {
                        let parent = ParentObservation {
                            selection: run.selected().iter().map(|s| s.path.clone()).collect(),
                            output,
                            variables,
                            command_finished: run.session_command_finished(),
                        };
                        if !parent.command_finished {
                            self.push(result(&case, Some(parent), None, None));
                            return self.done();
                        }
                        let mut child = case.child.clone();
                        child.env = parent.variables.child.clone();
                        match shell::Vm::new(&self.task.child_source, &child) {
                            Ok(vm) => self.stage = Some(Stage::Child(parent, vm)),
                            Err(e) => self.push(result(
                                &case,
                                Some(parent),
                                None,
                                Some(format!(
                                    "Cannot start the supplied verification program: {e}"
                                )),
                            )),
                        }
                    }
                }
            }
            Stage::Child(parent, mut vm) => {
                vm.tick(16);
                if vm.done() {
                    self.push(result(&case, Some(parent), Some(vm.outcome), None))
                } else {
                    self.stage = Some(Stage::Child(parent, vm))
                }
            }
        }
        self.done()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    pub example_worked: model::Files,
}
impl Definition {
    pub fn structural(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        self.example.validate_files(&self.example_worked)?;
        if self.initial.id == self.example.id {
            return Err("Practice and assignment need different source identities".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.structural()?;
        if self
            .initial
            .run(&self.initial.starter)?
            .iter()
            .all(CaseResult::passed)
            || self
                .example
                .run(&self.example.starter)?
                .iter()
                .all(CaseResult::passed)
        {
            return Err("Startup assignments need meaningful unfinished starters".into());
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked startup example cannot meet its actual requirements".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
struct Proof {
    task: Task,
    draft: model::Files,
    rows: Vec<CaseResult>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub files: model::Files,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub selected: String,
    pub case: usize,
    pub tested: Option<model::Files>,
    #[serde(skip)]
    proof: Option<Proof>,
}
impl Scope {
    pub fn new(task: &Task) -> Self {
        Self {
            files: task.starter.clone(),
            selected: task
                .cases
                .first()
                .and_then(|case| {
                    task.files_for(&task.starter, case)
                        .ok()
                        .and_then(|files| model::select(&case.invocation, &files).ok())
                })
                .and_then(|selected| {
                    selected
                        .into_iter()
                        .rev()
                        .find(|file| task.starter.0.contains_key(&file.path))
                        .map(|file| file.path)
                })
                .unwrap_or_else(|| task.starter.0.keys().next().cloned().unwrap_or_default()),
            case: 0,
            tested: None,
            proof: None,
        }
    }
    pub fn validate(&self, task: &Task) -> Result<(), String> {
        task.validate_files(&self.files)?;
        if !self.files.0.contains_key(&self.selected)
            || self.case >= task.cases.len()
            || self.tested.as_ref().is_some_and(|f| *f != self.files)
        {
            return Err("Invalid or stale saved startup work; original data retained".into());
        }
        Ok(())
    }
    pub fn replace(&mut self, task: &Task, files: &model::Files) -> Result<(), String> {
        task.validate_files(files)?;
        if self.files != *files {
            self.files = files.clone();
            self.tested = None;
            self.proof = None;
        }
        Ok(())
    }
    pub fn edit(&mut self, task: &Task, file: &str, text: &str) -> Result<(), String> {
        let mut next = self.files.clone();
        let target = next
            .0
            .get_mut(file)
            .ok_or("Select a supplied startup file")?;
        target.text = text.into();
        self.replace(task, &next)
    }
    pub fn begin(&mut self, task: &Task) -> Result<Run, String> {
        self.validate(task)?;
        self.proof = None;
        Run::new(task, &self.files)
    }
    pub fn finish(&mut self, task: &Task, run: &Run) -> Result<(), String> {
        self.validate(task)?;
        if run.task() != task || run.draft() != &self.files || !run.done() {
            return Err(
                "Startup checks are unfinished or no longer match the current files".into(),
            );
        }
        self.tested = Some(self.files.clone());
        self.proof = Some(Proof {
            task: task.clone(),
            draft: self.files.clone(),
            rows: run.results().unwrap().into(),
        });
        Ok(())
    }
    pub fn run(&mut self, task: &Task) -> Result<Vec<CaseResult>, String> {
        let mut run = self.begin(task)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                self.finish(task, &run)?;
                return Ok(run.rows);
            }
        }
        Err("Startup check limit reached".into())
    }
    pub fn pending(&self) -> bool {
        self.tested.as_ref() == Some(&self.files) && self.proof.is_none()
    }
    pub fn results(&self, task: &Task) -> Option<&[CaseResult]> {
        let p = self.proof.as_ref()?;
        (p.task == *task && p.draft == self.files && self.tested.as_ref() == Some(&self.files))
            .then_some(&p.rows)
    }
    pub fn passed(&self, task: &Task) -> bool {
        self.results(task).is_some_and(|rows| {
            rows.len() == task.cases.len()
                && rows
                    .iter()
                    .zip(&task.cases)
                    .all(|(r, c)| r.id == c.id && r.passed())
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: crate::script_teaching::View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(definition: &Definition) -> Self {
        Self {
            version: 1,
            origin: definition.initial.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Scope::new(&definition.initial),
            practice: Scope::new(&definition.example),
        }
    }
    pub fn validate(&self, definition: &Definition) -> Result<(), String> {
        definition.structural()?;
        self.validate_unbound()?;
        if self.version != 1
            || self.origin != definition.initial.id
            || !crate::cron_workspace::within(self, 131072)
        {
            return Err("Startup progress source/version/size mismatch".into());
        }
        self.assignment.validate(&definition.initial)?;
        self.practice.validate(&definition.example)
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.validate(d)?;
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native startup-file editor; this is an internal authoring codec")?;
        let files = model::Files::decode(raw)?;
        self.assignment.replace(&d.initial, &files)?;
        let rows = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Actual startup outcomes: {}/{} samples pass",
            rows.iter().filter(|r| r.passed()).count(),
            rows.len()
        ))
    }
}
impl Definition {
    pub fn solved(&self, s: &Session) -> bool {
        s.validate(self).is_ok() && s.assignment.passed(&self.initial)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = s.validate(self) {
            return vec![(false, e)];
        }
        let Some(rows) = s.assignment.results(&self.initial) else {
            return vec![(
                false,
                "Not yet verified. Run startup checks to test the current files.".into(),
            )];
        };
        rows.iter()
            .map(|r| {
                (
                    r.passed(),
                    format!(
                        "{}: {}",
                        r.label,
                        r.checks
                            .iter()
                            .find(|c| !c.0)
                            .map_or("Every startup and new-program requirement agrees.", |c| c
                                .1
                                .as_str())
                    ),
                )
            })
            .collect()
    }
}

pub const HELP: &str = "Edit supplied virtual Bash startup files, then Run startup checks. Interactive login reads the supplied system profile and first present personal profile; interactive non-login reads .bashrc; the supplied noninteractive BASH_ENV selection may read /work/startup.env. This activity supports ordinary settings, export, literal-format printf and simple conditions. Fixed virtual home, readable provided files and bounded source paths only. No real Bash, profiles, filesystem, processes or network. Source/dot, arbitrary BASH_ENV expansion, functions, loops, arrays, aliases, history and PATH command search are outside this startup activity; other scripting lessons retain their own supported scope.";
impl Session {
    pub fn validate_unbound(&self) -> Result<(), String> {
        if self.version != 1
            || !identifier(&self.origin)
            || !crate::cron_workspace::within(self, 131072)
        {
            return Err("Invalid bounded startup progress metadata".into());
        }
        for scope in [&self.assignment, &self.practice] {
            scope.files.validate()?;
            if scope.files.0.is_empty()
                || !scope.files.0.contains_key(&scope.selected)
                || scope.case >= 6
                || scope.tested.as_ref().is_some_and(|f| *f != scope.files)
            {
                return Err("Invalid saved startup selection or stale test stamp".into());
            }
        }
        Ok(())
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::StartupFiles { simulation },
        crate::engine::Goal::StartupFiles { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Startup-file source/state mismatch".into());
    };
    definition.structural()?;
    if !matches!(s.track.as_str(), "LPIC-1" | "Bash/Shell")
        || definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || serde_json::to_value(&**simulation).ok()
            != serde_json::to_value(Session::new(definition)).ok()
    {
        return Err("Startup source identity, task and untouched initial drafts must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::StartupFiles { simulation } = &a.state else {
        return Err("Saved startup files belong to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible startup metadata; previous work retained".into());
    }
    simulation.validate(d)
}
