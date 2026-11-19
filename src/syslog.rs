//! Bounded virtual syslog policy/events. No wire protocol, real clocks or I/O.
//! History is evidence; only fresh policy decisions drive current checks.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::net::Ipv4Addr;

pub const VERSION: u32 = 1;
pub const MAX_DEVICES: usize = 8;
pub const MAX_COLLECTORS: usize = 8;
pub const MAX_EVENTS: usize = 32;
pub const MAX_HOSTS: usize = 8;
pub const MAX_DELIVERIES: usize = 64;
pub const MAX_HISTORY: usize = 32;
pub const MAX_REQUIREMENTS: usize = 32;
pub const MAX_STATE_BYTES: usize = 262_144;
pub const MAX_OUTPUT: usize = 32_768;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unknown,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, text: &str) -> Error {
    Error {
        kind,
        message: text.into(),
    }
}
fn invalid(text: &str) -> Error {
    error(ErrorKind::Invalid, text)
}
fn unicast(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified()
        && !ip.is_broadcast()
        && !ip.is_multicast()
        && !ip.is_loopback()
        && ip.octets()[0] != 0
        && ip.octets()[0] < 224
}
fn safe_text(s: &str, n: usize) -> bool {
    !s.is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
fn symbol(s: &str, n: usize) -> bool {
    !s.is_empty()
        && s.len() <= n
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}
fn bounded_text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    struct V<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for V<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "nonempty printable UTF-8 up to{N}bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if safe_text(s, N) {
                Ok(s.into())
            } else {
                Err(E::custom("Bounded printable text required"))
            }
        }
        fn visit_string<E: serde::de::Error>(self, s: String) -> std::result::Result<String, E> {
            if safe_text(&s, N) {
                Ok(s)
            } else {
                Err(E::custom("Bounded printable text required"))
            }
        }
    }
    d.deserialize_string(V::<N>)
}
fn label<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    bounded_text::<D, 64>(d)
}
fn message<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    bounded_text::<D, 256>(d)
}
fn subsystem<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    bounded_text::<D, 16>(d)
}
fn mnemonic<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    bounded_text::<D, 24>(d)
}
fn bounded_map<'de, D, K, V, const N: usize>(d: D) -> std::result::Result<BTreeMap<K, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
{
    struct M<K, V, const N: usize>(std::marker::PhantomData<(K, V)>);
    impl<'de, K: Deserialize<'de> + Ord, V: Deserialize<'de>, const N: usize>
        serde::de::Visitor<'de> for M<K, V, N>
    {
        type Value = BTreeMap<K, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "up to{N}unique entries")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut m = BTreeMap::new();
            while let Some(k) = a.next_key::<K>()? {
                if m.len() >= N || m.contains_key(&k) {
                    return Err(serde::de::Error::custom(
                        "Map bound or duplicate key before value",
                    ));
                }
                m.insert(k, a.next_value()?);
            }
            Ok(m)
        }
    }
    d.deserialize_map(M::<K, V, N>(std::marker::PhantomData))
}
struct Excess;
impl<'de> Deserialize<'de> for Excess {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> std::result::Result<Self, D::Error> {
        Err(serde::de::Error::custom("Sequence excess before value"))
    }
}
fn bounded_vec<'de, D, V, const N: usize>(d: D) -> std::result::Result<Vec<V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct S<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for S<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "up to{N}elements")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Vec<V>, A::Error> {
            let mut v = Vec::new();
            while v.len() < N {
                match a.next_element()? {
                    Some(x) => v.push(x),
                    None => return Ok(v),
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(v)
        }
    }
    d.deserialize_seq(S::<V, N>(std::marker::PhantomData))
}
fn hosts<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<Ipv4Addr>, D::Error> {
    let v = bounded_vec::<D, Ipv4Addr, MAX_HOSTS>(d)?;
    let n = v.len();
    let s: BTreeSet<_> = v.into_iter().collect();
    if s.len() != n {
        return Err(serde::de::Error::custom("Duplicate remote host"));
    }
    Ok(s)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Informational = 6,
    Debug = 7,
}
impl Severity {
    pub fn from_number(n: u8) -> Result<Self> {
        match n {
            0 => Ok(Self::Emergency),
            1 => Ok(Self::Alert),
            2 => Ok(Self::Critical),
            3 => Ok(Self::Error),
            4 => Ok(Self::Warning),
            5 => Ok(Self::Notice),
            6 => Ok(Self::Informational),
            7 => Ok(Self::Debug),
            _ => Err(invalid("Severity must be0–7.")),
        }
    }
    pub fn number(self) -> u8 {
        self as u8
    }
    pub fn includes(self, event: Self) -> bool {
        event.number() <= self.number()
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Emergency => "Emergency",
            Self::Alert => "Alert",
            Self::Critical => "Critical",
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Notice => "Notice",
            Self::Informational => "Informational",
            Self::Debug => "Debug",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Facility(u8);
impl Facility {
    pub fn new(n: u8) -> Result<Self> {
        if n < 24 {
            Ok(Self(n))
        } else {
            Err(invalid("Facility must be0–23."))
        }
    }
    pub fn number(self) -> u8 {
        self.0
    }
    pub fn name(self) -> String {
        if self.0 >= 16 {
            format!("local{}", self.0 - 16)
        } else {
            format!("facility{}", self.0)
        }
    }
    pub fn priority(self, severity: Severity) -> u8 {
        self.0 * 8 + severity.number()
    }
}
impl<'de> Deserialize<'de> for Facility {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let n = u8::deserialize(d)?;
        Self::new(n).map_err(serde::de::Error::custom)
    }
}
pub fn split_priority(n: u16) -> Result<(Facility, Severity)> {
    if n > 191 {
        return Err(invalid("PRI must be0–191."));
    }
    Ok((
        Facility::new((n / 8) as u8)?,
        Severity::from_number((n % 8) as u8)?,
    ))
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct EventId(String);
impl EventId {
    pub fn new(s: &str) -> Result<Self> {
        if !s.is_empty()
            && s.len() <= 32
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        {
            Ok(Self(s.into()))
        } else {
            Err(invalid(
                "Event identifier needs1–32lowercase ASCII letters, digits, underscore or dash.",
            ))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for EventId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = bounded_text::<D, 32>(d)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    #[serde(deserialize_with = "subsystem")]
    pub subsystem: String,
    pub severity: Severity,
    #[serde(deserialize_with = "mnemonic")]
    pub mnemonic: String,
    #[serde(deserialize_with = "message")]
    pub text: String,
}
impl Event {
    fn validate(&self) -> Result<()> {
        if !symbol(&self.subsystem, 16)
            || !symbol(&self.mnemonic, 24)
            || !safe_text(&self.text, 256)
        {
            return Err(invalid("Invalid bounded event template."));
        }
        Ok(())
    }
    pub fn cisco_style_text(&self) -> Result<String> {
        self.validate()?;
        Ok(format!(
            "%{}-{}-{}: {}",
            self.subsystem,
            self.severity.number(),
            self.mnemonic,
            self.text
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "label")]
    pub name: String,
    pub history_capacity: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub device: Ipv4Addr,
    pub collector: Ipv4Addr,
    pub available: bool,
}
macro_rules! map_reader {
    ($name:ident,$k:ty,$v:ty,$n:expr) => {
        fn $name<'de, D: serde::Deserializer<'de>>(
            d: D,
        ) -> std::result::Result<BTreeMap<$k, $v>, D::Error> {
            bounded_map::<D, $k, $v, $n>(d)
        }
    };
}
map_reader!(device_endpoints, Ipv4Addr, Endpoint, MAX_DEVICES);
map_reader!(collector_endpoints, Ipv4Addr, Endpoint, MAX_COLLECTORS);
map_reader!(event_templates, EventId, Event, MAX_EVENTS);
fn delivery_facts<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Delivery>, D::Error> {
    bounded_vec::<D, Delivery, MAX_DELIVERIES>(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(deserialize_with = "device_endpoints")]
    pub devices: BTreeMap<Ipv4Addr, Endpoint>,
    #[serde(deserialize_with = "collector_endpoints")]
    pub collectors: BTreeMap<Ipv4Addr, Endpoint>,
    #[serde(deserialize_with = "event_templates")]
    pub events: BTreeMap<EventId, Event>,
    #[serde(deserialize_with = "delivery_facts")]
    pub deliveries: Vec<Delivery>,
}
impl Environment {
    pub fn validate(&self) -> Result<()> {
        if self.devices.is_empty()
            || self.devices.len() > MAX_DEVICES
            || self.collectors.is_empty()
            || self.collectors.len() > MAX_COLLECTORS
            || self.events.is_empty()
            || self.events.len() > MAX_EVENTS
            || self.deliveries.len() > MAX_DELIVERIES
        {
            return Err(invalid("Invalid bounded source/collector/event inventory."));
        }
        for (ip, e) in self.devices.iter().chain(&self.collectors) {
            if !unicast(*ip)
                || !safe_text(&e.name, 64)
                || !(1..=MAX_HISTORY as u8).contains(&e.history_capacity)
            {
                return Err(invalid("Invalid endpoint name/address/history capacity."));
            }
        }
        if self
            .devices
            .keys()
            .any(|ip| self.collectors.contains_key(ip))
        {
            return Err(invalid("Source and collector identities must be distinct."));
        }
        for (id, event) in &self.events {
            EventId::new(id.as_str())?;
            event.validate()?;
        }
        let mut seen = BTreeSet::new();
        for d in &self.deliveries {
            if !self.devices.contains_key(&d.device)
                || !self.collectors.contains_key(&d.collector)
                || !seen.insert((d.device, d.collector))
            {
                return Err(invalid("Foreign or duplicate directed delivery fact."));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevicePolicy {
    pub process_enabled: bool,
    pub buffer_threshold: Option<Severity>,
    pub remote_threshold: Option<Severity>,
    pub export_facility: Facility,
    #[serde(deserialize_with = "hosts")]
    pub hosts: BTreeSet<Ipv4Addr>,
}
impl DevicePolicy {
    fn validate(&self) -> Result<()> {
        if self.hosts.len() > MAX_HOSTS || self.hosts.iter().any(|ip| !unicast(*ip)) {
            return Err(invalid("Invalid bounded remote-host list."));
        }
        Facility::new(self.export_facility.number())?;
        Ok(())
    }
}
// JSON object keys are strings even inside serde's tagged-enum buffer. Keep
// scalar Facility fields numeric; only selector-map keys accept decimal text.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct FacilityKey(Facility);
impl<'de> Deserialize<'de> for FacilityKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Key;
        impl serde::de::Visitor<'_> for Key {
            type Value = FacilityKey;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("facility map key0–23")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                s: &str,
            ) -> std::result::Result<Self::Value, E> {
                if s.len() > 2 {
                    return Err(E::custom("Facility map key outside0–23"));
                }
                let n = s.parse::<u8>().map_err(E::custom)?;
                Facility::new(n).map(FacilityKey).map_err(E::custom)
            }
            fn visit_u64<E: serde::de::Error>(self, n: u64) -> std::result::Result<Self::Value, E> {
                let n = u8::try_from(n).map_err(E::custom)?;
                Facility::new(n).map(FacilityKey).map_err(E::custom)
            }
        }
        d.deserialize_any(Key)
    }
}
fn selectors<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<Facility, Severity>, D::Error> {
    bounded_map::<D, FacilityKey, Severity, 24>(d)
        .map(|m| m.into_iter().map(|(k, v)| (k.0, v)).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorPolicy {
    #[serde(deserialize_with = "selectors")]
    pub selectors: BTreeMap<Facility, Severity>,
}
impl CollectorPolicy {
    fn validate(&self) -> Result<()> {
        if self.selectors.len() > 24 {
            return Err(invalid("Too many collector facility selectors."));
        }
        for f in self.selectors.keys() {
            Facility::new(f.number())?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalRecord {
    pub sequence: u64,
    pub device: Ipv4Addr,
    pub event: EventId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteRecord {
    pub sequence: u64,
    pub device: Ipv4Addr,
    pub event: EventId,
    pub facility: Facility,
}
fn local_records<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<LocalRecord>, D::Error> {
    bounded_vec::<D, LocalRecord, MAX_HISTORY>(d)
}
fn remote_records<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<RemoteRecord>, D::Error> {
    bounded_vec::<D, RemoteRecord, MAX_HISTORY>(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct LocalHistory(#[serde(deserialize_with = "local_records")] Vec<LocalRecord>);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct RemoteHistory(#[serde(deserialize_with = "remote_records")] Vec<RemoteRecord>);
map_reader!(device_policies, Ipv4Addr, DevicePolicy, MAX_DEVICES);
map_reader!(
    collector_policies,
    Ipv4Addr,
    CollectorPolicy,
    MAX_COLLECTORS
);
map_reader!(local_histories, Ipv4Addr, LocalHistory, MAX_DEVICES);
map_reader!(remote_histories, Ipv4Addr, RemoteHistory, MAX_COLLECTORS);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    version: u32,
    environment: Environment,
    #[serde(deserialize_with = "device_policies")]
    devices: BTreeMap<Ipv4Addr, DevicePolicy>,
    #[serde(deserialize_with = "collector_policies")]
    collectors: BTreeMap<Ipv4Addr, CollectorPolicy>,
    #[serde(deserialize_with = "local_histories")]
    local: BTreeMap<Ipv4Addr, LocalHistory>,
    #[serde(deserialize_with = "remote_histories")]
    remote: BTreeMap<Ipv4Addr, RemoteHistory>,
    next_sequence: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BufferDisposition {
    ProcessDisabled,
    BufferDisabled,
    SeverityFiltered,
    Accepted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteDisposition {
    NotConfigured,
    ProcessDisabled,
    RemoteDisabled,
    SeverityFiltered,
    UnknownCollector,
    DeliveryUnavailable,
    CollectorFiltered,
    Accepted { facility: Facility },
}
impl BufferDisposition {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::ProcessDisabled => "logging is disabled for this buffer",
            Self::BufferDisabled => "the local buffer is disabled",
            Self::SeverityFiltered => "the event is less urgent than the local threshold permits",
            Self::Accepted => "the local buffer accepts this event",
        }
    }
}
impl RemoteDisposition {
    pub fn explanation(self) -> String {
        match self {
            Self::NotConfigured => "this collector is not configured on the source".into(),
            Self::ProcessDisabled => "remote logging is disabled on the source".into(),
            Self::RemoteDisabled => "the remote severity threshold is disabled".into(),
            Self::SeverityFiltered => {
                "the source threshold filters out this less urgent event".into()
            }
            Self::UnknownCollector => "the configured address is not a supplied collector".into(),
            Self::DeliveryUnavailable => "the supplied delivery path is unavailable".into(),
            Self::CollectorFiltered => {
                "the collector facility or severity selector rejects this event".into()
            }
            Self::Accepted { facility } => {
                format!("the collector accepts this event using {}", facility.name())
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Probe {
    pub local: BufferDisposition,
    pub remote: BTreeMap<Ipv4Addr, RemoteDisposition>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub device: Ipv4Addr,
    pub event: EventId,
    pub local: Option<BufferDisposition>,
    pub remote: BTreeMap<Ipv4Addr, RemoteDisposition>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub pass: bool,
    pub explanation: String,
}
impl State {
    pub fn new(
        environment: Environment,
        devices: BTreeMap<Ipv4Addr, DevicePolicy>,
        collectors: BTreeMap<Ipv4Addr, CollectorPolicy>,
    ) -> Result<Self> {
        environment.validate()?;
        if devices.len() != environment.devices.len()
            || collectors.len() != environment.collectors.len()
        {
            return Err(invalid(
                "Policies must match supplied endpoint inventories.",
            ));
        }
        let local = environment
            .devices
            .keys()
            .map(|ip| (*ip, LocalHistory(Vec::new())))
            .collect();
        let remote = environment
            .collectors
            .keys()
            .map(|ip| (*ip, RemoteHistory(Vec::new())))
            .collect();
        let s = Self {
            version: VERSION,
            environment,
            devices,
            collectors,
            local,
            remote,
            next_sequence: 1,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION || self.next_sequence == 0 {
            return Err(invalid("Invalid virtual syslog version/emission index."));
        }
        self.environment.validate()?;
        if !self.devices.keys().eq(self.environment.devices.keys())
            || !self
                .collectors
                .keys()
                .eq(self.environment.collectors.keys())
            || !self.local.keys().eq(self.environment.devices.keys())
            || !self.remote.keys().eq(self.environment.collectors.keys())
        {
            return Err(invalid(
                "Runtime endpoint keys differ from supplied environment.",
            ));
        }
        for p in self.devices.values() {
            p.validate()?;
        }
        for p in self.collectors.values() {
            p.validate()?;
        }
        let mut emissions: BTreeMap<u64, (Ipv4Addr, EventId, Option<Facility>)> = BTreeMap::new();
        let mut record = |seq: u64,
                          device: Ipv4Addr,
                          event: &EventId,
                          facility: Option<Facility>|
         -> Result<()> {
            if seq == 0
                || seq >= self.next_sequence
                || !self.devices.contains_key(&device)
                || !self.environment.events.contains_key(event)
            {
                return Err(invalid(
                    "Foreign history identity or invalid emission index.",
                ));
            }
            if let Some(old) = emissions.get_mut(&seq) {
                if old.0 != device
                    || old.1 != *event
                    || matches!((old.2, facility), (Some(a), Some(b)) if a != b)
                {
                    return Err(invalid(
                        "One emission index cannot describe different events or export facilities.",
                    ));
                }
                if facility.is_some() {
                    old.2 = facility;
                }
            } else {
                emissions.insert(seq, (device, event.clone(), facility));
            }
            Ok(())
        };
        for (ip, h) in &self.local {
            if h.0.len() > self.environment.devices[ip].history_capacity as usize {
                return Err(invalid("Local circular history exceeds supplied capacity."));
            }
            let mut previous = 0;
            for r in &h.0 {
                if r.device != *ip || r.sequence <= previous {
                    return Err(invalid("Local history order or owner differs."));
                }
                record(r.sequence, r.device, &r.event, None)?;
                previous = r.sequence;
            }
        }
        for (ip, h) in &self.remote {
            if h.0.len() > self.environment.collectors[ip].history_capacity as usize {
                return Err(invalid("Collector history exceeds supplied capacity."));
            }
            let mut previous = 0;
            for r in &h.0 {
                if r.sequence <= previous
                    || !self
                        .environment
                        .deliveries
                        .iter()
                        .any(|d| d.device == r.device && d.collector == *ip && d.available)
                {
                    return Err(invalid(
                        "Collector history order or immutable supplied delivery differs.",
                    ));
                }
                record(r.sequence, r.device, &r.event, Some(r.facility))?;
                Facility::new(r.facility.number())?;
                previous = r.sequence;
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let raw =
            serde_json::to_vec(self).map_err(|_| invalid("Cannot encode virtual syslog state."))?;
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Virtual syslog state exceeds raw bound.",
            ));
        }
        Ok(raw)
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "Virtual syslog JSON exceeds raw bound before decoding.",
            ));
        }
        let s: Self = serde_json::from_slice(raw).map_err(|_| {
            invalid(
                "Malformed bounded syslog JSON, duplicate/unknown fields or invalid typed value.",
            )
        })?;
        s.validate()?;
        Ok(s)
    }
    pub fn environment(&self) -> Result<&Environment> {
        self.validate()?;
        Ok(&self.environment)
    }
    pub fn environment_matches(&self, expected: &Environment) -> Result<bool> {
        self.validate()?;
        expected.validate()?;
        Ok(&self.environment == expected)
    }
    pub fn device_policy(&self, ip: Ipv4Addr) -> Result<&DevicePolicy> {
        self.validate()?;
        self.devices
            .get(&ip)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown source device."))
    }
    pub fn collector_policy(&self, ip: Ipv4Addr) -> Result<&CollectorPolicy> {
        self.validate()?;
        self.collectors
            .get(&ip)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown collector."))
    }
    pub fn local_history(&self, ip: Ipv4Addr) -> Result<&[LocalRecord]> {
        self.validate()?;
        self.local
            .get(&ip)
            .map(|x| x.0.as_slice())
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown source buffer."))
    }
    pub fn collector_history(&self, ip: Ipv4Addr) -> Result<&[RemoteRecord]> {
        self.validate()?;
        self.remote
            .get(&ip)
            .map(|x| x.0.as_slice())
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown collector inbox."))
    }
    pub fn set_device_policy(&mut self, ip: Ipv4Addr, policy: DevicePolicy) -> Result<()> {
        self.validate()?;
        policy.validate()?;
        if !self.devices.contains_key(&ip) {
            return Err(error(ErrorKind::Unknown, "Unknown source device."));
        }
        let mut candidate = self.clone();
        candidate.devices.insert(ip, policy);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    pub fn set_collector_policy(&mut self, ip: Ipv4Addr, policy: CollectorPolicy) -> Result<()> {
        self.validate()?;
        policy.validate()?;
        if !self.collectors.contains_key(&ip) {
            return Err(error(ErrorKind::Unknown, "Unknown collector."));
        }
        let mut candidate = self.clone();
        candidate.collectors.insert(ip, policy);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    fn event(&self, device: Ipv4Addr, event: &EventId) -> Result<(&DevicePolicy, &Event)> {
        let p = self
            .devices
            .get(&device)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown source device."))?;
        let e = self
            .environment
            .events
            .get(event)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown supplied event template."))?;
        Ok((p, e))
    }
    fn local_decision(policy: &DevicePolicy, severity: Severity) -> BufferDisposition {
        if !policy.process_enabled {
            BufferDisposition::ProcessDisabled
        } else {
            match policy.buffer_threshold {
                None => BufferDisposition::BufferDisabled,
                Some(limit) if !limit.includes(severity) => BufferDisposition::SeverityFiltered,
                _ => BufferDisposition::Accepted,
            }
        }
    }
    fn remote_decision(
        &self,
        device: Ipv4Addr,
        policy: &DevicePolicy,
        severity: Severity,
        target: Ipv4Addr,
    ) -> RemoteDisposition {
        if !policy.hosts.contains(&target) {
            return RemoteDisposition::NotConfigured;
        }
        if !policy.process_enabled {
            return RemoteDisposition::ProcessDisabled;
        }
        let Some(limit) = policy.remote_threshold else {
            return RemoteDisposition::RemoteDisabled;
        };
        if !limit.includes(severity) {
            return RemoteDisposition::SeverityFiltered;
        }
        let Some(collector) = self.collectors.get(&target) else {
            return RemoteDisposition::UnknownCollector;
        };
        if !self
            .environment
            .deliveries
            .iter()
            .any(|d| d.device == device && d.collector == target && d.available)
        {
            return RemoteDisposition::DeliveryUnavailable;
        }
        if collector
            .selectors
            .get(&policy.export_facility)
            .is_none_or(|limit| !limit.includes(severity))
        {
            return RemoteDisposition::CollectorFiltered;
        }
        RemoteDisposition::Accepted {
            facility: policy.export_facility,
        }
    }
    pub fn current_probe(&self, device: Ipv4Addr, event: &EventId) -> Result<Probe> {
        self.validate()?;
        let (p, e) = self.event(device, event)?;
        Ok(Probe {
            local: Self::local_decision(p, e.severity),
            remote: p
                .hosts
                .iter()
                .map(|ip| (*ip, self.remote_decision(device, p, e.severity, *ip)))
                .collect(),
        })
    }
    pub fn current_remote(
        &self,
        device: Ipv4Addr,
        event: &EventId,
        target: Ipv4Addr,
    ) -> Result<RemoteDisposition> {
        self.validate()?;
        if !unicast(target) {
            return Err(invalid("Invalid remote probe address."));
        }
        let (p, e) = self.event(device, event)?;
        Ok(self.remote_decision(device, p, e.severity, target))
    }
    pub fn emit(&mut self, device: Ipv4Addr, event: &EventId) -> Result<Probe> {
        let probe = self.current_probe(device, event)?;
        let next = self.next_sequence.checked_add(1).ok_or_else(|| {
            error(
                ErrorKind::Limit,
                "Simulator emission index exhausted; no history was changed.",
            )
        })?;
        let sequence = self.next_sequence;
        let mut candidate = self.clone();
        candidate.next_sequence = next;
        if probe.local == BufferDisposition::Accepted {
            let h = &mut candidate.local.get_mut(&device).unwrap().0;
            h.push(LocalRecord {
                sequence,
                device,
                event: event.clone(),
            });
            if h.len() > candidate.environment.devices[&device].history_capacity as usize {
                h.remove(0);
            }
        }
        for (ip, status) in &probe.remote {
            if let RemoteDisposition::Accepted { facility } = status {
                let h = &mut candidate.remote.get_mut(ip).unwrap().0;
                h.push(RemoteRecord {
                    sequence,
                    device,
                    event: event.clone(),
                    facility: *facility,
                });
                if h.len() > candidate.environment.collectors[ip].history_capacity as usize {
                    h.remove(0);
                }
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(probe)
    }
    pub fn clear_buffer(&mut self, device: Ipv4Addr) -> Result<()> {
        self.validate()?;
        let h = self
            .local
            .get_mut(&device)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown source buffer."))?;
        h.0.clear();
        Ok(())
    }
    pub fn clear_collector_history(&mut self, collector: Ipv4Addr) -> Result<()> {
        self.validate()?;
        let h = self
            .remote
            .get_mut(&collector)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown collector inbox."))?;
        h.0.clear();
        Ok(())
    }
    pub fn checks(&self, requirements: &[Requirement]) -> Result<Vec<Check>> {
        self.validate()?;
        if requirements.is_empty() || requirements.len() > MAX_REQUIREMENTS {
            return Err(invalid("Need1–32bounded current-policy requirements."));
        }
        let mut seen = BTreeSet::new();
        for r in requirements {
            if (r.local.is_none() && r.remote.is_empty())
                || r.remote.len() > MAX_HOSTS
                || !seen.insert((r.device, &r.event))
            {
                return Err(invalid("Empty, duplicate or oversized policy requirement."));
            }
            self.event(r.device, &r.event)?;
            if r.remote.keys().any(|ip| !self.collectors.contains_key(ip)) {
                return Err(invalid("Foreign collector identity in policy requirement."));
            }
        }
        let mut out = Vec::new();
        let mut bytes = 0usize;
        for r in requirements {
            let p = self.current_probe(r.device, &r.event)?;
            if let Some(want) = r.local {
                push_check(
                    &mut out,
                    &mut bytes,
                    Check {
                        pass: p.local == want,
                        explanation: format!(
                            "{} / {}: local policy {}; required {}. Historical records do not decide this check.",
                            r.device,
                            r.event.as_str(),
                            p.local.explanation(),
                            want.explanation()
                        ),
                    },
                )?;
            }
            for (ip, want) in &r.remote {
                let actual = self.current_remote(r.device, &r.event, *ip)?;
                push_check(
                    &mut out,
                    &mut bytes,
                    Check {
                        pass: actual == *want,
                        explanation: format!(
                            "{} / {} to {}: current policy {}; required {}.",
                            r.device,
                            r.event.as_str(),
                            ip,
                            actual.explanation(),
                            want.explanation()
                        ),
                    },
                )?;
            }
        }
        Ok(out)
    }
    pub fn show_device(&self, device: Ipv4Addr) -> Result<String> {
        self.validate()?;
        let p = self.device_policy(device)?;
        let mut out = format!(
            "{} ({device})\nVirtual logging process: {}\nLocal threshold: {}\nRemote threshold: {}\nExport facility: {} ({})\n",
            self.environment.devices[&device].name,
            if p.process_enabled {
                "Enabled"
            } else {
                "Disabled for modeled buffer and remote outputs"
            },
            threshold(p.buffer_threshold),
            threshold(p.remote_threshold),
            p.export_facility.name(),
            p.export_facility.number()
        );
        for ip in &p.hosts {
            out.push_str(&format!(
                "Configured remote: {ip}{}\n",
                if self.collectors.contains_key(ip) {
                    ""
                } else {
                    " (unknown supplied collector)"
                }
            ));
        }
        out.push_str("Historical local buffer, oldest first (simulator emission indices):\n");
        for r in &self.local[&device].0 {
            out.push_str(&format!(
                "#{} {}\n",
                r.sequence,
                self.environment.events[&r.event].cisco_style_text()?
            ));
        }
        bounded_output(out)
    }
    pub fn show_collector(&self, collector: Ipv4Addr) -> Result<String> {
        self.validate()?;
        let p = self.collector_policy(collector)?;
        let mut out = format!(
            "{} ({collector})\nVirtual facility selectors:\n",
            self.environment.collectors[&collector].name
        );
        for (f, s) in &p.selectors {
            out.push_str(&format!(
                "{}: severity0 through{} ({})\n",
                f.name(),
                s.number(),
                s.name()
            ));
        }
        out.push_str(
            "Historical collector inbox, oldest first; receipt is not current-policy proof:\n",
        );
        for r in &self.remote[&collector].0 {
            let e = &self.environment.events[&r.event];
            out.push_str(&format!(
                "#{} from{} PRI={} {} | {}\n",
                r.sequence,
                r.device,
                r.facility.priority(e.severity),
                r.facility.name(),
                e.cisco_style_text()?
            ));
        }
        bounded_output(out)
    }
}
fn threshold(s: Option<Severity>) -> String {
    s.map_or_else(
        || "Disabled".into(),
        |s| format!("0 through{} ({})", s.number(), s.name()),
    )
}
fn bounded_output(s: String) -> Result<String> {
    if s.len() > MAX_OUTPUT {
        Err(error(
            ErrorKind::Limit,
            "Virtual syslog view exceeds bounded output.",
        ))
    } else {
        Ok(s)
    }
}
fn push_check(out: &mut Vec<Check>, bytes: &mut usize, c: Check) -> Result<()> {
    *bytes = bytes
        .checked_add(c.explanation.len())
        .ok_or_else(|| error(ErrorKind::Limit, "Check output bound."))?;
    if *bytes > MAX_OUTPUT {
        return Err(error(ErrorKind::Limit, "Check output bound."));
    }
    out.push(c);
    Ok(())
}
