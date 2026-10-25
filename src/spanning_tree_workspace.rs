//! Bounded native drafts and fresh ideal-P2P spanning-tree outcomes. No CLI/protocol.
use crate::spanning_tree::{
    self as core, Configuration, Endpoint, PortConfig, Snapshot, State, Topology,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};
pub const MAX_REQUIREMENTS: usize = 32;
pub const MAX_DRAFT_BYTES: usize = 262_144;
pub const MAX_GOAL_BYTES: usize = 524_288;
pub const MAX_TEXT: usize = 64;
pub const PREFIX: &str = "spanning-tree ";
pub const HELP: &str = "Edit the supplied virtual bridge/port priorities, positive costs and physical availability, apply the draft and check current work. Results are an ideal point-to-point steady-state spanning tree, not BPDUs, measured convergence, packet delivery, PVST, PortFast or guards. This typed authoring adapter is not IOS or a shell.";
pub fn notice(s: &str) -> String {
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].to_owned()
}
pub(crate) fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
pub(crate) fn bounded(s: &str, n: usize) -> bool {
    s.len() <= n && !s.chars().any(char::is_control)
}
pub(crate) fn within<T: Serialize>(v: &T, cap: usize) -> bool {
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
                .ok_or_else(|| std::io::Error::other("Spanning-tree app bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, cap }, v).is_ok()
}
pub(crate) struct Text<const N: usize>(pub String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V<const N: usize>;
        impl<const N: usize> Visitor<'_> for V<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most{N} bounded UTF8 bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if bounded(s, N) {
                    Ok(Text(s.into()))
                } else {
                    Err(E::custom("Spanning-tree text/control bound"))
                }
            }
        }
        d.deserialize_str(V::<N>)
    }
}
fn text<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Text::<MAX_TEXT>::deserialize(d)?.0)
}
fn id<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let Text(s) = Text::<32>::deserialize(d)?;
    if name(&s) {
        Ok(s)
    } else {
        Err(D::Error::custom("Invalid spanning-tree identity"))
    }
}
pub(crate) fn optional_notice<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<Text<512>>::deserialize(d)?.map(|s| s.0))
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for M<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} distinct bounded keys")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Text(k)) = a.next_key::<Text<32>>()? {
                if !name(&k) || out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom("Spanning-tree key/count/duplicate bound"));
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
    struct Next<V> {
        allowed: bool,
        kind: PhantomData<V>,
    }
    impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for Next<V> {
        type Value = V;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<V, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("Spanning-tree item count bound"));
            }
            V::deserialize(d)
        }
    }
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for L<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next::<V> {
                allowed: out.len() < N,
                kind: PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(L::<V, N>(PhantomData))
}
fn bridges<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    map::<D, V, 12>(d)
}
fn ports<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    map::<D, V, 8>(d)
}
fn links<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    map::<D, V, 48>(d)
}
fn names<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(list::<D, Text<32>, 12>(d)?
        .into_iter()
        .map(|s| s.0)
        .collect())
}
fn path_names<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<String>>, D::Error> {
    #[derive(Deserialize)]
    struct Names(#[serde(deserialize_with = "list::<_,Text<32>,11>")] Vec<Text<32>>);
    Ok(Option::<Names>::deserialize(d)?.map(|n| n.0.into_iter().map(|s| s.0).collect()))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeAvailability {
    pub online: bool,
    #[serde(deserialize_with = "ports")]
    pub ports: BTreeMap<String, bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Availability {
    #[serde(deserialize_with = "bridges")]
    pub bridges: BTreeMap<String, BridgeAvailability>,
    #[serde(deserialize_with = "links")]
    pub links: BTreeMap<String, bool>,
}
impl Availability {
    pub fn new(state: &State) -> Result<Self, String> {
        state.validate().map_err(|e| notice(&e.message))?;
        Ok(Self {
            bridges: state
                .configuration()
                .bridges
                .iter()
                .map(|(k, b)| {
                    (
                        k.clone(),
                        BridgeAvailability {
                            online: b.online,
                            ports: b.ports.iter().map(|(p, c)| (p.clone(), c.up)).collect(),
                        },
                    )
                })
                .collect(),
            links: state.configuration().links.clone(),
        })
    }
    fn valid_for(&self, t: &Topology) -> bool {
        self.bridges.len() <= 12
            && self.links.len() <= 48
            && self
                .bridges
                .iter()
                .all(|(k, b)| name(k) && b.ports.len() <= 8 && b.ports.keys().all(|p| name(p)))
            && self.links.keys().all(|k| name(k))
            && self.bridges.keys().eq(t.bridges.keys())
            && self.links.keys().eq(t.links.keys())
            && self
                .bridges
                .iter()
                .all(|(k, b)| b.ports.keys().eq(t.bridges[k].ports.keys()))
    }
    fn matches(&self, s: &State) -> bool {
        self.bridges.iter().all(|(k, b)| {
            b.online == s.configuration().bridges[k].online
                && b.ports
                    .iter()
                    .all(|(p, up)| *up == s.configuration().bridges[k].ports[p].up)
        }) && self.links == s.configuration().links
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StableRole {
    Root,
    Designated,
    Alternate,
    Disabled,
}
impl StableRole {
    fn matches(self, role: core::Role) -> bool {
        matches!(
            (self, role),
            (Self::Root, core::Role::Root)
                | (Self::Designated, core::Role::Designated)
                | (Self::Alternate, core::Role::Alternate)
                | (Self::Disabled, core::Role::Disabled)
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Root => "Root",
            Self::Designated => "Designated",
            Self::Alternate => "Alternate",
            Self::Disabled => "Disabled",
        }
    }
}
fn role_label(r: core::Role) -> &'static str {
    match r {
        core::Role::Root => "Root",
        core::Role::Designated => "Designated",
        core::Role::Alternate => "Alternate",
        core::Role::Disabled => "Disabled",
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Availability {
        expected: Availability,
    },
    Connected {
        #[serde(deserialize_with = "names")]
        bridges: Vec<String>,
    },
    Root {
        #[serde(deserialize_with = "id")]
        bridge: String,
        #[serde(deserialize_with = "id")]
        root: String,
    },
    RootPort {
        endpoint: Endpoint,
    },
    PortRole {
        endpoint: Endpoint,
        role: StableRole,
    },
    RootCost {
        #[serde(deserialize_with = "id")]
        bridge: String,
        value: u32,
    },
    ForwardingPath {
        #[serde(deserialize_with = "id")]
        from: String,
        #[serde(deserialize_with = "id")]
        to: String,
        #[serde(default, deserialize_with = "path_names")]
        expected_links: Option<Vec<String>>,
    },
    BridgePriority {
        #[serde(deserialize_with = "id")]
        bridge: String,
        value: u16,
    },
    PortSettings {
        endpoint: Endpoint,
        expected: PortConfig,
    },
    Configuration {
        expected: Configuration,
    },
}
pub fn requirements<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    list::<D, Requirement, MAX_REQUIREMENTS>(d)
}
fn endpoint(t: &Topology, e: &Endpoint) -> bool {
    name(&e.bridge)
        && name(&e.port)
        && t.bridges
            .get(&e.bridge)
            .is_some_and(|b| b.ports.contains_key(&e.port))
}
fn bridge(t: &Topology, b: &str) -> bool {
    name(b) && t.bridges.contains_key(b)
}
fn priority(v: u16) -> bool {
    v <= 61440 && v.is_multiple_of(4096)
}
fn port_valid(p: &PortConfig) -> bool {
    p.priority.is_multiple_of(16) && (1..=core::MAX_COST).contains(&p.cost)
}
fn path_valid(t: &Topology, from: &str, to: &str, path: &[String]) -> bool {
    if path.is_empty()
        || path.len() > 11
        || path.iter().any(|p| !name(p) || !t.links.contains_key(p))
    {
        return false;
    }
    let mut current = from;
    let mut seen = BTreeSet::from([from]);
    for p in path {
        let l = &t.links[p];
        current = if l.a.bridge == current {
            &l.b.bridge
        } else if l.b.bridge == current {
            &l.a.bridge
        } else {
            return false;
        };
        if !seen.insert(current) {
            return false;
        }
    }
    current == to
}
fn positive(r: &Requirement) -> bool {
    matches!(
        r,
        Requirement::Root { .. }
            | Requirement::RootPort { .. }
            | Requirement::ForwardingPath { .. }
    ) || matches!(r,Requirement::PortRole{role,..}if *role!=StableRole::Disabled)
}
pub fn valid_requirements(initial: &State, rs: &[Requirement]) -> bool {
    if initial.validate().is_err() || rs.is_empty() || rs.len() > MAX_REQUIREMENTS {
        return false;
    }
    let t = initial.topology();
    // All scalar/shape bounds precede derived sets, duplicate comparisons or clones.
    if !rs.iter().all(|r| match r {
        Requirement::Availability { expected } => expected.valid_for(t),
        Requirement::Connected { bridges } => {
            bridges.len() >= 2
                && bridges.len() <= 12
                && bridges.iter().all(|b| bridge(t, b))
                && bridges
                    .iter()
                    .enumerate()
                    .all(|(i, b)| !bridges[..i].contains(b))
        }
        Requirement::Root { bridge: b, root } => bridge(t, b) && bridge(t, root),
        Requirement::RootPort { endpoint: e } | Requirement::PortRole { endpoint: e, .. } => {
            endpoint(t, e)
        }
        Requirement::RootCost { bridge: b, value } => bridge(t, b) && *value <= 2_200_000_000,
        Requirement::ForwardingPath {
            from,
            to,
            expected_links,
        } => {
            bridge(t, from)
                && bridge(t, to)
                && from != to
                && expected_links
                    .as_ref()
                    .is_none_or(|p| path_valid(t, from, to, p))
        }
        Requirement::BridgePriority { bridge: b, value } => bridge(t, b) && priority(*value),
        Requirement::PortSettings {
            endpoint: e,
            expected,
        } => endpoint(t, e) && port_valid(expected),
        Requirement::Configuration { expected } => expected.validate_for(t).is_ok(),
    }) {
        return false;
    }
    if rs
        .iter()
        .filter(|r| matches!(r, Requirement::Availability { .. }))
        .count()
        != 1
        || !rs.iter().any(positive)
    {
        return false;
    }
    let groups: Vec<_> = rs
        .iter()
        .filter_map(|r| {
            if let Requirement::Connected { bridges } = r {
                Some(bridges)
            } else {
                None
            }
        })
        .collect();
    if groups.is_empty() {
        return false;
    }
    let covered = |b: &str| groups.iter().any(|g| g.iter().any(|x| x == b));
    let together = |a: &str, b: &str| {
        groups
            .iter()
            .any(|g| g.iter().any(|x| x == a) && g.iter().any(|x| x == b))
    };
    if !rs.iter().all(|r| match r {
        Requirement::Root { bridge, root } => together(bridge, root),
        Requirement::RootPort { endpoint } | Requirement::PortRole { endpoint, .. } => {
            covered(&endpoint.bridge)
        }
        Requirement::RootCost { bridge, .. } => covered(bridge),
        Requirement::ForwardingPath { from, to, .. } => together(from, to),
        _ => true,
    }) {
        return false;
    }
    #[derive(Serialize)]
    struct Goal<'a> {
        initial: &'a State,
        requirements: &'a [Requirement],
    }
    rs.iter().enumerate().all(|(i, r)| !rs[..i].contains(r))
        && within(
            &Goal {
                initial,
                requirements: rs,
            },
            MAX_GOAL_BYTES,
        )
}
pub fn same_context(initial: &State, current: &State) -> bool {
    initial.validate().is_ok() && current.validate_context(initial.topology()).is_ok()
}
fn online_component(snapshot: &Snapshot, names: &[String]) -> bool {
    let root = snapshot
        .bridge(&names[0])
        .and_then(|b| b.root_bridge.as_ref());
    root.is_some()
        && names
            .iter()
            .all(|n| snapshot.bridge(n).and_then(|b| b.root_bridge.as_ref()) == root)
}
fn unavailable(u: Option<core::Unavailable>) -> &'static str {
    match u {
        Some(core::Unavailable::BridgeOffline) => "bridge is offline",
        Some(core::Unavailable::LocalPortDown) => "local port is down",
        Some(core::Unavailable::NoLink) => "no supplied link",
        Some(core::Unavailable::LinkDown) => "physical link is down",
        Some(core::Unavailable::PeerBridgeOffline) => "peer bridge is offline",
        Some(core::Unavailable::PeerPortDown) => "peer port is down",
        None => "operational point-to-point attachment",
    }
}
pub fn checks(initial: &State, current: &State, rs: &[Requirement]) -> Vec<(bool, String)> {
    if !same_context(initial, current) || !valid_requirements(initial, rs) {
        return vec![(
            false,
            "Spanning-tree state or outcomes differ from the supplied context".into(),
        )];
    }
    let Ok(snapshot) = current.snapshot() else {
        return vec![(
            false,
            "Current ideal spanning-tree result cannot be derived".into(),
        )];
    };
    rs.iter().map(|r|{let(p,m)=match r{
 Requirement::Availability{expected}=>{let pass=expected.matches(current);let first=expected.bridges.iter().find_map(|(b,e)|{let c=&current.configuration().bridges[b];if e.online!=c.online{Some(format!("{b}: bridge availability"))}else{e.ports.iter().find_map(|(p,up)|(*up!=c.ports[p].up).then(||format!("{b}/{p}: port availability")))}}).or_else(||expected.links.iter().find_map(|(l,up)|(*up!=current.configuration().links[l]).then(||format!("{l}: link availability"))));(pass,if pass{"Requested physical availability is preserved".into()}else{format!("{} differs from the task. Repair availability rather than isolating a competing bridge",first.unwrap_or_else(||"Physical availability".into()))})},
 Requirement::Connected{bridges}=>{let pass=online_component(&snapshot,bridges);(pass,if pass{"Required bridges are online in the same current operational component".into()}else{"Required bridges are offline or disconnected; inspect both ports and link availability".into()})},
 Requirement::Root{bridge,root}=>{let b=&snapshot.bridges()[bridge];let pass=b.root_bridge.as_ref()==Some(root);(pass,match &b.root_bridge{Some(actual)=>format!("{bridge}: current root is {actual}; {}",if pass{"requested root matches"}else{"compare bridge priority, system-ID extension and MAC"}),None=>format!("{bridge}: offline; no current root")})},
 Requirement::RootPort{endpoint}=>{let b=&snapshot.bridges()[&endpoint.bridge];let p=&b.ports[&endpoint.port];let pass=b.root_port.as_ref()==Some(&endpoint.port)&&p.unavailable.is_none();(pass,format!("{}/{}: {}; {}",endpoint.bridge,endpoint.port,if pass{"requested root port"}else{"not the current root port"},unavailable(p.unavailable)))},
 Requirement::PortRole{endpoint,role}=>{let p=&snapshot.bridges()[&endpoint.bridge].ports[&endpoint.port];let pass=role.matches(p.role)&&(*role==StableRole::Disabled||p.unavailable.is_none());(pass,format!("{}/{}: current {} role; {}",endpoint.bridge,endpoint.port,role_label(p.role),if pass{"requested role matches"}else{unavailable(p.unavailable)}))},
 Requirement::RootCost{bridge,value}=>{let cost=snapshot.bridges()[bridge].root_cost;(cost==Some(*value),match cost{Some(c)=>format!("{bridge}: current root cost is {c}; add local rootward port costs"),None=>format!("{bridge}: offline; no root cost")})},
 Requirement::ForwardingPath{from,to,expected_links}=>{let path=snapshot.forwarding_path(from,to).ok().flatten();let pass=path.as_ref().is_some_and(|p|!p.is_empty()&&expected_links.as_ref().is_none_or(|e|p==e));(pass,if pass{format!("{from} to {to}: current forwarding-tree path matches")}else if path.is_none(){format!("{from} to {to}: no current forwarding-tree path; inspect component availability")}else{format!("{from} to {to}: different current tree path; compare rootward costs and tie-break identities")})},
 Requirement::BridgePriority{bridge,value}=>{let pass=current.configuration().bridges[bridge].priority==*value;(pass,format!("{bridge}: {}",if pass{"explicitly requested bridge priority matches"}else{"bridge priority differs from the stated setting"}))},
 Requirement::PortSettings{endpoint,expected}=>{let pass=current.configuration().bridges[&endpoint.bridge].ports[&endpoint.port]==*expected;(pass,format!("{}/{}: {}",endpoint.bridge,endpoint.port,if pass{"explicitly requested port settings match"}else{"port priority, cost or availability differs from the stated settings"}))},
 Requirement::Configuration{expected}=>{let pass=current.configuration()==expected;(pass,if pass{"All explicitly requested configuration settings match".into()}else{"An explicitly preserved configuration setting differs; inspect unchanged bridges, ports and links".into()})},
 };(p,notice(&m))}).collect()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortDraft {
    #[serde(deserialize_with = "text")]
    pub priority: String,
    #[serde(deserialize_with = "text")]
    pub cost: String,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeDraft {
    #[serde(deserialize_with = "text")]
    pub priority: String,
    pub online: bool,
    #[serde(deserialize_with = "ports")]
    pub ports: BTreeMap<String, PortDraft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub version: u8,
    pub context: Topology,
    pub bridges: BTreeMap<String, BridgeDraft>,
    pub links: BTreeMap<String, bool>,
    pub selected_bridge: String,
    pub selected_port: Option<String>,
    pub selected_link: Option<String>,
    pub notice: Option<String>,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u8,
            context: Topology,
            #[serde(deserialize_with = "bridges")]
            bridges: BTreeMap<String, BridgeDraft>,
            #[serde(deserialize_with = "links")]
            links: BTreeMap<String, bool>,
            #[serde(deserialize_with = "id")]
            selected_bridge: String,
            selected_port: Option<Text<32>>,
            selected_link: Option<Text<32>>,
            #[serde(default, deserialize_with = "optional_notice")]
            notice: Option<String>,
        }
        let r = Raw::deserialize(d)
            .map_err(|_| D::Error::custom("Invalid bounded spanning-tree draft"))?;
        let v = Self {
            version: r.version,
            context: r.context,
            bridges: r.bridges,
            links: r.links,
            selected_bridge: r.selected_bridge,
            selected_port: r.selected_port.map(|s| s.0),
            selected_link: r.selected_link.map(|s| s.0),
            notice: r.notice,
        };
        if !v.valid() {
            return Err(D::Error::custom(
                "Foreign shape or bounded spanning-tree draft",
            ));
        }
        Ok(v)
    }
}
fn configured(context: &Topology, configuration: &Configuration) -> Result<State, String> {
    configuration
        .validate_for(context)
        .map_err(|e| notice(&e.message))?;
    #[derive(Serialize)]
    struct Proposed<'a> {
        topology: &'a Topology,
        configuration: &'a Configuration,
    }
    if !within(
        &Proposed {
            topology: context,
            configuration,
        },
        core::MAX_STATE_BYTES,
    ) {
        return Err("Proposed spanning-tree configuration exceeds the core state bound".into());
    }
    State::new(context.clone(), configuration.clone()).map_err(|e| notice(&e.message))
}
fn number(s: &str, label: &str) -> Result<u32, String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(notice(&format!(
            "{label}: enter a whole-number decimal value"
        )));
    }
    s.parse()
        .map_err(|_| notice(&format!("{label}: decimal value exceeds its bound")))
}
impl Draft {
    pub fn new(s: &State) -> Result<Self, String> {
        s.validate().map_err(|e| notice(&e.message))?;
        let first = s.topology().bridges.keys().next().unwrap();
        let v = Self {
            version: 1,
            context: s.topology().clone(),
            bridges: s
                .configuration()
                .bridges
                .iter()
                .map(|(k, b)| {
                    (
                        k.clone(),
                        BridgeDraft {
                            priority: b.priority.to_string(),
                            online: b.online,
                            ports: b
                                .ports
                                .iter()
                                .map(|(p, c)| {
                                    (
                                        p.clone(),
                                        PortDraft {
                                            priority: c.priority.to_string(),
                                            cost: c.cost.to_string(),
                                            up: c.up,
                                        },
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
            links: s.configuration().links.clone(),
            selected_bridge: first.clone(),
            selected_port: s.topology().bridges[first].ports.keys().next().cloned(),
            selected_link: s.topology().links.keys().next().cloned(),
            notice: None,
        };
        if !v.valid() {
            return Err("Spanning-tree draft aggregate bound".into());
        }
        Ok(v)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.context.validate().is_ok()
            && self.bridges.len() <= 12
            && self.links.len() <= 48
            && self.bridges.iter().all(|(k, b)| {
                name(k)
                    && bounded(&b.priority, MAX_TEXT)
                    && b.ports.len() <= 8
                    && b.ports.iter().all(|(p, c)| {
                        name(p) && bounded(&c.priority, MAX_TEXT) && bounded(&c.cost, MAX_TEXT)
                    })
            })
            && self.links.keys().all(|k| name(k))
            && self.bridges.keys().eq(self.context.bridges.keys())
            && self.links.keys().eq(self.context.links.keys())
            && self
                .bridges
                .iter()
                .all(|(k, b)| b.ports.keys().eq(self.context.bridges[k].ports.keys()))
            && name(&self.selected_bridge)
            && self
                .bridges
                .get(&self.selected_bridge)
                .is_some_and(|b| match &self.selected_port {
                    None => b.ports.is_empty(),
                    Some(p) => name(p) && b.ports.contains_key(p),
                })
            && self
                .selected_link
                .as_ref()
                .is_none_or(|l| name(l) && self.links.contains_key(l))
            && (self.selected_link.is_some() || self.links.is_empty())
            && self.notice.as_ref().is_none_or(|n| bounded(n, 512))
            && within(self, MAX_DRAFT_BYTES)
    }
    pub fn same_shape(&self, s: &State) -> bool {
        self.valid() && s.validate().is_ok() && &self.context == s.topology()
    }
    pub fn valid_for(&self, s: &State) -> bool {
        self.same_shape(s)
    }
    pub fn snapshot(&self, s: &State) -> Result<State, String> {
        if !self.same_shape(s) {
            return Err("Spanning-tree draft context differs from the supplied graph".into());
        }
        let mut bs = BTreeMap::new();
        for (k, b) in &self.bridges {
            let priority = u16::try_from(number(&b.priority, &format!("{k}: bridge priority"))?)
                .map_err(|_| format!("{k}: bridge priority exceeds61440"))?;
            let mut ps = BTreeMap::new();
            for (p, c) in &b.ports {
                ps.insert(
                    p.clone(),
                    PortConfig {
                        priority: u8::try_from(number(
                            &c.priority,
                            &format!("{k}/{p}: port priority"),
                        )?)
                        .map_err(|_| format!("{k}/{p}: port priority exceeds240"))?,
                        cost: number(&c.cost, &format!("{k}/{p}: path cost"))?,
                        up: c.up,
                    },
                );
            }
            bs.insert(
                k.clone(),
                core::BridgeConfig {
                    priority,
                    online: b.online,
                    ports: ps,
                },
            );
        }
        let c = Configuration {
            bridges: bs,
            links: self.links.clone(),
        };
        configured(&self.context, &c)
    }
    pub fn matches(&self, s: &State) -> bool {
        self.snapshot(s).is_ok_and(|candidate| candidate == *s)
    }
    pub fn apply(&mut self, initial: &State, s: &mut State) -> Result<(), String> {
        if !same_context(initial, s) {
            let e = "Current spanning-tree graph differs from the authored context";
            self.notice = Some(e.into());
            return Err(e.into());
        }
        let candidate = match self.snapshot(s) {
            Ok(v) => v,
            Err(e) => {
                self.notice = Some(notice(&e));
                return Err(e);
            }
        };
        if !same_context(initial, &candidate) {
            return Err("Spanning-tree draft changes the fixed context".into());
        }
        *s = candidate;
        self.notice=Some("Configuration applied; results are a new ideal steady-state calculation, not measured convergence.".into());
        Ok(())
    }
    pub fn fill(&mut self, s: &State, c: &Configuration) -> Result<(), String> {
        if !self.valid_for(s) {
            return Err("Spanning-tree draft differs from supplied context".into());
        }
        let candidate = configured(s.topology(), c)?;
        let mut next = Self::new(&candidate)?;
        next.selected_bridge = self.selected_bridge.clone();
        next.selected_port = self.selected_port.clone();
        next.selected_link = self.selected_link.clone();
        next.notice =
            Some("Example settings inserted into the draft only; Apply them explicitly.".into());
        if !next.valid() {
            return Err("Spanning-tree filled draft exceeds app bounds".into());
        }
        *self = next;
        Ok(())
    }
}
pub fn execute(initial: &State, current: &mut State, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + core::MAX_ACTION_BYTES {
        return Err("Spanning-tree typed input exceeds64KiB plus fixed prefix".into());
    }
    if !same_context(initial, current) {
        return Err("Spanning-tree context differs; prior state retained".into());
    }
    let json = input.strip_prefix(PREFIX).ok_or_else(|| HELP.to_owned())?;
    let action = core::Action::from_json(json.as_bytes()).map_err(|e| notice(&e.message))?;
    current.apply(&action).map_err(|e| notice(&e.message))?;
    Ok("Typed configuration applied; current ideal P2P tree is derived afresh.".into())
}
