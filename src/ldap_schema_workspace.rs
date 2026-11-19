//! Draft-isolated schema tasks graded by actual registered schema and entry data.
//! No host commands, server, files, credentials or network are used here.
use crate::{ldap_schema as model, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_DRAFT: usize = 8192;
fn key(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 64
        && v.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(v: &str, n: usize) -> bool {
    v.len() <= n
        && v.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn prose(v: &str, n: usize) -> bool {
    !v.trim().is_empty() && text(v, n)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Add,
    Replace,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub reason: String,
    pub operation: Operation,
    pub entry: model::EntrySource,
    pub accept: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,model::EntrySource,4>")]
    pub initial: Vec<model::EntrySource>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Probe,16>")]
    pub probes: Vec<Probe>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub schema_name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub oid_branch: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub reference: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,4>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub label: String,
    pub expected_accept: bool,
    pub accepted: bool,
    pub passed: bool,
    pub feedback: String,
    pub error: Option<model::Diagnostic>,
    pub before: Vec<model::EntrySource>,
    pub after: Vec<model::EntrySource>,
    pub modules: Vec<model::Module>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub label: String,
    pub passed: bool,
    pub runs: Vec<Run>,
    pub feedback: String,
}
impl Task {
    fn prepare(&self, draft: &str) -> Result<model::Directory, String> {
        self.shape()?;
        if !text(draft, MAX_DRAFT) {
            return Err(
                "Keep the schema document within 8192 bytes and ordinary editor characters.".into(),
            );
        }
        let mut d = model::Directory::new(model::Registry::teaching_base());
        let load = d.load(draft);
        if let Some(error) = load.error {
            return Err(format!(
                "{} schema Add(s) applied before failure. {error}",
                load.applied.len()
            ));
        }
        if d.modules().len() != 1 || !d.modules()[0].name.eq_ignore_ascii_case(&self.schema_name) {
            return Err(format!(
                "Add one schema entry named {}. Leave the supplied base schema unchanged.",
                self.schema_name
            ));
        }
        let prefix = format!("{}.", self.oid_branch);
        for module in d.modules() {
            for raw in &module.attributes {
                if !model::attribute_type(raw)
                    .map_err(|e| e.to_string())?
                    .oid
                    .starts_with(&prefix)
                {
                    return Err(format!(
                        "Use new attribute OIDs below the supplied documentation branch {}.",
                        self.oid_branch
                    ));
                }
            }
            for raw in &module.classes {
                if !model::object_class(raw)
                    .map_err(|e| e.to_string())?
                    .oid
                    .starts_with(&prefix)
                {
                    return Err(format!(
                        "Use new class OIDs below the supplied documentation branch {}.",
                        self.oid_branch
                    ));
                }
            }
        }
        Ok(d)
    }
    fn run_prepared(
        &self,
        prepared: &model::Directory,
        case: usize,
        probe: usize,
    ) -> Result<Run, String> {
        let c = self.cases.get(case).ok_or("Choose an existing sample.")?;
        let p = c
            .probes
            .get(probe)
            .ok_or("Choose an existing entry request.")?;
        let mut directory = prepared.clone();
        for initial in &c.initial {
            directory
                .add(initial)
                .map_err(|e| format!("The schema rejects an existing supplied entry: {e}"))?;
        }
        let before_directory = directory.clone();
        let before = directory.source();
        let result = match p.operation {
            Operation::Add => directory.add(&p.entry),
            Operation::Replace => directory.replace(&p.entry),
        };
        let error = result.err();
        let accepted = error.is_none();
        let passed = if p.accept {
            accepted && directory != before_directory
        } else {
            error
                .as_ref()
                .is_some_and(|e| e.category == model::Category::Violation)
                && directory == before_directory
        };
        let feedback = match (&error, p.accept) {
            (None, true) => {
                "Accepted: the actual stored entry changed and met the registered schema.".into()
            }
            (None, false) => format!(
                "Accepted unexpectedly. {} Review the schema constraint that should reject this data.",
                p.reason
            ),
            (Some(e), true) => format!("This valid sample should be accepted. {e} {}", p.reason),
            (Some(e), false) if e.category == model::Category::Violation => {
                format!("Rejected without changes: {e} {}", p.reason)
            }
            (Some(e), false) => format!(
                "This is not the required schema rejection: {e} Unsupported or malformed behavior cannot earn credit."
            ),
        };
        Ok(Run {
            label: p.label.clone(),
            expected_accept: p.accept,
            accepted,
            passed,
            feedback,
            error,
            before,
            after: directory.source(),
            modules: directory.modules().to_vec(),
        })
    }
    pub fn run(&self, draft: &str, case: usize, probe: usize) -> Result<Run, String> {
        self.run_prepared(&self.prepare(draft)?, case, probe)
    }
    pub fn reports(&self, draft: &str) -> Result<Vec<Report>, String> {
        let prepared = self.prepare(draft)?;
        let mut reports = Vec::new();
        for (case, c) in self.cases.iter().enumerate() {
            let runs = (0..c.probes.len())
                .map(|probe| self.run_prepared(&prepared, case, probe))
                .collect::<Result<Vec<_>, _>>()?;
            let matched = runs.iter().filter(|r| r.passed).count();
            let failed = runs
                .iter()
                .filter(|r| !r.passed)
                .map(|r| format!("{}: {}", r.label, r.feedback))
                .collect::<Vec<_>>();
            reports.push(Report {
                label: c.label.clone(),
                passed: matched == runs.len(),
                feedback: if failed.is_empty() {
                    format!("All {matched} actual entry requests matched.")
                } else {
                    format!(
                        "{matched}/{} requests matched. {}",
                        runs.len(),
                        failed.join(" ")
                    )
                },
                runs,
            });
        }
        Ok(reports)
    }
    fn shape(&self) -> Result<(), String> {
        if !key(&self.id)
            || !prose(&self.instructions, 2048)
            || !key(&self.schema_name)
            || self.schema_name.len() > 32
            || !model::numeric_oid(&self.oid_branch)
            || !self.oid_branch.starts_with("1.3.6.1.4.1.32473.")
            || self.oid_branch.len() > 112
            || !text(&self.starter, MAX_DRAFT)
            || !prose(&self.reference, MAX_DRAFT)
            || !(2..=4).contains(&self.cases.len())
        {
            return Err("Use a bounded schema task, a documentation OID branch and two to four distinct sample sets.".into());
        }
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            if !key(&c.id)
                || !ids.insert(&c.id)
                || !prose(&c.label, 120)
                || c.initial.len() > 4
                || !(2..=16).contains(&c.probes.len())
                || !c.probes.iter().any(|p| p.accept)
                || !c.probes.iter().any(|p| !p.accept)
            {
                return Err("Each schema sample needs a distinct ID, bounded data and both valid and invalid requests.".into());
            }
            for entry in &c.initial {
                entry.validate_shape().map_err(|e| e.to_string())?;
            }
            let mut labels = BTreeSet::new();
            for p in &c.probes {
                p.entry.validate_shape().map_err(|e| e.to_string())?;
                if !prose(&p.label, 120) || !prose(&p.reason, 256) || !labels.insert(&p.label) {
                    return Err("Entry requests need distinct labels and a concise explanation of their required behavior.".into());
                }
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if !self.reports(&self.reference)?.iter().all(|r| r.passed) {
            return Err(
                "The taught schema does not reach every actual valid/invalid sample outcome."
                    .into(),
            );
        }
        if self
            .reports(&self.starter)
            .is_ok_and(|v| v.iter().all(|r| r.passed))
        {
            return Err("The assignment starter must leave meaningful schema work.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.initial.id == self.example.id
            || self.initial.reference == self.example.reference
            || self.initial.cases == self.example.cases
        {
            return Err(
                "Use version 1 and separate assignment/practice data, drafts and context IDs."
                    .into(),
            );
        }
        self.initial.validate()?;
        self.example.validate()
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = s.bound(self) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.initial)
    }
}
fn checked_draft<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] String);
    Option::<Text>::deserialize(d).map(|t| t.map(|s| s.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub draft: String,
    pub selected_case: u8,
    pub selected_probe: u8,
    pub checked: bool,
    #[serde(deserialize_with = "checked_draft")]
    pub checked_draft: Option<String>,
    #[serde(skip)]
    pub output: Option<Run>,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            selected_case: 0,
            selected_probe: 0,
            checked: false,
            checked_draft: None,
            output: None,
        }
    }
    fn shape(&self) -> Result<(), String> {
        if !text(&self.draft, MAX_DRAFT)
            || self
                .checked_draft
                .as_ref()
                .is_some_and(|v| !text(v, MAX_DRAFT))
            || self.checked != self.checked_draft.is_some()
            || self.selected_case >= 4
            || self.selected_probe >= 16
        {
            return Err(
                "Saved schema work has invalid draft bounds, selection or check markers.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        t.shape()?;
        self.shape()?;
        if t.cases
            .get(usize::from(self.selected_case))
            .and_then(|c| c.probes.get(usize::from(self.selected_probe)))
            .is_none()
        {
            return Err("Choose an existing schema sample and entry request.".into());
        }
        Ok(())
    }
    pub fn edit(&mut self, draft: String) -> Result<(), String> {
        self.checked = false;
        self.checked_draft = None;
        self.output = None;
        if !text(&draft, MAX_DRAFT) {
            return Err("The schema draft exceeds 8192 bytes or contains unsupported editor controls; the previous draft is retained.".into());
        }
        self.draft = draft;
        Ok(())
    }
    pub fn select(&mut self, t: &Task, case: usize, probe: usize) -> Result<(), String> {
        if t.cases
            .get(case)
            .and_then(|c| c.probes.get(probe))
            .is_none()
        {
            return Err("Choose an existing schema sample and entry request.".into());
        }
        self.selected_case = case as u8;
        self.selected_probe = probe as u8;
        self.output = None;
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.output = None;
        self.output = Some(t.run(
            &self.draft,
            usize::from(self.selected_case),
            usize::from(self.selected_probe),
        )?);
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        self.checked = true;
        self.checked_draft = Some(self.draft.clone());
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = t.shape() {
            return vec![(false, e)];
        }
        if !self.checked || self.checked_draft.as_deref() != Some(&self.draft) {
            return t
                .cases
                .iter()
                .map(|c| (false, format!("{}: unchecked", c.label)))
                .collect();
        }
        match t.reports(&self.draft) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| (r.passed, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub context: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            context: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !key(&self.context) {
            return Err("Saved schema work has an invalid version or identity.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.context != d.initial.id {
            return Err(
                "Saved schema work belongs to a different context; original retained.".into(),
            );
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Scope::new(&d.initial);
    }
}
pub const PREFIX: &str = "LDAP SCHEMA WORK\n";
pub const HELP: &str = "Use the native schema editor and sample-entry controls. This bounded model supports documented schema Add, typed entries and field-specific checks. It does not execute LDAP commands, files, servers or network traffic on the host.";
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String, String> {
        self.bound(d)?;
        let value = raw.strip_prefix(PREFIX).ok_or(HELP)?;
        if value.len() > 64 * 1024 {
            return Err("Schema work envelope exceeds 64 KiB.".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Work {
            #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
            schema: String,
        }
        let work: Work =
            serde_json::from_str(value).map_err(|e| format!("Invalid schema draft: {e}"))?;
        let mut next = self.assignment.clone();
        next.edit(work.schema)?;
        next.check(&d.initial)?;
        let rows = next.checks(&d.initial);
        self.assignment = next;
        Ok(format!(
            "Schema sample sets: {}/{} match",
            rows.iter().filter(|r| r.0).count(),
            rows.len()
        ))
    }
}
fn admitted(s: &crate::pack::Scenario) -> Result<(&Definition, &Session), String> {
    let (
        crate::engine::Goal::LdapSchema { definition },
        crate::engine::State::LdapSchema { simulation },
    ) = (&s.goal, &s.initial)
    else {
        return Err("Schema source/state mismatch.".into());
    };
    let fresh = Session {
        version: 1,
        context: definition.initial.id.clone(),
        view: View::Introduction,
        assignment: Scope::new(&definition.initial),
        practice: Scope::new(&definition.example),
    };
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != fresh
    {
        return Err("Schema source identity, objective and fresh session must match.".into());
    }
    Ok((definition, simulation))
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (d, _) = admitted(s)?;
    d.validate()?;
    Ok(d)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let (d, _) = admitted(s)?;
    let crate::engine::State::LdapSchema { simulation } = &a.state else {
        return Err("Schema work belongs to a different workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible schema work metadata; original retained.".into());
    }
    simulation.bound(d)
}
