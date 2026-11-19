//! Bounded Apache HTTPS identity fixtures. No keys, TLS transport or filesystem access.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type Result<T> = std::result::Result<T, String>;
const MAX_CERTIFICATES: usize = 24;
const MAX_FILES: usize = 24;
const MAX_CHAIN: usize = 8;

fn certs<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Certificate>, D::Error> {
    crate::cron_workspace::sequence::<_, Certificate, MAX_CERTIFICATES>(d)
}

fn bundles<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Bundle>, D::Error> {
    crate::cron_workspace::sequence::<_, Bundle, MAX_FILES>(d)
}

fn keys<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<KeyFile>, D::Error> {
    crate::cron_workspace::sequence::<_, KeyFile, MAX_FILES>(d)
}

fn names<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Name>, D::Error> {
    crate::cron_workspace::sequence::<_, Name, MAX_CHAIN>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,96>")] pub String);

impl Name {
    fn identifier(&self) -> bool {
        !self.0.is_empty()
            && self.0.len() <= 96
            && self
                .0
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
    }

    fn dns(&self) -> bool {
        let bytes = self.0.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= 96
            && self.0.contains('.')
            && self.0.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
            && self
                .0
                .rsplit('.')
                .next()
                .is_some_and(|label| label.bytes().any(|b| b.is_ascii_alphabetic()))
    }
}

fn path(value: &str) -> bool {
    let relative = value
        .strip_prefix("/etc/ssl/")
        .or_else(|| value.strip_prefix("/etc/pki/"));
    value.len() <= 128
        && relative.is_some_and(|tail| {
            tail.split('/').all(|part| {
                !part.is_empty()
                    && !matches!(part, "." | "..")
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
            })
        })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    pub id: Name,
    pub public_key_id: Name,
    pub issuer: Name,
    pub is_ca: bool,
    pub first_day: u16,
    pub last_day: u16,
    #[serde(deserialize_with = "names")]
    pub dns_names: Vec<Name>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "names")]
    pub certificate_ids: Vec<Name>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyFile {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub matching_public_key_id: Name,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "certs")]
    pub certificates: Vec<Certificate>,
    #[serde(deserialize_with = "bundles")]
    pub bundles: Vec<Bundle>,
    #[serde(deserialize_with = "keys")]
    pub keys: Vec<KeyFile>,
}

impl Inventory {
    fn certificate(&self, id: &Name) -> Option<&Certificate> {
        self.certificates.iter().find(|c| &c.id == id)
    }

    pub fn validate(&self) -> Result<()> {
        let mut cert_ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        if self.certificates.is_empty()
            || self.certificates.len() > MAX_CERTIFICATES
            || self.bundles.is_empty()
            || self.bundles.len() > MAX_FILES
            || self.keys.is_empty()
            || self.keys.len() > MAX_FILES
        {
            return Err(
                "Supply a bounded inventory of virtual certificates, bundle paths and key paths."
                    .into(),
            );
        }
        for c in &self.certificates {
            if !c.id.identifier()
                || !c.public_key_id.identifier()
                || !cert_ids.insert(&c.id)
                || self.certificate(&c.issuer).is_none()
                || c.first_day > c.last_day
                || c.is_ca && !c.dns_names.is_empty()
                || !c.is_ca && c.dns_names.is_empty()
                || c.dns_names.len() > MAX_CHAIN
                || c.dns_names.iter().any(|name| !name.dns())
            {
                return Err("Use unique certificate IDs, literal leaf DNS names, registered issuers and valid model dates.".into());
            }
        }
        for bundle in &self.bundles {
            let mut provided = BTreeSet::new();
            if !path(&bundle.path)
                || !paths.insert(&bundle.path)
                || bundle.certificate_ids.is_empty()
                || bundle.certificate_ids.len() > MAX_CHAIN
                || bundle
                    .certificate_ids
                    .iter()
                    .any(|id| !provided.insert(id) || !cert_ids.contains(id))
            {
                return Err("Use distinct certificate paths and up to eight unique supplied IDs per bundle.".into());
            }
        }
        for key in &self.keys {
            if !path(&key.path)
                || !paths.insert(&key.path)
                || !key.matching_public_key_id.identifier()
            {
                return Err(
                    "Use distinct literal key paths with supplied public-key identities.".into(),
                );
            }
        }
        Ok(())
    }

