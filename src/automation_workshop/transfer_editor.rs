//! Separate editor fragments from the applied virtual transfer policy and replica.
use super::authenticated_transfer::{
    Client, KeyReference, Policy, Settings, Task, TransferReply, Work,
};
use crate::dns_access::config::parse_transfer_fragment;
use serde::{Deserialize, Serialize};

pub const CONFIG_LIMIT: usize = 2048;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub served_fragment: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub protected_fragment: String,
    pub selected_key: Option<KeyReference>,
    pub actual: Work,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "last_observation"
    )]
    pub observation: Option<String>,
}

fn last_observation<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Message(#[serde(deserialize_with = "crate::cron::text::<_,512>")] String);
    Ok(Option::<Message>::deserialize(d)?.map(|line| line.0))
}

fn text_allowed(input: &str) -> bool {
    input.len() <= CONFIG_LIMIT && !input.contains('\0')
}

impl Edit {
    pub fn new(task: &Task) -> Self {
        Self {
            served_fragment: format!("allow-transfer {{ {}; }};", task.secondary_address),
            protected_fragment: "allow-transfer { none; };".into(),
            selected_key: task.initial.secondary.clone(),
            actual: task.start(),
            observation: None,
        }
    }

    pub fn change(&mut self, protected: bool, text: String) -> Result<(), String> {
        if !text_allowed(&text) {
            return Err("Keep this configuration fragment within 2 KiB of text.".into());
        }
        if protected {
            self.protected_fragment = text;
        } else {
            self.served_fragment = text;
        }
        self.observation = None;
        Ok(())
    }

    fn parsed(&self, task: &Task) -> Result<Settings, String> {
        if !text_allowed(&self.served_fragment) || !text_allowed(&self.protected_fragment) {
            return Err("Keep both configuration fragments within 2 KiB of text.".into());
        }
        Ok(Settings {
            primary: parse_transfer_fragment(&self.served_fragment, task)
                .map_err(|e| format!("Served zone. {e}"))?,
            secondary: self.selected_key.clone(),
            unrelated: parse_transfer_fragment(&self.protected_fragment, task)
                .map_err(|e| format!("Protected zone. {e}"))?,
        })
    }

    pub fn validate(&self, task: &Task) -> Result<(), String> {
        if !text_allowed(&self.served_fragment)
            || !text_allowed(&self.protected_fragment)
            || self
                .selected_key
                .as_ref()
                .is_some_and(|key| key != &task.stale && !task.approved.contains(key))
            || self.actual.draft != self.actual.applied
            || self
                .observation
                .as_ref()
                .is_some_and(|line| line.len() > 512 || line.contains('\0'))
        {
            return Err("Use short fragments and one supplied secondary key. Keep the previous applied policy when a draft is rejected.".into());
        }
        task.check_work(&self.actual)
    }

    pub fn apply(&mut self, task: &Task) -> Result<(), String> {
        self.validate(task)?;
        let mut next = self.actual.clone();
        next.draft = self.parsed(task)?;
        task.apply(&mut next)?;
        self.actual = next;
        self.observation = None;
        Ok(())
    }

    pub fn trial(&mut self, task: &Task, client: Client, protected: bool) -> Result<(), String> {
        let context = if protected {
            "Protected zone"
        } else {
            match client {
                Client::ConfiguredSecondary => "Approved secondary",
                Client::UnsignedAtSecondaryAddress => "Unsigned request",
                Client::OldProofAtSecondaryAddress => "Outdated key proof",
                Client::UnrelatedClient => "Unrelated client",
            }
        };
        let result = match task.transfer(&mut self.actual, client, protected)? {
            TransferReply::Refused => {
                "The transfer rule refused the request. No data was copied.".into()
            }
            TransferReply::AuthenticationRejected => {
                "The supplied proof does not match a current key. No data was copied.".into()
            }
            TransferReply::Secondary(decision) => {
                let action = match decision {
                    crate::dns::Transfer::Updated => {
                        "The approved secondary received a fresh copy."
                    }
                    crate::dns::Transfer::UnchangedEqual => {
                        "The secondary already has this serial."
                    }
                    crate::dns::Transfer::UnchangedOlder => {
                        "The secondary retained its newer copy."
                    }
                };
                format!(
                    "{action} Secondary serial {}.",
                    self.actual.copies.secondary.serial
                )
            }
            TransferReply::UnauthorizedRead(zone) => format!(
                "The policy exposed {} records. The approved secondary was unchanged.",
                zone.records.len()
            ),
        };
        self.observation = Some(format!("{context}. {result}"));
        Ok(())
    }

    pub fn lookup(&mut self, task: &Task, secondary: bool) -> Result<(), String> {
        let answer = if secondary {
            self.actual
                .copies
                .secondary
                .query(&task.lookup, crate::dns::QueryType::A)
                .map_err(|error| error.message)?
        } else {
            task.public_lookup(&self.actual)?
        };
        let address = answer.records.iter().find_map(|row| match row.data {
            crate::dns::Data::A { address } => Some(address),
            _ => None,
        });
        let note = address.map_or_else(
            || "No A record returned.".into(),
            |ip| format!("{} returned {ip}.", task.lookup),
        );
        let label = if secondary {
            "Secondary lookup"
        } else {
            "Public lookup"
        };
        self.observation = Some(format!("{label}. {note}"));
        Ok(())
    }

    pub fn outcomes(&self, task: &Task) -> Result<[(bool, String); 6], String> {
        self.validate(task)?;
        let checked = task.checks(&self.actual)?;
        let preview = self.parsed(task);
        let applied = preview
            .as_ref()
            .is_ok_and(|parsed| parsed == &self.actual.applied);
        let pending = match preview {
            Ok(_) if applied => "Current configuration is applied.".into(),
            Ok(_) => "Apply this draft before checking its behavior.".into(),
            Err(error) => error,
        };
        let applied_key = checked[1];
        let allowed = match &self.actual.applied.primary {
            Policy::VerifiedKey { reference } if applied_key => {
                format!(
                    "The served zone requires a verified {} key.",
                    reference.name
                )
            }
            _ => "Require an approved key for the served zone and apply the new transfer rule."
                .into(),
        };
        let detail = |passed, when_passed: &str, retry: &str| {
            (passed, if passed { when_passed } else { retry }.into())
        };
        Ok([
            (applied, pending),
            (applied_key, allowed),
            detail(
                checked[2],
                "The secondary has the published serial and address record.",
                "Refresh with the matching current secondary reference, then inspect the serial and address record.",
            ),
            detail(
                checked[3],
                "Unauthorized requests receive no copy.",
                "The current rule still discloses data in an unauthorized trial. Check what it requires before allowing a copy.",
            ),
            detail(
                checked[4],
                "The unrelated zone refuses a transfer even with a current key.",
                "Keep transfers denied for the unrelated zone.",
            ),
            detail(
                checked[5],
                "An ordinary authoritative lookup returns the published address record.",
                "Verify that an ordinary authoritative lookup still returns the published record.",
            ),
        ])
    }
}
