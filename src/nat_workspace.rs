//! Bounded Basic NAT application context and editors. No packets or reachability.
use crate::nat::{self, Action, Configuration, Inventory, State};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{
    collections::BTreeMap,
    fmt,
    io::{self, Write},
    marker::PhantomData,
    net::Ipv4Addr,
};
pub const MAX_GOAL: usize = 512 * 1024;
pub const MAX_DRAFT: usize = 64 * 1024;
pub const MAX_REQUIREMENTS: usize = 32;
pub const MAX_TRIALS: usize = 4;
pub const MAX_STEPS: usize = 16;
pub const MAX_FRESH_CALLS: usize = 128;
pub const PREFIX: &str = "basic-nat ";
pub const HELP: &str = "Simulator-only typed Basic NAT actions: configuration, explicit address probes and ClearDynamic. Static maps and current dynamic bindings are separate from policy. No IOS/Linux command, PAT, ports, sessions, timers, routing, connectivity or security verdict is modeled.";
pub fn notice(s: &str) -> String {
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].to_owned()
}
struct Counter {
    used: usize,
    cap: usize,
}
impl Write for Counter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.used = self
            .used
            .checked_add(b.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| io::Error::other("NAT app bound"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    serde_json::to_writer(Counter { used: 0, cap }, v).is_ok()
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<'de, const N: usize> Visitor<'de> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} UTF8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                Err(E::custom("NAT text bound"))
            } else {
                Ok(s.to_owned())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn optional_id<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Optional;
    impl<'de> Visitor<'de> for Optional {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("optional bounded NAT ID")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            text::<D, 32>(d).map(Some)
        }
    }
    d.deserialize_option(Optional)
}
struct Element<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Element<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("NAT collection bound"));
        }
        T::deserialize(d)
    }
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} NAT items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(x) = a.next_element_seed(Element {
                allowed: v.len() < N,
                marker: PhantomData,
            })? {
                v.push(x);
            }
            Ok(v)
        }
    }
    d.deserialize_seq(Items::<T, N>(PhantomData))
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub fn admitted(s: &State) -> bool {
    s.validate().is_ok() && within(s, nat::MAX_STATE_JSON)
}
pub fn same_context(initial: &State, current: &State) -> bool {
    admitted(initial) && admitted(current) && initial.inventory() == current.inventory()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Static,
    Dynamic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Global {
    Any {},
    Exact { address: Ipv4Addr },
}
impl Global {
    fn accepts(self, a: Ipv4Addr) -> bool {
        match self {
            Self::Any {} => true,
            Self::Exact { address } => address == a,
        }
    }
    fn valid(self, i: &Inventory) -> bool {
        match self {
            Self::Any {} => true,
            Self::Exact { address } => i.globals.contains(&address),
        }
    }
}
fn mapping(s: &State, inside: &str) -> Option<(Ipv4Addr, Kind)> {
    s.configuration()
        .statics
        .get(inside)
        .map(|g| (*g, Kind::Static))
        .or_else(|| s.bindings().get(inside).map(|g| (*g, Kind::Dynamic)))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    CurrentMapping {
        #[serde(deserialize_with = "text::<_,32>")]
        inside: String,
        mapping_kind: Kind,
        global: Global,
    },
    NoCurrentMapping {
        #[serde(deserialize_with = "text::<_,32>")]
        inside: String,
    },
    StaticPreserved {
        #[serde(deserialize_with = "text::<_,32>")]
        inside: String,
    },
    DynamicPrefixMatches {
        #[serde(deserialize_with = "text::<_,32>")]
        inside: String,
        matches: bool,
    },
    DynamicCapacity {
        minimum: u8,
    },
    DynamicRulePreserved {},
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Expected {
    Translated { mapping_kind: Kind, global: Global },
    Untranslated {},
    PoolExhausted {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Step {
    Outbound {
        #[serde(deserialize_with = "text::<_,32>")]
        inside: String,
        #[serde(deserialize_with = "text::<_,32>")]
        peer: String,
        expected: Expected,
    },
    Reverse {
        #[serde(deserialize_with = "text::<_,32>")]
        inside: String,
        #[serde(deserialize_with = "text::<_,32>")]
        peer: String,
        mapping_kind: Kind,
        global: Global,
    },
    NoMapping {
        #[serde(deserialize_with = "text::<_,32>")]
        peer: String,
        global: Ipv4Addr,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    #[serde(deserialize_with = "text::<_,64>")]
    pub name: String,
    #[serde(deserialize_with = "list::<_,Step,MAX_STEPS>")]
    pub steps: Vec<Step>,
}
impl Trial {
    fn positive(&self, inside: Option<&str>, kind: Option<Kind>) -> bool {
        self.steps.iter().any(|s| {
            if let Step::Outbound {
                inside: h,
                expected:
                    Expected::Translated {
                        mapping_kind: k, ..
                    },
                ..
            } = s
            {
                inside.is_none_or(|i| i == h)
                    && kind.is_none_or(|v| v == *k)
                    && self
                        .steps
                        .iter()
                        .any(|s| matches!(s,Step::Reverse{inside:r,mapping_kind:q,..}if r==h&&q==k))
            } else {
                false
            }
        })
    }
    fn valid(&self, i: &Inventory) -> bool {
        self.name.len() <= 64
            && !self.name.contains('\0')
            && !self.name.trim().is_empty()
            && !self.steps.is_empty()
            && self.steps.len() <= MAX_STEPS
            && self.steps.iter().all(|s| match s {
                Step::Outbound {
                    inside,
                    peer,
                    expected,
                } => {
                    id(inside)
                        && id(peer)
                        && i.inside.contains_key(inside)
                        && i.outside.contains_key(peer)
                        && match expected {
                            Expected::Translated { global, .. } => global.valid(i),
                            _ => true,
                        }
                }
                Step::Reverse {
                    inside,
                    peer,
                    global,
                    ..
                } => {
                    id(inside)
                        && id(peer)
                        && i.inside.contains_key(inside)
                        && i.outside.contains_key(peer)
                        && global.valid(i)
                }
                Step::NoMapping { peer, .. } => id(peer) && i.outside.contains_key(peer),
            })
            && self.positive(None, None)
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Definition {
    pub initial: State,
    pub requirements: Vec<Requirement>,
    pub trials: Vec<Trial>,
}
impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            initial: State,
            #[serde(deserialize_with = "list::<_,Requirement,MAX_REQUIREMENTS>")]
            requirements: Vec<Requirement>,
            #[serde(deserialize_with = "list::<_,Trial,MAX_TRIALS>")]
            trials: Vec<Trial>,
        }
        let r = Raw::deserialize(d)?;
        let v = Self {
            initial: r.initial,
            requirements: r.requirements,
            trials: r.trials,
        };
        v.validate().map_err(D::Error::custom)?;
        Ok(v)
    }
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_GOAL {
            return Err("NAT definition raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NAT definition".into())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.requirements.is_empty()
            || self.requirements.len() > MAX_REQUIREMENTS
            || self.trials.is_empty()
            || self.trials.len() > MAX_TRIALS
        {
            return Err("NAT requirement/trial count bound".into());
        }
        // Preflight EVERY borrowed nested count/string before walking a State, counting
        // serialization or cloning. Direct constructors get the same typed bounds.
        if self.trials.iter().any(|t| {
            t.name.len() > 64
                || t.steps.is_empty()
                || t.steps.len() > MAX_STEPS
                || t.steps.iter().any(|s| match s {
                    Step::Outbound { inside, peer, .. } | Step::Reverse { inside, peer, .. } => {
                        inside.len() > 32 || peer.len() > 32
                    }
                    Step::NoMapping { peer, .. } => peer.len() > 32,
                })
        }) || self.requirements.iter().any(|r| match r {
            Requirement::CurrentMapping { inside, .. }
            | Requirement::NoCurrentMapping { inside }
            | Requirement::StaticPreserved { inside }
            | Requirement::DynamicPrefixMatches { inside, .. } => inside.len() > 32,
            _ => false,
        }) {
            return Err("NAT borrowed nested bound".into());
        }
        if !admitted(&self.initial) {
            return Err("NAT authored initial state admission".into());
        }
        let i = self.initial.inventory();
        let mut positive = false;
        for (n, r) in self.requirements.iter().enumerate() {
            let valid = match r {
                Requirement::CurrentMapping { inside, global, .. } => {
                    positive = true;
                    id(inside) && i.inside.contains_key(inside) && global.valid(i)
                }
                Requirement::NoCurrentMapping { inside } => {
                    id(inside) && i.inside.contains_key(inside)
                }
                Requirement::StaticPreserved { inside } => {
                    positive = true;
                    id(inside) && self.initial.configuration().statics.contains_key(inside)
                }
                Requirement::DynamicPrefixMatches { inside, matches } => {
                    positive |= *matches;
                    id(inside)
                        && i.inside.contains_key(inside)
                        && (!*matches
                            || self
                                .trials
                                .iter()
                                .any(|t| t.positive(Some(inside), Some(Kind::Dynamic))))
                }
                Requirement::DynamicCapacity { minimum } => {
                    positive = true;
                    *minimum > 0
                        && u64::from(*minimum) <= nat::MAX_POOL
                        && self
                            .trials
                            .iter()
                            .any(|t| t.positive(None, Some(Kind::Dynamic)))
                }
                Requirement::DynamicRulePreserved {} => true,
            };
            if !valid || self.requirements[..n].contains(r) {
                return Err(
                    "Invalid/duplicate NAT requirement or missing positive dynamic test".into(),
                );
            }
        }
        if !positive
            || self
                .trials
                .iter()
                .enumerate()
                .any(|(n, t)| !t.valid(i) || self.trials[..n].contains(t))
            || !within(self, MAX_GOAL)
        {
            return Err("NAT needs positive current/policy and independent positive reverse-tested policy trials".into());
        }
        Ok(())
    }
    pub fn same_assignment(&self, s: &State) -> bool {
        self.validate().is_ok() && same_context(&self.initial, s)
    }
    pub fn current_checks(&self, s: &State) -> Vec<(bool, String)> {
        if !self.same_assignment(s) {
            return vec![(
                false,
                "NAT current state differs from admitted authored inventory".into(),
            )];
        }
        self.requirements.iter().map(|r|{let(ok,label)=match r{
 Requirement::CurrentMapping{inside,mapping_kind,global}=>(mapping(s,inside).is_some_and(|(g,k)|k==*mapping_kind&&global.accepts(g)),format!("Existing {mapping_kind:?} mapping for {inside}")),
 Requirement::NoCurrentMapping{inside}=>(mapping(s,inside).is_none(),format!("No current mapping for {inside}")),
 Requirement::StaticPreserved{inside}=>(s.configuration().statics.get(inside)==self.initial.configuration().statics.get(inside),format!("Keep the supplied static mapping for {inside}")),
 Requirement::DynamicPrefixMatches{inside,matches}=>(s.configuration().dynamic.is_some_and(|r|r.source.contains(s.inventory().inside[inside])==*matches),format!("Applied dynamic prefix {} {inside}",if *matches {"includes"}else{"excludes"})),
 Requirement::DynamicCapacity{minimum}=>{let capacity=s.configuration().dynamic.map_or(0,|r|r.pool.cardinality().unwrap_or(0)-s.configuration().statics.values().filter(|g|r.pool.contains(**g)).count() as u64);(capacity>=u64::from(*minimum),format!("Policy has at least {minimum} dynamic address slots after static reservations"))},
 Requirement::DynamicRulePreserved{}=>(s.configuration().dynamic==self.initial.configuration().dynamic,"Keep the supplied dynamic rule unchanged".into()),
 };(ok,if ok{format!("Satisfied: {label}")}else{format!("Needs attention: {label}; inspect applied policy and actual mappings")})}).collect()
    }
    pub fn report(&self, s: &State) -> Report {
        if !self.same_assignment(s) {
            return Report {
                rows: vec![(false, "NAT current/goal context or bounds differ".into())],
                calls: 0,
                clones: 0,
            };
        }
        let mut report = Report {
            rows: self.current_checks(s),
            calls: 0,
            clones: 0,
        };
        for (n, t) in self.trials.iter().enumerate() {
            let mut trial = s.clone();
            report.clones += 1;
            report.calls += 1;
            let mut error = trial
                .apply(&Action::ClearDynamic {})
                .err()
                .map(|e| notice(&e.message));
            for step in &t.steps {
                if error.is_some() {
                    break;
                }
                report.calls += 1;
                if let Err(e) = trial_step(&mut trial, step) {
                    error = Some(notice(&e));
                }
            }
            report.rows.push((error.is_none(),notice(&match error{Some(e)=>format!("Policy test {} ({}): {e}",n+1,t.name),None=>format!("Policy test {} ({}): new mappings and reverse checks pass on a separate copy",n+1,t.name)})));
        }
        debug_assert!(report.calls <= MAX_FRESH_CALLS && report.clones <= MAX_TRIALS);
        report
    }
    pub fn checks(&self, s: &State) -> Vec<(bool, String)> {
        self.report(s).rows
    }
}
#[derive(Debug)]
pub struct Report {
    pub rows: Vec<(bool, String)>,
    pub calls: usize,
    pub clones: usize,
}
fn kind(k: nat::MappingKind) -> Kind {
    match k {
        nat::MappingKind::Static => Kind::Static,
        nat::MappingKind::Dynamic => Kind::Dynamic,
    }
}
fn trial_step(s: &mut State, step: &Step) -> Result<(), String> {
    let (action, expected) = match step {
        Step::Outbound {
            inside,
            peer,
            expected,
        } => (
            Action::Outbound {
                inside: inside.clone(),
                peer: peer.clone(),
            },
            *expected,
        ),
        Step::Reverse {
            inside,
            peer,
            mapping_kind,
            global,
        } => {
            let Some((g, k)) = mapping(s, inside) else {
                return Err(format!(
                    "No existing trial mapping for {inside}; reversal never allocates"
                ));
            };
            if k != *mapping_kind || !global.accepts(g) {
                return Err(format!("Wrong current trial mapping for {inside}"));
            }
            let effect = s
                .apply(&Action::Inbound {
                    peer: peer.clone(),
                    global: g,
                })
                .map_err(|e| notice(&e.message))?;
            return if matches!(effect,nat::Effect::Probe{result:nat::ProbeResult::Translated{before,after,kind:q}}if kind(q)==*mapping_kind&&before.source==s.inventory().outside[peer]&&before.destination==g&&after.source==before.source&&after.destination==s.inventory().inside[inside])
            {
                Ok(())
            } else {
                Err(format!("Reverse address rewrite differs for {inside}"))
            };
        }
        Step::NoMapping { peer, global } => {
            let effect = s
                .apply(&Action::Inbound {
                    peer: peer.clone(),
                    global: *global,
                })
                .map_err(|e| notice(&e.message))?;
            return if matches!(effect,nat::Effect::Probe{result:nat::ProbeResult::NoMapping{addresses}}if addresses.source==s.inventory().outside[peer]&&addresses.destination==*global)
            {
                Ok(())
            } else {
                Err("Unmapped inbound address unexpectedly has a mapping".into())
            };
        }
    };
    let Action::Outbound { inside, peer } = &action else {
        unreachable!()
    };
    let local = s.inventory().inside[inside];
    let remote = s.inventory().outside[peer];
    let effect = s.apply(&action).map_err(|e| notice(&e.message))?;
    let ok = match (effect, expected) {
        (
            nat::Effect::Probe {
                result:
                    nat::ProbeResult::Translated {
                        before,
                        after,
                        kind: q,
                    },
            },
            Expected::Translated {
                mapping_kind,
                global,
            },
        ) => {
            kind(q) == mapping_kind
                && global.accepts(after.source)
                && before.source == local
                && before.destination == remote
                && after.destination == remote
        }
        (
            nat::Effect::Probe {
                result: nat::ProbeResult::Untranslated { addresses },
            },
            Expected::Untranslated {},
        )
        | (
            nat::Effect::Probe {
                result: nat::ProbeResult::PoolExhausted { addresses },
            },
            Expected::PoolExhausted {},
        ) => addresses.source == local && addresses.destination == remote,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "Outbound address result differs for {inside}; inspect eligibility, capacity and mapping kind"
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticRow {
    #[serde(deserialize_with = "text::<_,32>")]
    pub inside: String,
    #[serde(deserialize_with = "text::<_,64>")]
    pub global: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub context: Inventory,
    pub statics: Vec<StaticRow>,
    pub dynamic_enabled: bool,
    pub network: String,
    pub prefix: String,
    pub first: String,
    pub last: String,
    pub inside: Option<String>,
    pub peer: Option<String>,
    pub inbound_global: String,
    pub selected_row: Option<u8>,
    pub notice: Option<String>,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            context: Inventory,
            #[serde(deserialize_with = "list::<_,StaticRow,32>")]
            statics: Vec<StaticRow>,
            dynamic_enabled: bool,
            #[serde(deserialize_with = "text::<_,64>")]
            network: String,
            #[serde(deserialize_with = "text::<_,16>")]
            prefix: String,
            #[serde(deserialize_with = "text::<_,64>")]
            first: String,
            #[serde(deserialize_with = "text::<_,64>")]
            last: String,
            #[serde(deserialize_with = "optional_id")]
            inside: Option<String>,
            #[serde(deserialize_with = "optional_id")]
            peer: Option<String>,
            #[serde(deserialize_with = "text::<_,64>")]
            inbound_global: String,
            selected_row: Option<u8>,
            notice: Option<Notice>,
        }
        #[derive(Deserialize)]
        struct Notice(#[serde(deserialize_with = "text::<_,512>")] String);
        let r = Raw::deserialize(d)?;
        let v = Self {
            context: r.context,
            statics: r.statics,
            dynamic_enabled: r.dynamic_enabled,
            network: r.network,
            prefix: r.prefix,
            first: r.first,
            last: r.last,
            inside: r.inside,
            peer: r.peer,
            inbound_global: r.inbound_global,
            selected_row: r.selected_row,
            notice: r.notice.map(|s| s.0),
        };
        if !v.valid() {
            return Err(D::Error::custom("Invalid bounded NAT draft"));
        }
        Ok(v)
    }
}
impl Draft {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DRAFT {
            return Err("NAT draft raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NAT draft".into())
    }
    pub fn new(s: &State) -> Result<Self, String> {
        if !admitted(s) {
            return Err("NAT state admission".into());
        }
        let c = s.configuration();
        let d = c.dynamic;
        let v = Self {
            context: s.inventory().clone(),
            statics: c
                .statics
                .iter()
                .map(|(i, g)| StaticRow {
                    inside: i.clone(),
                    global: g.to_string(),
                })
                .collect(),
            dynamic_enabled: d.is_some(),
            network: d.map_or(String::new(), |r| r.source.network.to_string()),
            prefix: d.map_or(String::new(), |r| r.source.length.to_string()),
            first: d.map_or(String::new(), |r| r.pool.first.to_string()),
            last: d.map_or(String::new(), |r| r.pool.last.to_string()),
            inside: s.inventory().inside.keys().next().cloned(),
            peer: s.inventory().outside.keys().next().cloned(),
            inbound_global: String::new(),
            selected_row: None,
            notice: None,
        };
        if v.valid() {
            Ok(v)
        } else {
            Err("NAT draft exceeds bounds".into())
        }
    }
    pub fn valid(&self) -> bool {
        self.statics.len() <= 32
            && self
                .statics
                .iter()
                .all(|r| r.inside.len() <= 32 && r.global.len() <= 64)
            && self.network.len() <= 64
            && self.prefix.len() <= 16
            && self.first.len() <= 64
            && self.last.len() <= 64
            && self.inbound_global.len() <= 64
            && self.notice.as_ref().is_none_or(|s| s.len() <= 512)
            && self.inside.as_ref().is_none_or(|s| s.len() <= 32)
            && self.peer.as_ref().is_none_or(|s| s.len() <= 32)
            && Action::ClearDynamic {}.validate_for(&self.context).is_ok()
            && self
                .inside
                .as_ref()
                .is_none_or(|s| self.context.inside.contains_key(s))
            && self
                .peer
                .as_ref()
                .is_none_or(|s| self.context.outside.contains_key(s))
            && self
                .selected_row
                .is_none_or(|n| usize::from(n) < self.statics.len())
            && within(self, MAX_DRAFT)
    }
    pub fn valid_for(&self, s: &State) -> bool {
        self.valid() && admitted(s) && &self.context == s.inventory()
    }
    pub fn configuration(&self) -> Result<Configuration, String> {
        if !self.valid() {
            return Err("NAT editor bounds/context differ".into());
        }
        let mut statics = BTreeMap::new();
        for row in &self.statics {
            let inside = row.inside.trim();
            if !id(inside) || !self.context.inside.contains_key(inside) {
                return Err("Choose a supplied inside identity for every static row".into());
            }
            let g = row
                .global
                .trim()
                .parse()
                .map_err(|_| "Static global needs a supplied IPv4 address")?;
            if statics.insert(inside.to_owned(), g).is_some() {
                return Err("Duplicate static inside identity".into());
            }
        }
        let dynamic = if self.dynamic_enabled {
            Some(nat::DynamicRule {
                source: nat::Prefix {
                    network: self
                        .network
                        .trim()
                        .parse()
                        .map_err(|_| "Dynamic network needs an IPv4 address")?,
                    length: self
                        .prefix
                        .trim()
                        .parse()
                        .map_err(|_| "Prefix length must be0..32")?,
                },
                pool: nat::Pool {
                    first: self
                        .first
                        .trim()
                        .parse()
                        .map_err(|_| "Pool first needs IPv4")?,
                    last: self
                        .last
                        .trim()
                        .parse()
                        .map_err(|_| "Pool last needs IPv4")?,
                },
            })
        } else {
            None
        };
        let action = Action::Configure {
            configuration: Configuration { statics, dynamic },
        };
        action
            .validate_for(&self.context)
            .map_err(|e| notice(&e.message))?;
        let Action::Configure { configuration } = action else {
            unreachable!()
        };
        Ok(configuration)
    }
    pub fn matches(&self, s: &State) -> bool {
        self.valid_for(s) && self.configuration().is_ok_and(|c| &c == s.configuration())
    }
    pub fn apply(&self, initial: &State, current: &mut State) -> Result<bool, String> {
        if !self.valid_for(current) || self.context != *initial.inventory() {
            return Err("NAT editor origin differs".into());
        }
        apply(
            initial,
            current,
            &Action::Configure {
                configuration: self.configuration()?,
            },
        )
    }
}
pub fn perform(
    initial: &State,
    current: &mut State,
    action: &Action,
) -> Result<(bool, nat::Effect), String> {
    if !same_context(initial, current) {
        return Err("NAT immutable assignment inventory differs".into());
    }
    action
        .validate_for(initial.inventory())
        .map_err(|e| notice(&e.message))?;
    let mut candidate = current.clone();
    let effect = candidate.apply(action).map_err(|e| notice(&e.message))?;
    if !admitted(&candidate) {
        return Err("NAT candidate exceeds app bounds".into());
    }
    let changed = &candidate != current;
    *current = candidate;
    Ok((changed, effect))
}
pub fn apply(initial: &State, current: &mut State, action: &Action) -> Result<bool, String> {
    perform(initial, current, action).map(|(changed, _)| changed)
}
pub fn execute(initial: &State, current: &mut State, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + nat::MAX_ACTION_JSON {
        return Err("Typed NAT input exceeds raw bound".into());
    }
    let raw = input
        .strip_prefix(PREFIX)
        .ok_or("Use the simulator-only typed Basic NAT action helper")?;
    let action = Action::from_json(raw.as_bytes()).map_err(|e| notice(&e.message))?;
    let changed = apply(initial, current, &action)?;
    Ok(if changed {
        "Applied Basic NAT state changed; inspect current mappings"
    } else {
        "Basic NAT action completed; no mapping state changed"
    }
    .into())
}
