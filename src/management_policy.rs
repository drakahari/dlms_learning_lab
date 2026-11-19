//! Explicit per-line metadata policy only: no network, crypto or authentication.
//! Admission means permission to attempt authentication, never a successful login.
use crate::acl;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, net::Ipv4Addr};

pub const VERSION: u32 = 1;
pub const MAX_LINES: usize = 16;
pub const MAX_TOTAL_RULES: usize = 128;
pub const MAX_JSON_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn fail(kind: ErrorKind, message: &str) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn acl_error(e: acl::Error) -> Error {
    let kind = match e.kind {
        acl::ErrorKind::Limit => ErrorKind::Limit,
        acl::ErrorKind::Unsupported | acl::ErrorKind::UnknownList => ErrorKind::Unsupported,
        acl::ErrorKind::Invalid => ErrorKind::Invalid,
    };
    Error {
        kind,
        message: e.message,
    }
}
fn named(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
fn unicast(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified() && !ip.is_multicast() && ip != Ipv4Addr::BROADCAST
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Ssh1,
    Ssh2,
    Telnet,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Versions {
    Version2Only,
    Compatibility,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    pub number: u8,
    pub ssh: bool,
    pub telnet: bool,
    pub source_list: Option<acl::AccessList>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: u32,
    /// Supplied virtual identity name, never key material or a host path.
    pub host_identity: Option<String>,
    pub versions: Versions,
    pub lines: Vec<Line>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt {
    /// Explicitly supplied policy line; no automatic IOS allocation is claimed.
    pub line: u8,
    pub protocol: Protocol,
    pub source: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub line: u8,
    pub protocol_allowed: bool,
    /// Not applicable to Telnet.
    pub identity_available: Option<bool>,
    /// Not applicable to Telnet; does not represent cryptographic negotiation.
    pub version_allowed: Option<bool>,
    pub source_decision: Option<acl::Decision>,
}
impl Assessment {
    pub fn allowed_to_authenticate(&self) -> bool {
        self.protocol_allowed
            && self.identity_available != Some(false)
            && self.version_allowed != Some(false)
            && self
                .source_decision
                .as_ref()
                .is_none_or(|d| d.action == acl::Action::Permit)
    }
}
impl Policy {
    pub fn decode(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > MAX_JSON_BYTES {
            return Err(fail(ErrorKind::Limit, "Management policy exceeds 256 KiB."));
        }
        let policy: Self = serde_json::from_slice(raw).map_err(|_| {
            fail(
                ErrorKind::Invalid,
                "Malformed or unsupported management policy fields or values.",
            )
        })?;
        policy.validate()?;
        Ok(policy)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION {
            return Err(fail(
                ErrorKind::Unsupported,
                "Unsupported management policy version.",
            ));
        }
        if self.lines.is_empty() || self.lines.len() > MAX_LINES {
            return Err(fail(ErrorKind::Limit, "Supply 1–16 explicit policy lines."));
        }
        if self.host_identity.as_ref().is_some_and(|s| !named(s)) {
            return Err(fail(
                ErrorKind::Invalid,
                "Use a 1–64 character lab identity name, not a path or key.",
            ));
        }
        let mut rules = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if line.number > 98 || self.lines[..i].iter().any(|old| old.number == line.number) {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Supplied line numbers must be unique and within 0–98.",
                ));
            }
            if let Some(list) = &line.source_list {
                if list.kind != acl::Kind::Standard {
                    return Err(fail(
                        ErrorKind::Unsupported,
                        "Only standard IPv4 source lists are modeled on these lines.",
                    ));
                }
                list.validate().map_err(acl_error)?;
                rules += list.rules.len();
                if rules > MAX_TOTAL_RULES {
                    return Err(fail(
                        ErrorKind::Limit,
                        "Management policy supports at most 128 total source rules.",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Validate the complete owned replacement before mutating the current policy.
    pub fn replace(&mut self, next: Self) -> Result<(), Error> {
        next.validate()?;
        *self = next;
        Ok(())
    }
    /// Independent conditions, not an asserted ordering of IOS login diagnostics.
    pub fn assess(&self, attempt: Attempt) -> Result<Assessment, Error> {
        self.validate()?;
        if !unicast(attempt.source) {
            return Err(fail(
                ErrorKind::Unsupported,
                "This policy trial requires a supplied IPv4 unicast source.",
            ));
        }
        let line = self
            .lines
            .iter()
            .find(|l| l.number == attempt.line)
            .ok_or_else(|| {
                fail(
                    ErrorKind::Invalid,
                    "The selected policy line was not supplied.",
                )
            })?;
        let ssh = attempt.protocol != Protocol::Telnet;
        let source_decision = if let Some(list) = &line.source_list {
            let policy = acl::Policy {
                schema_version: 1,
                lists: BTreeMap::from([("management-source".into(), list.clone())]),
            };
            // Standard selectors inspect only source. The other valid metadata is
            // deliberately neutral and implies no real destination or TCP session.
            Some(
                policy
                    .evaluate(
                        "management-source",
                        &acl::Packet {
                            source: attempt.source,
                            destination: Ipv4Addr::new(192, 0, 2, 1),
                            protocol: acl::PacketProtocol::Other { number: 0 },
                        },
                    )
                    .map_err(acl_error)?,
            )
        } else {
            None
        };
        Ok(Assessment {
            line: line.number,
            protocol_allowed: if ssh { line.ssh } else { line.telnet },
            identity_available: ssh.then_some(self.host_identity.is_some()),
            version_allowed: ssh.then_some(
                attempt.protocol == Protocol::Ssh2 || self.versions == Versions::Compatibility,
            ),
            source_decision,
        })
    }
}
