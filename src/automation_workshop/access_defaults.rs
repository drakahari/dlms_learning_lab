//! Source-bound default-access membership exercises. Existing Fabric serialization is unchanged.
use crate::{
    automation_workshop::workspace::Outcome,
    switching::{Fabric, Jack, Mode},
};
use serde::{Deserialize, Serialize};

const DEFAULT_ACCESS: u16 = 1;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub fabric: Fabric,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub desk: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub desk_peer: String,
    pub legacy: [String; 2],
    pub initial: Choice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub access: Option<u16>,
}

pub fn access_jack<'a>(fabric: &'a Fabric, host: &str) -> Result<(&'a str, &'a str, u16)> {
    for cable in &fabric.cables {
        for (here, other) in [(&cable.a, &cable.b), (&cable.b, &cable.a)] {
            if let (Jack::Host { host: name }, Jack::Switch { switch, port }) = (here, other)
                && name == host
            {
                let Some(cfg) = fabric.switches.get(switch).and_then(|s| s.ports.get(port)) else {
                    continue;
                };
                if let Mode::Access { vlan } = cfg.mode {
                    return Ok((switch, port, vlan));
                }
            }
        }
    }
    Err("The supplied station must have a known access-port attachment.".into())
}

impl Choice {
    pub fn effective(self) -> u16 {
        self.access.unwrap_or(DEFAULT_ACCESS)
    }
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        self.fabric.validate()?;
        let endpoints = [
            &self.desk,
            &self.desk_peer,
            &self.legacy[0],
            &self.legacy[1],
        ];
        if self.fabric.switches.len() != 2
            || self.fabric.hosts.len() != endpoints.len()
            || endpoints
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != endpoints.len()
            || !endpoints
                .iter()
                .all(|endpoint| self.fabric.hosts.contains_key(*endpoint))
            || self
                .fabric
                .switches
                .values()
                .any(|s| !s.vlans.contains(&DEFAULT_ACCESS))
            || self
                .legacy
                .iter()
                .any(|h| !matches!(access_jack(&self.fabric, h), Ok((_, _, DEFAULT_ACCESS))))
        {
            return Err("Supply two switches with VLAN 1, a new endpoint, its peer and two legacy endpoints.".into());
        }
        let (desk_switch, _, installed_vlan) = access_jack(&self.fabric, &self.desk)?;
        let (peer_switch, _, peer_vlan) = access_jack(&self.fabric, &self.desk_peer)?;
        if desk_switch == peer_switch
            || installed_vlan != self.initial.effective()
            || peer_vlan == DEFAULT_ACCESS
        {
            return Err("The desk must use the supplied starter membership and have a peer in another VLAN on the other switch.".into());
        }
        let Some((left, right)) = self.fabric.cables.iter().find_map(|c| match (&c.a, &c.b) {
            (
                Jack::Switch {
                    switch: a,
                    port: ap,
                },
                Jack::Switch {
                    switch: b,
                    port: bp,
                },
            ) => Some((
                &self.fabric.switches[a].ports[ap].mode,
                &self.fabric.switches[b].ports[bp].mode,
            )),
            _ => None,
        }) else {
            return Err("Supply one inter-switch trunk.".into());
        };
        if !matches!((left,right),(Mode::Trunk{ native: a, ..},Mode::Trunk{native: b, ..}) if a == b && *a != DEFAULT_ACCESS)
        {
            return Err("This exercise uses a matched trunk native VLAN distinct from default access VLAN 1.".into());
        }
        Ok(())
    }

    pub fn available(&self) -> Result<&std::collections::BTreeSet<u16>> {
        let (switch, _, _) = access_jack(&self.fabric, &self.desk)?;
        Ok(&self.fabric.switches[switch].vlans)
    }

    pub fn configured(&self, choice: Choice) -> Result<Fabric> {
        self.validate()?;
        let (switch, port, _) = access_jack(&self.fabric, &self.desk)?;
        if !self.available()?.contains(&choice.effective()) {
            return Err("Select a VLAN already present on the desk's switch.".into());
        }
        let mut fabric = self.fabric.clone();
        fabric
            .switches
            .get_mut(switch)
            .unwrap()
            .ports
            .get_mut(port)
            .unwrap()
            .mode = Mode::Access {
            vlan: choice.effective(),
        };
        Ok(fabric)
    }

    pub fn outcomes(&self, choice: Choice) -> Result<Vec<Outcome>> {
        let f = self.configured(choice)?;
        let spec = [
            (
                "New desk reaches its workgroup",
                &self.desk,
                &self.desk_peer,
                true,
            ),
            (
                "Legacy devices still share their segment",
                &self.legacy[0],
                &self.legacy[1],
                true,
            ),
            (
                "New desk is separate from the first legacy device",
                &self.desk,
                &self.legacy[0],
                false,
            ),
            (
                "New desk is separate from the second legacy device",
                &self.desk,
                &self.legacy[1],
                false,
            ),
        ];
        spec.into_iter().map(|(name, a, b, wanted)| {
            let value = f.visibility(a,b)?;
            let live = f.live_path(a,b)?;
            let passed = live && if wanted { value.mutual() } else { value.isolated() };
            let detail = if !live {
                "The physical path must remain available for this test.".into()
            } else if value.mutual() {
                format!("{a} and {b} received each other's Ethernet broadcasts.")
            } else if !value.isolated() {
                "Only one direction reached its peer. Compare port membership on both switches.".into()
            } else {
                format!("Neither Ethernet broadcast reached its peer. The desk currently uses VLAN {}.", choice.effective())
            };
            Ok(Outcome { name:name.into(), passed, detail })
        }).collect()
    }
}
