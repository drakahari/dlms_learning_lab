//! Native typed OSPF settings, unfinished drafts and fresh goal predicates.
//! No IOS/shell interpreter, protocol packets, timers or cached adjacency credit.
use crate::{
    ipv4_routing::{Assignment, Prefix, Topology, supported_address},
    ospf::{
        self, Action, Configuration, Endpoint, NetworkType, NextHop, PortConfig, ProcessConfig,
        RouterConfig, State,
    },
};
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
pub const MAX_REQUIREMENTS: usize = 32;
pub const MAX_DRAFT_BYTES: usize = 262_144;
pub const MAX_TEXT: usize = 64;
pub const HELP: &str = "Edit the supplied virtual router/process/interface settings, apply them and check current work. Results are derived point-to-point compatibility and remote SPF candidates, not observed Full adjacency, packet delivery, DR/BDR or IOS commands. No host network, packets or shell is used.";
pub fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn bounded(s: &str, n: usize) -> bool {
    s.len() <= n && !s.chars().any(char::is_control)
}
pub fn notice(s: &str) -> String {
    let mut n = s.len().min(ospf::MAX_ERROR_BYTES);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].to_owned()
}
struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V<const N: usize>;
        impl<const N: usize> Visitor<'_> for V<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} UTF-8 bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if bounded(s, N) {
                    Ok(Text(s.into()))
                } else {
                    Err(E::custom("OSPF text limit or control character"))
                }
            }
        }
        d.deserialize_str(V::<N>)
    }
}
fn text<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Text::<MAX_TEXT>::deserialize(d)?.0)
}
fn optional_notice<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<Text<512>>::deserialize(d)?.map(|s| s.0))
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for M<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} OSPF entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Text(k)) = a.next_key::<Text<32>>()? {
                if !name(&k) || out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom(
                        "OSPF duplicate key, name or collection bound",
                    ));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<V, N>(PhantomData))
}
pub(crate) fn list<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<V>, D::Error> {
    struct L<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for L<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} OSPF entries")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < N {
                match a
                    .next_element()
                    .map_err(|_| A::Error::custom("Invalid bounded OSPF list item"))?
                {
                    Some(v) => out.push(v),
                    None => return Ok(out),
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("OSPF collection bound"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(L::<V, N>(PhantomData))
}
fn routers<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, RouterDraft>, D::Error> {
    map::<D, RouterDraft, 12>(d)
}
fn ports<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, PortDraft>, D::Error> {
    map::<D, PortDraft, 8>(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortDraft {
    pub up: bool,
    #[serde(deserialize_with = "text")]
    pub address: String,
    #[serde(deserialize_with = "text")]
    pub prefix: String,
    pub enabled: bool,
    pub passive: bool,
    pub network_type: NetworkType,
    #[serde(deserialize_with = "text")]
    pub cost: String,
    #[serde(deserialize_with = "text")]
    pub hello_seconds: String,
    #[serde(deserialize_with = "text")]
    pub dead_seconds: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterDraft {
    pub enabled: bool,
    #[serde(deserialize_with = "text")]
    pub process_id: String,
    #[serde(deserialize_with = "text")]
    pub router_id: String,
    #[serde(deserialize_with = "text")]
    pub area: String,
    #[serde(deserialize_with = "ports")]
    pub interfaces: BTreeMap<String, PortDraft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u8,
    #[serde(deserialize_with = "routers")]
    pub routers: BTreeMap<String, RouterDraft>,
    pub selected: Endpoint,
    #[serde(default, deserialize_with = "optional_notice")]
    pub notice: Option<String>,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Data {
            version: u8,
            #[serde(deserialize_with = "routers")]
            routers: BTreeMap<String, RouterDraft>,
            selected: Endpoint,
            #[serde(default, deserialize_with = "optional_notice")]
            notice: Option<String>,
        }
        let data = Data::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded OSPF editing data"))?;
        let draft = Self {
            version: data.version,
            routers: data.routers,
            selected: data.selected,
            notice: data.notice,
        };
        if !draft.valid() {
            return Err(D::Error::custom("Invalid bounded OSPF editing identities"));
        }
        Ok(draft)
    }
}
pub fn same_context(initial: &State, state: &State) -> bool {
    state.valid_against(initial)
        && initial
            .topology()
            .devices
            .iter()
            .filter(|(_, d)| !d.forwarding)
            .all(|(id, d)| state.topology().devices.get(id) == Some(d))
}
fn integer(s: &str, label: &str) -> Result<u32, String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{label}: enter a whole decimal number"));
    }
    s.parse::<u32>()
        .map_err(|_| format!("{label}: number is outside the supported range"))
}
fn small(s: &str, label: &str) -> Result<u16, String> {
    let n = integer(s, label)?;
    u16::try_from(n)
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("{label}: enter 1 through 65535"))
}
impl Draft {
    pub fn new(state: &State) -> Result<Self, String> {
        if !state.valid() {
            return Err("Cannot edit an invalid OSPF state".into());
        }
        let mut routers = BTreeMap::new();
        for (id, c) in &state.configuration().routers {
            let mut interfaces = BTreeMap::new();
            for (port, p) in &c.interfaces {
                let i = &state.topology().devices[id].interfaces[port];
                interfaces.insert(
                    port.clone(),
                    PortDraft {
                        up: i.up,
                        address: i
                            .address
                            .as_ref()
                            .map_or(String::new(), |a| a.address.to_string()),
                        prefix: i
                            .address
                            .as_ref()
                            .map_or(String::new(), |a| a.prefix.to_string()),
                        enabled: p.enabled,
                        passive: p.passive,
                        network_type: p.network_type,
                        cost: p.cost.to_string(),
                        hello_seconds: p.hello_seconds.to_string(),
                        dead_seconds: p.dead_seconds.to_string(),
                    },
                );
            }
            routers.insert(
                id.clone(),
                RouterDraft {
                    enabled: c.process.enabled,
                    process_id: c.process.process_id.to_string(),
                    router_id: c
                        .process
                        .router_id
                        .map_or(String::new(), |id| id.to_string()),
                    area: c.process.area.to_string(),
                    interfaces,
                },
            );
        }
        let router = routers.keys().next().unwrap().clone();
        let interface = routers[&router].interfaces.keys().next().unwrap().clone();
        Ok(Self {
            version: 1,
            routers,
            selected: Endpoint { router, interface },
            notice: None,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && !self.routers.is_empty()
            && self.routers.len() <= 12
            && self.routers.iter().all(|(id, r)| {
                name(id)
                    && [&r.process_id, &r.router_id, &r.area]
                        .iter()
                        .all(|s| bounded(s, MAX_TEXT))
                    && !r.interfaces.is_empty()
                    && r.interfaces.len() <= 8
                    && r.interfaces.iter().all(|(id, p)| {
                        name(id)
                            && [
                                &p.address,
                                &p.prefix,
                                &p.cost,
                                &p.hello_seconds,
                                &p.dead_seconds,
                            ]
                            .iter()
                            .all(|s| bounded(s, MAX_TEXT))
                    })
            })
            && self
                .routers
                .get(&self.selected.router)
                .is_some_and(|r| r.interfaces.contains_key(&self.selected.interface))
            && self.notice.as_ref().is_none_or(|s| bounded(s, 512))
    }
    pub fn same_shape(&self, state: &State) -> bool {
        self.valid()
            && state.valid()
            && self.routers.keys().eq(state.configuration().routers.keys())
            && self.routers.iter().all(|(id, r)| {
                r.interfaces
                    .keys()
                    .eq(state.configuration().routers[id].interfaces.keys())
            })
    }
    pub fn from_json(input: &str, state: &State) -> Result<Self, String> {
        if input.len() > MAX_DRAFT_BYTES {
            return Err("OSPF draft JSON exceeds 256 KiB".into());
        }
        let draft: Self = serde_json::from_str(input)
            .map_err(|_| "Invalid bounded OSPF draft data".to_owned())?;
        if !draft.same_shape(state) {
            return Err("OSPF draft identities differ from this workspace".into());
        }
        Ok(draft)
    }
    pub fn snapshot(&self, current: &State) -> Result<State, String> {
        if !self.same_shape(current) {
            return Err("OSPF draft identities differ from this workspace".into());
        }
        let mut configuration = Configuration {
            routers: BTreeMap::new(),
        };
        let mut physical = Vec::new();
        // Parse ALL editable text and cross-port uniqueness before the topology clone.
        let mut addresses: BTreeSet<_> = current
            .topology()
            .devices
            .values()
            .filter(|d| !d.forwarding)
            .flat_map(|d| d.interfaces.values())
            .filter_map(|i| i.address.as_ref().map(|a| a.address))
            .collect();
        for (id, r) in &self.routers {
            let process = ProcessConfig {
                enabled: r.enabled,
                process_id: small(&r.process_id, &format!("{id}: process ID"))?,
                router_id: if r.router_id.is_empty() {
                    None
                } else {
                    Some(r.router_id.parse::<Ipv4Addr>().map_err(|_| {
                        format!("{id}: router ID needs dotted-decimal 32-bit notation")
                    })?)
                },
                area: integer(&r.area, &format!("{id}: area (decimal)"))?,
            };
            let mut interfaces = BTreeMap::new();
            for (port, p) in &r.interfaces {
                let endpoint = Endpoint::new(id, port);
                let config = PortConfig {
                    enabled: p.enabled,
                    passive: p.passive,
                    network_type: p.network_type,
                    cost: small(&p.cost, &format!("{id}/{port}: cost"))?,
                    hello_seconds: small(
                        &p.hello_seconds,
                        &format!("{id}/{port}: Hello interval"),
                    )?,
                    dead_seconds: integer(&p.dead_seconds, &format!("{id}/{port}: Dead interval"))?,
                };
                if config.dead_seconds == 0 {
                    return Err(format!("{id}/{port}: Dead interval must be positive"));
                }
                let address = if p.address.is_empty() && p.prefix.is_empty() {
                    None
                } else {
                    let address=p.address.parse::<Ipv4Addr>().map_err(|_|format!("{id}/{port}: enter a valid IPv4 address, or clear both address and prefix"))?;
                    let prefix = u8::try_from(integer(&p.prefix, &format!("{id}/{port}: prefix"))?)
                        .map_err(|_| format!("{id}/{port}: prefix must be 1 through 30"))?;
                    let a = Assignment { address, prefix };
                    if !a.valid() {
                        return Err(format!(
                            "{id}/{port}: only supplied private/documentation addressing with /1–/30 is supported"
                        ));
                    }
                    if !addresses.insert(address) {
                        return Err(format!("{id}/{port}: duplicate virtual interface address"));
                    }
                    Some(a)
                };
                interfaces.insert(port.clone(), config);
                physical.push((endpoint, p.up, address));
            }
            configuration.routers.insert(
                id.clone(),
                RouterConfig {
                    process,
                    interfaces,
                },
            );
        }
        State::validate_parts(current.topology(), &configuration).map_err(|e| e.to_string())?;
        let mut topology = current.topology().clone();
        for (at, up, address) in physical {
            let p = topology
                .devices
                .get_mut(&at.router)
                .unwrap()
                .interfaces
                .get_mut(&at.interface)
                .unwrap();
            p.up = up;
            p.address = address;
        }
        State::new(topology, configuration).map_err(|e| e.to_string())
    }
    pub fn matches(&self, state: &State) -> bool {
        self.snapshot(state).is_ok_and(|s| s == *state)
    }
    pub fn apply(&mut self, initial: &State, state: &mut State) -> Result<(), String> {
        if !same_context(initial, state) {
            let e = "OSPF state differs from the authored graph/roles/host context";
            self.notice = Some(notice(e));
            return Err(e.into());
        }
        let next = match self.snapshot(state) {
            Ok(next) => next,
            Err(e) => {
                self.notice = Some(notice(&e));
                return Err(e);
            }
        };
        if !same_context(initial, &next) {
            let e = "OSPF draft changes the fixed authored context";
            self.notice = Some(notice(e));
            return Err(e.into());
        }
        *state = next;
        self.notice=Some("Settings applied to the virtual snapshot; check current compatibility and remote SPF candidates.".into());
        Ok(())
    }
}
fn hops<'de, D: Deserializer<'de>>(d: D) -> Result<Option<BTreeSet<NextHop>>, D::Error> {
    #[derive(Deserialize)]
    struct H(#[serde(deserialize_with = "hop_list")] Vec<NextHop>);
    fn hop_list<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<NextHop>, D::Error> {
        list::<D, NextHop, 8>(d)
    }
    let values = Option::<H>::deserialize(d)?;
    match values {
        None => Ok(None),
        Some(H(v)) => {
            let set: BTreeSet<_> = v.iter().cloned().collect();
            if set.is_empty() || set.len() != v.len() {
                return Err(D::Error::custom(
                    "Empty or duplicate expected OSPF next hops",
                ));
            }
            Ok(Some(set))
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Compatibility {
        local: Endpoint,
        peer: Endpoint,
    },
    AbsentCompatibility {
        local: Endpoint,
    },
    RemoteRoute {
        #[serde(deserialize_with = "text")]
        router: String,
        destination: Prefix,
        cost: Option<u32>,
        #[serde(default, deserialize_with = "hops")]
        next_hops: Option<BTreeSet<NextHop>>,
    },
    AbsentRemoteRoute {
        #[serde(deserialize_with = "text")]
        router: String,
        destination: Prefix,
    },
    Physical {
        #[serde(deserialize_with = "ospf::deserialize_topology")]
        expected: Topology,
    },
    Process {
        #[serde(deserialize_with = "text")]
        router: String,
        expected: ProcessConfig,
    },
    Port {
        endpoint: Endpoint,
        expected: PortConfig,
    },
    RouterId {
        #[serde(deserialize_with = "text")]
        router: String,
        expected: Ipv4Addr,
    },
    WholeState {
        expected: State,
    },
}
pub fn requirements<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    list::<D, Requirement, MAX_REQUIREMENTS>(d)
}
fn endpoint(initial: &State, e: &Endpoint) -> bool {
    name(&e.router)
        && name(&e.interface)
        && initial
            .configuration()
            .routers
            .get(&e.router)
            .is_some_and(|r| r.interfaces.contains_key(&e.interface))
}
fn attachment(initial: &State, a: &Endpoint, b: &Endpoint) -> bool {
    if !endpoint(initial, a) || !endpoint(initial, b) || a.router == b.router {
        return false;
    }
    let link = &initial.topology().devices[&a.router].interfaces[&a.interface].link;
    link == &initial.topology().devices[&b.router].interfaces[&b.interface].link
        && initial
            .topology()
            .devices
            .values()
            .flat_map(|d| d.interfaces.values())
            .filter(|i| &i.link == link)
            .count()
            == 2
}
pub fn valid_requirements(initial: &State, rs: &[Requirement]) -> bool {
    if !initial.valid()
        || rs.is_empty()
        || rs.len() > MAX_REQUIREMENTS
        || !rs.iter().any(|r| {
            matches!(
                r,
                Requirement::Compatibility { .. } | Requirement::RemoteRoute { .. }
            )
        })
    {
        return false;
    }
    if rs.iter().any(|r| {
        matches!(
            r,
            Requirement::AbsentCompatibility { .. } | Requirement::AbsentRemoteRoute { .. }
        )
    }) && !rs.iter().any(|r| matches!(r, Requirement::Physical { .. }))
    {
        return false;
    }
    let valid = rs.iter().all(|r| match r {
        Requirement::Compatibility { local, peer } => attachment(initial, local, peer),
        Requirement::AbsentCompatibility { local } => endpoint(initial, local),
        Requirement::RemoteRoute {
            router,
            destination,
            cost,
            next_hops,
        } => {
            initial.configuration().routers.contains_key(router)
                && destination.valid()
                && cost.is_none_or(|n| (1..=786420).contains(&n))
                && next_hops.as_ref().is_none_or(|h| {
                    !h.is_empty()
                        && h.len() <= 8
                        && h.iter().all(|h| {
                            name(&h.interface)
                                && endpoint(initial, &h.peer)
                                && attachment(
                                    initial,
                                    &Endpoint::new(router, &h.interface),
                                    &h.peer,
                                )
                                && supported_address(h.via)
                        })
                })
        }
        Requirement::AbsentRemoteRoute {
            router,
            destination,
        } => initial.configuration().routers.contains_key(router) && destination.valid(),
        Requirement::Physical { expected } => {
            State::validate_parts(expected, initial.configuration()).is_ok()
                && expected.same_shape(initial.topology())
                && expected.devices.iter().all(|(id, d)| {
                    d.forwarding == initial.topology().devices[id].forwarding
                        && (d.forwarding || d == &initial.topology().devices[id])
                })
        }
        Requirement::Process { router, expected } => {
            initial.configuration().routers.contains_key(router) && expected.process_id > 0
        }
        Requirement::Port {
            endpoint: e,
            expected,
        } => {
            endpoint(initial, e)
                && expected.cost > 0
                && expected.hello_seconds > 0
                && expected.dead_seconds > 0
        }
        Requirement::RouterId { router, .. } => {
            initial.configuration().routers.contains_key(router)
        }
        Requirement::WholeState { expected } => same_context(initial, expected),
    });
    // Compare only after every item's scalar/context/collection bounds passed.
    valid && rs.iter().enumerate().all(|(i, r)| !rs[..i].contains(r))
}
pub fn describe_condition(c: &ospf::Condition) -> String {
    match c {
        ospf::Condition::Router { router, issue } => format!(
            "{router}: {}",
            match issue {
                ospf::RouterIssue::Disabled => "process disabled",
                ospf::RouterIssue::MissingRouterId => "explicit router ID missing",
                ospf::RouterIssue::ZeroRouterId => "zero is non-running in this explicit-ID model",
                ospf::RouterIssue::DuplicateRouterId { .. } =>
                    "router ID shared by enabled processes",
            }
        ),
        ospf::Condition::PortDisabled { at } => {
            format!("{}/{}: OSPF disabled", at.router, at.interface)
        }
        ospf::Condition::PortDown { at } => {
            format!("{}/{}: interface down", at.router, at.interface)
        }
        ospf::Condition::Passive { at } => format!(
            "{}/{}: passive, without protocol peering",
            at.router, at.interface
        ),
        ospf::Condition::AddressUnavailable { at } => format!(
            "{}/{}: usable numbered address unavailable",
            at.router, at.interface
        ),
        ospf::Condition::UnsupportedActiveBroadcast { .. } => {
            "active broadcast/DR/BDR operation is outside this model".into()
        }
        ospf::Condition::UnsupportedSegment { .. } => {
            "multiaccess active P2P is outside this model".into()
        }
        ospf::Condition::NoPeer => "no direct peer attached".into(),
        ospf::Condition::PeerNotRouter { .. } => "direct endpoint is not an OSPF router".into(),
        ospf::Condition::AreaMismatch { local, peer } => {
            format!("area values differ ({local} and {peer})")
        }
        ospf::Condition::HelloMismatch { local, peer } => {
            format!("Hello values differ ({local}s and {peer}s)")
        }
        ospf::Condition::DeadMismatch { local, peer } => {
            format!("Dead values differ ({local}s and {peer}s)")
        }
        ospf::Condition::PointToPointMaskIgnored { .. } => {
            "P2P mask difference is not a Hello blocker".into()
        }
        ospf::Condition::PointToPointSubnetIgnored => {
            "P2P source-subnet difference is not a Hello blocker".into()
        }
    }
}
fn process_difference(actual: &ProcessConfig, expected: &ProcessConfig) -> &'static str {
    if actual.enabled != expected.enabled {
        "process Enabled setting"
    } else if actual.process_id != expected.process_id {
        "local process number"
    } else if actual.router_id != expected.router_id {
        "router ID"
    } else {
        "area"
    }
}
fn port_difference(actual: &PortConfig, expected: &PortConfig) -> &'static str {
    if actual.enabled != expected.enabled {
        "OSPF Enabled setting"
    } else if actual.passive != expected.passive {
        "Passive setting"
    } else if actual.network_type != expected.network_type {
        "network type"
    } else if actual.cost != expected.cost {
        "outgoing cost"
    } else if actual.hello_seconds != expected.hello_seconds {
        "Hello interval"
    } else {
        "Dead interval"
    }
}
pub fn checks(initial: &State, state: &State, rs: &[Requirement]) -> Vec<(bool, String)> {
    if !same_context(initial, state) || !valid_requirements(initial, rs) {
        return vec![(
            false,
            "OSPF state or outcomes differ from the authored fixed context".into(),
        )];
    }
    let Ok(snapshot) = state.snapshot() else {
        return vec![(false, "Current OSPF snapshot cannot be derived".into())];
    };
    rs.iter().map(|r|{let (pass,message)=match r{
        Requirement::Compatibility{local,peer}=>{let l=snapshot.routers[&local.router].links.iter().find(|l|l.local==*local).unwrap();let pass=l.compatible&&l.peer.as_ref()==Some(peer);let message=if pass{format!("{}/{}: compatible with {}/{} in the current P2P model",local.router,local.interface,peer.router,peer.interface)}else{format!("{}/{}: {}",local.router,local.interface,l.conditions.iter().filter(|c|c.blocking()).map(describe_condition).collect::<Vec<_>>().join("; "))};(pass,message)},
        Requirement::AbsentCompatibility{local}=>{let l=snapshot.routers[&local.router].links.iter().find(|l|l.local==*local).unwrap();(!l.compatible,format!("{}/{}: {}",local.router,local.interface,if l.compatible{"a compatible direct peer remains"}else{"no compatible direct peer, as requested"}))},
        Requirement::RemoteRoute{router,destination,cost,next_hops}=>{
            let route=snapshot.routers[router].route(*destination);let pass=route.is_some_and(|r|cost.is_none_or(|c|r.cost==c)&&next_hops.as_ref().is_none_or(|h|r.next_hops==*h));
            (pass,match route{None=>format!("{router}: no current remote SPF candidate for {}; inspect enabled compatible paths and destination advertisement",destination.text()),Some(r) if cost.is_some_and(|n|n!=r.cost)=>format!("{router}: {} cost is {}; requested {}. Add outgoing link costs and destination stub cost",destination.text(),r.cost,cost.unwrap()),Some(_) if !pass=>format!("{router}: {} has a different equal-cost next-hop set from the requested exits",destination.text()),Some(r)=>format!("{router}: {} current remote SPF candidate matches (cost {}, {} equal-cost exits)",destination.text(),r.cost,r.next_hops.len())})
        },
        Requirement::AbsentRemoteRoute{router,destination}=>{let absent=snapshot.routers[router].route(*destination).is_none();(absent,format!("{router}: {} remote SPF candidate {}",destination.text(),if absent{"absent as requested"}else{"is still present"}))},
        Requirement::Physical{expected}=>{let pass=state.topology()==expected;let first=expected.devices.iter().find_map(|(r,d)|d.interfaces.iter().find_map(|(p,i)|(state.topology().devices[r].interfaces[p]!=*i).then(||format!("{r}/{p}"))));(pass,if pass{"Requested interface addresses and availability match".into()}else{format!("{}: address or interface availability differs from the task's physical requirement",first.unwrap_or_else(||"Physical context".into()))})},
        Requirement::Process{router,expected}=>{ let actual=&state.configuration().routers[router].process; let pass=actual==expected; (pass,if pass {format!("{router}: requested process enable/ID/area/router-ID settings")} else {format!("{router}: {} differs from the task's required settings. Check that field and retry.",process_difference(actual,expected))}) },
        Requirement::Port{endpoint,expected}=>{let actual=&state.configuration().routers[&endpoint.router].interfaces[&endpoint.interface];let pass=actual==expected;(pass,if pass{format!("{}/{}: requested OSPF enable/passive/type/cost/Hello/Dead settings",endpoint.router,endpoint.interface)}else{format!("{}/{}: {} differs from the task's required settings. Check that field and retry.",endpoint.router,endpoint.interface,port_difference(actual,expected))})},
        Requirement::RouterId{router,expected}=>(state.configuration().routers[router].process.router_id==Some(*expected),format!("{router}: requested opaque router identity {expected}")),
        Requirement::WholeState{expected}=>(state==expected,"All explicitly requested final settings, including preserved context".into()),
    };(pass,notice(&message))}).collect()
}
pub fn execute(initial: &State, state: &mut State, input: &str) -> Result<String, String> {
    if !same_context(initial, state) {
        return Err("OSPF state differs from the authored graph/host context".into());
    }
    let json = input.strip_prefix("ospf ").ok_or_else(|| {
        "Use native OSPF settings; this data adapter is not IOS or a shell".to_owned()
    })?;
    let action = Action::from_json(json).map_err(|e| e.to_string())?;
    state
        .apply_against(initial, &action)
        .map_err(|e| e.to_string())?;
    Ok(
        "Typed settings applied; current compatibility and SPF candidates are derived afresh."
            .into(),
    )
}
