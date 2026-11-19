//! Draft bounded forwarding composition. Not exported or catalog coverage yet.
use crate::switching::{Fabric, Hop, Jack, Mode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApMode {
    Local,
    FlexConnect,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Forwarding {
    Central,
    Local,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub label: String,
    pub forwarding: Forwarding,
    pub local_vlan: u16,
    pub controller_vlan: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub fabric: Fabric,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub ap: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub controller: String,
    pub mode: ApMode,
    pub ap_native: u16,
    pub controller_native: u16,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Mapping,4>")]
    pub wlans: Vec<Mapping>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Control {
    Ready,
    NativeMismatch { configured: u16, attached: u16 },
    ControllerNativeMismatch { configured: u16, attached: u16 },
    NoBidirectionalPath,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataPath {
    pub forwarding: Forwarding,
    pub vlan: u16,
    pub forward: Option<Vec<Hop>>,
    pub reverse: Option<Vec<Hop>>,
}
impl DataPath {
    pub fn bidirectional(&self) -> bool {
        self.forward.is_some() && self.reverse.is_some()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Deferred(Control),
    Data(DataPath),
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn vlan(n: u16) -> bool {
    (1..=4094).contains(&n)
}
impl World {
    pub fn validate(&self) -> Result<()> {
        self.fabric.validate()?;
        let endpoints = [&self.ap, &self.controller];
        if endpoints.iter().collect::<BTreeSet<_>>().len() != 2
            || endpoints
                .iter()
                .any(|s| !self.fabric.hosts.contains_key(*s))
        {
            return Err("Supply distinct AP and controller endpoints.".into());
        }
        if !vlan(self.ap_native) || !vlan(self.controller_native) {
            return Err("Native VLANs use IDs 1–4094 in this activity.".into());
        }
        if self.wlans.is_empty()
            || self.wlans.len() > 4
            || self
                .wlans
                .iter()
                .map(|w| &w.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.wlans.len()
            || self.wlans.iter().any(|w| {
                !id(&w.id)
                    || w.label.trim().is_empty()
                    || w.label.len() > 100
                    || w.label.chars().any(char::is_control)
                    || !vlan(w.local_vlan)
                    || !vlan(w.controller_vlan)
            })
        {
            return Err(
                "Supply 1–4 unique named WLAN mappings and valid local/controller VLANs.".into(),
            );
        }
        // Require wired attachments, not a direct AP/controller host-to-host cable.
        self.attached_native(&self.ap)?;
        self.attached_native(&self.controller)?;
        Ok(())
    }
    fn attached_port(&self, host: &str) -> Result<&crate::switching::Port> {
        let jack = Jack::Host { host: host.into() };
        let peer = self.fabric.cables.iter().find_map(|c| {
            if c.a == jack {
                Some(&c.b)
            } else if c.b == jack {
                Some(&c.a)
            } else {
                None
            }
        });
        let Some(Jack::Switch { switch, port }) = peer else {
            return Err(
                "Each supplied AP/controller endpoint needs its own switch attachment.".into(),
            );
        };
        Ok(&self.fabric.switches[switch].ports[port])
    }
    fn attached_native(&self, host: &str) -> Result<u16> {
        Ok(match self.attached_port(host)?.mode {
            Mode::Access { vlan } => vlan,
            Mode::Trunk { native, .. } => native,
        })
    }
    pub fn control(&self) -> Result<Control> {
        self.validate()?;
        let attached = self.attached_native(&self.ap)?;
        if attached != self.ap_native {
            return Ok(Control::NativeMismatch {
                configured: self.ap_native,
                attached,
            });
        }
        let controller_attached = self.attached_native(&self.controller)?;
        if controller_attached != self.controller_native {
            return Ok(Control::ControllerNativeMismatch {
                configured: self.controller_native,
                attached: controller_attached,
            });
        }
        Ok(
            if self.fabric.visibility(&self.ap, &self.controller)?.mutual() {
                Control::Ready
            } else {
                Control::NoBidirectionalPath
            },
        )
    }
    pub fn data_path(&self, wlan: &str, destination: &str) -> Result<Outcome> {
        self.validate()?;
        let mapping = self
            .wlans
            .iter()
            .find(|w| w.id == wlan)
            .ok_or("Choose a supplied WLAN mapping.")?;
        if !self.fabric.hosts.contains_key(destination)
            || [&self.ap, &self.controller]
                .iter()
                .any(|s| s.as_str() == destination)
        {
            return Err("Choose a separate supplied client-service endpoint.".into());
        }
        let control = self.control()?;
        if control != Control::Ready {
            return Ok(Outcome::Deferred(control));
        }
        // Cisco Local AP mode central-switches even if a WLAN permits local switching.
        let effective = if self.mode == ApMode::Local {
            Forwarding::Central
        } else {
            mapping.forwarding
        };
        let (source, native, data_vlan) = match effective {
            Forwarding::Central => (
                &self.controller,
                self.controller_native,
                mapping.controller_vlan,
            ),
            Forwarding::Local => (&self.ap, self.ap_native, mapping.local_vlan),
        };
        let tag = (data_vlan != native).then_some(data_vlan);
        if tag.is_some() && matches!(self.attached_port(source)?.mode, Mode::Access { .. }) {
            let role = if source == &self.ap {
                "AP"
            } else {
                "controller"
            };
            return Err(format!(
                "The {role} attachment is Access, but this WLAN needs tagged VLAN {data_vlan}. This activity requires a Trunk attachment carrying that VLAN for this path; tagged Access behavior is outside its subset."
            ));
        }
        let forward = self
            .fabric
            .broadcast_frame(source, tag, &BTreeMap::new())?
            .recipients
            .remove(destination);
        let receivers = match tag {
            Some(n) => BTreeMap::from([(source.clone(), BTreeSet::from([n]))]),
            None => BTreeMap::new(),
        };
        let reverse = self
            .fabric
            .broadcast_frame(destination, None, &receivers)?
            .recipients
            .remove(source);
        Ok(Outcome::Data(DataPath {
            forwarding: effective,
            vlan: data_vlan,
            forward,
            reverse,
        }))
    }
}

#[cfg(test)]
pub(crate) mod fixture;
#[cfg(test)]
mod tests;
