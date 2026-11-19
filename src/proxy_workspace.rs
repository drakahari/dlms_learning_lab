//! Separate source-bound proxy-policy editor practice and assignment.
use crate::{
    proxy_policy::{self as model, Action as DecisionAction, Policy, Request, World},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
fn valid_id(s: &str, limit: usize) -> bool {
    !s.is_empty()
        && s.len() <= limit
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn draft_text(s: &str) -> Result<()> {
    if s.len() > 32768
        || s.chars().count() > 8192
        || s.chars()
            .any(|c| c.is_control() && !['\n', '\r', '\t'].contains(&c))
    {
        Err("Keep the editor within 8192 displayed characters and free of nonprinting control characters.".into())
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub request: Request,
    pub expected: DecisionAction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub initial_config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub required_policy: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub expected: DecisionAction,
    pub actual: model::Decision,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolicyResult {
    pub label: String,
    pub counterexample: Option<model::Counterexample>,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub configuration_error: Option<String>,
    pub cases: Vec<CaseResult>,
    pub policies: Vec<PolicyResult>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.configuration_error.is_none()
            && !self.cases.is_empty()
            && self.cases.iter().all(|c| c.passed)
            && self.policies.len() == 2
            && self.policies.iter().all(|c| c.passed)
    }
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !valid_id(&self.id, 80)
            || self.cases.len() < 2
            || self.cases.len() > 8
        {
            return Err(
                "Proxy tasks need version1, a stable identifier and2–8 request cases.".into(),
            );
        }
        Policy::parse(&self.initial_config).map_err(|e| e.to_string())?;
        let required = Policy::parse(&self.required_policy).map_err(|e| e.to_string())?;
        required.compare(&required).map_err(|e| e.to_string())?;
        let mut ids = BTreeSet::new();
        let mut outcomes = BTreeSet::new();
        for c in &self.cases {
            if !valid_id(&c.id, 64)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
            {
                return Err(
                    "Request cases need distinct stable identifiers and readable labels.".into(),
                );
            }
            let d = required.evaluate(&c.request).map_err(|e| e.to_string())?;
            if d.action != c.expected {
                return Err(
                    "A request's required outcome disagrees with the stated target policy.".into(),
                );
            }
            outcomes.insert(matches!(c.expected, DecisionAction::Allow));
        }
        if outcomes.len() != 2 {
            return Err("Proxy tasks must include both allowed and refused requests.".into());
        }
        let initial = Policy::parse(&self.initial_config).map_err(|e| e.to_string())?;
        if initial
            .compare(&required)
            .map_err(|e| e.to_string())?
            .is_none()
        {
            return Err("The independent starter must retain a real policy problem.".into());
        }
        Ok(())
    }
    pub fn report(&self, d: &Draft) -> Result<Report> {
        self.validate()?;
        d.validate(self)?;
        let mut report = Report {
            configuration_error: None,
            cases: Vec::new(),
            policies: Vec::new(),
        };
        if d.editor != d.world.saved() {
            report.configuration_error=Some("The editor has unsaved changes. Save the draft, validate it and apply the saved policy before checking the handover.".into());
            return Ok(report);
        }
        let saved = match Policy::parse(d.world.saved()) {
            Ok(p) => p,
            Err(e) => {
                report.configuration_error = Some(format!("Saved configuration: {e}"));
                return Ok(report);
            }
        };
        let applied = Policy::parse(d.world.applied()).map_err(|e| e.to_string())?;
        let required = Policy::parse(&self.required_policy).map_err(|e| e.to_string())?;
        for c in &self.cases {
            let actual = applied.evaluate(&c.request).map_err(|e| e.to_string())?;
            let passed = actual.action == c.expected;
            report.cases.push(CaseResult {
                id: c.id.clone(),
                label: c.label.clone(),
                expected: c.expected,
                actual,
                passed,
            });
        }
        for (label, policy) in [("Applied policy", applied), ("Saved policy", saved)] {
            match policy.compare(&required) {
                Ok(counterexample) => report.policies.push(PolicyResult {
                    label: label.into(),
                    passed: counterexample.is_none(),
                    counterexample,
                }),
                Err(e) => {
                    report.configuration_error = Some(format!("{label}: {e}"));
                    return Ok(report);
                }
            }
        }
        Ok(report)
    }
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.cases == self.example.cases
            || self.initial.required_policy == self.example.required_policy
        {
            return Err("Practice must use its own supplied requests and policy, separate from the assignment.".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = s.validate(self) {
            return vec![(false, e)];
        }
        let (task, d) = if practice {
            (&self.example, &s.practice)
        } else {
            (&self.initial, &s.assignment)
        };
        if !d.checked() {
            return vec![(false,"Not checked since editing. Save and apply the configuration, then choose Check policy.".into())];
        }
        match task.report(d) {
            Err(e) => vec![(false, e)],
            Ok(r) => {
                if let Some(e) = r.configuration_error {
                    return vec![(false, e)];
                }
                let mut rows = r
                    .cases
                    .into_iter()
                    .map(|c| {
                        (
                            c.passed,
                            format!(
                                "{}: required {:?}, actual {:?}. {}",
                                c.label, c.expected, c.actual.action, c.actual.explanation
                            ),
                        )
                    })
                    .collect::<Vec<_>>();
                rows.extend(r.policies.into_iter().map(|p|{let why=p.counterexample.map_or_else(||"Every supported request region matches the requirements.".into(),|c|format!("For client {}, method {}, port {}, required {:?} but actual {:?}. Recheck rule order, subnet scope and port/method tests.",c.request.client,c.request.method,c.request.destination_port,c.required,c.actual));(p.passed,format!("{}: {why}",p.label))}));
                rows
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    pub world: World,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub task_id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    pub world: World,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub selected_case: String,
    pub checked_stamp: Option<Stamp>,
}
impl Draft {
    pub fn new(task: &Task) -> Result<Self> {
        task.validate()?;
        Ok(Self {
            task_id: task.id.clone(),
            editor: task.initial_config.clone(),
            world: World::new(&task.initial_config).map_err(|e| e.to_string())?,
            selected_case: task.cases[0].id.clone(),
            checked_stamp: None,
        })
    }
    fn stamp(&self) -> Stamp {
        Stamp {
            editor: self.editor.clone(),
            world: self.world.clone(),
        }
    }
    pub fn checked(&self) -> bool {
        self.checked_stamp
            .as_ref()
            .is_some_and(|s| s == &self.stamp())
    }
    pub fn shape(&self) -> Result<()> {
        if !valid_id(&self.task_id, 80) || !valid_id(&self.selected_case, 64) {
            return Err("Saved proxy work has an invalid activity or request identifier; original retained.".into());
        }
        draft_text(&self.editor)?;
        self.world.validate().map_err(|e| e.to_string())?;
        if let Some(s) = &self.checked_stamp {
            draft_text(&s.editor)?;
            s.world.validate().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.task_id != t.id || !t.cases.iter().any(|c| c.id == self.selected_case) {
            return Err(
                "Saved proxy work belongs to another activity or request set; original retained."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn edit(&mut self, text: String) -> Result<()> {
        draft_text(&text)?;
        if self.editor != text {
            self.editor = text;
            self.checked_stamp = None;
        }
        Ok(())
    }
    pub fn save(&mut self, t: &Task) -> Result<String> {
        self.validate(t)?;
        if self.world.save(&self.editor).map_err(|e| e.to_string())? {
            self.checked_stamp = None;
        }
        Ok("Saved the virtual configuration. Saving does not change the applied policy.".into())
    }
    pub fn validate_saved(&self, t: &Task) -> Result<String> {
        self.validate(t)?;
        self.world.validate_saved().map_err(|e| e.to_string())?;
        Ok("Saved syntax is valid within the supported subset. This does not verify the access requirements or apply changes.".into())
    }
    pub fn apply(&mut self, t: &Task) -> Result<String> {
        self.validate(t)?;
        if self.world.apply_saved().map_err(|e| e.to_string())? {
            self.checked_stamp = None;
        }
        Ok("Applied the saved policy to the simulated proxy. Inspect requests or check the full task.".into())
    }
    pub fn check(&mut self, t: &Task) -> Result<Report> {
        let report = t.report(self)?;
        self.checked_stamp = Some(self.stamp());
        Ok(report)
    }
    pub fn inspect(&self, t: &Task) -> Result<model::Decision> {
        self.validate(t)?;
        let case = t.cases.iter().find(|c| c.id == self.selected_case).unwrap();
        self.world
            .effective_decision(&case.request)
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    pub source: Box<Definition>,
    pub view: View,
    pub practice: Draft,
    pub assignment: Draft,
}
impl Session {
    pub fn new(def: &Definition) -> Result<Self> {
        def.validate()?;
        Ok(Self {
            version: 1,
            source: Box::new(def.clone()),
            view: View::Introduction,
            practice: Draft::new(&def.example)?,
            assignment: Draft::new(&def.initial)?,
        })
    }
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported proxy progress version; original retained.".into());
        }
        self.source.validate()?;
        self.practice.shape()?;
        self.assignment.shape()
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        d.validate()?;
        if *self.source != *d {
            return Err("The supplied proxy task changed; original drafts are retained for recovery rather than silently graded against new requirements.".into());
        }
        self.practice.validate(&d.example)?;
        self.assignment.validate(&d.initial)
    }
}

#[cfg(test)]
mod tests;

mod bindings;
pub use bindings::{Action, HELP, PREFIX, context, relevant, source};
