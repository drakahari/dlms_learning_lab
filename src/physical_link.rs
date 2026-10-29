//! Supplied-profile physical-uplink model. No host/network/clock access.
//! Qualification is not an optical/electrical conformance measurement.
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};
pub const MAX_PORTS: usize = 24;
pub const MAX_LINKS: usize = 12;
pub const MAX_CABLES: usize = 24;
pub const MAX_TOPOLOGY: usize = 131_072;
pub const MAX_STATE: usize = 524_288;
pub const MAX_ACTION: usize = 32_768;
pub const MAX_CONFIGURATION: usize = 16_384;
pub const MAX_READING: usize = 8_192;
pub const MAX_COUNTER: u64 = 1_000_000_000;
pub const MAX_PROBE: u32 = 1_024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
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
fn fail(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 120 && !s.chars().any(char::is_control)
}
struct Key(String);
impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct K;
        impl Visitor<'_> for K {
            type Value = Key;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a bounded uplink identifier")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<Key, E> {
                if id(s) {
                    Ok(Key(s.into()))
                } else {
                    Err(E::custom("Invalid uplink identifier before body"))
                }
            }
        }
        d.deserialize_str(K)
    }
}
fn identity<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    Key::deserialize(d).map(|Key(value)| value)
}
fn label<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    struct Label;
    impl Visitor<'_> for Label {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("nonempty control-free text up to120 bytes")
        }
        fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<String, E> {
            if !text(value) {
                return Err(E::custom("Invalid uplink label before owned field copy"));
            }
            Ok(value.into())
        }
    }
    d.deserialize_str(Label)
}
fn identity_map<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    // Keys AND values use the same bounded identity visitor. No intermediary
    // String value can grow without its identity bound in an embedded decode.
    map::<D, Key, N>(d).map(|values| values.into_iter().map(|(k, Key(v))| (k, v)).collect())
}
fn map<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, T>, D::Error> {
    struct M<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for M<T, N> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} unique uplink keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Key(k)) = a.next_key::<Key>()? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(de::Error::custom("Uplink map limit/duplicate before value"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<T, N>(PhantomData))
}
fn set<'de, D: serde::Deserializer<'de>, T: Deserialize<'de> + Ord, const N: usize>(
    d: D,
) -> std::result::Result<BTreeSet<T>, D::Error> {
    struct S<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de> + Ord, const N: usize> Visitor<'de> for S<T, N> {
        type Value = BTreeSet<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} unique capabilities")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            struct Next<T> {
                allowed: bool,
                marker: PhantomData<T>,
            }
            impl<'de, T: Deserialize<'de>> de::DeserializeSeed<'de> for Next<T> {
                type Value = T;
                fn deserialize<D: serde::Deserializer<'de>>(
                    self,
                    d: D,
                ) -> std::result::Result<T, D::Error> {
                    if !self.allowed {
                        return Err(de::Error::custom("Capability limit before body"));
                    }
                    T::deserialize(d)
                }
            }
            let mut out = BTreeSet::new();
            while let Some(v) = a.next_element_seed(Next::<T> {
                allowed: out.len() < N,
                marker: PhantomData,
            })? {
                if !out.insert(v) {
                    return Err(de::Error::custom("Duplicate capability"));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_seq(S::<T, N>(PhantomData))
}
fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    struct Count {
        n: usize,
        cap: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Uplink encoded limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    crate::native_observations::physical_json();
    serde_json::to_writer(Count { n: 0, cap }, v).is_ok()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rate {
    M10,
    M100,
    M1000,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Duplex {
    Half,
    Full,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub rate: Rate,
    pub duplex: Duplex,
}
impl Ability {
    pub fn valid(self) -> bool {
        self.rate != Rate::M1000 || self.duplex == Duplex::Full
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Profile {
    CopperRj45,
    SxOm2,
    LxLhOs2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CableKind {
    Cat5e,
    Om2Duplex,
    Os2Duplex,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum Condition {
    Clean,
    Disconnected,
    /// Deterministic injected bad receive frames, not predicted BER or real CRC bytes.
    ReceiveImpairment {
        every: u16,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cable {
    #[serde(deserialize_with = "label")]
    pub label: String,
    pub kind: CableKind,
    pub length_m: u32,
    pub condition: Condition,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortSpec {
    #[serde(deserialize_with = "identity")]
    pub device: String,
    #[serde(deserialize_with = "label")]
    pub label: String,
    #[serde(deserialize_with = "set::<_,Profile,3>")]
    pub profiles: BTreeSet<Profile>,
    #[serde(deserialize_with = "set::<_,Ability,5>")]
    pub abilities: BTreeSet<Ability>,
    /// Explicit legacy10/100 parallel detection capability; never assumed for optics.
    pub parallel_detection: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbePattern {
    Serialized,
    DuplexMismatchContention,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkSpec {
    #[serde(deserialize_with = "identity")]
    pub a: String,
    #[serde(deserialize_with = "identity")]
    pub b: String,
    /// Immutable supplied route requirement, not an inferred signal measurement.
    pub required_span_m: u32,
    pub pattern: ProbePattern,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    pub version: u32,
    #[serde(deserialize_with = "map::<_,PortSpec,MAX_PORTS>")]
    pub ports: BTreeMap<String, PortSpec>,
    #[serde(deserialize_with = "map::<_,Cable,MAX_CABLES>")]
    pub cables: BTreeMap<String, Cable>,
    #[serde(deserialize_with = "map::<_,LinkSpec,MAX_LINKS>")]
    pub links: BTreeMap<String, LinkSpec>,
}
impl Topology {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_TOPOLOGY {
            return Err(fail(ErrorKind::Limit, "Topology exceeds128KiB"));
        }
        let t: Self = serde_json::from_slice(raw).map_err(|e| {
            fail(
                ErrorKind::Invalid,
                format!("Invalid physical-uplink topology: {e}"),
            )
        })?;
        t.validate()?;
        Ok(t)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.ports.is_empty()
            || self.ports.len() > MAX_PORTS
            || self.links.is_empty()
            || self.links.len() > MAX_LINKS
            || self.cables.is_empty()
            || self.cables.len() > MAX_CABLES
            || !within(self, MAX_TOPOLOGY)
        {
            return Err(fail(
                ErrorKind::Limit,
                "Topology version/cardinality/encoded bound",
            ));
        }
        for (name, p) in &self.ports {
            if !id(name)
                || !id(&p.device)
                || !text(&p.label)
                || p.profiles.is_empty()
                || p.profiles.len() > 3
                || p.abilities.is_empty()
                || p.abilities.len() > 5
                || p.abilities.iter().any(|a| !a.valid())
            {
                return Err(fail(ErrorKind::Invalid, "Invalid port capabilities"));
            }
            if p.profiles.iter().any(|p| *p != Profile::CopperRj45)
                && !p.abilities.contains(&Ability {
                    rate: Rate::M1000,
                    duplex: Duplex::Full,
                })
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Fiber profile requires1G/full capability",
                ));
            }
            if p.parallel_detection
                && (!p.profiles.contains(&Profile::CopperRj45)
                    || ![Rate::M10, Rate::M100].iter().any(|rate| {
                        p.abilities.contains(&Ability {
                            rate: *rate,
                            duplex: Duplex::Half,
                        })
                    }))
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Parallel detection requires an explicit copper half-duplex capability",
                ));
            }
        }
        for (name, c) in &self.cables {
            if !id(name)
                || !text(&c.label)
                || c.length_m == 0
                || c.length_m > 100_000
                || matches!(c.condition, Condition::ReceiveImpairment { every: 0 })
            {
                return Err(fail(ErrorKind::Invalid, "Invalid supplied cable"));
            }
        }
        let mut used = BTreeSet::new();
        for (name, l) in &self.links {
            if !id(name)
                || l.required_span_m == 0
                || l.required_span_m > 100_000
                || l.a == l.b
                || !self.ports.contains_key(&l.a)
                || !self.ports.contains_key(&l.b)
                || !used.insert(&l.a)
                || !used.insert(&l.b)
            {
                return Err(fail(
                    ErrorKind::Identity,
                    "Point-to-point endpoint missing/reused; shared segments are not implemented",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum Negotiation {
    Auto { advertise: BTreeSet<Ability> },
    Fixed { ability: Ability },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortConfig {
    pub admin_up: bool,
    pub profile: Profile,
    pub negotiation: Negotiation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "map::<_,PortConfig,MAX_PORTS>")]
    pub ports: BTreeMap<String, PortConfig>,
    #[serde(deserialize_with = "identity_map::<_,MAX_LINKS>")]
    pub cables: BTreeMap<String, String>,
}
fn config_port(p: &PortSpec, c: &PortConfig) -> Result<()> {
    if !p.profiles.contains(&c.profile) {
        return Err(fail(
            ErrorKind::Unsupported,
            "This supplied port does not accept that interface profile",
        ));
    }
    match &c.negotiation {
        Negotiation::Auto { advertise } => {
            if c.profile != Profile::CopperRj45 {
                return Err(fail(
                    ErrorKind::Unsupported,
                    "Optical negotiation is outside this model; use the supplied fixed1G/full profile",
                ));
            }
            if advertise.is_empty() || advertise.len() > 5 || !advertise.is_subset(&p.abilities) {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Advertised ability is outside the supplied port capabilities",
                ));
            }
        }
        Negotiation::Fixed { ability } => {
            if !ability.valid() || !p.abilities.contains(ability) {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Fixed ability is outside port capabilities",
                ));
            }
            if c.profile == Profile::CopperRj45 && ability.rate == Rate::M1000 {
                return Err(fail(
                    ErrorKind::Unsupported,
                    "Forced1000BASE-T negotiation is not implemented; use Auto for this profile",
                ));
            }
            if c.profile != Profile::CopperRj45
                && *ability
                    != (Ability {
                        rate: Rate::M1000,
                        duplex: Duplex::Full,
                    })
            {
                return Err(fail(
                    ErrorKind::Unsupported,
                    "Supplied optical profiles use1G/full only",
                ));
            }
        }
    }
    Ok(())
}
impl Configuration {
    pub fn validate(&self, t: &Topology) -> Result<()> {
        if self.ports.len() > MAX_PORTS
            || self.cables.len() > MAX_LINKS
            || !within(self, MAX_CONFIGURATION)
        {
            return Err(fail(
                ErrorKind::Limit,
                "Whole uplink configuration exceeds its16KiB/cardinality allowance",
            ));
        }
        if !self.ports.keys().eq(t.ports.keys()) || !self.cables.keys().eq(t.links.keys()) {
            return Err(fail(
                ErrorKind::Identity,
                "Configuration needs every exact supplied port/link",
            ));
        }
        for (id, c) in &self.ports {
            config_port(&t.ports[id], c)?;
        }
        let mut used = BTreeSet::new();
        for cable in self.cables.values() {
            if !t.cables.contains_key(cable) || !used.insert(cable) {
                return Err(fail(
                    ErrorKind::Identity,
                    "Cable missing or assigned to more than one link",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Qualification {
    Qualified,
    TooShortForRoute,
    IncompatibleProfile,
    IncompatibleCable,
    OutsideReach,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkState {
    Up,
    DegradedDuplex,
    AdministrativelyDown,
    Disconnected,
    NoCommonAbility,
    OutsideModel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    Advertised,
    Fixed,
    ParallelDetected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operational {
    pub ability: Ability,
    pub resolution: Resolution,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkReport {
    pub qualification: Qualification,
    pub state: LinkState,
    pub a: Option<Operational>,
    pub b: Option<Operational>,
}
fn qualify(a: Profile, b: Profile, c: &Cable, required_span_m: u32) -> Qualification {
    if a != b {
        return Qualification::IncompatibleProfile;
    }
    let (kind, min, max) = match a {
        Profile::CopperRj45 => (CableKind::Cat5e, 1, 100),
        Profile::SxOm2 => (CableKind::Om2Duplex, 2, 550),
        Profile::LxLhOs2 => (CableKind::Os2Duplex, 2, 10_000),
    };
    if c.kind != kind {
        Qualification::IncompatibleCable
    } else if c.length_m < required_span_m {
        Qualification::TooShortForRoute
    } else if !(min..=max).contains(&c.length_m) {
        Qualification::OutsideReach
    } else {
        Qualification::Qualified
    }
}
fn resolve(
    a: &PortConfig,
    b: &PortConfig,
    sa: &PortSpec,
    sb: &PortSpec,
) -> Result<Option<(Operational, Operational)>> {
    let op = |ability, resolution| Operational {
        ability,
        resolution,
    };
    match (&a.negotiation, &b.negotiation) {
        (Negotiation::Auto { advertise: aa }, Negotiation::Auto { advertise: bb }) => Ok(aa
            .intersection(bb)
            .max()
            .copied()
            .map(|x| (op(x, Resolution::Advertised), op(x, Resolution::Advertised)))),
        (Negotiation::Fixed { ability: aa }, Negotiation::Fixed { ability: bb }) => Ok((aa.rate
            == bb.rate)
            .then_some((op(*aa, Resolution::Fixed), op(*bb, Resolution::Fixed)))),
        (Negotiation::Auto { advertise }, Negotiation::Fixed { ability }) => {
            parallel(advertise, *ability, sa).map(|x| {
                x.map(|x| {
                    (
                        op(x, Resolution::ParallelDetected),
                        op(*ability, Resolution::Fixed),
                    )
                })
            })
        }
        (Negotiation::Fixed { ability }, Negotiation::Auto { advertise }) => {
            parallel(advertise, *ability, sb).map(|x| {
                x.map(|x| {
                    (
                        op(*ability, Resolution::Fixed),
                        op(x, Resolution::ParallelDetected),
                    )
                })
            })
        }
    }
}
fn parallel(
    advertise: &BTreeSet<Ability>,
    fixed: Ability,
    spec: &PortSpec,
) -> Result<Option<Ability>> {
    if advertise != &spec.abilities {
        return Err(fail(
            ErrorKind::Unsupported,
            "Restricted-advertisement parallel detection is outside this subset",
        ));
    }
    if fixed.rate == Rate::M1000 || !spec.parallel_detection {
        return Err(fail(
            ErrorKind::Unsupported,
            "Mixed Auto/Fixed resolution needs this model's explicit legacy10/100 parallel detection capability",
        ));
    }
    if !advertise.iter().any(|a| a.rate == fixed.rate) {
        return Ok(None);
    }
    let half = Ability {
        rate: fixed.rate,
        duplex: Duplex::Half,
    };
    if !spec.abilities.contains(&half) {
        return Err(fail(
            ErrorKind::Unsupported,
            "Parallel detection at this rate requires supplied half-duplex support",
        ));
    }
    Ok(Some(half))
}
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counters {
    pub offered: u64,
    pub received: u64,
    pub crc_errors: u64,
    pub collisions: u64,
    pub late_collisions: u64,
}
impl Counters {
    fn valid(&self) -> bool {
        self.late_collisions <= self.collisions
            && [
                self.offered,
                self.received,
                self.crc_errors,
                self.collisions,
                self.late_collisions,
            ]
            .into_iter()
            .all(|n| n <= MAX_COUNTER)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadingOrigin {
    AuthoredSample,
    ReportedMeasurement,
}
/// Raw supplied observations have platform-specific meanings. No derived-counter
/// subset identity or rate/duplex diagnosis is imposed on these readings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedCounters {
    pub input_packets: u64,
    pub output_packets: u64,
    pub crc_errors: u64,
    pub collisions: u64,
    pub late_collisions: u64,
}
impl ObservedCounters {
    fn bounded(&self) -> bool {
        [
            self.input_packets,
            self.output_packets,
            self.crc_errors,
            self.collisions,
            self.late_collisions,
        ]
        .into_iter()
        .all(|n| n <= MAX_COUNTER)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuppliedReading {
    pub origin: ReadingOrigin,
    #[serde(deserialize_with = "label")]
    pub note: String,
    pub counters: ObservedCounters,
}
impl SuppliedReading {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_READING {
            return Err(fail(ErrorKind::Limit, "Supplied reading exceeds8KiB"));
        }
        let reading: Self = serde_json::from_slice(raw).map_err(|e| {
            fail(
                ErrorKind::Invalid,
                format!("Invalid supplied observation: {e}"),
            )
        })?;
        reading.validate()?;
        Ok(reading)
    }
    pub fn validate(&self) -> Result<()> {
        if !text(&self.note) || !self.counters.bounded() {
            Err(fail(
                ErrorKind::Invalid,
                "Invalid supplied reading; it is not a simulated diagnosis",
            ))
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProbeReport {
    pub link: LinkReport,
    pub before: BTreeMap<String, Counters>,
    pub after: BTreeMap<String, Counters>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    topology: Topology,
    configuration: Configuration,
    #[serde(deserialize_with = "map::<_,Counters,MAX_PORTS>")]
    counters: BTreeMap<String, Counters>,
    #[serde(deserialize_with = "map::<_,u64,MAX_PORTS>")]
    sequences: BTreeMap<String, u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum Action {
    Configure {
        configuration: Configuration,
    },
    ConfigurePort {
        port: String,
        configuration: PortConfig,
    },
    SelectCable {
        link: String,
        cable: String,
    },
    ClearCounters {
        port: String,
    },
    Probe {
        link: String,
        a_frames: u32,
        b_frames: u32,
    },
}
// Streaming wire admission avoids serde's internally-tagged Content buffering:
// an Action can be embedded in a larger pack without allocating arbitrary ID
// strings before their visitors run. Field order remains unrestricted.
#[derive(Clone, Copy)]
enum WireField {
    Kind,
    Configuration,
    Port,
    Link,
    Cable,
    AFrames,
    BFrames,
    Ports,
    Cables,
    AdminUp,
    Profile,
    Negotiation,
    Every,
    Advertise,
    Ability,
}
impl<'de> Deserialize<'de> for WireField {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Field;
        impl Visitor<'_> for Field {
            type Value = WireField;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a supported uplink action/configuration field")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<WireField, E> {
                match value {
                    "kind" => Ok(WireField::Kind),
                    "configuration" => Ok(WireField::Configuration),
                    "port" => Ok(WireField::Port),
                    "link" => Ok(WireField::Link),
                    "cable" => Ok(WireField::Cable),
                    "a_frames" => Ok(WireField::AFrames),
                    "b_frames" => Ok(WireField::BFrames),
                    "ports" => Ok(WireField::Ports),
                    "cables" => Ok(WireField::Cables),
                    "admin_up" => Ok(WireField::AdminUp),
                    "profile" => Ok(WireField::Profile),
                    "negotiation" => Ok(WireField::Negotiation),
                    "every" => Ok(WireField::Every),
                    "advertise" => Ok(WireField::Advertise),
                    "ability" => Ok(WireField::Ability),
                    _ => Err(E::custom("Unknown uplink field before value")),
                }
            }
        }
        d.deserialize_identifier(Field)
    }
}
enum WireKind {
    Configure,
    ConfigurePort,
    SelectCable,
    ClearCounters,
    Probe,
    Clean,
    Disconnected,
    ReceiveImpairment,
    Auto,
    Fixed,
}
impl<'de> Deserialize<'de> for WireKind {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Kind;
        impl Visitor<'_> for Kind {
            type Value = WireKind;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a supported uplink action kind")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<WireKind, E> {
                match value {
                    "Configure" => Ok(WireKind::Configure),
                    "ConfigurePort" => Ok(WireKind::ConfigurePort),
                    "SelectCable" => Ok(WireKind::SelectCable),
                    "ClearCounters" => Ok(WireKind::ClearCounters),
                    "Probe" => Ok(WireKind::Probe),
                    "Clean" => Ok(WireKind::Clean),
                    "Disconnected" => Ok(WireKind::Disconnected),
                    "ReceiveImpairment" => Ok(WireKind::ReceiveImpairment),
                    "Auto" => Ok(WireKind::Auto),
                    "Fixed" => Ok(WireKind::Fixed),
                    _ => Err(E::custom("Unknown uplink action kind before body")),
                }
            }
        }
        d.deserialize_str(Kind)
    }
}
// Unlike an untagged derived enum, this field visitor never buffers a general
// JSON Value to distinguish whole-configuration and individual-port actions.
enum WireConfiguration {
    Whole(Configuration),
    Port(PortConfig),
}
impl<'de> Deserialize<'de> for WireConfiguration {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Config;
        impl<'de> Visitor<'de> for Config {
            type Value = WireConfiguration;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a bounded whole or port configuration")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let (mut ports, mut cables, mut admin_up, mut profile, mut negotiation) =
                    (None, None, None, None, None);
                macro_rules! read {
                    ($slot:ident, $ty:ty) => {{
                        if $slot.is_some() {
                            return Err(de::Error::custom(
                                "Duplicate configuration field before value",
                            ));
                        }
                        $slot = Some(a.next_value::<$ty>()?);
                    }};
                }
                while let Some(field) = a.next_key::<WireField>()? {
                    match field {
                        WireField::Ports => {
                            if ports.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate configuration field before value",
                                ));
                            }
                            struct Ports(BTreeMap<String, PortConfig>);
                            impl<'de> Deserialize<'de> for Ports {
                                fn deserialize<D: serde::Deserializer<'de>>(
                                    d: D,
                                ) -> std::result::Result<Self, D::Error>
                                {
                                    map::<D, PortConfig, MAX_PORTS>(d).map(Self)
                                }
                            }
                            ports = Some(a.next_value::<Ports>()?.0);
                        }
                        WireField::Cables => {
                            if cables.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate configuration field before value",
                                ));
                            }
                            struct Cables(BTreeMap<String, String>);
                            impl<'de> Deserialize<'de> for Cables {
                                fn deserialize<D: serde::Deserializer<'de>>(
                                    d: D,
                                ) -> std::result::Result<Self, D::Error>
                                {
                                    identity_map::<D, MAX_LINKS>(d).map(Self)
                                }
                            }
                            cables = Some(a.next_value::<Cables>()?.0);
                        }
                        WireField::AdminUp => read!(admin_up, bool),
                        WireField::Profile => read!(profile, Profile),
                        WireField::Negotiation => read!(negotiation, Negotiation),
                        _ => {
                            return Err(de::Error::custom(
                                "Unknown configuration field before value",
                            ));
                        }
                    }
                }
                match (ports, cables, admin_up, profile, negotiation) {
                    (Some(ports), Some(cables), None, None, None) => {
                        Ok(WireConfiguration::Whole(Configuration { ports, cables }))
                    }
                    (None, None, Some(admin_up), Some(profile), Some(negotiation)) => {
                        Ok(WireConfiguration::Port(PortConfig {
                            admin_up,
                            profile,
                            negotiation,
                        }))
                    }
                    _ => Err(de::Error::custom(
                        "Configuration needs exact whole or port fields",
                    )),
                }
            }
        }
        d.deserialize_map(Config)
    }
}
impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Input;
        impl<'de> Visitor<'de> for Input {
            type Value = Condition;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a supplied cable condition")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Condition, A::Error> {
                let (mut kind, mut every) = (None, None);
                while let Some(field) = a.next_key::<WireField>()? {
                    match field {
                        WireField::Kind => {
                            if kind.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate condition field before value",
                                ));
                            }
                            kind = Some(a.next_value::<WireKind>()?);
                        }
                        WireField::Every => {
                            if every.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate condition field before value",
                                ));
                            }
                            every = Some(a.next_value::<u16>()?);
                        }
                        _ => return Err(de::Error::custom("Unknown condition field before value")),
                    }
                }
                match (kind, every) {
                    (Some(WireKind::Clean), None) => Ok(Condition::Clean),
                    (Some(WireKind::Disconnected), None) => Ok(Condition::Disconnected),
                    (Some(WireKind::ReceiveImpairment), Some(every)) => {
                        Ok(Condition::ReceiveImpairment { every })
                    }
                    _ => Err(de::Error::custom(
                        "Condition needs exactly its supported fields",
                    )),
                }
            }
        }
        d.deserialize_map(Input)
    }
}
struct Abilities(BTreeSet<Ability>);
impl<'de> Deserialize<'de> for Abilities {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        set::<D, Ability, 5>(d).map(Self)
    }
}
impl<'de> Deserialize<'de> for Negotiation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Input;
        impl<'de> Visitor<'de> for Input {
            type Value = Negotiation;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("supported uplink negotiation")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Negotiation, A::Error> {
                let (mut kind, mut advertise, mut ability) = (None, None, None);
                while let Some(field) = a.next_key::<WireField>()? {
                    match field {
                        WireField::Kind => {
                            if kind.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate negotiation field before value",
                                ));
                            }
                            kind = Some(a.next_value::<WireKind>()?);
                        }
                        WireField::Advertise => {
                            if advertise.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate negotiation field before value",
                                ));
                            }
                            advertise = Some(a.next_value::<Abilities>()?.0);
                        }
                        WireField::Ability => {
                            if ability.is_some() {
                                return Err(de::Error::custom(
                                    "Duplicate negotiation field before value",
                                ));
                            }
                            ability = Some(a.next_value::<Ability>()?);
                        }
                        _ => {
                            return Err(de::Error::custom(
                                "Unknown negotiation field before value",
                            ));
                        }
                    }
                }
                match (kind, advertise, ability) {
                    (Some(WireKind::Auto), Some(advertise), None) => {
                        Ok(Negotiation::Auto { advertise })
                    }
                    (Some(WireKind::Fixed), None, Some(ability)) => {
                        Ok(Negotiation::Fixed { ability })
                    }
                    _ => Err(de::Error::custom(
                        "Negotiation needs exactly its supported fields",
                    )),
                }
            }
        }
        d.deserialize_map(Input)
    }
}
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Input;
        impl<'de> Visitor<'de> for Input {
            type Value = Action;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a bounded uplink action")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Action, A::Error> {
                let (
                    mut kind,
                    mut configuration,
                    mut port,
                    mut link,
                    mut cable,
                    mut a_frames,
                    mut b_frames,
                ) = (None, None, None, None, None, None, None);
                macro_rules! read {
                    ($slot:ident, $ty:ty) => {{
                        if $slot.is_some() {
                            return Err(de::Error::custom("Duplicate action field before value"));
                        }
                        $slot = Some(a.next_value::<$ty>()?);
                    }};
                }
                while let Some(field) = a.next_key::<WireField>()? {
                    match field {
                        WireField::Kind => read!(kind, WireKind),
                        WireField::Configuration => read!(configuration, WireConfiguration),
                        WireField::Port => read!(port, Key),
                        WireField::Link => read!(link, Key),
                        WireField::Cable => read!(cable, Key),
                        WireField::AFrames => read!(a_frames, u32),
                        WireField::BFrames => read!(b_frames, u32),
                        _ => return Err(de::Error::custom("Unknown action field before value")),
                    }
                }
                match (kind, configuration, port, link, cable, a_frames, b_frames) {
                    (
                        Some(WireKind::Configure),
                        Some(WireConfiguration::Whole(configuration)),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ) => Ok(Action::Configure { configuration }),
                    (
                        Some(WireKind::ConfigurePort),
                        Some(WireConfiguration::Port(configuration)),
                        Some(Key(port)),
                        None,
                        None,
                        None,
                        None,
                    ) => Ok(Action::ConfigurePort {
                        port,
                        configuration,
                    }),
                    (
                        Some(WireKind::SelectCable),
                        None,
                        None,
                        Some(Key(link)),
                        Some(Key(cable)),
                        None,
                        None,
                    ) => Ok(Action::SelectCable { link, cable }),
                    (
                        Some(WireKind::ClearCounters),
                        None,
                        Some(Key(port)),
                        None,
                        None,
                        None,
                        None,
                    ) => Ok(Action::ClearCounters { port }),
                    (
                        Some(WireKind::Probe),
                        None,
                        None,
                        Some(Key(link)),
                        None,
                        Some(a_frames),
                        Some(b_frames),
                    ) => Ok(Action::Probe {
                        link,
                        a_frames,
                        b_frames,
                    }),
                    _ => Err(de::Error::custom(
                        "Action needs exactly its supported fields",
                    )),
                }
            }
        }
        d.deserialize_map(Input)
    }
}
impl Action {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_ACTION {
            return Err(fail(ErrorKind::Limit, "Uplink action exceeds32KiB"));
        }
        serde_json::from_slice(raw).map_err(|e| {
            fail(
                ErrorKind::Invalid,
                format!("Unsupported or malformed uplink action: {e}"),
            )
        })
    }
}
/// Borrowed action result for exact encoded-size admission before a full State
/// or Attempt copy. Config≤16KiB,24 bounded counter/sequence entries and a two-
/// endpoint report are the only possible owned scratch (under64KiB total).
#[derive(Serialize)]
pub struct ActionProjection<'a> {
    topology: &'a Topology,
    configuration: Cow<'a, Configuration>,
    counters: Cow<'a, BTreeMap<String, Counters>>,
    sequences: Cow<'a, BTreeMap<String, u64>>,
    #[serde(skip)]
    report: Option<ProbeReport>,
}
impl ActionProjection<'_> {
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn report(&self) -> Option<&ProbeReport> {
        self.report.as_ref()
    }
}
impl State {
    pub fn project<'a>(&'a self, action: &'a Action) -> Result<ActionProjection<'a>> {
        self.validate()?;
        if !within(action, MAX_ACTION) {
            return Err(fail(ErrorKind::Limit, "Uplink action encoded bound"));
        }
        self.preflight(action)?;
        let mut projected = ActionProjection {
            topology: &self.topology,
            configuration: Cow::Borrowed(&self.configuration),
            counters: Cow::Borrowed(&self.counters),
            sequences: Cow::Borrowed(&self.sequences),
            report: None,
        };
        match action {
            Action::Configure { configuration } => {
                projected.configuration = Cow::Borrowed(configuration)
            }
            Action::ConfigurePort {
                port,
                configuration,
            } => {
                projected
                    .configuration
                    .to_mut()
                    .ports
                    .insert(port.clone(), configuration.clone());
            }
            Action::SelectCable { link, cable } => {
                projected
                    .configuration
                    .to_mut()
                    .cables
                    .insert(link.clone(), cable.clone());
            }
            Action::ClearCounters { port } => {
                *projected.counters.to_mut().get_mut(port).unwrap() = Counters::default();
            }
            Action::Probe {
                link,
                a_frames,
                b_frames,
            } => {
                let report = self.report_valid(link)?;
                projected.report = Some(probe_values(
                    &self.topology,
                    &self.configuration,
                    projected.counters.to_mut(),
                    projected.sequences.to_mut(),
                    link,
                    (*a_frames, *b_frames),
                    report,
                )?);
            }
        }
        projected.configuration.validate(&self.topology)?;
        if projected.counters.values().any(|c| !c.valid())
            || projected.sequences.values().any(|n| *n > MAX_COUNTER)
            || !within(&projected, MAX_STATE)
        {
            return Err(fail(
                ErrorKind::Limit,
                "Projected physical state exceeds counter/encoded limits",
            ));
        }
        Ok(projected)
    }
    pub fn new(topology: Topology, configuration: Configuration) -> Result<Self> {
        topology.validate()?;
        configuration.validate(&topology)?;
        let counters = topology
            .ports
            .keys()
            .map(|k| (k.clone(), Counters::default()))
            .collect();
        let sequences = topology.ports.keys().map(|k| (k.clone(), 0)).collect();
        let s = Self {
            topology,
            configuration,
            counters,
            sequences,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_STATE {
            return Err(fail(ErrorKind::Limit, "Uplink state exceeds512KiB"));
        }
        let s: Self = serde_json::from_slice(raw)
            .map_err(|e| fail(ErrorKind::Invalid, format!("Invalid uplink state: {e}")))?;
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<()> {
        self.topology.validate()?;
        self.configuration.validate(&self.topology)?;
        if !self.counters.keys().eq(self.topology.ports.keys())
            || !self.sequences.keys().eq(self.topology.ports.keys())
            || self.counters.values().any(|c| !c.valid())
            || self.sequences.values().any(|n| *n > MAX_COUNTER)
            || !within(self, MAX_STATE)
        {
            return Err(fail(ErrorKind::Limit, "Uplink counters/state bound"));
        }
        Ok(())
    }
    pub fn topology(&self) -> &Topology {
        &self.topology
    }
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn counters(&self) -> &BTreeMap<String, Counters> {
        &self.counters
    }
    pub fn report(&self, id: &str) -> Result<LinkReport> {
        self.validate()?;
        self.report_valid(id)
    }
    fn report_valid(&self, id: &str) -> Result<LinkReport> {
        let l = self
            .topology
            .links
            .get(id)
            .ok_or_else(|| fail(ErrorKind::Identity, "Unknown supplied link"))?;
        let a = &self.configuration.ports[&l.a];
        let b = &self.configuration.ports[&l.b];
        let cable = &self.topology.cables[&self.configuration.cables[id]];
        let qualification = qualify(a.profile, b.profile, cable, l.required_span_m);
        let empty = |state| LinkReport {
            qualification,
            state,
            a: None,
            b: None,
        };
        if !a.admin_up || !b.admin_up {
            return Ok(empty(LinkState::AdministrativelyDown));
        }
        if cable.condition == Condition::Disconnected {
            return Ok(empty(LinkState::Disconnected));
        }
        if qualification != Qualification::Qualified {
            return Ok(empty(LinkState::OutsideModel));
        }
        let Some((a, b)) = resolve(a, b, &self.topology.ports[&l.a], &self.topology.ports[&l.b])?
        else {
            return Ok(empty(LinkState::NoCommonAbility));
        };
        Ok(LinkReport {
            qualification,
            state: if a.ability.duplex == b.ability.duplex {
                LinkState::Up
            } else {
                LinkState::DegradedDuplex
            },
            a: Some(a),
            b: Some(b),
        })
    }
    pub fn perform(&mut self, a: &Action) -> Result<Option<ProbeReport>> {
        self.validate()?;
        if !within(a, MAX_ACTION) {
            return Err(fail(ErrorKind::Limit, "Uplink action encoded bound"));
        }
        self.preflight(a)?;
        #[cfg(test)]
        LARGE_COPIES.with(|n| n.set(n.get() + 1));
        let mut next = self.clone();
        let out = next.perform_valid(a)?;
        next.validate()?;
        *self = next;
        Ok(out)
    }
    fn preflight(&self, a: &Action) -> Result<()> {
        match a {
            Action::Configure { configuration } => configuration.validate(&self.topology),
            Action::ConfigurePort {
                port,
                configuration,
            } => {
                let spec = self
                    .topology
                    .ports
                    .get(port)
                    .ok_or_else(|| fail(ErrorKind::Identity, "Unknown supplied port"))?;
                config_port(spec, configuration)
            }
            Action::SelectCable { link, cable } => {
                if !self.topology.links.contains_key(link)
                    || !self.topology.cables.contains_key(cable)
                    || self
                        .configuration
                        .cables
                        .iter()
                        .any(|(id, current)| id != link && current == cable)
                {
                    Err(fail(
                        ErrorKind::Identity,
                        "Choose a supplied cable not already used by another link",
                    ))
                } else {
                    Ok(())
                }
            }
            Action::ClearCounters { port } => self
                .topology
                .ports
                .contains_key(port)
                .then_some(())
                .ok_or_else(|| fail(ErrorKind::Identity, "Unknown supplied port")),
            Action::Probe {
                link,
                a_frames,
                b_frames,
            } => {
                if *a_frames > MAX_PROBE || *b_frames > MAX_PROBE || *a_frames + *b_frames == 0 {
                    return Err(fail(ErrorKind::Limit, "Probe frame allowance"));
                }
                self.topology
                    .links
                    .contains_key(link)
                    .then_some(())
                    .ok_or_else(|| fail(ErrorKind::Identity, "Unknown supplied link"))
            }
        }
    }
    fn perform_valid(&mut self, a: &Action) -> Result<Option<ProbeReport>> {
        match a {
            Action::Configure { configuration } => {
                self.configuration = configuration.clone();
                Ok(None)
            }
            Action::ConfigurePort {
                port,
                configuration,
            } => {
                let p = self
                    .topology
                    .ports
                    .get(port)
                    .ok_or_else(|| fail(ErrorKind::Identity, "Unknown supplied port"))?;
                config_port(p, configuration)?;
                self.configuration
                    .ports
                    .insert(port.clone(), configuration.clone());
                Ok(None)
            }
            Action::SelectCable { link, cable } => {
                if !self.topology.links.contains_key(link)
                    || !self.topology.cables.contains_key(cable)
                {
                    return Err(fail(
                        ErrorKind::Identity,
                        "Choose a supplied link and cable",
                    ));
                }
                self.configuration
                    .cables
                    .insert(link.clone(), cable.clone());
                self.configuration.validate(&self.topology)?;
                Ok(None)
            }
            Action::ClearCounters { port } => {
                let c = self
                    .counters
                    .get_mut(port)
                    .ok_or_else(|| fail(ErrorKind::Identity, "Unknown supplied port"))?;
                *c = Counters::default();
                Ok(None)
            }
            Action::Probe {
                link,
                a_frames,
                b_frames,
            } => self.probe(link, *a_frames, *b_frames).map(Some),
        }
    }
    fn probe(&mut self, id: &str, fa: u32, fb: u32) -> Result<ProbeReport> {
        if fa > MAX_PROBE || fb > MAX_PROBE || fa + fb == 0 {
            return Err(fail(
                ErrorKind::Limit,
                "Probe uses1..1024 offered frames per direction",
            ));
        }
        let report = self.report_valid(id)?;
        probe_values(
            &self.topology,
            &self.configuration,
            &mut self.counters,
            &mut self.sequences,
            id,
            (fa, fb),
            report,
        )
    }
}

// Shared deterministic probe arithmetic. Projection owns only small bounded
// counter/sequence maps; actual execution uses these same equations.
fn probe_values(
    topology: &Topology,
    configuration: &Configuration,
    counters: &mut BTreeMap<String, Counters>,
    sequences: &mut BTreeMap<String, u64>,
    id: &str,
    frames: (u32, u32),
    report: LinkReport,
) -> Result<ProbeReport> {
    let (fa, fb) = frames;
    if !matches!(report.state, LinkState::Up | LinkState::DegradedDuplex) {
        return Err(fail(
            ErrorKind::Unsupported,
            "No qualified operational link for this bounded probe; outside-profile qualification does not predict actual physical carrier",
        ));
    }
    let l = &topology.links[id];
    let cable = &topology.cables[&configuration.cables[id]];
    let contention = l.pattern == ProbePattern::DuplexMismatchContention;
    if contention
        && report.a.unwrap().ability.duplex == Duplex::Half
        && report.b.unwrap().ability.duplex == Duplex::Half
    {
        return Err(fail(
            ErrorKind::Unsupported,
            "Shared/half-duplex contention backoff and retries are not modeled; use a serialized probe profile",
        ));
    }
    let before = BTreeMap::from([
        (l.a.clone(), counters[&l.a].clone()),
        (l.b.clone(), counters[&l.b].clone()),
    ]);
    let overlap = if contention && report.state == LinkState::DegradedDuplex {
        fa.min(fb) as u64
    } else {
        0
    };
    for (source, dest, frames, op) in [
        (&l.a, &l.b, fa, report.a.unwrap()),
        (&l.b, &l.a, fb, report.b.unwrap()),
    ] {
        let frames = frames as u64;
        let prior = sequences[source];
        let sequence = prior
            .checked_add(frames)
            .filter(|n| *n <= MAX_COUNTER)
            .ok_or_else(|| fail(ErrorKind::Limit, "Probe sequence limit"))?;
        // CRC corruption is injected independently of duplex. No inference that every
        // CRC reading proves cabling or that every duplex mismatch produces CRC.
        let bad = match cable.condition {
            Condition::ReceiveImpairment { every } => {
                sequence / (every as u64) - (prior + overlap) / (every as u64)
            }
            _ => 0,
        };
        let sent = counters.get_mut(source).unwrap();
        sent.offered = sent
            .offered
            .checked_add(frames)
            .ok_or_else(|| fail(ErrorKind::Limit, "Counter overflow"))?;
        if op.ability.duplex == Duplex::Half {
            sent.collisions += overlap;
            sent.late_collisions += overlap;
        }
        let received = counters.get_mut(dest).unwrap();
        received.crc_errors += bad;
        received.received += frames - overlap - bad;
        sequences.insert(source.clone(), sequence);
    }
    let after = BTreeMap::from([
        (l.a.clone(), counters[&l.a].clone()),
        (l.b.clone(), counters[&l.b].clone()),
    ]);
    Ok(ProbeReport {
        link: report,
        before,
        after,
    })
}

#[cfg(test)]
thread_local! { static LARGE_COPIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
mod atomic_configuration_tests {
    use super::*;
    fn fixture() -> State {
        let ability = Ability {
            rate: Rate::M100,
            duplex: Duplex::Full,
        };
        let ports = ["a", "b", "c", "d"]
            .into_iter()
            .map(|id| {
                (
                    id.into(),
                    PortSpec {
                        device: format!("device-{id}"),
                        label: id.into(),
                        profiles: BTreeSet::from([Profile::CopperRj45]),
                        abilities: BTreeSet::from([ability]),
                        parallel_detection: false,
                    },
                )
            })
            .collect();
        let cables = ["cable-one", "cable-two"]
            .into_iter()
            .map(|id| {
                (
                    id.into(),
                    Cable {
                        label: id.into(),
                        kind: CableKind::Cat5e,
                        length_m: 10,
                        condition: Condition::Clean,
                    },
                )
            })
            .collect();
        let links = BTreeMap::from([
            (
                "one".into(),
                LinkSpec {
                    a: "a".into(),
                    b: "b".into(),
                    required_span_m: 5,
                    pattern: ProbePattern::Serialized,
                },
            ),
            (
                "two".into(),
                LinkSpec {
                    a: "c".into(),
                    b: "d".into(),
                    required_span_m: 5,
                    pattern: ProbePattern::Serialized,
                },
            ),
        ]);
        let topology = Topology {
            version: 1,
            ports,
            cables,
            links,
        };
        let configuration = Configuration {
            ports: ["a", "b", "c", "d"]
                .into_iter()
                .map(|p| {
                    (
                        p.into(),
                        PortConfig {
                            admin_up: true,
                            profile: Profile::CopperRj45,
                            negotiation: Negotiation::Auto {
                                advertise: BTreeSet::from([ability]),
                            },
                        },
                    )
                })
                .collect(),
            cables: BTreeMap::from([
                ("one".into(), "cable-one".into()),
                ("two".into(), "cable-two".into()),
            ]),
        };
        State::new(topology, configuration).unwrap()
    }
    #[test]
    fn whole_swap_is_atomic_and_retains_every_neighbor_field_and_counter() {
        let mut state = fixture();
        state
            .perform(&Action::Probe {
                link: "one".into(),
                a_frames: 7,
                b_frames: 9,
            })
            .unwrap();
        let before = state.clone();
        assert!(
            state
                .perform(&Action::SelectCable {
                    link: "one".into(),
                    cable: "cable-two".into()
                })
                .is_err()
        );
        assert_eq!(state, before);
        let mut configuration = state.configuration.clone();
        configuration
            .cables
            .insert("one".into(), "cable-two".into());
        configuration
            .cables
            .insert("two".into(), "cable-one".into());
        LARGE_COPIES.with(|n| n.set(0));
        state
            .perform(&Action::Configure {
                configuration: configuration.clone(),
            })
            .unwrap();
        assert_eq!(LARGE_COPIES.with(|n| n.get()), 1);
        assert_eq!(state.configuration, configuration);
        assert_eq!(state.topology, before.topology);
        assert_eq!(state.counters, before.counters);
        assert_eq!(state.sequences, before.sequences);
        assert_eq!(state.configuration.ports, before.configuration.ports);
    }
    #[test]
    fn invalid_whole_configuration_fields_refuse_before_any_large_copy() {
        let mut state = fixture();
        let before = state.clone();
        let base = state.configuration.clone();
        for mode in 0..6 {
            let mut configuration = base.clone();
            match mode {
                0 => {
                    configuration.ports.remove("a");
                }
                1 => {
                    configuration
                        .cables
                        .insert("one".into(), "cable-two".into());
                }
                2 => {
                    configuration.cables.insert("one".into(), "unknown".into());
                }
                3 => {
                    configuration.ports.get_mut("a").unwrap().profile = Profile::SxOm2;
                }
                4 => {
                    configuration.ports.get_mut("a").unwrap().negotiation = Negotiation::Auto {
                        advertise: BTreeSet::new(),
                    };
                }
                _ => {
                    configuration
                        .cables
                        .insert("unknown".into(), "cable-two".into());
                }
            }
            LARGE_COPIES.with(|n| n.set(0));
            assert!(state.perform(&Action::Configure { configuration }).is_err());
            assert_eq!(LARGE_COPIES.with(|n| n.get()), 0);
            assert_eq!(state, before);
        }
    }
    #[test]
    fn borrowed_action_projection_is_byte_exact_without_large_state_copy() {
        let state = fixture();
        let mut changed = state.configuration.clone();
        changed.ports.get_mut("a").unwrap().admin_up = false;
        let actions = [
            Action::Configure {
                configuration: changed,
            },
            Action::ConfigurePort {
                port: "a".into(),
                configuration: state.configuration.ports["a"].clone(),
            },
            Action::ClearCounters { port: "a".into() },
            Action::Probe {
                link: "one".into(),
                a_frames: 9,
                b_frames: 12,
            },
        ];
        for action in actions {
            LARGE_COPIES.with(|n| n.set(0));
            let projected = state.project(&action).unwrap();
            assert_eq!(LARGE_COPIES.with(|n| n.get()), 0);
            let encoded = serde_json::to_vec(&projected).unwrap();
            let mut real = state.clone();
            real.perform(&action).unwrap();
            assert_eq!(LARGE_COPIES.with(|n| n.get()), 1);
            assert_eq!(encoded, serde_json::to_vec(&real).unwrap());
        }
    }
    #[test]
    fn rejected_projection_has_no_large_copy_or_caller_mutation() {
        let state = fixture();
        let before = serde_json::to_vec(&state).unwrap();
        LARGE_COPIES.with(|n| n.set(0));
        assert!(
            state
                .project(&Action::SelectCable {
                    link: "one".into(),
                    cable: "cable-two".into()
                })
                .is_err()
        );
        assert_eq!(LARGE_COPIES.with(|n| n.get()), 0);
        assert_eq!(serde_json::to_vec(&state).unwrap(), before);
        let mut capped = state;
        for value in capped.counters.values_mut() {
            value.offered = MAX_COUNTER;
            value.received = MAX_COUNTER;
        }
        let before = serde_json::to_vec(&capped).unwrap();
        assert!(
            capped
                .project(&Action::Probe {
                    link: "one".into(),
                    a_frames: 1,
                    b_frames: 1
                })
                .is_err()
        );
        assert_eq!(LARGE_COPIES.with(|n| n.get()), 0);
        assert_eq!(serde_json::to_vec(&capped).unwrap(), before);
    }
}
