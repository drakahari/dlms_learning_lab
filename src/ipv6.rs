//! Pure bounded IPv6 addressing and deterministic static forwarding; no network I/O.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv6Addr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Unspecified,
    Loopback,
    Ipv4Mapped,
    Multicast,
    LinkLocal,
    UniqueLocal,
    Documentation,
    OtherUnicast,
}
pub fn category(address: Ipv6Addr) -> Category {
    let n = u128::from(address);
    if address.is_unspecified() {
        Category::Unspecified
    } else if address.is_loopback() {
        Category::Loopback
    } else if address.to_ipv4_mapped().is_some() {
        Category::Ipv4Mapped
    } else if n >> 120 == 0xff {
        Category::Multicast
    } else if n >> 118 == 0b1111111010 {
        Category::LinkLocal
    } else if n >> 121 == 0b1111110 {
        Category::UniqueLocal
    } else if n >> 96 == 0x20010db8 || n >> 108 == 0x3fff0 {
        Category::Documentation
    } else {
        Category::OtherUnicast
    }
}
pub fn mask(prefix: u8) -> Option<u128> {
    match prefix {
        0 => Some(0),
        1..=128 => Some(u128::MAX << (128 - prefix)),
        _ => None,
    }
}
pub fn network(address: Ipv6Addr, prefix: u8) -> Option<Ipv6Addr> {
    Some((u128::from(address) & mask(prefix)?).into())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub address: Ipv6Addr,
    pub length: u8,
}
impl Prefix {
    pub fn new(address: Ipv6Addr, length: u8) -> Option<Self> {
        Some(Self {
            address: network(address, length)?,
            length,
        })
    }
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 64 {
            return Err("IPv6 prefix is too long".into());
        }
        let (address, length) = text
            .split_once('/')
            .ok_or("Use an IPv6 network/prefix, such as 2001:db8:10::/64")?;
        let address: Ipv6Addr = address.parse().map_err(|_| "Malformed IPv6 address")?;
        if length.is_empty() || !length.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Prefix must be a whole number from 0 to128".into());
        }
        let length = length
            .parse()
            .map_err(|_| "Prefix must be a whole number from 0 to128")?;
        let prefix = Self::new(address, length).ok_or("Prefix must be from 0 to128")?;
        if prefix.address != address {
            return Err(format!(
                "This address has nonzero bits after the prefix. Use the network prefix {}.",
                prefix.text()
            ));
        }
        Ok(prefix)
    }
    pub fn valid(&self) -> bool {
        network(self.address, self.length) == Some(self.address)
    }
    pub fn contains(&self, address: Ipv6Addr) -> bool {
        self.valid() && network(address, self.length) == Some(self.address)
    }
    pub fn text(&self) -> String {
        format!("{}/{}", self.address, self.length)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub address: Ipv6Addr,
    pub prefix: u8,
    /// Explicit manual on-link declaration; an address prefix alone is not an RA on-link flag.
    pub on_link: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub link: String,
    pub up: bool,
    pub link_local: Ipv6Addr,
    pub address: Option<Assignment>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: Prefix,
    pub interface: String,
    pub via: Option<Ipv6Addr>,
    pub distance: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub forwarding: bool,
    pub interfaces: BTreeMap<String, Interface>,
    pub routes: Vec<Route>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    pub devices: BTreeMap<String, Device>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hop {
    pub device: String,
    pub interface: String,
    pub next_hop: Ipv6Addr,
    pub peer: String,
    pub peer_interface: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub hops: Vec<Hop>,
    pub destination_device: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ping {
    pub source: Ipv6Addr,
    pub forward: Trace,
    pub reply: Trace,
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn supported_unicast(address: Ipv6Addr) -> bool {
    matches!(category(address), Category::Documentation) || (u128::from(address) >> 120 == 0xfd)
}
fn supported_local(address: Ipv6Addr) -> bool {
    u128::from(address) >> 64 == 0xfe80000000000000 && u128::from(address) as u64 != 0
}
/// Known subnet-router and RFC2526 reservations for this unit's conventional /64s.
/// This does not identify arbitrary configured anycast from address bits.
pub fn reserved_workstation_id(address: Ipv6Addr) -> bool {
    let id = u128::from(address) as u64;
    id == 0 || id & !0x7f == 0xfdff_ffff_ffff_ff80
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteStatus {
    Active,
    InterfaceDown,
    Distance255,
    NextHopOffLink,
}
impl RouteStatus {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::Active => "Active in this static model",
            Self::InterfaceDown => "Outgoing interface is down",
            Self::Distance255 => "Distance255 prevents installation",
            Self::NextHopOffLink => {
                "Next hop is outside the explicitly on-link prefixes; recursive resolution is unsupported"
            }
        }
    }
}
fn route_status(node: &Device, route: &Route) -> RouteStatus {
    let iface = &node.interfaces[&route.interface];
    if !iface.up {
        RouteStatus::InterfaceDown
    } else if route.distance == 255 {
        RouteStatus::Distance255
    } else if route.via.is_some_and(|via| {
        category(via) != Category::LinkLocal
            && !iface.address.as_ref().is_some_and(|a| {
                a.on_link && Prefix::new(a.address, a.prefix).is_some_and(|p| p.contains(via))
            })
    }) {
        RouteStatus::NextHopOffLink
    } else {
        RouteStatus::Active
    }
}
#[derive(Clone, Debug)]
struct Selected {
    interface: String,
    via: Option<Ipv6Addr>,
    length: u8,
    distance: u8,
}
impl Topology {
    /// Configuration can change; authored devices, ports and physical links cannot.
    pub fn same_shape(&self, original: &Self) -> bool {
        self.devices.keys().eq(original.devices.keys())
            && self.devices.iter().all(|(id, device)| {
                let other = &original.devices[id];
                device.interfaces.keys().eq(other.interfaces.keys())
                    && device
                        .interfaces
                        .iter()
                        .all(|(port, iface)| iface.link == other.interfaces[port].link)
            })
    }
    pub fn valid(&self) -> bool {
        !self.devices.is_empty()
            && self.devices.len() <= 12
            && self.devices.iter().all(|(id, device)| {
                name(id)
                    && !device.interfaces.is_empty()
                    && device.interfaces.len() <= 8
                    && device.routes.len() <= 32
                    && device.interfaces.iter().all(|(id, iface)| {
                        name(id)
                            && name(&iface.link)
                            && supported_local(iface.link_local)
                            && iface
                                .address
                                .as_ref()
                                .is_none_or(|a| a.prefix <= 128 && supported_unicast(a.address))
                    })
                    && device.routes.iter().all(|r| {
                        r.destination.valid()
                            && device.interfaces.contains_key(&r.interface)
                            && r.distance > 0
                            && r.via
                                .is_none_or(|a| supported_unicast(a) || supported_local(a))
                    })
            })
    }
    pub fn route_status(&self, device: &str, index: usize) -> Result<RouteStatus, String> {
        if !self.valid() {
            return Err("Invalid IPv6 topology".into());
        }
        let node = self.devices.get(device).ok_or("Unknown device")?;
        let route = node.routes.get(index).ok_or("Unknown route")?;
        Ok(route_status(node, route))
    }
    fn local(&self, device: &str, address: Ipv6Addr, scope: Option<&str>) -> bool {
        self.devices[device].interfaces.iter().any(|(id, i)| {
            i.up && scope.is_none_or(|s| s == id)
                && (i.address.as_ref().is_some_and(|a| a.address == address)
                    || i.link_local == address)
        })
    }
    fn select(
        &self,
        device: &str,
        destination: Ipv6Addr,
        scope: Option<&str>,
    ) -> Result<Selected, String> {
        let node = &self.devices[device];
        let mut routes = Vec::new();
        for (id, i) in &node.interfaces {
            if !i.up || scope.is_some_and(|s| s != id) {
                continue;
            }
            if category(destination) == Category::LinkLocal {
                if scope == Some(id.as_str()) {
                    routes.push(Selected {
                        interface: id.clone(),
                        via: None,
                        length: 64,
                        distance: 0,
                    });
                }
                continue;
            }
            if let Some(a) = &i.address
                && a.on_link
                && Prefix::new(a.address, a.prefix).is_some_and(|p| p.contains(destination))
            {
                routes.push(Selected {
                    interface: id.clone(),
                    via: None,
                    length: a.prefix,
                    distance: 0,
                });
            }
        }
        if category(destination) != Category::LinkLocal {
            for r in &node.routes {
                if scope.is_some_and(|s| s != r.interface)
                    || !r.destination.contains(destination)
                    || route_status(node, r) != RouteStatus::Active
                {
                    continue;
                }
                routes.push(Selected {
                    interface: r.interface.clone(),
                    via: r.via,
                    length: r.destination.length,
                    distance: r.distance,
                });
            }
        }
        routes.sort_by_key(|r| (std::cmp::Reverse(r.length), r.distance));
        let Some(selected) = routes.first() else {
            let reasons = node
                .routes
                .iter()
                .filter(|r| r.destination.contains(destination))
                .filter_map(|r| {
                    let status = route_status(node, r);
                    (status != RouteStatus::Active)
                        .then(|| format!("{}: {}", r.destination.text(), status.explanation()))
                })
                .take(3)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "{device}: no active route to {destination}. Inspect prefixes, outgoing interfaces and next-hop reachability. {reasons}"
            ));
        };
        if routes.iter().skip(1).any(|r| {
            r.length == selected.length
                && r.distance == selected.distance
                && (r.interface != selected.interface || r.via != selected.via)
        }) {
            return Err(format!(
                "{device}: equal-cost forwarding choices are unsupported in this model. Use one preferred route or different distances."
            ));
        }
        Ok(selected.clone())
    }
    fn neighbors(&self, device: &str, interface: &str, address: Ipv6Addr) -> Vec<(String, String)> {
        let link = &self.devices[device].interfaces[interface].link;
        self.devices
            .iter()
            .flat_map(|(id, d)| {
                d.interfaces
                    .iter()
                    .filter(|(_, i)| {
                        i.up && &i.link == link
                            && (i.link_local == address
                                || i.address.as_ref().is_some_and(|a| a.address == address))
                    })
                    .map(|(port, _)| (id.clone(), port.clone()))
            })
            .collect()
    }
    fn path(
        &self,
        start: &str,
        destination: Ipv6Addr,
        scope: Option<&str>,
    ) -> Result<Trace, String> {
        let mut current = start.to_owned();
        let mut hops: Vec<Hop> = Vec::new();
        let mut visited = BTreeSet::new();
        for _ in 0..16 {
            let local_scope = if hops.is_empty() {
                scope
            } else if category(destination) == Category::LinkLocal {
                hops.last().map(|h| h.peer_interface.as_str())
            } else {
                None
            };
            if self.local(&current, destination, local_scope) {
                // Same-link duplicates are broken configuration, not random delivery.
                for (port, i) in &self.devices[&current].interfaces {
                    if i.up
                        && local_scope.is_none_or(|s| s == port)
                        && (i.link_local == destination
                            || i.address.as_ref().is_some_and(|a| a.address == destination))
                        && self.neighbors(&current, port, destination).len() > 1
                    {
                        return Err(format!(
                            "{current}: duplicate address {destination} on link {}. Give each interface a distinct address on that link.",
                            i.link
                        ));
                    }
                }
                return Ok(Trace {
                    hops,
                    destination_device: current,
                });
            }
            if !visited.insert(current.clone()) {
                return Err(format!(
                    "Routing loop: {current} was visited again. Inspect the next hops on both routers."
                ));
            }
            if !hops.is_empty() && !self.devices[&current].forwarding {
                return Err(format!(
                    "{current}: forwarding is disabled; this device cannot carry traffic to another network."
                ));
            }
            if !hops.is_empty() && category(destination) == Category::LinkLocal {
                return Err("Link-local traffic cannot be forwarded to another link.".into());
            }
            let route = self.select(&current, destination, local_scope)?;
            let next_hop = route.via.unwrap_or(destination);
            let peers = self.neighbors(&current, &route.interface, next_hop);
            if peers.len() > 1 {
                return Err(format!(
                    "{current}: duplicate next-hop address {next_hop} on the selected link."
                ));
            }
            let Some((peer, peer_interface)) = peers.first() else {
                return Err(format!(
                    "{current}: no up neighbor owns {next_hop} on {}. Check the address, link and outgoing interface.",
                    route.interface
                ));
            };
            if peer == &current {
                return Err(format!(
                    "{current}: the next hop points back to this device; choose its neighboring router."
                ));
            }
            hops.push(Hop {
                device: current.clone(),
                interface: route.interface,
                next_hop,
                peer: peer.clone(),
                peer_interface: peer_interface.clone(),
            });
            current = peer.clone();
        }
        Err("Forwarding hop limit reached (16); check for a routing loop.".into())
    }
    /// Stateless echo verification: both paths are derived from current typed state.
    pub fn ping(
        &self,
        from: &str,
        destination: Ipv6Addr,
        scope: Option<&str>,
    ) -> Result<Ping, String> {
        if !self.valid() {
            return Err("Invalid IPv6 topology; no probe was simulated.".into());
        }
        let node = self.devices.get(from).ok_or("Unknown source device")?;
        if scope.is_some_and(|s| !node.interfaces.contains_key(s)) {
            return Err("Unknown outgoing interface".into());
        }
        let local = category(destination) == Category::LinkLocal;
        if local && scope.is_none() {
            return Err("Choose an outgoing interface for this link-local destination; the same address can exist on different links.".into());
        }
        if !(supported_unicast(destination) || supported_local(destination)) {
            return Err("Unsupported probe destination: use a modeled documentation/locally assigned ULA or conventional link-local unicast address.".into());
        }
        let source = if self.local(from, destination, scope) {
            destination
        } else {
            let selected = self.select(from, destination, scope)?;
            let iface = &node.interfaces[&selected.interface];
            if local {
                iface.link_local
            } else {
                iface
                    .address
                    .as_ref()
                    .map(|a| a.address)
                    .or_else(|| {
                        node.interfaces
                            .values()
                            .filter(|i| i.up)
                            .find_map(|i| i.address.as_ref().map(|a| a.address))
                    })
                    .ok_or("No up non-link-local source address is configured")?
            }
        };
        // Source duplicates also make the reply target ambiguous, even for self probes.
        let forward = self.path(from, destination, scope)?;
        let reply_scope = if local {
            forward
                .hops
                .last()
                .map(|h| h.peer_interface.as_str())
                .or(scope)
        } else {
            None
        };
        let reply = self
            .path(&forward.destination_device, source, reply_scope)
            .map_err(|e| format!("Forward delivery works, but the reply fails: {e}"))?;
        if reply.destination_device != from {
            return Err("Reply reached a different device using the source address; inspect duplicate assignments.".into());
        }
        Ok(Ping {
            source,
            forward,
            reply,
        })
    }
}

