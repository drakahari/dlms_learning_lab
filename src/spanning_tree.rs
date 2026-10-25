//! Ideal single-instance P2P spanning-tree calculation, not a protocol or packet engine.
pub use crate::mac_learning::Mac;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    io::Write,
    marker::PhantomData,
};
pub const MAX_BRIDGES: usize = 12;
pub const MAX_PORTS: usize = 8;
pub const MAX_LINKS: usize = 48;
pub const MAX_COST: u32 = 200_000_000;
pub const MAX_STATE_BYTES: usize = 262_144;
pub const MAX_ACTION_BYTES: usize = 65_536;
pub const MAX_ERROR_BYTES: usize = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Invalid,
    Limit,
    Identity,
    Unsupported,
    Context,
    Invariant,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub message: String,
}
fn fault(kind: FaultKind, text: &str) -> Fault {
    let mut n = text.len().min(MAX_ERROR_BYTES);
    while !text.is_char_boundary(n) {
        n -= 1;
    }
    let mut message = String::with_capacity(n);
    message.push_str(&text[..n]);
    Fault { kind, message }
}
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
type Result<T> = std::result::Result<T, Fault>;
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
struct Name(String);
impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct N;
        impl Visitor<'_> for N {
            type Value = Name;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("one1..32-byte ASCII spanning-tree identity")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<Name, E> {
                if name(s) {
                    Ok(Name(s.into()))
                } else {
                    Err(E::custom("Invalid bounded spanning-tree identity"))
                }
            }
        }
        d.deserialize_str(N)
    }
}
fn text<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    Ok(Name::deserialize(d)?.0)
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for M<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} distinct bounded spanning-tree keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Name(k)) = a.next_key::<Name>()? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom("Spanning-tree map count/duplicate limit"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<V, N>(PhantomData))
}
fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    struct Count {
        n: usize,
        cap: usize,
    }
    impl Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Spanning-tree JSON limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, cap }, v).is_ok()
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "text")]
    pub bridge: String,
    #[serde(deserialize_with = "text")]
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bridge {
    pub mac: Mac,
    #[serde(deserialize_with = "map::<_,u16,MAX_PORTS>")]
    pub ports: BTreeMap<String, u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub a: Endpoint,
    pub b: Endpoint,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    pub system_id_extension: u16,
    #[serde(deserialize_with = "map::<_,Bridge,MAX_BRIDGES>")]
    pub bridges: BTreeMap<String, Bridge>,
    #[serde(deserialize_with = "map::<_,Link,MAX_LINKS>")]
    pub links: BTreeMap<String, Link>,
}
impl Topology {
    pub fn validate(&self) -> Result<()> {
        if self.bridges.is_empty()
            || self.bridges.len() > MAX_BRIDGES
            || self.links.len() > MAX_LINKS
        {
            return Err(fault(
                FaultKind::Limit,
                "Spanning-tree topology collection bound",
            ));
        }
        if self.system_id_extension > 4095 {
            return Err(fault(
                FaultKind::Unsupported,
                "Only one supplied12-bit system-ID extension is supported",
            ));
        }
        // All direct input collection/scalar checks precede derived sets/clones.
        for (k, b) in &self.bridges {
            if !name(k)
                || b.ports.len() > MAX_PORTS
                || b.ports
                    .iter()
                    .any(|(p, n)| !name(p) || !(1..=4095).contains(n))
            {
                return Err(fault(
                    FaultKind::Invalid,
                    "Invalid bridge/port identity or port-number bound",
                ));
            }
            if !b.mac.unicast() {
                return Err(fault(
                    FaultKind::Unsupported,
                    "Only conventional individual nonzero bridge MAC identities are supported",
                ));
            }
        }
        for (k, l) in &self.links {
            if !name(k)
                || [&l.a, &l.b]
                    .iter()
                    .any(|e| !name(&e.bridge) || !name(&e.port))
            {
                return Err(fault(
                    FaultKind::Invalid,
                    "Invalid bounded P2P link identity",
                ));
            }
        }
        let mut macs = BTreeSet::new();
        for b in self.bridges.values() {
            if !macs.insert(b.mac)
                || b.ports.values().collect::<BTreeSet<_>>().len() != b.ports.len()
            {
                return Err(fault(
                    FaultKind::Identity,
                    "Duplicate bridge MAC or local port number",
                ));
            }
        }
        let mut attached = BTreeSet::new();
        for l in self.links.values() {
            if l.a.bridge == l.b.bridge {
                return Err(fault(
                    FaultKind::Unsupported,
                    "Self-links/shared segments and backup ports are outside this P2P subset",
                ));
            }
            for e in [&l.a, &l.b] {
                if self
                    .bridges
                    .get(&e.bridge)
                    .is_none_or(|b| !b.ports.contains_key(&e.port))
                {
                    return Err(fault(FaultKind::Identity, "Unknown link endpoint"));
                }
                if !attached.insert(e) {
                    return Err(fault(
                        FaultKind::Unsupported,
                        "One supplied port cannot attach to multiple P2P links",
                    ));
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortConfig {
    pub priority: u8,
    pub cost: u32,
    pub up: bool,
}
impl PortConfig {
    fn valid(self) -> bool {
        self.priority.is_multiple_of(16) && (1..=MAX_COST).contains(&self.cost)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeConfig {
    pub priority: u16,
    pub online: bool,
    #[serde(deserialize_with = "map::<_,PortConfig,MAX_PORTS>")]
    pub ports: BTreeMap<String, PortConfig>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "map::<_,BridgeConfig,MAX_BRIDGES>")]
    pub bridges: BTreeMap<String, BridgeConfig>,
    #[serde(deserialize_with = "map::<_,bool,MAX_LINKS>")]
    pub links: BTreeMap<String, bool>,
}
impl Configuration {
    fn validate_shape(&self) -> Result<()> {
        if self.bridges.len() > MAX_BRIDGES || self.links.len() > MAX_LINKS {
            return Err(fault(
                FaultKind::Limit,
                "Spanning-tree configuration collection bound",
            ));
        }
        for (k, b) in &self.bridges {
            if !name(k)
                || b.ports.len() > MAX_PORTS
                || b.priority > 61440
                || !b.priority.is_multiple_of(4096)
                || b.ports.iter().any(|(k, p)| !name(k) || !p.valid())
            {
                return Err(fault(
                    FaultKind::Invalid,
                    "Invalid bridge priority, port priority or positive path cost",
                ));
            }
        }
        if self.links.keys().any(|k| !name(k)) {
            return Err(fault(FaultKind::Invalid, "Invalid link configuration key"));
        }
        Ok(())
    }
    pub fn validate_for(&self, t: &Topology) -> Result<()> {
        self.validate_shape()?;
        t.validate()?;
        if !self.bridges.keys().eq(t.bridges.keys())
            || !self.links.keys().eq(t.links.keys())
            || self
                .bridges
                .iter()
                .any(|(k, b)| !b.ports.keys().eq(t.bridges[k].ports.keys()))
        {
            return Err(fault(
                FaultKind::Context,
                "Configuration must preserve every supplied bridge, port and link identity",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    topology: Topology,
    configuration: Configuration,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            topology: Topology,
            configuration: Configuration,
        }
        let r = Raw::deserialize(d)
            .map_err(|_| D::Error::custom("Malformed bounded spanning-tree state"))?;
        Self::new(r.topology, r.configuration)
            .map_err(|_| D::Error::custom("Invalid or unsupported spanning-tree state"))
    }
}
impl State {
    pub fn new(topology: Topology, configuration: Configuration) -> Result<Self> {
        let s = Self {
            topology,
            configuration,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn topology(&self) -> &Topology {
        &self.topology
    }
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn validate(&self) -> Result<()> {
        self.configuration.validate_for(&self.topology)?;
        if !within(self, MAX_STATE_BYTES) {
            return Err(fault(
                FaultKind::Limit,
                "Spanning-tree compact state exceeds256KiB",
            ));
        }
        Ok(())
    }
    pub fn validate_context(&self, expected: &Topology) -> Result<()> {
        expected.validate()?;
        self.validate()?;
        if &self.topology != expected {
            return Err(fault(
                FaultKind::Context,
                "Spanning-tree fixed graph/MAC/port-number/instance context differs",
            ));
        }
        Ok(())
    }
    pub fn from_json(input: &[u8]) -> Result<Self> {
        if input.len() > MAX_STATE_BYTES {
            return Err(fault(
                FaultKind::Limit,
                "Spanning-tree raw JSON exceeds256KiB",
            ));
        }
        serde_json::from_slice(input).map_err(|_| {
            fault(
                FaultKind::Invalid,
                "Malformed or unsupported bounded spanning-tree JSON",
            )
        })
    }
    pub fn reconfigure(&mut self, c: Configuration) -> Result<()> {
        self.validate()?;
        c.validate_for(&self.topology)?;
        #[derive(Serialize)]
        struct Proposed<'a> {
            topology: &'a Topology,
            configuration: &'a Configuration,
        }
        if !within(
            &Proposed {
                topology: &self.topology,
                configuration: &c,
            },
            MAX_STATE_BYTES,
        ) {
            return Err(fault(
                FaultKind::Limit,
                "Proposed spanning-tree state exceeds256KiB",
            ));
        }
        self.configuration = c;
        Ok(())
    }
    pub fn apply(&mut self, a: &Action) -> Result<()> {
        self.validate()?;
        a.validate_for(&self.topology)?;
        let mut c = match a {
            Action::Reconfigure { configuration } => configuration.clone(),
            _ => self.configuration.clone(),
        };
        match a {
            Action::BridgePriority { bridge, priority } => {
                c.bridges.get_mut(bridge).unwrap().priority = *priority
            }
            Action::BridgeOnline { bridge, online } => {
                c.bridges.get_mut(bridge).unwrap().online = *online
            }
            Action::Port {
                bridge,
                port,
                settings,
            } => {
                c.bridges
                    .get_mut(bridge)
                    .unwrap()
                    .ports
                    .insert(port.clone(), *settings);
            }
            Action::LinkUp { link, up } => {
                c.links.insert(link.clone(), *up);
            }
            Action::Reconfigure { .. } => {}
        }
        self.reconfigure(c)
    }
    fn bid(&self, b: &str) -> BridgeId {
        BridgeId {
            priority: self.configuration.bridges[b].priority | self.topology.system_id_extension,
            mac: self.topology.bridges[b].mac,
        }
    }
    fn pid(&self, e: &Endpoint) -> PortId {
        PortId {
            priority: self.configuration.bridges[&e.bridge].ports[&e.port].priority,
            number: self.topology.bridges[&e.bridge].ports[&e.port],
        }
    }
    fn unavailable(
        &self,
        e: &Endpoint,
        attachment: Option<&(String, Endpoint)>,
    ) -> Option<Unavailable> {
        if !self.configuration.bridges[&e.bridge].online {
            return Some(Unavailable::BridgeOffline);
        }
        if !self.configuration.bridges[&e.bridge].ports[&e.port].up {
            return Some(Unavailable::LocalPortDown);
        }
        let Some((id, peer)) = attachment else {
            return Some(Unavailable::NoLink);
        };
        if !self.configuration.links[id] {
            return Some(Unavailable::LinkDown);
        }
        if !self.configuration.bridges[&peer.bridge].online {
            return Some(Unavailable::PeerBridgeOffline);
        }
        if !self.configuration.bridges[&peer.bridge].ports[&peer.port].up {
            return Some(Unavailable::PeerPortDown);
        }
        None
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        self.validate()?;
        let mut attachments = BTreeMap::new();
        for (k, l) in &self.topology.links {
            attachments.insert(l.a.clone(), (k.clone(), l.b.clone()));
            attachments.insert(l.b.clone(), (k.clone(), l.a.clone()));
        }
        let mut bridges = BTreeMap::new();
        for (k, b) in &self.topology.bridges {
            let mut ports = BTreeMap::new();
            for p in b.ports.keys() {
                let e = Endpoint {
                    bridge: k.clone(),
                    port: p.clone(),
                };
                let at = attachments.get(&e);
                let reason = self.unavailable(&e, at);
                ports.insert(
                    p.clone(),
                    PortView {
                        id: self.pid(&e),
                        role: Role::Disabled,
                        state: PortState::Discarding,
                        unavailable: reason,
                        peer: at.map(|(_, e)| e.clone()),
                        link: at.map(|(k, _)| k.clone()),
                        received: None,
                        candidate: None,
                        designated: None,
                    },
                );
            }
            bridges.insert(
                k.clone(),
                BridgeView {
                    id: self.bid(k),
                    root: None,
                    root_bridge: None,
                    root_cost: None,
                    root_port: None,
                    ports,
                },
            );
        }
        let active: BTreeSet<_> = self
            .topology
            .links
            .iter()
            .filter(|(_, l)| self.unavailable(&l.a, attachments.get(&l.a)).is_none())
            .map(|(k, _)| k.clone())
            .collect();
        let mut components = BTreeMap::new();
        let mut visited = BTreeSet::new();
        for (start, b) in &self.configuration.bridges {
            if !b.online || visited.contains(start) {
                continue;
            }
            let mut members = BTreeSet::from([start.clone()]);
            let mut q = VecDeque::from([start.clone()]);
            while let Some(n) = q.pop_front() {
                for id in &active {
                    let l = &self.topology.links[id];
                    let peer = if l.a.bridge == n {
                        Some(&l.b.bridge)
                    } else if l.b.bridge == n {
                        Some(&l.a.bridge)
                    } else {
                        None
                    };
                    if let Some(peer) = peer
                        && members.insert(peer.clone())
                    {
                        q.push_back(peer.clone());
                    }
                }
            }
            visited.extend(members.iter().cloned());
            let root = members.iter().min_by_key(|n| self.bid(n)).unwrap().clone();
            let root_id = self.bid(&root);
            let mut costs = BTreeMap::from([(root.clone(), 0u32)]);
            // Bounded Bellman-Ford relaxation. Entering a bridge adds its local port cost.
            for _ in 1..members.len() {
                let mut changed = false;
                for id in &active {
                    let l = &self.topology.links[id];
                    if !members.contains(&l.a.bridge) {
                        continue;
                    }
                    for (near, far) in [(&l.a, &l.b), (&l.b, &l.a)] {
                        if let Some(d) = costs.get(&near.bridge).copied() {
                            let c = d
                                .checked_add(
                                    self.configuration.bridges[&far.bridge].ports[&far.port].cost,
                                )
                                .ok_or_else(|| {
                                    fault(FaultKind::Limit, "Spanning-tree root path cost overflow")
                                })?;
                            if costs.get(&far.bridge).is_none_or(|old| c < *old) {
                                costs.insert(far.bridge.clone(), c);
                                changed = true;
                            }
                        }
                    }
                }
                if !changed {
                    break;
                }
            }
            if costs.len() != members.len() {
                return Err(fault(
                    FaultKind::Invariant,
                    "Operational component did not obtain complete root costs",
                ));
            }
            for n in &members {
                let v = bridges.get_mut(n).unwrap();
                v.root = Some(root_id);
                v.root_bridge = Some(root.clone());
                v.root_cost = Some(costs[n]);
            }
            for n in &members {
                if n == &root {
                    continue;
                }
                let mut best: Option<(RootPathPriority, String)> = None;
                for p in self.topology.bridges[n].ports.keys() {
                    let e = Endpoint {
                        bridge: n.clone(),
                        port: p.clone(),
                    };
                    if let Some((id, peer)) = attachments.get(&e) {
                        if !active.contains(id) {
                            continue;
                        }
                        let candidate = RootPathPriority {
                            root: root_id,
                            cost: costs[&peer.bridge]
                                .checked_add(self.configuration.bridges[n].ports[p].cost)
                                .ok_or_else(|| {
                                    fault(FaultKind::Limit, "Spanning-tree candidate cost overflow")
                                })?,
                            designated_bridge: self.bid(&peer.bridge),
                            designated_port: self.pid(peer),
                            receiving_port: self.pid(&e),
                        };
                        if candidate.cost == costs[n]
                            && best.as_ref().is_none_or(|(old, _)| candidate < *old)
                        {
                            best = Some((candidate, p.clone()));
                        }
                    }
                }
                bridges.get_mut(n).unwrap().root_port = Some(
                    best.ok_or_else(|| {
                        fault(
                            FaultKind::Invariant,
                            "Non-root bridge has no minimum root-port candidate",
                        )
                    })?
                    .1,
                );
            }
            components.insert(root, members);
        }
        let advertisement = |e: &Endpoint| -> Advertisement {
            let b = &bridges[&e.bridge];
            Advertisement {
                root: b.root.unwrap(),
                cost: b.root_cost.unwrap(),
                bridge: b.id,
                port: self.pid(e),
            }
        };
        let advertisements: BTreeMap<_, _> = attachments
            .keys()
            .filter(|e| active.contains(&attachments[*e].0))
            .map(|e| (e.clone(), advertisement(e)))
            .collect();
        let mut links = BTreeMap::new();
        for (k, l) in &self.topology.links {
            let operational = active.contains(k);
            let designated = if operational {
                Some(if advertisements[&l.a] < advertisements[&l.b] {
                    l.a.clone()
                } else {
                    l.b.clone()
                })
            } else {
                None
            };
            if let Some(winner) = &designated {
                for (local, peer) in [(&l.a, &l.b), (&l.b, &l.a)] {
                    let root_port = bridges[&local.bridge].root_port.as_ref() == Some(&local.port);
                    let role = if local == winner {
                        Role::Designated
                    } else if root_port {
                        Role::Root
                    } else {
                        Role::Alternate
                    };
                    if root_port && role != Role::Root {
                        return Err(fault(
                            FaultKind::Invariant,
                            "Selected root port incorrectly became designated",
                        ));
                    }
                    let received = advertisements[peer];
                    let v = bridges
                        .get_mut(&local.bridge)
                        .unwrap()
                        .ports
                        .get_mut(&local.port)
                        .unwrap();
                    v.role = role;
                    v.state = if role == Role::Alternate {
                        PortState::Discarding
                    } else {
                        PortState::Forwarding
                    };
                    v.received = Some(received);
                    v.candidate = Some(RootPathPriority {
                        root: received.root,
                        cost: received
                            .cost
                            .checked_add(
                                self.configuration.bridges[&local.bridge].ports[&local.port].cost,
                            )
                            .ok_or_else(|| fault(FaultKind::Limit, "Candidate cost overflow"))?,
                        designated_bridge: received.bridge,
                        designated_port: received.port,
                        receiving_port: v.id,
                    });
                    v.designated = Some(winner.clone());
                }
            }
            let forwarding = operational
                && bridges[&l.a.bridge].ports[&l.a.port].state == PortState::Forwarding
                && bridges[&l.b.bridge].ports[&l.b.port].state == PortState::Forwarding;
            links.insert(
                k.clone(),
                LinkView {
                    a: l.a.clone(),
                    b: l.b.clone(),
                    operational,
                    forwarding,
                    designated,
                },
            );
        }
        let snapshot = Snapshot {
            bridges,
            links,
            components,
        };
        snapshot.check_forest()?;
        Ok(snapshot)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    BridgePriority {
        #[serde(deserialize_with = "text")]
        bridge: String,
        priority: u16,
    },
    BridgeOnline {
        #[serde(deserialize_with = "text")]
        bridge: String,
        online: bool,
    },
    Port {
        #[serde(deserialize_with = "text")]
        bridge: String,
        #[serde(deserialize_with = "text")]
        port: String,
        settings: PortConfig,
    },
    LinkUp {
        #[serde(deserialize_with = "text")]
        link: String,
        up: bool,
    },
    Reconfigure {
        configuration: Configuration,
    },
}
impl Action {
    pub fn from_json(input: &[u8]) -> Result<Self> {
        if input.len() > MAX_ACTION_BYTES {
            return Err(fault(
                FaultKind::Limit,
                "Spanning-tree action JSON exceeds64KiB",
            ));
        }
        let action: Self = serde_json::from_slice(input).map_err(|_| {
            fault(
                FaultKind::Invalid,
                "Malformed bounded typed spanning-tree action",
            )
        })?;
        action.validate_shape()?;
        Ok(action)
    }
    fn validate_shape(&self) -> Result<()> {
        match self {
            Self::Reconfigure { configuration } => configuration.validate_shape(),
            Self::BridgePriority { bridge, priority }
                if !name(bridge) || *priority > 61440 || !priority.is_multiple_of(4096) =>
            {
                Err(fault(
                    FaultKind::Invalid,
                    "Invalid bounded bridge priority action",
                ))
            }
            Self::BridgeOnline { bridge, .. } if !name(bridge) => {
                Err(fault(FaultKind::Invalid, "Invalid bridge action identity"))
            }
            Self::Port {
                bridge,
                port,
                settings,
            } if !name(bridge) || !name(port) || !settings.valid() => {
                Err(fault(FaultKind::Invalid, "Invalid bounded port action"))
            }
            Self::LinkUp { link, .. } if !name(link) => {
                Err(fault(FaultKind::Invalid, "Invalid link action identity"))
            }
            _ => Ok(()),
        }
    }
    fn validate_for(&self, t: &Topology) -> Result<()> {
        self.validate_shape()?;
        match self {
            Self::Reconfigure { configuration } => configuration.validate_for(t),
            Self::LinkUp { link, .. } => {
                if name(link) && t.links.contains_key(link) {
                    Ok(())
                } else {
                    Err(fault(
                        FaultKind::Identity,
                        "Unknown or invalid supplied link",
                    ))
                }
            }
            Self::BridgePriority { bridge, priority } => {
                if *priority > 61440 || !priority.is_multiple_of(4096) {
                    return Err(fault(
                        FaultKind::Invalid,
                        "Bridge priority needs0..61440 in4096 increments",
                    ));
                }
                known_bridge(t, bridge)
            }
            Self::BridgeOnline { bridge, .. } => known_bridge(t, bridge),
            Self::Port {
                bridge,
                port,
                settings,
            } => {
                known_bridge(t, bridge)?;
                if !name(port) || !t.bridges[bridge].ports.contains_key(port) {
                    return Err(fault(
                        FaultKind::Identity,
                        "Unknown or invalid supplied port",
                    ));
                }
                if !settings.valid() {
                    return Err(fault(
                        FaultKind::Invalid,
                        "Port needs priority0..240 step16 and positive bounded cost",
                    ));
                }
                Ok(())
            }
        }
    }
}
fn known_bridge(t: &Topology, b: &str) -> Result<()> {
    if name(b) && t.bridges.contains_key(b) {
        Ok(())
    } else {
        Err(fault(
            FaultKind::Identity,
            "Unknown or invalid supplied bridge",
        ))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct BridgeId {
    pub priority: u16,
    pub mac: Mac,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct PortId {
    pub priority: u8,
    pub number: u16,
}
impl PortId {
    pub fn encoded(self) -> u16 {
        (u16::from(self.priority) << 8) | self.number
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Advertisement {
    pub root: BridgeId,
    pub cost: u32,
    pub bridge: BridgeId,
    pub port: PortId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RootPathPriority {
    pub root: BridgeId,
    pub cost: u32,
    pub designated_bridge: BridgeId,
    pub designated_port: PortId,
    pub receiving_port: PortId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Role {
    Root,
    Designated,
    Alternate,
    Disabled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PortState {
    Forwarding,
    Discarding,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Unavailable {
    BridgeOffline,
    LocalPortDown,
    NoLink,
    LinkDown,
    PeerBridgeOffline,
    PeerPortDown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PortView {
    pub id: PortId,
    pub role: Role,
    pub state: PortState,
    pub unavailable: Option<Unavailable>,
    pub peer: Option<Endpoint>,
    pub link: Option<String>,
    pub received: Option<Advertisement>,
    pub candidate: Option<RootPathPriority>,
    pub designated: Option<Endpoint>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BridgeView {
    pub id: BridgeId,
    pub root: Option<BridgeId>,
    pub root_bridge: Option<String>,
    pub root_cost: Option<u32>,
    pub root_port: Option<String>,
    pub ports: BTreeMap<String, PortView>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LinkView {
    pub a: Endpoint,
    pub b: Endpoint,
    pub operational: bool,
    pub forwarding: bool,
    pub designated: Option<Endpoint>,
}
/// Derived only. No decoder, saved authority or event-history credential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    bridges: BTreeMap<String, BridgeView>,
    links: BTreeMap<String, LinkView>,
    components: BTreeMap<String, BTreeSet<String>>,
}
impl Snapshot {
    pub fn bridges(&self) -> &BTreeMap<String, BridgeView> {
        &self.bridges
    }
    pub fn links(&self) -> &BTreeMap<String, LinkView> {
        &self.links
    }
    pub fn components(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.components
    }
    pub fn bridge(&self, name: &str) -> Option<&BridgeView> {
        self.bridges.get(name)
    }
    fn check_forest(&self) -> Result<()> {
        for (root, members) in &self.components {
            let edges = self
                .links
                .values()
                .filter(|l| l.forwarding && members.contains(&l.a.bridge))
                .count();
            if edges + 1 != members.len() {
                return Err(fault(
                    FaultKind::Invariant,
                    "Forwarding component does not have tree edge count",
                ));
            }
            let mut seen = BTreeSet::from([root.clone()]);
            let mut q = VecDeque::from([root.clone()]);
            while let Some(n) = q.pop_front() {
                for l in self.links.values().filter(|l| l.forwarding) {
                    let peer = if l.a.bridge == n {
                        Some(&l.b.bridge)
                    } else if l.b.bridge == n {
                        Some(&l.a.bridge)
                    } else {
                        None
                    };
                    if let Some(peer) = peer {
                        if !members.contains(peer) {
                            return Err(fault(
                                FaultKind::Invariant,
                                "Forwarding edge escaped its operational component",
                            ));
                        }
                        if seen.insert(peer.clone()) {
                            q.push_back(peer.clone());
                        }
                    }
                }
            }
            if &seen != members {
                return Err(fault(
                    FaultKind::Invariant,
                    "Forwarding tree does not span its operational component",
                ));
            }
        }
        Ok(())
    }
    pub fn forwarding_path(&self, from: &str, to: &str) -> Result<Option<Vec<String>>> {
        if !name(from)
            || !name(to)
            || !self.bridges.contains_key(from)
            || !self.bridges.contains_key(to)
        {
            return Err(fault(
                FaultKind::Identity,
                "Path query needs two known bounded bridge identities",
            ));
        }
        if self.bridges[from].root.is_none() || self.bridges[to].root.is_none() {
            return Ok(None);
        }
        let mut q = VecDeque::from([(from.to_owned(), Vec::new())]);
        let mut seen = BTreeSet::from([from.to_owned()]);
        while let Some((n, path)) = q.pop_front() {
            if n == to {
                return Ok(Some(path));
            }
            for (id, l) in &self.links {
                if !l.forwarding {
                    continue;
                }
                let peer = if l.a.bridge == n {
                    Some(&l.b.bridge)
                } else if l.b.bridge == n {
                    Some(&l.a.bridge)
                } else {
                    None
                };
                if let Some(peer) = peer
                    && seen.insert(peer.clone())
                {
                    let mut next = path.clone();
                    next.push(id.clone());
                    q.push_back((peer.clone(), next));
                }
            }
        }
        Ok(None)
    }
}
