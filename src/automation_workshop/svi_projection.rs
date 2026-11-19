//! Place IPv4 interfaces on actual Ethernet broadcast components for a fixed two-switch lab.
use super::access_defaults::access_jack;
use crate::{
    ipv4_routing::{Assignment, Device, Interface, Prefix, Route, Topology},
    switching::{Cable, Fabric, Host, Jack, Mode, Port},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};

type Result<T> = std::result::Result<T, String>;
const UPLINK: &str = "uplink";
const ROUTER: &str = "gateway";
const HOST_IF: &str = "lan";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub id: String,
    pub address: Assignment,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Svi {
    pub vlan: u16,
    pub subnet: Prefix,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub fabric: Fabric,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub router_switch: String,
    pub endpoints: [Endpoint; 3],
    pub interfaces: [Svi; 2],
    pub initial: Edit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SviConfig {
    pub enabled: bool,
    pub address: Option<Assignment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    pub routing: bool,
    pub interfaces: [SviConfig; 2],
    pub gateways: [Option<Ipv4Addr>; 3],
    pub allow_remote: [bool; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    Shutdown,
    NoForwardingMembers,
    Up,
}

impl Availability {
    pub fn text(&self) -> &'static str {
        match self {
            Self::Shutdown => "Administratively down",
            Self::NoForwardingMembers => "No active local member",
            Self::Up => "Up",
        }
    }
}

fn name(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 24
        && !v.starts_with("svi-")
        && v != ROUTER
        && v.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

fn internal(vlan: u16) -> String {
    format!("svi-{vlan}")
}

fn host_key(id: &str) -> Jack {
    Jack::Host { host: id.into() }
}

impl Edit {
    pub fn valid(&self) -> bool {
        self.interfaces
            .iter()
            .all(|s| s.address.as_ref().is_none_or(Assignment::valid))
            && self
                .gateways
                .iter()
                .all(|gateway| gateway.is_none_or(crate::ipv4_routing::supported_address))
    }
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        self.fabric.validate()?;
        let any_bad_key = self
            .fabric
            .hosts
            .keys()
            .chain(self.fabric.switches.keys())
            .any(|key| !name(key));
        if self.fabric.switches.len() != 2
            || self.fabric.hosts.len() != 3
            || self.fabric.cables.len() != 4
            || any_bad_key
            || !self.fabric.switches.contains_key(&self.router_switch)
            || !self.initial.valid()
        {
            return Err(
                "Supply two switches, three named stations, four cables and a supported starter."
                    .into(),
            );
        }
        let [a, b] = &self.interfaces;
        let vlan_ids = [a.vlan, b.vlan];
        if a.vlan == b.vlan
            || vlan_ids.iter().any(|v| !(2..=1001).contains(v))
            || !a.subnet.valid()
            || !b.subnet.valid()
            || a.subnet.contains(b.subnet.address)
            || b.subnet.contains(a.subnet.address)
            || self
                .fabric
                .switches
                .values()
                .any(|s| vlan_ids.iter().any(|v| !s.vlans.contains(v)))
            || self.fabric.switches[&self.router_switch].ports.len() > 14
        {
            return Err("Use two known normal-range VLANs, nonoverlapping subnets and a switch with space for both interfaces.".into());
        }
        let mut ids = BTreeSet::new();
        let mut addresses = BTreeSet::new();
        for (index, ep) in self.endpoints.iter().enumerate() {
            let (_, _, vlan) = access_jack(&self.fabric, &ep.id)?;
            let provisioned = if index < 2 { a } else { b };
            if !name(&ep.id)
                || !ids.insert(&ep.id)
                || !addresses.insert(ep.address.address)
                || vlan != provisioned.vlan
                || !ep.address.usable()
                || Prefix::new(ep.address.address, ep.address.prefix) != Some(provisioned.subnet)
            {
                return Err("The first two named stations must share one address plan. Supply the third station in the other VLAN and IPv4 subnet.".into());
            }
        }
        if self
            .fabric
            .switches
            .values()
            .any(|device| device.ports.keys().any(|p| p.starts_with("svi-")))
            || self.fabric.cables.iter().any(|c| c.id.starts_with("svi-"))
        {
            return Err("Reserve the internal SVI interface names for the simulator.".into());
        }
        let trunk = self.trunk()?;
        let first = &self.fabric.switches[trunk.0].ports[UPLINK].mode;
        let remote = &self.fabric.switches[trunk.1].ports[UPLINK].mode;
        if !matches!((first,remote),(Mode::Trunk{native:a,allowed:first},Mode::Trunk{native:b,allowed:second}) if a == b && !vlan_ids.contains(a) && first == second && vlan_ids.into_iter().all(|v| first.contains(&v)))
        {
            return Err("Supply matched, explicitly configured trunks with a separate native VLAN and both data VLANs carried.".into());
        }
        Ok(())
    }

    fn trunk(&self) -> Result<(&str, &str)> {
        for cable in &self.fabric.cables {
            for (a, b) in [(&cable.a, &cable.b), (&cable.b, &cable.a)] {
                if let (
                    Jack::Switch {
                        switch: a,
                        port: ap,
                    },
                    Jack::Switch {
                        switch: b,
                        port: bp,
                    },
                ) = (a, b)
                    && a == &self.router_switch
                    && ap == UPLINK
                    && bp == UPLINK
                {
                    return Ok((a, b));
                }
            }
        }
        Err("Provide the named uplink at each switch.".into())
    }

    fn configured(&self, edit: &Edit) -> Result<Fabric> {
        self.validate()?;
        if !edit.valid() {
            return Err("Use valid IPv4 host fields from the supplied address plan.".into());
        }
        let mut fabric = self.fabric.clone();
        let (_, remote) = self.trunk()?;
        let port = fabric
            .switches
            .get_mut(remote)
            .unwrap()
            .ports
            .get_mut(UPLINK)
            .unwrap();
        if let Mode::Trunk { allowed, .. } = &mut port.mode {
            for (svi, allowed_remote) in self.interfaces.iter().zip(edit.allow_remote) {
                if !allowed_remote {
                    allowed.remove(&svi.vlan);
                }
            }
        }
        Ok(fabric)
    }

    fn available(&self, f: &Fabric, edit: &Edit, index: usize) -> Availability {
        if !edit.interfaces[index].enabled {
            return Availability::Shutdown;
        }
        let vlan = self.interfaces[index].vlan;
        let router = &f.switches[&self.router_switch];
        for cable in f.cables.iter().filter(|c| c.up) {
            for (node, peer) in [(&cable.a, &cable.b), (&cable.b, &cable.a)] {
                let Jack::Switch { switch, port } = node else {
                    continue;
                };
                if switch != &self.router_switch {
                    continue;
                }
                let local = &router.ports[port];
                let peer_up = match peer {
                    Jack::Host { host } => f.hosts[host].up,
                    Jack::Switch { switch, port } => f.switches[switch].ports[port].up,
                };
                let carries = match &local.mode {
                    Mode::Access { vlan: bound } => *bound == vlan,
                    Mode::Trunk { allowed, .. } => allowed.contains(&vlan),
                };
                if local.up && peer_up && carries {
                    return Availability::Up;
                }
            }
        }
        Availability::NoForwardingMembers
    }

    pub fn availability(&self, edit: &Edit) -> Result<[Availability; 2]> {
        let f = self.configured(edit)?;
        Ok(std::array::from_fn(|i| self.available(&f, edit, i)))
    }

    fn with_interfaces(&self, edit: &Edit) -> Result<Fabric> {
        let mut fabric = self.configured(edit)?;
        for (index, svi) in self.interfaces.iter().enumerate() {
            let id = internal(svi.vlan);
            let up = self.available(&fabric, edit, index) == Availability::Up;
            fabric.hosts.insert(
                id.clone(),
                Host {
                    label: format!("VLAN {} switch interface", svi.vlan),
                    up,
                },
            );
            fabric
                .switches
                .get_mut(&self.router_switch)
                .unwrap()
                .ports
                .insert(
                    id.clone(),
                    Port {
                        up,
                        mode: Mode::Access { vlan: svi.vlan },
                    },
                );
            fabric.cables.push(Cable {
                id: id.clone(),
                a: host_key(&id),
                b: Jack::Switch {
                    switch: self.router_switch.clone(),
                    port: id,
                },
                up,
            });
        }
        Ok(fabric)
    }

    pub fn ethernet(
        &self,
        edit: &Edit,
        from: usize,
        to: usize,
    ) -> Result<crate::switching::Visibility> {
        let from = self
            .endpoints
            .get(from)
            .ok_or("Choose a supplied source station.")?;
        let to = self
            .endpoints
            .get(to)
            .ok_or("Choose a supplied destination station.")?;
        self.configured(edit)?.visibility(&from.id, &to.id)
    }

    pub fn routed_topology(&self, edit: &Edit) -> Result<Topology> {
        let f = self.with_interfaces(edit)?;
        let mut component: BTreeMap<&str, String> = BTreeMap::new();
        // Equal native IDs, no tagged endpoints and a fixed forest make the
        // mutual Layer-2 visibility relation symmetric and transitive here.
        for id in f.hosts.keys() {
            if component.contains_key(id.as_str()) {
                continue;
            }
            let link = format!("lan-{}", component.len());
            let peers = f.broadcast(id)?;
            component.insert(id, link.clone());
            for peer in peers.recipients.keys() {
                if let Some(other) = component.get(peer.as_str()) {
                    if other != &link {
                        return Err("Unsupported overlapping broadcast paths.".into());
                    }
                } else {
                    let (held, _) = f
                        .hosts
                        .get_key_value(peer)
                        .ok_or("Unknown derived broadcast endpoint.")?;
                    component.insert(held, link.clone());
                }
            }
        }
        let router = Device {
            forwarding: edit.routing,
            routes: vec![],
            interfaces: self
                .interfaces
                .iter()
                .zip(&edit.interfaces)
                .map(|(svi, config)| {
                    let id = internal(svi.vlan);
                    (
                        id.clone(),
                        Interface {
                            link: component[id.as_str()].clone(),
                            up: f.hosts[&id].up,
                            address: config.address.clone(),
                        },
                    )
                })
                .collect(),
        };
        let mut devices = BTreeMap::from([(ROUTER.into(), router)]);
        for (endpoint, gateway) in self.endpoints.iter().zip(edit.gateways) {
            let lan = Interface {
                link: component[endpoint.id.as_str()].clone(),
                up: f.hosts[&endpoint.id].up,
                address: Some(endpoint.address.clone()),
            };
            let routes = gateway
                .into_iter()
                .map(|via| Route {
                    destination: Prefix {
                        address: Ipv4Addr::UNSPECIFIED,
                        length: 0,
                    },
                    interface: HOST_IF.into(),
                    via,
                    distance: 1,
                })
                .collect();
            devices.insert(
                endpoint.id.clone(),
                Device {
                    forwarding: false,
                    interfaces: BTreeMap::from([(HOST_IF.into(), lan)]),
                    routes,
                },
            );
        }
        Ok(Topology { devices })
    }

    pub fn probe(&self, edit: &Edit, from: usize, to: usize) -> Result<crate::ipv4_routing::Probe> {
        if from == to {
            return Err("Choose different supplied endpoints.".into());
        }
        let from = self
            .endpoints
            .get(from)
            .ok_or("Choose a supplied source station.")?;
        let to = self
            .endpoints
            .get(to)
            .ok_or("Choose a supplied destination station.")?;
        self.routed_topology(edit)?
            .verify_endpoint(&from.id, &to.id, HOST_IF)
    }
}

impl Task {
    pub fn outcomes(&self, edit: &Edit) -> Result<Vec<super::workspace::Outcome>> {
        use super::workspace::Outcome;
        let f = self.configured(edit)?;
        let t = self.routed_topology(edit)?;
        let mut outcomes = Vec::new();
        for (i, svi) in self.interfaces.iter().enumerate() {
            let state = self.available(&f, edit, i);
            let address = edit.interfaces[i].address.as_ref();
            let distinct = address.is_some_and(|a| {
                self.endpoints
                    .iter()
                    .all(|e| e.address.address != a.address)
            });
            let correct = address.is_some_and(|a| {
                a.usable() && Prefix::new(a.address, a.prefix) == Some(svi.subnet)
            });
            let passed = state == Availability::Up && correct && distinct;
            let detail = if state != Availability::Up {
                format!(
                    "{}; the gateway cannot be reached while this VLAN interface is down.",
                    state.text()
                )
            } else if !correct {
                format!(
                    "Use a usable host address in {} with that subnet's prefix.",
                    svi.subnet.text()
                )
            } else if !distinct {
                "This address is already assigned to a station. Use a different host address for the gateway.".into()
            } else {
                format!(
                    "{} is on the supplied subnet, and the interface is up.",
                    address.unwrap().address
                )
            };
            outcomes.push(Outcome {
                name: format!("VLAN {} gateway interface", svi.vlan),
                passed,
                detail,
            });
        }
        let pair = f.visibility(&self.endpoints[0].id, &self.endpoints[1].id)?;
        outcomes.push(Outcome {
            name: "The two workgroup stations still share Ethernet".into(),
            passed: pair.mutual(),
            detail: if pair.mutual() {
                "Both directions work without IP routing."
            } else {
                "Compare the allowed VLAN on the remote uplink with the workgroup's access VLAN."
            }
            .into(),
        });
        let pair = f.visibility(&self.endpoints[0].id, &self.endpoints[2].id)?;
        outcomes.push(Outcome {
            name: "The two VLANs remain separate Ethernet segments".into(),
            passed: pair.isolated() && f.live_path(&self.endpoints[0].id, &self.endpoints[2].id)?,
            detail: if pair.isolated() {
                "The Ethernet broadcasts are separate. Use the IPv4 check to test routed traffic."
            } else {
                "A frame crossed the intended VLAN boundary. Inspect the supplied membership."
            }
            .into(),
        });
        for i in [0, 1] {
            let from = &self.endpoints[i].id;
            let to = &self.endpoints[2].id;
            let result = t.verify_endpoint(from, to, HOST_IF);
            outcomes.push(Outcome {
                name: format!("{from} exchanges IPv4 traffic with {to}"),
                passed: result.is_ok(),
                detail: match result {
                    Ok(p) => format!(
                        "Delivery and reply both worked, with {} outbound and {} return hops.",
                        p.forward.hops.len(),
                        p.reply.hops.len()
                    ),
                    Err(e) => e,
                },
            });
        }
        Ok(outcomes)
    }
}
