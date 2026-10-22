//! Pure bounded IPv4 static forwarding. No host network, filesystem or process API.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
    net::Ipv4Addr,
};
fn name(s: &str) -> bool {
    s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct Bounded<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Bounded<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique IPv4 names")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(key) = a.next_key::<String>()? {
                if out.len() >= N || !name(&key) || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate IPv4 name or mapping limit",
                    ));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Bounded::<V, N>(PhantomData))
}
fn devices<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Device>, D::Error> {
    map::<D, Device, 12>(d)
}
fn interfaces<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Interface>, D::Error> {
    map::<D, Interface, 8>(d)
}
fn routes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Route>, D::Error> {
    struct Routes;
    impl<'de> Visitor<'de> for Routes {
        type Value = Vec<Route>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most32 IPv4 static routes")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(route) = a.next_element::<Route>()? {
                if out.len() >= 32 {
                    return Err(A::Error::custom("IPv4 route limit exceeded"));
                }
                out.push(route);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Routes)
}
pub fn mask(length: u8) -> Option<u32> {
    match length {
        0 => Some(0),
        1..=32 => Some(u32::MAX << (32 - length)),
        _ => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub address: Ipv4Addr,
    pub length: u8,
}
impl Prefix {
    pub fn new(address: Ipv4Addr, length: u8) -> Option<Self> {
        Some(Self {
            address: (u32::from(address) & mask(length)?).into(),
            length,
        })
    }
    pub fn valid(&self) -> bool {
        Self::new(self.address, self.length) == Some(*self)
    }
    pub fn contains(&self, address: Ipv4Addr) -> bool {
        self.valid() && Self::new(address, self.length) == Some(*self)
    }
    /// Highest address in the prefix; /31-/32 route bounds are not broadcast semantics.
    pub fn last_address(&self) -> Option<Ipv4Addr> {
        if !self.valid() {
            return None;
        }
        Some((u32::from(self.address) | !mask(self.length)?).into())
    }
    pub fn text(&self) -> String {
        format!("{}/{}", self.address, self.length)
    }
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 32 {
            return Err("IPv4 prefix is too long".into());
        }
        let (ip, n) = text
            .split_once('/')
            .ok_or("Use a network/prefix such as 192.0.2.0/24")?;
        let ip = ip
            .parse::<Ipv4Addr>()
            .map_err(|_| "Malformed IPv4 address")?;
        if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Prefix must be a whole number from0 to32".into());
        }
        let n = n.parse::<u8>().map_err(|_| "Prefix must be from0 to32")?;
        let p = Self::new(ip, n).ok_or("Prefix must be from0 to32")?;
        if p.address != ip {
            return Err(format!(
                "This is a host address inside {}. Enter the aligned network prefix.",
                p.text()
            ));
        }
        Ok(p)
    }
}
pub fn supported_address(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_documentation()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub address: Ipv4Addr,
    pub prefix: u8,
}
impl Assignment {
    pub fn valid(&self) -> bool {
        (1..=30).contains(&self.prefix) && supported_address(self.address)
    }
    pub fn usable(&self) -> bool {
        self.valid()
            && Prefix::new(self.address, self.prefix).is_some_and(|p| {
                p.address != self.address && p.last_address() != Some(self.address)
            })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub link: String,
    pub up: bool,
    pub address: Option<Assignment>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: Prefix,
    pub interface: String,
    pub via: Ipv4Addr,
    pub distance: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub forwarding: bool,
    #[serde(deserialize_with = "interfaces")]
    pub interfaces: BTreeMap<String, Interface>,
    #[serde(deserialize_with = "routes")]
    pub routes: Vec<Route>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    #[serde(deserialize_with = "devices")]
    pub devices: BTreeMap<String, Device>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteStatus {
    Eligible,
    InterfaceDown,
    NoUsableAddress,
    Distance255,
    NextHopOffLink,
    ReservedNextHop,
    SelfNextHop,
}
impl RouteStatus {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::Eligible => {
                "Eligible static candidate; prefix and distance still determine selection"
            }
            Self::InterfaceDown => "Outgoing interface is down",
            Self::NoUsableAddress => {
                "Outgoing interface needs an ordinary host address and /1–/30 prefix"
            }
            Self::Distance255 => "Distance255 prevents installation",
            Self::NextHopOffLink => {
                "Next hop is outside the outgoing subnet; recursive next-hop lookup is unsupported"
            }
            Self::ReservedNextHop => {
                "Next hop is the network or broadcast address; choose the neighboring router's host address"
            }
            Self::SelfNextHop => {
                "Next hop is assigned on this device; choose the neighboring router's address"
            }
        }
    }
}
fn route_status(d: &Device, r: &Route) -> RouteStatus {
    let i = &d.interfaces[&r.interface];
    if !i.up {
        return RouteStatus::InterfaceDown;
    }
    if r.distance == 255 {
        return RouteStatus::Distance255;
    }
    let Some(a) = i.address.as_ref().filter(|a| a.usable()) else {
        return RouteStatus::NoUsableAddress;
    };
    let p = Prefix::new(a.address, a.prefix).unwrap();
    if d.interfaces
        .values()
        .any(|i| i.address.as_ref().is_some_and(|a| a.address == r.via))
    {
        RouteStatus::SelfNextHop
    } else if !p.contains(r.via) {
        RouteStatus::NextHopOffLink
    } else if p.address == r.via || p.last_address() == Some(r.via) {
        RouteStatus::ReservedNextHop
    } else {
        RouteStatus::Eligible
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub destination: Prefix,
    pub interface: String,
    pub via: Option<Ipv4Addr>,
    pub distance: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hop {
    pub device: String,
    pub interface: String,
    pub next_hop: Ipv4Addr,
    pub peer: String,
    pub peer_interface: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub hops: Vec<Hop>,
    pub destination_device: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub source: Ipv4Addr,
    pub forward: Trace,
    pub reply: Trace,
}
impl Topology {
    pub fn decode(text: &str) -> Result<Self, String> {
        if text.len() > 1_000_000 {
            return Err("IPv4 topology exceeds1MB".into());
        }
        let t: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if !t.valid() {
            return Err("Invalid or unsupported IPv4 topology".into());
        }
        Ok(t)
    }
    pub fn valid(&self) -> bool {
        !self.devices.is_empty()
            && self.devices.len() <= 12
            && self.devices.iter().all(|(id, d)| {
                name(id)
                    && !d.interfaces.is_empty()
                    && d.interfaces.len() <= 8
                    && d.routes.len() <= 32
                    && d.interfaces.iter().all(|(id, i)| {
                        name(id)
                            && name(&i.link)
                            && i.address.as_ref().is_none_or(Assignment::valid)
                    })
                    && d.routes.iter().all(|r| {
                        r.destination.valid()
                            && d.interfaces.contains_key(&r.interface)
                            && r.distance > 0
                            && supported_address(r.via)
                    })
            })
    }
    pub fn same_shape(&self, original: &Self) -> bool {
        self.devices.keys().eq(original.devices.keys())
            && self.devices.iter().all(|(id, d)| {
                let other = &original.devices[id];
                d.interfaces.keys().eq(other.interfaces.keys())
                    && d.interfaces
                        .iter()
                        .all(|(port, i)| i.link == other.interfaces[port].link)
            })
    }
    pub fn route_status(&self, device: &str, index: usize) -> Result<RouteStatus, String> {
        if !self.valid() {
            return Err("Invalid IPv4 topology".into());
        }
        let d = self.devices.get(device).ok_or("Unknown device")?;
        let r = d.routes.get(index).ok_or("Unknown route")?;
        Ok(route_status(d, r))
    }
    pub fn select(&self, device: &str, destination: Ipv4Addr) -> Result<Selected, String> {
        if !self.valid() {
            return Err("Invalid IPv4 topology".into());
        }
        let d = self.devices.get(device).ok_or("Unknown device")?;
        let mut candidates = Vec::new();
        for (id, i) in &d.interfaces {
            if !i.up {
                continue;
            }
            if let Some(a) = &i.address
                && a.usable()
            {
                let p = Prefix::new(a.address, a.prefix).unwrap();
                if p.contains(destination) {
                    candidates.push(Selected {
                        destination: p,
                        interface: id.clone(),
                        via: None,
                        distance: 0,
                    });
                }
            }
        }
        for r in &d.routes {
            if r.destination.contains(destination) && route_status(d, r) == RouteStatus::Eligible {
                candidates.push(Selected {
                    destination: r.destination,
                    interface: r.interface.clone(),
                    via: Some(r.via),
                    distance: r.distance,
                });
            }
        }
        candidates.sort_by_key(|r| (std::cmp::Reverse(r.destination.length), r.distance));
        let Some(selected) = candidates.first() else {
            let reasons = d
                .routes
                .iter()
                .filter(|r| r.destination.contains(destination))
                .filter_map(|r| {
                    let status = route_status(d, r);
                    (status != RouteStatus::Eligible)
                        .then(|| format!("{}: {}", r.destination.text(), status.explanation()))
                })
                .take(3)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "{device}: no eligible route to {destination}. Inspect connected prefixes and static routes. {reasons}"
            ));
        };
        if candidates.iter().skip(1).any(|r| {
            r.destination.length == selected.destination.length
                && r.distance == selected.distance
                && (r.interface != selected.interface || r.via != selected.via)
        }) {
            return Err(format!(
                "{device}: equal-cost distinct forwarding choices are unsupported; choose one preferred route or different distances"
            ));
        }
        Ok(selected.clone())
    }
    fn neighbors(&self, device: &str, interface: &str, address: Ipv4Addr) -> Vec<(String, String)> {
        let link = &self.devices[device].interfaces[interface].link;
        self.devices
            .iter()
            .flat_map(|(id, d)| {
                d.interfaces
                    .iter()
                    .filter(|(_, i)| {
                        i.up && &i.link == link
                            && i.address.as_ref().is_some_and(|a| a.address == address)
                    })
                    .map(|(port, _)| (id.clone(), port.clone()))
            })
            .collect()
    }
    fn local(&self, device: &str, address: Ipv4Addr) -> Result<bool, String> {
        for (port, i) in &self.devices[device].interfaces {
            if i.up && i.address.as_ref().is_some_and(|a| a.address == address) {
                if self.neighbors(device, port, address).len() > 1 {
                    return Err(format!(
                        "{device}: duplicate address {address} on link {}",
                        i.link
                    ));
                }
                if !i.address.as_ref().unwrap().usable() {
                    return Err(format!(
                        "{device}/{port}: {address} is the network or broadcast address for its configured subnet"
                    ));
                }
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn path(&self, start: &str, destination: Ipv4Addr) -> Result<Trace, String> {
        let mut current = start.to_string();
        let mut hops = Vec::new();
        let mut visited = BTreeSet::new();
        for _ in 0..16 {
            if self.local(&current, destination)? {
                return Ok(Trace {
                    hops,
                    destination_device: current,
                });
            }
            if !visited.insert(current.clone()) {
                return Err(format!("Routing loop: {current} was visited again"));
            }
            if !hops.is_empty() && !self.devices[&current].forwarding {
                return Err(format!(
                    "{current}: forwarding is disabled; this device cannot carry traffic between networks"
                ));
            }
            let selected = self.select(&current, destination)?;
            let next_hop = selected.via.unwrap_or(destination);
            let outgoing = &self.devices[&current].interfaces[&selected.interface];
            let a=outgoing.address.as_ref().filter(|a|a.usable()).ok_or("Outgoing interface has no usable IPv4 source; unnumbered interfaces are unsupported")?;
            let subnet = Prefix::new(a.address, a.prefix).unwrap();
            if subnet.address == next_hop || subnet.last_address() == Some(next_hop) {
                return Err(format!(
                    "{current}: {next_hop} is a network/broadcast address on the outgoing subnet, not a unicast neighbor"
                ));
            }
            let neighbors = self.neighbors(&current, &selected.interface, next_hop);
            if neighbors.len() > 1 {
                return Err(format!(
                    "{current}: duplicate next-hop address {next_hop} on the selected link"
                ));
            }
            let Some((peer, port)) = neighbors.first() else {
                return Err(format!(
                    "{current}: no up neighbor owns {next_hop} on {}. {}",
                    selected.interface,
                    if selected.via.is_some() {
                        "The static route is still eligible; absence alone does not activate a backup."
                    } else {
                        "The destination was treated as directly attached. Check its address, the local subnet prefix and the link."
                    }
                ));
            };
            if peer == &current {
                return Err(format!(
                    "{current}: the next hop points back to this device"
                ));
            }
            let peer_address = self.devices[peer].interfaces[port]
                .address
                .as_ref()
                .unwrap();
            if !peer_address.usable() {
                return Err(format!(
                    "{peer}/{port}: {next_hop} is a network/broadcast address under the neighbor's configured prefix"
                ));
            }
            hops.push(Hop {
                device: current.clone(),
                interface: selected.interface,
                next_hop,
                peer: peer.clone(),
                peer_interface: port.clone(),
            });
            current = peer.clone();
        }
        Err("Forwarding hop limit reached (16); inspect routes for a loop".into())
    }
    /// Raw address observation for inspection, never sufficient by itself for endpoint grading.
    pub fn probe(&self, from: &str, target: Ipv4Addr) -> Result<Probe, String> {
        if !self.valid() {
            return Err("Invalid IPv4 topology; no probe simulated".into());
        }
        let d = self.devices.get(from).ok_or("Unknown source device")?;
        if !supported_address(target) {
            return Err("Unsupported probe destination; only modeled private/documentation IPv4 endpoints are supported".into());
        }
        let source = if self.local(from, target)? {
            target
        } else {
            let selected = self.select(from, target)?;
            d.interfaces[&selected.interface]
                .address
                .as_ref()
                .filter(|a| a.usable())
                .map(|a| a.address)
                .ok_or("Selected outgoing interface has no usable IPv4 source")?
        };
        if !self.local(from, source)? {
            return Err("The source address is not usable on an up interface".into());
        }
        let forward = self.path(from, target)?;
        let reply = self
            .path(&forward.destination_device, source)
            .map_err(|e| format!("Forward delivery works, but the reply fails: {e}"))?;
        if reply.destination_device != from {
            return Err("Reply reached a different device using the source address; inspect duplicate assignments".into());
        }
        Ok(Probe {
            source,
            forward,
            reply,
        })
    }
    /// Bind the intended device/interface independently of mutable current address ownership.
    /// The interface selects the target address; weak-host local delivery does not pin ingress.
    /// Authored fixed-address/subnet constraints must be checked separately by the goal adapter.
    pub fn verify_endpoint(
        &self,
        from: &str,
        target_device: &str,
        target_interface: &str,
    ) -> Result<Probe, String> {
        if !self.valid() {
            return Err("Invalid IPv4 topology; no endpoint verification simulated".into());
        }
        let i = self
            .devices
            .get(target_device)
            .and_then(|d| d.interfaces.get(target_interface))
            .ok_or("Unknown intended destination device/interface")?;
        if !i.up {
            return Err(format!(
                "Intended destination {target_device}/{target_interface} is down"
            ));
        }
        let address = i
            .address
            .as_ref()
            .filter(|a| a.usable())
            .ok_or("Intended destination needs a usable host address")?
            .address;
        let result = self.probe(from, address)?;
        if result.forward.destination_device != target_device {
            return Err(format!(
                "The address reached {}, not intended destination {target_device}; inspect duplicate or relocated assignments",
                result.forward.destination_device
            ));
        }
        Ok(result)
    }
}
