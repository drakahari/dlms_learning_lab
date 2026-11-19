//! Opt-in composition of explicit bond snapshots and the existing tagged paths.
//! Carrier is derived from supplied physical attachment states, never IP or tags.
use crate::{
    bond_control as control, nm_profiles as nm,
    switching::Jack,
    vlan_host as vlan,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
type Result<T> = std::result::Result<T, nm::Error>;
fn error(kind: nm::ErrorKind, message: impl Into<String>) -> nm::Error {
    nm::Error {
        kind,
        line: 0,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    pub profiles: control::Source,
}
impl Facts {
    pub fn validate(&self, v: &vlan::Facts) -> Result<()> {
        self.profiles.validate()?;
        v.shape().map_err(|s| error(nm::ErrorKind::Invalid, s))?;
        if self.version != 1
            || self
                .profiles
                .members
                .iter()
                .map(|b| &b.interface)
                .collect::<BTreeSet<_>>()
                != v.parents.iter().map(|p| &p.device).collect()
            || v.interfaces
                .iter()
                .any(|n| n.0 == self.profiles.controller.interface)
        {
            return Err(error(
                nm::ErrorKind::Invalid,
                "Provide one distinct bond above exactly the two supplied physical parents; software VLAN names, member NICs and controller name are different roles",
            ));
        }
        Ok(())
    }
    pub fn carrier(&self, v: &vlan::Facts) -> Result<BTreeMap<String, bool>> {
        self.validate(v)?;
        v.parents
            .iter()
            .map(|p| {
                let host = Jack::Host {
                    host: p.endpoint.clone(),
                };
                let cable = v
                    .fabric
                    .cables
                    .iter()
                    .find(|c| c.a == host || c.b == host)
                    .ok_or_else(|| {
                        error(nm::ErrorKind::Invalid, "Missing supplied member cable")
                    })?;
                let other = if cable.a == host { &cable.b } else { &cable.a };
                let Jack::Switch { switch, port } = other else {
                    return Err(error(
                        nm::ErrorKind::Invalid,
                        "A physical member must attach to a provided switch port",
                    ));
                };
                // These are the exercise's settled physical observations. An active/allowed
                // VLAN or a successful destination probe is deliberately NOT a carrier test.
                Ok((
                    p.device.clone(),
                    v.fabric.hosts[&p.endpoint].up
                        && cable.up
                        && v.fabric.switches[switch].ports[port].up,
                ))
            })
            .collect()
    }
    fn binding(&self, v: &vlan::Facts, interface: &str, tag: Option<&nm::Vlan>) -> Result<()> {
        self.validate(v)?;
        let tag = tag.ok_or_else(|| {
            error(
                nm::ErrorKind::Unsupported,
                "These logical interfaces need explicit VLAN metadata above the bond",
            )
        })?;
        if !v.interfaces.iter().any(|n| n.0 == interface)
            || tag.parent != self.profiles.controller.interface
            || !(1..=4094).contains(&tag.id)
        {
            return Err(error(
                nm::ErrorKind::Invalid,
                "Use the supplied software VLAN name, bond parent and data ID1–4094. Do not bypass the bond with an enslaved physical NIC",
            ));
        }
        Ok(())
    }
    pub fn validate_settings(&self, v: &vlan::Facts, p: &nm::Settings) -> Result<()> {
        self.binding(v, &p.interface, p.vlan.as_ref())?;
        if !matches!(p.addressing, nm::Addressing::Manual { .. }) {
            return Err(error(
                nm::ErrorKind::Unsupported,
                "Only manual IPv4 on the supplied VLANs above this bond is modeled",
            ));
        }
        self.distinct_identity(&p.uuid)
    }
    fn distinct_identity(&self, uuid: &str) -> Result<()> {
        if std::iter::once(&self.profiles.controller)
            .chain(&self.profiles.members)
            .any(|b| b.uuid.eq_ignore_ascii_case(uuid))
        {
            return Err(error(
                nm::ErrorKind::Invalid,
                "Keep controller, member and VLAN UUID identities distinct",
            ));
        }
        Ok(())
    }
    pub fn validate_selection(
        &self,
        v: &vlan::Facts,
        profiles: &BTreeMap<String, nm::Profile>,
    ) -> Result<()> {
        self.validate(v)?;
        if profiles.len() > 2 {
            return Err(error(
                nm::ErrorKind::Limit,
                "At most two provided VLAN software interfaces can use this bond",
            ));
        }
        let mut tags = BTreeSet::new();
        for (key, p) in profiles {
            if key != &p.interface {
                return Err(error(
                    nm::ErrorKind::Invalid,
                    "The VLAN snapshot contains a foreign interface key",
                ));
            }
            self.binding(v, key, p.vlan.as_ref())?;
            self.distinct_identity(&p.uuid)?;
            if !tags.insert(p.vlan.as_ref().unwrap().id) {
                return Err(error(
                    nm::ErrorKind::Invalid,
                    "Distinct software VLANs cannot share the same bond parent and VLAN ID",
                ));
            }
        }
        Ok(())
    }
    pub fn selection(
        &self,
        v: &vlan::Facts,
        snapshot: &control::Snapshot,
    ) -> Result<control::Selection> {
        self.validate(v)?;
        if snapshot.plan.source != self.profiles {
            return Err(error(
                nm::ErrorKind::Invalid,
                "Control snapshot belongs to different supplied identities; retain its original source context",
            ));
        }
        snapshot.selection(&self.carrier(v)?)
    }
    /// Retain logical profile identity in the caller; derive a bounded physical
    /// projection using only the selected control snapshot and observed carrier.
    pub fn projected(
        &self,
        v: &vlan::Facts,
        snapshot: &control::Snapshot,
        profiles: &BTreeMap<String, nm::Profile>,
    ) -> Result<Option<BTreeMap<String, nm::Profile>>> {
        self.validate_selection(v, profiles)?;
        let selected = self.selection(v, snapshot)?;
        let Some(active) = selected.active else {
            return Ok(None);
        };
        let mut derived = profiles.clone();
        for p in derived.values_mut() {
            p.vlan.as_mut().unwrap().parent = active.clone();
        }
        v.validate_selection(&derived)
            .map_err(|s| error(nm::ErrorKind::Invalid, s))?;
        Ok(Some(derived))
    }
    pub fn delivery(
        &self,
        v: &vlan::Facts,
        snapshot: &control::Snapshot,
        profiles: &BTreeMap<String, nm::Profile>,
        device: &str,
        next_hop: Ipv4Addr,
    ) -> Result<vlan::Delivery> {
        if !profiles.contains_key(device) {
            return Err(error(
                nm::ErrorKind::Invalid,
                "No selected VLAN profile on the requested software interface",
            ));
        }
        let Some(derived) = self.projected(v, snapshot, profiles)? else {
            return Ok(vlan::Delivery {
                outward: false,
                returning: false,
                detail: format!(
                    "{}: no selected monitored member is available. Inspect controller/member eligibility and supplied carrier; an IP address alone does not create a physical path.",
                    self.profiles.controller.interface
                ),
            });
        };
        let active = &derived[device].vlan.as_ref().unwrap().parent;
        let mut r = v
            .delivery(&derived, device, next_hop)
            .map_err(|s| error(nm::ErrorKind::Unsupported, s))?;
        r.detail = format!(
            "{} active {active}: {}",
            self.profiles.controller.interface, r.detail
        );
        Ok(r)
    }
}
