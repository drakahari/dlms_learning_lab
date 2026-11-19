//! Source-bound SSH policy/path/trust decisions. No actual SSH or cryptography.
use crate::{
    filesystem::Tree,
    ssh_identity::{KnownHosts, PublicKey, Trust},
    ssh_key_access::{self, Authorization, Finding},
    ssh_policy::{self, Method, Policy},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub host: String,
    pub host_key: PublicKey,
    pub identities: BTreeMap<String, PublicKey>,
    pub initial_tree: Tree,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,32>")]
    pub editable_metadata: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,16>")]
    pub editable_text: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub version: u32,
    pub tree: Tree,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub known_hosts: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub saved_config: String,
    #[serde(
        default,
        deserialize_with = "crate::apache_static_workspace::option_text::<_,16384>"
    )]
    pub active_config: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub host: String,
    pub user: String,
    pub identity: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    UnknownHost,
    ChangedHostKey,
    NoActivePolicy,
    UnknownAccount,
    MethodDisabled,
    KeyRejected { findings: Vec<Finding> },
    ConditionsMet { path: String },
}
impl Outcome {
    pub fn permitted(&self) -> bool {
        matches!(self, Self::ConditionsMet { .. })
    }
    pub fn explanation(&self) -> String {
        match self {
  Self::UnknownHost=>Trust::UnknownHost.explanation().into(),Self::ChangedHostKey=>Trust::ChangedKey.explanation().into(),
  Self::NoActivePolicy=>"No supported server policy is active. Check the saved configuration, then apply it in this virtual workspace.".into(),
  Self::UnknownAccount=>"The requested account is absent from the supplied server inventory.".into(),
  Self::MethodDisabled=>"The active server policy does not offer public-key authentication to this account. Check AllowUsers, PermitRootLogin and the enabled authentication methods.".into(),
  Self::KeyRejected{findings}=>if findings.is_empty(){"The active policy has no authorized-key file locations.".into()}else{findings.iter().map(Finding::explanation).collect::<Vec<_>>().join("\n")},
  Self::ConditionsMet{..}=>"The supplied server identity, active policy, public identity and key-file checks permit this request in the simulation. No connection, private-key proof or cryptographic signature was performed.".into(),
 }
    }
}
fn editable(path: &str, tree: &Tree) -> bool {
    tree.users.keys().filter(|u| u.as_str() != "root").any(|u| {
        let h = format!("/home/{u}");
        [
            h.clone(),
            format!("{h}/.ssh"),
            format!("{h}/.ssh/authorized_keys"),
            format!("{h}/.ssh/authorized_keys2"),
        ]
        .iter()
        .any(|p| p == path)
    })
}
impl Scope {
    pub fn validate(&self) -> Result<(), String> {
        crate::ssh_identity::target(&self.host)?;
        if !self.initial_tree.valid()
            || self.initial_tree.command_location.is_some()
            || self.identities.is_empty()
            || self.identities.len() > 16
            || self.identities.keys().any(|k| !ssh_policy::user(k))
            || self.editable_metadata.len() > 32
            || self.editable_text.len() > 16
        {
            return Err("Invalid bounded supplied SSH inventory.".into());
        }
        for (paths, contents) in [
            (&self.editable_metadata, false),
            (&self.editable_text, true),
        ] {
            let mut seen = std::collections::BTreeSet::new();
            for path in paths {
                if !seen.insert(path)
                    || !editable(path, &self.initial_tree)
                    || !self.initial_tree.nodes.contains_key(path)
                    || contents && self.initial_tree.nodes[path].directory
                {
                    return Err("SSH editable paths must be distinct, supplied ordinary home/key entries; no external or arbitrary paths are enabled.".into());
                }
            }
        }
        Ok(())
    }
    pub fn validate_world(&self, w: &World) -> Result<(), String> {
        self.validate()?;
        if w.version != 1
            || !w.tree.valid()
            || w.known_hosts.len() > 16384
            || w.saved_config.len() > 16384
            || w.active_config
                .as_ref()
                .is_some_and(|t| t.len() > 16384 || Policy::parse(t).is_err())
        {
            return Err("Invalid bounded SSH workspace or active policy.".into());
        }
        let a = &self.initial_tree;
        let b = &w.tree;
        if (
            a.user.as_str(),
            a.home.as_str(),
            a.cwd.as_str(),
            a.previous.as_ref(),
            a.umask,
            &a.users,
            &a.command_location,
        ) != (
            b.user.as_str(),
            b.home.as_str(),
            b.cwd.as_str(),
            b.previous.as_ref(),
            b.umask,
            &b.users,
            &b.command_location,
        ) || a.nodes.keys().ne(b.nodes.keys())
            || a.files.keys().ne(b.files.keys())
        {
            return Err(
                "The supplied user/path inventory cannot be replaced to satisfy an access check."
                    .into(),
            );
        }
        for (path, node) in &a.nodes {
            let current = &b.nodes[path];
            if node.directory != current.directory
                || !self.editable_metadata.contains(path) && node != current
            {
                return Err(format!("Protected virtual path metadata changed: {path}"));
            }
        }
        for (path, text) in &a.files {
            if !self.editable_text.contains(path) && b.files[path] != *text {
                return Err(format!("Protected virtual file changed: {path}"));
            }
        }
        Ok(())
    }
    pub fn evaluate(&self, w: &World, r: &Request) -> Result<Outcome, String> {
        self.validate_world(w)?;
        if r.host != self.host || !ssh_policy::user(&r.user) {
            return Err("Unsupported request: choose the supplied host and a literal account. No other server is contacted.".into());
        }
        let key=self.identities.get(&r.identity).ok_or("Unsupported client identity: choose a source-supplied public identity; possession is an exercise assumption, not verified cryptography.")?;
        match KnownHosts::parse(&w.known_hosts)?.check(&r.host, self.host_key)? {
            Trust::UnknownHost => return Ok(Outcome::UnknownHost),
            Trust::ChangedKey => return Ok(Outcome::ChangedHostKey),
            Trust::Trusted => {}
        }
        let Some(config) = &w.active_config else {
            return Ok(Outcome::NoActivePolicy);
        };
        let policy = Policy::parse(config).map_err(|e| e.to_string())?;
        if !self.initial_tree.users.contains_key(&r.user) {
            return Ok(Outcome::UnknownAccount);
        }
        if !policy.offers(&r.user, Method::PublicKey)? {
            return Ok(Outcome::MethodDisabled);
        }
        if r.user == "root" {
            return Err("The active policy still offers public-key authentication to root. Root key-file checks are outside this activity; this task requires the request to stop at the policy stage.".into());
        }
        match ssh_key_access::check(
            &w.tree,
            &r.user,
            &policy.authorized_files,
            *key,
            policy.strict_modes,
        )? {
            Authorization::Allowed { path } => Ok(Outcome::ConditionsMet { path }),
            Authorization::Denied { findings } => Ok(Outcome::KeyRejected { findings }),
        }
    }
}
impl World {
    pub fn reload(&mut self) -> Result<(), String> {
        Policy::parse(&self.saved_config).map_err(|e| e.to_string())?;
        self.active_config = Some(self.saved_config.clone());
        Ok(())
    }
    pub fn published(&self) -> bool {
        fn normal(mut p: Policy) -> Policy {
            if let Some(users) = &mut p.allow_users {
                users.sort();
                users.dedup();
            }
            p.authorized_files.sort();
            p.authorized_files.dedup();
            p
        }
        let (Some(active), Ok(saved)) = (&self.active_config, Policy::parse(&self.saved_config))
        else {
            return false;
        };
        Policy::parse(active).is_ok_and(|p| normal(p) == normal(saved))
    }
}

