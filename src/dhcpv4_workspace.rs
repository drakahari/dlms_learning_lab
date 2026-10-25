//! Bounded typed DHCP editing and fresh state goals. No host commands or packets.
use crate::dhcpv4::{self, Outcome, Policy, Pool, Relay, State, Subnet, Topology};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::Write,
    marker::PhantomData,
    net::Ipv4Addr,
};
pub const MAX_DRAFT_BYTES: usize = 262_144;
pub const MAX_REQUIREMENTS: usize = 32;
pub const MAX_PROBES: usize = 4;
pub const MAX_PROBE_CLIENTS: usize = 8;
pub const MAX_TOTAL_PROBE_CLIENTS: usize = 32;
pub const MAX_ACTION_BYTES: usize = 262_144;
pub const HELP: &str = "Use the virtual DHCP policy editor and explicit Discover, Request, Renew, Rebind, Release and Advance controls. Offers do not configure clients. Apply changes policy only; held options require an actual exchange. This models DHCP configuration and supplied transport reachability, not routed connectivity, DNS service, Linux commands or packets.";
pub fn notice(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let mut n = s.len().min(dhcpv4::MAX_ERROR);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    let mut out = String::with_capacity(n);
    out.push_str(&s[..n]);
    out
}
pub fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn raw(s: &str) -> bool {
    s.len() <= 64 && !s.chars().any(char::is_control)
}
pub(crate) struct Text<const N: usize>(pub String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct T<const N: usize>;
        impl<const N: usize> Visitor<'_> for T<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if s.len() <= N && !s.chars().any(char::is_control) {
                    Ok(Text(s.into()))
                } else {
                    Err(E::custom("DHCP editing text bound"))
                }
            }
        }
        d.deserialize_str(T::<N>)
    }
}
fn text<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Text::<64>::deserialize(d)?.0)
}
fn name<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let s = Text::<48>::deserialize(d)?.0;
    if key(&s) {
        Ok(s)
    } else {
        Err(D::Error::custom("Invalid DHCP identity"))
    }
}
fn ip<'de, D: Deserializer<'de>>(d: D) -> Result<Ipv4Addr, D::Error> {
    Text::<15>::deserialize(d)?
        .0
        .parse()
        .map_err(|_| D::Error::custom("Invalid bounded DHCP IPv4 address"))
}
#[derive(Deserialize)]
struct Ip(#[serde(deserialize_with = "ip")] Ipv4Addr);
fn optional_ip<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Ipv4Addr>, D::Error> {
    Ok(Option::<Ip>::deserialize(d)?.map(|ip| ip.0))
}
fn dns<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Ipv4Addr>, D::Error> {
    Ok(list::<D, Ip, 4>(d)?.into_iter().map(|ip| ip.0).collect())
}
fn optional_name<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let s = Option::<Text<48>>::deserialize(d)?.map(|s| s.0);
    if s.as_ref().is_none_or(|s| key(s)) {
        Ok(s)
    } else {
        Err(D::Error::custom("Invalid selected DHCP pool"))
    }
}
pub(crate) fn list<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<V>, D::Error> {
    struct L<V, const N: usize>(PhantomData<V>);
    struct Next<V> {
        allowed: bool,
        value: PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("DHCP list count limit"));
            }
            V::deserialize(d)
        }
    }
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for L<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} DHCP items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
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
    d.deserialize_seq(L::<V, N>(PhantomData))
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for M<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique DHCP identities")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Text(k)) = a.next_key::<Text<48>>()? {
                if !key(&k) || out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom("DHCP map identity/count/duplicate limit"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<V, N>(PhantomData))
}
// Count compact JSON without allocating a second serialized copy. Structural
// bounds must be checked first; this is an aggregate admission ceiling.
pub(crate) fn within<T: Serialize>(value: &T, limit: usize) -> bool {
    struct Count {
        used: usize,
        limit: usize,
    }
    impl Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.used = self
                .used
                .checked_add(b.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("DHCP aggregate byte bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { used: 0, limit }, value).is_ok()
}
fn texts<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(list::<D, Text<64>, N>(d)?
        .into_iter()
        .map(|s| s.0)
        .collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReservationDraft {
    #[serde(deserialize_with = "text")]
    pub client: String,
    #[serde(deserialize_with = "text")]
    pub address: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoolDraft {
    #[serde(deserialize_with = "text")]
    pub network: String,
    #[serde(deserialize_with = "text")]
    pub prefix: String,
    #[serde(deserialize_with = "text")]
    pub first: String,
    #[serde(deserialize_with = "text")]
    pub last: String,
    #[serde(deserialize_with = "texts::<_,256>")]
    pub excluded: Vec<String>,
    #[serde(deserialize_with = "list::<_,ReservationDraft,32>")]
    pub reservations: Vec<ReservationDraft>,
    #[serde(deserialize_with = "text")]
    pub router: String,
    #[serde(deserialize_with = "texts::<_,4>")]
    pub dns: Vec<String>,
    #[serde(deserialize_with = "text")]
    pub lease_seconds: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelayDraft {
    pub enabled: bool,
    #[serde(deserialize_with = "text")]
    pub target: String,
    #[serde(deserialize_with = "text")]
    pub giaddr: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub version: u8,
    pub context: Topology,
    pub server_enabled: bool,
    pub pools: BTreeMap<String, PoolDraft>,
    pub relays: BTreeMap<String, RelayDraft>,
    pub selected_client: String,
    pub selected_pool: Option<String>,
    pub advance_seconds: String,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u8,
            context: Topology,
            server_enabled: bool,
            #[serde(deserialize_with = "map::<_,PoolDraft,8>")]
            pools: BTreeMap<String, PoolDraft>,
            #[serde(deserialize_with = "map::<_,RelayDraft,8>")]
            relays: BTreeMap<String, RelayDraft>,
            #[serde(deserialize_with = "name")]
            selected_client: String,
            #[serde(deserialize_with = "optional_name")]
            selected_pool: Option<String>,
            #[serde(deserialize_with = "text")]
            advance_seconds: String,
        }
        let r =
            Raw::deserialize(d).map_err(|_| D::Error::custom("Invalid bounded DHCP draft data"))?;
        let v = Self {
            version: r.version,
            context: r.context,
            server_enabled: r.server_enabled,
            pools: r.pools,
            relays: r.relays,
            selected_client: r.selected_client,
            selected_pool: r.selected_pool,
            advance_seconds: r.advance_seconds,
        };
        if !v.valid() {
            return Err(D::Error::custom(
                "Invalid DHCP draft shape/aggregate bounds",
            ));
        }
        Ok(v)
    }
}
impl Draft {
    pub fn new(s: &State) -> Result<Self, String> {
        s.validate().map_err(|e| notice(e.message))?;
        let p = s.policy();
        let v = Self {
            version: 1,
            context: s.topology().clone(),
            server_enabled: p.server_enabled,
            pools: p
                .pools
                .iter()
                .map(|(k, p)| {
                    (
                        k.clone(),
                        PoolDraft {
                            network: p.subnet.network.to_string(),
                            prefix: p.subnet.prefix.to_string(),
                            first: p.first.to_string(),
                            last: p.last.to_string(),
                            excluded: p.excluded.iter().map(ToString::to_string).collect(),
                            reservations: p
                                .reservations
                                .iter()
                                .map(|(c, a)| ReservationDraft {
                                    client: c.clone(),
                                    address: a.to_string(),
                                })
                                .collect(),
                            router: p.router.map_or_else(String::new, |a| a.to_string()),
                            dns: p.dns.iter().map(ToString::to_string).collect(),
                            lease_seconds: p.lease_seconds.to_string(),
                        },
                    )
                })
                .collect(),
            relays: p
                .relays
                .iter()
                .map(|(k, r)| {
                    (
                        k.clone(),
                        RelayDraft {
                            enabled: r.enabled,
                            target: r.target.to_string(),
                            giaddr: r.giaddr.to_string(),
                        },
                    )
                })
                .collect(),
            selected_client: s
                .topology()
                .clients
                .keys()
                .next()
                .ok_or("DHCP requires a supplied client")?
                .clone(),
            selected_pool: p.pools.keys().next().cloned(),
            advance_seconds: "30".into(),
        };
        if !v.valid() {
            return Err("DHCP draft exceeds aggregate bounds".into());
        }
        Ok(v)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.context.validate().is_ok()
            && self.pools.len() <= 8
            && self.relays.len() <= 8
            && key(&self.selected_client)
            && self.context.clients.contains_key(&self.selected_client)
            && self
                .selected_pool
                .as_ref()
                .is_none_or(|k| self.pools.contains_key(k))
            && raw(&self.advance_seconds)
            && self.pools.iter().all(|(k, p)| {
                key(k)
                    && p.excluded.len() <= 256
                    && p.reservations.len() <= 32
                    && p.dns.len() <= 4
                    && [
                        &p.network,
                        &p.prefix,
                        &p.first,
                        &p.last,
                        &p.router,
                        &p.lease_seconds,
                    ]
                    .into_iter()
                    .all(|s| raw(s))
                    && p.excluded.iter().chain(&p.dns).all(|s| raw(s))
                    && p.reservations
                        .iter()
                        .all(|r| raw(&r.client) && raw(&r.address))
            })
            && self.relays.iter().all(|(k, r)| {
                key(k) && self.context.segments.contains_key(k) && raw(&r.target) && raw(&r.giaddr)
            })
            && within(self, MAX_DRAFT_BYTES)
    }
    pub fn same_shape(&self, s: &State) -> bool {
        self.valid() && s.validate_context(&self.context).is_ok()
    }
    pub fn snapshot(&self) -> Result<Policy, String> {
        if !self.valid() {
            return Err("Invalid DHCP editing bounds or context".into());
        }
        let address = |s: &str| {
            s.parse::<Ipv4Addr>()
                .map_err(|_| "Enter a valid IPv4 address".to_owned())
        };
        let mut pools = BTreeMap::new();
        for (k, p) in &self.pools {
            let mut excluded = BTreeSet::new();
            for a in &p.excluded {
                if !excluded.insert(address(a)?) {
                    return Err("Duplicate DHCP excluded address".into());
                }
            }
            let mut reservations = BTreeMap::new();
            for r in &p.reservations {
                if !key(&r.client)
                    || reservations
                        .insert(r.client.clone(), address(&r.address)?)
                        .is_some()
                {
                    return Err("Invalid or duplicate DHCP reservation client".into());
                }
            }
            pools.insert(
                k.clone(),
                Pool {
                    subnet: Subnet {
                        network: address(&p.network)?,
                        prefix: p
                            .prefix
                            .parse()
                            .map_err(|_| "Enter a whole-number IPv4 prefix".to_owned())?,
                    },
                    first: address(&p.first)?,
                    last: address(&p.last)?,
                    excluded,
                    reservations,
                    router: if p.router.is_empty() {
                        None
                    } else {
                        Some(address(&p.router)?)
                    },
                    dns: p.dns.iter().map(|s| address(s)).collect::<Result<_, _>>()?,
                    lease_seconds: p
                        .lease_seconds
                        .parse()
                        .map_err(|_| "Enter a whole-number lease duration".to_owned())?,
                },
            );
        }
        let relays = self
            .relays
            .iter()
            .map(|(k, r)| {
                Ok((
                    k.clone(),
                    Relay {
                        enabled: r.enabled,
                        target: address(&r.target)?,
                        giaddr: address(&r.giaddr)?,
                    },
                ))
            })
            .collect::<Result<_, String>>()?;
        let p = Policy {
            server_enabled: self.server_enabled,
            pools,
            relays,
        };
        p.validate(&self.context).map_err(|e| notice(e.message))?;
        Ok(p)
    }
    pub fn matches(&self, s: &State) -> bool {
        self.same_shape(s) && self.snapshot().is_ok_and(|p| &p == s.policy())
    }
    pub fn apply(&self, s: &mut State) -> Result<(), String> {
        if !self.same_shape(s) {
            return Err("Foreign DHCP draft context; state retained".into());
        }
        let p = self.snapshot()?;
        s.replace_policy(p).map_err(|e| notice(e.message))
    }
    pub fn seconds(&self) -> Result<u64, String> {
        if !self.valid() {
            return Err("Invalid DHCP editing data".into());
        }
        let n = self
            .advance_seconds
            .parse::<u64>()
            .map_err(|_| "Enter a whole-number simulated duration".to_owned())?;
        if n > dhcpv4::MAX_ADVANCE {
            return Err("Advance is limited to86400 simulated seconds".into());
        }
        Ok(n)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientConfiguration {
    #[serde(deserialize_with = "name")]
    pub client: String,
    pub subnet: Subnet,
    #[serde(deserialize_with = "optional_ip")]
    pub router: Option<Ipv4Addr>,
    #[serde(deserialize_with = "dns")]
    pub dns: Vec<Ipv4Addr>,
    pub minimum_remaining: u32,
    /// An exact address is required only when the assignment expressly requests it.
    #[serde(deserialize_with = "optional_ip")]
    pub address: Option<Ipv4Addr>,
}
impl ClientConfiguration {
    fn valid(&self, t: &Topology) -> bool {
        key(&self.client)
            && t.clients
                .get(&self.client)
                .is_some_and(|c| t.segments[&c.segment].subnet == self.subnet)
            && self.dns.len() <= 4
            && self.dns.iter().collect::<BTreeSet<_>>().len() == self.dns.len()
            && (1..=dhcpv4::MAX_LEASE).contains(&self.minimum_remaining)
            && self.address.is_none_or(|a| self.subnet.host(a))
            && self.router.is_none_or(|a| self.subnet.host(a))
    }
    fn check(&self, s: &State) -> (bool, String) {
        let Some(b) = s.leases().get(&self.client) else {
            return (
                false,
                format!(
                    "{}: no current client lease; inspect the exchange and request a delivered offer",
                    self.client
                ),
            );
        };
        if s.bindings().get(&self.client) != Some(b) || b.remaining(s.now()) == 0 {
            return (
                false,
                format!("{}: lease is not currently usable", self.client),
            );
        }
        if !self.subnet.host(b.address) || b.parameters.subnet != self.subnet {
            return (
                false,
                format!(
                    "{}: client address/subnet differs from the requested physical segment",
                    self.client
                ),
            );
        }
        if self.address.is_some_and(|a| a != b.address) {
            return (
                false,
                format!(
                    "{}: client has not received the requested reserved address",
                    self.client
                ),
            );
        }
        let actual: BTreeSet<_> = b.parameters.dns.iter().collect();
        let expected: BTreeSet<_> = self.dns.iter().collect();
        if b.parameters.router != self.router
            || actual != expected
            || actual.len() != b.parameters.dns.len()
        {
            return (
                false,
                format!(
                    "{}: held router/DNS options differ; Apply alone does not replace a held lease",
                    self.client
                ),
            );
        }
        if b.remaining(s.now()) < u64::from(self.minimum_remaining) {
            return (
                false,
                format!(
                    "{}: remaining lease time is below the requested minimum",
                    self.client
                ),
            );
        }
        (
            true,
            format!(
                "{}: current client lease has the requested IPv4 parameters",
                self.client
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    ClientConfiguration {
        expected: ClientConfiguration,
    },
    NoClientLease {
        #[serde(deserialize_with = "name")]
        client: String,
    },
    PoolPreserved {
        #[serde(deserialize_with = "name")]
        pool: String,
    },
    RelayPreserved {
        #[serde(deserialize_with = "name")]
        segment: String,
    },
    BindingPreserved {
        #[serde(deserialize_with = "name")]
        client: String,
    },
    ServerEnabled {
        enabled: bool,
    },
    Excluded {
        #[serde(deserialize_with = "name")]
        pool: String,
        #[serde(deserialize_with = "ip")]
        address: Ipv4Addr,
    },
    Reserved {
        #[serde(deserialize_with = "name")]
        pool: String,
        #[serde(deserialize_with = "name")]
        client: String,
        #[serde(deserialize_with = "ip")]
        address: Ipv4Addr,
    },
}
impl Requirement {
    fn valid(&self, initial: &State) -> bool {
        let t = initial.topology();
        match self {
            Self::ClientConfiguration { expected } => expected.valid(t),
            Self::NoClientLease { client } | Self::BindingPreserved { client } => {
                key(client)
                    && t.clients.contains_key(client)
                    && (!matches!(self, Self::BindingPreserved { .. })
                        || initial.bindings().contains_key(client))
            }
            Self::PoolPreserved { pool } => key(pool) && initial.policy().pools.contains_key(pool),
            Self::RelayPreserved { segment } => key(segment) && t.segments.contains_key(segment),
            Self::ServerEnabled { .. } => true,
            Self::Excluded { pool, address } | Self::Reserved { pool, address, .. } => {
                key(pool)
                    && initial
                        .policy()
                        .pools
                        .get(pool)
                        .is_some_and(|p| p.subnet.host(*address))
                    && match self {
                        Self::Reserved { client, .. } => {
                            key(client) && t.clients.contains_key(client)
                        }
                        _ => true,
                    }
            }
        }
    }
    fn check(&self, initial: &State, s: &State) -> (bool, String) {
        match self {
            Self::ClientConfiguration { expected } => expected.check(s),
            Self::NoClientLease { client } => (
                !s.leases().contains_key(client),
                format!(
                    "{client}: {}",
                    if s.leases().contains_key(client) {
                        "client still holds a lease"
                    } else {
                        "no client-held lease; a server binding may remain after a lost Release"
                    }
                ),
            ),
            Self::PoolPreserved { pool } => {
                let ok = s.policy().pools.get(pool) == initial.policy().pools.get(pool);
                (
                    ok,
                    format!(
                        "{pool}: {}",
                        if ok {
                            "unaffected pool policy is preserved"
                        } else {
                            "restore the unaffected pool policy"
                        }
                    ),
                )
            }
            Self::RelayPreserved { segment } => {
                let ok = s.policy().relays.get(segment) == initial.policy().relays.get(segment);
                (
                    ok,
                    format!(
                        "{segment}: {}",
                        if ok {
                            "unaffected relay policy is preserved"
                        } else {
                            "restore the unaffected relay policy"
                        }
                    ),
                )
            }
            Self::BindingPreserved { client } => {
                let ok = s.bindings().get(client) == initial.bindings().get(client)
                    && s.leases().get(client) == initial.leases().get(client);
                (
                    ok,
                    format!(
                        "{client}: {}",
                        if ok {
                            "original grant and client-held state are preserved"
                        } else {
                            "original grant or client-held state changed"
                        }
                    ),
                )
            }
            Self::ServerEnabled { enabled } => {
                let ok = s.policy().server_enabled == *enabled;
                (
                    ok,
                    if ok {
                        "Server enable setting matches"
                    } else {
                        "Server enable setting differs"
                    }
                    .into(),
                )
            }
            Self::Excluded { pool, address } => {
                let ok = s
                    .policy()
                    .pools
                    .get(pool)
                    .is_some_and(|p| p.excluded.contains(address));
                (
                    ok,
                    format!(
                        "{pool}: {}",
                        if ok {
                            "requested protected address is excluded"
                        } else {
                            "requested protected address is not excluded"
                        }
                    ),
                )
            }
            Self::Reserved {
                pool,
                client,
                address,
            } => {
                let ok = s
                    .policy()
                    .pools
                    .get(pool)
                    .is_some_and(|p| p.reservations.get(client) == Some(address));
                (
                    ok,
                    format!(
                        "{client}: {}",
                        if ok {
                            "requested reservation is configured"
                        } else {
                            "requested reservation differs"
                        }
                    ),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    /// Order affects allocation, not the accepted dynamic address. No actions or
    /// expected event history can be injected by a saved learner workspace.
    #[serde(deserialize_with = "list::<_,ClientConfiguration,8>")]
    pub clients: Vec<ClientConfiguration>,
}
pub fn requirements<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    list::<D, Requirement, MAX_REQUIREMENTS>(d)
        .map_err(|_| D::Error::custom("Invalid bounded DHCP requirements"))
}
pub fn probes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Probe>, D::Error> {
    list::<D, Probe, MAX_PROBES>(d).map_err(|_| D::Error::custom("Invalid bounded DHCP probes"))
}
pub fn valid_goal(initial: &State, requirements: &[Requirement], probes: &[Probe]) -> bool {
    if initial.validate().is_err()
        || requirements.is_empty()
        || requirements.len() > MAX_REQUIREMENTS
        || probes.is_empty()
        || probes.len() > MAX_PROBES
    {
        return false;
    }
    if !requirements.iter().all(|r| r.valid(initial))
        || !requirements
            .iter()
            .any(|r| matches!(r, Requirement::ClientConfiguration { .. }))
        || requirements
            .iter()
            .enumerate()
            .any(|(i, r)| requirements[..i].contains(r))
    {
        return false;
    }
    if probes
        .iter()
        .enumerate()
        .any(|(i, p)| probes[..i].contains(p))
    {
        return false;
    }
    let mut total = 0usize;
    for p in probes {
        if p.clients.is_empty()
            || p.clients.len() > MAX_PROBE_CLIENTS
            || !p.clients.iter().all(|c| c.valid(initial.topology()))
        {
            return false;
        }
        total += p.clients.len();
        if total > MAX_TOTAL_PROBE_CLIENTS {
            return false;
        }
        let ids: BTreeSet<_> = p.clients.iter().map(|c| &c.client).collect();
        if ids.len() != p.clients.len() {
            return false;
        }
    }
    true
}
pub fn checks(
    initial: &State,
    s: &State,
    requirements: &[Requirement],
    probes: &[Probe],
) -> Vec<(bool, String)> {
    if !valid_goal(initial, requirements, probes) || s.validate_context(initial.topology()).is_err()
    {
        return vec![(
            false,
            "DHCP assignment context or authored objectives differ; previous work retained".into(),
        )];
    }
    let mut out: Vec<_> = requirements.iter().map(|r| r.check(initial, s)).collect();
    for (n, p) in probes.iter().enumerate() {
        let mut fresh = match State::new(initial.topology().clone(), s.policy().clone()) {
            Ok(v) => v,
            Err(_) => {
                out.push((
                    false,
                    format!("New-client test {}: applied policy is invalid", n + 1),
                ));
                continue;
            }
        };
        for c in &p.clients {
            let exchange = fresh.discover(&c.client).and_then(|offer| match offer {
                Outcome::Offered(o) => fresh.request(&c.client, o.token),
                other => Ok(other),
            });
            if !matches!(exchange, Ok(Outcome::Ack { .. })) {
                out.push((
                    false,
                    format!(
                        "New-client test {} · {}: applied policy did not configure a client lease",
                        n + 1,
                        c.client
                    ),
                ));
            } else {
                let (ok, text) = c.check(&fresh);
                out.push((ok, format!("New-client test {} · {text}", n + 1)));
            }
        }
    }
    out.into_iter()
        .map(|(ok, text)| (ok, notice(text)))
        .collect()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    ApplyPolicy {
        policy: Policy,
    },
    Discover {
        #[serde(deserialize_with = "name")]
        client: String,
    },
    Request {
        #[serde(deserialize_with = "name")]
        client: String,
        token: u64,
    },
    Renew {
        #[serde(deserialize_with = "name")]
        client: String,
    },
    Rebind {
        #[serde(deserialize_with = "name")]
        client: String,
    },
    Release {
        #[serde(deserialize_with = "name")]
        client: String,
    },
    Advance {
        seconds: u64,
    },
}
fn outcome(outcome: Outcome) -> String {
    notice(match outcome{
 Outcome::Offered(o)=>format!("Offer received for {} with {} seconds; client is not configured until Request",o.address,o.lease_seconds),
 Outcome::Ack{binding,kind}=>format!("{}: client holds {} with {} seconds in the grant",match kind{dhcpv4::AckKind::NewLease=>"New lease acknowledged",dhcpv4::AckKind::Renewed=>"Renewed grant acknowledged; current policy applied",dhcpv4::AckKind::CurrentLease=>"Existing lease acknowledged; held options and expiry unchanged"},binding.address,binding.granted_seconds),
 Outcome::Nak{..}=>"Request rejected by server; inspect policy and discover again".into(),
 Outcome::NoReply(reason)=>format!("No reply: {}",match reason{dhcpv4::Reason::ServerDisabled=>"server disabled",dhcpv4::Reason::NoRelay=>"no enabled relay",dhcpv4::Reason::WrongRelayTarget=>"relay targets another server",dhcpv4::Reason::WrongRelayAddress=>"relay client-facing address differs",dhcpv4::Reason::RelayUnreachable=>"supplied relay path is unavailable",dhcpv4::Reason::UnicastUnreachable=>"supplied unicast path is unavailable",dhcpv4::Reason::NoPool=>"no matching pool",dhcpv4::Reason::PoolChanged=>"offered subnet no longer selected",dhcpv4::Reason::Exhausted=>"pool has no eligible available address",dhcpv4::Reason::ReservationUnavailable=>"reservation unavailable",dhcpv4::Reason::NoOffer=>"no current offer",dhcpv4::Reason::StaleOffer=>"stale offer token"}),
 Outcome::Released{server_notified,..}=>if server_notified{"Client stopped using the lease; server binding released"}else{"Client stopped using the lease; server binding retained because notification could not arrive"}.into(),
 Outcome::Advanced{clients,bindings,offers}=>format!("Simulated time advanced; expired {} client leases, {} server bindings and {} offers; no requests sent",clients.len(),bindings.len(),offers.len()),
})
}
pub fn apply(initial: &State, s: &mut State, action: &Action) -> Result<String, String> {
    initial.validate().map_err(|e| notice(e.message))?;
    s.validate_context(initial.topology())
        .map_err(|e| notice(e.message))?;
    if !action.valid(initial.topology()) {
        return Err("Invalid DHCP action bounds or supplied identity; state retained".into());
    }
    let mut candidate = s.clone();
    let text = match action {
        Action::ApplyPolicy { policy } => {
            candidate
                .replace_policy(policy.clone())
                .map_err(|e| notice(e.message))?;
            "Policy applied; offers and held client parameters remain unchanged until an actual exchange".into()
        }
        Action::Discover { client } => {
            outcome(candidate.discover(client).map_err(|e| notice(e.message))?)
        }
        Action::Request { client, token } => outcome(
            candidate
                .request(client, *token)
                .map_err(|e| notice(e.message))?,
        ),
        Action::Renew { client } => {
            outcome(candidate.renew(client).map_err(|e| notice(e.message))?)
        }
        Action::Rebind { client } => {
            outcome(candidate.rebind(client).map_err(|e| notice(e.message))?)
        }
        Action::Release { client } => {
            outcome(candidate.release(client).map_err(|e| notice(e.message))?)
        }
        Action::Advance { seconds } => {
            outcome(candidate.advance(*seconds).map_err(|e| notice(e.message))?)
        }
    };
    candidate
        .validate_context(initial.topology())
        .map_err(|e| notice(e.message))?;
    *s = candidate;
    Ok(text)
}
impl Action {
    pub fn valid(&self, t: &Topology) -> bool {
        match self {
            Self::ApplyPolicy { policy } => policy.validate(t).is_ok(),
            Self::Advance { seconds } => *seconds <= dhcpv4::MAX_ADVANCE,
            Self::Discover { client }
            | Self::Request { client, .. }
            | Self::Renew { client }
            | Self::Rebind { client }
            | Self::Release { client } => key(client) && t.clients.contains_key(client),
        }
    }
}
pub fn execute(initial: &State, s: &mut State, input: &str) -> Result<String, String> {
    if input.len() > MAX_ACTION_BYTES {
        return Err("DHCP authoring action exceeds256KiB".into());
    }
    let json = input
        .strip_prefix("dhcpv4 ")
        .ok_or("Use native DHCP controls; this is not Linux or IOS command syntax")?;
    let a: Action = serde_json::from_str(json)
        .map_err(|_| "Invalid bounded typed DHCP authoring action".to_owned())?;
    apply(initial, s, &a)
}
/// Read-only derived summary, never a connectivity or completion credential.
pub fn describe(s: &State) -> String {
    format!(
        "Virtual DHCP: {} supplied segments, {} clients, {} configured pools; time {}; {} offers, {} server bindings, {} client-held leases. Parameters do not prove routed connectivity or DNS service.",
        s.topology().segments.len(),
        s.topology().clients.len(),
        s.policy().pools.len(),
        s.now(),
        s.offers().len(),
        s.bindings().len(),
        s.leases().len()
    )
}
