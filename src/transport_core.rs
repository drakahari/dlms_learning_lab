//! Finite in-process teaching reducer; no sockets, timers or host I/O.
//! Callbacks are trusted code. Reports are observations, never progress authority.
use serde::{Deserialize, Serialize};
use std::{fmt, net::Ipv4Addr};

/// Optional bounded IPv4 Echo model; not part of port-bearing TCP/UDP tuples.
pub mod echo;

pub const VERSION: u32 = 1;
pub const MAX_HOSTS: usize = 6;
pub const MAX_LISTENERS: usize = 8;
pub const MAX_EXCHANGES: usize = 8;
pub const MAX_PAYLOAD_BYTES: usize = 1024;
pub const MAX_CONFIG_BYTES: usize = 64 * 1024;
pub const MAX_TEXT_BYTES: usize = 16 * 1024;
pub const MAX_DETAIL_BYTES: usize = 256;
pub const MAX_TRACE_EVENTS: usize = 64;
pub const MAX_REPORT_TEXT_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Limit,
    Unsupported,
    Transition,
    Classifier,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn fail(kind: ErrorKind, text: &str) -> Error {
    Error::new(kind, text)
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
fn add_text(total: &mut usize, n: usize, limit: usize) -> Result<(), Error> {
    *total = total
        .checked_add(n)
        .ok_or_else(|| fail(ErrorKind::Limit, "Text budget overflow."))?;
    if *total > limit {
        return Err(fail(ErrorKind::Limit, "Text budget exceeded."));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Tcp,
    Udp,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub id: String,
    pub address: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub protocol: Protocol,
    pub endpoint: Endpoint,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exchange {
    pub id: String,
    pub protocol: Protocol,
    pub client: Endpoint,
    pub server: Endpoint,
    pub payload: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub hosts: Vec<Host>,
    pub listeners: Vec<Listener>,
    pub exchanges: Vec<Exchange>,
}
impl Config {
    /// Capped BEFORE Serde; escaped-string scratch can allocate within the cap.
    /// Direct typed callers must validate before cloning or initialization.
    pub fn decode(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > MAX_CONFIG_BYTES {
            return Err(fail(
                ErrorKind::Limit,
                "Transport configuration exceeds 64 KiB.",
            ));
        }
        let value: Self = serde_json::from_slice(raw).map_err(|_| {
            fail(
                ErrorKind::Invalid,
                "Malformed or unsupported transport configuration.",
            )
        })?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION {
            return Err(fail(
                ErrorKind::Unsupported,
                "Unsupported transport configuration version.",
            ));
        }
        if self.hosts.len() < 2
            || self.hosts.len() > MAX_HOSTS
            || self.listeners.len() > MAX_LISTENERS
            || self.exchanges.is_empty()
            || self.exchanges.len() > MAX_EXCHANGES
        {
            return Err(fail(
                ErrorKind::Limit,
                "Transport row count is outside the supported bounds.",
            ));
        }
        let mut total = 0;
        // Comparisons use borrowed inputs; bounded by six/eight rows.
        for (i, h) in self.hosts.iter().enumerate() {
            if !identifier(&h.id)
                || h.address.is_unspecified()
                || h.address.is_multicast()
                || h.address == Ipv4Addr::BROADCAST
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Supply a named IPv4 unicast host.",
                ));
            }
            add_text(&mut total, h.id.len(), MAX_TEXT_BYTES)?;
            if self.hosts[..i]
                .iter()
                .any(|old| old.id == h.id || old.address == h.address)
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Host identifiers and IPv4 addresses must be unique.",
                ));
            }
        }
        for (i, l) in self.listeners.iter().enumerate() {
            self.endpoint(&l.endpoint)?;
            add_text(&mut total, l.endpoint.host.len(), MAX_TEXT_BYTES)?;
            if self.listeners[..i].iter().any(|old| old == l) {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Duplicate listener binding for one protocol/address/port.",
                ));
            }
        }
        for (i, e) in self.exchanges.iter().enumerate() {
            if !identifier(&e.id) {
                return Err(fail(ErrorKind::Invalid, "Invalid exchange identifier."));
            }
            self.endpoint(&e.client)?;
            self.endpoint(&e.server)?;
            if e.client.host == e.server.host {
                return Err(fail(
                    ErrorKind::Unsupported,
                    "This activity requires two distinct endpoint hosts.",
                ));
            }
            if e.payload.is_empty()
                || e.payload.len() > MAX_PAYLOAD_BYTES
                || e.payload.contains('\0')
            {
                return Err(fail(
                    ErrorKind::Limit,
                    "Supply 1–1024 UTF-8 payload bytes without NUL.",
                ));
            }
            for n in [
                e.id.len(),
                e.client.host.len(),
                e.server.host.len(),
                e.payload.len(),
            ] {
                add_text(&mut total, n, MAX_TEXT_BYTES)?;
            }
            if self.exchanges[..i].iter().any(|old| {
                old.id == e.id
                    || (old.protocol == e.protocol
                        && old.client == e.client
                        && old.server == e.server)
            }) {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Conversation names and protocol/address/port pairs must be distinct. Give independent conversations different client ports.",
                ));
            }
        }
        Ok(())
    }
    fn endpoint(&self, e: &Endpoint) -> Result<(), Error> {
        if !identifier(&e.host) || e.port == 0 || !self.hosts.iter().any(|h| h.id == e.host) {
            return Err(fail(
                ErrorKind::Invalid,
                "Endpoint must name a supplied host and activity port 1–65535.",
            ));
        }
        Ok(())
    }
    fn socket(&self, e: &Endpoint) -> Result<Socket, Error> {
        let h = self
            .hosts
            .iter()
            .find(|h| h.id == e.host)
            .ok_or_else(|| fail(ErrorKind::Invalid, "Unknown endpoint host."))?;
        Ok(Socket {
            address: h.address,
            port: e.port,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Socket {
    pub address: Ipv4Addr,
    pub port: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketKind {
    Syn,
    SynAck,
    Ack,
    TcpData,
    Datagram,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Packet {
    pub protocol: Protocol,
    pub source: Socket,
    pub destination: Socket,
    pub kind: PacketKind,
    pub payload_bytes: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Delivered,
    Dropped,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Decision {
    pub disposition: Disposition,
    pub detail: String,
}
impl Decision {
    pub fn delivered(detail: &str) -> Self {
        Self {
            disposition: Disposition::Delivered,
            detail: detail.into(),
        }
    }
    pub fn dropped(detail: &str) -> Self {
        Self {
            disposition: Disposition::Dropped,
            detail: detail.into(),
        }
    }
    fn validate(&self) -> Result<(), Error> {
        if self.detail.len() > MAX_DETAIL_BYTES || self.detail.contains('\0') {
            return Err(fail(
                ErrorKind::Limit,
                "Classifier detail exceeds its bounded text format.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Ready,
    SynAckReady,
    FinalAckReady,
    Connected,
    Finished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Unrun,
    Opening,
    HandshakeComplete,
    Delivered,
    PacketDropped { kind: PacketKind },
    NoListener,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Event {
    pub packet: Packet,
    pub decision: Decision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub exchange: String,
    pub protocol: Protocol,
    pub client: Socket,
    pub server: Socket,
    pub client_state: Option<TcpState>,
    pub server_state: Option<TcpState>,
    pub phase: Phase,
    pub outcome: Outcome,
    pub trace: Vec<Event>,
    /// Received bytes, not a claim of receipt merely because data was sent.
    pub received: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Start,
    Advance,
    Write,
}

/// No Deserialize: future persisted sessions MUST replay against held source.
#[derive(Debug)]
pub struct Machine {
    report: Report,
    payload: String,
    has_listener: bool,
    detail_bytes: usize,
}
impl Machine {
    pub fn new(config: &Config, id: &str) -> Result<Self, Error> {
        config.validate()?; // all input checks BEFORE string/config clones
        let e = config
            .exchanges
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| fail(ErrorKind::Invalid, "Unknown supplied exchange."))?;
        let listening = config
            .listeners
            .iter()
            .any(|l| l.protocol == e.protocol && l.endpoint == e.server);
        let tcp = e.protocol == Protocol::Tcp;
        Ok(Self {
            report: Report {
                exchange: e.id.clone(),
                protocol: e.protocol,
                client: config.socket(&e.client)?,
                server: config.socket(&e.server)?,
                client_state: tcp.then_some(TcpState::Closed),
                server_state: tcp.then_some(if listening {
                    TcpState::Listen
                } else {
                    TcpState::Closed
                }),
                phase: Phase::Ready,
                outcome: Outcome::Unrun,
                trace: Vec::new(),
                received: None,
            },
            payload: e.payload.clone(),
            has_listener: listening,
            detail_bytes: 0,
        })
    }
    pub fn report(&self) -> &Report {
        &self.report
    }
    pub fn into_report(self) -> Report {
        self.report
    }

    /// Trusted readonly classifier; never learner-supplied/serialized code.
    /// Invalid actions, callback errors and oversize decisions leave this state exact.
    pub fn apply<F>(&mut self, action: Action, classify: &mut F) -> Result<(), Error>
    where
        F: FnMut(&Packet) -> Result<Decision, Error>,
    {
        let r = &self.report;
        let kind = match (r.protocol, r.phase, action) {
            (Protocol::Tcp, Phase::Ready, Action::Start) => PacketKind::Syn,
            (Protocol::Tcp, Phase::SynAckReady, Action::Advance) => PacketKind::SynAck,
            (Protocol::Tcp, Phase::FinalAckReady, Action::Advance) => PacketKind::Ack,
            (Protocol::Tcp, Phase::Connected, Action::Write) => PacketKind::TcpData,
            (Protocol::Udp, Phase::Ready, Action::Start) => PacketKind::Datagram,
            _ => {
                return Err(fail(
                    ErrorKind::Transition,
                    "That action is unavailable in the current transport phase.",
                ));
            }
        };
        if r.trace.len() >= MAX_TRACE_EVENTS {
            return Err(fail(ErrorKind::Limit, "Transport trace limit reached."));
        }
        let reverse = kind == PacketKind::SynAck;
        let packet = Packet {
            protocol: r.protocol,
            source: if reverse { r.server } else { r.client },
            destination: if reverse { r.client } else { r.server },
            kind,
            payload_bytes: if matches!(kind, PacketKind::TcpData | PacketKind::Datagram) {
                self.payload.len()
            } else {
                0
            },
        };
        let decision = match classify(&packet) {
            Ok(d) => d,
            Err(e) => {
                if e.message.len() > MAX_DETAIL_BYTES || e.message.contains('\0') {
                    return Err(fail(
                        ErrorKind::Limit,
                        "Classifier error text exceeds its bounded format.",
                    ));
                }
                return Err(e);
            }
        };
        decision.validate()?;
        let mut next_bytes = self.detail_bytes;
        add_text(
            &mut next_bytes,
            decision.detail.len(),
            MAX_REPORT_TEXT_BYTES,
        )?;
        // All validation precedes publication; at most four events and one
        // <=1024-byte payload are retained for this exchange.
        let delivered = decision.disposition == Disposition::Delivered;
        let r = &mut self.report;
        r.trace.push(Event { packet, decision });
        self.detail_bytes = next_bytes;
        match kind {
            PacketKind::Syn => {
                r.client_state = Some(TcpState::SynSent);
                if delivered && self.has_listener {
                    r.server_state = Some(TcpState::SynReceived);
                    r.phase = Phase::SynAckReady;
                    r.outcome = Outcome::Opening;
                }
            }
            PacketKind::SynAck if delivered => {
                r.client_state = Some(TcpState::Established);
                r.phase = Phase::FinalAckReady;
                r.outcome = Outcome::Opening;
            }
            PacketKind::Ack if delivered => {
                r.server_state = Some(TcpState::Established);
                r.phase = Phase::Connected;
                r.outcome = Outcome::HandshakeComplete;
            }
            PacketKind::TcpData | PacketKind::Datagram if delivered && self.has_listener => {
                r.received = Some(self.payload.clone());
                r.phase = Phase::Finished;
                r.outcome = Outcome::Delivered;
            }
            _ => {}
        }
        if !delivered {
            r.phase = Phase::Finished;
            r.outcome = Outcome::PacketDropped { kind };
        } else if matches!(kind, PacketKind::Syn | PacketKind::Datagram) && !self.has_listener {
            r.phase = Phase::Finished;
            r.outcome = Outcome::NoListener;
        }
        Ok(())
    }
}
pub fn run<F>(config: &Config, id: &str, classify: &mut F) -> Result<Report, Error>
where
    F: FnMut(&Packet) -> Result<Decision, Error>,
{
    let mut machine = Machine::new(config, id)?;
    for _ in 0..4 {
        let action = match machine.report.phase {
            Phase::Ready => Action::Start,
            Phase::SynAckReady | Phase::FinalAckReady => Action::Advance,
            Phase::Connected => Action::Write,
            Phase::Finished => return Ok(machine.into_report()),
        };
        machine.apply(action, classify)?;
    }
    if machine.report.phase != Phase::Finished {
        return Err(fail(
            ErrorKind::Limit,
            "Transport reducer exceeded four generated packets.",
        ));
    }
    Ok(machine.into_report())
}
pub fn run_all<F>(config: &Config, classify: &mut F) -> Result<Vec<Report>, Error>
where
    F: FnMut(&Packet) -> Result<Decision, Error>,
{
    config.validate()?;
    let mut reports = Vec::new();
    let mut events = 0usize;
    let mut texts = 0usize;
    for exchange in &config.exchanges {
        let report = run(config, &exchange.id, classify)?;
        events = events
            .checked_add(report.trace.len())
            .ok_or_else(|| fail(ErrorKind::Limit, "Trace count overflow."))?;
        if events > MAX_TRACE_EVENTS {
            return Err(fail(ErrorKind::Limit, "Batch trace limit exceeded."));
        }
        add_text(&mut texts, report.exchange.len(), MAX_REPORT_TEXT_BYTES)?;
        if let Some(text) = &report.received {
            add_text(&mut texts, text.len(), MAX_REPORT_TEXT_BYTES)?;
        }
        for event in &report.trace {
            add_text(
                &mut texts,
                event.decision.detail.len(),
                MAX_REPORT_TEXT_BYTES,
            )?;
        }
        reports.push(report);
    }
    Ok(reports)
}

/// Pure projection for the EXISTING stateless classifier. Flags stay in this
/// reducer; no old ACL enum/API or old tuple outcome is changed.
pub fn packet_protocol(packet: &Packet) -> crate::acl::PacketProtocol {
    match packet.protocol {
        Protocol::Tcp => crate::acl::PacketProtocol::Tcp {
            source_port: packet.source.port,
            destination_port: packet.destination.port,
        },
        Protocol::Udp => crate::acl::PacketProtocol::Udp {
            source_port: packet.source.port,
            destination_port: packet.destination.port,
        },
    }
}
