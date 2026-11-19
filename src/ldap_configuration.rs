//! In-process directory-backed policy replacement over supplied config entries.
//! This is not a slapd process, filesystem, schema loader or authentication API.
use crate::{
    ldap_access as acl,
    ldap_ldif::configuration as codec,
    ldap_query::{Dn, Scope},
};
pub use codec::{Backend, Modification, Name};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize)]
#[serde(transparent)]
struct BoundedDn(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] String);
#[derive(Deserialize)]
#[serde(transparent)]
struct BoundedAccess(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] String);
fn optional_dn<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    Option::<BoundedDn>::deserialize(d).map(|value| value.map(|n| n.0))
}
fn access_values<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    crate::cron_workspace::sequence::<_, BoundedAccess, 16>(d)
        .map(|values| values.into_iter().map(|n| n.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseSource {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub name: String,
    #[serde(default, deserialize_with = "optional_dn")]
    pub suffix: Option<String>,
    #[serde(default, deserialize_with = "optional_dn")]
    pub root_dn: Option<String>,
    #[serde(deserialize_with = "access_values")]
    pub access: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,DatabaseSource,4>")]
    pub databases: Vec<DatabaseSource>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Ldif(crate::ldap_ldif::Diagnostic),
    Invalid(&'static str),
    Unsupported(&'static str),
    Limit(&'static str),
    NoSuchObject,
    Policy(acl::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ldif(e)=>write!(f,"{e}"),
            Self::Invalid(s)=>write!(f,"Invalid configuration: {s}"),
            Self::Unsupported(s)=>write!(f,"Unsupported configuration: {s}"),
            Self::Limit(s)=>write!(f,"Configuration limit: {s}"),
            Self::NoSuchObject=>f.write_str("No such configuration entry. Choose its complete name from the supplied inventory; this request changed nothing."),
            Self::Policy(e)=>write!(f,"Access value: {e}"),
        }
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn text(s: &str, max: usize) -> bool {
    s.len() <= max && s.bytes().all(|b| (32..=126).contains(&b))
}
// Complete replacements accept either all implicit tags or a complete explicit
// set. Arbitrary insert/delete/mixed indexing is deliberately not approximated.
fn ordered(raw: &[String]) -> Result<Vec<String>> {
    if raw.len() > 16 || raw.iter().any(|s| s.len() > 1024) {
        return Err(Error::Limit("At most16access values of1024bytes each."));
    }
    let explicit = raw.first().is_some_and(|s| s.starts_with('{'));
    let mut values = Vec::with_capacity(raw.len());
    for (position, value) in raw.iter().enumerate() {
        if !text(value, 1024) {
            return Err(Error::Unsupported(
                "Access values must be printable ASCII; LDIF folding supplies logical lines.",
            ));
        }
        if value.starts_with('{') != explicit {
            return Err(Error::Unsupported(
                "Use either all unindexed values or a complete set of explicit {0}, {1}, … tags; mixed tags are outside this subset.",
            ));
        }
        let (index, rule) = if explicit {
            let (index, rule) =
                value
                    .strip_prefix('{')
                    .unwrap()
                    .split_once('}')
                    .ok_or(Error::Invalid(
                        "Close each ordered {N}tag before its to expression.",
                    ))?;
            let n = index.parse::<usize>().map_err(|_| {
                Error::Invalid("An access index must be a nonnegative whole number.")
            })?;
            if index != n.to_string() || n >= raw.len() {
                return Err(Error::Unsupported(
                    "A complete replacement uses contiguous canonical tags starting at {0}; gaps and other index forms are outside this subset.",
                ));
            }
            (n, rule)
        } else {
            (position, value.as_str())
        };
        if values.iter().any(|(n, _)| *n == index) {
            return Err(Error::Invalid("An ordered access index was repeated."));
        }
        if !rule.starts_with("to ") && !rule.starts_with("to\t") {
            return Err(Error::Invalid(
                "An olcAccess value begins with to; omit the static access keyword.",
            ));
        }
        // Validate each value as exactly one static rule, not injected lines.
        acl::Policy::parse(&format!("access {rule}\n")).map_err(Error::Policy)?;
        values.push((index, rule.to_owned()));
    }
    values.sort_by_key(|(n, _)| *n);
    Ok(values
        .into_iter()
        .map(|(n, s)| format!("{{{n}}}{s}"))
        .collect())
}
fn rules(values: &[String]) -> String {
    values
        .iter()
        .map(|s| format!("access {}\n", s.split_once('}').unwrap().1))
        .collect()
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Database {
    source: DatabaseSource,
    name: Name,
    suffix: Option<Dn>,
    root_dn: Option<Dn>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    databases: Vec<Database>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub name: Name,
    pub before: Vec<String>,
    pub after: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentRun {
    pub applied: Vec<Applied>,
    pub error: Option<Error>,
}
impl Configuration {
    /// Given entries and a given already-established configuration administrator
    /// are scenario facts. This API performs no Bind, SASL or credential check.
    pub fn new(source: &Source) -> Result<Self> {
        if source.version != 1 {
            return Err(Error::Unsupported(
                "Only configuration source version1is supported.",
            ));
        }
        if !(3..=4).contains(&source.databases.len()) {
            return Err(Error::Limit(
                "Supply one frontend, one config and one/two data backends.",
            ));
        }
        let mut databases: Vec<Database> = Vec::new();
        for s in &source.databases {
            let name = Name::parse(&s.name).map_err(Error::Ldif)?;
            if matches!(name, Name::Global) {
                return Err(Error::Unsupported(
                    "Only the supplied database entries belong in this inventory; global/module/schema settings are not loaded.",
                ));
            }
            if databases.iter().any(|d| d.name == name) {
                return Err(Error::Invalid("Configuration names must be unique."));
            }
            let suffix = s
                .suffix
                .as_deref()
                .map(Dn::parse)
                .transpose()
                .map_err(|_| Error::Invalid("Use a supported data DN for an MDB suffix."))?;
            let root_dn = s
                .root_dn
                .as_deref()
                .map(Dn::parse)
                .transpose()
                .map_err(|_| Error::Invalid("Use a supported data DN for a supplied rootDN."))?;
            let mdb = matches!(
                name,
                Name::Database {
                    backend: Backend::Mdb,
                    ..
                }
            );
            if mdb != suffix.is_some() || !mdb && root_dn.is_some() {
                return Err(Error::Unsupported(
                    "An MDB needs one supplied data suffix; frontend/config suffixes and administrator credentials are outside this subset.",
                ));
            }
            if let Some(suffix) = &suffix {
                for d in &databases {
                    if let Some(other) = &d.suffix
                        && (suffix.within(other, Scope::Sub) || other.within(suffix, Scope::Sub))
                    {
                        return Err(Error::Unsupported(
                            "Data suffixes must be disjoint; glued/nested backend routing is outside this subset.",
                        ));
                    }
                }
            }
            let access = ordered(&s.access)?;
            databases.push(Database {
                source: DatabaseSource {
                    name: name.text(),
                    suffix: s.suffix.clone(),
                    root_dn: s.root_dn.clone(),
                    access,
                },
                name,
                suffix,
                root_dn,
            });
        }
        if !databases.iter().any(|d| {
            matches!(
                d.name,
                Name::Database {
                    backend: Backend::Frontend,
                    ..
                }
            )
        }) || !databases.iter().any(|d| {
            matches!(
                d.name,
                Name::Database {
                    backend: Backend::Config,
                    ..
                }
            )
        }) {
            return Err(Error::Invalid(
                "The supplied frontend and config database entries must both be present.",
            ));
        }
        let result = Self { databases };
        for d in &result.databases {
            if d.suffix.is_some() {
                result.effective(d)?;
            }
        }
        Ok(result)
    }
    pub fn source(&self) -> Source {
        Source {
            version: 1,
            databases: self.databases.iter().map(|d| d.source.clone()).collect(),
        }
    }
    fn effective(&self, d: &Database) -> Result<acl::Policy> {
        let frontend = self
            .databases
            .iter()
            .find(|d| {
                matches!(
                    d.name,
                    Name::Database {
                        backend: Backend::Frontend,
                        ..
                    }
                )
            })
            .unwrap();
        let combined = rules(&d.source.access) + &rules(&frontend.source.access);
        acl::Policy::parse(&combined).map_err(Error::Policy)
    }
    /// Actual per-backend list followed by the supplied frontend list. RootDN
    /// bypass/default behavior remains in the existing ordered-policy evaluator.
    pub fn policy_for(&self, target: &Dn) -> Result<(acl::Policy, Option<Dn>)> {
        let d = self
            .databases
            .iter()
            .find(|d| {
                d.suffix
                    .as_ref()
                    .is_some_and(|s| target.within(s, Scope::Sub))
            })
            .ok_or(Error::NoSuchObject)?;
        Ok((self.effective(d)?, d.root_dn.clone()))
    }
    pub fn replace(&mut self, request: &Modification) -> Result<Applied> {
        if !matches!(
            request.name,
            Name::Database {
                backend: Backend::Mdb,
                ..
            }
        ) {
            return Err(Error::Unsupported(
                "Only a supplied MDB olcAccess replacement is implemented; no global, config, frontend, module, schema, password or other setting changed.",
            ));
        }
        let at = self
            .databases
            .iter()
            .position(|d| d.name == request.name)
            .ok_or(Error::NoSuchObject)?;
        let after = ordered(&request.values)?;
        let before = self.databases[at].source.access.clone();
        let mut staged = self.clone();
        staged.databases[at].source.access = after.clone();
        staged.effective(&staged.databases[at])?;
        // A single configuration request commits once, only after all values and
        // the complete effective policy pass. Other entries are never rebuilt.
        *self = staged;
        Ok(Applied {
            name: request.name.clone(),
            before,
            after,
        })
    }
    pub fn apply_document(&mut self, raw: &str) -> DocumentRun {
        let requests = match codec::parse(raw) {
            Ok(r) => r,
            Err(e) => {
                return DocumentRun {
                    applied: Vec::new(),
                    error: Some(Error::Ldif(e)),
                };
            }
        };
        let mut applied = Vec::new();
        for request in requests {
            match self.replace(&request) {
                Ok(r) => applied.push(r),
                Err(e) => {
                    return DocumentRun {
                        applied,
                        error: Some(e),
                    };
                }
            }
        }
        DocumentRun {
            applied,
            error: None,
        }
    }
}
