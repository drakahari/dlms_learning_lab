//! Bounded native-workspace adapter for the pure L2 LACP model. No host I/O.
use crate::etherchannel as core;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::Write,
    marker::PhantomData,
};
pub const MAX_DEFINITION: usize = 512 * 1024;
pub const MAX_DRAFT: usize = 64 * 1024;
pub const MAX_SCENARIO: usize = 1024 * 1024;
pub const MAX_ATTEMPT: usize = 2 * 1024 * 1024;
pub const PREFIX: &str = "sim-lacp ";
pub const HELP: &str = "Native simulated L2 LACP configuration only. The sim-lacp JSON bridge is an internal authoring helper, not IOS or a host command. No packets, timers or routed EtherChannel are modeled.";
pub fn notice(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].into()
}
pub(crate) fn text(s: &str, n: usize) -> bool {
    s.len() <= n && !s.contains('\0')
}
pub(crate) fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
}
struct Count {
    used: usize,
    cap: usize,
}
impl Write for Count {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.used = self
            .used
            .checked_add(b.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| std::io::Error::other("LACP aggregate bound"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
std::thread_local! {
    // In-memory development observation only: no data, timestamps or progress.
    static JSON_ADMISSION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
/// Actual LACP JSON-sizing passes on this thread, including nested admission.
/// This diagnostic is never a grade, saved field, telemetry or completion input.
pub fn json_admission_calls() -> usize {
    JSON_ADMISSION_CALLS.with(std::cell::Cell::get)
}
pub(crate) fn size<T: Serialize + ?Sized>(v: &T, cap: usize) -> Option<usize> {
    JSON_ADMISSION_CALLS.with(|count| count.set(count.get().saturating_add(1)));
    let mut c = Count { used: 0, cap };
    serde_json::to_writer(&mut c, v).ok()?;
    Some(c.used)
}
pub(crate) fn within<T: Serialize + ?Sized>(v: &T, cap: usize) -> bool {
    size(v, cap).is_some()
}
pub(crate) fn string<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct S<const N: usize>;
    impl<const N: usize> Visitor<'_> for S<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "non-NUL text at most{N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if text(s, N) {
                Ok(s.into())
            } else {
                Err(E::custom("LACP text bound before copy"))
            }
        }
    }
    d.deserialize_str(S::<N>)
}
struct Item<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Item<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom(
                "LACP collection frontier before next typed body",
            ));
        }
        T::deserialize(d)
    }
}
pub(crate) fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct L<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for L<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} entries")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(x) = a.next_element_seed(Item {
                allowed: v.len() < N,
                marker: PhantomData,
            })? {
                v.push(x);
            }
            Ok(v)
        }
    }
    d.deserialize_seq(L::<T, N>(PhantomData))
}
fn map<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, T>, D::Error> {
    struct M<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for M<T, N> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} uniquely named entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Key(#[serde(deserialize_with = "string::<_,32>")] String);
            let mut v = BTreeMap::new();
            while let Some(Key(k)) = a.next_key()? {
                if v.len() >= N || !id(&k) || v.contains_key(&k) {
                    return Err(A::Error::custom("LACP map frontier/key before value"));
                }
                v.insert(k, a.next_value()?);
            }
            Ok(v)
        }
    }
    d.deserialize_map(M::<T, N>(PhantomData))
}
fn names<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "string::<_,32>")] String);
    Ok(list::<D, Name, N>(d)?.into_iter().map(|n| n.0).collect())
}
fn unique<T: PartialEq>(v: &[T]) -> bool {
    v.iter().enumerate().all(|(i, x)| !v[..i].contains(x))
}
fn pair(a: &core::GroupRef, b: &core::GroupRef, c: &core::GroupRef, d: &core::GroupRef) -> bool {
    (a == c && b == d) || (a == d && b == c)
}
fn group(t: &core::Topology, g: &core::GroupRef) -> bool {
    id(&g.device)
        && id(&g.group)
        && t.devices
            .get(&g.device)
            .is_some_and(|d| d.groups.contains_key(&g.group))
}
fn endpoint(t: &core::Topology, p: &core::Endpoint) -> bool {
    id(&p.device)
        && id(&p.port)
        && t.devices
            .get(&p.device)
            .is_some_and(|d| d.ports.contains_key(&p.port))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Source {
    Current {},
    Fault {
        #[serde(deserialize_with = "string::<_,32>")]
        id: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedQuery {
    #[serde(deserialize_with = "string::<_,32>")]
    pub id: String,
    pub query: core::ServiceQuery,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedFault {
    #[serde(deserialize_with = "string::<_,32>")]
    pub id: String,
    pub condition: core::FaultCondition,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    BundlePair {
        source: Source,
        a: core::GroupRef,
        b: core::GroupRef,
        minimum_members: u8,
        #[serde(deserialize_with = "names::<_,32>")]
        cables: Vec<String>,
    },
    VlanService {
        source: Source,
        #[serde(deserialize_with = "string::<_,32>")]
        query: String,
    },
    RetiredMembership {
        group: core::GroupRef,
    },
    PreservePorts {
        #[serde(deserialize_with = "list::<_,core::Endpoint,64>")]
        ports: Vec<core::Endpoint>,
    },
    PreserveGroups {
        #[serde(deserialize_with = "list::<_,core::GroupRef,16>")]
        groups: Vec<core::GroupRef>,
    },
    PreserveVlans {
        #[serde(deserialize_with = "names::<_,8>")]
        devices: Vec<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Definition {
    pub initial: core::State,
    pub queries: Vec<NamedQuery>,
    pub faults: Vec<NamedFault>,
    pub requirements: Vec<Requirement>,
}
impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            initial: core::State,
            #[serde(deserialize_with = "list::<_,NamedQuery,8>")]
            queries: Vec<NamedQuery>,
            #[serde(deserialize_with = "list::<_,NamedFault,4>")]
            faults: Vec<NamedFault>,
            #[serde(deserialize_with = "list::<_,Requirement,32>")]
            requirements: Vec<Requirement>,
        }
        let r = Raw::deserialize(d)?;
        let v = Self {
            initial: r.initial,
            queries: r.queries,
            faults: r.faults,
            requirements: r.requirements,
        };
        v.validate().map_err(D::Error::custom)?;
        Ok(v)
    }
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("LACP Definition raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded LACP Definition".into())
    }
    pub fn row_count(&self) -> usize {
        1 + self.requirements.len()
    }
    pub fn same_assignment(&self, s: &core::State) -> bool {
        self.initial.topology() == s.topology() && self.initial.environment() == s.environment()
    }
    fn source_index(&self, s: &Source) -> Option<usize> {
        match s {
            Source::Current {} => Some(0),
            Source::Fault { id } => self.faults.iter().position(|f| &f.id == id).map(|n| n + 1),
        }
    }
    fn query_index(&self, id: &str) -> Option<usize> {
        self.queries.iter().position(|q| q.id == id)
    }
    fn evaluation(&self) -> core::Evaluation {
        core::Evaluation {
            queries: self.queries.iter().map(|q| q.query.clone()).collect(),
            faults: self.faults.iter().map(|f| f.condition.clone()).collect(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.queries.is_empty()
            || self.queries.len() > 8
            || self.faults.len() > 4
            || self.requirements.is_empty()
            || self.requirements.len() > 32
        {
            return Err("LACP Definition count frontier".into());
        }
        self.initial.validate().map_err(|e| e.to_string())?;
        if !self.queries.iter().all(|q| id(&q.id))
            || !self.faults.iter().all(|f| id(&f.id))
            || !unique(&self.queries.iter().map(|q| &q.id).collect::<Vec<_>>())
            || !unique(&self.faults.iter().map(|f| &f.id).collect::<Vec<_>>())
            || !unique(&self.requirements)
        {
            return Err("LACP duplicate/invalid requirement or named context".into());
        }
        let t = self.initial.topology();
        if self.queries.iter().any(|q| {
            !group(t, &q.query.a)
                || !group(t, &q.query.b)
                || q.query.a.device == q.query.b.device
                || !(1..=4094).contains(&q.query.vlan)
                || !(1..=8).contains(&q.query.minimum_members)
        }) || self.faults.iter().any(|f| {
            f.condition.down.is_empty()
                || f.condition.down.len() > 4
                || f.condition
                    .down
                    .iter()
                    .any(|c| !id(c) || !t.cables.contains_key(c))
        }) {
            return Err(
                "LACP query/fault scalar, supplied context or typed frontier differs".into(),
            );
        }
        let mut current_positive = false;
        for r in &self.requirements {
            match r {
                Requirement::BundlePair {
                    source,
                    a,
                    b,
                    minimum_members,
                    cables,
                } => {
                    if self.source_index(source).is_none()
                        || !group(t, a)
                        || !group(t, b)
                        || a.device == b.device
                        || !(2..=8).contains(minimum_members)
                        || cables.len() < usize::from(*minimum_members)
                        || cables.len() > 32
                        || !unique(cables)
                        || cables.iter().any(|c| {
                            !id(c)
                                || !t.cables.get(c).is_some_and(|c| {
                                    (c.a.device == a.device && c.b.device == b.device)
                                        || (c.a.device == b.device && c.b.device == a.device)
                                })
                        })
                    {
                        return Err("Invalid LACP intended pair/member domain/source".into());
                    }
                    let mut vlans = BTreeSet::new();
                    for row in &self.requirements {
                        if let Requirement::VlanService { source: at, query } = row
                            && at == source
                            && let Some(n) = self.query_index(query)
                        {
                            let q = &self.queries[n].query;
                            if pair(a, b, &q.a, &q.b)
                                && q.minimum_members >= 2
                                && q.minimum_members <= *minimum_members
                            {
                                vlans.insert(q.vlan);
                            }
                        }
                    }
                    // This initial adapter is deliberately the reviewed two-data-VLAN trunk task subset.
                    if vlans.len() < 2 {
                        return Err("Each LACP intended pair/source needs two distinct linked positive VLAN cases".into());
                    }
                    current_positive |= matches!(source, Source::Current {});
                }
                Requirement::VlanService { source, query } => {
                    let Some(n) = self.query_index(query) else {
                        return Err("Unknown LACP service query".into());
                    };
                    let q = &self.queries[n].query;
                    if self.source_index(source).is_none()||!self.requirements.iter().any(|r|matches!(r,Requirement::BundlePair{source:at,a,b,minimum_members,..}if at==source&&pair(a,b,&q.a,&q.b)&&q.minimum_members>=2&&q.minimum_members<=*minimum_members)){return Err("LACP service row lacks same-pair same-source positive member anchor".into());}
                }
                Requirement::RetiredMembership { group: g } => {
                    if !group(t, g) {
                        return Err("Unknown retired local LACP group".into());
                    }
                }
                Requirement::PreservePorts { ports } => {
                    if ports.is_empty()
                        || ports.len() > 64
                        || !unique(ports)
                        || ports.iter().any(|p| !endpoint(t, p))
                    {
                        return Err("Invalid LACP preserved port domain".into());
                    }
                }
                Requirement::PreserveGroups { groups } => {
                    if groups.is_empty()
                        || groups.len() > 16
                        || !unique(groups)
                        || groups.iter().any(|g| !group(t, g))
                    {
                        return Err("Invalid LACP preserved group domain".into());
                    }
                }
                Requirement::PreserveVlans { devices } => {
                    if devices.is_empty()
                        || devices.len() > 8
                        || !unique(devices)
                        || devices.iter().any(|d| !id(d) || !t.devices.contains_key(d))
                    {
                        return Err("Invalid LACP preserved VLAN domain".into());
                    }
                }
            }
        }
        if !current_positive
            || self.queries.iter().any(|q| {
                !self
                    .requirements
                    .iter()
                    .any(|r| matches!(r,Requirement::VlanService{query,..}if query==&q.id))
            })
            || self.faults.iter().any(|f| {
                !self.requirements.iter().any(
                |r| matches!(r,Requirement::BundlePair{source:Source::Fault{id},..}if id==&f.id),
            )
            })
        {
            return Err(
                "LACP Definition has no current positive service or unused fault/query".into(),
            );
        }
        if !within(self, MAX_DEFINITION) {
            return Err("LACP Definition aggregate bound".into());
        }
        self.evaluation().validate_for(t).map_err(|e| e.to_string())
    }
    pub fn checks(&self, s: &core::State) -> Vec<(bool, String)> {
        let bad = |e: String| vec![(false, notice(e))];
        if let Err(e) = self.validate() {
            return bad(e);
        }
        if !self.same_assignment(s) {
            return bad("LACP immutable topology or cable availability differs".into());
        }
        let plan = self.evaluation();
        let report = match s.evaluate(&plan) {
            Ok(r) => r,
            Err(e) => return bad(e.to_string()),
        };
        let mut rows = vec![(
            true,
            "Supplied LACP topology and current cable availability match this assignment".into(),
        )];
        for r in &self.requirements {
            let (ok, text) = match r {
                Requirement::BundlePair {
                    source,
                    a,
                    b,
                    minimum_members,
                    cables,
                } => {
                    let snap = &report.snapshots[self.source_index(source).unwrap()];
                    let participating = snap
                        .links()
                        .iter()
                        .filter(|l| {
                            l.participating
                                && l.a
                                    .is_some_and(|x| x.device == a.device && x.group == a.group)
                                && l.b
                                    .is_some_and(|x| x.device == b.device && x.group == b.group)
                                || l.participating
                                    && l.b
                                        .is_some_and(|x| x.device == a.device && x.group == a.group)
                                    && l.a
                                        .is_some_and(|x| x.device == b.device && x.group == b.group)
                        })
                        .collect::<Vec<_>>();
                    let ok = participating.len() >= usize::from(*minimum_members)
                        && participating
                            .iter()
                            .all(|l| cables.iter().any(|n| n == l.cable));
                    let at = match source {
                        Source::Current {} => "Current configuration".into(),
                        Source::Fault { id } => format!("If {id}"),
                    };
                    let diagnosis = if ok {
                        String::new()
                    } else if let Some(link) = participating
                        .iter()
                        .find(|l| !cables.iter().any(|n| n == l.cable))
                    {
                        format!("; {} is outside the supplied member domain", link.cable)
                    } else {
                        snap.links()
                            .iter()
                            .find(|l| !l.participating && cables.iter().any(|n| n == l.cable))
                            .and_then(|l| {
                                l.reasons
                                    .first()
                                    .map(|r| format!("; {}: {}", l.cable, exclusion_text(*r)))
                            })
                            .unwrap_or_default()
                    };
                    (
                        ok,
                        format!(
                            "{at}: {}:{} ↔ {}:{} has {} participating pairs; at least {} in the supplied member domain required{diagnosis}",
                            a.device,
                            a.group,
                            b.device,
                            b.group,
                            participating.len(),
                            minimum_members
                        ),
                    )
                }
                Requirement::VlanService { source, query } => {
                    let qi = self.query_index(query).unwrap();
                    let si = self.source_index(source).unwrap();
                    let q = &self.queries[qi].query;
                    let cell = report.services[si][qi];
                    (
                        matches!(cell, core::Admission::Ready { .. }),
                        format!(
                            "{}: VLAN {} on {}:{} ↔ {}:{}: {}",
                            match source {
                                Source::Current {} => "Current configuration".into(),
                                Source::Fault { id } => format!("If {id}"),
                            },
                            q.vlan,
                            q.a.device,
                            q.a.group,
                            q.b.device,
                            q.b.group,
                            admission_text(cell)
                        ),
                    )
                }
                Requirement::RetiredMembership { group: g } => {
                    let count = s.configuration().devices[&g.device]
                        .ports
                        .values()
                        .filter(|p| p.group.as_ref() == Some(&g.group))
                        .count();
                    let ok = count == 0;
                    (
                        ok,
                        format!(
                            "Retired group {}:{} has {count} configured members; none are required",
                            g.device, g.group
                        ),
                    )
                }
                Requirement::PreservePorts { ports } => {
                    let ok = ports.iter().all(|p| {
                        s.configuration().devices[&p.device].ports[&p.port]
                            == self.initial.configuration().devices[&p.device].ports[&p.port]
                    });
                    (
                        ok,
                        if ok {
                            "The named non-target port settings are unchanged"
                        } else {
                            "Protected port settings changed; restore the named working ports"
                        }
                        .into(),
                    )
                }
                Requirement::PreserveGroups { groups } => {
                    let ok = groups.iter().all(|g| {
                        s.configuration().devices[&g.device].groups[&g.group]
                            == self.initial.configuration().devices[&g.device].groups[&g.group]
                    });
                    (
                        ok,
                        if ok {
                            "The named non-target group settings are unchanged"
                        } else {
                            "Protected group settings changed; restore the named working groups"
                        }
                        .into(),
                    )
                }
                Requirement::PreserveVlans { devices } => {
                    let ok = devices.iter().all(|d| {
                        s.configuration().devices[d].vlans
                            == self.initial.configuration().devices[d].vlans
                    });
                    (
                        ok,
                        if ok {
                            "The named device VLAN databases are unchanged"
                        } else {
                            "Protected device VLAN databases changed; keep their supplied values"
                        }
                        .into(),
                    )
                }
            };
            rows.push((ok, notice(text)));
        }
        rows
    }
}

fn exclusion_text(r: core::Exclusion) -> &'static str {
    match r {
        core::Exclusion::CableDown => "the supplied cable is down",
        core::Exclusion::PortDownA | core::Exclusion::PortDownB => {
            "a port is administratively down"
        }
        core::Exclusion::NoMembershipA | core::Exclusion::NoMembershipB => {
            "a port has no local group membership"
        }
        core::Exclusion::GroupDownA | core::Exclusion::GroupDownB => {
            "a local group is administratively down"
        }
        core::Exclusion::TemplateMismatchA | core::Exclusion::TemplateMismatchB => {
            "a port differs from its own group's rate or profile"
        }
        core::Exclusion::HalfDuplex => "full duplex is required",
        core::Exclusion::LineRateMismatch => "the physical cable ends use different line rates",
        core::Exclusion::BothPassive => "both ends are Passive, so neither initiates",
        core::Exclusion::UnsupportedPartnerSelection => {
            "competing eligible partners are outside this simulator's selection model"
        }
    }
}
fn admission_text(a: core::Admission) -> String {
    match a {
        core::Admission::Ready { members } => {
            format!("Ready through {members} participating pairs")
        }
        core::Admission::Unavailable { reason } => match reason {
            core::ServiceFailure::NoReadyPair => "No reciprocal bundle is ready",
            core::ServiceFailure::TooFewMembers => {
                "Too few participating pairs for this requirement"
            }
            core::ServiceFailure::InactiveVlan => "The data VLAN is not active on both switches",
            core::ServiceFailure::VlanNotAllowed => {
                "The data VLAN is not admitted by both group profiles"
            }
        }
        .into(),
        core::Admission::Unsupported { reason } => match reason {
            core::UnsupportedService::PartnerSelection => {
                "Simulator limitation: competing eligible partner groups need investigation"
            }
            core::UnsupportedService::NativeMismatch => {
                "Simulator limitation: native VLANs differ across the two groups"
            }
            core::UnsupportedService::AccessTrunkMismatch => {
                "Simulator limitation: an access profile and a trunk profile are mixed"
            }
        }
        .into(),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileMode {
    Access,
    Trunk,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileDraft {
    pub mode: ProfileMode,
    #[serde(deserialize_with = "string::<_,8>")]
    pub access: String,
    #[serde(deserialize_with = "string::<_,8>")]
    pub native: String,
    #[serde(deserialize_with = "string::<_,384>")]
    pub allowed: String,
}
fn vlan_number(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Enter a VLAN number from1 through4094".into());
    }
    let n = s.parse::<u16>().map_err(|_| "VLAN number is too large")?;
    if !(1..=4094).contains(&n) {
        return Err("VLAN number must be from1 through4094".into());
    }
    Ok(n)
}
fn vlan_list(s: &str) -> Result<BTreeSet<u16>, String> {
    let mut out = BTreeSet::new();
    if s.trim().is_empty() {
        return Err("Enter at least one VLAN; use a comma-separated list".into());
    }
    for part in s.split(',') {
        if out.len() >= 64 {
            return Err("At most64 VLAN values are supported".into());
        }
        let n = vlan_number(part)?;
        if !out.insert(n) {
            return Err("Remove the repeated VLAN value".into());
        }
    }
    Ok(out)
}
fn active_vlans(s: &str) -> Result<BTreeSet<u16>, String> {
    if s.trim().is_empty() {
        Ok(BTreeSet::new())
    } else {
        vlan_list(s)
    }
}
fn vlan_text(values: &BTreeSet<u16>) -> String {
    values
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
impl ProfileDraft {
    fn from_profile(p: &core::Profile) -> Self {
        match p {
            core::Profile::Access { vlan } => Self {
                mode: ProfileMode::Access,
                access: vlan.to_string(),
                native: "1".into(),
                allowed: "1".into(),
            },
            core::Profile::Trunk { native, allowed } => Self {
                mode: ProfileMode::Trunk,
                access: "1".into(),
                native: native.to_string(),
                allowed: vlan_text(allowed),
            },
        }
    }
    fn valid(&self) -> bool {
        text(&self.access, 8) && text(&self.native, 8) && text(&self.allowed, 384)
    }
    fn parsed(&self) -> Result<core::Profile, String> {
        if !self.valid() {
            return Err("LACP profile text bound".into());
        }
        match self.mode {
            ProfileMode::Access => Ok(core::Profile::Access {
                vlan: vlan_number(&self.access)?,
            }),
            ProfileMode::Trunk => Ok(core::Profile::Trunk {
                native: vlan_number(&self.native)?,
                allowed: vlan_list(&self.allowed)?,
            }),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupDraft {
    pub up: bool,
    pub rate: core::Rate,
    pub profile: ProfileDraft,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortDraft {
    pub up: bool,
    #[serde(deserialize_with = "optional_name")]
    pub group: Option<String>,
    pub mode: core::LacpMode,
    pub rate: core::Rate,
    pub duplex: core::Duplex,
    pub profile: ProfileDraft,
}
fn optional_name<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "string::<_,32>")] String);
    let n = Option::<Name>::deserialize(d)?;
    if n.as_ref().is_some_and(|n| !id(&n.0)) {
        return Err(D::Error::custom("Invalid local group ID"));
    }
    Ok(n.map(|n| n.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceDraft {
    #[serde(deserialize_with = "string::<_,384>")]
    pub vlans: String,
    #[serde(deserialize_with = "map::<_,GroupDraft,4>")]
    pub groups: BTreeMap<String, GroupDraft>,
    #[serde(deserialize_with = "map::<_,PortDraft,8>")]
    pub ports: BTreeMap<String, PortDraft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub version: u32,
    pub devices: BTreeMap<String, DeviceDraft>,
    pub selected_device: String,
    pub selected_group: String,
    pub selected_service: String,
    pub selected_condition: String,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            #[serde(deserialize_with = "map::<_,DeviceDraft,8>")]
            devices: BTreeMap<String, DeviceDraft>,
            #[serde(deserialize_with = "string::<_,32>")]
            selected_device: String,
            #[serde(deserialize_with = "string::<_,32>")]
            selected_group: String,
            #[serde(deserialize_with = "string::<_,32>")]
            selected_service: String,
            #[serde(deserialize_with = "string::<_,32>")]
            selected_condition: String,
        }
        let r = Raw::deserialize(d)?;
        let d = Self {
            version: r.version,
            devices: r.devices,
            selected_device: r.selected_device,
            selected_group: r.selected_group,
            selected_service: r.selected_service,
            selected_condition: r.selected_condition,
        };
        if !d.valid() {
            return Err(D::Error::custom("Invalid bounded LACP draft"));
        }
        Ok(d)
    }
}
impl Draft {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DRAFT {
            return Err("LACP draft raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded LACP draft".into())
    }
    pub fn valid(&self) -> bool {
        if self.version != 1
            || !(2..=8).contains(&self.devices.len())
            || [
                &self.selected_device,
                &self.selected_group,
                &self.selected_service,
                &self.selected_condition,
            ]
            .iter()
            .any(|s| !s.is_empty() && !id(s))
        {
            return false;
        }
        let mut groups = 0usize;
        for (n, d) in &self.devices {
            if !id(n)
                || d.groups.len() > 4
                || d.ports.is_empty()
                || d.ports.len() > 8
                || !text(&d.vlans, 384)
            {
                return false;
            }
            groups += d.groups.len();
            if groups > 16 {
                return false;
            }
            if d.groups.iter().any(|(n, g)| !id(n) || !g.profile.valid())
                || d.ports.iter().any(|(n, p)| {
                    !id(n)
                        || p.group
                            .as_ref()
                            .is_some_and(|g| !id(g) || !d.groups.contains_key(g))
                        || !p.profile.valid()
                })
            {
                return false;
            }
        }
        within(self, MAX_DRAFT)
    }
    pub fn valid_for(&self, s: &core::State) -> bool {
        self.valid()
            && self.devices.keys().eq(s.topology().devices.keys())
            && self.devices.iter().all(|(n, d)| {
                d.groups.keys().eq(s.topology().devices[n].groups.keys())
                    && d.ports.keys().eq(s.topology().devices[n].ports.keys())
            })
            && (self.selected_device.is_empty() || self.devices.contains_key(&self.selected_device))
            && (self.selected_group.is_empty()
                || self
                    .devices
                    .get(&self.selected_device)
                    .is_some_and(|d| d.groups.contains_key(&self.selected_group)))
    }
    pub fn valid_for_definition(&self, d: &Definition, s: &core::State) -> bool {
        self.valid_for(s)
            && (self.selected_service.is_empty()
                || d.queries.iter().any(|q| q.id == self.selected_service))
            && (self.selected_condition.is_empty()
                || d.faults.iter().any(|f| f.id == self.selected_condition))
    }
    pub fn from_state(s: &core::State) -> Result<Self, String> {
        s.validate().map_err(|e| e.to_string())?;
        Self::from_configuration(s.configuration(), s.topology())
    }
    pub fn from_configuration(c: &core::Configuration, t: &core::Topology) -> Result<Self, String> {
        c.validate_for(t).map_err(|e| e.to_string())?;
        if !within(
            &ConfigureRef {
                kind: "Configure",
                configuration: c,
            },
            core::MAX_ACTION_BYTES,
        ) {
            return Err("LACP configuration exceeds its action limit".into());
        }
        fresh_draft_size(c, t).ok_or("LACP fresh draft aggregate before copy")?;
        #[cfg(test)]
        FRESH_DRAFT_COPIES.with(|n| n.set(n.get() + 1));
        let devices = c
            .devices
            .iter()
            .map(|(n, d)| {
                (
                    n.clone(),
                    DeviceDraft {
                        vlans: vlan_text(&d.vlans),
                        groups: d
                            .groups
                            .iter()
                            .map(|(n, g)| {
                                (
                                    n.clone(),
                                    GroupDraft {
                                        up: g.up,
                                        rate: g.template.rate,
                                        profile: ProfileDraft::from_profile(&g.template.profile),
                                    },
                                )
                            })
                            .collect(),
                        ports: d
                            .ports
                            .iter()
                            .map(|(n, p)| {
                                (
                                    n.clone(),
                                    PortDraft {
                                        up: p.up,
                                        group: p.group.clone(),
                                        mode: p.mode,
                                        rate: p.rate,
                                        duplex: p.duplex,
                                        profile: ProfileDraft::from_profile(&p.profile),
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        let draft = Self {
            version: 1,
            devices,
            selected_device: c.devices.keys().next().cloned().unwrap_or_default(),
            selected_group: String::new(),
            selected_service: String::new(),
            selected_condition: String::new(),
        };
        if !draft.valid() {
            return Err("LACP draft aggregate bound".into());
        }
        Ok(draft)
    }
    pub fn parsed(&self, t: &core::Topology) -> Result<core::Configuration, String> {
        if !self.valid()
            || !self.devices.keys().eq(t.devices.keys())
            || self.devices.iter().any(|(n, d)| {
                !d.groups.keys().eq(t.devices[n].groups.keys())
                    || !d.ports.keys().eq(t.devices[n].ports.keys())
            })
        {
            return Err("LACP draft does not match supplied devices/ports/groups".into());
        }
        let mut devices = BTreeMap::new();
        for (n, d) in &self.devices {
            let mut groups = BTreeMap::new();
            for (gn, g) in &d.groups {
                groups.insert(
                    gn.clone(),
                    core::GroupConfig {
                        up: g.up,
                        template: core::Template {
                            rate: g.rate,
                            profile: g
                                .profile
                                .parsed()
                                .map_err(|e| format!("{n}:{gn} group: {e}"))?,
                        },
                    },
                );
            }
            let mut ports = BTreeMap::new();
            for (pn, p) in &d.ports {
                ports.insert(
                    pn.clone(),
                    core::PortConfig {
                        up: p.up,
                        group: p.group.clone(),
                        mode: p.mode,
                        rate: p.rate,
                        duplex: p.duplex,
                        profile: p
                            .profile
                            .parsed()
                            .map_err(|e| format!("{n}:{pn} port: {e}"))?,
                    },
                );
            }
            devices.insert(
                n.clone(),
                core::DeviceConfig {
                    vlans: active_vlans(&d.vlans).map_err(|e| format!("{n} VLAN database: {e}"))?,
                    groups,
                    ports,
                },
            );
        }
        let c = core::Configuration { devices };
        c.validate_for(t).map_err(|e| e.to_string())?;
        if !within(
            &ConfigureRef {
                kind: "Configure",
                configuration: &c,
            },
            core::MAX_ACTION_BYTES,
        ) {
            return Err("LACP parsed action aggregate bound".into());
        }
        Ok(c)
    }
    pub fn matches(&self, s: &core::State) -> bool {
        self.valid_for(s)
            && self
                .parsed(s.topology())
                .is_ok_and(|c| &c == s.configuration())
    }
}

// Borrowed exact wire projection of a fresh draft. No identifier/list String is copied.
struct NumberText(u16);
impl Serialize for NumberText {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.0)
    }
}
struct VlanText<'a>(&'a BTreeSet<u16>);
impl fmt::Display for VlanText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}
impl Serialize for VlanText<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}
struct ProfileRef<'a>(&'a core::Profile);
impl Serialize for ProfileRef<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut out = s.serialize_struct("ProfileDraft", 4)?;
        match self.0 {
            core::Profile::Access { vlan } => {
                out.serialize_field("mode", &ProfileMode::Access)?;
                out.serialize_field("access", &NumberText(*vlan))?;
                out.serialize_field("native", "1")?;
                out.serialize_field("allowed", "1")?;
            }
            core::Profile::Trunk { native, allowed } => {
                out.serialize_field("mode", &ProfileMode::Trunk)?;
                out.serialize_field("access", "1")?;
                out.serialize_field("native", &NumberText(*native))?;
                out.serialize_field("allowed", &VlanText(allowed))?;
            }
        }
        out.end()
    }
}
#[derive(Serialize)]
struct GroupRefView<'a> {
    up: bool,
    rate: core::Rate,
    profile: ProfileRef<'a>,
}
struct GroupsRef<'a>(&'a BTreeMap<String, core::GroupConfig>);
impl Serialize for GroupsRef<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut out = s.serialize_map(Some(self.0.len()))?;
        for (n, g) in self.0 {
            out.serialize_entry(
                n,
                &GroupRefView {
                    up: g.up,
                    rate: g.template.rate,
                    profile: ProfileRef(&g.template.profile),
                },
            )?;
        }
        out.end()
    }
}
#[derive(Serialize)]
struct PortRefView<'a> {
    up: bool,
    group: &'a Option<String>,
    mode: core::LacpMode,
    rate: core::Rate,
    duplex: core::Duplex,
    profile: ProfileRef<'a>,
}
struct PortsRef<'a>(&'a BTreeMap<String, core::PortConfig>);
impl Serialize for PortsRef<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut out = s.serialize_map(Some(self.0.len()))?;
        for (n, p) in self.0 {
            out.serialize_entry(
                n,
                &PortRefView {
                    up: p.up,
                    group: &p.group,
                    mode: p.mode,
                    rate: p.rate,
                    duplex: p.duplex,
                    profile: ProfileRef(&p.profile),
                },
            )?;
        }
        out.end()
    }
}
#[derive(Serialize)]
struct DeviceRefView<'a> {
    vlans: VlanText<'a>,
    groups: GroupsRef<'a>,
    ports: PortsRef<'a>,
}
struct DevicesRef<'a>(&'a core::Configuration);
impl Serialize for DevicesRef<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut out = s.serialize_map(Some(self.0.devices.len()))?;
        for (n, d) in &self.0.devices {
            out.serialize_entry(
                n,
                &DeviceRefView {
                    vlans: VlanText(&d.vlans),
                    groups: GroupsRef(&d.groups),
                    ports: PortsRef(&d.ports),
                },
            )?;
        }
        out.end()
    }
}
#[derive(Serialize)]
pub(crate) struct DraftRef<'a> {
    version: u32,
    devices: DevicesRef<'a>,
    selected_device: &'a str,
    selected_group: &'static str,
    selected_service: &'static str,
    selected_condition: &'static str,
}
pub(crate) fn draft_view(c: &core::Configuration) -> DraftRef<'_> {
    DraftRef {
        version: 1,
        devices: DevicesRef(c),
        selected_device: c
            .devices
            .keys()
            .next()
            .map(String::as_str)
            .unwrap_or_default(),
        selected_group: "",
        selected_service: "",
        selected_condition: "",
    }
}
pub(crate) fn fresh_draft_size(c: &core::Configuration, t: &core::Topology) -> Option<usize> {
    c.validate_for(t).ok()?;
    if !within(
        &ConfigureRef {
            kind: "Configure",
            configuration: c,
        },
        core::MAX_ACTION_BYTES,
    ) {
        return None;
    }
    size(&draft_view(c), MAX_DRAFT)
}
#[cfg(test)]
thread_local! {static FRESH_DRAFT_COPIES:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
#[derive(Serialize)]
struct ConfigureRef<'a> {
    kind: &'static str,
    configuration: &'a core::Configuration,
}
#[derive(Serialize)]
pub(crate) struct StateRef<'a> {
    schema_version: u32,
    topology: &'a core::Topology,
    configuration: &'a core::Configuration,
    environment: &'a core::Environment,
}
pub(crate) fn proposed_state<'a>(s: &'a core::State, c: &'a core::Configuration) -> StateRef<'a> {
    StateRef {
        schema_version: 1,
        topology: s.topology(),
        configuration: c,
        environment: s.environment(),
    }
}
pub(crate) fn admit_configuration(s: &core::State, c: &core::Configuration) -> Result<(), String> {
    c.validate_for(s.topology()).map_err(|e| e.to_string())?;
    if !within(
        &ConfigureRef {
            kind: "Configure",
            configuration: c,
        },
        core::MAX_ACTION_BYTES,
    ) || !within(&proposed_state(s, c), core::MAX_STATE_BYTES)
    {
        return Err("LACP proposed configuration/state aggregate bound".into());
    }
    Ok(())
}

