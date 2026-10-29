//! Pure settled-state, two-router HSRP teaching model. No packets or host I/O.
//! Persisted incumbency matters: an election is not a continuous priority sort.
//! See HSRP_SUBSET.md for the deliberately restricted connected-LAN model.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::net::Ipv4Addr;

pub const VERSION: u32 = 1;
pub const MAX_CLIENTS: usize = 8;
pub const MAX_STATE_BYTES: usize = 16_384;
pub const HELP: &str = "Simulator state controls only; no IOS command parser. One connected IPv4 LAN, one HSRPv1 group, exactly two routers, up to eight clients. Elections and failure events are already settled, without timers, packets, ARP or measured convergence. Priority 1–255; optional one WAN-link decrement strictly smaller than base priority. Preemption needs a strictly higher numeric priority; a higher IP breaks initial-election ties only. A WAN/route failure does not automatically remove an active router. Peer partitions, priority zero, multiple groups, route/IP-SLA tracking, authentication, VRRP and GLBP are unsupported.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
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
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RouterId {
    A,
    B,
}
impl RouterId {
    pub const ALL: [Self; 2] = [Self::A, Self::B];
    fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
    fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}
fn valid_label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn label<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    struct V;
    impl serde::de::Visitor<'_> for V {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an ASCII endpoint label of 1–32 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if !valid_label(s) {
                return Err(E::custom("Invalid bounded HSRP endpoint label"));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(V)
}
fn clients<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Client>, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = Vec<Client>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("one to eight virtual clients")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut result = Vec::new();
            while let Some(c) = a.next_element::<Client>()? {
                if result.len() == MAX_CLIENTS {
                    return Err(serde::de::Error::custom("HSRP client bound"));
                }
                result.push(c);
            }
            Ok(result)
        }
    }
    d.deserialize_seq(V)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterConfig {
    pub participating: bool,
    pub priority: u8,
    pub preempt: bool,
    /// Only this router's modeled WAN-link line state, not general reachability.
    pub track_wan_decrement: Option<u8>,
}
impl RouterConfig {
    pub fn validate(&self) -> Result<()> {
        if self.priority == 0
            || self
                .track_wan_decrement
                .is_some_and(|d| d == 0 || d >= self.priority)
        {
            return Err(error(
                ErrorKind::Unsupported,
                "This subset requires priority 1–255 and a positive tracking decrement smaller than the base priority; priority zero and saturation behavior are not modeled.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterFacts {
    pub power_on: bool,
    pub lan_link_up: bool,
    pub wan_link_up: bool,
    /// Supplied virtual route fact, not computed routing/ICMP success.
    pub route_available: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Router {
    #[serde(deserialize_with = "label")]
    pub label: String,
    pub address: Ipv4Addr,
    pub config: RouterConfig,
    pub facts: RouterFacts,
}
impl Router {
    fn eligible(&self) -> bool {
        self.config.participating && self.facts.power_on && self.facts.lan_link_up
    }
    fn effective_priority(&self) -> u8 {
        // Every public entry point validates config before reaching this subtraction.
        self.config.priority
            - if self.facts.wan_link_up {
                0
            } else {
                self.config.track_wan_decrement.unwrap_or(0)
            }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub id: u8,
    #[serde(deserialize_with = "label")]
    pub label: String,
    pub address: Ipv4Addr,
    /// May be wrong, including an off-LAN or subnet-reserved address.
    pub gateway: Ipv4Addr,
    pub link_up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    version: u32,
    group: u8,
    prefix: u8,
    virtual_ip: Ipv4Addr,
    routers: [Router; 2],
    #[serde(deserialize_with = "clients")]
    clients: Vec<Client>,
    active: Option<RouterId>,
    standby: Option<RouterId>,
    revision: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Active,
    Standby,
    Disabled,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouterStatus {
    pub router: RouterId,
    pub role: Role,
    pub priority: u8,
    pub effective_priority: u8,
    pub preempt: bool,
    pub facts: RouterFacts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connectivity {
    ForwardingAvailable,
    ClientLinkDown,
    GatewayOutsideLan,
    GatewayNotUsableHost,
    UnknownGateway,
    NoActiveRouter,
    RouterPoweredOff,
    RouterLanDown,
    WanLinkDown,
    NoUpstreamRoute,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientStatus {
    pub client: u8,
    pub gateway: Ipv4Addr,
    pub uses_virtual_ip: bool,
    pub via: Option<RouterId>,
    pub outcome: Connectivity,
}
impl ClientStatus {
    pub fn forwarding_available(&self) -> bool {
        self.outcome == Connectivity::ForwardingAvailable
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultEvent {
    RouterPower {
        router: RouterId,
        up: bool,
    },
    LanLink {
        router: RouterId,
        up: bool,
    },
    WanLink {
        router: RouterId,
        up: bool,
    },
    UpstreamRoute {
        router: RouterId,
        available: bool,
    },
    ClientLink {
        client: u8,
        up: bool,
    },
    /// Explicitly unsupported: a partition is outside the shared connected-LAN assumption.
    PeerCommunicationLost,
}
fn unicast(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified()
        && !ip.is_multicast()
        && !ip.is_loopback()
        && ip.octets()[0] != 0
        && ip.octets()[0] < 240
}
impl State {
    /// Simultaneous initial startup. Later failure/return uses persistent roles instead.
    pub fn new(
        group: u8,
        prefix: u8,
        virtual_ip: Ipv4Addr,
        routers: [Router; 2],
        clients: Vec<Client>,
    ) -> Result<Self> {
        let mut s = Self {
            version: VERSION,
            group,
            prefix,
            virtual_ip,
            routers,
            clients,
            active: None,
            standby: None,
            revision: 0,
        };
        s.validate_shape()?;
        s.settle();
        s.validate()?;
        Ok(s)
    }
    pub fn group(&self) -> u8 {
        self.group
    }
    pub fn prefix(&self) -> u8 {
        self.prefix
    }
    pub fn virtual_ip(&self) -> Ipv4Addr {
        self.virtual_ip
    }
    pub fn active(&self) -> Option<RouterId> {
        self.active
    }
    pub fn standby(&self) -> Option<RouterId> {
        self.standby
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn router(&self, id: RouterId) -> &Router {
        &self.routers[id.index()]
    }
    pub fn clients(&self) -> &[Client] {
        &self.clients
    }
    fn mask(&self) -> u32 {
        crate::ipv4::mask(self.prefix).expect("validated conventional prefix")
    }
    fn on_lan(&self, ip: Ipv4Addr) -> bool {
        u32::from(ip) & self.mask() == u32::from(self.virtual_ip) & self.mask()
    }
    fn usable(&self, ip: Ipv4Addr) -> bool {
        if !unicast(ip) || !self.on_lan(ip) {
            return false;
        }
        let host = u32::from(ip) & !self.mask();
        host != 0 && host != !self.mask()
    }
    fn validate_shape(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(error(ErrorKind::Invalid, "Unsupported HSRP state version."));
        }
        if !(1..=30).contains(&self.prefix) {
            return Err(error(
                ErrorKind::Unsupported,
                "This shared-LAN exercise supports IPv4 prefixes /1 through /30; /31 and /32 are excluded.",
            ));
        }
        if !self.usable(self.virtual_ip) {
            return Err(error(
                ErrorKind::Invalid,
                "The virtual gateway must be a usable unicast address in its LAN.",
            ));
        }
        if self.clients.is_empty() || self.clients.len() > MAX_CLIENTS {
            return Err(error(
                ErrorKind::Limit,
                "Supply one to eight virtual clients.",
            ));
        }
        let mut addresses = BTreeSet::from([self.virtual_ip]);
        let mut labels = BTreeSet::new();
        for r in &self.routers {
            r.config.validate()?;
            if !valid_label(&r.label)
                || !labels.insert(r.label.as_str())
                || !self.usable(r.address)
                || !addresses.insert(r.address)
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Router labels and usable LAN addresses must be unique and distinct from the virtual gateway.",
                ));
            }
        }
        let mut ids = BTreeSet::new();
        for c in &self.clients {
            if !(1..=MAX_CLIENTS as u8).contains(&c.id)
                || !ids.insert(c.id)
                || !valid_label(&c.label)
                || !labels.insert(c.label.as_str())
                || !self.usable(c.address)
                || !addresses.insert(c.address)
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Client IDs, labels and usable LAN addresses must be bounded and unique across all endpoints.",
                ));
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.validate_shape()?;
        if !RouterId::ALL
            .into_iter()
            .any(|id| self.router(id).eligible())
        {
            if self.active.is_some() || self.standby.is_some() {
                return Err(error(
                    ErrorKind::Invalid,
                    "An unavailable group cannot retain active or standby roles.",
                ));
            }
            return Ok(());
        }
        let active = self.active.ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "An eligible settled group needs exactly one active router.",
            )
        })?;
        if !self.router(active).eligible() {
            return Err(error(
                ErrorKind::Invalid,
                "The active router is unavailable or not participating.",
            ));
        }
        let other = active.other();
        let expected_standby = self.router(other).eligible().then_some(other);
        if self.standby != expected_standby {
            return Err(error(
                ErrorKind::Invalid,
                "The standby must be the other eligible router, never a second active identity.",
            ));
        }
        if expected_standby.is_some()
            && self.router(other).config.preempt
            && self.router(other).effective_priority() > self.router(active).effective_priority()
        {
            return Err(error(
                ErrorKind::Invalid,
                "Unsettled snapshot: a strictly higher-priority eligible peer with preemption must have taken over.",
            ));
        }
        Ok(())
    }
    fn settle(&mut self) {
        let incumbent = self.active.filter(|id| self.router(*id).eligible());
        self.active = if let Some(active) = incumbent {
            let peer = active.other();
            let r = self.router(peer);
            if r.eligible()
                && r.config.preempt
                && r.effective_priority() > self.router(active).effective_priority()
            {
                Some(peer)
            } else {
                Some(active)
            }
        } else {
            RouterId::ALL
                .into_iter()
                .filter(|id| self.router(*id).eligible())
                .max_by_key(|id| {
                    (
                        self.router(*id).effective_priority(),
                        u32::from(self.router(*id).address),
                    )
                })
        };
        self.standby = self
            .active
            .and_then(|id| self.router(id.other()).eligible().then_some(id.other()));
    }
    fn change(&mut self, apply: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        self.validate()?;
        let revision = self.revision.checked_add(1).ok_or_else(|| {
            error(
                ErrorKind::Limit,
                "HSRP event counter exhausted; no change applied.",
            )
        })?;
        let mut next = self.clone();
        apply(&mut next)?;
        next.validate_shape()?;
        next.settle();
        next.revision = revision;
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn configure_router(&mut self, id: RouterId, config: RouterConfig) -> Result<()> {
        config.validate()?;
        self.change(|s| {
            s.routers[id.index()].config = config;
            Ok(())
        })
    }
    pub fn configure_gateway(&mut self, id: u8, gateway: Ipv4Addr) -> Result<()> {
        self.change(|s| {
            let c = s
                .clients
                .iter_mut()
                .find(|c| c.id == id)
                .ok_or_else(|| error(ErrorKind::Unknown, "Unknown simulated client ID."))?;
            c.gateway = gateway;
            Ok(())
        })
    }
    pub fn fault(&mut self, event: FaultEvent) -> Result<()> {
        if event == FaultEvent::PeerCommunicationLost {
            return Err(error(
                ErrorKind::Unsupported,
                "Peer loss/partition can cause competing active routers in real HSRP; this simulator models only a connected LAN and will not invent split-brain recovery.",
            ));
        }
        self.change(|s| {
            match event {
                FaultEvent::RouterPower { router, up } => {
                    s.routers[router.index()].facts.power_on = up
                }
                FaultEvent::LanLink { router, up } => {
                    s.routers[router.index()].facts.lan_link_up = up
                }
                FaultEvent::WanLink { router, up } => {
                    s.routers[router.index()].facts.wan_link_up = up
                }
                FaultEvent::UpstreamRoute { router, available } => {
                    s.routers[router.index()].facts.route_available = available
                }
                FaultEvent::ClientLink { client, up } => {
                    s.clients
                        .iter_mut()
                        .find(|c| c.id == client)
                        .ok_or_else(|| error(ErrorKind::Unknown, "Unknown simulated client ID."))?
                        .link_up = up;
                }
                FaultEvent::PeerCommunicationLost => unreachable!("rejected before clone"),
            }
            Ok(())
        })
    }
    pub fn router_status(&self, id: RouterId) -> Result<RouterStatus> {
        self.validate()?;
        let r = self.router(id);
        let role = if !r.facts.power_on || !r.facts.lan_link_up {
            Role::Unavailable
        } else if !r.config.participating {
            Role::Disabled
        } else if self.active == Some(id) {
            Role::Active
        } else {
            Role::Standby
        };
        Ok(RouterStatus {
            router: id,
            role,
            priority: r.config.priority,
            effective_priority: r.effective_priority(),
            preempt: r.config.preempt,
            facts: r.facts,
        })
    }
    pub fn client_status(&self, id: u8) -> Result<ClientStatus> {
        self.validate()?;
        let c = self
            .clients
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| error(ErrorKind::Unknown, "Unknown simulated client ID."))?;
        let mut result = ClientStatus {
            client: id,
            gateway: c.gateway,
            uses_virtual_ip: c.gateway == self.virtual_ip,
            via: None,
            outcome: Connectivity::UnknownGateway,
        };
        result.outcome = if !c.link_up {
            Connectivity::ClientLinkDown
        } else if !self.on_lan(c.gateway) {
            Connectivity::GatewayOutsideLan
        } else if !self.usable(c.gateway) {
            Connectivity::GatewayNotUsableHost
        } else {
            let via = if result.uses_virtual_ip {
                self.active
            } else {
                RouterId::ALL
                    .into_iter()
                    .find(|id| self.router(*id).address == c.gateway)
            };
            result.via = via;
            match via {
                None if result.uses_virtual_ip => Connectivity::NoActiveRouter,
                None => Connectivity::UnknownGateway,
                Some(id) => {
                    let f = self.router(id).facts;
                    if !f.power_on {
                        Connectivity::RouterPoweredOff
                    } else if !f.lan_link_up {
                        Connectivity::RouterLanDown
                    } else if !f.wan_link_up {
                        Connectivity::WanLinkDown
                    } else if !f.route_available {
                        Connectivity::NoUpstreamRoute
                    } else {
                        Connectivity::ForwardingAvailable
                    }
                }
            }
        };
        Ok(result)
    }
    /// Fixed physical identities only, not proof that current config/faults meet a task.
    pub fn environment_matches(&self, source: &Self) -> Result<bool> {
        self.validate()?;
        source.validate()?;
        Ok(self.group == source.group
            && self.prefix == source.prefix
            && self.virtual_ip == source.virtual_ip
            && RouterId::ALL.into_iter().all(|id| {
                self.router(id).label == source.router(id).label
                    && self.router(id).address == source.router(id).address
            })
            && self.clients.len() == source.clients.len()
            && self
                .clients
                .iter()
                .zip(&source.clients)
                .all(|(a, b)| a.id == b.id && a.label == b.label && a.address == b.address))
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "HSRP state exceeds the 16KiB input bound.",
            ));
        }
        let state: Self = serde_json::from_slice(raw).map_err(|_| error(ErrorKind::Invalid, "Invalid bounded HSRP JSON (unknown/duplicate fields, foreign identities or malformed values)."))?;
        state.validate()?;
        Ok(state)
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let raw = serde_json::to_vec(self)
            .map_err(|_| error(ErrorKind::Invalid, "Cannot encode HSRP state."))?;
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "HSRP state exceeds the encoding bound.",
            ));
        }
        Ok(raw)
    }
}
