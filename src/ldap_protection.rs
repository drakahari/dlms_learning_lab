//! Bounded, symbolic StartTLS/trust facts for later protected LDAP lessons.
//! No handshake, certificate parsing, encryption, DNS, socket, host clock or file
//! access occurs. This is not a TLS implementation or PKIX verifier. Facts are
//! authored inputs; connection/proof values are opaque and never deserialized.
use crate::ldap_client;
use std::{fmt, sync::Arc};

pub const MAX_NAMES: usize = 4;
pub const MAX_ROOTS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Limit,
    InvalidFacts,
    UnsupportedName,
    UnknownEndpoint,
    StartTlsUnavailable,
    MissingCertificate,
    UntrustedIssuer,
    NameMismatch,
    NotYetValid,
    Expired,
    NoConfidentiality,
    ProtectionRequired,
    ForeignConnection,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Limit => "The sample exceeds its bounded names, trust roots or identifier sizes.",
            Self::InvalidFacts => "The supplied certificate/trust facts are inconsistent or repeated.",
            Self::UnsupportedName => "This model uses exact ASCII DNS names only; IP identities, wildcards, Unicode and other URI schemes are unsupported.",
            Self::UnknownEndpoint => "That URI does not select the supplied virtual directory. No connection was attempted.",
            Self::StartTlsUnavailable => "The supplied server cannot start TLS. Required protection stops this attempt before authentication.",
            Self::MissingCertificate => "The sample has no server certificate facts to verify. Authentication was not attempted.",
            Self::UntrustedIssuer => "The sample certificate issuer is not in the supplied trusted-root selection. Authentication was not attempted.",
            Self::NameMismatch => "The directory host does not match a verified DNS name in the sample certificate. Authentication was not attempted.",
            Self::NotYetValid => "The sample certificate is not yet valid at this sample's stated time. Authentication was not attempted.",
            Self::Expired => "The sample certificate has expired at this sample's stated time. Authentication was not attempted.",
            Self::NoConfidentiality => "The modeled channel provides no confidentiality. Authentication was not attempted.",
            Self::ProtectionRequired => "This operation requires a verified, confidential modeled channel before authentication.",
            Self::ForeignConnection => "This connection belongs to a different simulated endpoint attempt. Establish this attempt's protection again.",
        })
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn key(raw: &str) -> Result<()> {
    if raw.len() > 64 {
        return Err(Error::Limit);
    }
    if raw.is_empty()
        || !raw
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(Error::InvalidFacts);
    }
    Ok(())
}
fn name(raw: &str) -> Result<String> {
    if raw.len() > 253 {
        return Err(Error::Limit);
    }
    if raw.is_empty()
        || !raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        || raw.bytes().all(|b| b.is_ascii_digit() || b == b'.')
    {
        return Err(Error::UnsupportedName);
    }
    // Certificate DNS names and endpoint URIs have separate byte budgets.
    if raw.split('.').any(|part| {
        part.is_empty()
            || part.len() > 63
            || !part.as_bytes()[0].is_ascii_alphanumeric()
            || !part.as_bytes()[part.len() - 1].is_ascii_alphanumeric()
    }) {
        return Err(Error::UnsupportedName);
    }
    Ok(raw.to_ascii_lowercase())
}
fn uri(raw: &str) -> Result<String> {
    if raw.len() > 256 {
        return Err(Error::Limit);
    }
    let normalized = ldap_client::uri_key(raw).map_err(|_| Error::UnsupportedName)?;
    let host = normalized
        .trim_start_matches("ldap://")
        .split(':')
        .next()
        .ok_or(Error::UnsupportedName)?;
    name(host)?;
    Ok(normalized)
}

