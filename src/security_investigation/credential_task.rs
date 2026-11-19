//! Independent proposal and sign-in cases for a simulated credential policy.
use super::{
    credentials::{PasswordCheck, PasswordProperties, Policy, SecondFactor, SignIn, SignInCheck},
    workspace::{Outcome, label},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Allow,
    Deny,
    ReplaceFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum Input {
    Proposal { supplied: PasswordProperties },
    SignIn { supplied: SignIn },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    #[serde(deserialize_with = "crate::cron::text::<_,72>")]
    pub name: String,
    pub input: Input,
    pub expected: Expected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialTask {
    pub starter: Policy,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Trial,12>")]
    pub trials: Vec<Trial>,
}

impl Expected {
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Allow => "Allow",
            Self::Deny => "Deny",
            Self::ReplaceFirst => "Require replacement",
        }
    }
}

impl Trial {
    pub fn test(&self, policy: Policy) -> (Expected, &'static str) {
        match self.input {
            Input::Proposal { supplied } => {
                let result = policy.proposed_password(supplied);
                let actual = if result == PasswordCheck::Accepted {
                    Expected::Allow
                } else {
                    Expected::Deny
                };
                (actual, result.describe())
            }
            Input::SignIn { supplied } => {
                let result = policy.sign_in(supplied);
                let actual = match result {
                    SignInCheck::Admitted => Expected::Allow,
                    SignInCheck::ReplacementRequired => Expected::ReplaceFirst,
                    _ => Expected::Deny,
                };
                (actual, result.describe(supplied.second_proof))
            }
        }
    }

    pub fn outcome(&self, policy: Policy) -> Outcome {
        let (observed, reason) = self.test(policy);
        let extra = if observed == Expected::Allow && self.expected != Expected::Allow {
            match self.input {
                Input::Proposal { supplied } if supplied.listed => {
                    " Enable the weak or compromised password check."
                }
                Input::Proposal { .. } => " Revisit the minimum length.",
                Input::SignIn { supplied } if supplied.compromise_reported => {
                    " A confirmed compromise needs a replacement before sign-in."
                }
                Input::SignIn { .. } => {
                    " Check whether this trial proves possession of its authenticator."
                }
            }
        } else {
            ""
        };
        Outcome::new(
            &self.name,
            self.expected == observed,
            format!(
                "Expected {}. Observed {}. {reason}{extra}",
                self.expected.describe(),
                observed.describe()
            ),
        )
    }
}

impl CredentialTask {
    pub fn outcomes(&self, policy: Policy) -> Vec<Outcome> {
        self.trials
            .iter()
            .map(|trial| trial.outcome(policy))
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        self.starter.validate()?;
        let mut names = BTreeSet::new();
        let expected = [Expected::Allow, Expected::Deny, Expected::ReplaceFirst];
        if !(6..=12).contains(&self.trials.len())
            || self
                .trials
                .iter()
                .any(|trial| !label(&trial.name, 72) || !names.insert(&trial.name))
            || !expected
                .iter()
                .all(|value| self.trials.iter().any(|trial| trial.expected == *value))
            || self.outcomes(self.starter).iter().all(|row| row.passed)
        {
            return Err("Provide distinct allowed, denied and replacement trials, and a starter that needs repair.".into());
        }
        let equivalent = (8..=64)
            .filter(|minimum_length| {
                let repaired = Policy {
                    minimum_length: *minimum_length,
                    reject_listed_passwords: true,
                    replace_after_compromise: true,
                    second_factor: SecondFactor::BoundAuthenticator,
                };
                self.trials
                    .iter()
                    .all(|trial| trial.test(repaired).0 == trial.expected)
            })
            .take(2)
            .count();
        if equivalent < 2 {
            return Err("Provide at least two minimum lengths that satisfy the authored requirements together.".into());
        }
        Ok(())
    }
}
