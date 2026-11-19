//! Controller intent, installed rules and an actual virtual IPv4 transport path.
use crate::ipv4_routing::{Probe, Topology};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub carrier_device: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub carrier_port: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub device: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub port: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exchange {
    pub from: usize,
    pub to: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub transport: Topology,
    pub edges: [Edge; 3],
    pub repairable: Link,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Current {
    // Rows and columns share the immutable edge ordering in Spec.
    pub controller: [[bool; 3]; 3],
    pub installed: [[bool; 3]; 3],
    pub uplink_up: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub desired: bool,
    pub installed: bool,
    pub carrier: Result<Probe, String>,
}

impl Trace {
    pub fn forwarded(&self) -> bool {
        self.installed && self.carrier.is_ok()
    }
}

impl Spec {
    fn endpoint(&self, index: usize) -> Result<(&Edge, Ipv4Addr), String> {
        let edge = self
            .edges
            .get(index)
            .ok_or("Choose a supplied fabric edge.")?;
        let address = self
            .transport
            .devices
            .get(&edge.carrier_device)
            .and_then(|d| d.interfaces.get(&edge.carrier_port))
            .and_then(|iface| iface.address.as_ref())
            .filter(|a| a.usable())
            .map(|a| a.address)
            .ok_or("A supplied fabric edge needs a usable transport address.")?;
        Ok((edge, address))
    }

    fn with_uplink(&self, up: bool) -> Result<Topology, String> {
        let mut world = self.transport.clone();
        let port = world
            .devices
            .get_mut(&self.repairable.device)
            .and_then(|d| d.interfaces.get_mut(&self.repairable.port))
            .ok_or("Select the supplied uplink.")?;
        port.up = up;
        if !world.valid() {
            return Err(
                "The supplied virtual transport has invalid address or route settings.".into(),
            );
        }
        Ok(world)
    }

    pub fn validate(&self) -> Result<(), String> {
        use std::collections::BTreeSet;
        let mut seen_names = BTreeSet::new();
        let mut seen_devices = BTreeSet::new();
        for index in 0..self.edges.len() {
            let (edge, _) = self.endpoint(index)?;
            if edge.name.trim().is_empty()
                || edge.name.chars().any(char::is_control)
                || !seen_names.insert(&edge.name)
                || !seen_devices.insert(&edge.carrier_device)
            {
                return Err(
                    "Use three distinct labeled edges with separate transport devices.".into(),
                );
            }
        }
        self.with_uplink(true)?;
        self.with_uplink(false)?;
        Ok(())
    }

    pub fn trace(&self, current: Current, trial: Exchange) -> Result<Trace, String> {
        let (from, _) = self.endpoint(trial.from)?;
        let (to, address) = self.endpoint(trial.to)?;
        if from == to {
            return Err("Choose two separate fabric edges for this exchange.".into());
        }
        let world = self.with_uplink(current.uplink_up)?;
        let carrier = world
            .probe(&from.carrier_device, address)
            .and_then(|probe| {
                if probe.forward.destination_device != to.carrier_device {
                    Err(
                        "The path reached a different device. Inspect the supplied transport."
                            .into(),
                    )
                } else {
                    Ok(probe)
                }
            });
        Ok(Trace {
            desired: current.controller[trial.from][trial.to],
            installed: current.installed[trial.from][trial.to],
            carrier,
        })
    }
}

impl Current {
    pub fn submit(&mut self, request: Exchange, allow: bool) -> Result<(), String> {
        if request.from >= self.controller.len()
            || request.to >= self.controller.len()
            || request.from == request.to
        {
            return Err("Select two distinct supplied fabric edges.".into());
        }
        self.controller[request.from][request.to] = allow;
        Ok(())
    }

    pub fn publish(&mut self) {
        self.installed = self.controller;
    }
}
