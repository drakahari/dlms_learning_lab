//! Optional source-held L2 attachment facts for NetworkManager VLAN activities.
//! Composes bounded switching with selected profile metadata, never host networking.
use crate::{
    cron_workspace as bounded, linux_host, nm_profiles,
    switching::{self, Jack, Mode},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
type Result<T> = std::result::Result<T, String>;
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 15
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parent {
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub device: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub endpoint: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    pub address: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub endpoint: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceName(#[serde(deserialize_with = "crate::cron::text::<_,15>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    pub fabric: switching::Fabric,
    #[serde(deserialize_with = "bounded::sequence::<_,Parent,2>")]
    pub parents: Vec<Parent>,
    #[serde(deserialize_with = "bounded::sequence::<_,InterfaceName,2>")]
    pub interfaces: Vec<InterfaceName>,
    #[serde(deserialize_with = "bounded::sequence::<_,Peer,12>")]
    pub peers: Vec<Peer>,
}
fn endpoint_mode<'a>(f: &'a switching::Fabric, host: &str) -> Result<&'a Mode> {
    let key = Jack::Host { host: host.into() };
    let other = f
        .cables
        .iter()
        .find_map(|c| {
            if c.a == key {
                Some(&c.b)
            } else if c.b == key {
                Some(&c.a)
            } else {
                None
            }
        })
        .ok_or("Every source parent/peer needs one supplied cable, which may be down")?;
    let Jack::Switch { switch, port } = other else {
        return Err("VLAN endpoints need a provided switch attachment".into());
    };
    Ok(&f.switches[switch].ports[port].mode)
}
impl Facts {
    pub(crate) fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported VLAN facts version".into());
        }
        self.fabric.validate()?;
        if !bounded::within(self, 64 * 1024)
            || self.parents.is_empty()
            || self.parents.len() > 2
            || self.interfaces.is_empty()
            || self.interfaces.len() > 2
            || self.peers.is_empty()
            || self.peers.len() > 12
            || self.fabric.switches.len() > 2
            || self.fabric.hosts.len() > 8
            || self.fabric.cables.len() > 12
            || self
                .fabric
                .cables
                .iter()
                .filter(|c| {
                    matches!(c.a, Jack::Switch { .. }) && matches!(c.b, Jack::Switch { .. })
                })
                .count()
                > 1
        {
            return Err("VLAN source facts exceed the small supplied acyclic fabric/parent/interface/peer subset".into());
        }
        let software = self
            .interfaces
            .iter()
            .map(|n| n.0.as_str())
            .collect::<BTreeSet<_>>();
        if software.len() != self.interfaces.len() || software.iter().any(|n| !name(n)) {
            return Err("Use distinct provided VLAN software-interface names".into());
        }
        let mut devices = BTreeSet::new();
        let mut parents = BTreeSet::new();
        let mut endpoints = BTreeSet::new();
        let mut ips = BTreeSet::new();
        for p in &self.parents {
            if !name(&p.device)
                || software.contains(p.device.as_str())
                || !devices.insert(&p.device)
                || !parents.insert(&p.endpoint)
                || !self.fabric.hosts.contains_key(&p.endpoint)
                || !matches!(
                    endpoint_mode(&self.fabric, &p.endpoint)?,
                    Mode::Trunk { .. }
                )
            {
                return Err("Physical parents need distinct provided names/endpoints attached to supported trunk ports; nested VLANs/tagged Access input are outside this activity".into());
            }
            endpoints.insert(p.endpoint.as_str());
        }
        for p in &self.peers {
            if !ips.insert(p.address)
                || parents.contains(&p.endpoint)
                || !self.fabric.hosts.contains_key(&p.endpoint)
                || !matches!(
                    endpoint_mode(&self.fabric, &p.endpoint)?,
                    Mode::Access { .. }
                )
            {
                return Err("Each provided IPv4 peer needs an untagged access endpoint, distinct from physical parents".into());
            }
            endpoints.insert(p.endpoint.as_str());
        }
        if endpoints != self.fabric.hosts.keys().map(String::as_str).collect() {
            return Err(
                "Every fabric endpoint needs an explained physical-parent or peer role".into(),
            );
        }
        Ok(())
    }
    pub fn validate(&self, ip: &linux_host::Profile) -> Result<()> {
        self.shape()?;
        ip.validate()?;
        if ip.ipv6.is_some()
            || self
                .interfaces
                .iter()
                .map(|n| &n.0)
                .collect::<BTreeSet<_>>()
                != ip.links.iter().map(|l| &l.name).collect()
            || ip.links.iter().any(|l| !l.carrier)
        {
            return Err("VLAN cases use the provided IPv4 logical-interface/network projection; real parent availability belongs to the separate L2 facts".into());
        }
        let known = ip
            .nodes
            .iter()
            .map(|n| n.address)
            .chain(ip.gateways.iter().map(|g| g.address))
            .collect::<BTreeSet<_>>();
        if self
            .peers
            .iter()
            .map(|p| p.address)
            .collect::<BTreeSet<_>>()
            != known
        {
            return Err("Every supplied IPv4 destination/next hop needs a source-held L2 peer, without extra invented addresses".into());
        }
        Ok(())
    }
    pub fn validate_profile(&self, p: &nm_profiles::Settings) -> Result<()> {
        self.shape()?;
        if !self.interfaces.iter().any(|n| n.0 == p.interface) {
            return Err("Choose one supplied VLAN software-interface name; discovery/new interfaces are outside this activity".into());
        }
        let v=p.vlan.as_ref().ok_or("This provided software interface needs type=vlan with an explicit parent and ID; Ethernet type does not create a VLAN")?;
        if !(1..=4094).contains(&v.id) || !self.parents.iter().any(|p| p.device == v.parent) {
            return Err("Use a provided physical parent and a data VLAN ID1–4094; priority tag0 is outside this subset".into());
        }
        if !matches!(p.addressing, nm_profiles::Addressing::Manual { .. }) {
            return Err(
                "DHCP-on-VLAN is not modeled here; retain the supplied manual IPv4 assignment"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn validate_selection(
        &self,
        profiles: &BTreeMap<String, nm_profiles::Profile>,
    ) -> Result<()> {
        self.receivers(profiles).map(|_| ())
    }
    fn receivers(
        &self,
        profiles: &BTreeMap<String, nm_profiles::Profile>,
    ) -> Result<BTreeMap<String, BTreeSet<u16>>> {
        self.shape()?;
        if profiles.len() > 2 {
            return Err("At most two provided VLAN software interfaces may be selected".into());
        }
        let mut receivers = self
            .parents
            .iter()
            .map(|p| (p.endpoint.clone(), BTreeSet::new()))
            .collect::<BTreeMap<_, _>>();
        let mut devices = BTreeSet::new();
        for (key, p) in profiles {
            if key != &p.interface || !self.interfaces.iter().any(|n| &n.0 == key) {
                return Err("The selected VLAN snapshot has a foreign software interface".into());
            }
            let v = p
                .vlan
                .as_ref()
                .ok_or("Selected software interface has no VLAN descriptor")?;
            if !(1..=4094).contains(&v.id) {
                return Err("Selected VLAN ID is outside the supported data range".into());
            }
            let parent = self
                .parents
                .iter()
                .find(|q| q.device == v.parent)
                .ok_or("Selected VLAN parent is not provided")?;
            if !devices.insert((&v.parent, v.id)) {
                return Err("Two VLAN software interfaces cannot share the same physical parent and ID in this namespace".into());
            }
            receivers.get_mut(&parent.endpoint).unwrap().insert(v.id);
        }
        Ok(receivers)
    }
    pub fn delivery(
        &self,
        profiles: &BTreeMap<String, nm_profiles::Profile>,
        device: &str,
        next_hop: Ipv4Addr,
    ) -> Result<Delivery> {
        let mut receivers = self.receivers(profiles)?;
        let selected = profiles
            .get(device)
            .ok_or("No active profile on the selected software interface")?;
        let v = selected
            .vlan
            .as_ref()
            .ok_or("No VLAN descriptor on the selected profile")?;
        let parent = self
            .parents
            .iter()
            .find(|p| p.device == v.parent)
            .ok_or("Unknown selected physical parent")?;
        let peer = self.peers.iter().find(|p| p.address == next_hop).ok_or(
            "This path has no supplied next-hop L2 endpoint; no real discovery is attempted",
        )?;
        let outward = self
            .fabric
            .broadcast_frame(&parent.endpoint, Some(v.id), &receivers)?;
        // Delivery to another VLAN on this NIC is not a return to the selected
        // software interface. The reverse probe must accept this ID only.
        receivers.insert(parent.endpoint.clone(), BTreeSet::from([v.id]));
        let returning = self
            .fabric
            .broadcast_frame(&peer.endpoint, None, &receivers)?;
        let forward = outward.recipients.contains_key(&peer.endpoint);
        let back = returning.recipients.contains_key(&parent.endpoint);
        let mut detail = format!(
            "VLAN{} on {}: outward to {next_hop} {}; return to this software interface {}.",
            v.id,
            v.parent,
            if forward {
                "delivered"
            } else {
                "not delivered"
            },
            if back { "delivered" } else { "not delivered" }
        );
        for (pass, r, target) in [
            (forward, &outward, peer.endpoint.as_str()),
            (back, &returning, parent.endpoint.as_str()),
        ] {
            if !pass {
                let prefix = format!("{target}:");
                if let Some(reason) = r
                    .drops
                    .iter()
                    .find(|s| s.starts_with(&prefix))
                    .or_else(|| r.drops.first())
                {
                    detail.push_str(&format!(" Observed drop: {reason}."));
                }
            }
        }
        Ok(Delivery {
            outward: forward,
            returning: back,
            detail,
        })
    }
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