/// Symbolic source facts, not a real certificate or a learner-provided proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certificate {
    names: Vec<String>,
    issuer: String,
    not_before: u64,
    not_after: u64,
    confidential: bool,
}
impl Certificate {
    pub fn new(
        names: &[&str],
        issuer: &str,
        not_before: u64,
        not_after: u64,
        confidential: bool,
    ) -> Result<Self> {
        if names.len() > MAX_NAMES {
            return Err(Error::Limit);
        }
        if names.is_empty() || not_before > not_after {
            return Err(Error::InvalidFacts);
        }
        key(issuer)?;
        let mut normalized = Vec::new();
        for raw in names {
            let n = name(raw)?;
            if normalized.contains(&n) {
                return Err(Error::InvalidFacts);
            }
            normalized.push(n);
        }
        Ok(Self {
            names: normalized,
            issuer: issuer.into(),
            not_before,
            not_after,
            confidential,
        })
    }
}
/// Selected symbolic CA keys. Never loads a CA file or changes system trust.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trust {
    roots: Vec<String>,
}
impl Trust {
    pub fn new(roots: &[&str]) -> Result<Self> {
        if roots.len() > MAX_ROOTS {
            return Err(Error::Limit);
        }
        let mut result = Vec::new();
        for root in roots {
            key(root)?;
            if result.iter().any(|r| r == root) {
                return Err(Error::InvalidFacts);
            }
            result.push(root.to_string());
        }
        Ok(Self { roots: result })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Plain,
    TryStartTls,
    RequireStartTls,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlainReason {
    NotRequested,
    StartTlsUnavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protection {
    Plain(PlainReason),
    VerifiedConfidential,
}
#[derive(Debug)]
struct Authority {
    uri: String,
}
/// Attempt-local authored endpoint; an identical new fixture is a new authority.
#[derive(Debug)]
pub struct Endpoint {
    authority: Arc<Authority>,
    supports_start_tls: bool,
    certificate: Option<Certificate>,
}
/// Opaque connection outcome. There is no setter or saved-progress input route.
#[derive(Clone, Debug)]
pub struct Connection {
    authority: Arc<Authority>,
    protection: Protection,
}
impl Connection {
    pub fn protection(&self) -> Protection {
        self.protection
    }
}
/// Opaque capability to require before a later sensitive operation. It proves
/// only the declared symbolic checks in one attempt, never real encryption.
#[derive(Clone, Debug)]
pub struct Proof {
    authority: Arc<Authority>,
}
impl Endpoint {
    pub fn new(
        target: &str,
        supports_start_tls: bool,
        certificate: Option<Certificate>,
    ) -> Result<Self> {
        Ok(Self {
            authority: Arc::new(Authority { uri: uri(target)? }),
            supports_start_tls,
            certificate,
        })
    }
    pub fn uri(&self) -> &str {
        &self.authority.uri
    }
    pub fn open(
        &self,
        target: &str,
        request: Request,
        trust: &Trust,
        sample_time: u64,
    ) -> Result<Connection> {
        if uri(target)? != self.authority.uri {
            return Err(Error::UnknownEndpoint);
        }
        let protection = match request {
            Request::Plain => Protection::Plain(PlainReason::NotRequested),
            Request::TryStartTls if !self.supports_start_tls => {
                Protection::Plain(PlainReason::StartTlsUnavailable)
            }
            Request::RequireStartTls if !self.supports_start_tls => {
                return Err(Error::StartTlsUnavailable);
            }
            Request::TryStartTls | Request::RequireStartTls => {
                let c = self.certificate.as_ref().ok_or(Error::MissingCertificate)?;
                if !trust.roots.contains(&c.issuer) {
                    return Err(Error::UntrustedIssuer);
                }
                let host = self
                    .authority
                    .uri
                    .trim_start_matches("ldap://")
                    .split(':')
                    .next()
                    .ok_or(Error::UnsupportedName)?;
                if !c.names.iter().any(|n| n == host) {
                    return Err(Error::NameMismatch);
                }
                if sample_time < c.not_before {
                    return Err(Error::NotYetValid);
                }
                if sample_time > c.not_after {
                    return Err(Error::Expired);
                }
                if !c.confidential {
                    return Err(Error::NoConfidentiality);
                }
                Protection::VerifiedConfidential
            }
        };
        Ok(Connection {
            authority: self.authority.clone(),
            protection,
        })
    }
    pub fn require_protected(&self, connection: &Connection) -> Result<Proof> {
        if !Arc::ptr_eq(&self.authority, &connection.authority) {
            return Err(Error::ForeignConnection);
        }
        if connection.protection != Protection::VerifiedConfidential {
            return Err(Error::ProtectionRequired);
        }
        Ok(Proof {
            authority: self.authority.clone(),
        })
    }
    pub fn accepts(&self, proof: &Proof) -> Result<()> {
        if Arc::ptr_eq(&self.authority, &proof.authority) {
            Ok(())
        } else {
            Err(Error::ForeignConnection)
        }
    }
}

/// Explicit virtual client configuration; existing plain Settings are unchanged.
pub mod client;
