//! Bounded one-way transit ACL inspection. No host packets, I/O or persistent effects.
use crate::{
    acl,
    ipv4_routing::{Boundary, ForwardResult, Selected, Topology, Trace},
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, marker::PhantomData, ops::ControlFlow};

pub const MAX_JSON_BYTES: usize = 2_000_000;
pub const MAX_FLOW_BYTES: usize = 1024;
pub const MAX_EVENTS: usize = 48;
pub const MAX_POSITIVES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    Invalid,
    Unsupported,
    Routing,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    pub message: String,
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Failure {}
fn fail(kind: FailureKind, message: impl Into<String>) -> Failure {
    Failure {
        kind,
        message: message.into(),
    }
}
fn classifier_failure(e: acl::Error) -> Failure {
    let kind = match e.kind {
        acl::ErrorKind::Unsupported | acl::ErrorKind::UnknownList => FailureKind::Unsupported,
        acl::ErrorKind::Limit => FailureKind::Limit,
        acl::ErrorKind::Invalid => FailureKind::Invalid,
    };
    fail(kind, e.message)
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
struct Id(String);
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Name;
        impl Visitor<'_> for Name {
            type Value = Id;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a 1 to 32 byte ASCII identifier")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Id, E> {
                if !name(value) {
                    return Err(E::custom("Invalid ACL routing identifier"));
                }
                Ok(Id(value.into()))
            }
        }
        d.deserialize_str(Name)
    }
}
fn id<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Id::deserialize(d).map(|v| v.0)
}
fn optional_id<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::<Id>::deserialize(d).map(|v| v.map(|v| v.0))
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct Map<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Map<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique ACL routing identifiers")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Id(key)) = a.next_key()? {
                if out.len() >= N || out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate key or ACL routing map limit"));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Map::<V, N>(PhantomData))
}
fn devices<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, DevicePolicy>, D::Error> {
    map::<D, DevicePolicy, 12>(d)
}
fn ports<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Attachment>, D::Error> {
    map::<D, Attachment, 8>(d)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Inbound,
    Outbound,
}
fn hosts<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, String>, D::Error> {
    map::<D, Id, 12>(d).map(|entries| {
        entries
            .into_iter()
            .map(|(key, Id(port))| (key, port))
            .collect()
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "id")]
    pub device: String,
    #[serde(deserialize_with = "id")]
    pub interface: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub source: Endpoint,
    pub destination: Endpoint,
    pub protocol: acl::PacketProtocol,
}
impl Flow {
    pub fn from_json(bytes: &[u8]) -> Result<Self, Failure> {
        if bytes.len() > MAX_FLOW_BYTES {
            return Err(fail(FailureKind::Limit, "ACL flow exceeds 1024 bytes."));
        }
        let flow: Self = serde_json::from_slice(bytes).map_err(|_| {
            fail(
                FailureKind::Invalid,
                "Malformed or unsupported ACL flow fields.",
            )
        })?;
        flow.validate()?;
        Ok(flow)
    }
    pub fn validate(&self) -> Result<(), Failure> {
        for e in [&self.source, &self.destination] {
            if !name(&e.device) || !name(&e.interface) {
                return Err(fail(FailureKind::Invalid, "Invalid endpoint identifier."));
            }
        }
        if self.source.device == self.destination.device {
            return Err(fail(
                FailureKind::Unsupported,
                "A transit flow needs two distinct endpoint devices.",
            ));
        }
        if matches!(
            self.protocol,
            acl::PacketProtocol::Other { number: 1 | 6 | 17 }
        ) {
            return Err(fail(
                FailureKind::Invalid,
                "Use structured ICMP, TCP or UDP packet fields.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    #[serde(default, deserialize_with = "optional_id")]
    pub inbound: Option<String>,
    #[serde(default, deserialize_with = "optional_id")]
    pub outbound: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevicePolicy {
    pub policy: acl::Policy,
    #[serde(deserialize_with = "ports")]
    pub attachments: BTreeMap<String, Attachment>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub schema_version: u32,
    pub topology: Topology,
    /// Authored host identity, separate from mutable forwarding configuration.
    #[serde(deserialize_with = "hosts")]
    pub host_endpoints: BTreeMap<String, String>,
    #[serde(deserialize_with = "devices")]
    pub devices: BTreeMap<String, DevicePolicy>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    #[serde(deserialize_with = "id")]
    pub device: String,
    #[serde(deserialize_with = "id")]
    pub interface: String,
    pub direction: Direction,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Boundary {
        at: Location,
        decision: Option<acl::Decision>,
    },
    RouteSelected {
        device: String,
        selected: Selected,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Denial {
    pub at: Location,
    pub decision: acl::Decision,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Baseline {
    Deliverable(Trace),
    Failed(Failure),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Delivered {
        trace: Trace,
        events: Vec<Event>,
    },
    Denied {
        denial: Denial,
        events: Vec<Event>,
    },
    Failed {
        failure: Failure,
        events: Vec<Event>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inspection {
    pub packet: acl::Packet,
    pub baseline: Baseline,
    pub actual: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedCause {
    Explicit {
        #[serde(default, deserialize_with = "optional_id")]
        rule: Option<String>,
    },
    Implicit {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedDenial {
    pub at: Location,
    pub cause: ExpectedCause,
}
impl ExpectedDenial {
    pub fn valid_for(&self, network: &Network) -> bool {
        name(&self.at.device)
            && name(&self.at.interface)
            && network
                .topology
                .devices
                .get(&self.at.device)
                .is_some_and(|d| d.interfaces.contains_key(&self.at.interface))
            && match &self.cause {
                ExpectedCause::Explicit { rule: Some(id) } => name(id),
                _ => true,
            }
    }
}
impl Inspection {
    pub fn delivered(&self) -> bool {
        matches!(self.baseline, Baseline::Deliverable(_))
            && matches!(self.actual, Outcome::Delivered { .. })
    }
    pub fn denied_at(&self, expected: &ExpectedDenial) -> bool {
        if !matches!(self.baseline, Baseline::Deliverable(_)) {
            return false;
        }
        let Outcome::Denied { denial, .. } = &self.actual else {
            return false;
        };
        denial.at == expected.at
            && match (&expected.cause, &denial.decision.cause) {
                (ExpectedCause::Implicit {}, acl::Cause::ImplicitDeny) => true,
                (ExpectedCause::Explicit { rule }, acl::Cause::Matched { rule: hit, .. }) => {
                    rule.as_ref().is_none_or(|rule| rule == hit)
                }
                _ => false,
            }
    }
    fn crosses(&self, at: &Location) -> bool {
        let Outcome::Delivered { events, .. } = &self.actual else {
            return false;
        };
        self.delivered()
            && events
                .iter()
                .any(|e| matches!(e,Event::Boundary{at:observed,..}if observed==at))
    }
}
/// A small pure guard, not a complete authored-goal validator. Always re-evaluate current state.
pub fn guarded_denial(
    negative: &Inspection,
    expected: &ExpectedDenial,
    required_positives: &[&Inspection],
) -> bool {
    !required_positives.is_empty()
        && required_positives.len() <= MAX_POSITIVES
        && negative.denied_at(expected)
        && required_positives.iter().all(|p| p.delivered())
        && required_positives.iter().any(|p| p.crosses(&expected.at))
}
impl Network {
    pub fn validate(&self) -> Result<(), Failure> {
        if self.schema_version != 1 {
            return Err(fail(
                FailureKind::Unsupported,
                "Unsupported ACL network version.",
            ));
        }
        if !self.topology.valid() || self.devices.len() > 12 || self.host_endpoints.len() > 12 {
            return Err(fail(
                FailureKind::Invalid,
                "Invalid or oversized IPv4 ACL topology.",
            ));
        }
        for (device, port) in &self.host_endpoints {
            if !name(device)
                || !name(port)
                || !self
                    .topology
                    .devices
                    .get(device)
                    .is_some_and(|d| d.interfaces.contains_key(port))
            {
                return Err(fail(
                    FailureKind::Invalid,
                    "Unknown or malformed declared host endpoint.",
                ));
            }
            if self.topology.devices[device].forwarding {
                return Err(fail(
                    FailureKind::Unsupported,
                    "Declared host endpoints cannot become forwarding routers in this transit-only model.",
                ));
            }
        }
        let mut total_lists = 0usize;
        let mut total_rules = 0usize;
        // Check counts/scalars before derived policy validation allocates ID sets.
        for (device, config) in &self.devices {
            if !name(device)
                || !self.topology.devices.contains_key(device)
                || config.attachments.len() > 8
                || config.policy.lists.len() > acl::MAX_LISTS
            {
                return Err(fail(
                    FailureKind::Invalid,
                    "Unknown device or invalid ACL policy shape.",
                ));
            }
            total_lists += config.policy.lists.len();
            for list in config.policy.lists.values() {
                if list.rules.len() > acl::MAX_RULES {
                    return Err(fail(FailureKind::Limit, "ACL rule limit exceeded."));
                }
                total_rules += list.rules.len();
            }
            if total_lists > acl::MAX_LISTS || total_rules > acl::MAX_LISTS * acl::MAX_RULES {
                return Err(fail(
                    FailureKind::Limit,
                    "Network supports at most eight ACLs and 512 total rules.",
                ));
            }
            for (port, a) in &config.attachments {
                if !name(port)
                    || !self.topology.devices[device].interfaces.contains_key(port)
                    || [&a.inbound, &a.outbound]
                        .iter()
                        .any(|v| v.as_ref().is_some_and(|v| !name(v)))
                {
                    return Err(fail(
                        FailureKind::Invalid,
                        "Unknown interface or malformed ACL attachment.",
                    ));
                }
                if self.host_endpoints.contains_key(device)
                    && (a.inbound.is_some() || a.outbound.is_some())
                {
                    return Err(fail(
                        FailureKind::Unsupported,
                        "ACL attachments on a declared host endpoint are outside the transit-only subset.",
                    ));
                }
            }
        }
        for config in self.devices.values() {
            config.policy.validate().map_err(classifier_failure)?;
            for a in config.attachments.values() {
                for name in [&a.inbound, &a.outbound].into_iter().flatten() {
                    if !config.policy.lists.contains_key(name) {
                        return Err(fail(
                            FailureKind::Unsupported,
                            "An attached ACL name is missing; undefined-list attachment behavior is unsupported.",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self, Failure> {
        if bytes.len() > MAX_JSON_BYTES {
            return Err(fail(FailureKind::Limit, "ACL network exceeds 2 MB."));
        }
        let network: Self = serde_json::from_slice(bytes).map_err(|_| {
            fail(
                FailureKind::Invalid,
                "Malformed or unsupported ACL network fields, identifiers or duplicate maps.",
            )
        })?;
        network.validate()?;
        Ok(network)
    }
    /// Future content/import adapters must pin both physical shape and authored host roles.
    pub fn same_identity(&self, original: &Self) -> bool {
        self.validate().is_ok()
            && original.validate().is_ok()
            && self.host_endpoints == original.host_endpoints
            && self.topology.same_shape(&original.topology)
    }
    fn endpoint_address(&self, e: &Endpoint) -> Result<std::net::Ipv4Addr, Failure> {
        if self.host_endpoints.get(&e.device) != Some(&e.interface) {
            return Err(fail(
                FailureKind::Unsupported,
                "Only the declared host device/interface pairs can originate or terminate a transit ACL test.",
            ));
        }
        let d = self
            .topology
            .devices
            .get(&e.device)
            .ok_or_else(|| fail(FailureKind::Invalid, "Unknown endpoint device."))?;
        if d.forwarding {
            return Err(fail(
                FailureKind::Unsupported,
                "Router-originated or router-terminated ACL traffic is outside the transit subset.",
            ));
        }
        if self.devices.get(&e.device).is_some_and(|c| {
            c.attachments
                .values()
                .any(|a| a.inbound.is_some() || a.outbound.is_some())
        }) {
            return Err(fail(
                FailureKind::Unsupported,
                "Endpoint ACL attachments are outside the transit-only subset.",
            ));
        }
        let i = d
            .interfaces
            .get(&e.interface)
            .ok_or_else(|| fail(FailureKind::Invalid, "Unknown bound endpoint interface."))?;
        let a = i
            .address
            .as_ref()
            .filter(|a| i.up && a.usable())
            .ok_or_else(|| {
                fail(
                    FailureKind::Routing,
                    "The bound endpoint interface needs an up, usable IPv4 address.",
                )
            })?;
        let owners = self
            .topology
            .devices
            .values()
            .flat_map(|d| d.interfaces.values())
            .filter(|i| {
                i.up && i
                    .address
                    .as_ref()
                    .is_some_and(|other| other.address == a.address)
            })
            .count();
        if owners != 1 {
            return Err(fail(
                FailureKind::Routing,
                "The bound endpoint address has duplicate active owners.",
            ));
        }
        Ok(a.address)
    }
    fn check_trace(&self, trace: Trace, flow: &Flow) -> Result<Trace, Failure> {
        if trace.destination_device != flow.destination.device {
            return Err(fail(
                FailureKind::Routing,
                "The packet reached a different owner than the bound destination.",
            ));
        }
        if trace.hops.len() < 2 {
            return Err(fail(
                FailureKind::Unsupported,
                "A transit ACL flow must cross at least one intermediate router; same-link endpoint traffic is unsupported.",
            ));
        }
        Ok(trace)
    }
    fn boundary(
        &self,
        at: &Location,
        packet: &acl::Packet,
    ) -> Result<Option<acl::Decision>, Failure> {
        let Some(c) = self.devices.get(&at.device) else {
            return Ok(None);
        };
        let Some(a) = c.attachments.get(&at.interface) else {
            return Ok(None);
        };
        let name = match at.direction {
            Direction::Inbound => &a.inbound,
            Direction::Outbound => &a.outbound,
        };
        name.as_ref()
            .map(|name| c.policy.evaluate(name, packet).map_err(classifier_failure))
            .transpose()
    }
    pub fn inspect(&self, flow: &Flow) -> Result<Inspection, Failure> {
        self.validate()?;
        flow.validate()?;
        let packet = acl::Packet {
            source: self.endpoint_address(&flow.source)?,
            destination: self.endpoint_address(&flow.destination)?,
            protocol: flow.protocol,
        };
        packet
            .validate()
            .map_err(|e| fail(FailureKind::Invalid, e.message))?;
        let first = self
            .topology
            .select(&flow.source.device, packet.destination)
            .map_err(|e| fail(FailureKind::Routing, e))?;
        if first.interface != flow.source.interface {
            return Err(fail(
                FailureKind::Unsupported,
                "The bound source interface must be the selected initial egress; alternate source-address policy is unsupported.",
            ));
        }
        let baseline =
            match self
                .topology
                .path_controlled(&flow.source.device, packet.destination, |_| {
                    Ok(ControlFlow::<()>::Continue(()))
                }) {
                Ok(ForwardResult::Delivered(trace)) => match self.check_trace(trace, flow) {
                    Ok(trace) => Baseline::Deliverable(trace),
                    Err(e) if e.kind == FailureKind::Unsupported => return Err(e),
                    Err(e) => Baseline::Failed(e),
                },
                Err(e) => Baseline::Failed(fail(FailureKind::Routing, e)),
                Ok(ForwardResult::Interrupted(())) => {
                    unreachable!("No-ACL baseline cannot interrupt")
                }
            };
        let mut events = Vec::new();
        let actual =
            self.topology
                .path_controlled(&flow.source.device, packet.destination, |boundary| {
                    let at = match boundary {
                        Boundary::Inbound { device, interface } => Location {
                            device: device.into(),
                            interface: interface.into(),
                            direction: Direction::Inbound,
                        },
                        Boundary::Outbound { device, selected } => {
                            if events.len() >= MAX_EVENTS {
                                return Err("ACL event limit reached".into());
                            }
                            events.push(Event::RouteSelected {
                                device: device.into(),
                                selected: selected.clone(),
                            });
                            Location {
                                device: device.into(),
                                interface: selected.interface.clone(),
                                direction: Direction::Outbound,
                            }
                        }
                    };
                    if events.len() >= MAX_EVENTS {
                        return Err("ACL event limit reached".into());
                    }
                    let decision = self.boundary(&at, &packet).map_err(|e| e.message)?;
                    events.push(Event::Boundary {
                        at: at.clone(),
                        decision: decision.clone(),
                    });
                    Ok(match decision {
                        Some(decision) if decision.action == acl::Action::Deny => {
                            ControlFlow::Break(Denial { at, decision })
                        }
                        _ => ControlFlow::Continue(()),
                    })
                });
        let actual = match actual {
            Ok(ForwardResult::Delivered(trace)) => match self.check_trace(trace, flow) {
                Ok(trace) => Outcome::Delivered { trace, events },
                Err(failure) => Outcome::Failed { failure, events },
            },
            Ok(ForwardResult::Interrupted(denial)) => Outcome::Denied { denial, events },
            Err(message) => Outcome::Failed {
                failure: fail(FailureKind::Routing, message),
                events,
            },
        };
        Ok(Inspection {
            packet,
            baseline,
            actual,
        })
    }
}
