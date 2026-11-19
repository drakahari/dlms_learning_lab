//! One represented physical router, separate tables.
//! Projection reuses actual bounded IPv4 forwarding; no packets or host API.
use crate::ipv4_routing as ip;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
/// Derived routing-instance ID -> physical router and context, never extra equipment.
pub type RoutingInstances = BTreeMap<String, (String, String)>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub id: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub router: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Context,3>")]
    pub contexts: Vec<Context>,
    pub network: ip::Topology,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub interface: String,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub context: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Binding,6>")]
    pub bindings: Vec<Binding>,
    pub router_up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub requested: String,
    pub address: std::net::Ipv4Addr,
    pub actual: String,
    pub reached_requested: bool,
    pub probe: ip::Probe,
    /// Projected routing instances are not additional physical routers.
    pub routing_instances: RoutingInstances,
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 24
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Facts {
    pub fn validate(&self) -> Result<(), String> {
        if !self.network.valid()
            || !(3..=7).contains(&self.network.devices.len())
            || !(2..=3).contains(&self.contexts.len())
            || !self.network.devices.contains_key(&self.router)
        {
            return Err(
                "Supply one router, two or three contexts and a small fixed endpoint topology."
                    .into(),
            );
        }
        let mut seen = BTreeSet::new();
        for c in &self.contexts {
            if !name(&c.id) || !seen.insert(&c.id) {
                return Err("Routing contexts must have distinct bounded names.".into());
            }
        }
        for i in 0..3 {
            if self.network.devices.contains_key(&format!("vrf-{i}")) {
                return Err("The internal routing-instance names are reserved.".into());
            }
        }
        let r = &self.network.devices[&self.router];
        if !r.forwarding || !(2..=6).contains(&r.interfaces.len()) || !r.routes.is_empty() {
            return Err("This introductory VRF model uses two to six router interfaces and connected routes only.".into());
        }
        let mut links = BTreeSet::new();
        for interface in r.interfaces.values() {
            if !interface.up
                || interface.address.as_ref().is_none_or(|a| !a.usable())
                || !links.insert(&interface.link)
            {
                return Err("Router interfaces need distinct fixed links and usable source-owned addresses.".into());
            }
            let peers: Vec<_> = self
                .network
                .devices
                .iter()
                .filter(|(id, _)| *id != &self.router)
                .flat_map(|(id, d)| {
                    d.interfaces
                        .values()
                        .filter(|i| i.link == interface.link)
                        .map(move |i| (id, i))
                })
                .collect();
            if peers.len() != 1 || peers[0].1.address.as_ref().is_none_or(|a| !a.usable()) {
                return Err(
                    "Each supplied router link needs one distinct ordinary endpoint.".into(),
                );
            }
        }
        for (id, d) in &self.network.devices {
            if id != &self.router
                && (d.forwarding
                    || d.interfaces.len() != 1
                    || d.interfaces
                        .values()
                        .any(|i| !i.up || !links.contains(&i.link)))
            {
                return Err("Every non-router endpoint needs one enabled interface on one supplied router link.".into());
            }
        }
        Ok(())
    }
    pub fn validate_configuration(&self, c: &Configuration) -> Result<(), String> {
        self.validate()?;
        let r = &self.network.devices[&self.router];
        if c.bindings.len() != r.interfaces.len() {
            return Err("Assign every supplied router interface to one routing context.".into());
        }
        let mut seen = BTreeSet::new();
        for b in &c.bindings {
            if !r.interfaces.contains_key(&b.interface)
                || !self.contexts.iter().any(|v| v.id == b.context)
                || !seen.insert(&b.interface)
            {
                return Err("An interface needs one known context; duplicates or unknown interface names are not supported.".into());
            }
        }
        Ok(())
    }
    pub fn project(&self, c: &Configuration) -> Result<(ip::Topology, RoutingInstances), String> {
        self.validate_configuration(c)?;
        let mut network = self.network.clone();
        let router = network.devices.remove(&self.router).unwrap();
        let mut mapping = BTreeMap::new();
        for (index, context) in self.contexts.iter().enumerate() {
            let mut interfaces = BTreeMap::new();
            for b in c.bindings.iter().filter(|b| b.context == context.id) {
                let mut i = router.interfaces[&b.interface].clone();
                i.up &= c.router_up;
                interfaces.insert(b.interface.clone(), i);
            }
            if interfaces.is_empty() {
                continue;
            }
            // An overlap inside a context is an invalid plan, not isolated tenants.
            // The exact same addresses in DIFFERENT contexts are allowed.
            let links: BTreeSet<_> = interfaces.values().map(|i| &i.link).collect();
            let mut addresses = BTreeSet::new();
            for i in interfaces.values().chain(
                network
                    .devices
                    .values()
                    .flat_map(|d| d.interfaces.values())
                    .filter(|i| links.contains(&i.link)),
            ) {
                if let Some(a) = &i.address
                    && !addresses.insert(a.address)
                {
                    return Err(format!(
                        "{} contains duplicate address {}. Put the two tenants in separate routing contexts.",
                        context.id, a.address
                    ));
                }
            }
            let id = format!("vrf-{index}");
            mapping.insert(id.clone(), (self.router.clone(), context.id.clone()));
            network.devices.insert(
                id,
                ip::Device {
                    forwarding: true,
                    interfaces,
                    routes: Vec::new(),
                },
            );
        }
        if !network.valid() {
            return Err(
                "The projected context graph exceeds the existing static-forwarding limits.".into(),
            );
        }
        Ok((network, mapping))
    }
    pub fn probe(
        &self,
        c: &Configuration,
        from: &str,
        target: &str,
    ) -> Result<Observation, String> {
        if from == self.router
            || target == self.router
            || from == target
            || !self.network.devices.contains_key(from)
        {
            return Err("Choose two different supplied host endpoints.".into());
        }
        let target_device = self
            .network
            .devices
            .get(target)
            .ok_or("Unknown intended endpoint.")?;
        let address = target_device
            .interfaces
            .values()
            .next()
            .and_then(|i| i.address.as_ref())
            .filter(|a| a.usable())
            .ok_or("Intended endpoint has no usable source-owned address.")?
            .address;
        let (network, mapping) = self.project(c)?;
        if !c.router_up {
            return Err(format!(
                "{} is unavailable. Restore the shared router before checking required paths.",
                self.router
            ));
        }
        let probe = network.probe(from, address)?;
        let actual = probe.forward.destination_device.clone();
        Ok(Observation {
            requested: target.into(),
            address,
            reached_requested: actual == target,
            actual,
            probe,
            routing_instances: mapping,
        })
    }
}
