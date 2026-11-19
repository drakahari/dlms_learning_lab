//! Optional utility work uses the existing bounded draft/session envelope.
//! The saved draft holds a fictional profile key, command and LDIF, not a password.
use super::*;
use crate::{ldap_identity as identity, ldap_utility as utility};
use std::fmt;

const HEADER: &str = "sample-credential: ";
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credential {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub principal: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub sample_password: String,
}
impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credential")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("principal", &self.principal)
            .field("sample_password", &"[redacted]")
            .finish()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    #[default]
    Ldif,
    DnList,
}
impl InputKind {
    fn is_ldif(&self) -> bool {
        *self == Self::Ldif
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Utility {
    #[serde(default, skip_serializing_if = "InputKind::is_ldif")]
    pub input_kind: InputKind,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub uri: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub input_file: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Credential,4>")]
    pub credentials: Vec<Credential>,
}
/// Internal work envelope. The UI presents its three parts separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub credential: String,
    pub command: String,
    pub ldif: String,
}
impl Draft {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if !text(raw) {
            return Err("Directory work exceeds its 16 KiB text bounds.".into());
        }
        let mut parts = raw.splitn(4, '\n');
        let credential = parts
            .next()
            .and_then(|s| s.strip_prefix(HEADER))
            .filter(|s| key(s))
            .ok_or("Saved utility work needs its supplied sample-credential key.")?;
        let command = parts
            .next()
            .ok_or("Saved utility work needs a command field.")?;
        if command.len() > client::MAX_COMMAND || parts.next() != Some("") {
            return Err(
                "Saved utility work needs one bounded command line and a separate LDIF editor."
                    .into(),
            );
        }
        let ldif = parts
            .next()
            .ok_or("Saved utility work needs an LDIF editor field.")?;
        Ok(Self {
            credential: credential.into(),
            command: command.into(),
            ldif: ldif.into(),
        })
    }
    pub fn encode(&self) -> Result<String, String> {
        if !key(&self.credential)
            || self.command.contains(['\n', '\r'])
            || self.command.len() > client::MAX_COMMAND
            || !text(&self.ldif)
        {
            return Err(
                "Keep the supplied profile, one command line and bounded LDIF text.".into(),
            );
        }
        // Bound before allocating the combined work envelope.
        if HEADER.len() + self.credential.len() + self.command.len() + self.ldif.len() + 3
            > MAX_DRAFT
        {
            return Err("Combined utility work exceeds16KiB; prior work is retained.".into());
        }
        let raw = format!(
            "{HEADER}{}\n{}\n\n{}",
            self.credential, self.command, self.ldif
        );
        Self::parse(&raw)?;
        Ok(raw)
    }
}
impl Utility {
    pub fn is_dn_list(&self) -> bool {
        self.input_kind == InputKind::DnList
    }
    pub(super) fn bound_draft(&self, raw: &str) -> Result<(), String> {
        let draft = Draft::parse(raw)?;
        if self.credentials.iter().any(|c| c.id == draft.credential) {
            Ok(())
        } else {
            Err("Saved work selects a fictional profile that this lesson does not supply.".into())
        }
    }
    pub(super) fn realm(&self, task: &Task, source: &q::Source) -> Result<identity::Realm, String> {
        let primary = task.grant.model()?;
        let mut identities = Vec::new();
        for c in &self.credentials {
            let dn = q::Dn::parse(&c.principal).map_err(|e| e.to_string())?;
            let full = dn.equivalent(primary.principal());
            let attrs = if full {
                task.grant.attributes.iter().map(|a| a.0.as_str()).collect()
            } else {
                Vec::new()
            };
            identities.push(
                identity::Identity::new(
                    dn,
                    &c.sample_password,
                    primary.subtree().clone(),
                    full && task.grant.add,
                    full && task.grant.delete,
                    &attrs,
                )
                .map_err(|e| e.to_string())?,
            );
        }
        identity::Realm::new(&self.uri, source, &identities).map_err(|e| e.to_string())
    }
    pub(super) fn validate(&self, task: &Task) -> Result<(), String> {
        if self.credentials.is_empty()
            || self.credentials.len() > 4
            || self.uri.len() > 256
            || self.input_file.len() > 128
        {
            return Err(
                "Utility work needs bounded virtual inputs and1–4fictional profiles.".into(),
            );
        }
        // The same virtual namespace checks as command execution; no file is opened.
        utility::Inputs::new("", &[(&self.input_file, "")]).map_err(|e| e.to_string())?;
        let primary = task.grant.model()?;
        let mut ids = BTreeSet::new();
        let mut labels = BTreeSet::new();
        let mut has_primary = false;
        for c in &self.credentials {
            if !key(&c.id)
                || !ids.insert(&c.id)
                || !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || c.principal.len() > 1024
                || c.sample_password.len() > 128
            {
                return Err(
                    "Use distinct bounded fictional profile keys/labels and supported identities."
                        .into(),
                );
            }
            has_primary |= q::Dn::parse(&c.principal)
                .map_err(|e| e.to_string())?
                .equivalent(primary.principal());
        }
        if !has_primary {
            return Err("Provide credentials for the delegated identity.".into());
        }
        for raw in [&task.starter, &task.reference] {
            self.bound_draft(raw)?;
        }
        for case in &task.cases {
            self.realm(task, &case.initial)?;
        }
        Ok(())
    }
    pub(super) fn run(&self, task: &Task, index: usize, raw: &str) -> Result<Run, String> {
        let draft = Draft::parse(raw)?;
        let credential = self
            .credentials
            .iter()
            .find(|c| c.id == draft.credential)
            .ok_or("Select a supplied fictional credential profile.")?;
        let mut realm = self.realm(task, &task.cases[index].initial)?;
        let inputs = utility::Inputs::new(&draft.ldif, &[(&self.input_file, &draft.ldif)])
            .map_err(|e| e.to_string())?;
        let run = if self.is_dn_list() {
            utility::run_delete
        } else {
            utility::run
        };
        let report = run(
            &draft.command,
            &credential.sample_password,
            &inputs,
            &mut realm,
        );
        let mut feedback = format!("Exit status: {}\n", report.exit_status());
        for (i, applied) in report.applied.iter().enumerate() {
            use fmt::Write;
            writeln!(
                feedback,
                "Record {} · {:?}: {}",
                i + 1,
                applied.operation,
                applied.dn
            )
            .map_err(|_| "Could not format request results.")?;
        }
        if let Some(failure) = &report.failure {
            feedback.push_str(&format!("{failure}\n"));
        }
        Ok(Run {
            applied: report.succeeded(),
            feedback,
            actual: output(realm.directory().source())?,
            directory: realm.directory().source().clone(),
        })
    }
}