    pub fn binding(
        &self,
        certificate_path: &str,
        key_path: &str,
    ) -> Result<(&Bundle, &Certificate)> {
        let bundle = self
            .bundles
            .iter()
            .find(|b| b.path == certificate_path)
            .ok_or("The supplied certificate file is absent.")?;
        let key = self
            .keys
            .iter()
            .find(|k| k.path == key_path)
            .ok_or("The supplied key file is absent.")?;
        let leaf = bundle
            .certificate_ids
            .first()
            .and_then(|id| self.certificate(id))
            .ok_or("The certificate file contains no supplied leaf.")?;
        if leaf.is_ca {
            return Err("The first certificate in the supplied server bundle is an issuing CA. Use a bundle beginning with the server leaf.".into());
        }
        if key.matching_public_key_id != leaf.public_key_id {
            return Err(
                "The selected key does not match the first certificate's public key.".into(),
            );
        }
        Ok((bundle, leaf))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub hostname: Name,
    pub sni: Option<Name>,
    pub day: u16,
    #[serde(deserialize_with = "names")]
    pub trusted: Vec<Name>,
}

impl Client {
    pub fn validate(&self, inventory: &Inventory) -> Result<()> {
        if !self.hostname.dns()
            || self.sni.as_ref().is_some_and(|name| !name.dns())
            || self.trusted.len() > MAX_CHAIN
            || self
                .trusted
                .iter()
                .any(|id| !inventory.certificate(id).is_some_and(|c| c.is_ca))
            || self.trusted.iter().collect::<BTreeSet<_>>().len() != self.trusted.len()
        {
            return Err("Use literal DNS hostnames and distinct supplied trust anchors.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trust {
    Verified(Vec<Name>),
    OutsideValidity(Name),
    MissingIssuer { from: Name, issuer: Name },
    UnknownSelfSigned(Name),
    IssuerNotCa(Name),
    Cycle(Name),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verification {
    pub presented: Name,
    pub hostname_matches: bool,
    pub trust: Trust,
}

pub fn verify(
    inventory: &Inventory,
    bundle: &Bundle,
    leaf: &Certificate,
    client: &Client,
) -> Verification {
    let hostname_matches = leaf
        .dns_names
        .iter()
        .any(|name| name.0.eq_ignore_ascii_case(&client.hostname.0));
    let trust = verify_path(inventory, bundle, leaf, client);
    Verification {
        presented: leaf.id.clone(),
        hostname_matches,
        trust,
    }
}

fn verify_path(
    inventory: &Inventory,
    bundle: &Bundle,
    leaf: &Certificate,
    client: &Client,
) -> Trust {
    let mut node = leaf;
    let mut path = Vec::new();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(&node.id) {
            return Trust::Cycle(node.id.clone());
        }
        path.push(node.id.clone());
        if node.first_day > client.day || node.last_day < client.day {
            return Trust::OutsideValidity(node.id.clone());
        }
        if client.trusted.contains(&node.id) {
            return Trust::Verified(path);
        }
        if node.id == node.issuer {
            return Trust::UnknownSelfSigned(node.id.clone());
        }
        let available = client.trusted.contains(&node.issuer)
            || bundle
                .certificate_ids
                .iter()
                .skip(1)
                .any(|id| id == &node.issuer);
        let Some(issuer) = inventory.certificate(&node.issuer).filter(|_| available) else {
            return Trust::MissingIssuer {
                from: node.id.clone(),
                issuer: node.issuer.clone(),
            };
        };
        if !issuer.is_ca {
            return Trust::IssuerNotCa(issuer.id.clone());
        }
        node = issuer;
    }
}

pub mod config;

pub mod workspace;

pub mod session;

pub mod source;
