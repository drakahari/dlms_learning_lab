//! Opt-in attribute Modify authorization over established protected identities.
//! No server/protocol or host execution. Legacy grants/password APIs are unchanged.
use super::{Authenticated, ProtectedRealm};
use crate::{ldap_access as acl, ldap_changes as changes, ldap_ldif::Change, ldap_query::Dn};
use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Authentication(super::Error),
    Request(changes::Error),
    Policy(acl::Error),
    Denied {
        attribute: String,
        decision: acl::Decision,
    },
    Unsupported,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authentication(e) => write!(f,"{e}"),
            Self::Request(e) => write!(f,"{e}"),
            Self::Policy(e) => write!(f,"{e}"),
            Self::Denied{attribute,decision} => write!(f,"Write access to {attribute} is denied. {} No attributes changed.",decision.explanation()),
            Self::Unsupported => f.write_str("This ACL adapter supports only mail/description Modify requests. Add, delete, rename, schema and password operations remain separate unsupported routes; nothing changed."),
        }
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub request: changes::Applied,
    pub decisions: Vec<(String, acl::Decision)>,
}
impl ProtectedRealm {
    /// A separate, explicitly selected ACL route. Old scoped grants are neither
    /// broadened nor unioned with this policy. Both routes keep the fixed schema.
    /// root_dn is a source-held configured backend identity, never a draft proof.
    pub fn modify_with_acl(
        &mut self,
        auth: &Authenticated,
        policy: &acl::Policy,
        root_dn: Option<&Dn>,
        change: &Change,
    ) -> Result<Applied, Error> {
        self.endpoint
            .accepts(&auth.protection)
            .map_err(|e| Error::Authentication(e.into()))?;
        if !std::sync::Arc::ptr_eq(&self.realm.authority, &auth.session.authority) {
            return Err(Error::Authentication(
                super::super::Error::ForeignSession.into(),
            ));
        }
        let principal = self.realm.authority.identities[auth.session.index]
            .grant
            .principal()
            .clone();
        if !super::person(&self.realm.directory, &principal) {
            return Err(Error::Authentication(
                super::super::Error::InvalidCredentials.into(),
            ));
        }
        let Change::Modify { dn, edits } = change else {
            return Err(Error::Unsupported);
        };
        // Validate complete shape/schema/existence on bounded staged state before
        // inspecting its now-bounded attributes. Never mutate live data on failure.
        let grant = changes::Grant::new(
            principal.clone(),
            self.realm.directory.read().suffix().clone(),
            false,
            false,
            &["mail", "description"],
        )
        .map_err(Error::Request)?;
        let mut staged = self.realm.directory.clone();
        let request = staged.apply(change, &grant).map_err(Error::Request)?;
        let subject = acl::Subject::Authenticated(principal);
        let mut decisions: Vec<(String, acl::Decision)> = Vec::new();
        for edit in edits {
            let name = edit.attribute.name.to_ascii_lowercase();
            if decisions.iter().any(|(seen, _)| seen == &name) {
                continue;
            }
            let decision = policy
                .evaluate(&subject, dn, &name, acl::Request::Write, root_dn)
                .map_err(Error::Policy)?;
            if !decision.allowed {
                return Err(Error::Denied {
                    attribute: name,
                    decision,
                });
            }
            decisions.push((name, decision));
        }
        // The only live mutation follows every attribute's successful ACL check.
        self.realm.directory = staged;
        Ok(Applied { request, decisions })
    }
}
