//! Source-bound native syslog policy editors and separate practice. No host I/O.
use crate::{
    cron_workspace as bounded,
    script_teaching::{Concept, View},
    syslog as core,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
pub mod binding;

pub const MAX_SESSION: usize = 512 * 1024;
pub const MAX_DEFINITION: usize = 768 * 1024;
pub const MAX_CONTEXT: usize = 2 * 1024 * 1024;
pub const MAX_ACTION: usize = 2048;
pub const MAX_ROWS: usize = 64;
pub const HELP: &str = "Use the native source/collector settings, Apply settings, Generate simulated event and Check configuration. This activity implements no IOS or Linux terminal commands. Historical logs do not prove current policy.";
type Result<T> = std::result::Result<T, String>;
fn text<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    crate::cron::text::<D, 15>(d)
}
fn map<'de, D, K, V, const N: usize>(d: D) -> std::result::Result<BTreeMap<K, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
{
    struct M<K, V, const N: usize>(std::marker::PhantomData<(K, V)>);
    impl<'de, K: Deserialize<'de> + Ord, V: Deserialize<'de>, const N: usize>
        serde::de::Visitor<'de> for M<K, V, N>
    {
        type Value = BTreeMap<K, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most{N} unique syslog entries")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key::<K>()? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(serde::de::Error::custom(
                        "Syslog map bound/duplicate before value",
                    ));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<K, V, N>(std::marker::PhantomData))
}
fn host_texts<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct T(#[serde(deserialize_with = "text")] String);
    bounded::sequence::<D, T, 8>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
fn source_drafts<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<Ipv4Addr, SourceDraft>, D::Error> {
    map::<D, _, _, 8>(d)
}
fn collector_drafts<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<Ipv4Addr, CollectorDraft>, D::Error> {
    map::<D, _, _, 8>(d)
}
fn wanted_remotes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<Ipv4Addr, WantedRemote>, D::Error> {
    map::<D, _, _, 8>(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WantedRemote {
    pub accept: bool,
    #[serde(deserialize_with = "bounded::sequence::<_,core::Facility,24>")]
    pub facilities: Vec<core::Facility>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub device: Ipv4Addr,
    pub event: core::EventId,
    pub local: Option<bool>,
    #[serde(deserialize_with = "wanted_remotes")]
    pub remote: BTreeMap<Ipv4Addr, WantedRemote>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    ApplySource {
        device: Ipv4Addr,
        policy: core::DevicePolicy,
    },
    ApplyCollector {
        collector: Ipv4Addr,
        policy: core::CollectorPolicy,
    },
    Generate {
        device: Ipv4Addr,
        event: core::EventId,
    },
    ClearLocal {
        device: Ipv4Addr,
    },
    ClearCollector {
        collector: Ipv4Addr,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Worked {
    pub action: Action,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub description: String,
    pub fixture: core::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,32>")]
    pub targets: Vec<Target>,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,8>")]
    pub protected_sources: Vec<Ipv4Addr>,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,8>")]
    pub protected_collectors: Vec<Ipv4Addr>,
    #[serde(deserialize_with = "bounded::sequence::<_,Worked,12>")]
    pub worked: Vec<Worked>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub try_change: String,
    pub experiment: Action,
    pub repair: Action,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: core::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Target,32>")]
    pub targets: Vec<Target>,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,8>")]
    pub protected_sources: Vec<Ipv4Addr>,
    #[serde(deserialize_with = "bounded::sequence::<_,Ipv4Addr,8>")]
    pub protected_collectors: Vec<Ipv4Addr>,
    pub lesson: Lesson,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "address",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Component {
    Source(Ipv4Addr),
    Collector(Ipv4Addr),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDraft {
    pub enabled: bool,
    pub buffer: Option<core::Severity>,
    pub remote: Option<core::Severity>,
    pub facility: core::Facility,
    #[serde(deserialize_with = "host_texts")]
    pub hosts: Vec<String>,
}
impl SourceDraft {
    pub fn from_policy(p: &core::DevicePolicy) -> Result<Self> {
        if p.hosts.len() > 8
            || p.hosts
                .iter()
                .any(|ip| ip.octets()[0] == 0 || ip.octets()[0] == 127 || ip.octets()[0] >= 224)
        {
            return Err("Invalid bounded source policy before copying editor fields".into());
        }
        Ok(Self {
            enabled: p.process_enabled,
            buffer: p.buffer_threshold,
            remote: p.remote_threshold,
            facility: p.export_facility,
            hosts: p.hosts.iter().map(ToString::to_string).collect(),
        })
    }
    fn shape(&self) -> bool {
        self.hosts.len() <= 8
            && self.hosts.iter().all(|s| {
                s.len() <= 15
                    && !s
                        .chars()
                        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
            })
    }
    pub fn policy(&self) -> Result<core::DevicePolicy> {
        if !self.shape() {
            return Err(
                "Host address fields exceed their supported size; previous settings retained"
                    .into(),
            );
        }
        let mut hosts = BTreeSet::new();
        for (n, s) in self.hosts.iter().enumerate() {
            if s.is_empty() {
                return Err(format!(
                    "Host row {} is blank. Enter an IPv4 address or remove that row.",
                    n + 1
                ));
            }
            let ip:Ipv4Addr=s.parse().map_err(|_|format!("Host row {} is not a complete IPv4 address. Use four decimal numbers separated by dots.",n+1))?;
            if ip.octets()[0] == 0 || ip.octets()[0] == 127 || ip.octets()[0] >= 224 {
                return Err(format!(
                    "Host row {} needs a unicast collector address.",
                    n + 1
                ));
            }
            if !hosts.insert(ip) {
                return Err(format!(
                    "Host row {} repeats an address already in this editor.",
                    n + 1
                ));
            }
        }
        Ok(core::DevicePolicy {
            process_enabled: self.enabled,
            buffer_threshold: self.buffer,
            remote_threshold: self.remote,
            export_facility: self.facility,
            hosts,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectorDraft {
    pub facility: Option<core::Facility>,
    pub threshold: core::Severity,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorDraft {
    #[serde(deserialize_with = "bounded::sequence::<_,SelectorDraft,24>")]
    pub rows: Vec<SelectorDraft>,
}
impl CollectorDraft {
    pub fn from_policy(p: &core::CollectorPolicy) -> Result<Self> {
        if p.selectors.len() > 24 {
            return Err("Invalid bounded collector policy before copying editor fields".into());
        }
        Ok(Self {
            rows: p
                .selectors
                .iter()
                .map(|(f, s)| SelectorDraft {
                    facility: Some(*f),
                    threshold: *s,
                })
                .collect(),
        })
    }
    pub fn policy(&self) -> Result<core::CollectorPolicy> {
        if self.rows.len() > 24 {
            return Err("At most 24 facility selectors are supported".into());
        }
        let mut selectors = BTreeMap::new();
        for (n, r) in self.rows.iter().enumerate() {
            let f = r.facility.ok_or_else(|| {
                format!(
                    "Selector row {} needs a facility or must be removed.",
                    n + 1
                )
            })?;
            if selectors.insert(f, r.threshold).is_some() {
                return Err(format!(
                    "Selector row {} repeats a facility. Keep one threshold per facility.",
                    n + 1
                ));
            }
        }
        Ok(core::CollectorPolicy { selectors })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Editor {
    pub selected: Component,
    pub event: core::EventId,
    #[serde(deserialize_with = "source_drafts")]
    pub sources: BTreeMap<Ipv4Addr, SourceDraft>,
    #[serde(deserialize_with = "collector_drafts")]
    pub collectors: BTreeMap<Ipv4Addr, CollectorDraft>,
}
impl Editor {
    fn new(s: &core::State) -> Result<Self> {
        let e = s.environment().map_err(|e| e.to_string())?;
        Ok(Self {
            selected: Component::Source(*e.devices.keys().next().ok_or("Missing source")?),
            event: e.events.keys().next().ok_or("Missing event")?.clone(),
            sources: e
                .devices
                .keys()
                .map(|ip| {
                    Ok((
                        *ip,
                        SourceDraft::from_policy(s.device_policy(*ip).map_err(|e| e.to_string())?)?,
                    ))
                })
                .collect::<Result<_>>()?,
            collectors: e
                .collectors
                .keys()
                .map(|ip| {
                    Ok((
                        *ip,
                        CollectorDraft::from_policy(
                            s.collector_policy(*ip).map_err(|e| e.to_string())?,
                        )?,
                    ))
                })
                .collect::<Result<_>>()?,
        })
    }
    fn validate(&self, s: &core::State) -> Result<()> {
        let e = s.environment().map_err(|e| e.to_string())?;
        let selected = match self.selected {
            Component::Source(ip) => e.devices.contains_key(&ip),
            Component::Collector(ip) => e.collectors.contains_key(&ip),
        };
        if !selected
            || !e.events.contains_key(&self.event)
            || !self.sources.keys().eq(e.devices.keys())
            || !self.collectors.keys().eq(e.collectors.keys())
            || self.sources.values().any(|d| !d.shape())
            || self.collectors.values().any(|d| d.rows.len() > 24)
            || !bounded::within(self, 32 * 1024)
        {
            return Err("Saved syslog editor does not match its bounded supplied inventory".into());
        }
        Ok(())
    }
    fn pending(&self, s: &core::State) -> bool {
        self.sources.iter().any(|(ip, d)| {
            !d.policy()
                .is_ok_and(|p| s.device_policy(*ip).is_ok_and(|q| *q == p))
        }) || self.collectors.iter().any(|(ip, d)| {
            !d.policy()
                .is_ok_and(|p| s.collector_policy(*ip).is_ok_and(|q| *q == p))
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub current: core::State,
    pub example: core::State,
    pub editor: Editor,
    pub example_editor: Editor,
    pub view: View,
    pub checked: bool,
    pub example_checked: bool,
    pub selected_example: usize,
}
fn valid_targets(s: &core::State, t: &[Target], ps: &[Ipv4Addr], pc: &[Ipv4Addr]) -> bool {
    let Ok(e) = s.environment() else {
        return false;
    };
    if t.is_empty()
        || t.len() > 32
        || ps.len() > 8
        || pc.len() > 8
        || t.iter()
            .map(|r| usize::from(r.local.is_some()) + r.remote.len())
            .sum::<usize>()
            + ps.len()
            + pc.len()
            > MAX_ROWS
    {
        return false;
    }
    let mut pairs = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut collectors = BTreeSet::new();
    for r in t {
        if !e.devices.contains_key(&r.device)
            || !e.events.contains_key(&r.event)
            || !pairs.insert((r.device, &r.event))
            || (r.local.is_none() && r.remote.is_empty())
            || r.remote.len() > 8
        {
            return false;
        }
        sources.insert(r.device);
        for (ip, w) in &r.remote {
            if !e.collectors.contains_key(ip)
                || w.facilities.len() > 24
                || w.facilities.iter().copied().collect::<BTreeSet<_>>().len() != w.facilities.len()
                || w.accept == w.facilities.is_empty()
            {
                return false;
            }
            collectors.insert(*ip);
        }
    }
    let mut a = BTreeSet::new();
    let mut b = BTreeSet::new();
    if ps
        .iter()
        .any(|ip| !e.devices.contains_key(ip) || !a.insert(*ip))
        || pc
            .iter()
            .any(|ip| !e.collectors.contains_key(ip) || !b.insert(*ip))
    {
        return false;
    }
    sources.extend(a);
    collectors.extend(b);
    sources.len() == e.devices.len() && collectors.len() == e.collectors.len()
}
fn outcomes(
    initial: &core::State,
    current: &core::State,
    t: &[Target],
    ps: &[Ipv4Addr],
    pc: &[Ipv4Addr],
) -> Vec<(bool, String)> {
    let fail = |e: String| vec![(false, e)];
    if !valid_targets(initial, t, ps, pc) {
        return fail("Invalid authored syslog requirements".into());
    }
    match current.environment_matches(match initial.environment() {
        Ok(e) => e,
        Err(e) => return fail(e.to_string()),
    }) {
        Ok(true) => (),
        _ => return fail("Supplied syslog event/delivery inventory changed; work retained".into()),
    }
    let mut out = Vec::new();
    for r in t {
        let p = match current.current_probe(r.device, &r.event) {
            Ok(p) => p,
            Err(e) => return fail(e.to_string()),
        };
        if let Some(want) = r.local {
            out.push((
                (p.local == core::BufferDisposition::Accepted) == want,
                format!(
                    "{} / {}: {}. Required: {}.",
                    r.device,
                    r.event.as_str(),
                    p.local.explanation(),
                    if want {
                        "accept in the local buffer"
                    } else {
                        "filter out of the local buffer"
                    }
                ),
            ));
        }
        for (ip, w) in &r.remote {
            let actual = match current.current_remote(r.device, &r.event, *ip) {
                Ok(a) => a,
                Err(e) => return fail(e.to_string()),
            };
            let pass = match actual {
                core::RemoteDisposition::Accepted { facility } => {
                    w.accept && w.facilities.contains(&facility)
                }
                _ => !w.accept,
            };
            out.push((
                pass,
                format!(
                    "{} / {} to {}: {}. Required: {}.",
                    r.device,
                    r.event.as_str(),
                    ip,
                    actual.explanation(),
                    if w.accept {
                        format!(
                            "accept using facility {}",
                            w.facilities
                                .iter()
                                .map(|f| f.number().to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    } else {
                        "do not deliver this event".into()
                    }
                ),
            ));
        }
    }
    for ip in ps {
        out.push((
            matches!((current.device_policy(*ip),initial.device_policy(*ip)),(Ok(a),Ok(b))if a==b),
            format!("Keep supplied source policy {ip} unchanged."),
        ));
    }
    for ip in pc {
        out.push((
            matches!((current.collector_policy(*ip),initial.collector_policy(*ip)),(Ok(a),Ok(b))if a==b),
            format!("Keep supplied collector policy {ip} unchanged."),
        ));
    }
    out
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
fn action_shape(a: &Action, s: &core::State) -> bool {
    if !bounded::within(a, MAX_ACTION) {
        return false;
    }
    let Ok(e) = s.environment() else {
        return false;
    };
    match a {
        Action::ApplySource { device, policy } => {
            e.devices.contains_key(device)
                && policy.hosts.len() <= 8
                && policy
                    .hosts
                    .iter()
                    .all(|ip| ip.octets()[0] != 0 && ip.octets()[0] != 127 && ip.octets()[0] < 224)
        }
        Action::ApplyCollector { collector, policy } => {
            e.collectors.contains_key(collector) && policy.selectors.len() <= 24
        }
        Action::Generate { device, event } => {
            e.devices.contains_key(device) && e.events.contains_key(event)
        }
        Action::ClearLocal { device } => e.devices.contains_key(device),
        Action::ClearCollector { collector } => e.collectors.contains_key(collector),
    }
}
fn apply_core(s: &mut core::State, a: &Action) -> Result<String> {
    match a {
        Action::ApplySource { device, policy } => s
            .set_device_policy(*device, policy.clone())
            .map(|_| "Source settings applied.".into()),
        Action::ApplyCollector { collector, policy } => s
            .set_collector_policy(*collector, policy.clone())
            .map(|_| "Collector settings applied.".into()),
        Action::Generate { device, event } => s.emit(*device, event).map(|p| {
            format!(
                "Simulated event generated. {}. Inspect historical logs for accepted deliveries.",
                p.local.explanation()
            )
        }),
        Action::ClearLocal { device } => s
            .clear_buffer(*device)
            .map(|_| "Selected historical local buffer cleared; policy unchanged.".into()),
        Action::ClearCollector { collector } => s
            .clear_collector_history(*collector)
            .map(|_| "Selected historical collector inbox cleared; policy unchanged.".into()),
    }
    .map_err(|e| e.to_string())
}
impl Definition {
    pub fn shape(&self) -> Result<()> {
        self.initial.validate().map_err(|e| e.to_string())?;
        self.lesson.fixture.validate().map_err(|e| e.to_string())?;
        let l = &self.lesson;
        let e = self.initial.environment().map_err(|e| e.to_string())?;
        let p = l.fixture.environment().map_err(|e| e.to_string())?;
        if self.version != 1
            || !valid_targets(
                &self.initial,
                &self.targets,
                &self.protected_sources,
                &self.protected_collectors,
            )
            || !valid_targets(
                &l.fixture,
                &l.targets,
                &l.protected_sources,
                &l.protected_collectors,
            )
            || e.devices
                .keys()
                .chain(e.collectors.keys())
                .any(|ip| p.devices.contains_key(ip) || p.collectors.contains_key(ip))
            || e.events.keys().any(|id| p.events.contains_key(id))
            || l.concepts.is_empty()
            || l.concepts.len() > 6
            || l.concepts.iter().any(|c| {
                c.title.is_empty()
                    || c.title.len() > 100
                    || c.text.is_empty()
                    || c.text.len() > 1200
            })
            || [&l.prerequisite_note, &l.description, &l.try_change]
                .iter()
                .any(|s| s.is_empty() || s.len() > 1600)
            || l.worked.is_empty()
            || l.worked.len() > 12
            || l.worked.iter().any(|w| {
                w.explanation.is_empty()
                    || w.explanation.len() > 800
                    || !action_shape(&w.action, &l.fixture)
            })
            || !action_shape(&l.experiment, &l.fixture)
            || !action_shape(&l.repair, &l.fixture)
            || !bounded::within(self, MAX_DEFINITION)
        {
            return Err("Invalid or nonseparate bounded syslog definition".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.shape()?;
        if all(&outcomes(
            &self.initial,
            &self.initial,
            &self.targets,
            &self.protected_sources,
            &self.protected_collectors,
        )) {
            return Err("Assignment must require a policy change".into());
        }
        let l = &self.lesson;
        let mut p = l.fixture.clone();
        if all(&outcomes(
            &l.fixture,
            &p,
            &l.targets,
            &l.protected_sources,
            &l.protected_collectors,
        )) {
            return Err("Separate practice must require a change".into());
        }
        for w in &l.worked {
            apply_core(&mut p, &w.action)?;
        }
        if !all(&outcomes(
            &l.fixture,
            &p,
            &l.targets,
            &l.protected_sources,
            &l.protected_collectors,
        )) {
            return Err("Worked practice does not reach its outcomes".into());
        }
        let before = p.clone();
        apply_core(&mut p, &l.experiment)?;
        if p == before
            || all(&outcomes(
                &l.fixture,
                &p,
                &l.targets,
                &l.protected_sources,
                &l.protected_collectors,
            ))
        {
            return Err("Practice change must expose a policy mistake".into());
        }
        apply_core(&mut p, &l.repair)?;
        if !all(&outcomes(
            &l.fixture,
            &p,
            &l.targets,
            &l.protected_sources,
            &l.protected_collectors,
        )) {
            return Err("Practice repair must restore its outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.shape()?;
        s.validate()?;
        if !s
            .current
            .environment_matches(self.initial.environment().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
            || !s
                .example
                .environment_matches(
                    self.lesson
                        .fixture
                        .environment()
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?
            || s.selected_example >= self.lesson.worked.len()
        {
            return Err("Saved syslog work belongs to another supplied task".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        self.rows(s, false)
    }
    pub fn practice_checks(&self, s: &Session) -> Vec<(bool, String)> {
        self.rows(s, true)
    }
    fn rows(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (world, editor) = s.scope(practice);
        let (initial, t, ps, pc) = if practice {
            (
                &self.lesson.fixture,
                &self.lesson.targets,
                &self.lesson.protected_sources,
                &self.lesson.protected_collectors,
            )
        } else {
            (
                &self.initial,
                &self.targets,
                &self.protected_sources,
                &self.protected_collectors,
            )
        };
        let mut rows = vec![(
            !editor.pending(world),
            if editor.pending(world) {
                "Unapplied changes: apply or repair pending settings before checking the complete configuration.".into()
            } else {
                "All editor settings match applied policy.".into()
            },
        )];
        rows.extend(outcomes(initial, world, t, ps, pc));
        if rows.iter().map(|r| r.1.len()).sum::<usize>() > core::MAX_OUTPUT {
            return vec![(
                false,
                "This task exceeds the supported feedback limit; no success is awarded.".into(),
            )];
        }
        rows
    }
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.shape()?;
        Ok(Self {
            version: 1,
            current: d.initial.clone(),
            example: d.lesson.fixture.clone(),
            editor: {
                let mut e = Editor::new(&d.initial)?;
                e.event = d.targets[0].event.clone();
                e
            },
            example_editor: {
                let mut e = Editor::new(&d.lesson.fixture)?;
                e.event = d.lesson.targets[0].event.clone();
                e
            },
            view: View::Introduction,
            checked: false,
            example_checked: false,
            selected_example: 0,
        })
    }
    pub fn validate(&self) -> Result<()> {
        self.current.validate().map_err(|e| e.to_string())?;
        self.example.validate().map_err(|e| e.to_string())?;
        self.editor.validate(&self.current)?;
        self.example_editor.validate(&self.example)?;
        if self.version != 1 || self.selected_example >= 12 || !bounded::within(self, MAX_SESSION) {
            return Err("Invalid bounded syslog session; original retained".into());
        }
        Ok(())
    }
    fn scope(&self, p: bool) -> (&core::State, &Editor) {
        if p {
            (&self.example, &self.example_editor)
        } else {
            (&self.current, &self.editor)
        }
    }
    pub fn inspect(&self, d: &Definition, p: bool) -> Result<(&core::State, &Editor)> {
        d.bound(self)?;
        Ok(self.scope(p))
    }
    pub fn has_unapplied(&self, d: &Definition, p: bool) -> Result<bool> {
        d.bound(self)?;
        let (world, editor) = self.scope(p);
        Ok(editor.pending(world))
    }
    fn scope_mut(&mut self, p: bool) -> (&mut core::State, &mut Editor, &mut bool) {
        if p {
            (
                &mut self.example,
                &mut self.example_editor,
                &mut self.example_checked,
            )
        } else {
            (&mut self.current, &mut self.editor, &mut self.checked)
        }
    }
    fn change(&mut self, d: &Definition, f: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        d.bound(self)?;
        let mut n = self.clone();
        f(&mut n)?;
        d.bound(&n)?;
        *self = n;
        Ok(())
    }
    pub fn select(&mut self, d: &Definition, p: bool, c: Component) -> Result<()> {
        self.change(d, |n| {
            n.scope_mut(p).1.selected = c;
            Ok(())
        })
    }
    pub fn select_event(&mut self, d: &Definition, p: bool, e: core::EventId) -> Result<()> {
        self.change(d, |n| {
            n.scope_mut(p).1.event = e;
            Ok(())
        })
    }
    pub fn set_view(&mut self, d: &Definition, v: View) -> Result<()> {
        self.change(d, |n| {
            n.view = v;
            Ok(())
        })
    }
    pub fn set_source_draft(
        &mut self,
        d: &Definition,
        p: bool,
        ip: Ipv4Addr,
        draft: SourceDraft,
    ) -> Result<()> {
        if !draft.shape() {
            return Err("Address edit refused; the complete previous draft was retained".into());
        }
        self.change(d, |n| {
            let (_, e, c) = n.scope_mut(p);
            *e.sources.get_mut(&ip).ok_or("Unknown source editor")? = draft;
            *c = false;
            Ok(())
        })
    }
    pub fn set_collector_draft(
        &mut self,
        d: &Definition,
        p: bool,
        ip: Ipv4Addr,
        draft: CollectorDraft,
    ) -> Result<()> {
        if draft.rows.len() > 24 {
            return Err("At most 24 selector rows".into());
        }
        self.change(d, |n| {
            let (_, e, c) = n.scope_mut(p);
            *e.collectors
                .get_mut(&ip)
                .ok_or("Unknown collector editor")? = draft;
            *c = false;
            Ok(())
        })
    }
    pub fn apply_selected(&mut self, d: &Definition, p: bool) -> Result<String> {
        d.bound(self)?;
        let (_, e) = self.scope(p);
        let action = match e.selected {
            Component::Source(ip) => Action::ApplySource {
                device: ip,
                policy: e.sources[&ip].policy()?,
            },
            Component::Collector(ip) => Action::ApplyCollector {
                collector: ip,
                policy: e.collectors[&ip].policy()?,
            },
        };
        self.action(d, p, &action)
    }
    pub fn action(&mut self, d: &Definition, p: bool, a: &Action) -> Result<String> {
        d.bound(self)?;
        if !action_shape(a, self.scope(p).0) {
            return Err("Action is outside this supplied syslog workspace".into());
        }
        let mut result = String::new();
        self.change(d, |n| {
            let (world, editor, checked) = n.scope_mut(p);
            result = apply_core(world, a)?;
            match a {
                Action::ApplySource { device, policy } => {
                    editor
                        .sources
                        .insert(*device, SourceDraft::from_policy(policy)?);
                    *checked = false;
                }
                Action::ApplyCollector { collector, policy } => {
                    editor
                        .collectors
                        .insert(*collector, CollectorDraft::from_policy(policy)?);
                    *checked = false;
                }
                _ => (),
            }
            Ok(())
        })?;
        Ok(result)
    }
    pub fn generate(&mut self, d: &Definition, p: bool) -> Result<String> {
        let (_, e) = self.scope(p);
        let device = match e.selected {
            Component::Source(ip) => ip,
            _ => return Err("Select a source to generate its simulated event".into()),
        };
        self.action(
            d,
            p,
            &Action::Generate {
                device,
                event: e.event.clone(),
            },
        )
    }
    pub fn clear_selected(&mut self, d: &Definition, p: bool) -> Result<String> {
        let a = match self.scope(p).1.selected {
            Component::Source(device) => Action::ClearLocal { device },
            Component::Collector(collector) => Action::ClearCollector { collector },
        };
        self.action(d, p, &a)
    }
    pub fn check(&mut self, d: &Definition, p: bool) -> Result<()> {
        self.change(d, |n| {
            *n.scope_mut(p).2 = true;
            Ok(())
        })
    }
    pub fn fill_worked(&mut self, d: &Definition, index: usize) -> Result<String> {
        d.bound(self)?;
        let a = d
            .lesson
            .worked
            .get(index)
            .ok_or("Unknown worked example")?
            .action
            .clone();
        self.change(d, |n| {
            n.selected_example = index;
            let (_, e, c) = n.scope_mut(true);
            *c = false;
            match &a {
                Action::ApplySource { device, policy } => {
                    e.selected = Component::Source(*device);
                    e.sources.insert(*device, SourceDraft::from_policy(policy)?);
                }
                Action::ApplyCollector { collector, policy } => {
                    e.selected = Component::Collector(*collector);
                    e.collectors
                        .insert(*collector, CollectorDraft::from_policy(policy)?);
                }
                Action::Generate { device, event } => {
                    e.selected = Component::Source(*device);
                    e.event = event.clone();
                }
                Action::ClearLocal { device } => e.selected = Component::Source(*device),
                Action::ClearCollector { collector } => {
                    e.selected = Component::Collector(*collector)
                }
            }
            Ok(())
        })?;
        Ok(match a {
            Action::ApplySource { .. } | Action::ApplyCollector { .. } => {
                "Worked example settings filled. Apply settings when you are ready."
            }
            Action::Generate { .. } => {
                "Worked event selected. Generate simulated event when you are ready."
            }
            _ => "Worked log selected. Clear history only if you choose.",
        }
        .into())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<()> {
        self.change(d, |n| {
            n.example = d.lesson.fixture.clone();
            n.example_editor = Editor::new(&d.lesson.fixture)?;
            n.example_editor.event = d.lesson.targets[0].event.clone();
            n.example_checked = false;
            n.selected_example = 0;
            Ok(())
        })
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("Syslog session exceeds512KiB before decoding".into());
        }
        let s: Self =
            serde_json::from_slice(raw).map_err(|e| format!("Unreadable syslog session: {e}"))?;
        s.validate()?;
        Ok(s)
    }
}
/// Internal author reachability data only; never a learner-facing IOS command.
pub fn execute_authored(s: &mut Session, d: &Definition, input: &str) -> Result<String> {
    if input.len() > MAX_ACTION {
        return Err("Authored syslog action exceeds2048bytes".into());
    }
    let a: Action = serde_json::from_str(input)
        .map_err(|_| "Unsupported syslog action data; use the native editor and event controls")?;
    s.action(d, false, &a)
}
/// The first sixteen facility labels are RFC identities, not invented IOS keywords.
pub fn facility_label(f: core::Facility) -> String {
    let n = f.number();
    let name = match n {
        0 => "kernel",
        1 => "user messages",
        2 => "mail",
        3 => "system daemons",
        4 | 10 => "security/authorization",
        5 => "syslog-internal",
        6 => "line printer",
        7 => "network news",
        8 => "UUCP",
        9 | 15 => "clock daemon",
        11 => "FTP",
        12 => "NTP",
        13 => "log audit",
        14 => "log alert",
        _ => return format!("{n} · local{}", n - 16),
    };
    format!("{n} · {name}")
}

pub fn metadata(a: &crate::progress::Attempt) -> Result<()> {
    if !matches!(a.state, crate::engine::State::Syslog { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() {
        return Err("Addressing metadata on Syslog work".into());
    }
    if let Some(w) = &a.workspace
        && !foreign_empty(w)
    {
        return Err(
            "Saved syslog work contains data from another activity; original retained".into(),
        );
    }
    Ok(())
}
fn foreign_empty(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        one_off_session: _,
        one_off_learning: _,
        physical_link_draft: _,
        physical_link_learning: _,
        file_backup_learning: _,
        etherchannel_draft: _,
        etherchannel_learning: _,
        cron_session: _,
        cron_learning: _,
        nfs_session: _,
        nfs_learning: _,
        nat_learning: _,
        rpm_session: _,
        rpm_learning: _,
        nat_draft: _,
        debian_packages_terminal: _,
        apache_static_draft: _,
        apache_static_learning: _,
        debian_packages_draft: _,
        debian_packages_learning: _,
        spanning_tree_draft: _,
        spanning_tree_learning: _,
        dhcpv4_draft: _,
        dhcpv4_learning: _,
        version: _,
        unit_manager_draft: _,
        unit_manager_learning: _,
        unit_manager_terminal: _,
        ospf_draft: _,
        ospf_learning: _,
        acl_draft: _,
        acl_learning: _,
        script_learning: _,
        script_history: _,
        script_history_more: _,
        terminal_learning: _,
        process_learning: _,
        accounts_learning: _,
        accounts_template_drafts: _,
        discovery_draft: _,
        discovery_learning: _,
        storage_draft: _,
        storage_learning: _,
        ipv6_draft: _,
        ipv6_learning: _,
        routing_v4_draft: _,
        routing_v4_learning: _,
        address_learning: _,
        service_learning: _,
        client_learning: _,
        seed_dns_learning: _,
        seed_mount_learning: _,
        legacy_access_learning: _,
        legacy_logs_learning: _,
        client_draft: _,
        dns_learning: _,
        dns_draft: _,
        switching_draft: _,
        switching_learning: _,
        mac_draft: _,
        mac_learning: _,
        command: _,
        service_port: _,
        dns_address: _,
        dns_serial: _,
        mount_device: _,
        checked: _,
        ipv4_check: _,
        practice_check: _,
        migrated_host: _,
        legacy_host_reviewed: _,
        legacy_practice_reviewed: _,
        migrated_practice_host: _,
    } = w;
    w.one_off_session.is_none()
        && w.one_off_learning.is_none()
        && w.physical_link_draft.is_none()
        && w.physical_link_learning.is_none()
        && w.file_backup_learning.is_none()
        && w.etherchannel_draft.is_none()
        && w.etherchannel_learning.is_none()
        && w.cron_session.is_none()
        && w.cron_learning.is_none()
        && w.nfs_session.is_none()
        && w.nfs_learning.is_none()
        && w.nat_learning.is_none()
        && w.rpm_session.is_none()
        && w.rpm_learning.is_none()
        && w.nat_draft.is_none()
        && w.debian_packages_terminal.is_none()
        && w.apache_static_draft.is_none()
        && w.apache_static_learning.is_none()
        && w.debian_packages_draft.is_none()
        && w.debian_packages_learning.is_none()
        && w.spanning_tree_draft.is_none()
        && w.spanning_tree_learning.is_none()
        && w.dhcpv4_draft.is_none()
        && w.dhcpv4_learning.is_none()
        && w.version == 1
        && w.unit_manager_draft.is_none()
        && w.unit_manager_learning.is_none()
        && w.unit_manager_terminal.is_none()
        && w.ospf_draft.is_none()
        && w.ospf_learning.is_none()
        && w.acl_draft.is_none()
        && w.acl_learning.is_none()
        && w.script_learning.is_none()
        && w.script_history.is_none()
        && w.script_history_more.is_empty()
        && w.terminal_learning.is_none()
        && w.process_learning.is_none()
        && w.accounts_learning.is_none()
        && w.accounts_template_drafts.is_none()
        && w.discovery_draft.is_none()
        && w.discovery_learning.is_none()
        && w.storage_draft.is_none()
        && w.storage_learning.is_none()
        && w.ipv6_draft.is_none()
        && w.ipv6_learning.is_none()
        && w.routing_v4_draft.is_none()
        && w.routing_v4_learning.is_none()
        && w.address_learning.is_none()
        && w.service_learning.is_none()
        && w.client_learning.is_none()
        && w.seed_dns_learning.is_none()
        && w.seed_mount_learning.is_none()
        && w.legacy_access_learning.is_none()
        && w.legacy_logs_learning.is_none()
        && w.client_draft.is_none()
        && w.dns_learning.is_none()
        && w.dns_draft.is_none()
        && w.switching_draft.is_none()
        && w.switching_learning.is_none()
        && w.mac_draft.is_none()
        && w.mac_learning.is_none()
        && w.command.is_empty()
        && w.service_port.is_none()
        && w.dns_address.is_none()
        && w.dns_serial.is_none()
        && w.mount_device.is_none()
        && !w.checked
        && w.ipv4_check.is_none()
        && w.practice_check.is_none()
        && !w.migrated_host
        && !w.legacy_host_reviewed
        && !w.legacy_practice_reviewed
        && !w.migrated_practice_host
}

pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    metadata(a)?;
    source(s)?;
    let (crate::engine::State::Syslog { simulation }, crate::engine::Goal::Syslog { definition }) =
        (&a.state, &s.goal)
    else {
        return Err("Saved syslog work does not match this task".into());
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.transcript
            .iter()
            .any(|t| t.len() > 8192 || t.contains('\0'))
        || !bounded::within(&(s, a), MAX_CONTEXT)
    {
        return Err(
            "Saved syslog work is too large or does not match this task; original retained".into(),
        );
    }
    definition.bound(simulation)
}
pub fn source(s: &crate::pack::Scenario) -> Result<()> {
    let (crate::engine::State::Syslog { simulation }, crate::engine::Goal::Syslog { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Syslog source/Goal kinds differ".into());
    };
    definition.shape()?;
    simulation.validate()?;
    let Session {
        version: _,
        current: _,
        example: _,
        editor: _,
        example_editor: _,
        view: _,
        checked: _,
        example_checked: _,
        selected_example: _,
    } = simulation.as_ref();
    let mut editor = Editor::new(&definition.initial)?;
    editor.event = definition.targets[0].event.clone();
    let mut example_editor = Editor::new(&definition.lesson.fixture)?;
    example_editor.event = definition.lesson.targets[0].event.clone();
    if simulation.version != 1
        || simulation.current != definition.initial
        || simulation.example != definition.lesson.fixture
        || simulation.editor != editor
        || simulation.example_editor != example_editor
        || simulation.view != View::Introduction
        || simulation.checked
        || simulation.example_checked
        || simulation.selected_example != 0
        || !bounded::within(s, MAX_CONTEXT)
        || !binding::fresh_context(s, simulation)
    {
        return Err(
            "Authored Syslog initial must be a fresh assignment and separate practice".into(),
        );
    }
    let crate::pack::Scenario {
        one_off_teaching: _,
        physical_link_teaching: _,
        file_backup_teaching: _,
        etherchannel_teaching: _,
        cron_teaching: _,
        nfs_teaching: _,
        nat_teaching: _,
        rpm_packages_teaching: _,
        apache_static_teaching: _,
        spanning_tree_teaching: _,
        dhcpv4_teaching: _,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
        teaching: _,
        terminal_teaching: _,
        process_teaching: _,
        accounts_teaching: _,
        unit_manager_teaching: _,
        ospf_teaching: _,
        discovery_teaching: _,
        storage_teaching: _,
        acl_teaching: _,
        ipv6_teaching: _,
        ipv4_routing_teaching: _,
        address_teaching: _,
        service_teaching: _,
        client_teaching: _,
        seed_dns_teaching: _,
        seed_mount_teaching: _,
        legacy_access_teaching: _,
        legacy_logs_teaching: _,
        dns_teaching: _,
        switching_teaching: _,
        mac_teaching: _,
        debian_packages_teaching: _,
    } = s;
    if s.one_off_teaching.is_some()
        || s.physical_link_teaching.is_some()
        || s.file_backup_teaching.is_some()
        || s.etherchannel_teaching.is_some()
        || s.cron_teaching.is_some()
        || s.nfs_teaching.is_some()
        || s.nat_teaching.is_some()
        || s.rpm_packages_teaching.is_some()
        || s.apache_static_teaching.is_some()
        || s.spanning_tree_teaching.is_some()
        || s.dhcpv4_teaching.is_some()
        || s.teaching.is_some()
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
    {
        return Err("Foreign teaching metadata on Syslog source".into());
    }
    Ok(())
}