pub fn perform(d: &Definition, s: &mut core::State, a: &core::Action) -> Result<(), String> {
    d.validate()?;
    s.validate().map_err(|e| e.to_string())?;
    if !d.same_assignment(s) {
        return Err("LACP supplied topology/environment differs".into());
    }
    match a {
        core::Action::Configure { configuration } => {
            admit_configuration(s, configuration)?;
            s.reconfigure(configuration.clone())
                .map_err(|e| e.to_string())
        }
    }
}
pub fn execute(d: &Definition, s: &mut core::State, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + core::MAX_ACTION_BYTES {
        return Err("LACP internal action exceeds raw bound before trimming".into());
    }
    d.validate()?;
    s.validate().map_err(|e| e.to_string())?;
    if !d.same_assignment(s) {
        return Err("LACP supplied context differs".into());
    }
    match input.trim() {
        "help" => return Ok(HELP.into()),
        "show" => {
            return Ok(format!(
                "{} supplied devices; configuration is virtual and no frames are sent",
                s.topology().devices.len()
            ));
        }
        _ => {}
    }
    let raw = input
        .strip_prefix(PREFIX)
        .ok_or("Use native LACP editors; no host/IOS command is executed")?;
    let a = core::Action::from_json(raw.as_bytes()).map_err(|e| e.to_string())?;
    perform(d, s, &a)?;
    Ok("Virtual LACP configuration applied".into())
}
pub(crate) fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    if w.one_off_session.is_some() || w.one_off_learning.is_some() {
        return true;
    }
    w.cron_session.is_some()
        || w.cron_learning.is_some()
        || w.file_backup_learning.is_some()
        || w.nfs_session.is_some()
        || w.nfs_learning.is_some()
        || w.nat_learning.is_some()
        || w.rpm_session.is_some()
        || w.rpm_learning.is_some()
        || w.nat_draft.is_some()
        || w.debian_packages_terminal.is_some()
        || w.apache_static_draft.is_some()
        || w.apache_static_learning.is_some()
        || w.debian_packages_draft.is_some()
        || w.debian_packages_learning.is_some()
        || w.spanning_tree_draft.is_some()
        || w.spanning_tree_learning.is_some()
        || w.dhcpv4_draft.is_some()
        || w.dhcpv4_learning.is_some()
        || w.unit_manager_draft.is_some()
        || w.unit_manager_learning.is_some()
        || w.unit_manager_terminal.is_some()
        || w.ospf_draft.is_some()
        || w.ospf_learning.is_some()
        || w.acl_draft.is_some()
        || w.acl_learning.is_some()
        || w.script_learning.is_some()
        || w.terminal_learning.is_some()
        || w.process_learning.is_some()
        || w.accounts_learning.is_some()
        || w.discovery_draft.is_some()
        || w.discovery_learning.is_some()
        || w.storage_draft.is_some()
        || w.storage_learning.is_some()
        || w.ipv6_draft.is_some()
        || w.ipv6_learning.is_some()
        || w.routing_v4_draft.is_some()
        || w.routing_v4_learning.is_some()
        || w.address_learning.is_some()
        || w.service_learning.is_some()
        || w.client_learning.is_some()
        || w.seed_dns_learning.is_some()
        || w.seed_mount_learning.is_some()
        || w.legacy_access_learning.is_some()
        || w.legacy_logs_learning.is_some()
        || w.client_draft.is_some()
        || w.dns_learning.is_some()
        || w.dns_draft.is_some()
        || w.switching_draft.is_some()
        || w.switching_learning.is_some()
        || w.mac_draft.is_some()
        || w.mac_learning.is_some()
}
fn foreign_source(s: &crate::pack::Scenario) -> bool {
    if s.one_off_teaching.is_some() {
        return true;
    }
    s.cron_teaching.is_some()
        || s.physical_link_teaching.is_some()
        || s.file_backup_teaching.is_some()
        || s.teaching.is_some()
        || s.nfs_teaching.is_some()
        || s.nat_teaching.is_some()
        || s.rpm_packages_teaching.is_some()
        || s.apache_static_teaching.is_some()
        || s.spanning_tree_teaching.is_some()
        || s.dhcpv4_teaching.is_some()
        || s.terminal_teaching.is_some()
        || s.process_teaching.is_some()
        || s.accounts_teaching.is_some()
        || s.unit_manager_teaching.is_some()
        || s.ospf_teaching.is_some()
        || s.discovery_teaching.is_some()
        || s.storage_teaching.is_some()
        || s.acl_teaching.is_some()
        || s.ipv6_teaching.is_some()
        || s.ipv4_routing_teaching.is_some()
        || s.address_teaching.is_some()
        || s.service_teaching.is_some()
        || s.client_teaching.is_some()
        || s.seed_dns_teaching.is_some()
        || s.seed_mount_teaching.is_some()
        || s.legacy_access_teaching.is_some()
        || s.legacy_logs_teaching.is_some()
        || s.dns_teaching.is_some()
        || s.switching_teaching.is_some()
        || s.mac_teaching.is_some()
        || s.debian_packages_teaching.is_some()
}

