//! Separate virtual Apache draft, saved configuration and live verification.
use super::{
    Client, Inventory, Name, Result, Trust,
    config::{self, Connection},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_CASES: usize = 6;

fn cases<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Case>, D::Error> {
    crate::cron_workspace::sequence::<_, Case, MAX_CASES>(d)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Verified,
    NameMismatchOnDefault,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    pub client: Client,
    pub site: Name,
    pub expected: Expected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub inventory: Inventory,
    #[serde(deserialize_with = "crate::cron::text::<_,{config::CONFIG_LIMIT}>")]
    pub starter: String,
    #[serde(deserialize_with = "cases")]
    pub checks: Vec<Case>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    #[serde(deserialize_with = "crate::cron::text::<_,{config::CONFIG_LIMIT}>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,{config::CONFIG_LIMIT}>")]
    pub saved: String,
    #[serde(deserialize_with = "crate::cron::text::<_,{config::CONFIG_LIMIT}>")]
    pub applied: String,
    pub checked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub passed: bool,
    pub feedback: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    Draft {
        #[serde(deserialize_with = "crate::cron::text::<_,{config::CONFIG_LIMIT}>")]
        contents: String,
    },
    Save,
    Apply,
    Check,
    Reset,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        self.inventory.validate()?;
        config::load(&self.inventory, &self.starter)?;
        if self.checks.is_empty() || self.checks.len() > MAX_CASES {
            return Err("Supply one to six explicit virtual client checks.".into());
        }
        let mut labels = BTreeSet::new();
        for case in &self.checks {
            if case.label.trim().is_empty()
                || case.label.len() > 64
                || case.label.chars().any(char::is_control)
                || !labels.insert(&case.label)
                || !case.site.dns()
            {
                return Err(
                    "Use distinct client labels and expected virtual host DNS names.".into(),
                );
            }
            case.client.validate(&self.inventory)?;
        }
        let fresh = Work::new(self);
        if fresh.checks(self).iter().all(|check| check.passed) {
            return Err("Supply a virtual HTTPS failure that the learner can repair.".into());
        }
        Ok(())
    }
}

impl Work {
    pub fn new(task: &Task) -> Self {
        let starter = task.starter.clone();
        Self {
            draft: starter.clone(),
            saved: starter.clone(),
            applied: starter,
            checked: false,
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        if self.draft.len() > config::CONFIG_LIMIT {
            return Err("Saved draft exceeds the 4 KiB virtual configuration limit.".into());
        }
        config::parse(&self.saved)?;
        config::load(&task.inventory, &self.applied)?;
        Ok(())
    }

    pub fn apply(&mut self, task: &Task, action: Action) -> Result<()> {
        match action {
            Action::Draft { contents } => {
                if contents.len() > config::CONFIG_LIMIT {
                    return Err("Keep the virtual configuration within 4 KiB.".into());
                }
                if self.draft != contents {
                    self.draft = contents;
                    self.checked = false;
                }
            }
            Action::Save => {
                config::parse(&self.draft)?;
                self.saved.clone_from(&self.draft);
                self.checked = false;
            }
            Action::Apply => {
                config::load(&task.inventory, &self.saved)?;
                self.applied.clone_from(&self.saved);
                self.checked = false;
            }
            Action::Check => self.checked = true,
            Action::Reset => *self = Self::new(task),
        }
        Ok(())
    }

    pub fn inspect(&self, task: &Task, case: &Case) -> Result<Connection> {
        let sites = config::load(&task.inventory, &self.applied)?;
        config::inspect(&task.inventory, &sites, &case.client)
    }

    pub fn checks(&self, task: &Task) -> Vec<Check> {
        let mut out = vec![Check {
            label: "Saved and applied configuration".into(),
            passed: self.draft == self.saved && self.saved == self.applied,
            feedback: if self.draft != self.saved {
                "Save your draft. Client inspection still uses the applied settings.".into()
            } else if self.saved != self.applied {
                "Apply the saved file. Check any file and key diagnostics before inspecting the client again.".into()
            } else {
                "The client checks use the current saved settings.".into()
            },
        }];
        match config::load(&task.inventory, &self.applied) {
            Err(reason) => out.push(Check {
                label: "Applied HTTPS settings".into(),
                passed: false,
                feedback: reason,
            }),
            Ok(sites) => {
                for case in &task.checks {
                    match config::inspect(&task.inventory, &sites, &case.client) {
                        Err(reason) => out.push(Check {
                            label: case.label.clone(),
                            passed: false,
                            feedback: reason,
                        }),
                        Ok(actual) => out.push(case.compare(&actual)),
                    }
                }
            }
        }
        out
    }
}

impl Case {
    fn compare(&self, actual: &Connection) -> Check {
        let right_site = actual.selected.0.eq_ignore_ascii_case(&self.site.0);
        let trusted = matches!(actual.verification.trust, Trust::Verified(_));
        let right_certificate = match self.expected {
            Expected::Verified => actual.verification.hostname_matches,
            Expected::NameMismatchOnDefault => {
                actual.used_default && !actual.verification.hostname_matches
            }
        };
        let passed = right_site && trusted && right_certificate;
        let feedback = if !right_site {
            format!(
                "The listener selected {}. Compare this client's SNI with the applied ServerName values and the default order.",
                actual.selected.0
            )
        } else if !trusted {
            trust_explanation(&actual.verification.trust)
        } else if !right_certificate {
            match self.expected {
                Expected::Verified=>format!("The served certificate {} does not list {} as a DNS name.", actual.verification.presented.0,self.client.hostname.0),
                Expected::NameMismatchOnDefault=>"This client should use the first virtual host and receive a name mismatch for its unserved hostname.".into(),
            }
        } else if matches!(self.expected, Expected::NameMismatchOnDefault) {
            "The expected default host served a certificate with a trusted path. The client's other name still fails verification.".into()
        } else {
            format!(
                "{} selected the intended virtual host, matched its certificate DNS name and built a trusted path.",
                self.client.hostname.0
            )
        };
        Check {
            label: self.label.clone(),
            passed,
            feedback,
        }
    }
}

pub fn trust_explanation(trust: &Trust) -> String {
    match trust {
        Trust::Verified(path) => format!(
            "Trusted path · {}",
            path.iter()
                .map(|n| n.0.as_str())
                .collect::<Vec<_>>()
                .join(" → ")
        ),
        Trust::OutsideValidity(id) => format!(
            "{} falls outside its documented validity days at this trial date.",
            id.0
        ),
        Trust::MissingIssuer { from, issuer } => format!(
            "{} needs issuer {}. This client could find it in neither the served chain nor its supplied trust anchors.",
            from.0, issuer.0
        ),
        Trust::UnknownSelfSigned(id) => format!(
            "{} is self-signed and this client has no matching trust anchor.",
            id.0
        ),
        Trust::IssuerNotCa(id) => format!(
            "{} cannot issue the next certificate in this virtual path.",
            id.0
        ),
        Trust::Cycle(id) => format!("The supplied issuer links loop through {}.", id.0),
    }
}
