//! Data-only monitoring tasks and separate browser/practice/assignment drafts.
use crate::{
    script_teaching::View,
    snmp::{
        Oid, Reply, Width,
        monitor::{self, Direction, LinkRule, Outcome, Plan, Stream},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
mod bindings;
pub use bindings::{HELP, context, relevant, source};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    Link,
    Traffic,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fields {
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub description: String,
    pub link_rule: LinkRule,
    pub direction: Direction,
    pub width: Width,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub threshold: String,
}
impl Fields {
    pub fn shape(&self) -> Result<()> {
        if self.description.len() > 256 || self.description.chars().any(char::is_control) {
            return Err("Keep the description entry short and free of control characters.".into());
        }
        if self.threshold.len() > 16 || self.threshold.chars().any(char::is_control) {
            return Err("Keep the warning entry short and free of control characters.".into());
        }
        Ok(())
    }
    fn plan(&self, activity: Activity) -> Result<Plan> {
        self.shape()?;
        if self.description.trim().is_empty() {
            return Err("Enter the interface description shown in the agent table. A blank field does not select a link.".into());
        }
        let p = match activity {
            Activity::Link => Plan::Link {
                description: self.description.clone(),
                rule: self.link_rule,
            },
            Activity::Traffic => {
                if self.threshold.trim().is_empty() {
                    return Err(
                        "Enter the percent of link capacity that should raise a warning.".into(),
                    );
                }
                let threshold_percent = self
                    .threshold
                    .trim()
                    .parse::<u8>()
                    .ok()
                    .filter(|v| (1..=100).contains(v))
                    .ok_or(
                        "Use a whole warning percentage from 1 to 100, without the percent sign.",
                    )?;
                Plan::Traffic {
                    description: self.description.clone(),
                    direction: self.direction,
                    width: self.width,
                    threshold_percent,
                }
            }
        };
        p.validate()?;
        Ok(p)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub stream: Stream,
    pub expected: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub activity: Activity,
    pub initial: Fields,
    pub reference: Fields,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub expected: Outcome,
    pub actual: monitor::Observation,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub configuration_error: Option<String>,
    pub cases: Vec<CaseResult>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.configuration_error.is_none()
            && !self.cases.is_empty()
            && self.cases.iter().all(|c| c.passed)
    }
}
fn identifier(v: &str, max: usize) -> bool {
    !v.is_empty()
        && v.len() <= max
        && v.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && v.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Task {
    pub fn authorized(&self, fields: &Fields) -> Result<()> {
        fields.shape()?;
        let mut protected = fields.clone();
        protected.description = self.initial.description.clone();
        match self.activity {
            Activity::Link => protected.link_rule = self.initial.link_rule,
            Activity::Traffic => {
                protected.direction = self.initial.direction;
                protected.width = self.initial.width;
                protected.threshold = self.initial.threshold.clone();
            }
        }
        if protected != self.initial {
            return Err("The saved configuration changes settings outside this activity; original work retained.".into());
        }
        Ok(())
    }
    pub fn report(&self, fields: &Fields) -> Result<Report> {
        self.authorized(fields)?;
        let plan = match fields.plan(self.activity) {
            Ok(p) => p,
            Err(message) => {
                return Ok(Report {
                    configuration_error: Some(message),
                    cases: vec![],
                });
            }
        };
        let mut cases = Vec::new();
        for c in &self.cases {
            let actual = monitor::observe(&plan, &c.stream)?;
            let passed = actual.outcome == c.expected;
            cases.push(CaseResult {
                id: c.id.clone(),
                label: c.label.clone(),
                expected: c.expected,
                actual,
                passed,
            })
        }
        Ok(Report {
            configuration_error: None,
            cases,
        })
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !identifier(&self.id, 80)
            || self.cases.is_empty()
            || self.cases.len() > 8
        {
            return Err("Monitoring tasks need version 1, a valid identity and 1–8 cases.".into());
        }
        self.initial.shape()?;
        self.authorized(&self.reference)?;
        self.reference.plan(self.activity)?;
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            if !identifier(&c.id, 64)
                || !ids.insert(c.id.as_str())
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
            {
                return Err(
                    "Monitoring cases need unique identities and short readable labels.".into(),
                );
            }
            c.stream.validate()?;
        }
        if !self.cases.iter().any(|c| c.expected == Outcome::Alert)
            || !self.cases.iter().any(|c| c.expected == Outcome::Quiet)
        {
            return Err("Include both alert and quiet cases; unknown-only fixtures cannot demonstrate a working monitor.".into());
        }
        if !self.report(&self.reference)?.passed() {
            return Err(
                "The authored monitoring reference cannot meet every required outcome.".into(),
            );
        }
        if self.report(&self.initial)?.passed() {
            return Err("Start with a genuinely unfinished monitor configuration.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub case: String,
    pub before: bool,
    pub next: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,384>")]
    pub oid: String,
}
impl Query {
    fn new(t: &Task) -> Self {
        Self {
            case: t.cases.first().map(|c| c.id.clone()).unwrap_or_default(),
            before: false,
            next: false,
            oid: "1.3.6.1.2.1.1.3.0".into(),
        }
    }
    fn shape(&self, t: &Task) -> Result<()> {
        if self.oid.len() > 384
            || self.oid.chars().any(char::is_control)
            || !t.cases.iter().any(|c| c.id == self.case)
        {
            return Err("Saved browser input must belong to a supplied case and stay within the numeric-OID field limit.".into());
        }
        Ok(())
    }
    pub fn run(&self, t: &Task) -> Result<Reply> {
        self.shape(t)?;
        let c = t
            .cases
            .iter()
            .find(|c| c.id == self.case)
            .ok_or("Choose a supplied monitoring case.")?;
        let s = if self.before {
            c.stream
                .before
                .as_ref()
                .ok_or("This case has no earlier measurement; choose the current snapshot.")?
        } else {
            &c.stream.after
        };
        let oid = Oid::parse(&self.oid)?;
        let mut r = if self.next {
            s.agent.get_next(&[oid])?
        } else {
            s.agent.get(&[oid])?
        };
        Ok(r.remove(0))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub task_id: String,
    pub fields: Fields,
    pub checked_fields: Option<Fields>,
    pub query: Query,
    pub queried: Option<Query>,
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            fields: t.initial.clone(),
            checked_fields: None,
            query: Query::new(t),
            queried: None,
        }
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        if self.task_id != t.id {
            return Err("Saved monitoring work belongs to another task; original retained.".into());
        }
        t.authorized(&self.fields)?;
        if let Some(f) = &self.checked_fields {
            t.authorized(f)?
        }
        self.query.shape(t)?;
        if let Some(q) = &self.queried {
            q.shape(t)?
        }
        Ok(())
    }
    pub fn checked(&self) -> bool {
        self.checked_fields.as_ref() == Some(&self.fields)
    }
    pub fn check(&mut self, t: &Task) -> Result<Report> {
        t.validate()?;
        self.validate(t)?;
        let r = t.report(&self.fields)?;
        self.checked_fields = Some(self.fields.clone());
        Ok(r)
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.checked()
            && t.validate().is_ok()
            && self.validate(t).is_ok()
            && t.report(&self.fields).is_ok_and(|r| r.passed())
    }
    pub fn fill(&mut self, t: &Task) -> Result<()> {
        t.validate()?;
        self.validate(t)?;
        self.fields = t.reference.clone();
        self.checked_fields = None;
        Ok(())
    }
    pub fn changed(&mut self) {
        self.checked_fields = None
    }
    pub fn poll(&mut self, t: &Task) -> Result<Reply> {
        self.validate(t)?;
        let result = self.query.run(t);
        self.queried = Some(self.query.clone());
        result
    }
    pub fn observed(&self, t: &Task) -> Option<Result<Reply>> {
        (self.queried.as_ref() == Some(&self.query)).then(|| self.query.run(t))
    }
    pub fn reset(&mut self, t: &Task) {
        *self = Self::new(t)
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
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.initial.id == self.example.id
            || self.initial.activity != self.example.activity
        {
            return Err("Use version 1 and different example/assignment identities for the same monitoring skill.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        fn worlds(t: &Task) -> Vec<&Stream> {
            t.cases.iter().map(|c| &c.stream).collect()
        }
        if serde_json::to_value(worlds(&self.initial)).ok()
            == serde_json::to_value(worlds(&self.example)).ok()
        {
            return Err(
                "Use different virtual measurements for the worked example and assignment.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub practice: Draft,
    pub assignment: Draft,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            practice: Draft::new(&d.example),
            assignment: Draft::new(&d.initial),
        }
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        d.validate()?;
        self.shape()?;
        if self.version != 1 {
            return Err("Unsupported monitoring progress version; original retained.".into());
        }
        self.practice.validate(&d.example)?;
        self.assignment.validate(&d.initial)
    }
    pub fn solved(&self, d: &Definition) -> bool {
        self.validate(d).is_ok() && self.assignment.solved(&d.initial)
    }
}

#[cfg(test)]
mod tests;
