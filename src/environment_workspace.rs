//! Typed parent/child environment exercises. Both programs use the in-process VM.
//! No subprocess, host environment, filesystem or network access.
use crate::{script_teaching::View, shell};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub mod path_lookup;
pub const MAX_SOURCE: usize = 8192;
pub const MAX_SESSION: usize = 2 * 1024 * 1024;
pub const MAX_JOB_UNITS: usize = 524288;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub input: shell::Case,
    pub parent: BTreeMap<String, shell::EnvironmentVariable>,
    pub child: shell::Case,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: String,
    pub instructions: String,
    pub starter: String,
    pub child_source: String,
    pub cases: Vec<Case>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_lookup: Option<path_lookup::Plan>,
}
fn text(s: &str, max: usize) -> bool {
    s.len() <= max && !s.contains('\0')
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn missing() -> shell::EnvironmentVariable {
    shell::EnvironmentVariable {
        value: None,
        exported: false,
    }
}
fn binding_name(n: &str) -> bool {
    let mut chars = n.chars();
    !n.is_empty()
        && !matches!(n, "OPTIND" | "OPTARG" | "OPTERR" | "LANG" | "LANGUAGE")
        && !n.starts_with("LC_")
        && n.len() <= 80
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn snapshot_valid(s: &shell::EnvironmentSnapshot) -> bool {
    s.parent.len() <= 128
        && s.child.len() <= 128
        && s.parent
            .iter()
            .all(|(n, v)| binding_name(n) && v.value.as_ref().is_none_or(|v| text(v, 16384)))
        && s.parent
            .iter()
            .map(|(n, v)| n.len() + v.value.as_ref().map_or(0, String::len))
            .sum::<usize>()
            <= 16384
        && s.child == child_map(&s.parent)
}
fn outcome_valid(o: &shell::Outcome) -> bool {
    (0..=255).contains(&o.status)
        && o.stdout.len() + o.stderr.len() <= 32_768
        && o.error.as_ref().is_none_or(|e| text(e, 4096))
        && o.files.len() <= 64
        && o.files.iter().all(|(p, v)| {
            p.strip_prefix("/work/").is_some_and(|n| {
                !n.is_empty()
                    && n.len() <= 250
                    && !matches!(n, "." | "..")
                    && !n.contains(['/', '\0'])
            }) && text(v, 262144)
        })
        && o.files
            .iter()
            .map(|(n, v)| n.len() + v.len())
            .sum::<usize>()
            <= 262144
}
fn child_map(parent: &BTreeMap<String, shell::EnvironmentVariable>) -> BTreeMap<String, String> {
    parent
        .iter()
        .filter_map(|(n, v)| {
            if v.exported {
                v.value.as_ref().map(|s| (n.clone(), s.clone()))
            } else {
                None
            }
        })
        .collect()
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id)
            || self.instructions.trim().is_empty()
            || !text(&self.instructions, 4096)
            || !text(&self.starter, MAX_SOURCE)
            || self.child_source.trim().is_empty()
            || !text(&self.child_source, 2048)
            || !(2..=8).contains(&self.cases.len())
        {
            return Err("Environment task needs bounded instructions, source and two to eight sample inputs".into());
        }
        let mut names = std::collections::BTreeSet::new();
        for c in &self.cases {
            if !c.input.valid()
                || !c.child.valid()
                || !names.insert(&c.input.name)
                || c.parent.is_empty()
                || c.parent.len() > 32
                || c.input
                    .env
                    .keys()
                    .chain(c.child.env.keys())
                    .chain(c.parent.keys())
                    .any(|n| !binding_name(n))
                || c.input.files != c.input.after
                || c.child.files != c.input.files
                || c.child.after != c.input.files
            {
                return Err("Invalid environment sample inputs or protected virtual files".into());
            }
            let mut desired: BTreeMap<_, _> = c
                .input
                .env
                .iter()
                .map(|(n, v)| {
                    (
                        n.clone(),
                        shell::EnvironmentVariable {
                            value: Some(v.clone()),
                            exported: true,
                        },
                    )
                })
                .collect();
            for (n, v) in &c.parent {
                if !text(n, 80) || !v.value.as_ref().is_none_or(|v| text(v, 4096)) {
                    return Err("Environment requirement exceeds its text bounds".into());
                }
                desired.insert(n.clone(), v.clone());
            }
            let mut targets = c.input.clone();
            targets.env = c
                .parent
                .iter()
                .map(|(n, v)| (n.clone(), v.value.clone().unwrap_or_default()))
                .collect();
            if !targets.valid() || c.child.env != child_map(&desired) {
                return Err(
                    "Parent requirements and supplied child example are inconsistent".into(),
                );
            }
        }
        if let Some(plan)=&self.path_lookup { plan.validate(&self.cases)?; if !path_lookup::task_fits(self){return Err("PATH task exceeds its bounded record budget".into());} }
        Ok(())
    }
    /// Authoring gate only: runtime Job creation never executes examples synchronously.
    pub fn validate_child_examples(&self) -> Result<(), String> {
        self.validate()?;
        if self
            .cases
            .iter()
            .any(|c| !c.child.accepts(&shell::run(&self.child_source, &c.child)))
        {
            return Err("Supplied child does not produce its declared example behavior".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub assignment: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id)
            || self.assignment.id == self.example.id
            || !serde_json::to_vec(self).is_ok_and(|raw| raw.len() <= MAX_SESSION / 4)
        {
            return Err("Separate environment assignment and example identifiers required".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseRun {
    pub parent: shell::Outcome,
    pub bindings: Option<shell::EnvironmentSnapshot>,
    pub child: Option<shell::Outcome>,
    pub diagnostic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_lookup: Option<path_lookup::Run>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub revision: u32,
    pub task: Task,
    pub source: String,
    pub results: Vec<CaseRun>,
    // In-process execution evidence only. Never serialized or accepted from a file.
    #[serde(skip)]
    verification: Option<u64>,
}
impl Record {
    fn content_id(&self) -> Option<u64> {
        use std::hash::Hasher;
        let raw = serde_json::to_vec(self).ok()?;
        if raw.len() > MAX_SESSION / 4 {
            return None;
        }
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        hash.write(&raw);
        Some(hash.finish())
    }
    /// A runtime consistency token, not a signature or tamper-proof progress claim.
    pub fn verified(&self) -> bool {
        self.verification
            .is_some_and(|token| self.content_id() == Some(token))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub source: String,
    pub result: Option<Record>,
}
impl Draft {
    pub fn needs_reverification(&self, task: &Task) -> bool {
        self.result.as_ref().is_some_and(|r| {
            r.revision == shell::EVALUATOR_REVISION
                && r.task == *task
                && r.source == self.source
                && !r.verified()
        })
    }
    pub fn new(t: &Task) -> Self {
        Self {
            source: t.starter.clone(),
            result: None,
        }
    }
    pub fn edit(&mut self, source: String) -> Result<(), String> {
        if !text(&source, MAX_SOURCE) {
            return Err("Setup scripts are limited to 8 KiB without NUL characters".into());
        }
        if source != self.source {
            self.source = source;
            self.result = None;
        }
        Ok(())
    }
    pub fn store(&mut self, record: Record, task: &Task) -> bool {
        if record.source == self.source
            && record.task == *task
            && record.revision == shell::EVALUATOR_REVISION
            && record.results.len() == task.cases.len()
            && record.verified()
            && serde_json::to_vec(&record).is_ok_and(|raw| raw.len() <= MAX_SESSION / 4)
        {
            self.result = Some(record);
            true
        } else {
            false
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub definition: Definition,
    pub view: View,
    pub assignment: Draft,
    pub example: Draft,
}
impl Session {
    /// Bound raw bytes before deserialization; decoded observations are unverified.
    pub fn decode(raw: &[u8], expected: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved environment attempt exceeds 2 MiB; original retained".into());
        }
        let value: Self = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
        value.validate(expected)?;
        Ok(value)
    }
    pub fn new(definition: Definition) -> Self {
        Self {
            assignment: Draft::new(&definition.assignment),
            example: Draft::new(&definition.example),
            definition,
            view: View::Introduction,
        }
    }
    pub fn validate(&self, expected: &Definition) -> Result<(), String> {
        expected.validate()?;
        if self.definition != *expected
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err(
                "Environment attempt does not match its bundled task or exceeds storage limits"
                    .into(),
            );
        }
        for (d, t) in [
            (&self.assignment, &expected.assignment),
            (&self.example, &expected.example),
        ] {
            if !text(&d.source, MAX_SOURCE) {
                return Err("Invalid saved environment draft".into());
            }
            if let Some(r) = &d.result {
                if r.task != *t
                    || r.source != d.source
                    || r.revision < 17
                    || r.revision > shell::EVALUATOR_REVISION
                    || (t.path_lookup.is_some() && r.revision < 19)
                    || r.results.len() != t.cases.len()
                {
                    return Err(
                        "Saved environment result has a different task, source or revision".into(),
                    );
                }
                for row in &r.results {
                    if !outcome_valid(&row.parent)
                        || row.child.as_ref().is_some_and(|o| !outcome_valid(o))
                        || row.bindings.as_ref().is_some_and(|s| !snapshot_valid(s))
                        || (row.child.is_some() && row.bindings.is_none())
                        || (row.parent.error.is_some() && row.bindings.is_some())
                        || row.diagnostic.as_ref().is_some_and(|s| !text(s, 4096))
                        || !path_lookup::metadata_valid(t,row)
                    {
                        return Err("Invalid saved environment observation".into());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn solved(&self) -> bool {
        let r = checks(&self.definition.assignment, &self.assignment);
        !r.is_empty() && r.iter().all(|(ok, _)| *ok)
    }
}
fn result_check(task:&Task, case: &Case, row: &CaseRun) -> Result<(), String> {
    if !path_lookup::metadata_valid(task,row){return Err("Worker lookup metadata is missing or inconsistent; run this draft again.".into());}
    if let Some(e) = &row.parent.error {
        return Err(format!("Setup could not run: {e}"));
    }
    if !case.input.accepts(&row.parent) {
        return Err(if row.parent.status != case.input.status {
            "The setup finished with the wrong exit status."
        } else if row.parent.files != case.input.after {
            "The setup changed a protected simulated file."
        } else {
            "The setup printed unexpected text. The supplied child produces the report."
        }
        .into());
    }
    let Some(snapshot) = &row.bindings else {
        return Err(row
            .diagnostic
            .clone()
            .unwrap_or_else(|| "No parent environment observation was produced.".into()));
    };
    if snapshot.child != child_map(&snapshot.parent) {
        return Err("The child settings do not match the parent's export attributes.".into());
    }
    for (n, wanted) in &case.parent {
        let actual = snapshot.parent.get(n).cloned().unwrap_or_else(missing);
        if task.path_lookup.is_some() && n=="PATH" && (!actual.exported || actual.value.is_none()) {
            return Err("The worker needs a defined exported PATH. A private setup value is not inherited.".into());
        }
        if actual.value != wanted.value && !(task.path_lookup.is_some() && n=="PATH") {
            return Err(format!(
                "{n} has the wrong parent value. Use this test's supplied inputs; an unset name is different from an empty value."
            ));
        }
        if actual.exported != wanted.exported {
            return Err(if wanted.exported {
                format!(
                    "{n} exists in the setup, but the child does not receive it. Check which values are exported."
                )
            } else {
                format!("Keep {n} available in the setup without passing it to the child.")
            });
        }
    }
    let same_child=if task.path_lookup.is_some(){
        snapshot.child.iter().filter(|(k,_)|k.as_str()!="PATH").eq(case.child.env.iter().filter(|(k,_)|k.as_str()!="PATH"))
    }else{snapshot.child==case.child.env};
    if !same_child {
        let missing: Vec<_> = case
            .child
            .env
            .keys()
            .filter(|n| !(task.path_lookup.is_some() && n.as_str()=="PATH") && !snapshot.child.contains_key(*n))
            .cloned()
            .collect();
        let extra: Vec<_> = snapshot
            .child
            .keys()
            .filter(|n| !(task.path_lookup.is_some() && n.as_str()=="PATH") && !case.child.env.contains_key(*n))
            .cloned()
            .collect();
        return Err(if !missing.is_empty() {
            format!("Child settings are missing: {}.", missing.join(", "))
        } else if !extra.is_empty() {
            format!(
                "Unexpected settings passed to the child: {}.",
                extra.join(", ")
            )
        } else {
            "A child's inherited value changed. Preserve the supplied settings and use this test's inputs.".into()
        });
    }
    if let Some(plan)=&task.path_lookup{
        plan.check(&case.input.name,row.path_lookup.as_ref().ok_or("Worker lookup has not run")?)?;
    }
    let Some(child) = &row.child else {
        return Err(row
            .diagnostic
            .clone()
            .unwrap_or_else(|| "The supplied child did not run.".into()));
    };
    if !case.child.accepts(child) {
        return Err(if let Some(e) = &child.error {
            format!("The supplied child could not run with these settings: {e}")
        } else {
            "Child output, exit status or virtual files differ from the required result. Inspect the actual child report.".into()
        });
    }
    Ok(())
}
pub fn checks(task: &Task, draft: &Draft) -> Vec<(bool, String)> {
    if task.validate().is_err() || !text(&draft.source, MAX_SOURCE) {
        return vec![(
            false,
            "Task or draft exceeds the supported environment exercise scope.".into(),
        )];
    }
    let Some(r) = &draft.result else {
        return vec![(
            false,
            "Not run. Run the setup and supplied child to check the assignment.".into(),
        )];
    };
    if r.revision != shell::EVALUATOR_REVISION
        || r.source != draft.source
        || r.task != *task
        || r.results.len() != task.cases.len()
    {
        return vec![(
            false,
            "Earlier or changed result. Run this draft again to check its current behavior.".into(),
        )];
    }
    if !r.verified() {
        return vec![(false,"Saved result awaits verification. Re-run this saved source before counting it as current success.".into())];
    }
    task.cases
        .iter()
        .zip(&r.results)
        .map(|(c, r)| match result_check(task,c, r) {
            Ok(()) => (
                true,
                if task.path_lookup.is_some(){format!("{}: worker lookup, settings and report match.",c.input.name)}else{format!("{}: parent and child settings match.", c.input.name)},
            ),
            Err(e) => (false, format!("{}: {e}", c.input.name)),
        })
        .collect()
}
fn failed(case: &shell::Case, error: String) -> shell::Outcome {
    shell::Outcome {
        stdout: String::new(),
        stderr: String::new(),
        status: 2,
        files: case.files.clone(),
        error: Some(error),
    }
}
/// Cooperative two-phase runner. Retains actual observations, never a pass flag.
pub struct Job {
    task: Task,
    source: String,
    index: usize,
    vm: Option<shell::Vm>,
    parent: Option<(shell::Outcome, shell::EnvironmentSnapshot)>,
    path_lookup: Option<path_lookup::Run>,
    results: Vec<CaseRun>,
    units: usize,
    stopped: bool,
}
impl Job {
    pub fn new(task: &Task, source: &str) -> Result<Self, String> {
        task.validate()?;
        if !text(source, MAX_SOURCE) {
            return Err("Setup scripts are limited to 8 KiB without NUL characters".into());
        }
        Ok(Self {
            task: task.clone(),
            source: source.into(),
            index: 0,
            vm: None,
            parent: None,
            path_lookup: None,
            results: Vec::new(),
            units: 0,
            stopped: false,
        })
    }
    pub fn done(&self) -> bool {
        self.stopped || self.index == self.task.cases.len()
    }
    pub fn stop(&mut self) {
        if let Some(vm) = &mut self.vm {
            vm.stop();
        }
        self.vm = None;
        self.parent = None;
        self.path_lookup = None;
        self.stopped = true;
    }
    fn finish_case(&mut self, row: CaseRun) {
        self.results.push(row);
        self.index += 1;
        self.vm = None;
        self.parent = None;
        self.path_lookup = None;
    }
    pub fn advance(&mut self, budget: usize) {
        if self.done() {
            return;
        }
        let units = budget.clamp(1, 64);
        if self.units > MAX_JOB_UNITS - units {
            self.stop();
            return;
        }
        self.units += units;
        let c = &self.task.cases[self.index];
        if self.vm.is_none() {
            let (source, input) = if let Some((parent, bindings)) = &self.parent {
                let mut input = c.child.clone();
                input.env = bindings.child.clone();
                input.files = parent.files.clone();
                (&self.task.child_source, input)
            } else {
                (&self.source, c.input.clone())
            };
            match shell::Vm::new(source, &input) {
                Ok(vm) => self.vm = Some(vm),
                Err(e) => {
                    let outcome = failed(&input, e);
                    let row = if let Some((parent, bindings)) = self.parent.take() {
                        CaseRun {
                            parent,
                            bindings: Some(bindings),
                            child: Some(outcome),
                            diagnostic: None,
                            path_lookup: self.path_lookup.take(),
                        }
                    } else {
                        CaseRun {
                            parent: outcome,
                            bindings: None,
                            child: None,
                            diagnostic: None,
                            path_lookup: None,
                        }
                    };
                    self.finish_case(row);
                    return;
                }
            }
        }
        let vm = self.vm.as_mut().unwrap();
        vm.tick(units);
        if !vm.done() {
            return;
        }
        let vm = self.vm.take().unwrap();
        if let Some((parent, bindings)) = self.parent.take() {
            let path_lookup=self.path_lookup.take();
            self.finish_case(CaseRun {
                parent,
                bindings: Some(bindings),
                child: Some(vm.outcome),
                diagnostic: None,
                path_lookup,
            });
        } else {
            if vm.outcome.error.is_some() {
                self.finish_case(CaseRun {
                    parent: vm.outcome,
                    bindings: None,
                    child: None,
                    diagnostic: None,
                    path_lookup: None,
                });
                return;
            }
            match vm.environment_snapshot() {
                Ok(bindings) if snapshot_valid(&bindings) => {
                    if let Some(plan)=&self.task.path_lookup{
                        if self.units>MAX_JOB_UNITS-path_lookup::MAX_CHARGE {self.stop();return;}
                        let observed=plan.observe(&bindings);
                        self.units+=path_lookup::charge(&bindings,&observed);
                        self.path_lookup=Some(observed);
                    }
                    self.parent=Some((vm.outcome,bindings));
                },
                Ok(_) => self.finish_case(CaseRun {
                    parent: vm.outcome, bindings: None, child: None,
                    diagnostic: Some("Environment exercise observation limit: use at most 128 ordinary names and 16 KiB of values.".into()),
                    path_lookup: None,
                }),
                Err(e) => self.finish_case(CaseRun {
                    parent: vm.outcome,
                    bindings: None,
                    child: None,
                    diagnostic: Some(e),
                    path_lookup: None,
                }),
            }
        }
    }
    pub fn record(self) -> Option<Record> {
        if !self.stopped && self.index == self.task.cases.len() {
            let mut record = Record {
                revision: shell::EVALUATOR_REVISION,
                task: self.task,
                source: self.source,
                results: self.results,
                verification: None,
            };
            record.verification = record.content_id();
            if record.verification.is_some() {
                Some(record)
            } else {
                None
            }
        } else {
            None
        }
    }
}
pub fn run(task: &Task, source: &str) -> Result<Record, String> {
    let mut job = Job::new(task, source)?;
    while !job.done() {
        job.advance(64);
    }
    job.record().ok_or_else(|| {
        "Environment exercise stopped at its whole-job execution limit; no completion awarded."
            .into()
    })
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::Environment { simulation },
        crate::engine::Goal::Environment { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Environment source/state mismatch".into());
    };
    definition.validate()?;
    if definition.id != s.id
        || definition.assignment.id != s.id
        || definition.assignment.instructions != s.objective
        || **simulation != Session::new((**definition).clone())
    {
        return Err(
            "Environment identity, requirements and untouched initial work must match".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let definition = source(s)?;
    let crate::engine::State::Environment { simulation } = &a.state else {
        return Err("Saved environment work belongs to another activity".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible environment attempt metadata; previous work retained".into());
    }
    simulation.validate(definition)
}
