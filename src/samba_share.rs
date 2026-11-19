//! Bounded standalone-share access over existing virtual Unix files. No host I/O.
//! Supplied credential proofs are fixtures, not real SMB authentication/crypto.
use crate::{
    filesystem::{Access, Tree},
    samba_share_config as config,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_FILES_BYTES: usize = 32 * 1024;
pub const MAX_OUTPUT: usize = 32 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    InvalidState,
    Unsupported,
    Limit,
    Syntax,
    UnknownShare,
    Authentication,
    AccountDisabled,
    UnixIdentity,
    SharePolicy,
    DirectorySearch,
    MissingFile,
    FilePermission,
    ReadOnly,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    pub path: String,
    pub bytes: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadFile {
    pub path: String,
    pub text: String,
}
fn fault(kind: FaultKind, detail: impl Into<String>) -> Fault {
    Fault {
        kind,
        detail: detail.into(),
    }
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub user: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub schema_version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub server: String,
    pub files: Tree,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Account,16>")]
    pub accounts: Vec<Account>,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub guest: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub configuration_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub published: String,
    pub generation: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Credential {
    Valid {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        user: String,
    },
    Invalid {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        user: String,
    },
    Anonymous,
}
/// A verified simulated connection retains share policy, not an open OS handle.
/// Construct only through World::connect; do not deserialize privileged handles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    server: String,
    user: String,
    groups: Vec<String>,
    share: String,
    policy: config::Share,
    generation: u32,
}
impl Connection {
    pub fn user(&self) -> &str {
        &self.user
    }
    pub fn share(&self) -> &str {
        &self.share
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }
}
fn listed(files: &Tree, user: &str, list: &[config::Principal]) -> bool {
    list.iter().any(|p| match p {
        config::Principal::User(n) => n == user,
        config::Principal::Group(g) => files
            .users
            .get(user)
            .is_some_and(|groups| groups.contains(g)),
    })
}
impl World {
    pub fn validate(&self) -> Result<(), Fault> {
        if self.schema_version != 1
            || self.server.is_empty()
            || self.server.len() > 48
            || !self
                .server
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !(1..=1_000_000).contains(&self.generation)
            || !self.files.valid()
            || self.files.files.values().map(String::len).sum::<usize>() > MAX_FILES_BYTES
            || !name(&self.guest)
            || self.guest == "root"
            || !self.files.users.contains_key(&self.guest)
            || self.accounts.len() > 16
            || self.configuration_path.len() > 512
            || !self
                .files
                .nodes
                .get(&self.configuration_path)
                .is_some_and(|n| !n.directory)
            || self.saved_config().is_err()
            || self
                .saved_config()
                .is_ok_and(|s| s.len() > config::MAX_CONFIG)
            || self.published.len() > config::MAX_CONFIG
        {
            return Err(fault(
                FaultKind::InvalidState,
                "Use a bounded version1 standalone server with supplied virtual files, guest identity and generation.",
            ));
        }
        let mut users = BTreeSet::new();
        if self
            .accounts
            .iter()
            .any(|a| !name(&a.user) || a.user == "root" || !users.insert(&a.user))
        {
            return Err(fault(
                FaultKind::InvalidState,
                "SMB accounts must name distinct non-root canonical users.",
            ));
        }
        config::parse(&self.published).map_err(|e| {
            fault(
                FaultKind::InvalidState,
                format!("Applied configuration is not supported: {}", e.detail),
            )
        })?;
        Ok(())
    }
    /// Saving malformed but bounded text is allowed, so a learner can diagnose it.
    /// Publishing validates first and leaves the previous applied text on failure.
    pub fn saved_config(&self) -> Result<&str, Fault> {
        self.files
            .files
            .get(&self.configuration_path)
            .map(String::as_str)
            .ok_or_else(|| {
                fault(
                    FaultKind::InvalidState,
                    "The simulated configuration file is missing.",
                )
            })
    }
    pub fn save(&mut self, text: String) -> Result<(), Fault> {
        self.validate()?;
        if !self.files.access(
            &self.files.user,
            &self.configuration_path,
            Access::WriteFile,
        ) {
            return Err(fault(
                FaultKind::FilePermission,
                "The current virtual Unix user cannot write the configuration file. Inspect its mode and parent search permissions.",
            ));
        }
        let path = self.configuration_path.clone();
        self.overwrite(&path, &text)
    }
    pub fn check_saved(&self) -> Result<config::Configuration, Fault> {
        self.validate()?;
        if !self
            .files
            .access(&self.files.user, &self.configuration_path, Access::ReadFile)
        {
            return Err(fault(
                FaultKind::FilePermission,
                "The current virtual Unix user cannot read the saved configuration file.",
            ));
        }
        config::parse(self.saved_config()?).map_err(|e| {
            fault(
                match e.kind {
                    config::Kind::Syntax => FaultKind::Syntax,
                    config::Kind::Unsupported => FaultKind::Unsupported,
                    config::Kind::Limit => FaultKind::Limit,
                },
                format!(
                    "{:?} at line {}: {} Applied settings were retained.",
                    e.kind, e.line, e.detail
                ),
            )
        })
    }
    pub fn publish(&mut self) -> Result<(), Fault> {
        self.check_saved()?;
        let text = self.saved_config()?.to_owned();
        if text != self.published {
            if self.generation == 1_000_000 {
                return Err(fault(
                    FaultKind::Limit,
                    "Configuration generation limit reached; applied settings were retained.",
                ));
            }
            self.published = text;
            self.generation += 1;
        }
        Ok(())
    }
    /// Provision/enable only a supplied Unix identity, with a fictional test proof.
    /// No password, passdb file or actual system account is read or written.
    pub fn set_account(&mut self, user: &str, enabled: bool) -> Result<(), Fault> {
        self.validate()?;
        if !name(user) || user == "root" || !self.files.users.contains_key(user) {
            return Err(fault(
                FaultKind::UnixIdentity,
                "Supply an existing non-root Unix identity before configuring its simulated SMB account.",
            ));
        }
        if let Some(a) = self.accounts.iter_mut().find(|a| a.user == user) {
            a.enabled = enabled;
            return Ok(());
        }
        if self.accounts.len() == 16 {
            return Err(fault(
                FaultKind::Limit,
                "At most16SMB accounts are supported.",
            ));
        }
        self.accounts.push(Account {
            user: user.into(),
            enabled,
        });
        self.accounts.sort_by(|a, b| a.user.cmp(&b.user));
        Ok(())
    }
    pub fn connect(&self, share: &str, credential: &Credential) -> Result<Connection, Fault> {
        self.validate()?;
        if !name(share) {
            return Err(fault(
                FaultKind::Unsupported,
                "Choose a supported simple share name.",
            ));
        }
        let (user, anonymous) = match credential {
            Credential::Invalid { user } => {
                if !name(user) {
                    return Err(fault(
                        FaultKind::Unsupported,
                        "Use a supplied canonical client identity.",
                    ));
                }
                return Err(fault(
                    FaultKind::Authentication,
                    "The supplied test credential is invalid. With map to guest=Never, a failed named login is not an anonymous login.",
                ));
            }
            Credential::Valid { user } => {
                if !name(user) || user == "root" {
                    return Err(fault(
                        FaultKind::Unsupported,
                        "Root and noncanonical client identities are outside this profile.",
                    ));
                }
                let account = self
                    .accounts
                    .iter()
                    .find(|a| a.user == *user)
                    .ok_or_else(|| {
                        fault(
                            FaultKind::Authentication,
                            "There is no simulated SMB account for this Unix user.",
                        )
                    })?;
                if !account.enabled {
                    return Err(fault(
                        FaultKind::AccountDisabled,
                        "This simulated SMB account is disabled; Unix file permission alone cannot authenticate it.",
                    ));
                }
                if !self.files.users.contains_key(user) {
                    return Err(fault(
                        FaultKind::UnixIdentity,
                        "The SMB account has no supplied Unix identity for file access.",
                    ));
                }
                (user.clone(), false)
            }
            Credential::Anonymous => (self.guest.clone(), true),
        };
        let cfg =
            config::parse(&self.published).map_err(|e| fault(FaultKind::InvalidState, e.detail))?;
        let id = share.to_ascii_lowercase();
        let policy = cfg.shares.get(&id).ok_or_else(|| {
            fault(
                FaultKind::UnknownShare,
                "The applied configuration does not publish that share.",
            )
        })?;
        if anonymous && !policy.guest_ok {
            return Err(fault(
                FaultKind::Authentication,
                "This share does not permit an anonymous connection.",
            ));
        }
        if listed(&self.files, &user, &policy.invalid_users)
            || (!policy.valid_users.is_empty() && !listed(&self.files, &user, &policy.valid_users))
        {
            return Err(fault(
                FaultKind::SharePolicy,
                "The share's user policy does not admit this identity. Invalid users takes precedence over valid users.",
            ));
        }
        if !self
            .files
            .nodes
            .get(&policy.path)
            .is_some_and(|n| n.directory)
        {
            return Err(fault(
                FaultKind::MissingFile,
                "The share path is not a supplied virtual directory. A syntax check does not prove availability.",
            ));
        }
        if !self
            .files
            .access(&user, &policy.path, Access::SearchDirectory)
        {
            return Err(fault(
                FaultKind::DirectorySearch,
                "The mapped Unix user cannot traverse the share path. Share flags do not grant Unix directory search permission.",
            ));
        }
        Ok(Connection {
            server: self.server.clone(),
            groups: self.files.users[&user].clone(),
            user,
            share: id,
            policy: policy.clone(),
            generation: self.generation,
        })
    }
    fn file_path(&self, c: &Connection, relative: &str) -> Result<String, Fault> {
        self.validate()?;
        if c.server != self.server {
            return Err(fault(
                FaultKind::InvalidState,
                "This simulated connection belongs to another supplied server.",
            ));
        }
        if !self.files.users.contains_key(&c.user) {
            return Err(fault(
                FaultKind::UnixIdentity,
                "The connection's Unix identity is no longer supplied.",
            ));
        }
        if self.files.users.get(&c.user) != Some(&c.groups) {
            return Err(fault(
                FaultKind::Unsupported,
                "The supplied Unix group map changed. Open a new simulated connection; existing security-token changes are outside this profile.",
            ));
        }
        if relative.is_empty()
            || relative.len() > 512
            || relative.starts_with('/')
            || relative.contains('\\')
            || relative.chars().any(char::is_control)
            || relative
                .split('/')
                .any(|p| p.is_empty() || matches!(p, "." | ".."))
        {
            return Err(fault(
                FaultKind::Unsupported,
                "Use a canonical share-relative virtual file path; absolute, dot, backslash and control-character paths are not supported.",
            ));
        }
        let path = format!("{}/{}", c.policy.path.trim_end_matches('/'), relative);
        if path.len() > 1024 {
            return Err(fault(FaultKind::Limit, "Virtual path exceeds1024bytes."));
        }
        if !self.files.nodes.get(&path).is_some_and(|n| !n.directory) {
            return Err(fault(
                FaultKind::MissingFile,
                "Choose an existing supplied regular file; creation and other file types are not simulated here.",
            ));
        }
        Ok(path)
    }
    pub fn read(&self, c: &Connection, relative: &str) -> Result<String, Fault> {
        self.read_record(c, relative).map(|r| r.text)
    }
    pub fn read_record(&self, c: &Connection, relative: &str) -> Result<ReadFile, Fault> {
        let path = self.file_path(c, relative)?;
        if !self.files.access(&c.user, &path, Access::ReadFile) {
            return Err(fault(
                FaultKind::FilePermission,
                "The mapped Unix user cannot read this file; inspect its owner/group/mode and parent search permissions.",
            ));
        }
        let text = &self.files.files[&path];
        if text.len() > MAX_OUTPUT {
            return Err(fault(
                FaultKind::Limit,
                "File exceeds the32KiB preview limit; no partial output is reported as complete.",
            ));
        }
        Ok(ReadFile {
            path,
            text: text.clone(),
        })
    }
    pub fn replace(
        &mut self,
        c: &Connection,
        relative: &str,
        text: &str,
    ) -> Result<Written, Fault> {
        let path = self.file_path(c, relative)?;
        let override_write = listed(&self.files, &c.user, &c.policy.write_list);
        if !override_write
            && (c.policy.read_only || listed(&self.files, &c.user, &c.policy.read_list))
        {
            return Err(fault(
                FaultKind::ReadOnly,
                "This connection's share policy is read-only for this user. A write-list match can override read-only/read-list policy, but never Unix permissions.",
            ));
        }
        if !self.files.access(&c.user, &path, Access::WriteFile) {
            return Err(fault(
                FaultKind::FilePermission,
                "Share policy permits writing, but the mapped Unix user cannot write the file or traverse its parents.",
            ));
        }
        self.overwrite(&path, text)?;
        Ok(Written {
            path,
            bytes: text.len(),
        })
    }
    fn overwrite(&mut self, path: &str, text: &str) -> Result<(), Fault> {
        if text.contains('\0') {
            return Err(fault(
                FaultKind::Unsupported,
                "These simulated files contain text; NUL bytes are not supported. The file was retained.",
            ));
        }
        let old = self.files.files[path].len();
        let total = self.files.files.values().map(String::len).sum::<usize>();
        if text.len() > MAX_FILES_BYTES
            || (path == self.configuration_path && text.len() > config::MAX_CONFIG)
            || total - old + text.len() > MAX_FILES_BYTES
            || self.files.clock == 1_000_000
        {
            return Err(fault(
                FaultKind::Limit,
                "Replacement exceeds the virtual content/clock limit; the file was retained.",
            ));
        }
        let mut next = self.clone();
        next.files.files.insert(path.into(), text.into());
        next.files.clock += 1;
        next.files.modified.insert(path.into(), next.files.clock);
        next.validate()?;
        *self = next;
        Ok(())
    }
}
