//! Bounded, explicit-event LLDP neighbor discovery. No host/network I/O or real clock.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};

pub const MAX_DEVICES: usize = 12;
pub const MAX_PORTS: usize = 16;
pub const MAX_LINKS: usize = 96;
pub const MAX_NEIGHBORS: usize = MAX_DEVICES * MAX_PORTS;
/// Input cap for one declarative topology, including JSON escaping/formatting.
pub const MAX_BYTES: usize = 262_144;
/// A snapshot repeats endpoint and advertised strings in up to 192 cache entries.
/// Separately bounded so every legal maximum-size state fits compact or pretty JSON.
pub const MAX_STATE_BYTES: usize = 1_048_576;
pub const MAX_TIME: u64 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Limit,
    Identity,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn identity(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| (33..=126).contains(&b))
}
fn label(s: &Option<String>) -> bool {
    s.as_ref().is_none_or(|s| {
        s.len() <= 80 && !s.trim().is_empty() && s.bytes().all(|b| (32..=126).contains(&b))
    })
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct Entries<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique discovery keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(k) = a.next_key::<String>()? {
                if result.len() >= N || !key(&k) || result.contains_key(&k) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate discovery key or mapping limit",
                    ));
                }
                result.insert(k, a.next_value()?);
            }
            Ok(result)
        }
    }
    d.deserialize_map(Entries::<V, N>(PhantomData))
}
fn list<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<V>, D::Error> {
    struct Entries<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} discovery entries")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut result = Vec::new();
            while let Some(v) = a.next_element()? {
                if result.len() >= N {
                    return Err(A::Error::custom("Discovery collection limit exceeded"));
                }
                result.push(v);
            }
            Ok(result)
        }
    }
    d.deserialize_seq(Entries::<V, N>(PhantomData))
}
fn devices<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Device>, D::Error> {
    map::<D, Device, MAX_DEVICES>(d)
}
fn ports<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeMap<String, Port>, D::Error> {
    map::<D, Port, MAX_PORTS>(d)
}
fn links<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<Link>, D::Error> {
    list::<D, Link, MAX_LINKS>(d)
}
fn neighbors<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<Neighbor>, D::Error> {
    list::<D, Neighbor, MAX_NEIGHBORS>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    #[serde(deserialize_with = "devices")]
    pub devices: BTreeMap<String, Device>,
    #[serde(deserialize_with = "links")]
    pub links: Vec<Link>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    /// LLDP chassis-ID subtype: locally assigned only.
    pub chassis_id: String,
    pub system_name: Option<String>,
    pub enabled: bool,
    /// Advertised lifetime in seconds; independent of the receiving device's TTL.
    pub ttl: u16,
    #[serde(deserialize_with = "ports")]
    pub ports: BTreeMap<String, Port>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    /// LLDP port-ID subtype: locally assigned only.
    pub port_id: String,
    pub description: Option<String>,
    pub up: bool,
    pub transmit: bool,
    pub receive: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub device: String,
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub id: String,
    pub a: Endpoint,
    pub b: Endpoint,
    pub up: bool,
}

