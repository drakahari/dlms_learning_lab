//! Preserve unfinished IPv4 fields independently from the compiled routing view.
use super::svi_projection::{Edit, SviConfig};
use crate::ipv4_routing::{Assignment, supported_address};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

type Result<T> = std::result::Result<T, String>;
const ADDRESS_BYTES: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AddressText(
    #[serde(deserialize_with = "crate::cron::text::<_,ADDRESS_BYTES>")] pub String,
);

impl AddressText {
    pub fn valid_draft(&self) -> bool {
        self.0.len() <= ADDRESS_BYTES && !self.0.chars().any(char::is_control)
    }

    pub fn gateway(&self) -> Result<Option<Ipv4Addr>> {
        let text = self.0.trim();
        if text.is_empty() {
            return Ok(None);
        }
        let address = text
            .parse::<Ipv4Addr>()
            .ok()
            .filter(|a| supported_address(*a))
            .ok_or(
                "Enter an IPv4 address from the supplied plan as four numbers separated by dots.",
            )?;
        Ok(Some(address))
    }

    pub fn cidr(&self) -> Result<Option<Assignment>> {
        let text = self.0.trim();
        if text.is_empty() {
            return Ok(None);
        }
        let (ip, length) = text
            .split_once('/')
            .ok_or("Include the prefix after a slash, for example 192.0.2.9/24.")?;
        let address = AddressText(ip.into())
            .gateway()?
            .ok_or("Put the IPv4 address before the slash.")?;
        if length.is_empty() || !length.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Use a whole prefix length from 1 to 30 in this exercise.".into());
        }
        let prefix = length
            .parse::<u8>()
            .ok()
            .filter(|p| (1..=30).contains(p))
            .ok_or("Use a whole prefix length from 1 to 30 in this exercise.")?;
        Ok(Some(Assignment { address, prefix }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub routing: bool,
    pub enable: [bool; 2],
    pub interfaces: [AddressText; 2],
    pub gateways: [AddressText; 3],
    pub allow_remote: [bool; 2],
}

impl From<&Edit> for Draft {
    fn from(edit: &Edit) -> Self {
        Self {
            routing: edit.routing,
            enable: std::array::from_fn(|i| edit.interfaces[i].enabled),
            interfaces: std::array::from_fn(|i| {
                AddressText(
                    edit.interfaces[i]
                        .address
                        .as_ref()
                        .map_or_else(String::new, |a| format!("{}/{}", a.address, a.prefix)),
                )
            }),
            gateways: std::array::from_fn(|i| {
                AddressText(edit.gateways[i].map_or_else(String::new, |a| a.to_string()))
            }),
            allow_remote: edit.allow_remote,
        }
    }
}

impl Draft {
    pub fn valid(&self) -> bool {
        self.interfaces
            .iter()
            .chain(&self.gateways)
            .all(AddressText::valid_draft)
    }

    pub fn compile(&self) -> Edit {
        Edit {
            routing: self.routing,
            interfaces: std::array::from_fn(|i| SviConfig {
                enabled: self.enable[i],
                address: self.interfaces[i].cidr().ok().flatten(),
            }),
            gateways: std::array::from_fn(|i| self.gateways[i].gateway().ok().flatten()),
            allow_remote: self.allow_remote,
        }
    }
}
