//! Separate source-bound power plans; edits always clear stale checks.
use crate::{
    poe::Plan,
    poe_task::{self as task, Check, Task},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
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
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.source_id != self.assignment.id
            || !task::identity(&self.source_id)
            || self.assignment.id == self.example.id
            || self.assignment.facts == self.example.facts
        {
            return Err("Keep a versioned power assignment and a separate worked example with different equipment facts.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.source_id != self.source_id {
            return Err(
                "Saved power work belongs to another lesson; keep the original progress.".into(),
            );
        }
        s.assignment.bound(&self.assignment)?;
        s.example.bound(&self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        s.assignment
            .checks(&self.assignment)
            .into_iter()
            .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub source_task: String,
    pub plan: Plan,
    pub checked: bool,
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self, String> {
        t.validate()?;
        Ok(Self {
            source_task: serde_json::to_string(t).map_err(|e| e.to_string())?,
            plan: t.initial.clone(),
            checked: false,
        })
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        if self.source_task.len() > 16384
            || serde_json::to_string(t).map_err(|e| e.to_string())? != self.source_task
        {
            return Err("Saved power facts or requirements changed; keep the original work rather than inventing new answers.".into());
        }
        t.facts.validate_plan(&self.plan)
    }
    pub fn edit(&mut self, t: &Task, p: Plan) -> Result<(), String> {
        self.bound(t)?;
        t.facts.validate_plan(&p)?;
        if self.plan != p {
            self.plan = p;
            self.checked = false
        }
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        if let Err(e) = self.bound(t) {
            return vec![Check {
                passed: false,
                label: "Saved power plan".into(),
                detail: e,
            }];
        }
        if !self.checked {
            return vec![Check {
                passed: false,
                label: "Power plan unchecked".into(),
                detail:
                    "Select Check power plan after your final connection or power-control change."
                        .into(),
            }];
        }
        t.outcomes(&self.plan)
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.bound(t).is_ok() && self.checked && t.solved(&self.plan)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Edit { plan: Plan },
    Check,
    Reset,
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
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            example: Scope::new(&d.example)?,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !task::identity(&self.source_id)
            || self.assignment.source_task.len() > 16384
            || self.example.source_task.len() > 16384
            || self.assignment.plan.attachments.len() > 8
            || self.example.plan.attachments.len() > 8
            || self.assignment.plan.enabled.len() > 8
            || self.example.plan.enabled.len() > 8
        {
            return Err(
                "Keep the saved power workspace within its versioned small-equipment bounds."
                    .into(),
            );
        }
        let assignment: Task = serde_json::from_str(&self.assignment.source_task)
            .map_err(|e| format!("Malformed saved power assignment: {e}"))?;
        let example: Task = serde_json::from_str(&self.example.source_task)
            .map_err(|e| format!("Malformed saved power example: {e}"))?;
        assignment.validate()?;
        example.validate()?;
        if assignment.id != self.source_id
            || assignment.id == example.id
            || assignment.facts == example.facts
        {
            return Err(
                "Keep the supplied assignment identity and separate practice facts.".into(),
            );
        }
        self.assignment.bound(&assignment)?;
        self.example.bound(&example)
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        let (scope, t) = if practice {
            (&mut self.example, &d.example)
        } else {
            (&mut self.assignment, &d.assignment)
        };
        match a {
            Action::Edit { plan } => scope.edit(t, plan.clone())?,
            Action::Check => {
                scope.bound(t)?;
                scope.checked = true
            }
            Action::Reset => *scope = Scope::new(t)?,
        }
        Ok(())
    }
}
