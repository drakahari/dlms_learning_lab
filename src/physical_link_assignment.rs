//! Typed outcome adapter over the physical-uplink core. No registered Scenario.
//! Fresh probes use the current configuration, never saved readings or run flags.
use crate::physical_link as core;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_DEFINITION: usize = 262_144;
pub const MAX_DRAFT: usize = 65_536;
pub type Rows = Vec<(bool, String)>;
pub(crate) fn short<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "text up to{N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N || s.contains('\0') {
                return Err(E::custom("Physical lesson text bound before copy"));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(Text::<N>)
}
pub(crate) fn list<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    use serde::de::{DeserializeSeed, SeqAccess, Visitor};
    use std::{fmt, marker::PhantomData};
    struct Next<T> {
        allowed: bool,
        p: PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Next<T> {
        type Value = T;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
            if !self.allowed {
                return Err(serde::de::Error::custom(
                    "Physical lesson collection limit before body",
                ));
            }
            T::deserialize(d)
        }
    }
    struct Items<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} physical lesson rows")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Next {
                allowed: out.len() < N,
                p: PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(PhantomData))
}
fn protected<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<BTreeSet<String>, D::Error> {
    struct Name(String);
    impl<'de> Deserialize<'de> for Name {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            short::<D, 48>(d).map(Self)
        }
    }
    let names = list::<D, Name, N>(d)?;
    let mut out = BTreeSet::new();
    for Name(n) in names {
        if !id(&n) || !out.insert(n) {
            return Err(serde::de::Error::custom(
                "Invalid/duplicate protected identifier",
            ));
        }
    }
    Ok(out)
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 120 && !s.chars().any(char::is_control)
}
pub(crate) fn bytes<T: Serialize + ?Sized>(v: &T, cap: usize) -> Option<usize> {
    struct Counter {
        n: usize,
        cap: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Physical assignment byte bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Counter { n: 0, cap };
    crate::native_observations::physical_json();
    serde_json::to_writer(&mut c, v).ok()?;
    Some(c.n)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeCase {
    pub a_frames: u32,
    pub b_frames: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkRequirement {
    #[serde(deserialize_with = "short::<_,48>")]
    pub id: String,
    #[serde(deserialize_with = "short::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "short::<_,48>")]
    pub link: String,
    pub minimum_rate: core::Rate,
    pub full_duplex: bool,
    #[serde(deserialize_with = "list::<_,ProbeCase,8>")]
    pub probes: Vec<ProbeCase>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortObservation {
    #[serde(deserialize_with = "short::<_,48>")]
    pub port: String,
    pub reading: core::SuppliedReading,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: core::State,
    #[serde(deserialize_with = "list::<_,LinkRequirement,8>")]
    pub requirements: Vec<LinkRequirement>,
    #[serde(deserialize_with = "protected::<_,24>")]
    pub protected_ports: BTreeSet<String>,
    #[serde(deserialize_with = "protected::<_,12>")]
    pub protected_links: BTreeSet<String>,
    #[serde(default, deserialize_with = "protected::<_,24>")]
    pub retired_ports: BTreeSet<String>,
    #[serde(deserialize_with = "list::<_,PortObservation,24>")]
    pub observations: Vec<PortObservation>,
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("Physical assignment exceeds256KiB".into());
        }
        let d: Self =
            serde_json::from_slice(raw).map_err(|e| format!("Invalid physical assignment: {e}"))?;
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.to_string())?;
        if self.version != 1
            || self.requirements.is_empty()
            || self.requirements.len() > 8
            || self.protected_ports.len() > core::MAX_PORTS
            || self.protected_links.len() > core::MAX_LINKS
            || self.retired_ports.len() > core::MAX_PORTS
            || self.observations.len() > core::MAX_PORTS
            || bytes(self, MAX_DEFINITION).is_none()
        {
            return Err("Physical assignment version/cardinality/byte bound".into());
        }
        let topology = self.initial.topology();
        let mut ids = BTreeSet::new();
        let mut links = BTreeSet::new();
        for r in &self.requirements {
            if !id(&r.id)
                || !label(&r.label)
                || !ids.insert(&r.id)
                || !links.insert(&r.link)
                || !topology.links.contains_key(&r.link)
                || r.probes.len() < 2
                || r.probes.len() > 8
                || r.probes.iter().collect::<BTreeSet<_>>().len() != r.probes.len()
                || r.probes.iter().any(|p| {
                    p.a_frames == 0
                        || p.b_frames == 0
                        || p.a_frames > core::MAX_PROBE
                        || p.b_frames > core::MAX_PROBE
                })
            {
                return Err("Each target needs a unique supplied link and2..8 distinct bidirectional probe cases".into());
            }
            // Every candidate receive impairment must be exercised by every fresh case.
            // An authored sparse fault cannot evade grading merely through sample phase.
            let shortest = r
                .probes
                .iter()
                .map(|p| p.a_frames.min(p.b_frames))
                .min()
                .unwrap();
            if topology.cables.values().any(|c|matches!(c.condition,core::Condition::ReceiveImpairment{every} if u32::from(every)>shortest)){return Err("Probe cases are too short to exercise a supplied receive impairment".into());}
        }
        if self
            .protected_ports
            .iter()
            .chain(&self.retired_ports)
            .any(|p| !topology.ports.contains_key(p))
            || self
                .protected_links
                .iter()
                .any(|p| !topology.links.contains_key(p))
        {
            return Err("Protected context names an unknown port/link".into());
        }
        // Every non-target interface/cable stays explicitly protected. An old
        // link whose two interfaces are retired is a managed handover scope.
        let mut managed_ports = self.retired_ports.clone();
        let mut managed_links = BTreeSet::new();
        for r in &self.requirements {
            let link = &topology.links[&r.link];
            managed_ports.insert(link.a.clone());
            managed_ports.insert(link.b.clone());
            managed_links.insert(r.link.clone());
        }
        for (name, link) in &topology.links {
            if self.retired_ports.contains(&link.a) && self.retired_ports.contains(&link.b) {
                managed_links.insert(name.clone());
            }
        }
        if topology
            .ports
            .keys()
            .any(|name| !managed_ports.contains(name) && !self.protected_ports.contains(name))
            || topology
                .links
                .keys()
                .any(|name| !managed_links.contains(name) && !self.protected_links.contains(name))
        {
            return Err(
                "Every interface/link outside the stated task must remain protected".into(),
            );
        }
        let mut observed = BTreeSet::new();
        for observation in &self.observations {
            if !topology.ports.contains_key(&observation.port)
                || !observed.insert(&observation.port)
            {
                return Err("Supplied observation needs a unique known port".into());
            }
            observation.reading.validate().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn context(&self, current: &core::State) -> Result<(), String> {
        self.validate()?;
        current.validate().map_err(|e| e.to_string())?;
        if current.topology() != self.initial.topology() {
            return Err("The supplied equipment, cable conditions, route spans or traffic pattern changed; your work is retained".into());
        }
        Ok(())
    }
    pub fn checks(&self, current: &core::State) -> Rows {
        if let Err(e) = self.context(current) {
            return vec![(false, e)];
        }
        let mut out = vec![(
            true,
            "Supplied topology and route requirements are unchanged".into(),
        )];
        for name in &self.protected_ports {
            out.push((
                current.configuration().ports[name] == self.initial.configuration().ports[name],
                format!("Preserve the configuration of {name}"),
            ));
        }
        for name in &self.protected_links {
            out.push((
                current.configuration().cables[name] == self.initial.configuration().cables[name],
                format!("Preserve the assigned cable on {name}"),
            ));
        }
        for name in &self.retired_ports {
            out.push((
                !current.configuration().ports[name].admin_up,
                format!("Retire the old interface {name} by stopping it"),
            ));
        }
        for r in &self.requirements {
            match current.report(&r.link) {
                Err(e) => {
                    out.push((false, format!("{}: {e}", r.label)));
                    out.push((
                        false,
                        format!(
                            "{}: operating mode cannot be verified in this supported subset",
                            r.label
                        ),
                    ));
                }
                Ok(report) => {
                    let qualified = report.qualification == core::Qualification::Qualified;
                    let spec = &current.topology().links[&r.link];
                    let cable =
                        &current.topology().cables[&current.configuration().cables[&r.link]];
                    let qualification_why=match report.qualification {
                        core::Qualification::Qualified=>"Selected media and cable span fit the supplied profile".into(),
                        core::Qualification::IncompatibleProfile=>"The two interface profiles differ. Choose compatible profiles at both ends".into(),
                        core::Qualification::IncompatibleCable=>"The cable type does not match both selected interface profiles".into(),
                        core::Qualification::TooShortForRoute=>format!("The selected cable is{}m long; this route needs at least{}m",cable.length_m,spec.required_span_m),
                        core::Qualification::OutsideReach=>"This length is outside the supplied profile. Choose a supported media/span combination".into(),
                    };
                    out.push((qualified, format!("{}: {qualification_why}", r.label)));
                    let operating = report.state == core::LinkState::Up
                        && report.a.zip(report.b).is_some_and(|(a, b)| {
                            a.ability.rate >= r.minimum_rate
                                && b.ability.rate >= r.minimum_rate
                                && (!r.full_duplex
                                    || (a.ability.duplex == core::Duplex::Full
                                        && b.ability.duplex == core::Duplex::Full))
                        });
                    let operating_why: String = if operating {
                        "Both ends use the required compatible operating mode".into()
                    } else {
                        match report.state {
                        core::LinkState::AdministrativelyDown=>"A required port is stopped. Bring both ends online".into(),
                        core::LinkState::Disconnected=>"The selected cable is declared disconnected in this exercise".into(),
                        core::LinkState::NoCommonAbility=>"The ends have no common operating rate. Compare their Auto/Fixed and rate settings".into(),
                        core::LinkState::DegradedDuplex=>"One end uses half duplex and the other full duplex. Match the settings or use Auto on both ends".into(),
                        core::LinkState::OutsideModel=>"Media/span is outside the supplied profile; traffic cannot be verified by this model".into(),
                        core::LinkState::Up=>"The link is up but its rate or duplex does not meet this assignment's requirement".into(),
                    }
                    };
                    out.push((operating, format!("{}: {operating_why}", r.label)));
                }
            }
            for (n, case) in r.probes.iter().enumerate() {
                // Fresh deterministic sample state. Current saved counters/sequence and
                // reported observations confer no authority and are not altered by checking.
                let result =
                    core::State::new(current.topology().clone(), current.configuration().clone())
                        .and_then(|mut sample| {
                            sample.perform(&core::Action::Probe {
                                link: r.link.clone(),
                                a_frames: case.a_frames,
                                b_frames: case.b_frames,
                            })
                        });
                let (ok, why) = match result {
                    Ok(Some(report)) => {
                        let spec = &current.topology().links[&r.link];
                        let a = &report.after[&spec.a];
                        let b = &report.after[&spec.b];
                        let ok = a.received == u64::from(case.b_frames)
                            && b.received == u64::from(case.a_frames)
                            && a.crc_errors == 0
                            && b.crc_errors == 0
                            && a.collisions == 0
                            && b.collisions == 0
                            && a.late_collisions == 0
                            && b.late_collisions == 0;
                        (
                            ok,
                            if ok {
                                "both directions delivered all sample traffic without new modeled errors".into()
                            } else {
                                format!(
                                    "{} received{}of{}; {} received{}of{}. Inspect fresh errors rather than historical totals",
                                    current.topology().ports[&spec.a].label,
                                    a.received,
                                    case.b_frames,
                                    current.topology().ports[&spec.b].label,
                                    b.received,
                                    case.a_frames
                                )
                            },
                        )
                    }
                    Ok(None) => (false, "The sample did not produce a probe result".into()),
                    Err(e) => (false, e.to_string()),
                };
                out.push((ok, format!("{} · fresh sample{}: {why}", r.label, n + 1)));
            }
        }
        out
    }
    pub fn solved(&self, current: &core::State) -> bool {
        let rows = self.checks(current);
        !rows.is_empty() && rows.iter().all(|r| r.0)
    }
    pub fn validate_reference(&self, configuration: &core::Configuration) -> Result<(), String> {
        self.validate()?;
        if self.solved(&self.initial) {
            return Err(
                "An authored assignment must begin with a meaningful unmet requirement".into(),
            );
        }
        configuration
            .validate(self.initial.topology())
            .map_err(|e| e.to_string())?;
        let mut current = self.initial.clone();
        current
            .perform(&core::Action::Configure {
                configuration: configuration.clone(),
            })
            .map_err(|e| e.to_string())?;
        if !self.solved(&current) {
            return Err("Reference configuration does not satisfy every actual/protected/fresh-probe outcome".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub configuration: core::Configuration,
    #[serde(deserialize_with = "short::<_,48>")]
    pub selected_port: String,
    #[serde(deserialize_with = "short::<_,48>")]
    pub selected_link: String,
}
impl Draft {
    pub fn valid(&self) -> bool {
        self.version == 1
            && bytes(self, MAX_DRAFT).is_some()
            && self.configuration.ports.len() <= core::MAX_PORTS
            && self.configuration.cables.len() <= core::MAX_LINKS
            && self.configuration.ports.keys().all(|k| id(k))
            && self
                .configuration
                .cables
                .iter()
                .all(|(k, v)| id(k) && id(v))
            && (self.selected_port.is_empty() || id(&self.selected_port))
            && (self.selected_link.is_empty() || id(&self.selected_link))
    }

    pub fn from_state(state: &core::State) -> Result<Self, String> {
        state.validate().map_err(|e| e.to_string())?;
        if bytes(state.configuration(), MAX_DRAFT - 1024).is_none() {
            return Err("Configuration cannot fit its native draft".into());
        }
        Ok(Self {
            version: 1,
            configuration: state.configuration().clone(),
            selected_port: state
                .topology()
                .ports
                .keys()
                .next()
                .cloned()
                .unwrap_or_default(),
            selected_link: state
                .topology()
                .links
                .keys()
                .next()
                .cloned()
                .unwrap_or_default(),
        })
    }
    pub fn valid_shape(&self, t: &core::Topology) -> bool {
        self.version == 1
            && self.configuration.ports.keys().eq(t.ports.keys())
            && self.configuration.cables.keys().eq(t.links.keys())
            && (self.selected_port.is_empty() || t.ports.contains_key(&self.selected_port))
            && (self.selected_link.is_empty() || t.links.contains_key(&self.selected_link))
            && self
                .configuration
                .ports
                .values()
                .all(|p| match &p.negotiation {
                    core::Negotiation::Auto { advertise } => advertise.len() <= 5,
                    core::Negotiation::Fixed { .. } => true,
                })
            && bytes(self, MAX_DRAFT).is_some()
    }
    pub fn matches(&self, current: &core::State) -> bool {
        self.configuration == *current.configuration()
    }
}
pub fn current_checks(definition: &Definition, current: &core::State, draft: &Draft) -> Rows {
    if !draft.valid_shape(definition.initial.topology()) {
        return vec![(
            false,
            "The draft does not match the supplied ports/links".into(),
        )];
    }
    let applied = draft.matches(current);
    let mut rows = vec![(
        applied,
        if applied {
            "The current configuration is applied".into()
        } else {
            "Apply the current draft before checking these outcomes".into()
        },
    )];
    rows.extend(definition.checks(current));
    rows
}
pub fn apply(
    definition: &Definition,
    current: &mut core::State,
    draft: &Draft,
) -> Result<(), String> {
    definition.context(current)?;
    if !draft.valid_shape(current.topology()) {
        return Err("Draft belongs to another supplied workspace".into());
    }
    draft
        .configuration
        .validate(current.topology())
        .map_err(|e| e.to_string())?;
    current
        .perform(&core::Action::Configure {
            configuration: draft.configuration.clone(),
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Borrowed shape of a new native draft, before copying the Configuration.
#[derive(Serialize)]
pub(crate) struct FreshDraft<'a> {
    version: u32,
    configuration: &'a core::Configuration,
    selected_port: &'a str,
    selected_link: &'a str,
}
pub(crate) fn fresh_draft_for<'a>(state: &'a core::State, link: &'a str) -> FreshDraft<'a> {
    FreshDraft {
        version: 1,
        configuration: state.configuration(),
        selected_port: state
            .topology()
            .links
            .get(link)
            .map_or("", |l| l.a.as_str()),
        selected_link: link,
    }
}
