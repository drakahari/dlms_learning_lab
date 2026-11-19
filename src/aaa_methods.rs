//! Bounded ordered training-identity methods; no real AAA protocol or host calls.
use crate::local_access as local;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_METHODS: usize = 4;
pub const MAX_SERVICES: usize = 4;
pub const MAX_INPUT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Method {
    LocalCase,
    TrainingService {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        name: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Method,4>")]
    pub methods: Vec<Method>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub training_token: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingService {
    /// Explicit fixture state, not a real socket/timeout or guessed packet loss.
    pub available: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Identity,16>")]
    pub identities: Vec<Identity>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Services {
    pub version: u32,
    #[serde(deserialize_with = "service_map")]
    pub named: BTreeMap<String, TrainingService>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Unavailable,
    Rejected(local::Rejection),
    Authenticated { user: String },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub method: Method,
    pub reply: Reply,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authentication {
    pub steps: Vec<Step>,
    pub outcome: Reply,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecAuthorization {
    NotAuthenticated,
    MissingLocalRecord {
        user: String,
    },
    Authorized {
        user: String,
        privilege: local::Privilege,
    },
}
fn method_key(m: &Method) -> String {
    match m {
        Method::LocalCase => "local-case".into(),
        Method::TrainingService { name } => format!("service:{name}"),
    }
}
impl Policy {
    pub fn validate(&self, services: &Services) -> Result<(), String> {
        services.validate()?;
        if self.version != 1 || self.methods.is_empty() || self.methods.len() > MAX_METHODS {
            return Err("Supply version 1 and one to four explicit authentication methods.".into());
        }
        let mut seen = BTreeSet::new();
        for m in &self.methods {
            if !seen.insert(method_key(m)) {
                return Err("State each method once in this simulator's ordered list.".into());
            }
            if let Method::TrainingService { name } = m
                && (!local::valid_name(name) || !services.named.contains_key(name))
            {
                return Err("Select a supplied training identity service; a missing definition is not an outage.".into());
            }
        }
        Ok(())
    }
    pub fn from_json(raw: &str, services: &Services) -> Result<Self, String> {
        if raw.len() > MAX_INPUT_BYTES {
            return Err("The authentication policy exceeds 64 KiB.".into());
        }
        let p: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported authentication policy.".to_string())?;
        p.validate(services)?;
        Ok(p)
    }
}
impl Services {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.named.len() > MAX_SERVICES {
            return Err("Supply version 1 and at most four training identity services.".into());
        }
        for (name, service) in &self.named {
            if !local::valid_name(name) || service.identities.len() > local::MAX_RECORDS {
                return Err(
                    "Use a supported service name and at most sixteen identity records.".into(),
                );
            }
            let mut names = BTreeSet::new();
            for record in &service.identities {
                local::User {
                    name: record.name.clone(),
                    training_token: record.training_token.clone(),
                    privilege: local::Privilege::User,
                }
                .validate()?;
                if !names.insert(&record.name) {
                    return Err("Names within a training service must be unique; letter case is significant in this declared fixture.".into());
                }
            }
        }
        if !crate::cron_workspace::within(self, MAX_INPUT_BYTES) {
            return Err("Training identity services exceed 64 KiB.".into());
        }
        Ok(())
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_INPUT_BYTES {
            return Err("Training identity services exceed 64 KiB.".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported training identity services.".to_string())?;
        s.validate()?;
        Ok(s)
    }
}
pub fn authenticate(
    policy: &Policy,
    services: &Services,
    local: &local::Policy,
    name: &str,
    token: &str,
) -> Result<Authentication, String> {
    policy.validate(services)?;
    local.validate()?;
    // Preserve the existing supported attempt format, including an empty wrong token.
    if !local::valid_name(name)
        || token.len() > 128
        || !token.bytes().all(|c| (0x20..=0x7e).contains(&c))
    {
        return Err("Use a supplied supported account name and training word.".into());
    }
    let mut steps = Vec::new();
    for method in &policy.methods {
        let reply = match method {
            Method::LocalCase => match local.authenticate(name, token)? {
                local::Decision::Rejected(reason) => Reply::Rejected(reason),
                local::Decision::Authenticated { user, .. } => Reply::Authenticated { user },
            },
            Method::TrainingService { name: service } => {
                let s = &services.named[service];
                if !s.available {
                    Reply::Unavailable
                } else {
                    match s.identities.iter().find(|u| u.name == name) {
                        None => Reply::Rejected(local::Rejection::UnknownUser),
                        Some(u) if token != u.training_token => {
                            Reply::Rejected(local::Rejection::WrongToken)
                        }
                        Some(u) => Reply::Authenticated {
                            user: u.name.clone(),
                        },
                    }
                }
            }
        };
        steps.push(Step {
            method: method.clone(),
            reply: reply.clone(),
        });
        // Only an unavailable method advances. A returned rejection is terminal.
        if reply != Reply::Unavailable {
            return Ok(Authentication {
                steps,
                outcome: reply,
            });
        }
    }
    Ok(Authentication {
        steps,
        outcome: Reply::Unavailable,
    })
}
/// Explicit local EXEC authorization, separately from authentication. This does
/// not grant commands, model remote authorization attributes, or perform accounting.
pub fn authorize_local_exec(
    local: &local::Policy,
    authentication: &Authentication,
) -> Result<ExecAuthorization, String> {
    local.validate()?;
    let Reply::Authenticated { user } = &authentication.outcome else {
        return Ok(ExecAuthorization::NotAuthenticated);
    };
    Ok(match local.users.iter().find(|u| &u.name == user) {
        Some(record) => ExecAuthorization::Authorized {
            user: record.name.clone(),
            privilege: record.privilege,
        },
        None => ExecAuthorization::MissingLocalRecord { user: user.clone() },
    })
}

fn service_map<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, TrainingService>, D::Error> {
    use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
    struct Name;
    impl<'de> DeserializeSeed<'de> for Name {
        type Value = String;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
            crate::cron::text::<D, 32>(d)
        }
    }
    struct Services;
    impl<'de> Visitor<'de> for Services {
        type Value = BTreeMap<String, TrainingService>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most four uniquely named training services")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some(name) = a.next_key_seed(Name)? {
                if values.len() == MAX_SERVICES || values.contains_key(&name) {
                    return Err(A::Error::custom("Too many or duplicate training services"));
                }
                values.insert(name, a.next_value()?);
            }
            Ok(values)
        }
    }
    d.deserialize_map(Services)
}
