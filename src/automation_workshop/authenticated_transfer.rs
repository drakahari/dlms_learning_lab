//! Zone transfer access decisions using opaque supplied proof identities, without cryptography.
use crate::dns::{self, Answer, QueryType, SerialOrder, Server, Transfer, Zone};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyVersion {
    ApprovedActive,
    ApprovedStandby,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyReference {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub name: String,
    pub version: KeyVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Policy {
    Deny,
    SourceAddress,
    VerifiedKey { reference: KeyReference },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub primary: Policy,
    pub secondary: Option<KeyReference>,
    pub unrelated: Policy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub served: Server,
    pub protected: Zone,
    pub initial: Settings,
    pub approved: [KeyReference; 2],
    pub stale: KeyReference,
    pub secondary_address: Ipv4Addr,
    pub other_address: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub lookup: String,
    pub expected_record: Ipv4Addr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub draft: Settings,
    pub applied: Settings,
    pub copies: Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Client {
    ConfiguredSecondary,
    UnsignedAtSecondaryAddress,
    OldProofAtSecondaryAddress,
    UnrelatedClient,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferReply {
    Refused,
    AuthenticationRejected,
    Secondary(Transfer),
    UnauthorizedRead(Zone),
}

impl Task {
    pub fn start(&self) -> Work {
        Work {
            draft: self.initial.clone(),
            applied: self.initial.clone(),
            copies: self.served.clone(),
        }
    }

    fn known(&self, reference: &KeyReference) -> bool {
        self.approved.contains(reference) || reference == &self.stale
    }

    fn check_settings(&self, settings: &Settings) -> Result<(), String> {
        let known_policy = |policy: &Policy| match policy {
            Policy::VerifiedKey { reference } => self.approved.contains(reference),
            Policy::Deny | Policy::SourceAddress => true,
        };
        if !known_policy(&settings.primary)
            || !known_policy(&settings.unrelated)
            || settings
                .secondary
                .as_ref()
                .is_some_and(|key| !self.known(key))
        {
            return Err(
                "Select a supplied virtual key reference; no key data can be loaded.".into(),
            );
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        self.served.validate().map_err(|error| error.message)?;
        self.protected
            .check_configuration()
            .map_err(|error| error.message)?;
        if self.served.primary.origin == self.protected.origin
            || self.approved[0].version != KeyVersion::ApprovedActive
            || self.approved[1].version != KeyVersion::ApprovedStandby
            || self.stale.version != KeyVersion::Superseded
            || self.stale.name != self.approved[0].name
            || self.approved[0].name == self.approved[1].name
            || self.secondary_address == self.other_address
            || [self.secondary_address, self.other_address]
                .into_iter()
                .any(|ip| ip.is_unspecified() || ip.is_broadcast() || ip.is_multicast())
            || dns::serial_order(self.served.primary.serial, self.served.secondary.serial)
                != SerialOrder::Newer
            || self.initial.primary != Policy::SourceAddress
            || self.initial.unrelated != Policy::Deny
            || !self.lookup_matches(&self.served.primary)?
            || self.lookup_matches(&self.served.secondary)?
        {
            return Err("Supply two distinct zones, an outdated secondary and fixed matching or superseded proof references.".into());
        }
        for selected in self.approved.iter().chain(std::iter::once(&self.stale)) {
            let name = dns::canonical_name(&selected.name).map_err(|error| error.message)?;
            if name != selected.name {
                return Err("Use canonical names for the supplied key references.".into());
            }
        }
        self.check_settings(&self.initial)
    }

    fn lookup_matches(&self, zone: &Zone) -> Result<bool, String> {
        let answer = zone
            .query(&self.lookup, QueryType::A)
            .map_err(|error| error.message)?;
        Ok(answer.authoritative
            && answer.records.iter().any(
                |row| matches!(row.data,dns::Data::A{address} if address==self.expected_record),
            ))
    }

    pub fn check_work(&self, work: &Work) -> Result<(), String> {
        self.check_settings(&work.draft)?;
        self.check_settings(&work.applied)?;
        work.copies.validate().map_err(|error| error.message)?;
        if work.copies.primary != self.served.primary
            || work.copies.configured != self.served.configured
            || ![&self.served.secondary, &self.served.primary].contains(&&work.copies.secondary)
        {
            return Err("This state does not belong to the supplied primary and its two possible secondary snapshots.".into());
        }
        Ok(())
    }

    pub fn apply(&self, work: &mut Work) -> Result<(), String> {
        self.check_work(work)?;
        work.applied = work.draft.clone();
        Ok(())
    }

    pub fn transfer(
        &self,
        work: &mut Work,
        client: Client,
        protected: bool,
    ) -> Result<TransferReply, String> {
        self.check_work(work)?;
        let policy = if protected {
            &work.applied.unrelated
        } else {
            &work.applied.primary
        };
        let address = if client == Client::UnrelatedClient {
            self.other_address
        } else {
            self.secondary_address
        };
        let proof = match client {
            Client::ConfiguredSecondary => work.applied.secondary.as_ref(),
            Client::OldProofAtSecondaryAddress => Some(&self.stale),
            _ => None,
        };
        // A supplied invalid signed request fails before it can use an address rule.
        if proof.is_some_and(|reference| !self.approved.contains(reference)) {
            return Ok(TransferReply::AuthenticationRejected);
        }
        let permitted = match policy {
            Policy::Deny => false,
            Policy::SourceAddress => address == self.secondary_address,
            Policy::VerifiedKey { reference } => proof == Some(reference),
        };
        if !permitted {
            return Ok(TransferReply::Refused);
        }
        if protected || client != Client::ConfiguredSecondary {
            return Ok(TransferReply::UnauthorizedRead(if protected {
                self.protected.clone()
            } else {
                work.copies.primary.clone()
            }));
        }
        work.copies
            .update_secondary()
            .map(TransferReply::Secondary)
            .map_err(|error| error.message)
    }

    pub fn public_lookup(&self, work: &Work) -> Result<Answer, String> {
        self.check_work(work)?;
        work.copies
            .primary
            .query(&self.lookup, QueryType::A)
            .map_err(|error| error.message)
    }

    pub fn checks(&self, work: &Work) -> Result<[bool; 6], String> {
        self.check_work(work)?;
        let approved = matches!(&work.applied.primary,Policy::VerifiedKey{reference} if self.approved.contains(reference));
        let refreshed = self.lookup_matches(&work.copies.secondary)?;
        let mut test = work.clone();
        test.copies.secondary = self.served.secondary.clone();
        let got_actual_update = matches!(
            self.transfer(&mut test, Client::ConfiguredSecondary, false)?,
            TransferReply::Secondary(Transfer::Updated)
        ) && test.copies.secondary == test.copies.primary;
        let mut no_leak = true;
        for client in [
            Client::UnsignedAtSecondaryAddress,
            Client::OldProofAtSecondaryAddress,
            Client::UnrelatedClient,
        ] {
            no_leak &= matches!(
                self.transfer(&mut test, client, false)?,
                TransferReply::Refused | TransferReply::AuthenticationRejected
            );
        }
        // Test with a current proof so an earlier authentication rejection cannot
        // be mistaken for enforcement of the protected zone's own rule.
        test.applied.secondary = Some(self.approved[0].clone());
        let other_zone = work.applied.unrelated == Policy::Deny
            && self.transfer(&mut test, Client::ConfiguredSecondary, true)?
                == TransferReply::Refused;
        Ok([
            work.draft == work.applied,
            approved,
            refreshed && got_actual_update,
            no_leak,
            other_zone,
            self.lookup_matches(&work.copies.primary)?,
        ])
    }
}
