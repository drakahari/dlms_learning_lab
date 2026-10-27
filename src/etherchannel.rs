//! Bounded L2 LACP eligibility snapshots; no protocol, packets, timers or host I/O.
pub use crate::mac_learning::Mac;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::Write,
    marker::PhantomData,
};
pub const MAX_STATE_BYTES: usize = 256 * 1024;
pub const MAX_ACTION_BYTES: usize = 64 * 1024;
pub const MAX_REPORT_BYTES: usize = 128 * 1024;
pub const MAX_DEVICES: usize = 8;
pub const MAX_PORTS: usize = 8;
pub const MAX_GROUPS: usize = 16;
pub const MAX_CABLES: usize = 32;
pub const MAX_QUERIES: usize = 8;
pub const MAX_FAULTS: usize = 4;
pub const MAX_WORK: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Invalid,
    Limit,
    Context,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
pub type Result<T> = std::result::Result<T, Fault>;
fn require(ok: bool, kind: FaultKind, message: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Fault {
            kind,
            message: message.into(),
        })
    }
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_".contains(&c))
}
struct Id(String);
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl Visitor<'_> for V {
            type Value = Id;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a1..32-byte ASCII identity")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<Id, E> {
                if identifier(s) {
                    Ok(Id(s.into()))
                } else {
                    Err(E::custom("EtherChannel identity bound/syntax"))
                }
            }
        }
        d.deserialize_str(V)
    }
}
fn id<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    Ok(Id::deserialize(d)?.0)
}
struct Key {
    allowed: bool,
}
impl<'de> DeserializeSeed<'de> for Key {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("EtherChannel map limit"));
        }
        id(d)
    }
}
fn map<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, T>, D::Error> {
    struct V<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for V<T, N> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} unique bounded entries")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key_seed(Key {
                allowed: out.len() < N,
            })? {
                if out.contains_key(&k) {
                    return Err(A::Error::custom("Duplicate EtherChannel key"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(V::<T, N>(PhantomData))
}
struct Element<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Element<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("EtherChannel sequence limit"));
        }
        T::deserialize(d)
    }
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct V<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for V<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<Vec<T>, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Element {
                allowed: out.len() < N,
                marker: PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(V::<T, N>(PhantomData))
}
fn set<'de, D: Deserializer<'de>, T: Deserialize<'de> + Ord, const N: usize>(
    d: D,
) -> std::result::Result<BTreeSet<T>, D::Error> {
    let mut out = BTreeSet::new();
    for v in list::<D, T, N>(d)? {
        if !out.insert(v) {
            return Err(D::Error::custom("Duplicate EtherChannel set value"));
        }
    }
    Ok(out)
}
fn down<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeSet<String>, D::Error> {
    let mut out = BTreeSet::new();
    for Id(v) in list::<D, Id, 4>(d)? {
        if !out.insert(v) {
            return Err(D::Error::custom("Duplicate cable override"));
        }
    }
    Ok(out)
}
fn optional_id<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Option<String>, D::Error> {
    Ok(Option::<Id>::deserialize(d)?.map(|s| s.0))
}
fn within<T: Serialize + ?Sized>(value: &T, cap: usize) -> bool {
    struct Counter {
        used: usize,
        cap: usize,
    }
    impl Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.used = self
                .used
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("EtherChannel byte bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { used: 0, cap }, value).is_ok()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rate {
    R100,
    R1000,
    R10000,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Duplex {
    Full,
    Half,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LacpMode {
    Active,
    Passive,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Profile {
    Access {
        vlan: u16,
    },
    Trunk {
        native: u16,
        #[serde(deserialize_with = "set::<_,u16,64>")]
        allowed: BTreeSet<u16>,
    },
}
fn vlan(v: u16) -> bool {
    (1..=4094).contains(&v)
}
impl Profile {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Access { vlan: v } => {
                require(vlan(*v), FaultKind::Invalid, "Invalid access VLAN")
            }
            Self::Trunk { native, allowed } => {
                require(
                    !allowed.is_empty() && allowed.len() <= 64,
                    FaultKind::Limit,
                    "Allowed VLAN count bound",
                )?;
                require(
                    vlan(*native) && allowed.iter().all(|v| vlan(*v)),
                    FaultKind::Invalid,
                    "Invalid trunk VLAN",
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "id")]
    pub device: String,
    #[serde(deserialize_with = "id")]
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupRef {
    #[serde(deserialize_with = "id")]
    pub device: String,
    #[serde(deserialize_with = "id")]
    pub group: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupSlot {}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortSpec {
    #[serde(deserialize_with = "set::<_,Rate,3>")]
    pub rates: BTreeSet<Rate>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub mac: Mac,
    #[serde(deserialize_with = "map::<_,PortSpec,8>")]
    pub ports: BTreeMap<String, PortSpec>,
    #[serde(deserialize_with = "map::<_,GroupSlot,4>")]
    pub groups: BTreeMap<String, GroupSlot>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cable {
    pub a: Endpoint,
    pub b: Endpoint,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    #[serde(deserialize_with = "id")]
    pub fixture: String,
    #[serde(deserialize_with = "map::<_,Device,8>")]
    pub devices: BTreeMap<String, Device>,
    #[serde(deserialize_with = "map::<_,Cable,32>")]
    pub cables: BTreeMap<String, Cable>,
}
impl Topology {
    pub fn validate(&self) -> Result<()> {
        require(
            (2..=8).contains(&self.devices.len()) && self.cables.len() <= 32,
            FaultKind::Limit,
            "Topology collection bound",
        )?;
        require(
            identifier(&self.fixture),
            FaultKind::Invalid,
            "Invalid fixture identity",
        )?;
        let mut groups = 0usize;
        // Full borrowed shape preflight precedes derived identity/attachment sets.
        for (n, d) in &self.devices {
            require(
                !d.ports.is_empty() && d.ports.len() <= 8 && d.groups.len() <= 4,
                FaultKind::Limit,
                "Device collection bound",
            )?;
            groups += d.groups.len();
            require(
                identifier(n) && d.mac.unicast(),
                FaultKind::Invalid,
                "Invalid supplied device identity",
            )?;
            for (p, s) in &d.ports {
                require(
                    !s.rates.is_empty() && s.rates.len() <= 3,
                    FaultKind::Limit,
                    "Supported rate count",
                )?;
                require(identifier(p), FaultKind::Invalid, "Invalid supplied port")?;
            }
            require(
                d.groups.keys().all(|g| identifier(g)),
                FaultKind::Invalid,
                "Invalid local group slot",
            )?;
        }
        require(
            groups <= MAX_GROUPS,
            FaultKind::Limit,
            "Total group slot bound",
        )?;
        for (n, c) in &self.cables {
            require(
                identifier(n)
                    && identifier(&c.a.device)
                    && identifier(&c.a.port)
                    && identifier(&c.b.device)
                    && identifier(&c.b.port),
                FaultKind::Invalid,
                "Invalid cable identity",
            )?;
        }
        let mut macs = BTreeSet::new();
        let mut ends = BTreeSet::new();
        for d in self.devices.values() {
            require(
                macs.insert(d.mac),
                FaultKind::Context,
                "Duplicate system MAC",
            )?;
        }
        for c in self.cables.values() {
            require(
                c.a.device != c.b.device && self.has_port(&c.a) && self.has_port(&c.b),
                FaultKind::Context,
                "Unknown or self cable attachment",
            )?;
            require(
                ends.insert(&c.a) && ends.insert(&c.b),
                FaultKind::Context,
                "Physical jack reused",
            )?;
        }
        Ok(())
    }
    fn has_port(&self, e: &Endpoint) -> bool {
        self.devices
            .get(&e.device)
            .is_some_and(|d| d.ports.contains_key(&e.port))
    }
    fn has_group(&self, g: &GroupRef) -> bool {
        identifier(&g.device)
            && identifier(&g.group)
            && self
                .devices
                .get(&g.device)
                .is_some_and(|d| d.groups.contains_key(&g.group))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub rate: Rate,
    pub profile: Profile,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupConfig {
    pub up: bool,
    pub template: Template,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortConfig {
    pub up: bool,
    #[serde(deserialize_with = "optional_id")]
    pub group: Option<String>,
    pub mode: LacpMode,
    pub rate: Rate,
    pub duplex: Duplex,
    pub profile: Profile,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceConfig {
    #[serde(deserialize_with = "set::<_,u16,64>")]
    pub vlans: BTreeSet<u16>,
    #[serde(deserialize_with = "map::<_,GroupConfig,4>")]
    pub groups: BTreeMap<String, GroupConfig>,
    #[serde(deserialize_with = "map::<_,PortConfig,8>")]
    pub ports: BTreeMap<String, PortConfig>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "map::<_,DeviceConfig,8>")]
    pub devices: BTreeMap<String, DeviceConfig>,
}
impl Configuration {
    fn shape(&self) -> Result<()> {
        require(
            (2..=8).contains(&self.devices.len()),
            FaultKind::Limit,
            "Configuration device count",
        )?;
        let mut groups = 0;
        for (n, d) in &self.devices {
            require(
                d.ports.len() <= 8 && d.groups.len() <= 4 && d.vlans.len() <= 64,
                FaultKind::Limit,
                "Configuration nested count",
            )?;
            groups += d.groups.len();
            require(
                identifier(n),
                FaultKind::Invalid,
                "Invalid configuration device",
            )?;
            require(
                d.vlans.iter().all(|v| vlan(*v)),
                FaultKind::Invalid,
                "Invalid active VLAN",
            )?;
            for (n, g) in &d.groups {
                require(
                    identifier(n),
                    FaultKind::Invalid,
                    "Invalid configured group",
                )?;
                g.template.profile.validate()?;
            }
            for (n, p) in &d.ports {
                require(
                    identifier(n) && p.group.as_ref().is_none_or(|g| identifier(g)),
                    FaultKind::Invalid,
                    "Invalid configured port/group",
                )?;
                p.profile.validate()?;
            }
        }
        require(groups <= 16, FaultKind::Limit, "Configured group count")
    }
    pub fn validate_for(&self, t: &Topology) -> Result<()> {
        self.shape()?;
        t.validate()?;
        require(
            self.devices.keys().eq(t.devices.keys()),
            FaultKind::Context,
            "Configuration device keyset differs",
        )?;
        for (n, d) in &self.devices {
            let td = &t.devices[n];
            require(
                d.groups.keys().eq(td.groups.keys()) && d.ports.keys().eq(td.ports.keys()),
                FaultKind::Context,
                "Configuration port/group keyset differs",
            )?;
            for (p, c) in &d.ports {
                require(
                    td.ports[p].rates.contains(&c.rate)
                        && c.group.as_ref().is_none_or(|g| d.groups.contains_key(g)),
                    FaultKind::Context,
                    "Unsupported supplied port rate or foreign group",
                )?;
            }
            // At most8 physical ports exist, so no group can have more than8 members.
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(deserialize_with = "map::<_,bool,32>")]
    pub cables: BTreeMap<String, bool>,
}
impl Environment {
    fn validate_for(&self, t: &Topology) -> Result<()> {
        require(
            self.cables.len() <= 32,
            FaultKind::Limit,
            "Environment cable count",
        )?;
        require(
            self.cables.keys().all(|n| identifier(n)) && self.cables.keys().eq(t.cables.keys()),
            FaultKind::Context,
            "Environment cable identities differ",
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    schema_version: u32,
    topology: Topology,
    configuration: Configuration,
    environment: Environment,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema_version: u32,
            topology: Topology,
            configuration: Configuration,
            environment: Environment,
        }
        let r = Raw::deserialize(d)?;
        if r.schema_version != 1 {
            return Err(D::Error::custom("Unsupported EtherChannel State version"));
        }
        Self::new(r.topology, r.configuration, r.environment)
            .map_err(|_| D::Error::custom("Invalid bounded EtherChannel State"))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Configure { configuration: Configuration },
}
impl Action {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        require(
            raw.len() <= MAX_ACTION_BYTES,
            FaultKind::Limit,
            "Action raw cap before decode",
        )?;
        let a: Self = serde_json::from_slice(raw).map_err(|_| Fault {
            kind: FaultKind::Invalid,
            message: "Invalid bounded EtherChannel Action".into(),
        })?;
        match &a {
            Self::Configure { configuration } => configuration.shape()?,
        };
        require(
            within(&a, MAX_ACTION_BYTES),
            FaultKind::Limit,
            "Action encoded bound",
        )?;
        Ok(a)
    }
}
impl State {
    pub fn new(
        topology: Topology,
        configuration: Configuration,
        environment: Environment,
    ) -> Result<Self> {
        let s = Self {
            schema_version: 1,
            topology,
            configuration,
            environment,
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
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == 1,
            FaultKind::Invalid,
            "Unsupported State version",
        )?;
        self.configuration.validate_for(&self.topology)?;
        self.environment.validate_for(&self.topology)?;
        require(
            within(self, MAX_STATE_BYTES),
            FaultKind::Limit,
            "Encoded State exceeds256KiB",
        )
    }
    pub fn validate_context(&self, expected: &Topology) -> Result<()> {
        expected.validate()?;
        self.validate()?;
        require(
            &self.topology == expected,
            FaultKind::Context,
            "Immutable topology/system identities differ",
        )
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        require(
            raw.len() <= MAX_STATE_BYTES,
            FaultKind::Limit,
            "State raw cap before decode",
        )?;
        serde_json::from_slice(raw).map_err(|_| Fault {
            kind: FaultKind::Invalid,
            message: "Invalid bounded EtherChannel State JSON".into(),
        })
    }
    pub fn reconfigure(&mut self, c: Configuration) -> Result<()> {
        self.validate()?;
        c.validate_for(&self.topology)?;
        #[derive(Serialize)]
        struct Proposed<'a> {
            schema_version: u32,
            topology: &'a Topology,
            configuration: &'a Configuration,
            environment: &'a Environment,
        }
        require(
            within(
                &Proposed {
                    schema_version: 1,
                    topology: &self.topology,
                    configuration: &c,
                    environment: &self.environment,
                },
                MAX_STATE_BYTES,
            ),
            FaultKind::Limit,
            "Proposed State bound",
        )?;
        self.configuration = c;
        Ok(())
    }
    pub fn apply(&mut self, a: &Action) -> Result<()> {
        match a {
            Action::Configure { configuration } => {
                self.validate()?;
                configuration.validate_for(&self.topology)?;
                require(
                    within(a, MAX_ACTION_BYTES),
                    FaultKind::Limit,
                    "Direct Action bound before clone",
                )?;
                self.reconfigure(configuration.clone())
            }
        }
    }
    pub fn snapshot(&self) -> Result<Snapshot<'_>> {
        self.validate()?;
        let s = self.calculate(&BTreeSet::new());
        require(
            within(&s, MAX_REPORT_BYTES),
            FaultKind::Limit,
            "Snapshot output cap",
        )?;
        Ok(s)
    }
    pub fn evaluate<'a>(&'a self, plan: &Evaluation) -> Result<Report<'a>> {
        self.validate()?;
        plan.validate_for(&self.topology)?;
        // Each physical cable visited once/snapshot. Queries consume those results, not new snapshots.
        let visits = self
            .topology
            .cables
            .len()
            .checked_mul(1 + plan.faults.len())
            .ok_or(Fault {
                kind: FaultKind::Limit,
                message: "Snapshot work overflow".into(),
            })?;
        let work = visits
            .checked_add(plan.queries.len() * (1 + plan.faults.len()))
            .ok_or(Fault {
                kind: FaultKind::Limit,
                message: "Query work overflow".into(),
            })?;
        require(
            work <= MAX_WORK && visits <= 160,
            FaultKind::Limit,
            "Evaluation work bound before snapshots",
        )?;
        let mut snapshots = Vec::with_capacity(1 + plan.faults.len());
        snapshots.push(self.calculate(&BTreeSet::new()));
        for f in &plan.faults {
            snapshots.push(self.calculate(&f.down));
        }
        let mut services = Vec::with_capacity(snapshots.len());
        for s in &snapshots {
            services.push(plan.queries.iter().map(|q| s.admission(q)).collect());
        }
        let report = Report {
            snapshots,
            services,
            cable_visits: visits,
            work,
        };
        require(
            within(&report, MAX_REPORT_BYTES),
            FaultKind::Limit,
            "Complete report output cap",
        )?;
        Ok(report)
    }
    fn calculate(&self, down: &BTreeSet<String>) -> Snapshot<'_> {
        let mut buckets: BTreeMap<GroupIdentity<'_>, Bucket<'_>> = BTreeMap::new();
        for (dn, d) in &self.configuration.devices {
            for (gn, g) in &d.groups {
                buckets.insert(
                    GroupIdentity {
                        device: dn,
                        group: gn,
                    },
                    Bucket {
                        config: g,
                        vlans: &d.vlans,
                        members: Vec::new(),
                        candidates: Vec::new(),
                        neighbors: BTreeSet::new(),
                    },
                );
            }
        }
        for (dn, d) in &self.configuration.devices {
            for (pn, p) in &d.ports {
                if let Some(g) = &p.group {
                    buckets
                        .get_mut(&GroupIdentity {
                            device: dn,
                            group: g,
                        })
                        .expect("validated group")
                        .members
                        .push(pn.as_str());
                }
            }
        }
        let mut links = Vec::with_capacity(self.topology.cables.len());
        for (n, c) in &self.topology.cables {
            let pa = &self.configuration.devices[&c.a.device].ports[&c.a.port];
            let pb = &self.configuration.devices[&c.b.device].ports[&c.b.port];
            let mut reasons = Vec::new();
            if !self.environment.cables[n] || down.contains(n) {
                reasons.push(Exclusion::CableDown)
            }
            if !pa.up {
                reasons.push(Exclusion::PortDownA)
            }
            if !pb.up {
                reasons.push(Exclusion::PortDownB)
            }
            let a = pa.group.as_ref().map(|g| GroupIdentity {
                device: c.a.device.as_str(),
                group: g.as_str(),
            });
            let b = pb.group.as_ref().map(|g| GroupIdentity {
                device: c.b.device.as_str(),
                group: g.as_str(),
            });
            for (key, p, is_a) in [(a, pa, true), (b, pb, false)] {
                if let Some(key) = key {
                    let group = buckets[&key].config;
                    if !group.up {
                        reasons.push(if is_a {
                            Exclusion::GroupDownA
                        } else {
                            Exclusion::GroupDownB
                        });
                    }
                    if p.rate != group.template.rate || p.profile != group.template.profile {
                        reasons.push(if is_a {
                            Exclusion::TemplateMismatchA
                        } else {
                            Exclusion::TemplateMismatchB
                        });
                    }
                } else {
                    reasons.push(if is_a {
                        Exclusion::NoMembershipA
                    } else {
                        Exclusion::NoMembershipB
                    });
                }
            }
            if pa.duplex != Duplex::Full || pb.duplex != Duplex::Full {
                reasons.push(Exclusion::HalfDuplex)
            }
            if pa.rate != pb.rate {
                reasons.push(Exclusion::LineRateMismatch)
            }
            if pa.mode == LacpMode::Passive && pb.mode == LacpMode::Passive {
                reasons.push(Exclusion::BothPassive)
            }
            let eligible = reasons.is_empty();
            if eligible {
                let a = a.expect("eligible assigned end");
                let b = b.expect("eligible assigned end");
                let left = buckets.get_mut(&a).unwrap();
                left.candidates.push(n.as_str());
                left.neighbors.insert(b);
                let right = buckets.get_mut(&b).unwrap();
                right.candidates.push(n.as_str());
                right.neighbors.insert(a);
            }
            links.push(LinkView {
                cable: n,
                a,
                b,
                eligible,
                participating: false,
                reasons,
            });
        }
        // Only live, locally compatible, initiating pairs entered neighbors. No disabled/malformed
        // noncandidate can poison an otherwise unambiguous healthy aggregation.
        let groups = buckets
            .iter()
            .map(|(key, b)| {
                let status = if b.neighbors.is_empty() {
                    GroupStatus::Unavailable {}
                } else if b.neighbors.len() != 1 {
                    GroupStatus::UnsupportedPartnerSelection {}
                } else {
                    let peer = *b.neighbors.first().unwrap();
                    let remote = &buckets[&peer];
                    if remote.neighbors.len() == 1 && remote.neighbors.contains(key) {
                        GroupStatus::Ready { partner: peer }
                    } else {
                        GroupStatus::UnsupportedPartnerSelection {}
                    }
                };
                GroupView {
                    identity: *key,
                    configured_members: b.members.clone(),
                    eligible_members: b.candidates.clone(),
                    status,
                    profile: &b.config.template.profile,
                    active_vlans: b.vlans,
                }
            })
            .collect::<Vec<_>>();
        let statuses = groups
            .iter()
            .map(|g| (g.identity, g.status))
            .collect::<BTreeMap<_, _>>();
        for l in &mut links {
            if l.eligible {
                let a = l.a.unwrap();
                let b = l.b.unwrap();
                l.participating = matches!(statuses[&a], GroupStatus::Ready { partner } if partner == b)
                    && matches!(statuses[&b], GroupStatus::Ready { partner } if partner == a);
                if !l.participating {
                    l.reasons.push(Exclusion::UnsupportedPartnerSelection)
                }
            }
        }
        Snapshot { groups, links }
    }
}
struct Bucket<'a> {
    config: &'a GroupConfig,
    vlans: &'a BTreeSet<u16>,
    members: Vec<&'a str>,
    candidates: Vec<&'a str>,
    neighbors: BTreeSet<GroupIdentity<'a>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GroupIdentity<'a> {
    pub device: &'a str,
    pub group: &'a str,
}
impl GroupIdentity<'_> {
    fn matches(&self, r: &GroupRef) -> bool {
        self.device == r.device && self.group == r.group
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Exclusion {
    CableDown,
    PortDownA,
    PortDownB,
    NoMembershipA,
    NoMembershipB,
    GroupDownA,
    GroupDownB,
    TemplateMismatchA,
    TemplateMismatchB,
    HalfDuplex,
    LineRateMismatch,
    BothPassive,
    UnsupportedPartnerSelection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum GroupStatus<'a> {
    Unavailable {},
    Ready { partner: GroupIdentity<'a> },
    UnsupportedPartnerSelection {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GroupView<'a> {
    pub identity: GroupIdentity<'a>,
    pub configured_members: Vec<&'a str>,
    pub eligible_members: Vec<&'a str>,
    pub status: GroupStatus<'a>,
    pub profile: &'a Profile,
    pub active_vlans: &'a BTreeSet<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LinkView<'a> {
    pub cable: &'a str,
    pub a: Option<GroupIdentity<'a>>,
    pub b: Option<GroupIdentity<'a>>,
    pub eligible: bool,
    pub participating: bool,
    pub reasons: Vec<Exclusion>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot<'a> {
    groups: Vec<GroupView<'a>>,
    links: Vec<LinkView<'a>>,
}
impl<'a> Snapshot<'a> {
    pub fn groups(&self) -> &[GroupView<'a>] {
        &self.groups
    }
    pub fn links(&self) -> &[LinkView<'a>] {
        &self.links
    }
    pub fn group(&self, r: &GroupRef) -> Option<&GroupView<'a>> {
        self.groups.iter().find(|g| g.identity.matches(r))
    }
    fn admission(&self, q: &ServiceQuery) -> Admission {
        let a = self.group(&q.a).expect("admitted query group");
        let b = self.group(&q.b).expect("admitted query group");
        if matches!(a.status, GroupStatus::UnsupportedPartnerSelection {})
            || matches!(b.status, GroupStatus::UnsupportedPartnerSelection {})
        {
            return Admission::Unsupported {
                reason: UnsupportedService::PartnerSelection,
            };
        }
        if !matches!(a.status,GroupStatus::Ready{partner}if partner==b.identity)
            || !matches!(b.status,GroupStatus::Ready{partner}if partner==a.identity)
        {
            return Admission::Unavailable {
                reason: ServiceFailure::NoReadyPair,
            };
        }
        // A reciprocal Ready pair has the same candidate cable set at both ends.
        // Queries do not traverse physical links or compute another snapshot.
        debug_assert_eq!(a.eligible_members, b.eligible_members);
        let members = a.eligible_members.len();
        if members < q.minimum_members as usize {
            return Admission::Unavailable {
                reason: ServiceFailure::TooFewMembers,
            };
        }
        if !a.active_vlans.contains(&q.vlan) || !b.active_vlans.contains(&q.vlan) {
            return Admission::Unavailable {
                reason: ServiceFailure::InactiveVlan,
            };
        }
        match (a.profile, b.profile) {
            (Profile::Access { vlan: x }, Profile::Access { vlan: y })
                if *x == q.vlan && *y == q.vlan =>
            {
                Admission::Ready { members }
            }
            (
                Profile::Trunk {
                    native: x,
                    allowed: av,
                },
                Profile::Trunk {
                    native: y,
                    allowed: bv,
                },
            ) => {
                if x != y {
                    Admission::Unsupported {
                        reason: UnsupportedService::NativeMismatch,
                    }
                } else if av.contains(&q.vlan) && bv.contains(&q.vlan) {
                    Admission::Ready { members }
                } else {
                    Admission::Unavailable {
                        reason: ServiceFailure::VlanNotAllowed,
                    }
                }
            }
            (Profile::Access { .. }, Profile::Access { .. }) => Admission::Unavailable {
                reason: ServiceFailure::VlanNotAllowed,
            },
            _ => Admission::Unsupported {
                reason: UnsupportedService::AccessTrunkMismatch,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ServiceFailure {
    NoReadyPair,
    TooFewMembers,
    InactiveVlan,
    VlanNotAllowed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum UnsupportedService {
    PartnerSelection,
    NativeMismatch,
    AccessTrunkMismatch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum Admission {
    Ready { members: usize },
    Unavailable { reason: ServiceFailure },
    Unsupported { reason: UnsupportedService },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceQuery {
    pub a: GroupRef,
    pub b: GroupRef,
    pub vlan: u16,
    pub minimum_members: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaultCondition {
    #[serde(deserialize_with = "down")]
    pub down: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    #[serde(deserialize_with = "list::<_,ServiceQuery,8>")]
    pub queries: Vec<ServiceQuery>,
    #[serde(deserialize_with = "list::<_,FaultCondition,4>")]
    pub faults: Vec<FaultCondition>,
}
impl Evaluation {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        require(
            raw.len() <= MAX_ACTION_BYTES,
            FaultKind::Limit,
            "Evaluation raw cap before decode",
        )?;
        let q: Self = serde_json::from_slice(raw).map_err(|_| Fault {
            kind: FaultKind::Invalid,
            message: "Invalid bounded EtherChannel evaluation".into(),
        })?;
        q.shape()?;
        Ok(q)
    }
    fn shape(&self) -> Result<()> {
        require(
            self.queries.len() <= 8 && self.faults.len() <= 4,
            FaultKind::Limit,
            "Evaluation collection limit before work",
        )?;
        for q in &self.queries {
            require(
                identifier(&q.a.device)
                    && identifier(&q.a.group)
                    && identifier(&q.b.device)
                    && identifier(&q.b.group)
                    && q.a.device != q.b.device
                    && vlan(q.vlan)
                    && (1..=8).contains(&q.minimum_members),
                FaultKind::Invalid,
                "Invalid positive VLAN admission query",
            )?;
        }
        for f in &self.faults {
            require(f.down.len() <= 4, FaultKind::Limit, "Fault override count")?;
            require(
                f.down.iter().all(|s| identifier(s)),
                FaultKind::Invalid,
                "Invalid cable override identity",
            )?;
        }
        require(
            within(self, MAX_ACTION_BYTES),
            FaultKind::Limit,
            "Evaluation compact bound",
        )
    }
    pub fn validate_for(&self, t: &Topology) -> Result<()> {
        self.shape()?;
        t.validate()?;
        for q in &self.queries {
            require(
                t.has_group(&q.a) && t.has_group(&q.b),
                FaultKind::Context,
                "Query names unknown group",
            )?;
        }
        for f in &self.faults {
            require(
                f.down.iter().all(|c| t.cables.contains_key(c)),
                FaultKind::Context,
                "Override names unknown cable",
            )?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report<'a> {
    pub snapshots: Vec<Snapshot<'a>>,
    pub services: Vec<Vec<Admission>>,
    pub cable_visits: usize,
    pub work: usize,
}
