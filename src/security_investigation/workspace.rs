//! Separate incident investigation and facility policy over supplied records.
use super::{
    access_plan as access, change_evidence as evidence,
    credential_task::CredentialTask,
    credentials,
    vpn_design::Profile,
    vpn_design_task::{Draft, VpnTask},
};
use crate::{management_document, management_path, management_policy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type Result<T> = std::result::Result<T, String>;
const CONFIG_BYTES: usize = 8192;

fn records<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<evidence::Record>, D::Error> {
    crate::cron_workspace::sequence::<_, evidence::Record, 16>(d)
}
fn route_trials<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<evidence::Trial>, D::Error> {
    crate::cron_workspace::sequence::<_, evidence::Trial, 6>(d)
}
fn access_trials<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<FacilityTrial>, D::Error> {
    crate::cron_workspace::sequence::<_, FacilityTrial, 8>(d)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeTask {
    pub network: management_path::Network,
    pub supplied: management_policy::Policy,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter: String,
    #[serde(deserialize_with = "records")]
    pub journal: Vec<evidence::Record>,
    #[serde(deserialize_with = "route_trials")]
    pub trials: Vec<evidence::Trial>,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub affected_trial: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityTrial {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    pub trial: access::Trial,
    pub intended: bool,
    pub require_log: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityTask {
    pub starter: access::Policy,
    #[serde(deserialize_with = "access_trials")]
    pub trials: Vec<FacilityTrial>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "activity", rename_all = "snake_case")]
pub enum Task {
    Change { supplied: Box<ChangeTask> },
    Facility { supplied: FacilityTask },
    Credentials { supplied: CredentialTask },
    VpnDesign { supplied: VpnTask },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "activity", rename_all = "snake_case")]
pub enum Scope {
    Change {
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        draft: String,
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        applied: String,
        report: evidence::Report,
    },
    Facility {
        plan: access::Policy,
    },
    Credentials {
        policy: credentials::Policy,
    },
    VpnDesign {
        drafts: [Draft; 2],
        applied: [Profile; 2],
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub current: Scope,
    pub checked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

impl Outcome {
    pub(super) fn new(label: impl Into<String>, passed: bool, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            passed,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "op")]
pub enum Action {
    Report {
        report: evidence::Report,
    },
    Draft {
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        contents: String,
    },
    Apply,
    Facility {
        plan: access::Policy,
    },
    Credentials {
        policy: credentials::Policy,
    },
    VpnDrafts {
        drafts: [Draft; 2],
    },
    ApplyVpn,
    Check,
    Reset,
}

pub(super) fn label(s: &str, limit: usize) -> bool {
    !s.trim().is_empty() && s.len() <= limit && !s.chars().any(char::is_control)
}

impl ChangeTask {
    pub fn parse(&self, text: &str) -> Result<management_policy::Policy> {
        if text.len() > CONFIG_BYTES {
            return Err("Use up to 8 KiB of supplied management policy.".into());
        }
        management_document::parse(&self.network, &self.supplied, text).map_err(|e| e.to_string())
    }

    pub fn validate(&self) -> Result<()> {
        let starter = self.parse(&self.starter)?;
        if self.journal.len() < 3
            || self.journal.len() > 16
            || self.trials.len() < 2
            || self.trials.len() > 6
            || !label(&self.affected_trial, 48)
        {
            return Err("Supply a bounded journal and two or more distinct client trials.".into());
        }
        let mut incoming = BTreeSet::new();
        if self
            .journal
            .iter()
            .any(|r| r.id == 0 || !incoming.insert(r.id) || !label(&r.device, 32))
            || self
                .journal
                .windows(2)
                .any(|pair| pair[0].minute > pair[1].minute)
        {
            return Err("Use unique journal record IDs and order the observed times.".into());
        }
        let mut names = BTreeSet::new();
        if self
            .trials
            .iter()
            .any(|trial| !names.insert(&trial.name) || !label(&trial.name, 48))
            || !self.trials.iter().any(|trial| trial.intended_admission)
            || !self
                .trials
                .iter()
                .any(|trial| trial.name == self.affected_trial && !trial.intended_admission)
        {
            return Err("Keep one working approved client and a named restricted trial.".into());
        }
        let actual = evidence::trials(&self.network, &starter, &self.trials)?;
        if !self.trials.iter().zip(actual).any(|(trial, outcome)| {
            trial.name == self.affected_trial && !outcome.expected && outcome.reached
        }) {
            return Err(
                "The supplied restricted-client observation must match the starter policy.".into(),
            );
        }
        self.validate_journal()
    }

    fn validate_journal(&self) -> Result<()> {
        use evidence::RecordKind::*;
        let device = &self.network.management.device;
        let records = || {
            self.journal
                .iter()
                .filter(|record| &record.device == device)
        };
        let approval = records().find(|r| matches!(r.kind, TicketApproved { .. }));
        let applied = records().find(|r| matches!(r.kind, RevisionApplied { .. }));
        let observed = records().find(|r| {
            matches!(&r.kind,
            AdmissionObserved { trial, admitted: true } if trial == &self.affected_trial)
        });
        let recorded_use = applied.is_some_and(|applied| {
            records().any(|r| {
                r.minute >= applied.minute
                    && matches!(&r.kind,
                SuccessfulSignin { trial } if trial == &self.affected_trial)
            })
        });
        let report = evidence::Report {
            approval: approval.map(|r| r.id),
            application: applied.map(|r| r.id),
            observed_extra_access: observed.map(|r| r.id),
            successful_use_recorded: Some(recorded_use),
        };
        if evidence::findings(&self.journal, device, &self.affected_trial, &report)
            .iter()
            .all(|result| result.passed)
        {
            Ok(())
        } else {
            Err(
                "Supply a coherent approved, applied and observed sequence on the affected relay."
                    .into(),
            )
        }
    }
}

impl FacilityTask {
    pub fn outcomes(&self, plan: access::Policy) -> Vec<Outcome> {
        let mut checks: Vec<_> = self
            .trials
            .iter()
            .map(|t| {
                let result = plan.entry(t.trial);
                let admitted = result.access == access::Decision::Admitted;
                let detail = if t.require_log && !result.recorded {
                    format!(
                        "{} This trial also requires an access log entry.",
                        result.access.detail()
                    )
                } else {
                    result.access.detail().into()
                };
                Outcome::new(
                    &t.label,
                    admitted == t.intended && (!t.require_log || result.recorded),
                    detail,
                )
            })
            .collect();
        let reported = plan.report_reaches_security();
        checks.push(Outcome::new(
            "Route the supplied suspicious contact report",
            reported,
            if reported {
                "The report reaches the security desk."
            } else {
                "The current report destination leaves the security desk out."
            },
        ));
        checks
    }

    pub fn validate(&self) -> Result<()> {
        let mut labels = BTreeSet::new();
        if self.trials.len() < 3
            || self.trials.len() > 8
            || self
                .trials
                .iter()
                .any(|t| !label(&t.label, 64) || !labels.insert(&t.label))
            || !self.trials.iter().any(|t| !t.intended)
            || !self.trials.iter().any(|t| t.intended)
            || self.outcomes(self.starter).iter().all(|r| r.passed)
        {
            return Err(
                "Supply distinct allowed and denied facility trials and a repairable starter plan."
                    .into(),
            );
        }
        Ok(())
    }
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Change { supplied } => supplied.validate(),
            Self::Facility { supplied } => supplied.validate(),
            Self::Credentials { supplied } => supplied.validate(),
            Self::VpnDesign { supplied } => supplied.validate(),
        }
    }
}

impl Work {
    pub fn new(task: &Task) -> Self {
        let current = match task {
            Task::Change { supplied } => Scope::Change {
                draft: supplied.starter.clone(),
                applied: supplied.starter.clone(),
                report: evidence::Report {
                    approval: None,
                    application: None,
                    observed_extra_access: None,
                    successful_use_recorded: None,
                },
            },
            Task::Facility { supplied } => Scope::Facility {
                plan: supplied.starter,
            },
            Task::Credentials { supplied } => Scope::Credentials {
                policy: supplied.starter,
            },
            Task::VpnDesign { supplied } => Scope::VpnDesign {
                drafts: supplied.starter.map(Draft::from_profile),
                applied: supplied.starter,
            },
        };
        Self {
            current,
            checked: false,
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        match (&self.current, task) {
            (
                Scope::Change {
                    draft,
                    applied,
                    report,
                },
                Task::Change { supplied },
            ) => {
                supplied.parse(applied)?;
                if draft.len() > CONFIG_BYTES {
                    return Err("The saved virtual policy draft exceeds 8 KiB.".into());
                }
                for selection in [
                    report.approval,
                    report.application,
                    report.observed_extra_access,
                ]
                .into_iter()
                .flatten()
                {
                    if !supplied.journal.iter().any(|entry| entry.id == selection) {
                        return Err("Select evidence from this supplied journal.".into());
                    }
                }
                Ok(())
            }
            (Scope::Facility { .. }, Task::Facility { .. }) => Ok(()),
            (Scope::Credentials { policy }, Task::Credentials { .. }) => policy.validate(),
            (Scope::VpnDesign { drafts, applied }, Task::VpnDesign { .. }) => {
                for draft in drafts {
                    draft.validate_size()?;
                }
                for profile in applied {
                    profile.validate()?;
                }
                Ok(())
            }
            _ => Err("This saved work belongs to a different security activity.".into()),
        }
    }

    pub fn outcomes(&self, task: &Task) -> Vec<Outcome> {
        if let Err(problem) = self.validate(task) {
            return vec![Outcome::new("Virtual workspace", false, problem)];
        }
        match (&self.current, task) {
            (
                Scope::Change {
                    draft,
                    applied,
                    report,
                },
                Task::Change { supplied },
            ) => {
                let mut checks = evidence::findings(
                    &supplied.journal,
                    &supplied.network.management.device,
                    &supplied.affected_trial,
                    report,
                )
                .into_iter()
                .map(|r| Outcome::new(r.label, r.passed, r.detail))
                .collect::<Vec<_>>();
                let applied_current = draft == applied;
                checks.push(Outcome::new(
                    "Applied policy",
                    applied_current,
                    if applied_current {
                        "The probes use the current virtual policy."
                    } else {
                        "Apply the current draft before checking the client probes."
                    },
                ));
                match supplied.parse(applied).and_then(|policy| {
                    evidence::trials(&supplied.network, &policy, &supplied.trials)
                }) {
                    Err(problem) => checks.push(Outcome::new("Client trials", false, problem)),
                    Ok(results) => {
                        for (trial, result) in supplied.trials.iter().zip(results) {
                            let expected = if result.expected {
                                "admit to the sign-in boundary"
                            } else {
                                "deny before sign-in"
                            };
                            let observed = if result.reached {
                                "admit to the sign-in boundary"
                            } else {
                                "deny before sign-in"
                            };
                            checks.push(Outcome::new(
                                &trial.name,
                                result.expected == result.reached,
                                format!(
                                    "Expected {expected}. Observed {observed}. {}",
                                    result.detail
                                ),
                            ));
                        }
                    }
                }
                checks
            }
            (Scope::Facility { plan }, Task::Facility { supplied }) => supplied.outcomes(*plan),
            (Scope::Credentials { policy }, Task::Credentials { supplied }) => {
                supplied.outcomes(*policy)
            }
            (Scope::VpnDesign { drafts, applied }, Task::VpnDesign { supplied }) => {
                let current_applied = drafts
                    .iter()
                    .zip(applied)
                    .all(|(draft, profile)| draft.profile().ok() == Some(*profile));
                let mut rows = vec![Outcome::new(
                    "Applied profiles",
                    current_applied,
                    if current_applied {
                        "The trials use the current profiles."
                    } else {
                        "Apply the current profiles before checking their paths."
                    },
                )];
                rows.extend(supplied.outcomes(applied));
                rows
            }
            _ => unreachable!("Validated security workspace"),
        }
    }

    pub fn apply(&mut self, task: &Task, action: Action) -> Result<()> {
        let mut updated = self.clone();
        match (&mut updated.current, task, action) {
            (
                Scope::Change { report, .. },
                Task::Change { .. },
                Action::Report { report: next },
            ) => {
                if *report != next {
                    *report = next;
                    updated.checked = false;
                }
            }
            (Scope::Change { draft, .. }, Task::Change { .. }, Action::Draft { contents }) => {
                if contents.len() > CONFIG_BYTES {
                    return Err("Keep the policy draft within 8 KiB.".into());
                }
                if *draft != contents {
                    *draft = contents;
                    updated.checked = false;
                }
            }
            (Scope::Change { draft, applied, .. }, Task::Change { supplied }, Action::Apply) => {
                supplied.parse(draft)?;
                if *applied != *draft {
                    applied.clone_from(draft);
                    updated.checked = false;
                }
            }
            (Scope::Facility { plan }, Task::Facility { .. }, Action::Facility { plan: next }) => {
                if *plan != next {
                    *plan = next;
                    updated.checked = false;
                }
            }
            (
                Scope::Credentials { policy },
                Task::Credentials { .. },
                Action::Credentials { policy: next },
            ) => {
                if *policy != next {
                    *policy = next;
                    updated.checked = false;
                }
            }
            (
                Scope::VpnDesign { drafts, .. },
                Task::VpnDesign { .. },
                Action::VpnDrafts { drafts: next },
            ) => {
                if *drafts != next {
                    *drafts = next;
                    updated.checked = false;
                }
            }
            (Scope::VpnDesign { drafts, applied }, Task::VpnDesign { .. }, Action::ApplyVpn) => {
                let profiles = [
                    drafts[0]
                        .profile()
                        .map_err(|error| format!("Profile 1. {error}"))?,
                    drafts[1]
                        .profile()
                        .map_err(|error| format!("Profile 2. {error}"))?,
                ];
                if *applied != profiles {
                    *applied = profiles;
                    updated.checked = false;
                }
            }
            (_, _, Action::Check) => updated.checked = true,
            (_, _, Action::Reset) => updated = Self::new(task),
            _ => return Err("Use the controls for this security activity.".into()),
        }
        updated.validate(task)?;
        *self = updated;
        Ok(())
    }
}