/// Declarative positive outcome checks. Conditional probes never mutate saved state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceCondition {
    pub device: String,
    pub interface: String,
    pub up: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    AddressIn {
        device: String,
        interface: String,
        network: Prefix,
        exclude: Vec<Ipv6Addr>,
        on_link: bool,
    },
    KeepAddress {
        device: String,
        interface: String,
        assignment: Option<Assignment>,
    },
    Forwarding {
        device: String,
        enabled: bool,
    },
    Reachable {
        label: String,
        from: String,
        target: Ipv6Addr,
        scope: Option<String>,
        #[serde(default)]
        first_hop: Option<String>,
        when: Vec<InterfaceCondition>,
    },
}
impl Requirement {
    pub fn valid_for(&self, t: &Topology) -> bool {
        let port = |d: &str, p: &str| t.devices.get(d).and_then(|d| d.interfaces.get(p));
        match self {
            Self::AddressIn {
                device,
                interface,
                network,
                exclude,
                ..
            } => {
                port(device, interface).is_some()
                    && network.valid()
                    && network.length == 64
                    && supported_unicast(network.address)
                    && exclude.len() <= 16
            }
            Self::KeepAddress {
                device,
                interface,
                assignment,
            } => {
                port(device, interface).is_some()
                    && assignment
                        .as_ref()
                        .is_none_or(|a| a.prefix <= 128 && supported_unicast(a.address))
            }
            Self::Forwarding { device, .. } => t.devices.contains_key(device),
            Self::Reachable {
                label,
                from,
                target,
                scope,
                first_hop,
                when,
            } => {
                !label.trim().is_empty()
                    && label.len() <= 160
                    && t.devices.contains_key(from)
                    && (supported_unicast(*target) || supported_local(*target))
                    && (category(*target) != Category::LinkLocal || scope.is_some())
                    && scope.as_ref().is_none_or(|s| port(from, s).is_some())
                    && first_hop.as_ref().is_none_or(|s| port(from, s).is_some())
                    && when.len() <= 4
                    && when.iter().all(|c| port(&c.device, &c.interface).is_some())
                    && when
                        .iter()
                        .map(|c| (&c.device, &c.interface))
                        .collect::<BTreeSet<_>>()
                        .len()
                        == when.len()
            }
        }
    }
    pub fn check(&self, t: &Topology) -> (bool, String) {
        if !t.valid() || !self.valid_for(t) {
            return (
                false,
                "Required IPv6 device, interface or supported probe is missing/invalid.".into(),
            );
        }
        match self {
            Self::AddressIn {
                device,
                interface,
                network,
                exclude,
                on_link,
            } => {
                let address = &t.devices[device].interfaces[interface].address;
                let issue = match address {
                    None => Some("No address is assigned yet. Enter the workstation address and /64.".to_owned()),
                    Some(a) if a.prefix != 64 => Some(format!("The assigned prefix is /{}. This task uses /64; the prefix determines which addresses are treated as local.",a.prefix)),
                    Some(a) if !network.contains(a.address) => Some(format!("This address is outside {}. Keep the supplied subnet prefix and choose the workstation part within it.",network.text())),
                    Some(a) if reserved_workstation_id(a.address) => Some("This interface identifier is reserved for subnet anycast. Choose an ordinary workstation address; IPv6 does not have an IPv4-style broadcast address.".into()),
                    Some(a) if exclude.contains(&a.address) => Some("This address is already assigned or explicitly excluded by the task. Choose a different address in the same subnet.".into()),
                    Some(a) if a.on_link != *on_link => Some("The manual on-link setting differs from the task. An assigned address alone is not an on-link declaration.".into()),
                    _ => None,
                };
                (issue.is_none(),format!("{device}/{interface}: {}",issue.unwrap_or_else(||"Address and connected prefix meet the task.".into())))
            }
            Self::KeepAddress {
                device,
                interface,
                assignment,
            } => (
                t.devices[device].interfaces[interface].address == *assignment,
                format!("Keep the supplied address/prefix on {device}/{interface}."),
            ),
            Self::Forwarding { device, enabled } => (
                t.devices[device].forwarding == *enabled,
                format!(
                    "{device}: forwarding must be {}.",
                    if *enabled { "enabled" } else { "disabled" }
                ),
            ),
            Self::Reachable {
                label,
                from,
                target,
                scope,
                first_hop,
                when,
            } => {
                let mut candidate = t.clone();
                for c in when {
                    candidate
                        .devices
                        .get_mut(&c.device)
                        .unwrap()
                        .interfaces
                        .get_mut(&c.interface)
                        .unwrap()
                        .up = c.up;
                }
                match candidate.ping(from, *target, scope.as_deref()) {
                    Ok(ping)
                        if first_hop.as_ref().is_some_and(|required| {
                            ping.forward
                                .hops
                                .first()
                                .is_none_or(|hop| &hop.interface != required)
                        }) =>
                    {
                        (
                            false,
                            format!(
                                "{label}: traffic uses {} as its first interface; the task requires {}. Inspect equally specific routes and their distances.",
                                ping.forward
                                    .hops
                                    .first()
                                    .map_or("local delivery", |hop| hop.interface.as_str()),
                                first_hop.as_deref().unwrap()
                            ),
                        )
                    }
                    Ok(_) => (true, format!("{label}: forward delivery and reply work.")),
                    Err(e) => (false, format!("{label}: {e}")),
                }
            }
        }
    }
}
