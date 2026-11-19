//! Source-bound IPv4-to-bridge attachment composition, not IP routing or host I/O.
use crate::{
    bridge_control as control, bridge_fabric as lan, cron_workspace as bounded,
    ipv4_routing::Assignment, linux_host, nm_profiles as ip,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
type Result<T> = std::result::Result<T, ip::Error>;
fn invalid(text: impl Into<String>) -> ip::Error {
    ip::Error {
        kind: ip::ErrorKind::Invalid,
        line: 0,
        message: text.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    pub address: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub endpoint: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    pub profiles: control::Source,
    pub lan: lan::Facts,
    #[serde(deserialize_with = "bounded::sequence::<_,Peer,6>")]
    pub peers: Vec<Peer>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub outward: bool,
    pub returning: bool,
    pub detail: String,
}
impl Delivery {
    pub fn round_trip(&self) -> bool {
        self.outward && self.returning
    }
}
impl Facts {
    fn shape(&self) -> Result<()> {
        self.profiles.validate()?;
        self.lan.validate().map_err(invalid)?;
        let source_members = self
            .profiles
            .members
            .iter()
            .map(|b| b.interface.as_str())
            .collect::<BTreeSet<_>>();
        let lan_members = self
            .lan
            .ports
            .iter()
            .map(|p| p.interface.0.as_str())
            .collect::<BTreeSet<_>>();
        let nics = self
            .lan
            .ports
            .iter()
            .map(|p| p.endpoint.as_str())
            .collect::<BTreeSet<_>>();
        let expected = self
            .lan
            .fabric
            .hosts
            .keys()
            .filter(|n| !nics.contains(n.as_str()))
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let endpoints = self
            .peers
            .iter()
            .map(|p| p.endpoint.as_str())
            .collect::<BTreeSet<_>>();
        if self.version != 1
            || self.profiles.controller.interface != self.lan.interface.0
            || source_members != lan_members
            || self.peers.is_empty()
            || self.peers.len() > 6
            || endpoints.len() != self.peers.len()
            || endpoints != expected
            || self
                .peers
                .iter()
                .map(|p| p.address)
                .collect::<BTreeSet<_>>()
                .len()
                != self.peers.len()
            || !bounded::within(self, 96 * 1024)
        {
            return Err(invalid(
                "Match one source bridge/two NICs and one distinct IPv4 address for every supplied LAN peer; do not invent ports, peers or duplicate addresses",
            ));
        }
        Ok(())
    }
    pub fn validate(&self, host: &linux_host::Profile) -> Result<()> {
        self.shape()?;
        host.validate().map_err(invalid)?;
        let bridge = &self.profiles.controller.interface;
        let link=host.links.iter().find(|l|&l.name==bridge).ok_or_else(||invalid("Provide the bridge controller's logical IPv4 link, not an address on a member NIC"))?;
        if host.ipv6.is_some()
            || self
                .profiles
                .members
                .iter()
                .any(|b| host.links.iter().any(|l| l.name == b.interface))
        {
            return Err(invalid(
                "This bridge activity uses manual IPv4 on the controller, with no IPv6 or IP-on-member roles",
            ));
        }
        for p in &self.peers {
            if !link.network.contains(p.address)
                || !(Assignment {
                    address: p.address,
                    prefix: link.network.length,
                })
                .usable()
            {
                return Err(invalid(
                    "Each bridge peer needs a distinct conventional address in the supplied bridge LAN",
                ));
            }
        }
        let required = host
            .nodes
            .iter()
            .filter(|n| link.network.contains(n.address))
            .map(|n| n.address)
            .chain(
                host.gateways
                    .iter()
                    .filter(|g| &g.device == bridge)
                    .map(|g| g.address),
            )
            .collect::<BTreeSet<_>>();
        if required != self.peers.iter().map(|p| p.address).collect() {
            return Err(invalid(
                "Map every supplied bridge-LAN destination/next hop to its actual provided peer, without invented IPv4 endpoints",
            ));
        }
        Ok(())
    }
    pub fn current(&self, snapshot: &control::Snapshot) -> Result<lan::Current> {
        self.shape()?;
        if snapshot.plan.source() != &self.profiles {
            return Err(invalid(
                "A bridge snapshot requires its original source identities/topology, not another controller's settings",
            ));
        }
        snapshot.projection()
    }
    pub fn validate_selection(
        &self,
        snapshot: &control::Snapshot,
        profiles: &BTreeMap<String, ip::Profile>,
    ) -> Result<()> {
        self.current(snapshot)?;
        if profiles.len() > 2 || profiles.iter().any(|(name, p)| name != &p.interface) {
            return Err(invalid(
                "Keep at most two original logical IP selections with matching interface keys",
            ));
        }
        let controller = &self.profiles.controller.interface;
        let selected = profiles.get(controller);
        if snapshot.current.controller != selected.is_some() {
            return Err(invalid(
                "A selected manual-IP bridge controller needs its matching current IP selection; do not invent one from future files",
            ));
        }
        if let Some(p) = selected {
            let saved = &snapshot.plan.controller().ipv4;
            let ip::Addressing::Manual { address, gateway } = &saved.addressing else {
                return Err(invalid(
                    "The bridge snapshot needs its original manual IPv4 configuration",
                ));
            };
            if *p != saved.resolved(address.clone(), *gateway) {
                return Err(invalid(
                    "Selected bridge IPv4 properties must belong to this owned controller snapshot",
                ));
            }
            if self
                .peers
                .iter()
                .any(|peer| peer.address == p.address.address)
            {
                return Err(invalid(
                    "The bridge address duplicates a provided peer; choose a different usable host address",
                ));
            }
        }
        if profiles.values().any(|p| {
            p.vlan.is_some()
                || self
                    .profiles
                    .members
                    .iter()
                    .any(|b| b.interface == p.interface)
        }) {
            return Err(invalid(
                "Nested VLAN profiles and IP configuration on bridge members are outside this source context",
            ));
        }
        Ok(())
    }
    pub fn delivery(
        &self,
        host: &linux_host::Profile,
        snapshot: &control::Snapshot,
        profiles: &BTreeMap<String, ip::Profile>,
        device: &str,
        next_hop: Ipv4Addr,
    ) -> Result<Delivery> {
        self.validate(host)?;
        self.validate_selection(snapshot, profiles)?;
        if device != self.profiles.controller.interface || !profiles.contains_key(device) {
            return Err(invalid(
                "Use the actually selected bridge controller for this Layer2 next hop",
            ));
        }
        let peer=self.peers.iter().find(|p|p.address==next_hop).ok_or_else(||invalid("No supplied bridge peer represents this next hop; no real discovery is attempted"))?;
        let current = self.current(snapshot)?;
        let visibility = self
            .lan
            .visibility(
                &current,
                lan::Endpoint::Local,
                lan::Endpoint::Peer(&peer.endpoint),
            )
            .map_err(invalid)?;
        Ok(Delivery {
            outward: visibility.forward,
            returning: visibility.reverse,
            detail: format!(
                "{device}: untagged delivery to {next_hop} {}; return to the bridge {}; attached NICs: {}. Address/route checks are separate from this Layer2 path.",
                if visibility.forward {
                    "delivered"
                } else {
                    "blocked"
                },
                if visibility.reverse {
                    "delivered"
                } else {
                    "blocked"
                },
                if current.members.is_empty() {
                    "none".into()
                } else {
                    current
                        .members
                        .iter()
                        .map(|m| m.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
        })
    }
    pub fn peer_delivery(
        &self,
        host: &linux_host::Profile,
        snapshot: &control::Snapshot,
        from: Ipv4Addr,
        to: Ipv4Addr,
    ) -> Result<Delivery> {
        self.validate(host)?;
        let current = self.current(snapshot)?;
        let endpoint = |address| {
            self.peers.iter().find(|p|p.address==address).ok_or_else(||invalid("Choose two provided bridge-LAN peers; unknown addresses are not discovered"))
        };
        let a = endpoint(from)?;
        let b = endpoint(to)?;
        let v = self
            .lan
            .visibility(
                &current,
                lan::Endpoint::Peer(&a.endpoint),
                lan::Endpoint::Peer(&b.endpoint),
            )
            .map_err(invalid)?;
        Ok(Delivery {
            outward: v.forward,
            returning: v.reverse,
            detail: format!(
                "Untagged broadcast {from} → {to} {}; reverse {}; this is bridge forwarding, not host IP routing or a real ping.",
                if v.forward { "delivered" } else { "blocked" },
                if v.reverse { "delivered" } else { "blocked" }
            ),
        })
    }
}