impl Topology {
    pub fn from_json(text: &str) -> Result<Self> {
        let result: Self = decode(text, MAX_BYTES)?;
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<()> {
        if self.devices.is_empty()
            || self.devices.len() > MAX_DEVICES
            || self.links.len() > MAX_LINKS
        {
            return Err(error(
                ErrorKind::Limit,
                "Discovery requires 1–12 devices and at most 96 links",
            ));
        }
        let mut chassis = BTreeSet::new();
        for (name, d) in &self.devices {
            if !key(name) || !identity(&d.chassis_id) || !label(&d.system_name) || d.ttl == 0 {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid discovery device key, identity, label or nonzero TTL",
                ));
            }
            if !chassis.insert(&d.chassis_id) {
                return Err(error(ErrorKind::Identity, "Duplicate chassis ID"));
            }
            if d.ports.is_empty() || d.ports.len() > MAX_PORTS {
                return Err(error(
                    ErrorKind::Limit,
                    "Each discovery device requires 1–16 ports",
                ));
            }
            let mut port_ids = BTreeSet::new();
            for (name, p) in &d.ports {
                if !key(name) || !identity(&p.port_id) || !label(&p.description) {
                    return Err(error(
                        ErrorKind::Invalid,
                        "Invalid discovery port key, identity or description",
                    ));
                }
                if !port_ids.insert(&p.port_id) {
                    return Err(error(
                        ErrorKind::Identity,
                        "Duplicate port ID within a chassis",
                    ));
                }
            }
        }
        let mut ids = BTreeSet::new();
        let mut occupied = BTreeSet::new();
        for l in &self.links {
            if !key(&l.id) || !ids.insert(&l.id) || l.a.device == l.b.device {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid/duplicate link ID or same-device discovery link",
                ));
            }
            for e in [&l.a, &l.b] {
                self.port(e)?;
                if !occupied.insert(e) {
                    return Err(error(
                        ErrorKind::Invalid,
                        "A discovery port can attach to only one link",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Fixed authored identity/attachments; configuration and descriptive fields may differ.
    pub fn same_identity(&self, initial: &Self) -> bool {
        if self.validate().is_err()
            || initial.validate().is_err()
            || self.devices.len() != initial.devices.len()
            || self.links.len() != initial.links.len()
        {
            return false;
        }
        self.devices.iter().all(|(id, d)| {
            initial.devices.get(id).is_some_and(|old| {
                d.chassis_id == old.chassis_id
                    && d.ports.len() == old.ports.len()
                    && d.ports
                        .iter()
                        .all(|(p, v)| old.ports.get(p).is_some_and(|v0| v.port_id == v0.port_id))
            })
        }) && self.links.iter().all(|l| {
            initial.links.iter().any(|old| {
                l.id == old.id && ((l.a == old.a && l.b == old.b) || (l.a == old.b && l.b == old.a))
            })
        })
    }
    /// Exact configuration comparison with insignificant link/endpoint ordering.
    pub fn same_configuration(&self, expected: &Self) -> bool {
        self.same_identity(expected)
            && self.devices == expected.devices
            && self.links.iter().all(|link| {
                expected
                    .links
                    .iter()
                    .any(|other| link.id == other.id && link.up == other.up)
            })
    }
    fn port(&self, e: &Endpoint) -> Result<(&Device, &Port)> {
        let d = self
            .devices
            .get(&e.device)
            .ok_or_else(|| error(ErrorKind::Invalid, "Unknown discovery device"))?;
        let p = d
            .ports
            .get(&e.port)
            .ok_or_else(|| error(ErrorKind::Invalid, "Unknown discovery port"))?;
        Ok((d, p))
    }
    fn attachment(&self, e: &Endpoint) -> Option<(&Link, &Endpoint)> {
        self.links.iter().find_map(|l| {
            if &l.a == e {
                Some((l, &l.b))
            } else if &l.b == e {
                Some((l, &l.a))
            } else {
                None
            }
        })
    }
    fn tx_enabled(&self, e: &Endpoint) -> bool {
        self.port(e).is_ok_and(|(d, p)| d.enabled && p.transmit)
    }
    fn reception_active(&self, e: &Endpoint) -> bool {
        self.port(e)
            .is_ok_and(|(d, p)| d.enabled && p.receive && p.up)
            && self
                .attachment(e)
                .is_some_and(|(l, peer)| l.up && self.port(peer).is_ok_and(|(_, p)| p.up))
    }
    fn delivery(&self, from: &Endpoint) -> Result<Transmission> {
        let (d, p) = self.port(from)?;
        let attachment = self.attachment(from);
        let receiver = attachment.map(|(_, peer)| peer.clone());
        let outcome = if !d.enabled || !p.transmit {
            Outcome::TransmitDisabled
        } else if !p.up {
            Outcome::LinkDown
        } else if let Some((l, peer)) = attachment {
            let (rd, rp) = self.port(peer)?;
            if !l.up || !rp.up {
                Outcome::LinkDown
            } else if !rd.enabled || !rp.receive {
                Outcome::ReceiveDisabled
            } else {
                Outcome::Recorded
            }
        } else {
            Outcome::Unattached
        };
        Ok(Transmission {
            sender: from.clone(),
            receiver,
            outcome,
        })
    }
    fn advertisement(&self, from: &Endpoint) -> Result<Advertisement> {
        let (d, p) = self.port(from)?;
        Ok(Advertisement {
            chassis_id: d.chassis_id.clone(),
            port_id: p.port_id.clone(),
            ttl: d.ttl,
            system_name: d.system_name.clone(),
            port_description: p.description.clone(),
        })
    }
}
fn decode<T: for<'de> Deserialize<'de>>(text: &str, max_bytes: usize) -> Result<T> {
    if text.len() > max_bytes {
        return Err(error(
            ErrorKind::Limit,
            format!("Discovery JSON exceeds {max_bytes} bytes"),
        ));
    }
    serde_json::from_str(text)
        .map_err(|e| error(ErrorKind::Invalid, format!("Invalid discovery JSON: {e}")))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advertisement {
    pub chassis_id: String,
    pub port_id: String,
    pub ttl: u16,
    pub system_name: Option<String>,
    pub port_description: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Neighbor {
    pub local: Endpoint,
    pub peer: Endpoint,
    pub advertisement: Advertisement,
    pub received_at: u64,
    pub expires_at: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Recorded,
    Withdrawn,
    TransmitDisabled,
    ReceiveDisabled,
    LinkDown,
    Unattached,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transmission {
    pub sender: Endpoint,
    pub receiver: Option<Endpoint>,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborView {
    pub record: Neighbor,
    pub age_seconds: u64,
    pub remaining_seconds: u16,
    /// Current direct delivery is possible, independently of historical cache content.
    pub can_receive: bool,
    /// Simulator-only diagnostic; not an LLDP wire feature.
    pub matches_current: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    topology: Topology,
    now: u64,
    cache: Vec<Neighbor>,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            topology: Topology,
            now: u64,
            #[serde(deserialize_with = "neighbors")]
            cache: Vec<Neighbor>,
        }
        let saved = Saved::deserialize(d)?;
        let result = State {
            topology: saved.topology,
            now: saved.now,
            cache: saved.cache,
        };
        result.validate().map_err(D::Error::custom)?;
        Ok(result)
    }
}
impl State {
    pub fn new(topology: Topology) -> Result<Self> {
        topology.validate()?;
        Ok(Self {
            topology,
            now: 0,
            cache: Vec::new(),
        })
    }
    pub fn from_json(text: &str) -> Result<Self> {
        decode(text, MAX_STATE_BYTES)
    }
    pub fn topology(&self) -> &Topology {
        &self.topology
    }
    pub fn now(&self) -> u64 {
        self.now
    }
    pub fn validate(&self) -> Result<()> {
        self.topology.validate()?;
        if self.now > MAX_TIME || self.cache.len() > MAX_NEIGHBORS {
            return Err(error(
                ErrorKind::Limit,
                "Discovery clock/cache limit exceeded",
            ));
        }
        let mut seen = BTreeSet::new();
        for n in &self.cache {
            let (peer, port) = self.topology.port(&n.peer)?;
            let a = &n.advertisement;
            if !seen.insert(&n.local)
                || !self.topology.reception_active(&n.local)
                || !self
                    .topology
                    .attachment(&n.local)
                    .is_some_and(|(_, p)| p == &n.peer)
                || a.chassis_id != peer.chassis_id
                || a.port_id != port.port_id
                || a.ttl == 0
                || !label(&a.system_name)
                || !label(&a.port_description)
                || n.received_at > self.now
                || n.expires_at <= self.now
                || n.received_at.checked_add(u64::from(a.ttl)) != Some(n.expires_at)
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid discovery cache identity, attachment, timestamp or duplicate entry",
                ));
            }
        }
        Ok(())
    }
    /// Configuration changes never silently change authored identity or attachments.
    pub fn reconfigure(&mut self, next: Topology) -> Result<()> {
        next.validate()?;
        if !next.same_identity(&self.topology) {
            return Err(error(
                ErrorKind::Identity,
                "Discovery reconfiguration must preserve identities and attachments",
            ));
        }
        self.cache.retain(|n| {
            next.reception_active(&n.local)
                && !(self.topology.tx_enabled(&n.peer) && !next.tx_enabled(&n.peer))
        });
        self.topology = next;
        Ok(())
    }
    /// One explicit LLDP information advertisement; no automatic schedule is implied.
    pub fn advertise(&mut self, from: &Endpoint) -> Result<Transmission> {
        let result = self.topology.delivery(from)?;
        if result.outcome == Outcome::Recorded {
            let advertisement = self.topology.advertisement(from)?;
            // delivery guarantees a direct receiver; constructing records is bounded by ports.
            if let Some(local) = &result.receiver {
                let record = Neighbor {
                    local: local.clone(),
                    peer: from.clone(),
                    expires_at: self.now + u64::from(advertisement.ttl),
                    received_at: self.now,
                    advertisement,
                };
                self.cache.retain(|n| &n.local != local);
                self.cache.push(record);
                self.cache.sort_by(|a, b| a.local.cmp(&b.local));
            }
        }
        Ok(result)
    }
    /// A TTL-zero shutdown event, not a persistent configuration disable.
    pub fn shutdown(&mut self, from: &Endpoint) -> Result<Transmission> {
        let mut result = self.topology.delivery(from)?;
        if result.outcome == Outcome::Recorded {
            self.cache
                .retain(|n| Some(&n.local) != result.receiver.as_ref() || &n.peer != from);
            result.outcome = Outcome::Withdrawn;
        }
        Ok(result)
    }
    /// Ages only: time advancement does not schedule advertisements or contact a host clock.
    pub fn advance(&mut self, seconds: u64) -> Result<()> {
        let next = self
            .now
            .checked_add(seconds)
            .filter(|n| *n <= MAX_TIME)
            .ok_or_else(|| {
                error(
                    ErrorKind::Limit,
                    "Discovery simulated time exceeds 1000000000 seconds",
                )
            })?;
        self.now = next;
        self.cache.retain(|n| n.expires_at > next);
        Ok(())
    }
    /// Predicts the next advertisement's result without recording or refreshing a neighbor.
    pub fn delivery(&self, from: &Endpoint) -> Result<Transmission> {
        self.topology.delivery(from)
    }
    pub fn neighbor(&self, local: &Endpoint) -> Result<Option<NeighborView>> {
        self.topology.port(local)?;
        Ok(self
            .cache
            .iter()
            .find(|n| &n.local == local)
            .map(|n| self.view(n)))
    }
    pub fn neighbors(&self, device: &str) -> Result<Vec<NeighborView>> {
        if !self.topology.devices.contains_key(device) {
            return Err(error(ErrorKind::Invalid, "Unknown discovery device"));
        }
        Ok(self
            .cache
            .iter()
            .filter(|n| n.local.device == device)
            .map(|n| self.view(n))
            .collect())
    }
    fn view(&self, n: &Neighbor) -> NeighborView {
        let can_receive = self
            .topology
            .delivery(&n.peer)
            .is_ok_and(|r| r.outcome == Outcome::Recorded && r.receiver.as_ref() == Some(&n.local));
        NeighborView {
            record: n.clone(),
            age_seconds: self.now - n.received_at,
            remaining_seconds: (n.expires_at - self.now) as u16,
            can_receive,
            matches_current: can_receive
                && self
                    .topology
                    .advertisement(&n.peer)
                    .is_ok_and(|a| a == n.advertisement),
        }
    }
}

/// Device-wide settings, excluding fixed chassis identity and per-port configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalSettings {
    pub enabled: bool,
    pub ttl: u16,
    pub system_name: Option<String>,
}
impl GlobalSettings {
    pub fn from_device(device: &Device) -> Self {
        Self {
            enabled: device.enabled,
            ttl: device.ttl,
            system_name: device.system_name.clone(),
        }
    }
    fn validate(&self) -> Result<()> {
        if self.ttl == 0 || !label(&self.system_name) {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid expected discovery global settings",
            ));
        }
        Ok(())
    }
}

