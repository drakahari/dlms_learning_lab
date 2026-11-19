//! Compose existing Ethernet visibility and static IPv4 forwarding. No I/O.
use crate::{ipv4_routing as ip, switching as ethernet};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub endpoint: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub device: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub interface: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub fabric: ethernet::Fabric,
    pub network: ip::Topology,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Attachment,8>")]
    pub attachments: Vec<Attachment>,
}

fn root(parents: &[usize], mut index: usize) -> usize {
    while parents[index] != index {
        index = parents[index];
    }
    index
}

impl Facts {
    pub fn validate(&self) -> Result<(), String> {
        self.fabric.validate()?;
        if !self.network.valid()
            || !(2..=8).contains(&self.attachments.len())
            || self.network.devices.len() > 6
            || self.fabric.switches.len() > 3
            || self.fabric.hosts.len() != self.attachments.len()
            || self.fabric.cables.len() > 8
            || self.fabric.hosts.values().any(|host| !host.up)
            || self.fabric.switches.values().any(|switch| {
                switch.vlans != BTreeSet::from([1])
                    || switch.ports.len() > 6
                    || switch
                        .ports
                        .values()
                        .any(|port| !port.up || port.mode != (ethernet::Mode::Access { vlan: 1 }))
            })
            || self.network.devices.values().any(|device| {
                device.interfaces.len() > 2
                    || device.interfaces.values().any(|interface| {
                        !interface.up
                            || interface
                                .address
                                .as_ref()
                                .is_none_or(|address| !address.usable())
                    })
            })
        {
            return Err(
                "Supply a small enabled untagged VLAN1 fabric and fixed ordinary IPv4 interfaces."
                    .into(),
            );
        }
        let endpoints: BTreeSet<_> = self.attachments.iter().map(|item| &item.endpoint).collect();
        let ports: BTreeSet<_> = self
            .attachments
            .iter()
            .map(|item| (&item.device, &item.interface))
            .collect();
        let actual_ports: BTreeSet<_> = self
            .network
            .devices
            .iter()
            .flat_map(|(device, value)| {
                value
                    .interfaces
                    .keys()
                    .map(move |interface| (device, interface))
            })
            .collect();
        if endpoints.len() != self.attachments.len()
            || ports.len() != self.attachments.len()
            || endpoints != self.fabric.hosts.keys().collect()
            || ports != actual_ports
        {
            return Err(
                "Bind every IPv4 interface to one distinct supplied Ethernet endpoint.".into(),
            );
        }
        Ok(())
    }

    /// Router interfaces remain distinct Ethernet endpoints. Only the existing
    /// IPv4 model can forward between them.
    pub fn project(
        &self,
        cables: &[ethernet::Cable],
    ) -> Result<(ethernet::Fabric, ip::Topology), String> {
        self.validate()?;
        if cables.len() != self.fabric.cables.len()
            || cables
                .iter()
                .zip(&self.fabric.cables)
                .any(|(proposed, original)| proposed.id != original.id)
        {
            return Err("Keep the supplied cable identities and equipment.".into());
        }
        let mut fabric = self.fabric.clone();
        fabric.cables = cables.to_vec();
        fabric.validate()?;
        let mut parents: Vec<_> = (0..self.attachments.len()).collect();
        for (index, a) in self.attachments.iter().enumerate() {
            for (other, b) in self.attachments.iter().enumerate().skip(index + 1) {
                let visibility = fabric.visibility(&a.endpoint, &b.endpoint)?;
                if visibility.forward != visibility.reverse {
                    return Err(
                        "Directional Ethernet visibility is outside this untagged exercise.".into(),
                    );
                }
                if visibility.mutual() {
                    let first = root(&parents, index);
                    let second = root(&parents, other);
                    parents[second] = first;
                }
            }
        }
        let mut network = self.network.clone();
        for (index, attachment) in self.attachments.iter().enumerate() {
            let interface = network
                .devices
                .get_mut(&attachment.device)
                .and_then(|device| device.interfaces.get_mut(&attachment.interface))
                .ok_or("A supplied interface binding no longer matches the equipment.")?;
            interface.link = format!("wire-lan-{}", root(&parents, index));
        }
        if !network.valid() {
            return Err("The projected IPv4 topology is invalid.".into());
        }
        Ok((fabric, network))
    }
}