pub(crate) fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    let present = crate::workspace::has_etherchannel_metadata(a);
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return if present {
            Err(
                "LACP workspace metadata belongs to its LACP activity; previous work retained"
                    .into(),
            )
        } else {
            Ok(())
        };
    };
    simulation.validate().map_err(|e| e.to_string())?;
    if a.ipv4.is_some()
        || a.transcript.len() > 120
        || a.workspace.as_ref().is_some_and(|w| {
            foreign(w)
                || !w.valid()
                || w.etherchannel_draft
                    .as_ref()
                    .is_some_and(|d| !d.valid_for(simulation))
                || w.etherchannel_learning.as_ref().is_some_and(|l| {
                    !crate::etherchannel_teaching::separate(&l.context.initial, simulation)
                })
        })
        || !within(a, MAX_ATTEMPT)
    {
        return Err(
            "LACP saved capacity, draft or foreign metadata differs; previous work retained".into(),
        );
    }
    Ok(())
}
pub(crate) fn context<'a>(
    a: &crate::progress::Attempt,
    s: &'a crate::pack::Scenario,
) -> Result<&'a Definition, String> {
    if s.physical_link_teaching.is_some() {
        return Err("Physical-link teaching belongs to its own activity".into());
    }
    crate::workspace::reject_file_backup_workspace(a)?;
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || foreign_source(s)
        || !within(s, MAX_SCENARIO)
    {
        return Err("LACP full source/version/presentation bound differs".into());
    }
    metadata(a)?;
    let (
        crate::engine::State::EtherChannel { simulation },
        crate::engine::State::EtherChannel {
            simulation: initial,
        },
        crate::engine::Goal::EtherChannel { definition },
    ) = (&a.state, &s.initial, &s.goal)
    else {
        return Err("LACP State/Goal kind differs".into());
    };
    if initial != &definition.initial
        || !definition.same_assignment(simulation)
        || definition.validate().is_err()
    {
        return Err("LACP exact authored initial, topology or environment differs".into());
    }
    if s.etherchannel_teaching
        .as_ref()
        .is_some_and(|l| !l.bounded() || !l.separate_from(initial))
        || a.workspace.as_ref().is_some_and(|w| {
            w.etherchannel_draft
                .as_ref()
                .is_some_and(|d| !d.valid_for_definition(definition, simulation))
                || w.etherchannel_learning.as_ref().is_some_and(|l| {
                    s.etherchannel_teaching
                        .as_ref()
                        .is_none_or(|source| !l.valid_for(source))
                })
        })
    {
        return Err("LACP draft/practice differs from the supplied source".into());
    }
    Ok(definition)
}
#[derive(Serialize)]
#[serde(tag = "kind")]
enum EngineRef<'a, T: Serialize + ?Sized> {
    EtherChannel { simulation: &'a T },
}
fn field_bytes(name: &str, n: Option<usize>) -> Option<usize> {
    match n {
        None => Some(0),
        Some(n) => n.checked_add(name.len() + 4),
    }
}
fn option_size<T: Serialize>(v: Option<&T>, cap: usize) -> Option<Option<usize>> {
    match v {
        None => Some(None),
        Some(v) => Some(Some(size(v, cap)?)),
    }
}
fn projection<T: Serialize + ?Sized>(
    a: &crate::progress::Attempt,
    simulation: &T,
    draft: Option<usize>,
    learning: Option<usize>,
    visited: Option<u64>,
) -> Option<usize> {
    let old = size(a, MAX_ATTEMPT)?;
    let old_state = size(&a.state, MAX_ATTEMPT)?;
    let new_state = size(&EngineRef::EtherChannel { simulation }, MAX_ATTEMPT)?;
    let blank = crate::workspace::WorkspaceData::new();
    let w = a.workspace.as_ref().unwrap_or(&blank);
    let old_w = size(w, MAX_ATTEMPT)?;
    let old_d = option_size(w.etherchannel_draft.as_ref(), MAX_DRAFT)?;
    let old_l = option_size(
        w.etherchannel_learning.as_ref(),
        crate::etherchannel_teaching::MAX_LEARNING,
    )?;
    let next_w = old_w
        .checked_sub(field_bytes("etherchannel_draft", old_d)?)?
        .checked_sub(field_bytes("etherchannel_learning", old_l)?)?
        .checked_add(field_bytes("etherchannel_draft", draft)?)?
        .checked_add(field_bytes("etherchannel_learning", learning)?)?;
    let mut n = old.checked_sub(old_state)?.checked_add(new_state)?;
    n = if a.workspace.is_some() {
        n.checked_sub(old_w)?.checked_add(next_w)?
    } else {
        n.checked_add("workspace".len() + 4)?.checked_add(next_w)?
    };
    if let Some(visited) = visited {
        n = n
            .checked_sub(size(&a.visited, 32)?)?
            .checked_add(size(&visited, 32)?)?;
    }
    Some(n)
}
fn admitted(n: Option<usize>) -> Result<(), String> {
    if n.is_some_and(|n| n <= MAX_ATTEMPT) {
        Ok(())
    } else {
        Err("LACP complete saved Attempt exceeds2MiB; previous work retained".into())
    }
}
fn retained(a: &crate::progress::Attempt) -> Result<(Option<usize>, Option<usize>), String> {
    let w = a.workspace.as_ref();
    Ok((
        option_size(w.and_then(|w| w.etherchannel_draft.as_ref()), MAX_DRAFT)
            .ok_or("LACP retained draft bound")?,
        option_size(
            w.and_then(|w| w.etherchannel_learning.as_ref()),
            crate::etherchannel_teaching::MAX_LEARNING,
        )
        .ok_or("LACP retained practice bound")?,
    ))
}
pub(crate) fn prepare_draft(a: &mut crate::progress::Attempt) -> Result<(), String> {
    metadata(a)?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("Not an LACP workspace".into());
    };
    if a.workspace
        .as_ref()
        .is_some_and(|w| w.etherchannel_draft.is_some())
    {
        return Ok(());
    }
    let draft_bytes = fresh_draft_size(simulation.configuration(), simulation.topology())
        .ok_or("LACP draft bound")?;
    let (_, learning) = retained(a)?;
    admitted(projection(a, simulation, Some(draft_bytes), learning, None))?;
    let draft = Draft::from_state(simulation)?;
    a.workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .etherchannel_draft = Some(draft);
    Ok(())
}
pub(crate) fn prepare_attempt(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    new: bool,
) -> Result<(), String> {
    context(a, s)?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let needs_draft = a
        .workspace
        .as_ref()
        .is_none_or(|w| w.etherchannel_draft.is_none());
    let (old_d, old_l) = retained(a)?;
    let draft_bytes = if needs_draft {
        Some(
            fresh_draft_size(simulation.configuration(), simulation.topology())
                .ok_or("LACP new draft bound")?,
        )
    } else {
        old_d
    };
    let new_learning = old_l.is_none() && s.etherchannel_teaching.is_some();
    let learning_bytes = if let Some(l) = s.etherchannel_teaching.as_ref().filter(|_| new_learning)
    {
        Some(
            crate::etherchannel_teaching::Learning::new_size(l, new)
                .ok_or("LACP new practice bound")?,
        )
    } else {
        old_l
    };
    admitted(projection(a, simulation, draft_bytes, learning_bytes, None))?;
    let draft = if needs_draft {
        Some(Draft::from_state(simulation)?)
    } else {
        None
    };
    let fresh = s
        .etherchannel_teaching
        .as_ref()
        .filter(|_| new_learning)
        .map(|l| crate::etherchannel_teaching::Learning::new(l, new))
        .transpose()?;
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if let Some(d) = draft {
        w.etherchannel_draft = Some(d);
    }
    if let Some(l) = fresh {
        w.etherchannel_learning = Some(l);
    }
    Ok(())
}
/// Source-aware borrowed admission for the native workspace; no grading or mutation.
pub fn validate_context(
    a: &crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s).map(|_| ())
}