/// One bounded outcome; callers must bound their enclosing requirement collection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Requirement {
    CurrentNeighbor {
        local: Endpoint,
        peer: Endpoint,
    },
    AbsentNeighbor {
        local: Endpoint,
    },
    GlobalConfiguration {
        device: String,
        expected: GlobalSettings,
    },
    PortConfiguration {
        local: Endpoint,
        expected: Port,
    },
    Configuration {
        expected: Topology,
    },
}

/// Current-state result. This is neither an event-history proof nor LLDP wire data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Satisfied,
    IdentityChanged,
    ConfigurationMismatch,
    MissingCurrentNeighbor,
    WrongPeer,
    HistoricalNeighbor,
    UnexpectedNeighbor,
}
impl Check {
    pub fn passed(self) -> bool {
        self == Self::Satisfied
    }
}

impl Requirement {
    pub fn from_json(text: &str, initial: &Topology) -> Result<Self> {
        let result: Self = decode(text, MAX_BYTES)?;
        result.validate(initial)?;
        Ok(result)
    }
    /// Pins references and identities to the complete authored graph, not cached history.
    pub fn validate(&self, initial: &Topology) -> Result<()> {
        initial.validate()?;
        match self {
            Self::CurrentNeighbor { local, peer } => {
                initial.port(local)?;
                initial.port(peer)?;
                if !initial
                    .attachment(local)
                    .is_some_and(|(_, attached)| attached == peer)
                {
                    return Err(error(
                        ErrorKind::Identity,
                        "Expected discovery neighbor is not directly attached to the named local port",
                    ));
                }
            }
            Self::AbsentNeighbor { local } => {
                initial.port(local)?;
            }
            Self::GlobalConfiguration { device, expected } => {
                if !initial.devices.contains_key(device) {
                    return Err(error(
                        ErrorKind::Invalid,
                        "Unknown expected discovery device",
                    ));
                }
                expected.validate()?;
            }
            Self::PortConfiguration { local, expected } => {
                let (_, port) = initial.port(local)?;
                if expected.port_id != port.port_id {
                    return Err(error(
                        ErrorKind::Identity,
                        "Expected discovery port must preserve its authored port ID",
                    ));
                }
                if !label(&expected.description) {
                    return Err(error(
                        ErrorKind::Invalid,
                        "Invalid expected discovery port description",
                    ));
                }
            }
            Self::Configuration { expected } => {
                expected.validate()?;
                if !expected.same_identity(initial) {
                    return Err(error(
                        ErrorKind::Identity,
                        "Expected discovery configuration must preserve all authored identities and attachments",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Every invocation rechecks the current state. A prior pass is never retained.
    pub fn check(&self, initial: &Topology, state: &State) -> Result<Check> {
        self.validate(initial)?;
        state.validate()?;
        if !state.topology.same_identity(initial) {
            return Ok(Check::IdentityChanged);
        }
        Ok(match self {
            Self::CurrentNeighbor { local, peer } => match state.neighbor(local)? {
                None => Check::MissingCurrentNeighbor,
                Some(neighbor) if neighbor.record.peer != *peer => Check::WrongPeer,
                Some(neighbor)
                    if !neighbor.can_receive
                        || !neighbor.matches_current
                        || neighbor.remaining_seconds == 0 =>
                {
                    Check::HistoricalNeighbor
                }
                Some(_) => Check::Satisfied,
            },
            Self::AbsentNeighbor { local } => {
                if state.neighbor(local)?.is_none() {
                    Check::Satisfied
                } else {
                    Check::UnexpectedNeighbor
                }
            }
            Self::GlobalConfiguration { device, expected } => {
                if GlobalSettings::from_device(&state.topology.devices[device]) == *expected {
                    Check::Satisfied
                } else {
                    Check::ConfigurationMismatch
                }
            }
            Self::PortConfiguration { local, expected } => {
                if state.topology.port(local)?.1 == expected {
                    Check::Satisfied
                } else {
                    Check::ConfigurationMismatch
                }
            }
            Self::Configuration { expected } => {
                if state.topology.same_configuration(expected) {
                    Check::Satisfied
                } else {
                    Check::ConfigurationMismatch
                }
            }
        })
    }
}
