//! Bounded observation of supplied initial DHCP exchanges over virtual Ethernet.
//! No packet bytes, allocator, sockets, host interfaces, processes or timers.
use crate::{
    mac_learning::{Mac, Observation, State, access_outputs::AccessOutputs},
    switching::{Jack, Mode},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
pub const MAX_RECORDS: usize = 64;
pub const MAX_JSON: usize = 768 * 1024;
pub const MAX_LEASE: u32 = 86_400;
pub const TRANSACTION_SECONDS: u64 = 60;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
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
fn error(kind: ErrorKind, text: &str) -> Error {
    Error {
        kind,
        message: text.into(),
    }
}
fn invalid(text: &str) -> Error {
    error(ErrorKind::Invalid, text)
}
fn unsupported(text: &str) -> Error {
    error(ErrorKind::Unsupported, text)
}
fn forwarding_error(message: String) -> Error {
    if message.contains("limit") || message.contains("exceeds") {
        error(
            ErrorKind::Limit,
            "A bounded forwarding or simulated-clock limit was reached; no state changed.",
        )
    } else {
        invalid("The supplied bounded forwarding state or frame is invalid; no state changed.")
    }
}
fn unicast(ip: Ipv4Addr) -> bool {
    let first = ip.octets()[0];
    first > 0 && first < 224 && first != 127 && !ip.is_broadcast()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortTrust {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub port: String,
    pub trusted: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u16,16>")]
    pub vlans: Vec<u16>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PortTrust,16>")]
    pub ports: Vec<PortTrust>,
    pub verify_source: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Message {
    Discover,
    Request,
    Offer { address: Ipv4Addr },
    Ack { address: Ipv4Addr, lease: u32 },
    Release { address: Ipv4Addr },
    Nak,
    Decline,
    Inform,
    LeaseQuery,
}
impl Message {
    fn server(self) -> bool {
        matches!(
            self,
            Self::Offer { .. } | Self::Ack { .. } | Self::Nak | Self::LeaseQuery
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packet {
    pub source: Mac,
    pub destination: Mac,
    pub transaction: u32,
    pub client: Mac,
    pub relay: Ipv4Addr,
    pub option82: bool,
    pub message: Message,
}
impl Packet {
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > 4096 {
            return Err(error(ErrorKind::Limit, "DHCP metadata input exceeds4KiB."));
        }
        let p: Self = serde_json::from_str(text).map_err(|_| {
            invalid("Invalid or unsupported DHCP metadata. Use the disclosed typed message fields.")
        })?;
        p.validate()?;
        Ok(p)
    }
    fn validate(&self) -> Result<()> {
        if !self.source.unicast()
            || !self.client.unicast()
            || (!self.destination.unicast() && self.destination != Mac::BROADCAST)
        {
            return Err(invalid(
                "Use nonzero unicast source/client MACs and an individual or broadcast destination.",
            ));
        }
        let address = match self.message {
            Message::Offer { address }
            | Message::Ack { address, .. }
            | Message::Release { address } => Some(address),
            _ => None,
        };
        if address.is_some_and(|ip| !unicast(ip)) {
            return Err(invalid(
                "Use an ordinary unicast IPv4 address for this supplied exchange.",
            ));
        }
        if matches!(self.message,Message::Ack{lease,..}if !(1..=MAX_LEASE).contains(&lease)) {
            return Err(invalid(
                "A supported ACK lease is1–86400 simulated seconds.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Discover,
    Request,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    transaction: u32,
    client: Mac,
    vlan: u16,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    host: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    port: String,
    phase: Phase,
    untrusted: bool,
    expires: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub client: Mac,
    pub address: Ipv4Addr,
    pub vlan: u16,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub port: String,
    pub expires: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct World {
    schema_version: u8,
    baseline: State,
    active: State,
    information_option_disabled: bool,
    config: Config,
    pending: Vec<Pending>,
    bindings: Vec<Binding>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u8,
    baseline: State,
    active: State,
    information_option_disabled: bool,
    config: Config,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Pending,64>")]
    pending: Vec<Pending>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Binding,64>")]
    bindings: Vec<Binding>,
}
impl<'de> Deserialize<'de> for World {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let w = Wire::deserialize(d)?;
        let s = Self {
            schema_version: w.schema_version,
            baseline: w.baseline,
            active: w.active,
            information_option_disabled: w.information_option_disabled,
            config: w.config,
            pending: w.pending,
            bindings: w.bindings,
        };
        s.validate().map_err(serde::de::Error::custom)?;
        Ok(s)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    UntrustedServer,
    SourceMismatch,
    RelayMetadata,
    WrongReleasePort,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Unavailable {
        reason: String,
    },
    Bypassed {
        forwarding: Observation,
    },
    Rejected {
        cause: Rejection,
    },
    Forwarded {
        forwarding: Observation,
        learned: Option<Binding>,
        removed: bool,
    },
}
impl World {
    pub fn new(baseline: State, information_option_disabled: bool) -> Result<Self> {
        let ports = baseline
            .fabric()
            .switches
            .values()
            .next()
            .map(|s| {
                s.ports
                    .keys()
                    .map(|p| PortTrust {
                        port: p.clone(),
                        trusted: false,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let s = Self {
            schema_version: 1,
            active: baseline.clone(),
            baseline,
            information_option_disabled,
            config: Config {
                enabled: false,
                vlans: vec![],
                ports,
                verify_source: true,
            },
            pending: vec![],
            bindings: vec![],
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_JSON {
            return Err(error(
                ErrorKind::Limit,
                "DHCP observer state exceeds768KiB.",
            ));
        }
        serde_json::from_str(text).map_err(|_| {
            invalid("Invalid, unsupported or oversized versioned DHCP observer state.")
        })
    }
    pub fn baseline(&self) -> &State {
        &self.baseline
    }
    pub fn forwarding(&self) -> &State {
        &self.active
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    fn switch(&self) -> &str {
        self.baseline.fabric().switches.keys().next().unwrap()
    }
    fn trust(&self, port: &str) -> bool {
        self.config
            .ports
            .iter()
            .any(|p| p.port == port && p.trusted)
    }
    fn attachment(&self, host: &str) -> Result<(String, u16)> {
        let endpoint = Jack::Host { host: host.into() };
        let f = self.active.fabric();
        let peer = f
            .cables
            .iter()
            .find_map(|c| {
                if c.a == endpoint {
                    Some(&c.b)
                } else if c.b == endpoint {
                    Some(&c.a)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                unsupported("This observer requires a directly attached supplied endpoint.")
            })?;
        match peer {
            Jack::Switch { switch, port } if switch == self.switch() => {
                match f.switches[switch].ports[port].mode {
                    Mode::Access { vlan } => Ok((port.clone(), vlan)),
                    _ => Err(unsupported("Trunks are outside this DHCP observer subset.")),
                }
            }
            _ => Err(unsupported("Use the supplied single access switch.")),
        }
    }
    fn source_failure(&self, host: &str, port: &str, vlan: u16) -> Option<&'static str> {
        let f = self.active.fabric();
        if !f.hosts[host].up {
            return Some("The source endpoint is stopped; no message reached the switch.");
        }
        if !f.switches[self.switch()].ports[port].up {
            return Some("The source access port is down; no message reached the switch.");
        }
        if !f.switches[self.switch()].vlans.contains(&vlan) {
            return Some("The source port's VLAN is inactive; no message reached the switch.");
        }
        if !f.cables.iter().any(|c| {
            c.up && (c.a == Jack::Host { host: host.into() }
                || c.b == Jack::Host { host: host.into() })
        }) {
            return Some("The source cable is down; no message reached the switch.");
        }
        None
    }
    fn validate_config(&self, c: &Config) -> Result<()> {
        let f = self.active.fabric();
        let sw = &f.switches[self.switch()];
        if c.vlans.len() > 16
            || c.ports.len() != sw.ports.len()
            || c.vlans.iter().any(|v| !sw.vlans.contains(v))
            || c.vlans.iter().collect::<BTreeSet<_>>().len() != c.vlans.len()
            || c.ports
                .iter()
                .map(|p| p.port.as_str())
                .collect::<BTreeSet<_>>()
                != sw.ports.keys().map(String::as_str).collect()
        {
            return Err(invalid(
                "Use unique supplied VLANs and exactly one DHCP trust setting for every supplied port.",
            ));
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.baseline.validate().map_err(forwarding_error)?;
        self.active.validate().map_err(forwarding_error)?;
        if self.schema_version != 1 {
            return Err(invalid("Unsupported DHCP observer schema version."));
        }
        if !self.information_option_disabled {
            return Err(unsupported(
                "This supplied observer setup requires Option82 insertion explicitly disabled; insertion/relay are not emulated.",
            ));
        }
        let f = self.baseline.fabric();
        if f.switches.len() != 1 {
            return Err(unsupported(
                "This DHCP observer supports one access-only switch.",
            ));
        }
        AccessOutputs::new(f, self.switch(), BTreeSet::new()).map_err(|_| {
            unsupported("This observer requires at most16 supplied access-only ports.")
        })?;
        if self.active.fabric() != f
            || self.active.aging_seconds() != self.baseline.aging_seconds()
            || self.active.now() < self.baseline.now()
            || f.hosts
                .keys()
                .any(|h| self.active.address(h) != self.baseline.address(h))
        {
            return Err(invalid(
                "Preserve the supplied equipment, NIC identities and forward-only simulated clock.",
            ));
        }
        self.validate_config(&self.config)?;
        for host in f.hosts.keys() {
            self.attachment(host)?;
        }
        if self.pending.len() > MAX_RECORDS || self.bindings.len() > MAX_RECORDS {
            return Err(error(
                ErrorKind::Limit,
                "At most64 pending exchanges and64 observed leases are supported.",
            ));
        }
        let now = self.active.now();
        let mut keys = BTreeSet::new();
        for p in &self.pending {
            let (port, vlan) = self.attachment(&p.host)?;
            if !p.client.unicast()
                || port != p.port
                || vlan != p.vlan
                || p.expires <= now
                || p.expires - now > TRANSACTION_SECONDS
                || !keys.insert((p.vlan, p.client))
            {
                return Err(invalid(
                    "Invalid, duplicate or expired observed client transaction.",
                ));
            }
        }
        let mut keys = BTreeSet::new();
        let mut addresses = BTreeSet::new();
        let sw = &f.switches[self.switch()];
        for b in &self.bindings {
            if !b.client.unicast()
                || !unicast(b.address)
                || b.expires <= now
                || b.expires - now > u64::from(MAX_LEASE)
                || !matches!(sw.ports.get(&b.port).map(|p|&p.mode),Some(Mode::Access{vlan})if *vlan==b.vlan)
                || !keys.insert((b.vlan, b.client))
                || !addresses.insert((b.vlan, b.address))
            {
                return Err(invalid("Invalid, duplicate or expired observed lease."));
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| invalid("Cannot encode bounded DHCP observer state."))?
            .len()
            > MAX_JSON
        {
            return Err(error(
                ErrorKind::Limit,
                "Encoded DHCP observer state exceeds768KiB.",
            ));
        }
        Ok(())
    }
    pub fn configure(&mut self, mut config: Config) -> Result<()> {
        self.validate()?;
        self.validate_config(&config)?;
        let mut trial = self.clone();
        config.vlans.sort_unstable();
        config.ports.sort_by(|a, b| a.port.cmp(&b.port));
        let mut previous = trial.config.clone();
        previous.vlans.sort_unstable();
        previous.ports.sort_by(|a, b| a.port.cmp(&b.port));
        if previous != config {
            trial.pending.clear();
        } // documented sample routing-cache reset
        trial.config = config;
        trial.validate()?;
        *self = trial;
        Ok(())
    }
    pub fn advance(&mut self, seconds: u64) -> Result<()> {
        self.validate()?;
        if seconds > 86_400 {
            return Err(error(
                ErrorKind::Limit,
                "Advance at most one simulated day at a time; no state changed.",
            ));
        }
        let mut trial = self.clone();
        trial.active.advance(seconds).map_err(forwarding_error)?;
        let now = trial.active.now();
        trial.pending.retain(|p| p.expires > now);
        trial.bindings.retain(|b| b.expires > now);
        trial.validate()?;
        *self = trial;
        Ok(())
    }
    /// Ordinary Ethernet delivery for a composed protocol, after its own ingress
    /// policy. Shares this observer's actual CAM/clock without changing DHCP data.
    /// The supplied endpoint ID is a virtual-map lookup, never a host command.
    pub fn forward_ordinary_frame(
        &mut self,
        host: &str,
        source: Mac,
        destination: Mac,
    ) -> Result<Observation> {
        self.validate()?;
        let outputs = AccessOutputs::new(
            self.active.fabric(),
            self.switch(),
            self.active.fabric().switches[self.switch()]
                .ports
                .keys()
                .cloned()
                .collect(),
        )
        .map_err(forwarding_error)?;
        let mut trial = self.clone();
        let observation = trial
            .active
            .send_frame_on_access_ports(host, source, destination, &outputs)
            .map_err(forwarding_error)?;
        trial.validate()?;
        *self = trial;
        Ok(observation)
    }
    pub fn transmit(&mut self, host: &str, packet: &Packet) -> Result<Outcome> {
        self.validate()?;
        packet.validate()?;
        if !self.active.fabric().hosts.contains_key(host) {
            return Err(invalid("Choose a known supplied endpoint."));
        }
        let mut trial = self.clone();
        let out = trial.transmit_inner(host, packet)?;
        trial.validate()?;
        *self = trial;
        Ok(out)
    }
    fn transmit_inner(&mut self, host: &str, p: &Packet) -> Result<Outcome> {
        let (port, vlan) = self.attachment(host)?;
        let trusted = self.trust(&port);
        let enabled = self.config.enabled && self.config.vlans.contains(&vlan);
        if enabled && let Some(reason) = self.source_failure(host, &port, vlan) {
            return Ok(Outcome::Unavailable {
                reason: reason.into(),
            });
        }
        if enabled && !trusted {
            if p.message.server() {
                return Ok(Outcome::Rejected {
                    cause: Rejection::UntrustedServer,
                });
            }
            if self.config.verify_source && p.source != p.client {
                return Ok(Outcome::Rejected {
                    cause: Rejection::SourceMismatch,
                });
            }
            if !p.relay.is_unspecified() || p.option82 {
                return Ok(Outcome::Rejected {
                    cause: Rejection::RelayMetadata,
                });
            }
            if matches!(p.message, Message::Release { .. } | Message::Decline)
                && self
                    .bindings
                    .iter()
                    .any(|b| b.vlan == vlan && b.client == p.client && b.port != port)
            {
                return Ok(Outcome::Rejected {
                    cause: Rejection::WrongReleasePort,
                });
            }
        }
        if !p.relay.is_unspecified() || p.option82 {
            return Err(unsupported(
                "Relay/Option82 transformation on this path is unsupported; no frame applied.",
            ));
        }
        if matches!(
            p.message,
            Message::Nak | Message::Decline | Message::Inform | Message::LeaseQuery
        ) {
            return Err(unsupported(
                "This conversation subset supports Discover/Offer/Request/ACK/Release; that message is not emulated on this path.",
            ));
        }
        if !enabled {
            return self
                .active
                .send_frame_with_static(
                    host,
                    p.source,
                    p.destination,
                    &crate::static_cam::Table::new(vec![]).unwrap(),
                )
                .map(|f| Outcome::Bypassed {
                    forwarding: f.forwarding,
                })
                .map_err(forwarding_error);
        }
        let mut allowed = if trusted {
            self.active.fabric().switches[self.switch()]
                .ports
                .keys()
                .cloned()
                .collect()
        } else {
            self.config
                .ports
                .iter()
                .filter(|p| p.trusted)
                .map(|p| p.port.clone())
                .collect()
        };
        let pending = if p.message.server() {
            let q=self.pending.iter().find(|q|q.client==p.client&&q.vlan==vlan&&q.transaction==p.transaction).cloned().ok_or_else(||unsupported("No matching observed initial client exchange; unsolicited/ambiguous replies are outside this subset."))?;
            if matches!(p.message, Message::Ack { .. }) && q.phase != Phase::Request {
                return Err(unsupported(
                    "An observed Request must precede ACK in this initial-exchange subset.",
                ));
            }
            allowed = BTreeSet::from([q.port.clone()]);
            Some(q)
        } else {
            None
        };
        let outputs = AccessOutputs::new(self.active.fabric(), self.switch(), allowed)
            .map_err(forwarding_error)?;
        let observation = self
            .active
            .send_frame_on_access_ports(host, p.source, p.destination, &outputs)
            .map_err(forwarding_error)?;
        let mut learned = None;
        let mut removed = false;
        match p.message {
            Message::Discover | Message::Request if !observation.arrivals.is_empty() => {
                if self
                    .pending
                    .iter()
                    .any(|q| q.vlan == vlan && q.client == p.client && q.port != port)
                {
                    return Err(unsupported(
                        "This initial-exchange subset does not resolve the same client on competing attachments.",
                    ));
                }
                self.pending
                    .retain(|q| !(q.vlan == vlan && q.client == p.client));
                if self.pending.len() == MAX_RECORDS {
                    return Err(error(
                        ErrorKind::Limit,
                        "Observed transaction limit reached; no frame applied.",
                    ));
                }
                let expires = self
                    .active
                    .now()
                    .checked_add(TRANSACTION_SECONDS)
                    .ok_or_else(|| {
                        error(ErrorKind::Limit, "Simulated transaction clock overflow.")
                    })?;
                self.pending.push(Pending {
                    transaction: p.transaction,
                    client: p.client,
                    vlan,
                    host: host.into(),
                    port,
                    phase: if p.message == Message::Discover {
                        Phase::Discover
                    } else {
                        Phase::Request
                    },
                    untrusted: !trusted,
                    expires,
                });
            }
            Message::Ack { address, lease } => {
                let q = pending.unwrap();
                if !observation.accepted_by.contains(&q.host) {
                    return Err(unsupported(
                        "This observer only records initial ACKs delivered to the observed client NIC; failed downstream ACK handling is outside the subset. No frame applied.",
                    ));
                }
                if q.untrusted && !self.trust(&q.port) {
                    if self
                        .bindings
                        .iter()
                        .any(|b| b.vlan == vlan && b.address == address && b.client != p.client)
                    {
                        return Err(unsupported(
                            "Conflicting address leases are outside this initial-exchange subset; no frame applied.",
                        ));
                    }
                    self.bindings
                        .retain(|b| !(b.vlan == vlan && b.client == p.client));
                    if self.bindings.len() == MAX_RECORDS {
                        return Err(error(
                            ErrorKind::Limit,
                            "Observed lease limit reached; no frame applied.",
                        ));
                    }
                    let expires =
                        self.active
                            .now()
                            .checked_add(u64::from(lease))
                            .ok_or_else(|| {
                                error(ErrorKind::Limit, "Simulated lease clock overflow.")
                            })?;
                    let binding = Binding {
                        client: p.client,
                        address,
                        vlan,
                        port: q.port,
                        expires,
                    };
                    self.bindings.push(binding.clone());
                    learned = Some(binding);
                }
                self.pending.retain(|q| {
                    !(q.vlan == vlan && q.client == p.client && q.transaction == p.transaction)
                });
            }
            Message::Release { address } => {
                let count = self.bindings.len();
                self.bindings.retain(|b| {
                    !(b.vlan == vlan
                        && b.client == p.client
                        && b.port == port
                        && b.address == address)
                });
                removed = self.bindings.len() != count;
                self.pending
                    .retain(|q| !(q.vlan == vlan && q.client == p.client && q.port == port));
            }
            _ => {}
        }
        Ok(Outcome::Forwarded {
            forwarding: observation,
            learned,
            removed,
        })
    }
}