pub fn set_assignment_draft(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    draft: &Draft,
) -> Result<(), String> {
    let d = context(a, s)?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    if !draft.valid_for_definition(d, simulation) {
        return Err("LACP proposed draft differs from supplied context".into());
    }
    let (_, learning) = retained(a)?;
    admitted(projection(
        a,
        simulation,
        Some(size(draft, MAX_DRAFT).ok_or("LACP proposed draft aggregate")?),
        learning,
        None,
    ))?;
    a.workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .etherchannel_draft = Some(copy_draft(draft));
    Ok(())
}
pub fn apply_assignment(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let draft = a
        .workspace
        .as_ref()
        .and_then(|w| w.etherchannel_draft.as_ref())
        .ok_or("LACP draft is not prepared")?;
    let candidate = draft.parsed(simulation.topology())?;
    admit_configuration(simulation, &candidate)?;
    let (draft_size, learning_size) = retained(a)?;
    admitted(projection(
        a,
        &proposed_state(simulation, &candidate),
        draft_size,
        learning_size,
        None,
    ))?;
    let crate::engine::State::EtherChannel { simulation } = &mut a.state else {
        unreachable!()
    };
    simulation.reconfigure(candidate).map_err(|e| e.to_string())
}
pub fn set_practice_draft(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    draft: &Draft,
) -> Result<(), String> {
    context(a, s)?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.etherchannel_learning.as_ref())
        .ok_or("LACP practice is not prepared")?;
    if !draft.valid_for_definition(&l.context, &l.simulation) {
        return Err("LACP practice draft differs from its supplied context".into());
    }
    let learning = l
        .proposed_size(&l.simulation, draft, l.view, l.selected_example, false)
        .ok_or("LACP practice draft aggregate")?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let (draft_size, _) = retained(a)?;
    admitted(projection(a, simulation, draft_size, Some(learning), None))?;
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .etherchannel_learning
        .as_mut()
        .unwrap();
    l.draft = copy_draft(draft);
    l.checked = false;
    Ok(())
}
pub fn apply_practice(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.etherchannel_learning.as_ref())
        .ok_or("LACP practice is not prepared")?;
    let candidate = l.draft.parsed(l.simulation.topology())?;
    admit_configuration(&l.simulation, &candidate)?;
    let learning = l
        .proposed_size(
            &proposed_state(&l.simulation, &candidate),
            &l.draft,
            l.view,
            l.selected_example,
            false,
        )
        .ok_or("LACP proposed practice aggregate")?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let (draft_size, _) = retained(a)?;
    admitted(projection(a, simulation, draft_size, Some(learning), None))?;
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .etherchannel_learning
        .as_mut()
        .unwrap();
    l.simulation
        .reconfigure(candidate)
        .map_err(|e| e.to_string())?;
    l.checked = false;
    Ok(())
}
pub fn set_practice_presentation(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    view: Option<crate::script_teaching::View>,
    selected: Option<usize>,
    checked: Option<bool>,
) -> Result<(), String> {
    context(a, s)?;
    let source = s
        .etherchannel_teaching
        .as_ref()
        .ok_or("LACP lesson absent")?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.etherchannel_learning.as_ref())
        .ok_or("LACP practice is not prepared")?;
    let view = view.unwrap_or(l.view);
    let selected = selected.unwrap_or(l.selected_example);
    let checked = checked.unwrap_or(l.checked);
    if selected >= source.example_actions.len() {
        return Err("LACP example selection differs".into());
    }
    let learning = l
        .proposed_size(&l.simulation, &l.draft, view, selected, checked)
        .ok_or("LACP presentation bound")?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let (draft_size, _) = retained(a)?;
    admitted(projection(a, simulation, draft_size, Some(learning), None))?;
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .etherchannel_learning
        .as_mut()
        .unwrap();
    l.view = view;
    l.selected_example = selected;
    l.checked = checked;
    Ok(())
}
pub fn fill_practice(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.etherchannel_learning.as_ref())
        .ok_or("LACP practice is not prepared")?;
    let source = s
        .etherchannel_teaching
        .as_ref()
        .ok_or("LACP lesson absent")?;
    let core::Action::Configure { configuration } =
        &source.example_actions[l.selected_example].action;
    fresh_draft_size(configuration, l.simulation.topology()).ok_or("LACP Fill draft bound")?;
    let learning = l
        .proposed_size(
            &l.simulation,
            &draft_view(configuration),
            l.view,
            l.selected_example,
            false,
        )
        .ok_or("LACP Fill practice bound")?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let (draft_size, _) = retained(a)?;
    admitted(projection(a, simulation, draft_size, Some(learning), None))?;
    let draft = Draft::from_configuration(configuration, l.simulation.topology())?;
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .etherchannel_learning
        .as_mut()
        .unwrap();
    l.draft = draft;
    l.checked = false;
    Ok(())
}
pub fn reset_practice(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let source = s
        .etherchannel_teaching
        .as_ref()
        .ok_or("LACP lesson absent")?;
    if a.workspace
        .as_ref()
        .and_then(|w| w.etherchannel_learning.as_ref())
        .is_none()
    {
        return Err("LACP practice is not prepared".into());
    }
    let n = crate::etherchannel_teaching::Learning::new_size_view(
        source,
        crate::script_teaching::View::Example,
    )
    .ok_or("LACP reset practice aggregate")?;
    let crate::engine::State::EtherChannel { simulation } = &a.state else {
        return Err("LACP kind differs".into());
    };
    let (draft_size, _) = retained(a)?;
    admitted(projection(a, simulation, draft_size, Some(n), None))?;
    let fresh = crate::etherchannel_teaching::Learning::new_view(
        source,
        crate::script_teaching::View::Example,
    )?;
    a.workspace.as_mut().unwrap().etherchannel_learning = Some(fresh);
    Ok(())
}
pub fn record_visit(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    visited: u64,
) -> Result<(), String> {
    context(a, s)?;
    let old = size(a, MAX_ATTEMPT).ok_or("LACP saved capacity bound")?;
    admitted(
        old.checked_sub(size(&a.visited, 32).ok_or("LACP prior visit bound")?)
            .and_then(|n| n.checked_add(size(&visited, 32)?)),
    )?;
    a.visited = visited;
    Ok(())
}

