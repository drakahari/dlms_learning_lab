//! Fictional, bounded training identities for an in-process directory.
//! This is not a network Bind, real credential store, TLS stack or OpenLDAP ACL engine.
use crate::{
    ldap_changes::{self as changes, Applied, Grant},
    ldap_client,
    ldap_query::{Dn, Source},
};
use std::{fmt, sync::Arc};

pub const MAX_IDENTITIES: usize = 8;
pub const MAX_SAMPLE_PASSWORD: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidCredentials,
    ForeignSession,
    Unsupported(String),
    Limit(String),
    Invalid(String),
    Request(changes::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(
                f,
                "InvalidCredentials (49): the supplied training identity and sample password do not match."
            ),
            Self::ForeignSession => write!(
                f,
                "InsufficientAccessRights (50): this session belongs to a different simulated directory attempt."
            ),
            Self::Unsupported(s) => write!(f, "Unsupported: {s}"),
            Self::Limit(s) => write!(f, "Limit: {s}"),
            Self::Invalid(s) => write!(f, "Invalid training context: {s}"),
            Self::Request(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn password_shape(raw: &str) -> Result<()> {
    if raw.len() > MAX_SAMPLE_PASSWORD {
        return Err(Error::Limit(
            "Sample passwords allow at most 128 bytes.".into(),
        ));
    }
    if raw.is_empty() || !raw.bytes().all(|b| (32..=126).contains(&b)) {
        return Err(Error::Unsupported("Use a nonempty printable ASCII sample password supplied by the lesson. Anonymous, unauthenticated and internationalized authentication are outside this subset.".into()));
    }
    Ok(())
}

/// Author-supplied fictional facts; never deserialize learner rights or secrets.
#[derive(Clone)]
pub struct Identity {
    grant: Grant,
    sample_password: String,
}
impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity")
            .field("grant", &self.grant)
            .field("sample_password", &"[redacted]")
            .finish()
    }
}
impl Identity {
    pub fn new(
        principal: Dn,
        sample_password: &str,
        subtree: Dn,
        add: bool,
        delete: bool,
        attributes: &[&str],
    ) -> Result<Self> {
        password_shape(sample_password)?;
        let grant =
            Grant::new(principal, subtree, add, delete, attributes).map_err(Error::Request)?;
        Ok(Self {
            grant,
            sample_password: sample_password.into(),
        })
    }
}

#[derive(Debug)]
struct Authority {
    identities: Vec<Identity>,
}
/// Opaque attempt-local proof of a successful fictional name/password check.
/// It retains no submitted password copy and cannot be restored from learner input.
#[derive(Clone, Debug)]
pub struct Session {
    authority: Arc<Authority>,
    index: usize,
}
#[derive(Debug)]
pub struct Realm {
    uri: String,
    directory: changes::Directory,
    authority: Arc<Authority>,
}
fn person(directory: &changes::Directory, principal: &Dn) -> bool {
    directory.read().entries().iter().any(|e| {
        e.dn().equivalent(principal)
            && e.attributes().iter().any(|a| {
                a.name.eq_ignore_ascii_case("objectclass")
                    && a.values.iter().any(|v| v.eq_ignore_ascii_case("person"))
            })
    })
}
impl Realm {
    pub fn new(uri: &str, source: &Source, identities: &[Identity]) -> Result<Self> {
        if identities.is_empty() || identities.len() > MAX_IDENTITIES {
            return Err(Error::Limit(
                "A training directory needs 1–8 fictional identities.".into(),
            ));
        }
        let uri = ldap_client::uri_key(uri).map_err(|e| Error::Invalid(e.to_string()))?;
        let directory = changes::Directory::new(source).map_err(Error::Request)?;
        for (i, identity) in identities.iter().enumerate() {
            let principal = identity.grant.principal();
            if identities[..i]
                .iter()
                .any(|old| old.grant.principal().equivalent(principal))
            {
                return Err(Error::Invalid(
                    "Each fictional identity must have a distinct distinguished name.".into(),
                ));
            }
            if !person(&directory, principal)
                || !directory
                    .read()
                    .entries()
                    .iter()
                    .any(|e| e.dn().equivalent(identity.grant.subtree()))
            {
                return Err(Error::Invalid(
                    "An identity must name an existing person and an existing delegated subtree."
                        .into(),
                ));
            }
        }
        Ok(Self {
            uri,
            directory,
            authority: Arc::new(Authority {
                identities: identities.to_vec(),
            }),
        })
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn directory(&self) -> &changes::Directory {
        &self.directory
    }
    /// A fresh utility invocation gets a new session; no rebind API is implied.
    pub fn authenticate(&self, principal: &Dn, sample_password: &str) -> Result<Session> {
        password_shape(sample_password)?;
        let index = self
            .authority
            .identities
            .iter()
            .position(|i| {
                i.grant.principal().equivalent(principal) && i.sample_password == sample_password
            })
            .filter(|_| person(&self.directory, principal))
            .ok_or(Error::InvalidCredentials)?;
        Ok(Session {
            authority: self.authority.clone(),
            index,
        })
    }
    pub fn apply(
        &mut self,
        session: &Session,
        change: &crate::ldap_ldif::Change,
    ) -> Result<Applied> {
        if !Arc::ptr_eq(&session.authority, &self.authority) {
            return Err(Error::ForeignSession);
        }
        self.directory
            .apply(change, &self.authority.identities[session.index].grant)
            .map_err(Error::Request)
    }
}

/// Opt-in protected self-service credential model; no registered lesson yet.
pub mod protected;
