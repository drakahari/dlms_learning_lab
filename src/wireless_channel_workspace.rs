//! Independent bounded channel plans. No stored survey output or host execution.
use crate::{
    script_teaching::View,
    wireless_channels::{Check, Network},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
mod bindings;
pub use bindings::{HELP, context, relevant, source};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub radio: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8>")]
    pub value: String,
}
fn fields<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Field>, D::Error> {
    crate::cron_workspace::sequence::<_, Field, 8>(d)
}
fn checked_fields<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Vec<Field>>, D::Error> {
    #[derive(Deserialize)]
    struct Bounded(#[serde(deserialize_with = "fields")] Vec<Field>);
    Option::<Bounded>::deserialize(d).map(|v| v.map(|v| v.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub network: Network,
    #[serde(deserialize_with = "fields")]
    pub reference: Vec<Field>,
}
fn channel(text: &str) -> Result<u8> {
    let s = text.trim();
    if s.is_empty() {
        return Err("Enter a channel number before checking this radio.".into());
    }
    if !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(
            "Use a whole channel number, such as 6; do not enter a frequency or an address.".into(),
        );
    }
    let n = s
        .parse::<u16>()
        .map_err(|_| "Choose a channel from 1 to 11 in this supplied band.".to_string())?;
    if !(1..=11).contains(&n) {
        return Err("Choose a channel from 1 to 11 in this supplied band.".into());
    }
    Ok(n as u8)
}
impl Task {
    pub fn initial_fields(&self) -> Vec<Field> {
        self.network
            .radios
            .iter()
            .filter(|r| r.managed)
            .map(|r| Field {
                radio: r.id.clone(),
                value: r.initial_channel.to_string(),
            })
            .collect()
    }
    pub fn validate_fields(&self, values: &[Field]) -> Result<()> {
        self.network.validate()?;
        let expected: BTreeSet<_> = self
            .network
            .radios
            .iter()
            .filter(|r| r.managed)
            .map(|r| r.id.as_str())
            .collect();
        let actual: BTreeSet<_> = values.iter().map(|f| f.radio.as_str()).collect();
        if values.len() != expected.len()
            || actual != expected
            || values.iter().any(|f| {
                f.radio.len() > 64 || f.value.len() > 8 || f.value.chars().any(char::is_control)
            })
        {
            return Err("Keep one bounded channel field for each managed radio; protected and unknown radios cannot be edited.".into());
        }
        Ok(())
    }
    pub fn preview(&self, values: &[Field]) -> Result<BTreeMap<String, u8>> {
        self.validate_fields(values)?;
        let mut plan = self.network.initial_plan();
        for field in values {
            plan.insert(field.radio.clone(), channel(&field.value)?);
        }
        Ok(plan)
    }
    pub fn rows(&self, values: &[Field]) -> Result<Vec<Check>> {
        self.validate_fields(values)?;
        let mut errors = vec![];
        for f in values {
            if let Err(feedback) = channel(&f.value) {
                let radio = self
                    .network
                    .radios
                    .iter()
                    .find(|r| r.id == f.radio)
                    .unwrap();
                errors.push(Check {
                    id: format!("field-{}", f.radio),
                    label: radio.label.clone(),
                    correct: false,
                    feedback,
                });
            }
        }
        if !errors.is_empty() {
            return Ok(errors);
        }
        self.network.checks(&self.preview(values)?)
    }
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("Use a stable channel-planning task identifier.".into());
        }
        self.network.validate()?;
        self.validate_fields(&self.reference)?;
        if !self.rows(&self.reference)?.iter().all(|r| r.correct) {
            return Err("The supplied reference plan cannot meet every survey and preservation requirement.".into());
        }
        if self.rows(&self.initial_fields())?.iter().all(|r| r.correct) {
            return Err("Independent work must begin with a meaningful planning problem.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id || self.initial.network == self.example.network {
            return Err(
                "Use different supplied radios/surveys for practice and assignment.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task_id: String,
    #[serde(deserialize_with = "fields")]
    pub fields: Vec<Field>,
    #[serde(deserialize_with = "checked_fields")]
    pub last_check: Option<Vec<Field>>,
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            fields: t.initial_fields(),
            last_check: None,
        }
    }
    pub fn shape(&self) -> Result<()> {
        if self.task_id.is_empty()
            || self.task_id.len() > 64
            || self.fields.is_empty()
            || self.fields.len() > 8
            || self
                .last_check
                .as_ref()
                .is_some_and(|f| f.is_empty() || f.len() > 8)
            || self
                .fields
                .iter()
                .chain(self.last_check.iter().flatten())
                .any(|f| {
                    f.radio.is_empty()
                        || f.radio.len() > 64
                        || f.value.len() > 8
                        || f.value.chars().any(char::is_control)
                })
        {
            return Err(
                "Saved channel fields exceed the supported bounds; original retained.".into(),
            );
        }
        Ok(())
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.task_id != t.id {
            return Err("This plan belongs to a different supplied activity.".into());
        }
        t.validate_fields(&self.fields)?;
        if let Some(values) = &self.last_check {
            t.validate_fields(values)?;
        }
        Ok(())
    }
    pub fn checked(&self) -> bool {
        self.last_check.as_ref() == Some(&self.fields)
    }
    pub fn set(&mut self, t: &Task, id: &str, value: String) -> Result<()> {
        self.validate(t)?;
        let mut next = self.clone();
        let field = next
            .fields
            .iter_mut()
            .find(|f| f.radio == id)
            .ok_or("Choose a managed radio from this activity.")?;
        if field.value != value {
            field.value = value;
            next.last_check = None;
        }
        next.validate(t)?;
        *self = next;
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<Check>> {
        self.validate(t)?;
        let rows = t.rows(&self.fields)?;
        self.last_check = Some(self.fields.clone());
        Ok(rows)
    }
    pub fn fill_reference(&mut self, t: &Task) -> Result<()> {
        t.validate()?;
        self.validate(t)?;
        self.fields = t.reference.clone();
        self.last_check = None;
        Ok(())
    }
    pub fn solved(&self, t: &Task) -> bool {
        t.validate().is_ok()
            && self.validate(t).is_ok()
            && self.checked()
            && t.rows(&self.fields)
                .is_ok_and(|r| r.iter().all(|c| c.correct))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        }
    }
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported channel-plan workspace version; original retained.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        d.validate()?;
        self.assignment.validate(&d.initial)?;
        self.practice.validate(&d.example)
    }
    pub fn solved(&self, d: &Definition) -> bool {
        self.validate(d).is_ok() && self.assignment.solved(&d.initial)
    }
    pub fn reset_practice(&mut self, d: &Definition) {
        self.practice = Draft::new(&d.example);
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Draft::new(&d.initial);
    }
}
#[cfg(test)]
mod tests;
