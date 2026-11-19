//! Bounded ARP admission over the existing DHCP observer's actual Ethernet state.
//! No host ARP cache, packet socket, CLI, subprocess or network access.
use crate::{
    dhcp_snooping::{self as dhcp, PortTrust},
    mac_learning::{Mac, Observation},
    switching::{Jack, Mode},
};
pub use dhcp::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
type Result<T> = std::result::Result<T, Error>;
pub const MAX_JSON: usize = 1024 * 1024;
pub const MAX_ARRIVALS: u8 = 8;
fn error(kind: ErrorKind, message: &str) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn invalid(message: &str) -> Error {
    error(ErrorKind::Invalid, message)
}
fn unsupported(message: &str) -> Error {
    error(ErrorKind::Unsupported, message)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Request,
    Reply,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packet {
    pub operation: Operation,
    pub ethernet_source: Mac,
    pub ethernet_destination: Mac,
    pub sender_mac: Mac,
    pub sender_ip: Ipv4Addr,
    pub target_mac: Mac,
    pub target_ip: Ipv4Addr,
}
impl Packet {
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > 4096 {
            return Err(error(ErrorKind::Limit, "ARP metadata input exceeds4KiB."));
        }
        let p: Self = serde_json::from_str(text)
            .map_err(|_| invalid("Invalid or unsupported ARP metadata."))?;
        p.validate()?;
        Ok(p)
    }
    fn validate(&self) -> Result<()> {
        if !self.ethernet_source.unicast() || !self.sender_mac.unicast() {
            return Err(unsupported(
                "This ARP subset uses complete nonzero unicast Ethernet and sender MACs.",
            ));
        }
        if !self.ethernet_destination.unicast() && self.ethernet_destination != Mac::BROADCAST {
            return Err(unsupported(
                "Use an individual or broadcast Ethernet destination in this ARP subset.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checks {
    pub source_mac: bool,
    pub destination_mac: bool,
    pub ip: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Permit,
    Deny,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub action: Action,
    pub sender_ip: Ipv4Addr,
    pub sender_mac: Mac,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    pub vlan: u16,
    pub static_only: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,16>")]
    pub rules: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u16,16>")]
    pub vlans: Vec<u16>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PortTrust,16>")]
    pub ports: Vec<PortTrust>,
    pub checks: Checks,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Filter,16>")]
    pub filters: Vec<Filter>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    MissingBinding,
    ExplicitAclDeny,
    StaticAclMiss,
    SourceMacMismatch,
    DestinationMacMismatch,
    InvalidIp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    VlanNotInspected,
    TrustedIngress,
    DhcpBinding,
    ArpAcl,
}
impl Admission {
    pub fn inspected(self) -> bool {
        matches!(self, Self::DhcpBinding | Self::ArpAcl)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Unavailable {
        reason: String,
    },
    Rejected {
        cause: Rejection,
    },
    Forwarded {
        admission: Admission,
        forwarding: Observation,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct World {
    schema_version: u8,
    dhcp: dhcp::World,
    config: Config,
    last_arp_second: Option<u64>,
    arrivals: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u8,
    dhcp: dhcp::World,
    config: Config,
    last_arp_second: Option<u64>,
    arrivals: u8,
}
impl<'de> Deserialize<'de> for World {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let w = Wire::deserialize(d)?;
        let s = Self {
            schema_version: w.schema_version,
            dhcp: w.dhcp,
            config: w.config,
            last_arp_second: w.last_arp_second,
            arrivals: w.arrivals,
        };
        s.validate().map_err(serde::de::Error::custom)?;
        Ok(s)
    }
}
fn bad_ip(ip: Ipv4Addr) -> bool {
    ip.is_unspecified() || ip.is_broadcast() || ip.is_multicast()
}
impl World {
    pub fn new(dhcp: dhcp::World) -> Result<Self> {
        dhcp.validate()?;
        let config = Config {
            vlans: vec![],
            ports: dhcp
                .config()
                .ports
                .iter()
                .map(|p| PortTrust {
                    port: p.port.clone(),
                    trusted: false,
                })
                .collect(),
            checks: Checks::default(),
            filters: vec![],
        };
        let s = Self {
            schema_version: 1,
            dhcp,
            config,
            last_arp_second: None,
            arrivals: 0,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_JSON {
            return Err(error(ErrorKind::Limit, "ARP inspection state exceeds1MiB."));
        }
        serde_json::from_str(text).map_err(|_| {
            invalid("Invalid, unsupported or oversized versioned ARP inspection state.")
        })
    }
    pub fn dhcp(&self) -> &dhcp::World {
        &self.dhcp
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    fn validate_config(&self, config: &Config) -> Result<()> {
        let f = self.dhcp.forwarding().fabric();
        let sw = f
            .switches
            .values()
            .next()
            .ok_or_else(|| invalid("The supplied access switch is missing."))?;
        if config.vlans.len() > 16
            || config.filters.len() > 16
            || config.ports.len() != sw.ports.len()
            || config.vlans.iter().collect::<BTreeSet<_>>().len() != config.vlans.len()
            || config.vlans.iter().any(|v| !sw.vlans.contains(v))
            || config
                .ports
                .iter()
                .map(|p| p.port.as_str())
                .collect::<BTreeSet<_>>()
                != sw.ports.keys().map(String::as_str).collect()
            || config
                .filters
                .iter()
                .map(|f| f.vlan)
                .collect::<BTreeSet<_>>()
                .len()
                != config.filters.len()
            || config.filters.iter().any(|f| {
                !sw.vlans.contains(&f.vlan)
                    || f.rules.len() > 16
                    || f.rules.iter().any(|r| !r.sender_mac.unicast())
            })
        {
            return Err(invalid(
                "Use unique supplied VLANs, one ARP trust setting for each supplied port, and bounded exact IP/unicast-MAC rules.",
            ));
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.dhcp.validate()?;
        if self.schema_version != 1 {
            return Err(invalid("Unsupported ARP inspection schema version."));
        }
        self.validate_config(&self.config)?;
        let now = self.dhcp.forwarding().now();
        if self.arrivals > MAX_ARRIVALS
            || self.last_arp_second.is_some_and(|second| second > now)
            || (self.last_arp_second.is_none() && self.arrivals != 0)
            || (self.last_arp_second.is_some() && self.arrivals == 0)
        {
            return Err(invalid("Invalid bounded ARP arrival observations."));
        }
        Ok(())
    }
    pub fn configure(&mut self, mut config: Config) -> Result<()> {
        self.validate()?;
        self.validate_config(&config)?;
        config.vlans.sort_unstable();
        config.ports.sort_by(|a, b| a.port.cmp(&b.port));
        config.filters.sort_by_key(|f| f.vlan);
        // Rule order is meaningful and is never canonicalized.
        self.config = config;
        Ok(())
    }
    pub fn configure_dhcp(&mut self, config: dhcp::Config) -> Result<()> {
        self.validate()?;
        let mut trial = self.clone();
        trial.dhcp.configure(config)?;
        trial.validate()?;
        *self = trial;
        Ok(())
    }
    pub fn transmit_dhcp(&mut self, host: &str, packet: &dhcp::Packet) -> Result<dhcp::Outcome> {
        self.validate()?;
        let mut trial = self.clone();
        let outcome = trial.dhcp.transmit(host, packet)?;
        trial.validate()?;
        *self = trial;
        Ok(outcome)
    }
    pub fn advance(&mut self, seconds: u64) -> Result<()> {
        self.validate()?;
        let mut trial = self.clone();
        trial.dhcp.advance(seconds)?;
        trial.validate()?;
        *self = trial;
        Ok(())
    }
    fn ingress(&self, host: &str) -> Result<(String, u16, Option<&'static str>)> {
        let f = self.dhcp.forwarding().fabric();
        let endpoint = f
            .hosts
            .get(host)
            .ok_or_else(|| invalid("Choose a supplied endpoint."))?;
        let cable = f
            .cables
            .iter()
            .find(|c| {
                matches!(&c.a,Jack::Host{host:h}if h==host)
                    || matches!(&c.b,Jack::Host{host:h}if h==host)
            })
            .ok_or_else(|| unsupported("This ARP subset requires a directly attached endpoint."))?;
        let peer = if matches!(&cable.a,Jack::Host{host:h}if h==host) {
            &cable.b
        } else {
            &cable.a
        };
        let Jack::Switch { switch, port } = peer else {
            return Err(unsupported("Use the supplied access-switch attachment."));
        };
        let sw = &f.switches[switch];
        let p = &sw.ports[port];
        let Mode::Access { vlan } = p.mode else {
            return Err(unsupported("Trunks are outside this ARP subset."));
        };
        let failure = if !endpoint.up {
            Some("The source endpoint is stopped.")
        } else if !cable.up {
            Some("The source cable is down.")
        } else if !p.up {
            Some("The source access port is down.")
        } else if !sw.vlans.contains(&vlan) {
            Some("The source VLAN is inactive.")
        } else {
            None
        };
        Ok((port.clone(), vlan, failure))
    }
    fn inspect(
        &self,
        port: &str,
        vlan: u16,
        packet: &Packet,
    ) -> Result<std::result::Result<Admission, Rejection>> {
        let c = &self.config;
        if !c.vlans.contains(&vlan) {
            return Ok(Ok(Admission::VlanNotInspected));
        }
        if c.ports.iter().any(|p| p.port == port && p.trusted) {
            return Ok(Ok(Admission::TrustedIngress));
        }
        if c.checks.source_mac && packet.ethernet_source != packet.sender_mac {
            return Ok(Err(Rejection::SourceMacMismatch));
        }
        if c.checks.destination_mac
            && packet.operation == Operation::Reply
            && packet.ethernet_destination != packet.target_mac
        {
            return Ok(Err(Rejection::DestinationMacMismatch));
        }
        if c.checks.ip
            && (bad_ip(packet.sender_ip)
                || (packet.operation == Operation::Reply && bad_ip(packet.target_ip)))
        {
            return Ok(Err(Rejection::InvalidIp));
        }
        if packet.sender_ip.is_unspecified() {
            return Err(unsupported(
                "Special handling for ARP with a zero sender IP is not modeled when the IP check is off; no forwarding or rejection is asserted.",
            ));
        }
        if let Some(filter) = c.filters.iter().find(|f| f.vlan == vlan) {
            if let Some(rule) = filter
                .rules
                .iter()
                .find(|r| r.sender_ip == packet.sender_ip && r.sender_mac == packet.sender_mac)
            {
                return Ok(match rule.action {
                    Action::Permit => Ok(Admission::ArpAcl),
                    Action::Deny => Err(Rejection::ExplicitAclDeny),
                });
            }
            if filter.static_only {
                return Ok(Err(Rejection::StaticAclMiss));
            }
        }
        let matching = self.dhcp.bindings().iter().find(|b| {
            b.vlan == vlan && b.client == packet.sender_mac && b.address == packet.sender_ip
        });
        match matching {
            Some(b) if b.port != port => Err(unsupported(
                "This simulation does not decide what happens when a known DHCP address/MAC pair arrives on another port; nothing changed.",
            )),
            Some(_) => Ok(Ok(Admission::DhcpBinding)),
            None => Ok(Err(Rejection::MissingBinding)),
        }
    }
    pub fn transmit(&mut self, host: &str, packet: &Packet) -> Result<Outcome> {
        self.validate()?;
        packet.validate()?;
        let (port, vlan, failure) = self.ingress(host)?;
        if let Some(reason) = failure {
            return Ok(Outcome::Unavailable {
                reason: reason.into(),
            });
        }
        let mut trial = self.clone();
        let now = trial.dhcp.forwarding().now();
        if trial.last_arp_second != Some(now) {
            trial.last_arp_second = Some(now);
            trial.arrivals = 0;
        }
        if trial.arrivals == MAX_ARRIVALS {
            return Err(error(
                ErrorKind::Limit,
                "This simulator supports at most8ARP arrivals per virtual second. Advance the sample clock; hardware rate/error-disable behavior is not emulated.",
            ));
        }
        trial.arrivals += 1;
        let outcome = match trial.inspect(&port, vlan, packet)? {
            Err(cause) => Outcome::Rejected { cause },
            Ok(admission) => Outcome::Forwarded {
                admission,
                forwarding: trial.dhcp.forward_ordinary_frame(
                    host,
                    packet.ethernet_source,
                    packet.ethernet_destination,
                )?,
            },
        };
        trial.validate()?;
        *self = trial;
        Ok(outcome)
    }
}
