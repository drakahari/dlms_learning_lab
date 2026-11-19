//! Bounded declarative WLAN tasks and isolated drafts. No equipment execution.
use crate::{
    script_teaching::View,
    wlan_identity::{Authentication, Compatibility, Configuration, ManagementProtection, Network},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, String>;
mod bindings;
pub use bindings::{HELP, context, relevant, source};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Editable {
    Enabled,
    Ssid,
    Authentication,
    ManagementProtection,
    Passphrase,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub wlan: String,
    pub configuration: Configuration,
}
fn fields<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Field>, D::Error> {
    crate::cron_workspace::sequence::<_, Field, 4>(d)
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
pub struct Policy {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub wlan: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub ssid: String,
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Authentication,3>")]
    pub allowed_authentication: Vec<Authentication>,
    pub required_protection: Option<ManagementProtection>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Editable,5>")]
    pub editable: Vec<Editable>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Compatible {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        wlan: String,
        authentication: Authentication,
    },
    Rejected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub client: String,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub network: Network,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Policy,4>")]
    pub policies: Vec<Policy>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,12>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "fields")]
    pub reference: Vec<Field>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub id: String,
    pub label: String,
    pub correct: bool,
    pub feedback: String,
}
impl Task {
    pub fn initial_fields(&self) -> Vec<Field> {
        self.network
            .wlans
            .iter()
            .filter(|w| w.managed)
            .map(|w| Field {
                wlan: w.id.clone(),
                configuration: w.initial.clone(),
            })
            .collect()
    }
    fn shape(&self) -> Result<()> {
        self.network.validate()?;
        if self.id.is_empty()
            || self.id.len() > 64
            || !self.id.as_bytes()[0].is_ascii_lowercase()
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("Use a stable WLAN task identifier.".into());
        }
        let managed: BTreeSet<_> = self
            .network
            .wlans
            .iter()
            .filter(|w| w.managed)
            .map(|w| w.id.as_str())
            .collect();
        let policies: BTreeSet<_> = self.policies.iter().map(|p| p.wlan.as_str()).collect();
        let clients: BTreeSet<_> = self.network.clients.iter().map(|c| c.id.as_str()).collect();
        let cases: BTreeSet<_> = self.cases.iter().map(|c| c.client.as_str()).collect();
        if policies != managed
            || self.policies.len() != managed.len()
            || cases != clients
            || self.cases.len() != clients.len()
        {
            return Err(
                "Supply one policy per managed WLAN and one expectation per supplied client."
                    .into(),
            );
        }
        for p in &self.policies {
            if p.ssid.is_empty()
                || p.ssid.len() > 32
                || !p.ssid.bytes().all(|b| (0x20..=0x7e).contains(&b))
                || p.allowed_authentication.is_empty()
                || p.allowed_authentication.len() > 3
                || p.allowed_authentication
                    .iter()
                    .collect::<BTreeSet<_>>()
                    .len()
                    != p.allowed_authentication.len()
                || p.editable.is_empty()
                || p.editable.len() > 5
                || p.editable.iter().collect::<BTreeSet<_>>().len() != p.editable.len()
            {
                return Err(
                    "Invalid WLAN policy name, authentication set or editing scope.".into(),
                );
            }
        }
        let mut managed_positive = false;
        for case in &self.cases {
            if let Expected::Compatible { wlan, .. } = &case.expected {
                if !self.network.wlans.iter().any(|w| w.id == *wlan) {
                    return Err("A client expectation refers to an unknown WLAN.".into());
                }
                managed_positive |= managed.contains(wlan.as_str());
            }
        }
        if !managed_positive {
            return Err(
                "At least one supplied client must use a managed WLAN successfully.".into(),
            );
        }
        Ok(())
    }
    pub fn validate_fields(&self, values: &[Field]) -> Result<()> {
        self.shape()?;
        let ids: BTreeSet<_> = values.iter().map(|f| f.wlan.as_str()).collect();
        let wanted: BTreeSet<_> = self.policies.iter().map(|p| p.wlan.as_str()).collect();
        if values.len() != wanted.len() || ids != wanted {
            return Err(
                "Keep one configuration for each managed WLAN; no missing or foreign profiles."
                    .into(),
            );
        }
        for f in values {
            let p = self.policies.iter().find(|p| p.wlan == f.wlan).unwrap();
            let initial = &self
                .network
                .wlans
                .iter()
                .find(|w| w.id == f.wlan)
                .unwrap()
                .initial;
            let v = &f.configuration;
            if v.ssid.len() > 128 || v.training_passphrase.len() > 128 {
                return Err("WLAN draft text exceeds its supported size.".into());
            }
            for (field, changed) in [
                (Editable::Enabled, v.enabled != initial.enabled),
                (Editable::Ssid, v.ssid != initial.ssid),
                (
                    Editable::Authentication,
                    v.authentication != initial.authentication,
                ),
                (
                    Editable::ManagementProtection,
                    v.management_protection != initial.management_protection,
                ),
                (
                    Editable::Passphrase,
                    v.training_passphrase != initial.training_passphrase,
                ),
            ] {
                if changed && !p.editable.contains(&field) {
                    return Err("A protected setting changed outside this task's editing scope; restore the supplied setting.".into());
                }
            }
        }
        Ok(())
    }
    pub fn preview(&self, values: &[Field]) -> Result<BTreeMap<String, Configuration>> {
        self.validate_fields(values)?;
        let mut all = self.network.initial_configurations();
        for f in values {
            all.insert(f.wlan.clone(), f.configuration.clone());
        }
        Ok(all)
    }
    pub fn rows(&self, values: &[Field]) -> Result<Vec<Check>> {
        let all = self.preview(values)?;
        let report = self.network.inspect(&all)?;
        if !report.issues.is_empty() {
            return Ok(report
                .issues
                .into_iter()
                .map(|i| Check {
                    id: format!("field-{}-{}", i.wlan, i.field),
                    label: i.field.into(),
                    correct: false,
                    feedback: i.feedback,
                })
                .collect());
        }
        let mut rows = vec![];
        for p in &self.policies {
            let v = &all[&p.wlan];
            let label = &self
                .network
                .wlans
                .iter()
                .find(|w| w.id == p.wlan)
                .unwrap()
                .label;
            for (part, correct, feedback) in [
                (
                    "name",
                    v.ssid == p.ssid,
                    "Use the network name in the supplied site requirements; case and spaces matter.",
                ),
                (
                    "enabled",
                    v.enabled == p.enabled,
                    "The WLAN's enabled state does not meet the site requirement.",
                ),
                (
                    "authentication",
                    p.allowed_authentication.contains(&v.authentication),
                    "The authentication method does not meet this site's security requirement.",
                ),
                (
                    "protection",
                    p.required_protection
                        .is_none_or(|required| required == v.management_protection),
                    "The protected-management-frame policy does not meet the site requirement.",
                ),
            ] {
                rows.push(Check {
                    id: format!("policy-{}-{part}", p.wlan),
                    label: format!("{label} · {part}"),
                    correct,
                    feedback: if correct {
                        "Site requirement met.".into()
                    } else {
                        feedback.into()
                    },
                });
            }
        }
        for case in &self.cases {
            let actual = &report.clients[&case.client];
            let correct = match (&case.expected, actual) {
                (
                    Expected::Compatible {
                        wlan,
                        authentication,
                    },
                    Compatibility::Compatible {
                        wlan: found,
                        authentication: method,
                        ..
                    },
                ) => wlan == found && authentication == method,
                (
                    Expected::Rejected,
                    Compatibility::NameNotFound
                    | Compatibility::Disabled
                    | Compatibility::AuthenticationMismatch
                    | Compatibility::ManagementProtectionRequired
                    | Compatibility::PassphraseMismatch,
                ) => true,
                _ => false,
            };
            let label = self
                .network
                .clients
                .iter()
                .find(|c| c.id == case.client)
                .unwrap()
                .label
                .clone();
            let feedback = if correct {
                format!("Required outcome verified. {}", actual.feedback())
            } else if let Compatibility::Compatible {
                wlan: found,
                authentication: method,
                ..
            } = actual
            {
                match &case.expected {
                    Expected::Rejected => "This profile is compatible, but the task requires it to remain incompatible. Review the site policy and supplied client profile.".into(),
                    Expected::Compatible { wlan: wanted, .. } if found != wanted => {
                        let name = |id: &str| self.network.wlans.iter().find(|w| w.id == id).unwrap().label.as_str();
                        format!("This profile matches {}, but the task requires {}. Compare the exact network names.", name(found), name(wanted))
                    }
                    Expected::Compatible { authentication, .. } => format!("This profile is compatible using {}, but the task requires {} for this profile.", method.label(), authentication.label()),
                }
            } else {
                format!(
                    "This profile must be compatible with the required WLAN. {}",
                    actual.feedback()
                )
            };
            rows.push(Check {
                id: format!("client-{}", case.client),
                label,
                correct,
                feedback,
            });
        }
        Ok(rows)
    }
    pub fn validate(&self) -> Result<()> {
        self.shape()?;
        if !self.rows(&self.reference)?.iter().all(|r| r.correct) {
            return Err(
                "The reference configuration cannot meet every site and client requirement.".into(),
            );
        }
        if self.rows(&self.initial_fields())?.iter().all(|r| r.correct) {
            return Err("Start the activity with a meaningful configuration problem.".into());
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
            return Err("Use distinct example and independent WLAN tasks.".into());
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
    pub fn shape(&self) -> Result<()> {
        fn fields_shape(fields: &[Field]) -> bool {
            (1..=4).contains(&fields.len())
                && fields
                    .iter()
                    .map(|f| &f.wlan)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == fields.len()
                && fields.iter().all(|f| {
                    !f.wlan.is_empty()
                        && f.wlan.len() <= 64
                        && f.configuration.ssid.len() <= 128
                        && f.configuration.training_passphrase.len() <= 128
                })
        }
        if self.task_id.is_empty()
            || self.task_id.len() > 64
            || !fields_shape(&self.fields)
            || self.last_check.as_ref().is_some_and(|f| !fields_shape(f))
        {
            return Err("Malformed or oversized WLAN draft; original retained.".into());
        }
        Ok(())
    }
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            fields: t.initial_fields(),
            last_check: None,
        }
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.task_id != t.id {
            return Err("Saved WLAN work belongs to another activity; original retained.".into());
        }
        t.validate_fields(&self.fields)?;
        if let Some(old) = &self.last_check {
            t.validate_fields(old)?;
        }
        Ok(())
    }
    pub fn set(&mut self, t: &Task, id: &str, value: Configuration) -> Result<()> {
        self.validate(t)?;
        let mut next = self.fields.clone();
        let f = next
            .iter_mut()
            .find(|f| f.wlan == id)
            .ok_or("Choose a managed WLAN in this activity.")?;
        f.configuration = value;
        t.validate_fields(&next)?;
        if next != self.fields {
            self.fields = next;
            self.last_check = None;
        }
        Ok(())
    }
    pub fn fill_reference(&mut self, t: &Task) -> Result<()> {
        t.validate()?;
        self.validate(t)?;
        t.validate_fields(&t.reference)?;
        self.fields = t.reference.clone();
        self.last_check = None;
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<()> {
        t.validate()?;
        self.validate(t)?;
        t.rows(&self.fields)?;
        self.last_check = Some(self.fields.clone());
        Ok(())
    }
    pub fn checked(&self) -> bool {
        self.last_check.as_ref() == Some(&self.fields)
    }
    pub fn solved(&self, t: &Task) -> bool {
        t.validate().is_ok()
            && self.checked()
            && self.validate(t).is_ok()
            && t.rows(&self.fields)
                .is_ok_and(|rows| !rows.is_empty() && rows.iter().all(|r| r.correct))
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
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported WLAN session version; original retained.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        }
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        d.validate()?;
        self.assignment.validate(&d.initial)?;
        self.practice.validate(&d.example)
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        self.validate(d)?;
        if practice {
            self.practice = Draft::new(&d.example)
        } else {
            self.assignment = Draft::new(&d.initial)
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
