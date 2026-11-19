//! Bounded training-token local-case authentication; no crypto or host execution.
use crate::{management_path as path, management_policy as management};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_RECORDS: usize = 16;
pub const MAX_POLICY_BYTES: usize = 32 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    LocalCase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authorization {
    Local,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Privilege {
    User,
    Administrator,
}
impl Privilege {
    pub fn level(self) -> u8 {
        match self {
            Self::User => 1,
            Self::Administrator => 15,
        }
    }
}
pub(crate) fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
}
fn valid_token(s: &str) -> bool {
    !s.is_empty() && s.len() <= 128 && s.bytes().all(|c| (0x20..=0x7e).contains(&c))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    /// Ordinary lesson data, never a real credential or an encoded Cisco hash.
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub training_token: String,
    pub privilege: Privilege,
}
impl User {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_name(&self.name) {
            return Err(
                "Local account names need 1–32 ASCII letters, digits, dots, underscores or dashes."
                    .into(),
            );
        }
        if !valid_token(&self.training_token) {
            return Err("Every local record needs a supplied training token: 1–128 printable ASCII characters. Missing/no-password defaults are outside this subset.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: u32,
    pub method: Method,
    pub exec_authorization: Authorization,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,User,16>")]
    pub users: Vec<User>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    UnknownUser,
    WrongToken,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Rejected(Rejection),
    Authenticated { user: String, privilege: Privilege },
}
impl Policy {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.users.len() > MAX_RECORDS {
            return Err("Local access supports version 1 and at most 16 named records.".into());
        }
        let mut names = BTreeSet::new();
        for u in &self.users {
            u.validate()?;
            if !names.insert(&u.name) {
                return Err("Local account names must be unique. The explicit local-case method keeps letter case significant.".into());
            }
        }
        if !crate::cron_workspace::within(self, MAX_POLICY_BYTES) {
            return Err("Local access policy exceeds 32 KiB.".into());
        }
        Ok(())
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_POLICY_BYTES {
            return Err("Local access policy exceeds 32 KiB.".into());
        }
        let p: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported local access policy.".to_string())?;
        p.validate()?;
        Ok(p)
    }
    pub fn replace_users(&mut self, users: Vec<User>) -> Result<(), String> {
        let candidate = Self {
            users,
            ..self.clone()
        };
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    pub fn authenticate(&self, name: &str, token: &str) -> Result<Decision, String> {
        self.validate()?;
        validate_attempt(name, token)?;
        let Some(user) = self.users.iter().find(|u| u.name == name) else {
            return Ok(Decision::Rejected(Rejection::UnknownUser));
        };
        let expected = &user.training_token;
        if token != expected.as_str() {
            return Ok(Decision::Rejected(Rejection::WrongToken));
        }
        Ok(Decision::Authenticated {
            user: user.name.clone(),
            privilege: user.privilege,
        })
    }
}
fn validate_attempt(name: &str, token: &str) -> Result<(), String> {
    if !valid_name(name) {
        return Err("Enter a supplied local account name in the supported format before trying authentication.".into());
    }
    // Empty is a wrong submitted token, not a successful missing-token bypass.
    if token.len() > 128 || !token.bytes().all(|c| (0x20..=0x7e).contains(&c)) {
        return Err(
            "The submitted training token is outside the supported length or character range."
                .into(),
        );
    }
    Ok(())
}
#[derive(Clone, Debug)]
pub enum Observation {
    PathUnavailable {
        reason: String,
    },
    PolicyRejected {
        path: crate::ipv4_routing::Probe,
        conditions: management::Assessment,
    },
    Local {
        path: crate::ipv4_routing::Probe,
        decision: Decision,
    },
}
/// Compose pre-authentication routing/policy with the explicitly supplied local
/// method. No IP-path or policy rejection becomes a wrong-password observation.
pub fn inspect(
    network: &path::Network,
    incoming: &management::Policy,
    local: &Policy,
    probe: management::Attempt,
    client: &str,
    name: &str,
    token: &str,
) -> Result<Observation, String> {
    local.validate()?;
    validate_attempt(name, token)?;
    network.validate().map_err(|e| e.to_string())?;
    // The Network owns the source binding; a caller cannot spoof the probe's source.
    let binding = network
        .clients
        .iter()
        .find(|c| c.id == client)
        .ok_or("Select a supplied management client.")?;
    let source = network
        .topology
        .devices
        .get(&binding.binding.device)
        .and_then(|d| d.interfaces.get(&binding.binding.interface))
        .and_then(|i| i.address.as_ref())
        .ok_or("The selected management client has no supplied source address.")?
        .address;
    if source != probe.source {
        return Err("The local-access trial source must match its supplied client binding.".into());
    }
    match network
        .inspect(incoming, client, probe.line, probe.protocol)
        .map_err(|e| e.to_string())?
    {
        path::Observation::PathUnavailable { reason } => {
            Ok(Observation::PathUnavailable { reason })
        }
        path::Observation::Policy { path, conditions } if !conditions.allowed_to_authenticate() => {
            Ok(Observation::PolicyRejected { path, conditions })
        }
        path::Observation::Policy { path, .. } => Ok(Observation::Local {
            path,
            decision: local.authenticate(name, token)?,
        }),
    }
}
