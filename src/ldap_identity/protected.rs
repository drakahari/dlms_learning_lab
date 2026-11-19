//! Protected, self-service fictional password state. No real credentials, TLS,
//! password hashing/store, Bind exchange, server, network or host changes.
//! Existing plain Realm APIs and directory attribute operations are unchanged.
use super::{Identity, Realm, Session, password_shape, person};
use crate::{
    ldap_protection as protection,
    ldap_query::{Dn, Source},
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Protection(protection::Error),
    Identity(super::Error),
    ExtensionUnavailable,
    PolicyDenied,
    OtherIdentityUnsupported,
    OldPasswordRequired,
    OldPasswordMismatch,
    NewPasswordRequired,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protection(e) => write!(f,"{e}"),
            Self::Identity(e) => write!(f,"{e}"),
            Self::ExtensionUnavailable => f.write_str("This supplied server does not support the modeled password-change operation. Nothing changed."),
            Self::PolicyDenied => f.write_str("The supplied server policy does not allow this self-service password change. Nothing changed."),
            Self::OtherIdentityUnsupported => f.write_str("This subset changes only the authenticated person's own sample password. Administrative reset and other identity forms are unsupported; nothing changed."),
            Self::OldPasswordRequired => f.write_str("This sample's policy requires the current target password as well as authentication. Nothing changed."),
            Self::OldPasswordMismatch => f.write_str("The supplied old target sample password does not match its current value. Nothing changed."),
            Self::NewPasswordRequired => f.write_str("Provide a supplied replacement sample password. Server-generated passwords are outside this subset; nothing changed."),
        }
    }
}
impl std::error::Error for Error {}
impl From<protection::Error> for Error {
    fn from(e: protection::Error) -> Self {
        Self::Protection(e)
    }
}
impl From<super::Error> for Error {
    fn from(e: super::Error) -> Self {
        Self::Identity(e)
    }
}
type Result<T> = std::result::Result<T, Error>;

/// Authored server-policy facts. Not inferred from ordinary attribute-write rights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub supports_password_modify: bool,
    pub allow_self_change: bool,
    pub require_old_password: bool,
}
/// Opaque established authentication; it stores no submitted password copy and
/// cannot be supplied through a saved draft. A bound context can remain valid
/// after a password change; fresh authentication checks the updated credential.
#[derive(Clone, Debug)]
pub struct Authenticated {
    protection: protection::Proof,
    session: Session,
}

pub struct ProtectedRealm {
    realm: Realm,
    endpoint: protection::Endpoint,
    policy: Policy,
    // This credential state is deliberately separate from LDAP attributes and
    // from the immutable authority that establishes an authenticated identity.
    current: Vec<String>,
}
impl fmt::Debug for ProtectedRealm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProtectedRealm")
            .field("realm", &self.realm)
            .field("endpoint", &self.endpoint)
            .field("policy", &self.policy)
            .field("current", &"[redacted]")
            .finish()
    }
}
impl ProtectedRealm {
    pub fn new(
        endpoint: protection::Endpoint,
        source: &Source,
        identities: &[Identity],
        policy: Policy,
    ) -> Result<Self> {
        // Validate bounds/person/grants before copying any mutable credentials.
        let realm = Realm::new(endpoint.uri(), source, identities)?;
        Ok(Self {
            realm,
            endpoint,
            policy,
            current: identities
                .iter()
                .map(|i| i.sample_password.clone())
                .collect(),
        })
    }
    pub fn uri(&self) -> &str {
        self.realm.uri()
    }
    pub fn directory(&self) -> &crate::ldap_changes::Directory {
        self.realm.directory()
    }
    pub fn open(
        &self,
        target: &str,
        request: protection::Request,
        trust: &protection::Trust,
        sample_time: u64,
    ) -> Result<protection::Connection> {
        Ok(self.endpoint.open(target, request, trust, sample_time)?)
    }
    pub fn authenticate(
        &self,
        connection: &protection::Connection,
        principal: &Dn,
        sample_password: &str,
    ) -> Result<Authenticated> {
        // No credential evaluation precedes this attempt's protection checks.
        let protected = self.endpoint.require_protected(connection)?;
        password_shape(sample_password)?;
        let index = self
            .realm
            .authority
            .identities
            .iter()
            .enumerate()
            .position(|(index, identity)| {
                identity.grant.principal().equivalent(principal)
                    && self.current[index] == sample_password
            })
            .filter(|_| person(&self.realm.directory, principal))
            .ok_or(super::Error::InvalidCredentials)?;
        Ok(Authenticated {
            protection: protected,
            session: Session {
                authority: self.realm.authority.clone(),
                index,
            },
        })
    }
    pub fn change_password(
        &mut self,
        auth: &Authenticated,
        target: Option<&Dn>,
        old_password: Option<&str>,
        new_password: Option<&str>,
    ) -> Result<()> {
        self.endpoint.accepts(&auth.protection)?;
        if !std::sync::Arc::ptr_eq(&self.realm.authority, &auth.session.authority) {
            return Err(super::Error::ForeignSession.into());
        }
        let index = auth.session.index;
        let identity = &self.realm.authority.identities[index];
        if !person(&self.realm.directory, identity.grant.principal()) {
            return Err(super::Error::InvalidCredentials.into());
        }
        if !self.policy.supports_password_modify {
            return Err(Error::ExtensionUnavailable);
        }
        if !self.policy.allow_self_change {
            return Err(Error::PolicyDenied);
        }
        if target.is_some_and(|target| !target.equivalent(identity.grant.principal())) {
            return Err(Error::OtherIdentityUnsupported);
        }
        if self.policy.require_old_password && old_password.is_none() {
            return Err(Error::OldPasswordRequired);
        }
        if let Some(old) = old_password {
            password_shape(old)?;
            if old != self.current[index] {
                return Err(Error::OldPasswordMismatch);
            }
        }
        let new = new_password.ok_or(Error::NewPasswordRequired)?;
        password_shape(new)?;
        // The sole mutation happens after every validation. It never changes
        // directory attributes, other credentials, grants or existing sessions.
        self.current[index] = new.into();
        Ok(())
    }
}

/// Explicit ordered-ACL attribute-modification route; no new lesson yet.
pub mod access;
