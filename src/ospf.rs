//! Pure bounded OSPFv2 compatibility/SPF snapshots, not a protocol FSM or packet engine.
use crate::ipv4_routing::{Assignment, Prefix, Topology};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
    net::Ipv4Addr,
};

pub const MAX_DEVICES: usize = 12;
pub const MAX_PORTS: usize = 8;
pub const MAX_STATE_BYTES: usize = 262_144;
pub const MAX_ACTION_BYTES: usize = 4096;
pub const MAX_ERROR_BYTES: usize = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    Invalid,
    Limit,
    Unsupported,
    Context,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    pub message: String,
}
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
fn fault(kind: FaultKind, text: &str) -> Fault {
    let mut end = text.len().min(MAX_ERROR_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Fault {
        kind,
        message: text[..end].to_owned(),
    }
}
type Result<T> = std::result::Result<T, Fault>;
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
struct Name(String);
impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Text;
        impl Visitor<'_> for Text {
            type Value = Name;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("one bounded ASCII OSPF name")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<Name, E> {
                if name(s) {
                    Ok(Name(s.into()))
                } else {
                    Err(E::custom("Invalid OSPF name"))
                }
            }
        }
        d.deserialize_str(Text)
    }
}
fn text32<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    Ok(Name::deserialize(d)?.0)
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct Bounded<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Bounded<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique OSPF names")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Name(key)) = a.next_key::<Name>()? {
                if out.len() >= N || out.contains_key(&key) {
                    return Err(A::Error::custom("OSPF duplicate name or collection limit"));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Bounded::<V, N>(PhantomData))
}
fn routers<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, RouterConfig>, D::Error> {
    map::<D, RouterConfig, MAX_DEVICES>(d)
}
fn ports<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, PortConfig>, D::Error> {
    map::<D, PortConfig, MAX_PORTS>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "text32")]
    pub router: String,
    #[serde(deserialize_with = "text32")]
    pub interface: String,
}
impl Endpoint {
    pub fn new(router: &str, interface: &str) -> Self {
        Self {
            router: router.into(),
            interface: interface.into(),
        }
    }
    fn valid(&self) -> bool {
        name(&self.router) && name(&self.interface)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    PointToPoint,
    Broadcast,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    pub enabled: bool,
    pub process_id: u16,
    pub router_id: Option<Ipv4Addr>,
    pub area: u32,
}
impl ProcessConfig {
    fn valid(&self) -> bool {
        self.process_id > 0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortConfig {
    pub enabled: bool,
    pub passive: bool,
    pub network_type: NetworkType,
    pub cost: u16,
    pub hello_seconds: u16,
    pub dead_seconds: u32,
}
impl PortConfig {
    fn valid(&self) -> bool {
        self.cost > 0 && self.hello_seconds > 0 && self.dead_seconds > 0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterConfig {
    pub process: ProcessConfig,
    #[serde(deserialize_with = "ports")]
    pub interfaces: BTreeMap<String, PortConfig>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "routers")]
    pub routers: BTreeMap<String, RouterConfig>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct State {
    schema_version: u8,
    topology: Topology,
    configuration: Configuration,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Data {
    schema_version: u8,
    topology: Topology,
    configuration: Configuration,
}

fn topology_valid(topology: &Topology) -> Result<()> {
    if !topology.valid() {
        return Err(fault(
            FaultKind::Invalid,
            "Invalid or oversized bounded IPv4 topology",
        ));
    }
    if !topology.devices.values().any(|d| d.forwarding) {
        return Err(fault(
            FaultKind::Invalid,
            "At least one forwarding router is required",
        ));
    }
    let mut addresses = BTreeSet::new();
    for device in topology.devices.values() {
        if !device.routes.is_empty() {
            return Err(fault(
                FaultKind::Unsupported,
                "Static route competition is outside this OSPF-only foundation",
            ));
        }
        let mut links = BTreeSet::new();
        for port in device.interfaces.values() {
            if !links.insert(&port.link) {
                return Err(fault(
                    FaultKind::Invalid,
                    "One device cannot attach twice to the same virtual segment",
                ));
            }
            if let Some(a) = &port.address
                && !addresses.insert(a.address)
            {
                return Err(fault(
                    FaultKind::Invalid,
                    "Duplicate virtual interface addresses are ambiguous",
                ));
            }
        }
    }
    Ok(())
}
fn config_valid(topology: &Topology, configuration: &Configuration) -> Result<()> {
    if configuration.routers.len() > MAX_DEVICES {
        return Err(fault(FaultKind::Limit, "OSPF router configuration limit"));
    }
    let router_ids = topology
        .devices
        .iter()
        .filter(|(_, d)| d.forwarding)
        .map(|(id, _)| id);
    if !router_ids.eq(configuration.routers.keys()) {
        return Err(fault(
            FaultKind::Context,
            "OSPF configuration must name every forwarding router exactly once",
        ));
    }
    for (id, router) in &configuration.routers {
        if !name(id) || !router.process.valid() || router.interfaces.len() > MAX_PORTS {
            return Err(fault(
                FaultKind::Invalid,
                "Invalid process identity or interface configuration bounds",
            ));
        }
        if !router
            .interfaces
            .keys()
            .eq(topology.devices[id].interfaces.keys())
        {
            return Err(fault(
                FaultKind::Context,
                "OSPF interface keys must match the fixed topology",
            ));
        }
        if router
            .interfaces
            .iter()
            .any(|(id, p)| !name(id) || !p.valid())
        {
            return Err(fault(
                FaultKind::Invalid,
                "OSPF costs and Hello/Dead parameters must be positive bounded integers",
            ));
        }
    }
    Ok(())
}
impl State {
    pub fn new(topology: Topology, configuration: Configuration) -> Result<Self> {
        topology_valid(&topology)?;
        config_valid(&topology, &configuration)?;
        Ok(Self {
            schema_version: 1,
            topology,
            configuration,
        })
    }
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_STATE_BYTES {
            return Err(fault(FaultKind::Limit, "OSPF state JSON exceeds256KiB"));
        }
        let data: Data = serde_json::from_str(text).map_err(|_| {
            fault(
                FaultKind::Invalid,
                "Malformed OSPF state, unknown fields, duplicate names or decode bounds",
            )
        })?;
        if data.schema_version != 1 {
            return Err(fault(FaultKind::Invalid, "Unsupported OSPF state schema"));
        }
        Self::new(data.topology, data.configuration)
    }
    pub fn valid(&self) -> bool {
        self.schema_version == 1
            && topology_valid(&self.topology).is_ok()
            && config_valid(&self.topology, &self.configuration).is_ok()
    }
    pub fn valid_against(&self, initial: &Self) -> bool {
        self.valid()
            && initial.valid()
            && self.topology.same_shape(&initial.topology)
            && self
                .topology
                .devices
                .iter()
                .all(|(id, d)| d.forwarding == initial.topology.devices[id].forwarding)
    }
    pub fn topology(&self) -> &Topology {
        &self.topology
    }
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn reconfigure(&mut self, configuration: &Configuration) -> Result<()> {
        self.validate()?;
        config_valid(&self.topology, configuration)?;
        self.configuration = configuration.clone();
        Ok(())
    }
    fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(fault(FaultKind::Invalid, "Unsupported OSPF state schema"));
        }
        topology_valid(&self.topology)?;
        config_valid(&self.topology, &self.configuration)
    }
    pub fn apply(&mut self, action: &Action) -> Result<()> {
        self.validate()?;
        match action {
            Action::SetProcess { router, process } => {
                if !name(router) || !self.configuration.routers.contains_key(router) {
                    return Err(fault(FaultKind::Context, "Unknown OSPF router"));
                }
                if !process.valid() {
                    return Err(fault(
                        FaultKind::Invalid,
                        "OSPF process ID must be positive",
                    ));
                }
            }
            Action::SetPort {
                endpoint,
                configuration,
            } => {
                self.endpoint(endpoint)?;
                if !configuration.valid() {
                    return Err(fault(
                        FaultKind::Invalid,
                        "Invalid positive OSPF cost or timer",
                    ));
                }
            }
            Action::SetAddress { endpoint, address } => {
                self.endpoint(endpoint)?;
                if let Some(a) = address {
                    if !a.valid() {
                        return Err(fault(
                            FaultKind::Unsupported,
                            "Only ordinary private/documentation IPv4 assignments with /1–/30 are supported",
                        ));
                    }
                    if self.topology.devices.iter().any(|(r, d)| {
                        d.interfaces.iter().any(|(i, p)| {
                            (r != &endpoint.router || i != &endpoint.interface)
                                && p.address
                                    .as_ref()
                                    .is_some_and(|other| other.address == a.address)
                        })
                    }) {
                        return Err(fault(
                            FaultKind::Invalid,
                            "Duplicate virtual interface address",
                        ));
                    }
                }
            }
            Action::SetUp { endpoint, .. } => {
                self.endpoint(endpoint)?;
            }
        }
        // Every input collection/name/value/context above was checked by borrowing.
        let mut candidate = self.clone();
        match action {
            Action::SetProcess { router, process } => {
                candidate
                    .configuration
                    .routers
                    .get_mut(router)
                    .unwrap()
                    .process = process.clone()
            }
            Action::SetPort {
                endpoint,
                configuration,
            } => {
                *candidate
                    .configuration
                    .routers
                    .get_mut(&endpoint.router)
                    .unwrap()
                    .interfaces
                    .get_mut(&endpoint.interface)
                    .unwrap() = configuration.clone()
            }
            Action::SetAddress { endpoint, address } => {
                candidate
                    .topology
                    .devices
                    .get_mut(&endpoint.router)
                    .unwrap()
                    .interfaces
                    .get_mut(&endpoint.interface)
                    .unwrap()
                    .address = address.clone()
            }
            Action::SetUp { endpoint, up } => {
                candidate
                    .topology
                    .devices
                    .get_mut(&endpoint.router)
                    .unwrap()
                    .interfaces
                    .get_mut(&endpoint.interface)
                    .unwrap()
                    .up = *up
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    fn endpoint(&self, endpoint: &Endpoint) -> Result<()> {
        if endpoint.valid()
            && self
                .configuration
                .routers
                .get(&endpoint.router)
                .is_some_and(|r| r.interfaces.contains_key(&endpoint.interface))
        {
            Ok(())
        } else {
            Err(fault(
                FaultKind::Context,
                "Unknown OSPF router/interface endpoint",
            ))
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    SetProcess {
        #[serde(deserialize_with = "text32")]
        router: String,
        process: ProcessConfig,
    },
    SetPort {
        endpoint: Endpoint,
        configuration: PortConfig,
    },
    SetAddress {
        endpoint: Endpoint,
        address: Option<Assignment>,
    },
    SetUp {
        endpoint: Endpoint,
        up: bool,
    },
}
impl Action {
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_ACTION_BYTES {
            return Err(fault(FaultKind::Limit, "OSPF action JSON exceeds4KiB"));
        }
        serde_json::from_str(text).map_err(|_| {
            fault(
                FaultKind::Invalid,
                "Malformed or oversized typed OSPF action",
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum RouterIssue {
    Disabled,
    MissingRouterId,
    InvalidRouterId,
    DuplicateRouterId {
        router_id: Ipv4Addr,
        routers: BTreeSet<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Condition {
    Router { router: String, issue: RouterIssue },
    PortDisabled { at: Endpoint },
    PortDown { at: Endpoint },
    Passive { at: Endpoint },
    AddressUnavailable { at: Endpoint },
    UnsupportedActiveBroadcast { at: Endpoint },
    UnsupportedSegment { endpoints: usize },
    NoPeer,
    PeerNotRouter { peer: Endpoint },
    AreaMismatch { local: u32, peer: u32 },
    HelloMismatch { local: u16, peer: u16 },
    DeadMismatch { local: u32, peer: u32 },
    PointToPointMaskIgnored { local: u8, peer: u8 },
    PointToPointSubnetIgnored,
}
impl Condition {
    pub fn blocking(&self) -> bool {
        !matches!(
            self,
            Self::PointToPointMaskIgnored { .. } | Self::PointToPointSubnetIgnored
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LinkView {
    pub local: Endpoint,
    pub peer: Option<Endpoint>,
    pub compatible: bool,
    pub conditions: Vec<Condition>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct NextHop {
    pub interface: String,
    pub via: Ipv4Addr,
    pub peer: Endpoint,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Route {
    pub destination: Prefix,
    pub cost: u32,
    pub next_hops: BTreeSet<NextHop>,
    pub advertisers: BTreeSet<Endpoint>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Connected {
    pub destination: Prefix,
    pub interfaces: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Selection {
    Connected(Connected),
    Ospf(Route),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RouterView {
    pub issues: Vec<RouterIssue>,
    pub links: Vec<LinkView>,
    pub distances: BTreeMap<String, u32>,
    pub connected: Vec<Connected>,
    pub routes: Vec<Route>,
}
impl RouterView {
    pub fn route(&self, destination: Prefix) -> Option<&Route> {
        self.routes.iter().find(|r| r.destination == destination)
    }
    pub fn lookup(&self, address: Ipv4Addr) -> Option<Selection> {
        let local = self
            .connected
            .iter()
            .filter(|r| r.destination.contains(address))
            .max_by_key(|r| r.destination.length);
        let remote = self
            .routes
            .iter()
            .filter(|r| r.destination.contains(address))
            .max_by_key(|r| r.destination.length);
        match (local, remote) {
            (Some(l), Some(r)) if r.destination.length > l.destination.length => {
                Some(Selection::Ospf(r.clone()))
            }
            (Some(l), _) => Some(Selection::Connected(l.clone())),
            (None, Some(r)) => Some(Selection::Ospf(r.clone())),
            _ => None,
        }
    }
}
/// Observational output only. No Deserialize/restore path or cached success flag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub routers: BTreeMap<String, RouterView>,
}
type Segments = BTreeMap<String, Vec<Endpoint>>;
type Spf = (BTreeMap<String, u32>, BTreeMap<String, BTreeSet<NextHop>>);
fn subnet(a: &Assignment) -> Prefix {
    Prefix::new(a.address, a.prefix).expect("validated assignment")
}
fn prefix_key(p: Prefix) -> (u32, u8) {
    (u32::from(p.address), p.length)
}
impl State {
    fn segments(&self) -> Segments {
        let mut out = Segments::new();
        for (router, d) in &self.topology.devices {
            for (interface, p) in &d.interfaces {
                out.entry(p.link.clone())
                    .or_default()
                    .push(Endpoint::new(router, interface));
            }
        }
        out
    }
    fn router_issues(&self) -> BTreeMap<String, Vec<RouterIssue>> {
        let mut ids: BTreeMap<Ipv4Addr, BTreeSet<String>> = BTreeMap::new();
        for (r, c) in &self.configuration.routers {
            if c.process.enabled
                && let Some(id) = c.process.router_id
            {
                ids.entry(id).or_default().insert(r.clone());
            }
        }
        self.configuration
            .routers
            .iter()
            .map(|(r, c)| {
                let issues = if !c.process.enabled {
                    vec![RouterIssue::Disabled]
                } else {
                    match c.process.router_id {
                        None => vec![RouterIssue::MissingRouterId],
                        Some(id)
                            if id.is_unspecified() || id.is_multicast() || id.is_broadcast() =>
                        {
                            vec![RouterIssue::InvalidRouterId]
                        }
                        Some(id) if ids[&id].len() > 1 => vec![RouterIssue::DuplicateRouterId {
                            router_id: id,
                            routers: ids[&id].clone(),
                        }],
                        Some(_) => vec![],
                    }
                };
                (r.clone(), issues)
            })
            .collect()
    }
    fn port_conditions(
        &self,
        at: &Endpoint,
        issues: &BTreeMap<String, Vec<RouterIssue>>,
    ) -> Vec<Condition> {
        let port = &self.topology.devices[&at.router].interfaces[&at.interface];
        let config = &self.configuration.routers[&at.router].interfaces[&at.interface];
        let mut out: Vec<_> = issues[&at.router]
            .iter()
            .cloned()
            .map(|issue| Condition::Router {
                router: at.router.clone(),
                issue,
            })
            .collect();
        if !config.enabled {
            out.push(Condition::PortDisabled { at: at.clone() });
        }
        if !port.up {
            out.push(Condition::PortDown { at: at.clone() });
        }
        if config.passive {
            out.push(Condition::Passive { at: at.clone() });
        }
        if port.address.as_ref().is_none_or(|a| !a.usable()) {
            out.push(Condition::AddressUnavailable { at: at.clone() });
        }
        if config.network_type == NetworkType::Broadcast && !config.passive {
            out.push(Condition::UnsupportedActiveBroadcast { at: at.clone() });
        }
        out
    }
    fn link(
        &self,
        at: &Endpoint,
        segments: &Segments,
        issues: &BTreeMap<String, Vec<RouterIssue>>,
    ) -> LinkView {
        let port = &self.topology.devices[&at.router].interfaces[&at.interface];
        let segment = &segments[&port.link];
        let mut conditions = self.port_conditions(at, issues);
        let peer = if segment.len() == 2 {
            segment.iter().find(|p| *p != at).cloned()
        } else {
            None
        };
        if segment.len() > 2 {
            if !self.configuration.routers[&at.router].interfaces[&at.interface].passive {
                conditions.push(Condition::UnsupportedSegment {
                    endpoints: segment.len(),
                });
            }
        } else if let Some(peer) = &peer {
            if !self.configuration.routers.contains_key(&peer.router) {
                conditions.push(Condition::PeerNotRouter { peer: peer.clone() });
            } else {
                conditions.extend(self.port_conditions(peer, issues));
                let own = &self.configuration.routers[&at.router];
                let other = &self.configuration.routers[&peer.router];
                let a = &own.interfaces[&at.interface];
                let b = &other.interfaces[&peer.interface];
                if own.process.area != other.process.area {
                    conditions.push(Condition::AreaMismatch {
                        local: own.process.area,
                        peer: other.process.area,
                    });
                }
                if a.hello_seconds != b.hello_seconds {
                    conditions.push(Condition::HelloMismatch {
                        local: a.hello_seconds,
                        peer: b.hello_seconds,
                    });
                }
                if a.dead_seconds != b.dead_seconds {
                    conditions.push(Condition::DeadMismatch {
                        local: a.dead_seconds,
                        peer: b.dead_seconds,
                    });
                }
                if a.network_type == NetworkType::PointToPoint
                    && b.network_type == NetworkType::PointToPoint
                {
                    let peer_port =
                        &self.topology.devices[&peer.router].interfaces[&peer.interface];
                    if let (Some(local), Some(remote)) = (&port.address, &peer_port.address) {
                        if local.prefix != remote.prefix {
                            conditions.push(Condition::PointToPointMaskIgnored {
                                local: local.prefix,
                                peer: remote.prefix,
                            });
                        }
                        if !subnet(local).contains(remote.address)
                            || !subnet(remote).contains(local.address)
                        {
                            conditions.push(Condition::PointToPointSubnetIgnored);
                        }
                    }
                }
            }
        } else {
            conditions.push(Condition::NoPeer);
        }
        LinkView {
            local: at.clone(),
            peer,
            compatible: !conditions.iter().any(Condition::blocking),
            conditions,
        }
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        self.validate()?;
        let segments = self.segments();
        let issues = self.router_issues();
        let all_links: BTreeMap<String, Vec<LinkView>> = self
            .configuration
            .routers
            .iter()
            .map(|(r, c)| {
                (
                    r.clone(),
                    c.interfaces
                        .keys()
                        .map(|i| self.link(&Endpoint::new(r, i), &segments, &issues))
                        .collect(),
                )
            })
            .collect();
        let mut routers = BTreeMap::new();
        for source in self.configuration.routers.keys() {
            let (distances, hops) = self.spf(source, &all_links, &issues)?;
            let mut connected: BTreeMap<(u32, u8), Connected> = BTreeMap::new();
            for (i, p) in &self.topology.devices[source].interfaces {
                if p.up
                    && let Some(a) = p.address.as_ref().filter(|a| a.usable())
                {
                    connected
                        .entry(prefix_key(subnet(a)))
                        .or_insert_with(|| Connected {
                            destination: subnet(a),
                            interfaces: BTreeSet::new(),
                        })
                        .interfaces
                        .insert(i.clone());
                }
            }
            let mut routes: BTreeMap<(u32, u8), Route> = BTreeMap::new();
            for (advertiser, distance) in &distances {
                for (i, c) in &self.configuration.routers[advertiser].interfaces {
                    let port = &self.topology.devices[advertiser].interfaces[i];
                    if !c.enabled
                        || !port.up
                        || !c.passive && c.network_type != NetworkType::PointToPoint
                        || !c.passive && segments[&port.link].len() > 2
                    {
                        continue;
                    }
                    let Some(a) = port.address.as_ref().filter(|a| a.usable()) else {
                        continue;
                    };
                    let key = prefix_key(subnet(a));
                    if connected.contains_key(&key) {
                        continue;
                    }
                    let cost = distance
                        .checked_add(u32::from(c.cost))
                        .ok_or_else(|| fault(FaultKind::Limit, "OSPF cost arithmetic limit"))?;
                    let origin = Endpoint::new(advertiser, i);
                    let candidate = Route {
                        destination: subnet(a),
                        cost,
                        next_hops: hops[advertiser].clone(),
                        advertisers: BTreeSet::from([origin.clone()]),
                    };
                    match routes.get_mut(&key) {
                        Some(old) if old.cost < cost => {}
                        Some(old) if old.cost == cost => {
                            old.next_hops.extend(candidate.next_hops);
                            old.advertisers.insert(origin);
                        }
                        _ => {
                            routes.insert(key, candidate);
                        }
                    }
                }
            }
            routers.insert(
                source.clone(),
                RouterView {
                    issues: issues[source].clone(),
                    links: all_links[source].clone(),
                    distances,
                    connected: connected.into_values().collect(),
                    routes: routes.into_values().collect(),
                },
            );
        }
        Ok(Snapshot { routers })
    }
    fn spf(
        &self,
        source: &str,
        links: &BTreeMap<String, Vec<LinkView>>,
        issues: &BTreeMap<String, Vec<RouterIssue>>,
    ) -> Result<Spf> {
        let mut distances = BTreeMap::new();
        let mut hops = BTreeMap::new();
        let mut visited = BTreeSet::new();
        if !issues[source].is_empty() {
            return Ok((distances, hops));
        }
        distances.insert(source.to_owned(), 0u32);
        hops.insert(source.to_owned(), BTreeSet::new());
        while let Some(router) = distances
            .iter()
            .filter(|(r, _)| !visited.contains(*r))
            .min_by_key(|(_, d)| **d)
            .map(|(r, _)| r.clone())
        {
            visited.insert(router.clone());
            for link in links[&router].iter().filter(|l| l.compatible) {
                let peer = link.peer.as_ref().expect("compatible numbered P2P peer");
                let cost =
                    self.configuration.routers[&router].interfaces[&link.local.interface].cost;
                let distance = distances[&router]
                    .checked_add(u32::from(cost))
                    .ok_or_else(|| fault(FaultKind::Limit, "OSPF cost arithmetic limit"))?;
                let next = if router == source {
                    BTreeSet::from([NextHop {
                        interface: link.local.interface.clone(),
                        via: self.topology.devices[&peer.router].interfaces[&peer.interface]
                            .address
                            .as_ref()
                            .unwrap()
                            .address,
                        peer: peer.clone(),
                    }])
                } else {
                    hops[&router].clone()
                };
                match distances.get(&peer.router) {
                    Some(old) if *old < distance => {}
                    Some(old) if *old == distance => {
                        hops.get_mut(&peer.router).unwrap().extend(next);
                    }
                    _ => {
                        distances.insert(peer.router.clone(), distance);
                        hops.insert(peer.router.clone(), next);
                    }
                }
            }
        }
        Ok((distances, hops))
    }
}
