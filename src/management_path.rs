//! Real existing IPv4 endpoint paths plus explicit per-line metadata conditions.
//! No TCP/SSH/authentication claim and no persisted result is progress authority.
use crate::{ipv4_routing as ip, management_policy as policy};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};

pub const MAX_JSON_BYTES: usize = 1536 * 1024;
pub const MAX_CLIENTS: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub device: String,
    pub interface: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub id: String,
    pub binding: Binding,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub version: u32,
    pub topology: ip::Topology,
    pub management: Binding,
    pub clients: Vec<Client>,
}
#[derive(Clone, Debug)]
pub enum Observation {
    PathUnavailable {
        reason: String,
    },
    Policy {
        path: ip::Probe,
        conditions: policy::Assessment,
    },
}
impl Observation {
    pub fn allowed_to_authenticate(&self) -> bool {
        matches!(self, Self::Policy { conditions, .. } if conditions.allowed_to_authenticate())
    }
}
fn fail(kind: policy::ErrorKind, message: &str) -> policy::Error {
    policy::Error {
        kind,
        message: message.into(),
    }
}
fn named(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
}
impl Network {
    pub fn decode(raw: &[u8]) -> Result<Self, policy::Error> {
        if raw.len() > MAX_JSON_BYTES {
            return Err(fail(
                policy::ErrorKind::Limit,
                "Management path data exceeds 1.5 MiB.",
            ));
        }
        let value: Self = serde_json::from_slice(raw).map_err(|_| {
            fail(
                policy::ErrorKind::Invalid,
                "Malformed or unsupported management path fields or topology.",
            )
        })?;
        value.validate()?;
        Ok(value)
    }
    fn address(&self, b: &Binding) -> Result<Ipv4Addr, policy::Error> {
        if !named(&b.device) || !named(&b.interface) {
            return Err(fail(
                policy::ErrorKind::Invalid,
                "Use a supplied device/interface binding.",
            ));
        }
        let address = self
            .topology
            .devices
            .get(&b.device)
            .and_then(|d| d.interfaces.get(&b.interface))
            .and_then(|i| i.address.as_ref())
            .filter(|a| a.usable())
            .ok_or_else(|| {
                fail(
                    policy::ErrorKind::Invalid,
                    "A bound endpoint needs a supplied usable IPv4 address.",
                )
            })?
            .address;
        let owners = self
            .topology
            .devices
            .values()
            .flat_map(|d| d.interfaces.values())
            .filter(|i| i.address.as_ref().is_some_and(|a| a.address == address))
            .count();
        if owners != 1 {
            return Err(fail(
                policy::ErrorKind::Invalid,
                "A bound address has multiple owners; endpoint identity is ambiguous.",
            ));
        }
        Ok(address)
    }
    pub fn validate(&self) -> Result<(), policy::Error> {
        if self.version != 1 {
            return Err(fail(
                policy::ErrorKind::Unsupported,
                "Unsupported management path version.",
            ));
        }
        if !self.topology.valid() {
            return Err(fail(
                policy::ErrorKind::Invalid,
                "Invalid or oversized supplied IPv4 topology.",
            ));
        }
        if self.clients.is_empty() || self.clients.len() > MAX_CLIENTS {
            return Err(fail(
                policy::ErrorKind::Limit,
                "Supply 1–6 named management clients.",
            ));
        }
        self.address(&self.management)?;
        let mut names = BTreeSet::new();
        let mut bindings = BTreeSet::new();
        for client in &self.clients {
            if !named(&client.id)
                || !names.insert(&client.id)
                || !bindings.insert((&client.binding.device, &client.binding.interface))
                || client.binding.device == self.management.device
            {
                return Err(fail(
                    policy::ErrorKind::Invalid,
                    "Client names/bindings must be unique and separate from the management device.",
                ));
            }
            self.address(&client.binding)?;
        }
        Ok(())
    }
    /// No learner source address, topology callback or host connection is accepted.
    pub fn inspect(
        &self,
        settings: &policy::Policy,
        client: &str,
        line: u8,
        protocol: policy::Protocol,
    ) -> Result<Observation, policy::Error> {
        self.validate()?;
        settings.validate()?;
        if !named(client) {
            return Err(fail(
                policy::ErrorKind::Invalid,
                "Select a supplied management client.",
            ));
        }
        let bound = &self
            .clients
            .iter()
            .find(|c| c.id == client)
            .ok_or_else(|| {
                fail(
                    policy::ErrorKind::Invalid,
                    "The selected management client was not supplied.",
                )
            })?
            .binding;
        if !settings.lines.iter().any(|l| l.number == line) {
            return Err(fail(
                policy::ErrorKind::Invalid,
                "The selected policy line was not supplied.",
            ));
        }
        let source = self.address(bound)?;
        if !self.topology.devices[&bound.device].interfaces[&bound.interface].up {
            return Ok(Observation::PathUnavailable {
                reason: "The supplied client's interface is down.".into(),
            });
        }
        let path = match self.topology.verify_endpoint(
            &bound.device,
            &self.management.device,
            &self.management.interface,
        ) {
            Ok(path) => path,
            Err(reason) => return Ok(Observation::PathUnavailable { reason }),
        };
        if path.source != source
            || path
                .forward
                .hops
                .first()
                .is_none_or(|hop| hop.interface != bound.interface)
        {
            return Err(fail(
                policy::ErrorKind::Unsupported,
                "Routing selected another source interface; alternate source-address policy is outside this activity.",
            ));
        }
        let conditions = settings.assess(policy::Attempt {
            line,
            protocol,
            source,
        })?;
        Ok(Observation::Policy { path, conditions })
    }
}
