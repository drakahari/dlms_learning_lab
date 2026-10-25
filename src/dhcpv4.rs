//! Bounded logical DHCPv4 exchanges, not wire packets or a host DHCP service.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
    net::Ipv4Addr,
};
pub const MAX_SEGMENTS: usize = 8;
pub const MAX_CLIENTS: usize = 32;
pub const MAX_POOLS: usize = 8;
pub const MAX_RANGE: usize = 256;
pub const MAX_TIME: u64 = 1_000_000_000;
pub const MAX_ADVANCE: u64 = 86_400;
pub const MIN_LEASE: u32 = 8;
pub const MAX_LEASE: u32 = 86_400;
pub const OFFER_HOLD: u64 = 30;
pub const MAX_INPUT: usize = 1_048_576;
pub const MAX_ERROR: usize = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Limit,
    Identity,
    Action,
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
fn error(kind: ErrorKind, text: impl AsRef<str>) -> Error {
    let s = text.as_ref();
    let mut n = s.len().min(MAX_ERROR);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    let mut message = String::with_capacity(n);
    message.push_str(&s[..n]);
    Error { kind, message }
}
fn need(ok: bool, kind: ErrorKind, text: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(error(kind, text)) }
}
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn unicast(ip: Ipv4Addr) -> bool {
    let [a, ..] = ip.octets();
    a > 0 && a < 224 && a != 127 && !ip.is_broadcast()
}
struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct T<const N: usize>;
        impl<const N: usize> Visitor<'_> for T<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most{N} bytes")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                s: &str,
            ) -> std::result::Result<Self::Value, E> {
                if s.len() > N {
                    Err(E::custom("DHCP text limit"))
                } else {
                    Ok(Text(s.to_owned()))
                }
            }
            fn visit_string<E: serde::de::Error>(
                self,
                s: String,
            ) -> std::result::Result<Self::Value, E> {
                self.visit_str(&s)
            }
        }
        d.deserialize_string(T::<N>)
    }
}
fn name<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    Ok(Text::<48>::deserialize(d)?.0)
}
fn ip<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Ipv4Addr, D::Error> {
    Text::<15>::deserialize(d)?
        .0
        .parse()
        .map_err(|_| D::Error::custom("Invalid DHCP IPv4 address"))
}
#[derive(Deserialize)]
struct Ip(#[serde(deserialize_with = "ip")] Ipv4Addr);
fn optional_ip<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Option<Ipv4Addr>, D::Error> {
    Ok(Option::<Ip>::deserialize(d)?.map(|i| i.0))
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for M<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} unique DHCP entries")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key::<Text<48>>()? {
                if out.len() == N || out.contains_key(&k.0) {
                    return Err(A::Error::custom("DHCP map count/duplicate key"));
                }
                out.insert(k.0, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<V, N>(PhantomData))
}
fn sequence<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<V>, D::Error> {
    struct S<V, const N: usize>(PhantomData<V>);
    struct Next<V> {
        allowed: bool,
        value: PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> serde::de::DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("DHCP sequence count limit"));
            }
            V::deserialize(d)
        }
    }
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for S<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} DHCP values")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next::<V> {
                allowed: out.len() < N,
                value: PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(S::<V, N>(PhantomData))
}
fn ip_set<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeSet<Ipv4Addr>, D::Error> {
    let list = sequence::<D, Ip, N>(d)?;
    let mut out = BTreeSet::new();
    for v in list {
        if !out.insert(v.0) {
            return Err(D::Error::custom("Duplicate DHCP address"));
        }
    }
    Ok(out)
}
fn dns<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<Ipv4Addr>, D::Error> {
    Ok(sequence::<D, Ip, 4>(d)?.into_iter().map(|i| i.0).collect())
}
fn reservations<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Ipv4Addr>, D::Error> {
    Ok(map::<D, Ip, MAX_CLIENTS>(d)?
        .into_iter()
        .map(|(k, v)| (k, v.0))
        .collect())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subnet {
    #[serde(deserialize_with = "ip")]
    pub network: Ipv4Addr,
    pub prefix: u8,
}
impl Subnet {
    fn bounds(&self) -> Option<(u32, u32)> {
        if !(1..=30).contains(&self.prefix) {
            return None;
        }
        let mask = u32::MAX << (32 - self.prefix);
        let n = u32::from(self.network);
        if n & mask != n {
            return None;
        }
        Some((n, n | !mask))
    }
    pub fn valid(&self) -> bool {
        self.bounds().is_some()
    }
    pub fn contains(&self, address: Ipv4Addr) -> bool {
        self.bounds().is_some_and(|(n, b)| {
            let a = u32::from(address);
            a >= n && a <= b
        })
    }
    pub fn host(&self, address: Ipv4Addr) -> bool {
        unicast(address)
            && self.bounds().is_some_and(|(n, b)| {
                let a = u32::from(address);
                a > n && a < b
            })
    }
    fn overlaps(&self, other: &Self) -> bool {
        self.bounds()
            .zip(other.bounds())
            .is_some_and(|((a, b), (c, d))| a <= d && c <= b)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub subnet: Subnet,
    #[serde(deserialize_with = "optional_ip")]
    pub server_interface: Option<Ipv4Addr>,
    #[serde(deserialize_with = "optional_ip")]
    pub relay_interface: Option<Ipv4Addr>,
    pub relay_to_server: bool,
    pub client_unicast_to_server: bool,
    #[serde(deserialize_with = "ip_set::<_,16>")]
    pub occupied: BTreeSet<Ipv4Addr>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    #[serde(deserialize_with = "name")]
    pub segment: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    #[serde(deserialize_with = "ip")]
    pub server: Ipv4Addr,
    #[serde(deserialize_with = "map::<_,Segment,MAX_SEGMENTS>")]
    pub segments: BTreeMap<String, Segment>,
    #[serde(deserialize_with = "map::<_,Client,MAX_CLIENTS>")]
    pub clients: BTreeMap<String, Client>,
}
impl Topology {
    pub fn validate(&self) -> Result<()> {
        need(
            !self.segments.is_empty()
                && self.segments.len() <= MAX_SEGMENTS
                && self.clients.len() <= MAX_CLIENTS,
            ErrorKind::Limit,
            "DHCP topology count limit",
        )?;
        need(
            unicast(self.server)
                && self.segments.iter().all(|(k, s)| {
                    key(k)
                        && s.subnet.valid()
                        && s.occupied.len() <= 16
                        && s.occupied.iter().all(|a| s.subnet.host(*a))
                        && [s.server_interface, s.relay_interface]
                            .into_iter()
                            .flatten()
                            .all(|a| s.subnet.host(a))
                })
                && self.clients.iter().all(|(k, c)| {
                    key(k) && key(&c.segment) && self.segments.contains_key(&c.segment)
                }),
            ErrorKind::Invalid,
            "Invalid supplied DHCP identity/subnet/interface",
        )?;
        for (k, s) in &self.segments {
            need(
                self.segments
                    .iter()
                    .all(|(other, t)| other <= k || !s.subnet.overlaps(&t.subnet)),
                ErrorKind::Invalid,
                "Overlapping supplied DHCP segments unsupported",
            )?;
        }
        Ok(())
    }
    fn selector(&self, client: &str) -> Option<Ipv4Addr> {
        let client = self.clients.get(client)?;
        let segment = self.segments.get(&client.segment)?;
        segment.server_interface.or(segment.relay_interface)
    }
    fn infrastructure(&self, address: Ipv4Addr) -> bool {
        address == self.server
            || self.segments.values().any(|s| {
                s.server_interface == Some(address)
                    || s.relay_interface == Some(address)
                    || s.occupied.contains(&address)
            })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    pub subnet: Subnet,
    #[serde(deserialize_with = "ip")]
    pub first: Ipv4Addr,
    #[serde(deserialize_with = "ip")]
    pub last: Ipv4Addr,
    #[serde(deserialize_with = "ip_set::<_,MAX_RANGE>")]
    pub excluded: BTreeSet<Ipv4Addr>,
    #[serde(deserialize_with = "reservations")]
    pub reservations: BTreeMap<String, Ipv4Addr>,
    #[serde(deserialize_with = "optional_ip")]
    pub router: Option<Ipv4Addr>,
    #[serde(deserialize_with = "dns")]
    pub dns: Vec<Ipv4Addr>,
    pub lease_seconds: u32,
}
impl Pool {
    fn valid(&self, t: &Topology) -> bool {
        self.excluded.len() <= MAX_RANGE
            && self.reservations.len() <= MAX_CLIENTS
            && self.dns.len() <= 4
            && self.subnet.valid()
            && self.subnet.host(self.first)
            && self.subnet.host(self.last)
            && u32::from(self.last)
                .checked_sub(u32::from(self.first))
                .is_some_and(|n| u64::from(n) < MAX_RANGE as u64)
            && self.excluded.iter().all(|a| self.subnet.host(*a))
            && self
                .reservations
                .iter()
                .all(|(k, a)| key(k) && t.clients.contains_key(k) && self.subnet.host(*a))
            && (MIN_LEASE..=MAX_LEASE).contains(&self.lease_seconds)
    }
    fn parameters(&self) -> Parameters {
        Parameters {
            subnet: self.subnet,
            router: self.router,
            dns: self.dns.clone(),
        }
    }
    fn permits(&self, client: &str, address: Ipv4Addr) -> bool {
        if self.excluded.contains(&address) {
            return false;
        }
        if let Some(a) = self.reservations.get(client) {
            return *a == address;
        }
        let a = u32::from(address);
        a >= u32::from(self.first)
            && a <= u32::from(self.last)
            && !self.reservations.values().any(|v| *v == address)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relay {
    pub enabled: bool,
    #[serde(deserialize_with = "ip")]
    pub target: Ipv4Addr,
    #[serde(deserialize_with = "ip")]
    pub giaddr: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub server_enabled: bool,
    #[serde(deserialize_with = "map::<_,Pool,MAX_POOLS>")]
    pub pools: BTreeMap<String, Pool>,
    #[serde(deserialize_with = "map::<_,Relay,MAX_SEGMENTS>")]
    pub relays: BTreeMap<String, Relay>,
}
impl Policy {
    pub fn validate(&self, t: &Topology) -> Result<()> {
        t.validate()?;
        need(
            self.pools.len() <= MAX_POOLS && self.relays.len() <= MAX_SEGMENTS,
            ErrorKind::Limit,
            "DHCP policy count limit",
        )?;
        need(
            self.pools.iter().all(|(k, p)| key(k) && p.valid(t))
                && self
                    .relays
                    .keys()
                    .all(|k| key(k) && t.segments.contains_key(k)),
            ErrorKind::Invalid,
            "Invalid DHCP pool/range/options/reference bounds",
        )?;
        let mut reserved_clients = BTreeSet::new();
        let mut reserved_addresses = BTreeSet::new();
        for (k, p) in &self.pools {
            need(
                self.pools
                    .iter()
                    .all(|(other, q)| other <= k || !p.subnet.overlaps(&q.subnet)),
                ErrorKind::Invalid,
                "Overlapping DHCP pools unsupported",
            )?;
            for (c, a) in &p.reservations {
                need(
                    reserved_clients.insert(c) && reserved_addresses.insert(a),
                    ErrorKind::Invalid,
                    "Ambiguous DHCP reservation",
                )?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub subnet: Subnet,
    #[serde(deserialize_with = "optional_ip")]
    pub router: Option<Ipv4Addr>,
    #[serde(deserialize_with = "dns")]
    pub dns: Vec<Ipv4Addr>,
}
impl Parameters {
    fn valid(&self) -> bool {
        self.subnet.valid() && self.dns.len() <= 4
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Delivery {
    Direct {
        #[serde(deserialize_with = "name")]
        segment: String,
    },
    Relay {
        #[serde(deserialize_with = "name")]
        segment: String,
        #[serde(deserialize_with = "ip")]
        giaddr: Ipv4Addr,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    pub token: u64,
    /// Relative lease duration in this offer, including a remaining old binding.
    pub lease_seconds: u32,
    #[serde(deserialize_with = "name")]
    pub pool: String,
    #[serde(deserialize_with = "ip")]
    pub address: Ipv4Addr,
    pub parameters: Parameters,
    pub created_at: u64,
    pub expires_at: u64,
    pub delivery: Delivery,
    pub existing_binding: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// Original grant duration; remaining(now) is the current usable duration.
    pub granted_seconds: u32,
    #[serde(deserialize_with = "name")]
    pub pool: String,
    #[serde(deserialize_with = "ip")]
    pub address: Ipv4Addr,
    pub parameters: Parameters,
    pub acquired_at: u64,
    pub renew_at: u64,
    pub rebind_at: u64,
    pub expires_at: u64,
}
impl Binding {
    fn valid(&self, now: u64) -> bool {
        key(&self.pool)
            && (MIN_LEASE..=MAX_LEASE).contains(&self.granted_seconds)
            && self.parameters.valid()
            && self.parameters.subnet.host(self.address)
            && self.acquired_at <= now
            && now < self.expires_at
            && self.expires_at <= MAX_TIME
            && self
                .acquired_at
                .checked_add(u64::from(self.granted_seconds))
                == Some(self.expires_at)
            && self
                .acquired_at
                .checked_add(u64::from(self.granted_seconds) / 2)
                == Some(self.renew_at)
            && self
                .acquired_at
                .checked_add(u64::from(self.granted_seconds) * 7 / 8)
                == Some(self.rebind_at)
            && self.acquired_at < self.renew_at
            && self.renew_at < self.rebind_at
            && self.rebind_at < self.expires_at
    }
    pub fn remaining(&self, now: u64) -> u64 {
        self.expires_at.saturating_sub(now)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    Unconfigured,
    Selecting,
    Bound,
    Renewal,
    Rebinding,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    ServerDisabled,
    NoRelay,
    WrongRelayTarget,
    WrongRelayAddress,
    RelayUnreachable,
    UnicastUnreachable,
    NoPool,
    PoolChanged,
    Exhausted,
    ReservationUnavailable,
    NoOffer,
    StaleOffer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckKind {
    NewLease,
    Renewed,
    CurrentLease,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Offered(Offer),
    Ack {
        binding: Binding,
        kind: AckKind,
    },
    Nak {
        address: Ipv4Addr,
    },
    NoReply(Reason),
    Released {
        address: Ipv4Addr,
        server_notified: bool,
    },
    Advanced {
        clients: Vec<String>,
        bindings: Vec<String>,
        offers: Vec<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    topology: Topology,
    policy: Policy,
    now: u64,
    next_token: u64,
    offers: BTreeMap<String, Offer>,
    bindings: BTreeMap<String, Binding>,
    leases: BTreeMap<String, Binding>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawState {
    topology: Topology,
    policy: Policy,
    now: u64,
    next_token: u64,
    #[serde(deserialize_with = "map::<_,Offer,MAX_CLIENTS>")]
    offers: BTreeMap<String, Offer>,
    #[serde(deserialize_with = "map::<_,Binding,MAX_CLIENTS>")]
    bindings: BTreeMap<String, Binding>,
    #[serde(deserialize_with = "map::<_,Binding,MAX_CLIENTS>")]
    leases: BTreeMap<String, Binding>,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let r = RawState::deserialize(d)?;
        let s = Self {
            topology: r.topology,
            policy: r.policy,
            now: r.now,
            next_token: r.next_token,
            offers: r.offers,
            bindings: r.bindings,
            leases: r.leases,
        };
        s.validate().map_err(|e| D::Error::custom(e.message))?;
        Ok(s)
    }
}
impl State {
    pub fn new(topology: Topology, policy: Policy) -> Result<Self> {
        topology.validate()?;
        policy.validate(&topology)?;
        Ok(Self {
            topology,
            policy,
            now: 0,
            next_token: 0,
            offers: BTreeMap::new(),
            bindings: BTreeMap::new(),
            leases: BTreeMap::new(),
        })
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        need(
            bytes.len() <= MAX_INPUT,
            ErrorKind::Limit,
            "DHCP snapshot exceeds1MiB",
        )?;
        serde_json::from_slice(bytes).map_err(|e| error(ErrorKind::Invalid, e.to_string()))
    }
    /// Future adapters must compare this immutable supplied context, not policy.
    pub fn validate_context(&self, expected: &Topology) -> Result<()> {
        expected.validate()?;
        self.validate()?;
        need(
            &self.topology == expected,
            ErrorKind::Identity,
            "Foreign supplied DHCP topology",
        )
    }
    pub fn topology(&self) -> &Topology {
        &self.topology
    }
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
    pub fn now(&self) -> u64 {
        self.now
    }
    pub fn offers(&self) -> &BTreeMap<String, Offer> {
        &self.offers
    }
    pub fn bindings(&self) -> &BTreeMap<String, Binding> {
        &self.bindings
    }
    pub fn leases(&self) -> &BTreeMap<String, Binding> {
        &self.leases
    }
    pub fn validate(&self) -> Result<()> {
        need(
            self.now <= MAX_TIME
                && self.next_token <= MAX_TIME
                && self.offers.len() <= MAX_CLIENTS
                && self.bindings.len() <= MAX_CLIENTS
                && self.leases.len() <= MAX_CLIENTS,
            ErrorKind::Limit,
            "DHCP runtime count/clock limit",
        )?;
        self.topology.validate()?;
        self.policy.validate(&self.topology)?;
        need(
            self.bindings.iter().all(|(c, b)| {
                key(c)
                    && self.topology.clients.contains_key(c)
                    && b.valid(self.now)
                    && !self.topology.infrastructure(b.address)
                    && self
                        .topology
                        .selector(c)
                        .is_some_and(|selector| b.parameters.subnet.contains(selector))
            }) && self
                .leases
                .iter()
                .all(|(c, b)| self.bindings.get(c) == Some(b)),
            ErrorKind::Identity,
            "Invalid DHCP lease owner/snapshot",
        )?;
        let mut addresses = BTreeMap::new();
        for (c, b) in &self.bindings {
            need(
                addresses.insert(b.address, c).is_none(),
                ErrorKind::Identity,
                "Duplicate DHCP active binding",
            )?;
        }
        let mut tokens = BTreeSet::new();
        for (c, o) in &self.offers {
            need(
                key(c)
                    && self.topology.clients.contains_key(c)
                    && !self.leases.contains_key(c)
                    && key(&o.pool)
                    && o.token > 0
                    && o.token <= self.next_token
                    && tokens.insert(o.token)
                    && o.parameters.valid()
                    && o.lease_seconds > 0
                    && o.lease_seconds <= MAX_LEASE
                    && (o.existing_binding || o.lease_seconds >= MIN_LEASE)
                    && o.parameters.subnet.host(o.address)
                    && !self.topology.infrastructure(o.address)
                    && o.created_at <= self.now
                    && self.now < o.expires_at
                    && o.expires_at <= MAX_TIME
                    && o.created_at
                        .checked_add(OFFER_HOLD)
                        .is_some_and(|max| o.expires_at <= max),
                ErrorKind::Identity,
                "Invalid DHCP pending offer",
            )?;
            let segment = &self.topology.clients[c].segment;
            let supplied = &self.topology.segments[segment];
            need(
                match &o.delivery {
                    Delivery::Direct { segment: s } => {
                        s == segment && supplied.server_interface.is_some()
                    }
                    Delivery::Relay { segment: s, giaddr } => {
                        s == segment
                            && supplied.server_interface.is_none()
                            && supplied.relay_interface == Some(*giaddr)
                    }
                },
                ErrorKind::Identity,
                "Foreign DHCP offer delivery context",
            )?;
            need(
                self.topology
                    .selector(c)
                    .is_some_and(|selector| o.parameters.subnet.contains(selector)),
                ErrorKind::Identity,
                "Foreign DHCP offered subnet/segment selector",
            )?;
            if o.existing_binding {
                need(
                    self.bindings.get(c).is_some_and(|b| {
                        b.address == o.address
                            && b.parameters == o.parameters
                            && b.expires_at.checked_sub(o.created_at)
                                == Some(u64::from(o.lease_seconds))
                            && o.expires_at <= b.expires_at
                    }),
                    ErrorKind::Identity,
                    "Foreign DHCP existing-binding offer",
                )?;
            }
            match addresses.get(&o.address) {
                Some(owner) => need(
                    *owner == c,
                    ErrorKind::Identity,
                    "Duplicate DHCP offered address",
                )?,
                None => {
                    need(
                        !o.existing_binding,
                        ErrorKind::Identity,
                        "Missing DHCP offered binding",
                    )?;
                    addresses.insert(o.address, c);
                }
            }
        }
        Ok(())
    }
    pub fn replace_policy(&mut self, policy: Policy) -> Result<()> {
        self.validate()?;
        policy.validate(&self.topology)?;
        self.policy = policy;
        Ok(())
    }
    fn client(&self, id: &str) -> Result<&Client> {
        need(
            key(id),
            ErrorKind::Invalid,
            "Invalid DHCP client identifier",
        )?;
        self.topology
            .clients
            .get(id)
            .ok_or_else(|| error(ErrorKind::Identity, "Unknown supplied DHCP client"))
    }
    pub fn window(&self, id: &str) -> Result<Window> {
        self.validate()?;
        self.client(id)?;
        Ok(if let Some(b) = self.leases.get(id) {
            if self.now >= b.rebind_at {
                Window::Rebinding
            } else if self.now >= b.renew_at {
                Window::Renewal
            } else {
                Window::Bound
            }
        } else if self.offers.contains_key(id) {
            Window::Selecting
        } else {
            Window::Unconfigured
        })
    }
    fn broadcast(&self, id: &str) -> std::result::Result<(Delivery, Ipv4Addr), Reason> {
        if !self.policy.server_enabled {
            return Err(Reason::ServerDisabled);
        }
        let c = &self.topology.clients[id];
        let s = &self.topology.segments[&c.segment];
        if let Some(address) = s.server_interface {
            return Ok((
                Delivery::Direct {
                    segment: c.segment.clone(),
                },
                address,
            ));
        }
        let r = self
            .policy
            .relays
            .get(&c.segment)
            .filter(|r| r.enabled)
            .ok_or(Reason::NoRelay)?;
        if r.target != self.topology.server {
            return Err(Reason::WrongRelayTarget);
        }
        if s.relay_interface != Some(r.giaddr) {
            return Err(Reason::WrongRelayAddress);
        }
        if !s.relay_to_server {
            return Err(Reason::RelayUnreachable);
        }
        Ok((
            Delivery::Relay {
                segment: c.segment.clone(),
                giaddr: r.giaddr,
            },
            r.giaddr,
        ))
    }
    fn pool_at(&self, address: Ipv4Addr) -> Option<(&str, &Pool)> {
        self.policy
            .pools
            .iter()
            .find(|(_, p)| p.subnet.contains(address))
            .map(|(id, p)| (id.as_str(), p))
    }
    fn available(&self, client: &str, address: Ipv4Addr) -> bool {
        !self.topology.infrastructure(address)
            && self
                .bindings
                .iter()
                .all(|(c, b)| c == client || b.address != address)
            && self
                .offers
                .iter()
                .all(|(c, o)| c == client || o.address != address)
    }
    fn candidate(&self, client: &str, p: &Pool) -> std::result::Result<(Ipv4Addr, bool), Reason> {
        if let Some(b) = self.bindings.get(client)
            && p.subnet == b.parameters.subnet
            && p.permits(client, b.address)
            && self.available(client, b.address)
        {
            return Ok((b.address, true));
        }
        if let Some(a) = p.reservations.get(client) {
            return if p.permits(client, *a) && self.available(client, *a) {
                Ok((*a, false))
            } else {
                Err(Reason::ReservationUnavailable)
            };
        }
        let first = u32::from(p.first);
        let count = u32::from(p.last) - first + 1;
        for offset in 0..count {
            let a = Ipv4Addr::from(first + offset);
            if p.permits(client, a) && self.available(client, a) {
                return Ok((a, false));
            }
        }
        Err(Reason::Exhausted)
    }
    fn add_time(&self, seconds: u64) -> Result<u64> {
        self.now
            .checked_add(seconds)
            .filter(|t| *t <= MAX_TIME)
            .ok_or_else(|| error(ErrorKind::Limit, "DHCP virtual clock addition limit"))
    }
    pub fn discover(&mut self, id: &str) -> Result<Outcome> {
        self.validate()?;
        self.client(id)?;
        need(
            !self.leases.contains_key(id),
            ErrorKind::Action,
            "Client already holds a lease; renew or release it",
        )?;
        let (delivery, selector) = match self.broadcast(id) {
            Ok(v) => v,
            Err(r) => {
                self.offers.remove(id);
                return Ok(Outcome::NoReply(r));
            }
        };
        let Some((pool, p)) = self.pool_at(selector) else {
            self.offers.remove(id);
            return Ok(Outcome::NoReply(Reason::NoPool));
        };
        let (address, existing) = match self.candidate(id, p) {
            Ok(v) => v,
            Err(r) => {
                self.offers.remove(id);
                return Ok(Outcome::NoReply(r));
            }
        };
        let token = self
            .next_token
            .checked_add(1)
            .filter(|t| *t <= MAX_TIME)
            .ok_or_else(|| error(ErrorKind::Limit, "DHCP offer token limit"))?;
        let lease_seconds = if existing {
            u32::try_from(self.bindings[id].remaining(self.now))
                .map_err(|_| error(ErrorKind::Limit, "DHCP offered duration limit"))?
        } else {
            p.lease_seconds
        };
        let hold = self.add_time(if existing {
            OFFER_HOLD.min(u64::from(lease_seconds))
        } else {
            OFFER_HOLD
        })?;
        let parameters = if existing {
            self.bindings[id].parameters.clone()
        } else {
            p.parameters()
        };
        let expires_at = if existing {
            hold.min(self.bindings[id].expires_at)
        } else {
            hold
        };
        let offer = Offer {
            token,
            lease_seconds,
            pool: pool.into(),
            address,
            parameters,
            created_at: self.now,
            expires_at,
            delivery,
            existing_binding: existing,
        };
        self.next_token = token;
        self.offers.insert(id.into(), offer.clone());
        Ok(Outcome::Offered(offer))
    }
    fn make_binding(
        &self,
        pool: &str,
        address: Ipv4Addr,
        parameters: Parameters,
        granted_seconds: u32,
    ) -> Result<Binding> {
        let expires_at = self.add_time(u64::from(granted_seconds))?;
        Ok(Binding {
            granted_seconds,
            pool: pool.into(),
            address,
            renew_at: self.now + u64::from(granted_seconds) / 2,
            rebind_at: self.now + u64::from(granted_seconds) * 7 / 8,
            parameters,
            acquired_at: self.now,
            expires_at,
        })
    }
    pub fn request(&mut self, id: &str, token: u64) -> Result<Outcome> {
        self.validate()?;
        self.client(id)?;
        let Some(o) = self.offers.get(id) else {
            return Ok(Outcome::NoReply(Reason::NoOffer));
        };
        if o.token != token {
            return Ok(Outcome::NoReply(Reason::StaleOffer));
        }
        let selector = match self.broadcast(id) {
            Ok((_, a)) => a,
            Err(r) => return Ok(Outcome::NoReply(r)),
        };
        let Some((pool, p)) = self.pool_at(selector) else {
            return Ok(Outcome::NoReply(Reason::NoPool));
        };
        if p.subnet != o.parameters.subnet {
            return Ok(Outcome::NoReply(Reason::PoolChanged));
        }
        if !p.permits(id, o.address) || !self.available(id, o.address) {
            let address = o.address;
            self.offers.remove(id);
            return Ok(Outcome::Nak { address });
        }
        let (binding, kind) = if o.existing_binding {
            (self.bindings[id].clone(), AckKind::CurrentLease)
        } else {
            (
                self.make_binding(pool, o.address, o.parameters.clone(), o.lease_seconds)?,
                AckKind::NewLease,
            )
        };
        self.bindings.insert(id.into(), binding.clone());
        self.leases.insert(id.into(), binding.clone());
        self.offers.remove(id);
        Ok(Outcome::Ack { binding, kind })
    }
    fn extend(&mut self, id: &str, broadcast: bool) -> Result<Outcome> {
        self.validate()?;
        let c = self.client(id)?;
        let old = self
            .leases
            .get(id)
            .ok_or_else(|| error(ErrorKind::Action, "No current client lease to extend"))?;
        let selector = if broadcast {
            need(
                self.now >= old.rebind_at,
                ErrorKind::Action,
                "Rebinding window has not begun",
            )?;
            match self.broadcast(id) {
                Ok((_, a)) => a,
                Err(r) => return Ok(Outcome::NoReply(r)),
            }
        } else {
            if !self.policy.server_enabled {
                return Ok(Outcome::NoReply(Reason::ServerDisabled));
            }
            if !self.topology.segments[&c.segment].client_unicast_to_server {
                return Ok(Outcome::NoReply(Reason::UnicastUnreachable));
            }
            old.address
        };
        let eligible = self.pool_at(selector).filter(|(_, p)| {
            p.subnet == old.parameters.subnet
                && p.permits(id, old.address)
                && self.available(id, old.address)
        });
        let Some((pool, p)) = eligible else {
            return Ok(Outcome::Ack {
                binding: old.clone(),
                kind: AckKind::CurrentLease,
            });
        };
        let binding = self.make_binding(pool, old.address, p.parameters(), p.lease_seconds)?;
        self.bindings.insert(id.into(), binding.clone());
        self.leases.insert(id.into(), binding.clone());
        Ok(Outcome::Ack {
            binding,
            kind: AckKind::Renewed,
        })
    }
    pub fn renew(&mut self, id: &str) -> Result<Outcome> {
        self.extend(id, false)
    }
    pub fn rebind(&mut self, id: &str) -> Result<Outcome> {
        self.extend(id, true)
    }
    pub fn release(&mut self, id: &str) -> Result<Outcome> {
        self.validate()?;
        let c = self.client(id)?;
        let b = self
            .leases
            .get(id)
            .ok_or_else(|| error(ErrorKind::Action, "No current client lease to release"))?;
        let address = b.address;
        let server_notified = self.policy.server_enabled
            && self.topology.segments[&c.segment].client_unicast_to_server;
        self.leases.remove(id);
        self.offers.remove(id);
        if server_notified {
            self.bindings.remove(id);
        }
        Ok(Outcome::Released {
            address,
            server_notified,
        })
    }
    pub fn advance(&mut self, seconds: u64) -> Result<Outcome> {
        self.validate()?;
        need(
            seconds <= MAX_ADVANCE,
            ErrorKind::Limit,
            "DHCP advance limit86400 seconds",
        )?;
        let now = self.add_time(seconds)?;
        let clients: Vec<_> = self
            .leases
            .iter()
            .filter(|(_, b)| b.expires_at <= now)
            .map(|(c, _)| c.clone())
            .collect();
        let bindings: Vec<_> = self
            .bindings
            .iter()
            .filter(|(_, b)| b.expires_at <= now)
            .map(|(c, _)| c.clone())
            .collect();
        let offers: Vec<_> = self
            .offers
            .iter()
            .filter(|(_, o)| o.expires_at <= now)
            .map(|(c, _)| c.clone())
            .collect();
        for c in &clients {
            self.leases.remove(c);
        }
        for c in &bindings {
            self.bindings.remove(c);
        }
        for c in &offers {
            self.offers.remove(c);
        }
        self.now = now;
        Ok(Outcome::Advanced {
            clients,
            bindings,
            offers,
        })
    }
}