/// Expectations identify the intended stage, never just a false boolean. A blocked
/// server-trust check cannot demonstrate key revocation or server-policy denial.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expected {
    Permitted,
    UnknownHost,
    ChangedHost,
    NoActivePolicy,
    AccountAbsent,
    PolicyDenies,
    KeyDenies,
}
impl Expected {
    pub fn matches(self, outcome: &Outcome) -> bool {
        matches!(
            (self, outcome),
            (Self::Permitted, Outcome::ConditionsMet { .. })
                | (Self::UnknownHost, Outcome::UnknownHost)
                | (Self::ChangedHost, Outcome::ChangedHostKey)
                | (Self::NoActivePolicy, Outcome::NoActivePolicy)
                | (Self::AccountAbsent, Outcome::UnknownAccount)
                | (Self::PolicyDenies, Outcome::MethodDisabled)
                | (Self::KeyDenies, Outcome::KeyRejected { .. })
        )
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Permitted => "supplied access conditions permit the request",
            Self::UnknownHost => "the host is not yet trusted",
            Self::ChangedHost => "the presented host key differs from saved trust",
            Self::NoActivePolicy => "no server policy is active",
            Self::AccountAbsent => "the account is absent",
            Self::PolicyDenies => "the active policy refuses this account/method",
            Self::KeyDenies => "the authorized-key/path checks refuse this identity",
        }
    }
}
impl Scope {
    pub fn check_expected(
        &self,
        world: &World,
        request: &Request,
        expected: Expected,
    ) -> (bool, String) {
        match self.evaluate(world, request) {
            Err(message) => (false, format!("Not evaluated: {message}")),
            Ok(outcome) => {
                let pass = expected.matches(&outcome);
                (
                    pass,
                    format!(
                        "{} · Expected: {}. {}",
                        if pass { "Verified" } else { "Needs attention" },
                        expected.description(),
                        outcome.explanation()
                    ),
                )
            }
        }
    }
}