pub(crate) fn reject_foreign_source(s: &crate::pack::Scenario) -> Result<(), String> {
    crate::physical_link_workspace::reject_foreign_source(s)?;
    if s.etherchannel_teaching.is_some()
        && !matches!(
            (&s.initial, &s.goal),
            (
                crate::engine::State::EtherChannel { .. },
                crate::engine::Goal::EtherChannel { .. }
            )
        )
    {
        Err("LACP teaching belongs to its LACP activity".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
thread_local! {static DRAFT_COPIES:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
fn copy_draft(d: &Draft) -> Draft {
    #[cfg(test)]
    DRAFT_COPIES.with(|n| n.set(n.get() + 1));
    d.clone()
}
pub fn record_hint(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    hints: usize,
) -> Result<(), String> {
    context(a, s)?;
    if hints > s.hints.len() {
        return Err("LACP hint selection differs from its source".into());
    }
    let old = size(a, MAX_ATTEMPT).ok_or("LACP saved capacity bound")?;
    admitted(
        old.checked_sub(size(&a.hints, 32).ok_or("LACP previous hint count")?)
            .and_then(|n| n.checked_add(size(&hints, 32)?)),
    )?;
    a.hints = hints;
    Ok(())
}
pub fn set_saved_check(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    checked: bool,
) -> Result<(), String> {
    context(a, s)?;
    let w = a
        .workspace
        .as_ref()
        .ok_or("LACP workspace is not prepared")?;
    let old = size(a, MAX_ATTEMPT).ok_or("LACP saved capacity bound")?;
    admitted(
        old.checked_sub(size(&w.checked, 8).ok_or("LACP previous check bit")?)
            .and_then(|n| n.checked_add(size(&checked, 8)?)),
    )?;
    a.workspace.as_mut().unwrap().checked = checked;
    Ok(())
}

pub fn record_mode(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    challenge: bool,
) -> Result<(), String> {
    context(a, s)?;
    let old = size(a, MAX_ATTEMPT).ok_or("LACP saved capacity bound")?;
    admitted(
        old.checked_sub(size(&a.challenge, 8).ok_or("LACP prior mode bound")?)
            .and_then(|n| n.checked_add(size(&challenge, 8)?)),
    )?;
    a.challenge = challenge;
    Ok(())
}
#[cfg(test)]
mod copy_frontier_tests {
    use super::*;
    use crate::{engine::State, etherchannel_teaching as teach, progress::Attempt};
    use serde_json::json;
    fn source() -> crate::pack::Scenario {
        source_case(0)
    }
    fn source_case(case: usize) -> crate::pack::Scenario {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/etherchannel-workbench-v1.json"
        ))
        .unwrap();
        let c = &f["cases"][case];
        let solution = c["reference"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| format!("{PREFIX}{}", serde_json::to_string(a).unwrap()))
            .collect::<Vec<_>>();
        crate::pack::load(&json!({"schema_version":1,"scenarios":[{"id":"lacp-frontier","version":1,"track":"CCNA","topic":"Technical closure","unit":"","title":"Unregistered LACP frontier","summary":"Bounded proposal fixture","objective":"Preserve separate saved work and apply a real bundle repair","retry_guidance":"Inspect actual state","difficulty":"Intermediate","minutes":15,"steps":["Inspect","Apply","Check"],"guide":[],"explanation":"No host actions","hints":["Apply differs from edit"],"sources":["https://1.ieee802.org/tsn/802-1ax-rev/"],"simplifications":"Bounded L2 eligibility only","initial":{"kind":"EtherChannel","simulation":c["definition"]["initial"]},"goal":{"kind":"EtherChannel","definition":c["definition"]},"solution":solution,"prerequisites":[],"etherchannel_teaching":c["lesson"]}]}).to_string()).unwrap().remove(0)
    }
    fn full(a: &mut Attempt) {
        let mut lo = 0;
        let mut hi = MAX_ATTEMPT;
        while lo < hi {
            let n = (lo + hi).div_ceil(2);
            a.transcript = vec!["x".repeat(n)];
            if size(a, MAX_ATTEMPT).is_some() {
                lo = n;
            } else {
                hi = n - 1;
            }
        }
        a.transcript = vec!["x".repeat(lo)];
        assert_eq!(serde_json::to_vec(a).unwrap().len(), MAX_ATTEMPT);
    }
    #[test]
    fn whole_attempt_refusal_precedes_real_draft_and_learning_copy_sites() {
        let s = source();
        let mut low = Attempt::new(&s, 9);
        teach::prepare(&mut low, &s, true).unwrap();
        let mut high = low.clone();
        full(&mut high);
        let mut d = low
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_draft
            .as_ref()
            .unwrap()
            .clone();
        d.devices
            .get_mut("task0-left")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .profile
            .access = "\u{1}".repeat(8);
        DRAFT_COPIES.with(|n| n.set(0));
        set_assignment_draft(&mut low, &s, &d).unwrap();
        assert_eq!(DRAFT_COPIES.with(|n| n.get()), 1);
        DRAFT_COPIES.with(|n| n.set(0));
        let before = serde_json::to_value(&high).unwrap();
        assert!(set_assignment_draft(&mut high, &s, &d).is_err());
        assert_eq!(DRAFT_COPIES.with(|n| n.get()), 0);
        assert!(serde_json::to_value(&high).unwrap() == before);
        let mut high = Attempt::new(&s, 9);
        full(&mut high);
        let before = serde_json::to_value(&high).unwrap();
        teach::NEW_LEARNING_COPIES.with(|n| n.set(0));
        FRESH_DRAFT_COPIES.with(|n| n.set(0));
        assert!(teach::prepare(&mut high, &s, true).is_err());
        assert_eq!(FRESH_DRAFT_COPIES.with(|n| n.get()), 0);
        assert_eq!(teach::NEW_LEARNING_COPIES.with(|n| n.get()), 0);
        assert!(serde_json::to_value(&high).unwrap() == before);
        let mut low = Attempt::new(&s, 9);
        teach::prepare(&mut low, &s, true).unwrap();
        assert_eq!(teach::NEW_LEARNING_COPIES.with(|n| n.get()), 1);
        assert_eq!(FRESH_DRAFT_COPIES.with(|n| n.get()), 2);
    }
    #[test]
    fn borrowed_full_projection_matches_independently_encoded_optional_and_escaped_values() {
        let s = source();
        for existing in [false, true] {
            let mut a = Attempt::new(&s, 9);
            if existing {
                teach::prepare(&mut a, &s, true).unwrap();
            }
            let State::EtherChannel { simulation } = &a.state else {
                panic!()
            };
            let mut d = Draft::from_state(simulation).unwrap();
            d.devices
                .get_mut("task0-left")
                .unwrap()
                .ports
                .get_mut("p1")
                .unwrap()
                .profile
                .access = "\u{1}".repeat(8);
            let learning =
                teach::Learning::new(s.etherchannel_teaching.as_ref().unwrap(), true).unwrap();
            let expected = projection(
                &a,
                simulation,
                Some(size(&d, MAX_DRAFT).unwrap()),
                Some(size(&learning, teach::MAX_LEARNING).unwrap()),
                Some(10),
            )
            .unwrap();
            let mut own = a.clone();
            let w = own
                .workspace
                .get_or_insert_with(crate::workspace::WorkspaceData::new);
            w.etherchannel_draft = Some(d);
            w.etherchannel_learning = Some(learning);
            own.visited = 10;
            assert_eq!(expected, serde_json::to_vec(&own).unwrap().len());
        }
    }
    #[test]
    fn typed_n_plus_one_map_and_sequence_reject_before_the_next_body() {
        thread_local! {static BODIES:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
        struct Probe;
        impl<'de> Deserialize<'de> for Probe {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                BODIES.with(|n| n.set(n.get() + 1));
                serde::de::IgnoredAny::deserialize(d)?;
                Ok(Self)
            }
        }
        BODIES.with(|n| n.set(0));
        let mut d = serde_json::Deserializer::from_str("[0,{\"unfinished\"");
        let e = list::<_, Probe, 1>(&mut d).err().unwrap().to_string();
        assert!(e.contains("frontier"));
        assert_eq!(BODIES.with(|n| n.get()), 1);
        BODIES.with(|n| n.set(0));
        let mut d = serde_json::Deserializer::from_str("{\"first\":0,\"second\":{\"unfinished\"");
        let e = map::<_, Probe, 1>(&mut d).err().unwrap().to_string();
        assert!(e.contains("frontier"));
        assert_eq!(BODIES.with(|n| n.get()), 1);
    }
    #[test]
    fn borrowed_fresh_draft_wire_and_learning_size_are_exact_without_owning_source() {
        let s = source();
        let crate::engine::State::EtherChannel { simulation } = &s.initial else {
            panic!()
        };
        for access in [false, true] {
            let mut c = simulation.configuration().clone();
            if access {
                for d in c.devices.values_mut() {
                    for p in d.ports.values_mut() {
                        p.profile = core::Profile::Access { vlan: 10 };
                    }
                    for g in d.groups.values_mut() {
                        g.template.profile = core::Profile::Access { vlan: 10 };
                    }
                }
            }
            let owned = Draft::from_configuration(&c, simulation.topology()).unwrap();
            assert_eq!(
                serde_json::to_value(draft_view(&c)).unwrap(),
                serde_json::to_value(&owned).unwrap()
            );
            assert_eq!(
                fresh_draft_size(&c, simulation.topology()).unwrap(),
                serde_json::to_vec(&owned).unwrap().len()
            );
        }
        let lesson = s.etherchannel_teaching.as_ref().unwrap();
        for view in [
            crate::script_teaching::View::Introduction,
            crate::script_teaching::View::Example,
            crate::script_teaching::View::Assignment,
        ] {
            let n = teach::Learning::new_size_view(lesson, view).unwrap();
            let owned = teach::Learning::new_view(lesson, view).unwrap();
            assert_eq!(n, serde_json::to_vec(&owned).unwrap().len());
        }
    }
    #[test]
    fn fill_frontier_refuses_before_any_fresh_draft_copy_but_delivers_same_positive_action() {
        let s = source_case(1);
        let mut low = Attempt::new(&s, 9);
        teach::prepare(&mut low, &s, true).unwrap();
        let mut high = low.clone();
        full(&mut high);
        let before = serde_json::to_value(&high).unwrap();
        FRESH_DRAFT_COPIES.with(|n| n.set(0));
        assert!(fill_practice(&mut high, &s).is_err());
        assert_eq!(FRESH_DRAFT_COPIES.with(|n| n.get()), 0);
        assert!(serde_json::to_value(&high).unwrap() == before);
        fill_practice(&mut low, &s).unwrap();
        assert_eq!(FRESH_DRAFT_COPIES.with(|n| n.get()), 1);
        assert_ne!(
            low.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .draft,
            high.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .draft
        );
    }
}
