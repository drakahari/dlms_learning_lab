//! Source-held editor tasks and isolated drafts over bounded PAM control flow.
use crate::{pam_control as core, script_teaching::View};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
pub const PREFIX: &str = "PAM STACK\n";
// Four bounded8KiB raw draft/tested strings may each double under JSON escaping.
pub const MAX_SESSION: usize = 72 * 1024;
pub const MAX_DEFINITION: usize = 64 * 1024;
pub const MAX_TICKS: usize = 16;
pub const HELP: &str = "Edit the virtual auth stack, then Test stack. Each sample supplies a synthetic pam_unix.so result; this activity neither checks passwords nor authenticates anyone. Supported: auth rules with required/requisite/sufficient/optional and pam_unix.so, pam_permit.so or pam_deny.so, without arguments. No host files, modules, accounts or commands are accessed. Other PAM groups, includes, bracket controls and module options are unsupported and produce an explicit diagnostic.";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(s: &str) -> bool {
    s.len() <= core::MAX_TEXT
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn optional_text<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Body(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] String);
    Option::<Body>::deserialize(d).map(|o| o.map(|x| x.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub facts: core::Facts,
    pub accepted: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !key(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1800
            || !text(&self.instructions)
            || !text(&self.starter)
            || !(3..=8).contains(&self.cases.len())
        {
            return Err("PAM task needs a bounded identity, clear instructions, starter and three to eight samples.".into());
        }
        core::parse(&self.starter)
            .map_err(|e| format!("Starter must use supported unfinished rules: {e}"))?;
        let mut ids = BTreeSet::new();
        let mut facts = BTreeSet::new();
        for c in &self.cases {
            if !key(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || !facts.insert(c.facts.unix_result.label())
            {
                return Err("Authentication samples need unique bounded identities and distinct supplied module results.".into());
            }
        }
        if !self.cases.iter().any(|c| c.accepted) || !self.cases.iter().any(|c| !c.accepted) {
            return Err("An authentication task must test acceptance and rejection; denying everyone is not a solution.".into());
        }
        Ok(())
    }
    pub fn run(&self, draft: &str) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, draft)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.rows);
            }
        }
        Err("Authentication sample limit reached.".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub case: Case,
    pub actual: Option<core::Outcome>,
    pub diagnostic: Option<String>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.diagnostic.is_none()
            && self
                .actual
                .as_ref()
                .is_some_and(|o| o.accepted() == self.case.accepted)
    }
    pub fn feedback(&self) -> String {
        let Some(actual) = &self.actual else {
            return self
                .diagnostic
                .clone()
                .unwrap_or_else(|| "This sample has not run.".into());
        };
        let expected = if self.case.accepted {
            "Accepted"
        } else {
            "Rejected"
        };
        let observed = if actual.accepted() {
            "Accepted"
        } else {
            "Rejected"
        };
        let base = format!(
            "Expected {expected}; actual {observed} ({}).",
            actual.result.label()
        );
        if self.passed() {
            return base;
        }
        let unix = actual.trace.iter().find(|s| s.module == core::Module::Unix);
        if actual.accepted() {
            if unix.is_none() {
                return format!(
                    "{base} The stack finished before checking the supplied password result. Inspect an early successful rule and its control field."
                );
            }
            if unix.is_some_and(|u| u.returned != core::Code::Success) {
                return format!(
                    "{base} The password check did not succeed, but its result contributed no failure. A later success allowed this sample; inspect the password rule's control field."
                );
            }
        }
        format!(
            "{base} Read the visited rules below: a required failure remains a failure even after a later success. A requisite failure stops immediately."
        )
    }
}
pub struct Run {
    task: Task,
    draft: String,
    parsed: Result<core::Stack, core::Error>,
    index: usize,
    rows: Vec<CaseResult>,
    pending: Option<CaseResult>,
    cancelled: bool,
}
impl Run {
    pub fn new(task: &Task, draft: &str) -> Result<Self, String> {
        task.validate()?;
        if !text(draft) {
            return Err("Keep the virtual stack within 8KiB without control characters.".into());
        }
        Ok(Self {
            task: task.clone(),
            draft: draft.into(),
            parsed: core::parse(draft),
            index: 0,
            rows: vec![],
            pending: None,
            cancelled: false,
        })
    }
    pub fn done(&self) -> bool {
        !self.cancelled && self.index == self.task.cases.len()
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.rows)
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.rows.clear();
        self.pending = None;
    }
    /// Evaluate one bounded sample, then yield before publishing it. Stop can
    /// discard a pending result before the next sample or completion is committed.
    pub fn tick(&mut self) -> bool {
        if self.cancelled {
            return false;
        }
        if self.done() {
            return true;
        }
        if let Some(row) = self.pending.take() {
            self.rows.push(row);
            self.index += 1;
            return self.done();
        }
        let case = self.task.cases[self.index].clone();
        let (actual, diagnostic) = match &self.parsed {
            Ok(stack) => (Some(core::evaluate(stack, case.facts)), None),
            Err(e) => (None, Some(e.to_string())),
        };
        self.pending = Some(CaseResult {
            case,
            actual,
            diagnostic,
        });
        false
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub example_worked: String,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.starter == self.example.starter
            || !text(&self.example_worked)
            || self.initial.cases.iter().any(|a| {
                self.example
                    .cases
                    .iter()
                    .any(|b| a.id == b.id || a.label == b.label)
            })
        {
            return Err("Use separate practice identities, samples and starter rules.".into());
        }
        for t in [&self.initial, &self.example] {
            if t.run(&t.starter)?.iter().all(CaseResult::passed) {
                return Err(
                    "Keep meaningful unfinished work in each authentication starter.".into(),
                );
            }
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked example must satisfy every practice sample.".into());
        }
        Ok(())
    }
    pub fn bound(&self, session: &Session) -> Result<(), String> {
        self.validate()?;
        session.validate()?;
        if session.origin != self.initial.id {
            return Err(
                "This authentication draft belongs to another lesson; previous work retained."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn checks(&self, session: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(session) {
            return vec![(false, e)];
        }
        if session.assignment.tested.as_ref() != Some(&session.assignment.draft) {
            return vec![(
                false,
                "Not checked · test the current authentication stack against the supplied samples."
                    .into(),
            )];
        }
        match self.initial.run(&session.assignment.draft) {
            Ok(rows) => rows
                .iter()
                .map(|r| (r.passed(), format!("{} · {}", r.case.label, r.feedback())))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
    pub fn solved(&self, session: &Session) -> bool {
        let rows = self.checks(session);
        !rows.is_empty() && rows.iter().all(|r| r.0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub draft: String,
    #[serde(deserialize_with = "optional_text")]
    pub tested: Option<String>,
}
impl Scope {
    pub fn edit(&mut self, draft: &str) -> Result<(), String> {
        if !text(draft) {
            return Err(
                "Use at most 8KiB without control characters; previous draft retained.".into(),
            );
        }
        if self.draft != draft {
            self.draft = draft.into();
            self.tested = None;
        }
        Ok(())
    }
    pub fn begin(&mut self, task: &Task) -> Result<Run, String> {
        self.tested = None;
        Run::new(task, &self.draft)
    }
    pub fn finish(&mut self, task: &Task, run: &Run) -> Result<(), String> {
        if !run.done() || run.task != *task || run.draft != self.draft {
            return Err(
                "Stopped, unfinished or changed checks cannot award current credit.".into(),
            );
        }
        self.tested = Some(self.draft.clone());
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
        Err("Authentication sample limit reached.".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        let scope = |t: &Task| Scope {
            draft: t.starter.clone(),
            tested: None,
        };
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: scope(&d.initial),
            practice: scope(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.origin)
            || [&self.assignment, &self.practice]
                .iter()
                .any(|s| !text(&s.draft) || s.tested.as_ref().is_some_and(|t| !text(t)))
        {
            return Err("Invalid authentication progress version, identity or text bounds.".into());
        }
        Ok(())
    }
    pub fn from_json(raw: &str, d: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Authentication progress exceeds72KiB.".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid authentication progress: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native editor; this is internal data, not a shell command.")?;
        if raw.len() > core::MAX_TEXT * 2 {
            return Err("Authentication reference exceeds its byte limit.".into());
        }
        #[derive(Deserialize)]
        struct Body(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] String);
        let Body(draft) = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid authentication reference: {e}"))?;
        self.assignment.edit(&draft)?;
        let rows = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Authentication samples checked: {}/{} passed",
            rows.iter().filter(|r| r.passed()).count(),
            rows.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::PamStack { simulation },
        crate::engine::Goal::PamStack { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Authentication source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Authentication source identity, task and supplied drafts must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::PamStack { simulation } = &a.state else {
        return Err("Authentication work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible authentication attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}
