//! Review an immutable change journal alongside real virtual admission trials.
use crate::{acl, management_path, management_policy};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum RecordKind {
    TicketApproved {
        revision: u16,
    },
    RevisionApplied {
        revision: u16,
    },
    AdmissionObserved {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        trial: String,
        admitted: bool,
    },
    SuccessfulSignin {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        trial: String,
    },
    UnrelatedMaintenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u16,
    pub minute: u16,
    /// The source inventory contains only names for virtual devices.
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub device: String,
    pub kind: RecordKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub client: String,
    pub line: u8,
    pub protocol: management_policy::Protocol,
    pub intended_admission: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub approval: Option<u16>,
    pub application: Option<u16>,
    pub observed_extra_access: Option<u16>,
    pub successful_use_recorded: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub label: &'static str,
    pub passed: bool,
    pub detail: &'static str,
}

pub fn findings(records: &[Record], device: &str, exposed_trial: &str, r: &Report) -> Vec<Finding> {
    let record = |id| {
        records
            .iter()
            .find(|event| Some(event.id) == id && event.device == device)
    };
    let approval = record(r.approval).and_then(|event| {
        if let RecordKind::TicketApproved { revision } = event.kind {
            Some((event.minute, revision))
        } else {
            None
        }
    });
    let applied = record(r.application).and_then(|event| {
        if let RecordKind::RevisionApplied { revision } = event.kind {
            Some((event.minute, revision))
        } else {
            None
        }
    });
    let observed = record(r.observed_extra_access);
    let applied_differently =
        matches!((approval,applied),(Some(a),Some(changed)) if changed.0>=a.0&&changed.1!=a.1);
    let observed_extra = matches!((applied,observed),
        (Some(changed),Some(event))
        if event.minute>=changed.0&&matches!(&event.kind,RecordKind::AdmissionObserved{trial,admitted:true} if trial==exposed_trial));
    let successful_use = applied.is_some_and(|(at, _)| {
        records.iter().any(|event| {
            event.device == device
                && event.minute >= at
                && matches!(&event.kind, RecordKind::SuccessfulSignin { trial } if trial == exposed_trial)
        })
    });
    vec![
        Finding {
            label: "Approved change",
            passed: approval.is_some(),
            detail: "Select the approval record for the affected device.",
        },
        Finding {
            label: "Applied change differs from the ticket",
            passed: applied_differently,
            detail: "Compare the approved and applied revisions on the same device in time order.",
        },
        Finding {
            label: "Observed access boundary",
            passed: observed_extra,
            detail: "Find the post-change test showing that the restricted client could reach the authentication boundary.",
        },
        Finding {
            label: "Sign-in evidence",
            passed: observed_extra && r.successful_use_recorded == Some(successful_use),
            detail: "Before recording successful use, check for a distinct sign-in observation on the affected device after the change.",
        },
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialResult {
    pub expected: bool,
    pub reached: bool,
    pub detail: String,
}

fn explain(observation: &management_path::Observation) -> String {
    let conditions = match observation {
        management_path::Observation::Policy { conditions, .. } => conditions,
        management_path::Observation::PathUnavailable { reason } => {
            return format!("The virtual path is unavailable. {reason}");
        }
    };
    if !conditions.protocol_allowed {
        return format!(
            "The incoming protocol is disabled on line {}.",
            conditions.line
        );
    }
    if conditions.identity_available == Some(false) {
        return "The supplied SSH identity is unavailable.".into();
    }
    if conditions.version_allowed == Some(false) {
        return "The active policy rejects that SSH version.".into();
    }
    let Some(rule) = &conditions.source_decision else {
        return format!("Line {} has no selected source list.", conditions.line);
    };
    match rule.cause {
        acl::Cause::ImplicitDeny => format!(
            "No rule in source list {} permits this client. Its implicit deny applies.",
            rule.list
        ),
        acl::Cause::Matched { row, .. } => {
            let verb = if rule.action == acl::Action::Permit {
                "permits"
            } else {
                "denies"
            };
            format!(
                "Row {} of source list {} {verb} this client.",
                row + 1,
                rule.list
            )
        }
    }
}

pub fn trials(
    network: &management_path::Network,
    policy: &management_policy::Policy,
    cases: &[Trial],
) -> Result<Vec<TrialResult>> {
    if cases.len() > 6 || cases.is_empty() {
        return Err("Supply one to six named virtual management path checks.".into());
    }
    cases
        .iter()
        .map(|case| {
            let observation = network
                .inspect(policy, &case.client, case.line, case.protocol)
                .map_err(|error| error.to_string())?;
            Ok(TrialResult {
                expected: case.intended_admission,
                reached: observation.allowed_to_authenticate(),
                detail: explain(&observation),
            })
        })
        .collect()
}
