//! Bounded ideal Basic NAT address rewriting; no packets, ports, host I/O or reachability.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::{self, Write},
    net::Ipv4Addr,
};
pub const MAX_STATE_JSON: usize = 256 * 1024;
pub const MAX_ACTION_JSON: usize = 64 * 1024;
pub const MAX_NOTICE: usize = 512;
pub const MAX_INSIDE: usize = 64;
pub const MAX_OUTSIDE: usize = 64;
pub const MAX_GLOBALS: usize = 128;
pub const MAX_STATICS: usize = 32;
pub const MAX_POOL: u64 = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Limit,
    Invalid,
    Context,
    Unsupported,
    ClearRequired,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub message: String,
}
impl Fault {
    fn new(kind: FaultKind, message: &str) -> Self {
        let mut n = message.len().min(MAX_NOTICE);
        while !message.is_char_boundary(n) {
            n -= 1;
        }
        Self {
            kind,
            message: message[..n].to_owned(),
        }
    }
}
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
type Result<T> = std::result::Result<T, Fault>;
fn fail<T>(kind: FaultKind, text: &str) -> Result<T> {
    Err(Fault::new(kind, text))
}
fn identity(s: &str, n: usize) -> bool {
    !s.is_empty()
        && s.len() <= n
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn id<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    struct Text;
    impl<'de> Visitor<'de> for Text {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an ASCII ID of1..32 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if !identity(s, 32) {
                return Err(E::custom("NAT identity bound/syntax"));
            }
            Ok(s.to_owned())
        }
    }
    d.deserialize_str(Text)
}
struct Key {
    allowed: bool,
}
impl<'de> DeserializeSeed<'de> for Key {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("NAT map count limit"));
        }
        id(d)
    }
}
fn mapping<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, Ipv4Addr>, D::Error> {
    struct Items<const N: usize>;
    impl<'de, const N: usize> Visitor<'de> for Items<N> {
        type Value = BTreeMap<String, Ipv4Addr>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} unique NAT IDs")
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
                    return Err(A::Error::custom("Duplicate NAT identity"));
                }
                let value = a.next_value()?;
                out.insert(k, value);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Items::<N>)
}
struct Address {
    allowed: bool,
}
impl<'de> DeserializeSeed<'de> for Address {
    type Value = Ipv4Addr;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<Ipv4Addr, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("NAT global count limit"));
        }
        Ipv4Addr::deserialize(d)
    }
}
fn globals<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeSet<Ipv4Addr>, D::Error> {
    struct Items;
    impl<'de> Visitor<'de> for Items {
        type Value = BTreeSet<Ipv4Addr>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most128 distinct global addresses")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeSet::new();
            while let Some(v) = a.next_element_seed(Address {
                allowed: out.len() < MAX_GLOBALS,
            })? {
                if !out.insert(v) {
                    return Err(A::Error::custom("Duplicate NAT global address"));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "id")]
    pub fixture: String,
    #[serde(deserialize_with = "mapping::<_,MAX_INSIDE>")]
    pub inside: BTreeMap<String, Ipv4Addr>,
    #[serde(deserialize_with = "mapping::<_,MAX_OUTSIDE>")]
    pub outside: BTreeMap<String, Ipv4Addr>,
    #[serde(deserialize_with = "globals")]
    pub globals: BTreeSet<Ipv4Addr>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub network: Ipv4Addr,
    pub length: u8,
}
impl Prefix {
    pub fn valid(self) -> bool {
        self.length <= 32 && (u32::from(self.network) & !self.mask()) == 0
    }
    fn mask(self) -> u32 {
        if self.length == 0 {
            0
        } else if self.length <= 32 {
            u32::MAX << (32 - self.length)
        } else {
            0
        }
    }
    pub fn contains(self, address: Ipv4Addr) -> bool {
        self.valid() && (u32::from(address) & self.mask()) == u32::from(self.network)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    pub first: Ipv4Addr,
    pub last: Ipv4Addr,
}
impl Pool {
    pub fn cardinality(self) -> Result<u64> {
        let first = u64::from(u32::from(self.first));
        let last = u64::from(u32::from(self.last));
        if last < first {
            return fail(FaultKind::Invalid, "NAT pool endpoints are reversed");
        }
        let n = last - first + 1;
        if n > MAX_POOL {
            return fail(FaultKind::Limit, "NAT pool exceeds64 addresses");
        }
        Ok(n)
    }
    pub fn contains(self, a: Ipv4Addr) -> bool {
        u32::from(self.first) <= u32::from(a) && u32::from(a) <= u32::from(self.last)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicRule {
    pub source: Prefix,
    pub pool: Pool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "mapping::<_,MAX_STATICS>")]
    pub statics: BTreeMap<String, Ipv4Addr>,
    pub dynamic: Option<DynamicRule>,
}
fn inventory_shape(i: &Inventory) -> Result<()> {
    if !identity(&i.fixture, 32)
        || i.inside.len() > MAX_INSIDE
        || i.outside.len() > MAX_OUTSIDE
        || i.globals.len() > MAX_GLOBALS
    {
        return fail(FaultKind::Limit, "NAT inventory identity/count bound");
    }
    if i.inside
        .keys()
        .chain(i.outside.keys())
        .any(|n| !identity(n, 32))
    {
        return fail(FaultKind::Invalid, "NAT endpoint ID needs1..32 ASCII bytes");
    }
    Ok(())
}
fn inventory_valid(i: &Inventory) -> Result<()> {
    inventory_shape(i)?;
    let mut ids = BTreeSet::new();
    let mut addresses = BTreeSet::new();
    for (name, address) in i.inside.iter().chain(&i.outside) {
        if !ids.insert(name) || !addresses.insert(*address) {
            return fail(
                FaultKind::Unsupported,
                "Duplicate/overlapping endpoint identities or address realms are outside this NAT subset",
            );
        }
    }
    if i.globals.iter().any(|a| addresses.contains(a)) {
        return fail(
            FaultKind::Unsupported,
            "Translated globals must be distinct from actual endpoint addresses in this subset",
        );
    }
    Ok(())
}
fn config_shape(c: &Configuration) -> Result<()> {
    if c.statics.len() > MAX_STATICS {
        return fail(FaultKind::Limit, "NAT static map count exceeds32");
    }
    if c.statics.keys().any(|n| !identity(n, 32)) {
        return fail(FaultKind::Invalid, "NAT static endpoint ID bound/syntax");
    }
    if let Some(d) = c.dynamic {
        if !d.source.valid() {
            return fail(
                FaultKind::Invalid,
                "NAT dynamic prefix needs a canonical IPv4 network and length0..32",
            );
        }
        d.pool.cardinality()?;
    }
    Ok(())
}
fn config_valid(i: &Inventory, c: &Configuration) -> Result<()> {
    config_shape(c)?;
    let mut globals = BTreeSet::new();
    for (inside, global) in &c.statics {
        if !i.inside.contains_key(inside) || !i.globals.contains(global) {
            return fail(
                FaultKind::Context,
                "NAT static map uses a foreign endpoint/global",
            );
        }
        if !globals.insert(*global) {
            return fail(
                FaultKind::Invalid,
                "Two static maps cannot reserve the same global",
            );
        }
    }
    if let Some(d) = c.dynamic {
        let n = d.pool.cardinality()?;
        let start = u64::from(u32::from(d.pool.first));
        for offset in 0..n {
            let address = Ipv4Addr::from((start + offset) as u32);
            if !i.globals.contains(&address) {
                return fail(
                    FaultKind::Context,
                    "Every dynamic pool address must be supplied in allowed globals",
                );
            }
        }
    }
    Ok(())
}
struct Counter {
    used: usize,
    cap: usize,
}
impl Write for Counter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() > self.cap.saturating_sub(self.used) {
            return Err(io::Error::other("NAT JSON byte bound"));
        }
        self.used += b.len();
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    serde_json::to_writer(Counter { used: 0, cap }, v).is_ok()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    schema_version: u32,
    inventory: Inventory,
    configuration: Configuration,
    bindings: BTreeMap<String, Ipv4Addr>,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema_version: u32,
            inventory: Inventory,
            configuration: Configuration,
            #[serde(deserialize_with = "mapping::<_,MAX_INSIDE>")]
            bindings: BTreeMap<String, Ipv4Addr>,
        }
        let r = Raw::deserialize(d)?;
        let state = Self {
            schema_version: r.schema_version,
            inventory: r.inventory,
            configuration: r.configuration,
            bindings: r.bindings,
        };
        state.validate().map_err(|e| D::Error::custom(e.message))?;
        Ok(state)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Configure {
        configuration: Configuration,
    },
    ClearDynamic {},
    Outbound {
        #[serde(deserialize_with = "id")]
        inside: String,
        #[serde(deserialize_with = "id")]
        peer: String,
    },
    Inbound {
        #[serde(deserialize_with = "id")]
        peer: String,
        global: Ipv4Addr,
    },
}
impl Action {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_ACTION_JSON {
            return fail(FaultKind::Limit, "NAT action JSON exceeds64KiB");
        }
        let a: Self = serde_json::from_slice(raw)
            .map_err(|_| Fault::new(FaultKind::Invalid, "Invalid bounded NAT action JSON"))?;
        a.shape()?;
        Ok(a)
    }
    fn shape(&self) -> Result<()> {
        match self {
            Self::Configure { configuration } => config_shape(configuration)?,
            Self::ClearDynamic {} => {}
            Self::Outbound { inside, peer } => {
                if !identity(inside, 32) || !identity(peer, 32) {
                    return fail(FaultKind::Invalid, "NAT probe endpoint ID bound/syntax");
                }
            }
            Self::Inbound { peer, .. } => {
                if !identity(peer, 32) {
                    return fail(FaultKind::Invalid, "NAT probe peer ID bound/syntax");
                }
            }
        }
        if !within(self, MAX_ACTION_JSON) {
            return fail(FaultKind::Limit, "NAT action exceeds compact admission");
        }
        Ok(())
    }
    pub fn validate_for(&self, i: &Inventory) -> Result<()> {
        inventory_shape(i)?;
        self.shape()?;
        inventory_valid(i)?;
        match self {
            Self::Configure { configuration } => config_valid(i, configuration),
            Self::ClearDynamic {} => Ok(()),
            Self::Outbound { inside, peer } => {
                if i.inside.contains_key(inside) && i.outside.contains_key(peer) {
                    Ok(())
                } else {
                    fail(
                        FaultKind::Context,
                        "NAT probe needs supplied inside/outside identities",
                    )
                }
            }
            Self::Inbound { peer, .. } => {
                if i.outside.contains_key(peer) {
                    Ok(())
                } else {
                    fail(
                        FaultKind::Context,
                        "NAT inbound probe needs a supplied outside identity",
                    )
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Addresses {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MappingKind {
    Static,
    Dynamic,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ProbeResult {
    Translated {
        before: Addresses,
        after: Addresses,
        kind: MappingKind,
    },
    Untranslated {
        addresses: Addresses,
    },
    NoMapping {
        addresses: Addresses,
    },
    PoolExhausted {
        addresses: Addresses,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Effect {
    Configured,
    Cleared { removed: usize },
    Probe { result: ProbeResult },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Mapping {
    pub inside: String,
    pub local: Ipv4Addr,
    pub global: Ipv4Addr,
    pub kind: MappingKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub mappings: Vec<Mapping>,
    pub free_pool: Vec<Ipv4Addr>,
}
impl State {
    pub fn new(i: &Inventory, c: &Configuration) -> Result<Self> {
        inventory_shape(i)?;
        config_shape(c)?;
        inventory_valid(i)?;
        config_valid(i, c)?;
        let s = Self {
            schema_version: 1,
            inventory: i.clone(),
            configuration: c.clone(),
            bindings: BTreeMap::new(),
        };
        s.validate()?;
        Ok(s)
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn bindings(&self) -> &BTreeMap<String, Ipv4Addr> {
        &self.bindings
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_STATE_JSON {
            return fail(FaultKind::Limit, "NAT State JSON exceeds256KiB");
        }
        serde_json::from_slice(raw)
            .map_err(|_| Fault::new(FaultKind::Invalid, "Invalid bounded NAT State JSON"))
    }
    pub fn from_json_for(raw: &[u8], expected: &Inventory) -> Result<Self> {
        if raw.len() > MAX_STATE_JSON {
            return fail(FaultKind::Limit, "NAT State JSON exceeds256KiB");
        }
        inventory_valid(expected)?;
        let s = Self::from_json(raw)?;
        s.valid_against(expected)?;
        Ok(s)
    }
    pub fn valid_against(&self, expected: &Inventory) -> Result<()> {
        inventory_valid(expected)?;
        self.validate()?;
        if &self.inventory != expected {
            return fail(
                FaultKind::Context,
                "NAT supplied inventory differs from fixed context",
            );
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        inventory_shape(&self.inventory)?;
        config_shape(&self.configuration)?;
        if self.schema_version != 1 || self.bindings.len() > MAX_INSIDE {
            return fail(FaultKind::Limit, "NAT State version/binding count bound");
        }
        if self.bindings.keys().any(|n| !identity(n, 32)) {
            return fail(FaultKind::Invalid, "NAT binding endpoint ID bound/syntax");
        }
        inventory_valid(&self.inventory)?;
        config_valid(&self.inventory, &self.configuration)?;
        let mut assigned = BTreeSet::new();
        for (inside, global) in &self.bindings {
            let Some(local) = self.inventory.inside.get(inside) else {
                return fail(
                    FaultKind::Context,
                    "NAT binding has foreign inside identity",
                );
            };
            let Some(rule) = self.configuration.dynamic else {
                return fail(
                    FaultKind::ClearRequired,
                    "Clear dynamic bindings before disabling their rule",
                );
            };
            if !rule.source.contains(*local)
                || !rule.pool.contains(*global)
                || !self.inventory.globals.contains(global)
                || self.configuration.statics.contains_key(inside)
                || self.configuration.statics.values().any(|a| a == global)
                || !assigned.insert(*global)
            {
                return fail(
                    FaultKind::ClearRequired,
                    "Dynamic bindings conflict with this configuration; clear them explicitly first",
                );
            }
        }
        if !within(self, MAX_STATE_JSON) {
            return fail(FaultKind::Limit, "NAT State exceeds compact admission");
        }
        Ok(())
    }
    pub fn apply_json(&mut self, raw: &[u8]) -> Result<Effect> {
        let a = Action::from_json(raw)?;
        self.apply(&a)
    }
    pub fn apply(&mut self, a: &Action) -> Result<Effect> {
        a.shape()?;
        self.validate()?;
        a.validate_for(&self.inventory)?;
        match a {
            Action::Configure { configuration } => {
                for (inside, global) in &self.bindings {
                    let Some(rule) = configuration.dynamic else {
                        return fail(
                            FaultKind::ClearRequired,
                            "Clear dynamic bindings before disabling their rule",
                        );
                    };
                    if !rule.source.contains(self.inventory.inside[inside])
                        || !rule.pool.contains(*global)
                        || configuration.statics.contains_key(inside)
                        || configuration.statics.values().any(|g| g == global)
                    {
                        return fail(
                            FaultKind::ClearRequired,
                            "Clear dynamic bindings before changing an occupied mapping",
                        );
                    }
                }
                self.configuration = configuration.clone();
                Ok(Effect::Configured)
            }
            Action::ClearDynamic {} => {
                let removed = self.bindings.len();
                self.bindings.clear();
                Ok(Effect::Cleared { removed })
            }
            Action::Outbound { inside, peer } => {
                let before = Addresses {
                    source: self.inventory.inside[inside],
                    destination: self.inventory.outside[peer],
                };
                let (global, kind) = if let Some(g) = self.configuration.statics.get(inside) {
                    (*g, MappingKind::Static)
                } else if let Some(g) = self.bindings.get(inside) {
                    (*g, MappingKind::Dynamic)
                } else {
                    let Some(rule) = self.configuration.dynamic else {
                        return Ok(Effect::Probe {
                            result: ProbeResult::Untranslated { addresses: before },
                        });
                    };
                    if !rule.source.contains(before.source) {
                        return Ok(Effect::Probe {
                            result: ProbeResult::Untranslated { addresses: before },
                        });
                    }
                    let n = rule.pool.cardinality()?;
                    let start = u64::from(u32::from(rule.pool.first));
                    let free = (0..n)
                        .map(|o| Ipv4Addr::from((start + o) as u32))
                        .find(|g| {
                            !self.configuration.statics.values().any(|v| v == g)
                                && !self.bindings.values().any(|v| v == g)
                        });
                    let Some(g) = free else {
                        return Ok(Effect::Probe {
                            result: ProbeResult::PoolExhausted { addresses: before },
                        });
                    };
                    if self.bindings.len() >= MAX_INSIDE {
                        return fail(FaultKind::Limit, "NAT binding capacity reached");
                    }
                    self.bindings.insert(inside.clone(), g);
                    (g, MappingKind::Dynamic)
                };
                Ok(Effect::Probe {
                    result: ProbeResult::Translated {
                        before,
                        after: Addresses {
                            source: global,
                            destination: before.destination,
                        },
                        kind,
                    },
                })
            }
            Action::Inbound { peer, global } => {
                let before = Addresses {
                    source: self.inventory.outside[peer],
                    destination: *global,
                };
                let found = self
                    .configuration
                    .statics
                    .iter()
                    .find(|(_, g)| **g == *global)
                    .map(|(id, _)| (id, MappingKind::Static))
                    .or_else(|| {
                        self.bindings
                            .iter()
                            .find(|(_, g)| **g == *global)
                            .map(|(id, _)| (id, MappingKind::Dynamic))
                    });
                Ok(Effect::Probe {
                    result: if let Some((id, kind)) = found {
                        ProbeResult::Translated {
                            before,
                            after: Addresses {
                                source: before.source,
                                destination: self.inventory.inside[id],
                            },
                            kind,
                        }
                    } else {
                        ProbeResult::NoMapping { addresses: before }
                    },
                })
            }
        }
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        self.validate()?;
        let mut mappings = Vec::new();
        for (maps, kind) in [
            (&self.configuration.statics, MappingKind::Static),
            (&self.bindings, MappingKind::Dynamic),
        ] {
            for (inside, global) in maps {
                mappings.push(Mapping {
                    inside: inside.clone(),
                    local: self.inventory.inside[inside],
                    global: *global,
                    kind,
                });
            }
        }
        let mut free_pool = Vec::new();
        if let Some(rule) = self.configuration.dynamic {
            let n = rule.pool.cardinality()?;
            let start = u64::from(u32::from(rule.pool.first));
            for o in 0..n {
                let g = Ipv4Addr::from((start + o) as u32);
                if !mappings.iter().any(|m| m.global == g) {
                    free_pool.push(g);
                }
            }
        }
        Ok(Snapshot {
            mappings,
            free_pool,
        })
    }
}
