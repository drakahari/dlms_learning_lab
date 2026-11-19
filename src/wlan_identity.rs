//! Configuration compatibility for supplied WLAN/client profiles, not RF or crypto.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    Open,
    Wpa2Psk,
    Wpa3Sae,
}
impl Authentication {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open (no encryption)",
            Self::Wpa2Psk => "WPA2-Personal · PSK / AES-CCMP",
            Self::Wpa3Sae => "WPA3-Personal · SAE / AES-CCMP",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagementProtection {
    Disabled,
    Optional,
    Required,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub ssid: String,
    pub authentication: Authentication,
    pub management_protection: ManagementProtection,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub training_passphrase: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wlan {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub managed: bool,
    pub initial: Configuration,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub ssid: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Authentication,3>")]
    pub allowed_authentication: Vec<Authentication>,
    pub management_protection_capable: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,63>")]
    pub training_passphrase: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Wlan,4>")]
    pub wlans: Vec<Wlan>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Client,12>")]
    pub clients: Vec<Client>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub wlan: String,
    pub field: &'static str,
    pub feedback: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Compatibility {
    Deferred,
    NameNotFound,
    Disabled,
    AuthenticationMismatch,
    ManagementProtectionRequired,
    PassphraseMismatch,
    Compatible {
        wlan: String,
        authentication: Authentication,
        protected_management: bool,
    },
}
impl Compatibility {
    pub fn feedback(&self) -> &'static str {
        match self {
            Self::Deferred => "Fix the configuration fields before checking client compatibility.",
            Self::NameNotFound => {
                "No WLAN has this saved network name. Names are case-sensitive; spaces also matter."
            }
            Self::Disabled => "The saved name matches, but this WLAN is disabled.",
            Self::AuthenticationMismatch => {
                "This client's saved profile does not allow the WLAN's authentication method."
            }
            Self::ManagementProtectionRequired => {
                "This WLAN requires protected management frames, which this client does not support."
            }
            Self::PassphraseMismatch => {
                "The training passphrase differs between the WLAN and this saved client profile. It is case-sensitive; spaces matter."
            }
            Self::Compatible { .. } => {
                "The supplied configuration and client profile are compatible. No real radio connection was attempted."
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub issues: Vec<Issue>,
    pub clients: BTreeMap<String, Compatibility>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 120 && !s.chars().any(char::is_control)
}
fn printable_ascii(s: &str, minimum: usize, maximum: usize) -> bool {
    (minimum..=maximum).contains(&s.len()) && s.bytes().all(|b| (0x20..=0x7e).contains(&b))
}
impl Network {
    /// Initial configurations may contain intentional learner-repairable errors.
    /// Source shape and supplied client facts must still be valid and bounded.
    pub fn validate(&self) -> Result<()> {
        if !(1..=4).contains(&self.wlans.len()) || !(1..=12).contains(&self.clients.len()) {
            return Err("Supply 1–4 WLANs and 1–12 client profiles.".into());
        }
        let mut ids = BTreeSet::new();
        for w in &self.wlans {
            if !identifier(&w.id)
                || !label(&w.label)
                || !ids.insert(w.id.as_str())
                || w.initial.ssid.len() > 128
                || w.initial.training_passphrase.len() > 128
            {
                return Err("Invalid, duplicate or oversized supplied WLAN.".into());
            }
        }
        let mut clients = BTreeSet::new();
        for c in &self.clients {
            let methods: BTreeSet<_> = c.allowed_authentication.iter().copied().collect();
            if !identifier(&c.id)
                || !label(&c.label)
                || !clients.insert(c.id.as_str())
                || !printable_ascii(&c.ssid, 1, 32)
                || c.allowed_authentication.is_empty()
                || c.allowed_authentication.len() > 3
                || methods.len() != c.allowed_authentication.len()
                || (c.allowed_authentication.contains(&Authentication::Wpa3Sae)
                    && !c.management_protection_capable)
                || (c
                    .allowed_authentication
                    .iter()
                    .any(|a| *a != Authentication::Open)
                    && !printable_ascii(&c.training_passphrase, 8, 63))
                || c.training_passphrase.len() > 63
            {
                return Err("Invalid, duplicate or unsupported supplied client profile.".into());
            }
        }
        if !self.wlans.iter().any(|w| w.managed) {
            return Err("At least one WLAN must be managed in this activity.".into());
        }
        Ok(())
    }
    pub fn initial_configurations(&self) -> BTreeMap<String, Configuration> {
        self.wlans
            .iter()
            .map(|w| (w.id.clone(), w.initial.clone()))
            .collect()
    }
    pub fn inspect(&self, values: &BTreeMap<String, Configuration>) -> Result<Report> {
        self.validate()?;
        if values.len() != self.wlans.len()
            || self.wlans.iter().any(|w| !values.contains_key(&w.id))
        {
            return Err(
                "Configuration keys must match exactly the supplied WLAN identifiers.".into(),
            );
        }
        let mut issues = Vec::new();
        let mut names = BTreeSet::new();
        for w in &self.wlans {
            let v = &values[&w.id];
            if v.ssid.len() > 128 || v.training_passphrase.len() > 128 {
                return Err("WLAN configuration exceeds the bounded draft size.".into());
            }
            let mut issue = |field, feedback: &str| {
                issues.push(Issue {
                    wlan: w.id.clone(),
                    field,
                    feedback: feedback.into(),
                })
            };
            if !w.managed && *v != w.initial {
                issue(
                    "authority",
                    "This WLAN is outside your editing authority. Restore its supplied settings.",
                );
            }
            if v.ssid.is_empty() {
                issue("ssid", "Enter a network name before checking.");
            } else if !printable_ascii(&v.ssid, 1, 32) {
                issue(
                    "ssid",
                    "This simulator accepts 1–32 printable ASCII bytes for a network name. Other encodings are outside this activity.",
                );
            } else if !names.insert(v.ssid.as_str()) {
                issue(
                    "ssid",
                    "This activity needs one distinct name per supplied WLAN profile; duplicate-name profile selection is not modeled.",
                );
            }
            if v.authentication != Authentication::Open
                && !printable_ascii(&v.training_passphrase, 8, 63)
            {
                issue(
                    "training_passphrase",
                    "Use the supplied 8–63 character ASCII training passphrase. Hex keys and other SAE passphrase ranges are outside this activity.",
                );
            }
            if v.authentication == Authentication::Wpa3Sae
                && v.management_protection != ManagementProtection::Required
            {
                issue(
                    "management_protection",
                    "WPA3-Personal requires protected management frames. Select Required.",
                );
            }
            if v.authentication == Authentication::Open
                && v.management_protection != ManagementProtection::Disabled
            {
                issue(
                    "management_protection",
                    "Protected open networks are outside this model. Open mode here is unencrypted with protection Disabled.",
                );
            }
        }
        let clients = self
            .clients
            .iter()
            .map(|c| {
                let outcome = if !issues.is_empty() {
                    Compatibility::Deferred
                } else if let Some((id, v)) = values.iter().find(|(_, v)| v.ssid == c.ssid) {
                    if !v.enabled {
                        Compatibility::Disabled
                    } else if !c.allowed_authentication.contains(&v.authentication) {
                        Compatibility::AuthenticationMismatch
                    } else if v.management_protection == ManagementProtection::Required
                        && !c.management_protection_capable
                    {
                        Compatibility::ManagementProtectionRequired
                    } else if v.authentication != Authentication::Open
                        && v.training_passphrase != c.training_passphrase
                    {
                        Compatibility::PassphraseMismatch
                    } else {
                        Compatibility::Compatible {
                            wlan: id.clone(),
                            authentication: v.authentication,
                            protected_management: v.management_protection
                                != ManagementProtection::Disabled
                                && c.management_protection_capable,
                        }
                    }
                } else {
                    Compatibility::NameNotFound
                };
                (c.id.clone(), outcome)
            })
            .collect();
        Ok(Report { issues, clients })
    }
}

#[cfg(test)]
mod tests;
