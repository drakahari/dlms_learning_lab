//! Bounded source-bound profile editor/checking state. No host execution.
use crate::{
    ipv4_routing::{Assignment, Prefix},
    nm_policy as model, nm_profiles as keyfile,
    script_teaching::View,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    net::Ipv4Addr,
};
pub const MAX_DEFINITION: usize = 1_048_576;
pub const MAX_SESSION: usize = 1_100_000;
pub const MAX_TICKS: usize = 64;
pub const PREFIX: &str = "NM PROFILES\n";
pub type Files = BTreeMap<String, String>;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text_bound(s: &str) -> bool {
    s.len() <= keyfile::MAX_TEXT
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn document<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    struct Text;
    impl Visitor<'_> for Text {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most16KiB of configuration text")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > keyfile::MAX_TEXT {
                Err(E::custom("Profile draft byte limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text)
}
fn files<'de, D: Deserializer<'de>>(d: D) -> Result<Files, D::Error> {
    struct Bundle;
    impl<'de> Visitor<'de> for Bundle {
        type Value = Files;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most8bounded virtual profile files")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Files, A::Error> {
            #[derive(Deserialize)]
            struct Name(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
            #[derive(Deserialize)]
            struct Body(#[serde(deserialize_with = "document")] String);
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Virtual profile file-count limit"))
                }
            }
            let mut out = Files::new();
            while out.len() < keyfile::MAX_PROFILES {
                let Some(Name(name)) = a.next_key()? else {
                    return Ok(out);
                };
                if out.contains_key(&name) {
                    return Err(A::Error::custom("Duplicate virtual filename"));
                }
                let Body(body) = a.next_value()?;
                out.insert(name, body);
            }
            let _ = a.next_key::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_map(Bundle)
}
fn optional_files<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Files>, D::Error> {
    struct Maybe;
    impl<'de> Visitor<'de> for Maybe {
        type Value = Option<Files>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("optional checked virtual files")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            files(d).map(Some)
        }
    }
    d.deserialize_option(Maybe)
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N}bounded items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<T>, A::Error> {
            let mut out = vec![];
            while out.len() < N {
                match a.next_element()? {
                    Some(x) => out.push(x),
                    None => return Ok(out),
                }
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Profile exercise item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn identities<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Identity(#[serde(deserialize_with = "crate::cron::text::<_,36>")] String);
    list::<D, Identity, 2>(d).map(|v| v.into_iter().map(|x| x.0).collect())
}
fn protected_profiles<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    list::<D, Name, 8>(d).map(|v| v.into_iter().map(|n| n.0).collect())
}
fn fits<T: Serialize>(value: &T, max: usize) -> bool {
    struct Count(usize, usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Profile exercise byte limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0, max), value).is_ok()
}
fn shape(f: &Files) -> bool {
    !f.is_empty()
        && f.len() <= 8
        && f.iter().all(|(name, text)| {
            !name.starts_with('.')
                && name.len() <= 64
                && name.ends_with(".nmconnection")
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
                && text_bound(text)
        })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    CurrentUnchanged {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
    },
    FutureProfile {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
        #[serde(deserialize_with = "crate::cron::text::<_,36>")]
        uuid: String,
    },
    FutureDhcp {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
    },
    FutureAddress {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
        address: Assignment,
    },
    FutureNetwork {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
        network: Prefix,
    },
    FutureBridgeDelivery {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
        from: Ipv4Addr,
        to: Ipv4Addr,
        delivered: bool,
    },
    FuturePath {
        target: Ipv4Addr,
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
    },
    FutureNoDefault {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
    },
    DefaultOwner {
        #[serde(deserialize_with = "crate::cron::text::<_,15>")]
        interface: String,
    },
}
impl Requirement {
    fn interface(&self) -> &str {
        match self {
            Self::CurrentUnchanged { interface }
            | Self::FutureProfile { interface, .. }
            | Self::FutureDhcp { interface }
            | Self::FutureAddress { interface, .. }
            | Self::FutureNetwork { interface, .. }
            | Self::FutureBridgeDelivery { interface, .. }
            | Self::FuturePath { interface, .. }
            | Self::FutureNoDefault { interface }
            | Self::DefaultOwner { interface } => interface,
        }
    }
    fn identity(&self) -> String {
        match self {
            Self::CurrentUnchanged { interface } => format!("current-{interface}"),
            Self::FutureProfile { interface, .. } => format!("profile-{interface}"),
            Self::FutureAddress { interface, .. } => format!("address-{interface}"),
            Self::FutureDhcp { interface } => format!("dhcp-{interface}"),
            Self::FutureNetwork { interface, .. } => format!("network-{interface}"),
            Self::FutureBridgeDelivery {
                interface,
                from,
                to,
                ..
            } => format!("bridge-{interface}-{from}-{to}"),
            Self::FuturePath { target, .. } => format!("path-{target}"),
            Self::DefaultOwner { .. } => "default".into(),
            Self::FutureNoDefault { interface } => format!("no-default-{interface}"),
        }
    }
    fn check(
        &self,
        case: &Case,
        initial: &model::Choices,
        warm: &model::Choices,
        future: &model::Choices,
    ) -> (bool, String) {
        match self {
            Self::CurrentUnchanged { interface } => {
                let before = initial.for_device(interface);
                let after = warm.for_device(interface);
                let preserved = before == after
                    && initial.controls() == warm.controls()
                    && initial.bridge_controls() == warm.bridge_controls();
                (
                    before.is_some() && preserved,
                    format!(
                        "{interface}: re-reading saved files {} the supplied current settings. Current selections are separate from edited profiles.",
                        if preserved { "preserves" } else { "changed" }
                    ),
                )
            }
            Self::FutureProfile { interface, uuid } => {
                let got = future.for_device(interface);
                (
                    got.is_some_and(|p| p.uuid == *uuid),
                    format!(
                        "{interface}: next startup needs profile {uuid}; actual {}. Check interface binding, autoconnect and its preference.",
                        got.map(|p| format!("{} ({})", p.id, p.uuid))
                            .unwrap_or_else(|| "no eligible profile".into())
                    ),
                )
            }
            Self::FutureDhcp { interface } => {
                let used = future.uses_dhcp(interface);
                (
                    used,
                    format!(
                        "{interface}: next startup needs an address from the supplied DHCP outcome; actual {}. A saved manual address, even one matching a sample, is not automatic configuration.",
                        if used {
                            "DHCP-derived settings"
                        } else if future.for_device(interface).is_some() {
                            "manual settings"
                        } else {
                            "no active address; inspect the supplied lease and saved profile"
                        }
                    ),
                )
            }
            Self::FutureAddress { interface, address } => {
                let got = future.for_device(interface).map(|p| &p.address);
                (
                    got == Some(address),
                    format!(
                        "{interface}: next startup needs {}/{}; actual {}. Check the saved address, interface binding and whether the intended profile is eligible for autoconnect.",
                        address.address,
                        address.prefix,
                        got.map(|a| format!("{}/{}", a.address, a.prefix))
                            .unwrap_or_else(|| "no configured address".into())
                    ),
                )
            }
            Self::FutureNetwork { interface, network } => {
                let got = future
                    .for_device(interface)
                    .and_then(|p| Prefix::new(p.address.address, p.address.prefix));
                (
                    got == Some(*network),
                    format!(
                        "{interface}: use a usable host address in {} with that prefix; actual {}. A different host address can be valid, but the address/mask must identify the requested LAN.",
                        network.text(),
                        got.map(|p| p.text())
                            .unwrap_or_else(|| "no selected IPv4 address".into())
                    ),
                )
            }
            Self::FutureBridgeDelivery {
                from,
                to,
                delivered,
                ..
            } => match future.bridge_peer_delivery(&case.environment, *from, *to) {
                Err(e) => (false, e.to_string()),
                Ok(d) => {
                    let ok = if *delivered {
                        d.round_trip()
                    } else {
                        !d.outward && !d.returning
                    };
                    (
                        ok,
                        format!(
                            "Expected peer path {} in this supplied physical case. {}",
                            if *delivered {
                                "delivered both ways"
                            } else {
                                "blocked both ways"
                            },
                            d.detail
                        ),
                    )
                }
            },
            Self::FuturePath { target, interface } => {
                let p = future.probe(&case.environment, *target);
                let ok = p.pass
                    && p.selection
                        .as_ref()
                        .is_some_and(|s| s.route.device == *interface);
                (
                    ok,
                    format!(
                        "{target}: next startup needs a usable supplied path through {interface}; actual {}. {}",
                        p.selection
                            .as_ref()
                            .map(|s| s.route.device.as_str())
                            .unwrap_or("no route"),
                        p.reason
                    ),
                )
            }
            Self::FutureNoDefault { interface } => match future.route_table(&case.environment) {
                Err(e) => (false, e.to_string()),
                Ok(table) => {
                    let absent = !table
                        .iter()
                        .any(|r| r.device == *interface && r.destination.length == 0);
                    (
                        absent,
                        format!(
                            "{interface}: next startup must keep its local address without adding a default route; actual {}. Raising a metric only makes a default less preferred; it does not remove it.",
                            if absent {
                                "no default route on this adapter"
                            } else {
                                "a default route is still present; inspect never-default"
                            }
                        ),
                    )
                }
            },
            Self::DefaultOwner { interface } => match future.route_table(&case.environment) {
                Err(e) => (false, e.to_string()),
                Ok(table) => {
                    let mut defaults = table
                        .iter()
                        .filter(|r| r.destination.length == 0)
                        .collect::<Vec<_>>();
                    defaults.sort_by_key(|r| r.metric);
                    let chosen = defaults.first();
                    let tied = chosen
                        .is_some_and(|c| defaults.get(1).is_some_and(|d| d.metric == c.metric));
                    (
                        !tied && chosen.is_some_and(|r| r.device == *interface),
                        format!(
                            "Preferred default uplink: expected {interface}; actual {}. Lower route metrics win after destination prefix length.",
                            if tied {
                                "an unsupported metric tie".into()
                            } else {
                                chosen
                                    .map(|r| format!("{} (metric {})", r.device, r.metric))
                                    .unwrap_or_else(|| "none".into())
                            }
                        ),
                    )
                }
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_bridge: Option<crate::bridge_control::Current>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_controls: Option<crate::bond_control::Current>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub environment: model::Environment,
    #[serde(deserialize_with = "files")]
    pub current_files: Files,
    #[serde(deserialize_with = "identities")]
    pub initially_selected: Vec<String>,
    #[serde(deserialize_with = "list::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
impl Case {
    fn initial(&self) -> Result<model::Choices, String> {
        let p = model::Policy::from_texts_for(&self.environment, &self.current_files)
            .map_err(|e| e.to_string())?;
        let mut c = model::Choices::empty();
        match (&self.environment.bond,&self.current_controls) {
            (Some(_),Some(current))=>c=c.seed_current_controls(&self.environment,&p,current).map_err(|e|e.to_string())?,
            (None,None)=>{},
            _=>return Err("Bond cases need explicit supplied current control membership; old unbonded cases must not invent it".into()),
        }
        match (&self.environment.bridge,&self.current_bridge) {
            (Some(facts),Some(current))=>{
                let declared=self.initially_selected.iter().any(|u|u.eq_ignore_ascii_case(&facts.profiles.controller.uuid));
                if declared!=current.controller {return Err("The supplied current bridge selection and explicit current IP UUID must agree".into());}
                c=c.seed_current_bridge(&self.environment,&p,current).map_err(|e|e.to_string())?;
            },
            (None,None)=>{},
            _=>return Err("Bridge cases need explicit source current membership; older contexts must not invent it".into()),
        }
        for uuid in &self.initially_selected {
            c = c
                .manual(&self.environment, &p, uuid)
                .map_err(|e| e.to_string())?;
        }
        Ok(c)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    #[serde(deserialize_with = "files")]
    pub starter: Files,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "protected_profiles"
    )]
    pub protected_profiles: Vec<String>,
    #[serde(deserialize_with = "list::<_,Case,3>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !key(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || !shape(&self.starter)
            || self.protected_profiles.len() > 8
            || self
                .protected_profiles
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != self.protected_profiles.len()
            || self
                .protected_profiles
                .iter()
                .any(|n| !self.starter.contains_key(n))
            || !(2..=3).contains(&self.cases.len())
            || !fits(self, MAX_DEFINITION / 2)
        {
            return Err("Invalid bounded persistent-profile task".into());
        }
        if self
            .cases
            .iter()
            .any(|c| c.environment.vlan.is_some() != self.cases[0].environment.vlan.is_some())
        {
            return Err("Keep each task's source-enabled VLAN capability consistent across all sample cases".into());
        }
        if self
            .cases
            .iter()
            .any(|c| c.environment.bond.is_some() != self.cases[0].environment.bond.is_some())
        {
            return Err(
                "Keep the source-enabled bond capability consistent across all sample cases".into(),
            );
        }
        if self
            .cases
            .iter()
            .any(|c| c.environment.bridge.is_some() != self.cases[0].environment.bridge.is_some())
        {
            return Err(
                "Keep each task's source bridge capability consistent across sample cases".into(),
            );
        }
        let initial_profiles =
            model::Policy::from_texts_for(&self.cases[0].environment, &self.starter).map_err(
                |e| {
                    format!("The starter must be supported, intentionally unfinished settings: {e}")
                },
            )?;
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            c.environment.validate().map_err(|e| e.to_string())?;
            if !key(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || !shape(&c.current_files)
                || c.initially_selected.len() > 2
                || c.initially_selected.iter().collect::<BTreeSet<_>>().len()
                    != c.initially_selected.len()
                || c.requirements.is_empty()
                || c.requirements.len() > 12
            {
                return Err("Invalid or duplicate persistent-profile sample case".into());
            }
            let current_policy = model::Policy::from_texts_for(&c.environment, &c.current_files)
                .map_err(|e| e.to_string())?;
            let mut devices = BTreeSet::new();
            for uuid in &c.initially_selected {
                let p = current_policy
                    .profile(uuid)
                    .ok_or("Unknown current profile identity")?;
                if !devices.insert(&p.interface) {
                    return Err("Two initial selections target the same interface".into());
                }
            }
            let initial = c.initial()?;
            let mut goals = BTreeSet::new();
            for r in &c.requirements {
                if !goals.insert(r.identity())
                    || !c
                        .environment
                        .physical
                        .links
                        .iter()
                        .any(|l| l.name == r.interface())
                {
                    return Err("Unknown or duplicate profile outcome reference".into());
                }
                match r {
                    Requirement::CurrentUnchanged { interface }
                        if initial.for_device(interface).is_none() =>
                    {
                        return Err(
                            "Preservation needs an actually supplied current selection".into()
                        );
                    }
                    Requirement::FutureProfile { uuid, .. }
                        if initial_profiles.profile(uuid).is_none() =>
                    {
                        return Err(
                            "Expected next profile must use a supplied starter identity".into()
                        );
                    }
                    Requirement::FutureDhcp { interface }
                        if !c.environment.dhcp.contains_key(interface) =>
                    {
                        return Err("A DHCP outcome goal needs an explicit supplied result for that interface".into());
                    }
                    Requirement::FutureAddress { address, .. } if !address.usable() => {
                        return Err("Expected conventional IPv4 address is invalid".into());
                    }
                    Requirement::FutureNetwork { interface, network } => {
                        if !network.valid()
                            || !(1..=30).contains(&network.length)
                            || !c
                                .environment
                                .physical
                                .links
                                .iter()
                                .any(|l| l.name == *interface && l.network == *network)
                        {
                            return Err("A network outcome needs the actual supplied conventional LAN prefix".into());
                        }
                    }
                    Requirement::FutureBridgeDelivery {
                        interface,
                        from,
                        to,
                        ..
                    } => {
                        let b =
                            c.environment.bridge.as_ref().ok_or(
                                "A bridge peer outcome needs explicit source bridge facts",
                            )?;
                        if interface != &b.profiles.controller.interface
                            || from == to
                            || ![*from, *to]
                                .iter()
                                .all(|a| b.peers.iter().any(|p| p.address == *a))
                        {
                            return Err("A bridge peer outcome needs two distinct supplied endpoints on the provided bridge controller".into());
                        }
                    }
                    Requirement::FuturePath { target, .. }
                        if !c
                            .environment
                            .physical
                            .nodes
                            .iter()
                            .any(|n| n.address == *target)
                            && !c
                                .environment
                                .physical
                                .gateways
                                .iter()
                                .any(|g| g.address == *target) =>
                    {
                        return Err("A path outcome needs a supplied virtual destination".into());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn run(&self, files: &Files) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, files)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.results);
            }
        }
        Err("Profile checks exceeded the bounded phase budget".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub diagnostic: Option<String>,
    pub checks: Vec<(bool, String)>,
    pub current: Option<model::Choices>,
    pub future: Option<model::Choices>,
    pub decisions: Vec<model::Decision>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.diagnostic.is_none() && !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
struct Working {
    policy: model::Policy,
    initial: model::Choices,
    warm: Option<model::Choices>,
    future: Option<model::Choices>,
    decisions: Vec<model::Decision>,
}
pub struct Run {
    task: Task,
    files: Files,
    index: usize,
    stage: u8,
    working: Option<Working>,
    results: Vec<CaseResult>,
    cancelled: bool,
}
impl Run {
    pub fn new(task: &Task, files: &Files) -> Result<Self, String> {
        task.validate()?;
        if !shape(files) || files.keys().ne(task.starter.keys()) {
            return Err("Retain the supplied bounded virtual filenames; previous work kept".into());
        }
        Ok(Self {
            task: task.clone(),
            files: files.clone(),
            index: 0,
            stage: 0,
            working: None,
            results: vec![],
            cancelled: false,
        })
    }
    pub fn done(&self) -> bool {
        !self.cancelled && self.index == self.task.cases.len()
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.working = None;
        self.results.clear();
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.results)
    }
    fn fail(&mut self, reason: String) {
        let c = &self.task.cases[self.index];
        let w = self.working.take();
        self.results.push(CaseResult {
            id: c.id.clone(),
            label: c.label.clone(),
            checks: vec![(false, reason.clone())],
            diagnostic: Some(reason),
            current: w.and_then(|w| w.warm),
            future: None,
            decisions: vec![],
        });
        self.index += 1;
        self.stage = 0;
    }
    pub fn tick(&mut self) -> bool {
        if self.cancelled {
            return false;
        }
        if self.done() {
            return true;
        }
        let case = &self.task.cases[self.index];
        if self.stage == 0 {
            let setup = (|| -> Result<Working, String> {
                Ok(Working {
                    policy: model::Policy::from_texts_for(&case.environment, &self.files)
                        .map_err(|e| e.to_string())?,
                    initial: case.initial()?,
                    warm: None,
                    future: None,
                    decisions: vec![],
                })
            })();
            match setup {
                Ok(w) => {
                    self.working = Some(w);
                    self.stage = 1;
                }
                Err(e) => self.fail(e),
            }
            return self.done();
        }
        let mut working = self.working.take().unwrap();
        if self.stage == 1 {
            match working
                .initial
                .automatic(&case.environment, &working.policy)
            {
                Ok((warm, _)) => {
                    working.warm = Some(warm);
                    self.working = Some(working);
                    self.stage = 2;
                }
                Err(e) => {
                    self.working = Some(working);
                    self.fail(e.to_string());
                }
            }
            return self.done();
        }
        if self.stage == 2 {
            match model::Choices::empty().automatic(&case.environment, &working.policy) {
                Ok((future, decisions)) => {
                    working.future = Some(future);
                    working.decisions = decisions;
                    self.working = Some(working);
                    self.stage = 3;
                }
                Err(e) => {
                    self.working = Some(working);
                    self.fail(e.to_string());
                }
            }
            return self.done();
        }
        let original =
            model::Policy::from_texts_for(&case.environment, &self.task.starter).unwrap();
        let identities = original.profiles().all(|(file, p)| {
            working
                .policy
                .profiles()
                .any(|(f, new)| f == file && new.uuid == p.uuid)
        });
        let mut checks=vec![(identities,"Retain the supplied filenames and UUID identities; edit settings without inventing replacement identities.".into())];
        for name in &self.task.protected_profiles {
            checks.push((original.protected_unchanged(&working.policy,name), format!("{name}: retain the supplied protected profile settings. Comments and equivalent supported spelling may differ; changing its configuration is outside this task.")));
        }
        let warm = working.warm.as_ref().unwrap();
        let future = working.future.as_ref().unwrap();
        checks.extend(
            case.requirements
                .iter()
                .map(|r| r.check(case, &working.initial, warm, future)),
        );
        self.results.push(CaseResult {
            id: case.id.clone(),
            label: case.label.clone(),
            diagnostic: None,
            checks,
            current: working.warm,
            future: working.future,
            decisions: working.decisions,
        });
        self.index += 1;
        self.stage = 0;
        self.done()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "files")]
    pub example_worked: Files,
}
// Only a successful, exact teaching-source value is retained. Never cache a
// learner draft, proof, run result or failure. One bounded entry per thread;
// source changes require the complete reachability/identity checks again.
thread_local! {
    static VALIDATED_CONTROL_SOURCE: std::cell::RefCell<Option<Definition>> = const { std::cell::RefCell::new(None) };
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        let controlled = self
            .initial
            .cases
            .first()
            .is_some_and(|c| c.environment.bond.is_some() || c.environment.bridge.is_some());
        if controlled
            && VALIDATED_CONTROL_SOURCE.with(|entry| entry.borrow().as_ref() == Some(self))
        {
            return Ok(());
        }
        self.validate_uncached()?;
        if controlled {
            VALIDATED_CONTROL_SOURCE.with(|entry| *entry.borrow_mut() = Some(self.clone()));
        }
        Ok(())
    }
    fn validate_uncached(&self) -> Result<(), String> {
        if !fits(self, MAX_DEFINITION) {
            return Err("Profile lesson exceeds its byte limit".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.cases[0].environment.bond.is_some()
            != self.example.cases[0].environment.bond.is_some()
        {
            return Err("Separate practice needs the same explicit bond-control capability as its assignment".into());
        }
        if self.initial.cases[0].environment.bridge.is_some()
            != self.example.cases[0].environment.bridge.is_some()
        {
            return Err(
                "Separate practice needs the same source bridge capability as the assignment"
                    .into(),
            );
        }
        if self.initial.id == self.example.id {
            return Err("Practice needs its own task identity".into());
        }
        let a = model::Policy::from_texts_for(
            &self.initial.cases[0].environment,
            &self.initial.starter,
        )
        .map_err(|e| e.to_string())?;
        let b = model::Policy::from_texts_for(
            &self.example.cases[0].environment,
            &self.example.starter,
        )
        .map_err(|e| e.to_string())?;
        if a.profiles()
            .any(|(_, p)| b.profiles().any(|(_, q)| p.uuid == q.uuid))
        {
            return Err("Practice and assignment profile identities must differ".into());
        }
        if let (Some(a), Some(b)) = (a.control(), b.control()) {
            let mut aa = std::iter::once(&a.source.controller).chain(&a.source.members);
            if aa.any(|x| {
                std::iter::once(&b.source.controller)
                    .chain(&b.source.members)
                    .any(|y| x.uuid.eq_ignore_ascii_case(&y.uuid))
            }) {
                return Err(
                    "Practice and assignment control-profile identities must differ too".into(),
                );
            }
        }
        if let (Some(a), Some(b)) = (a.bridge_control(), b.bridge_control())
            && std::iter::once(&a.source().controller)
                .chain(&a.source().members)
                .any(|x| {
                    std::iter::once(&b.source().controller)
                        .chain(&b.source().members)
                        .any(|y| x.uuid.eq_ignore_ascii_case(&y.uuid))
                })
        {
            return Err(
                "Bridge practice and assignment controller/port UUIDs must be disjoint".into(),
            );
        }
        for x in &self.initial.cases {
            for y in &self.example.cases {
                if x.environment.physical.links.iter().any(|p| {
                    y.environment.physical.links.iter().any(|q| {
                        p.network.contains(q.network.address)
                            || q.network.contains(p.network.address)
                    })
                }) {
                    return Err(
                        "Worked practice networks must differ from assignment networks".into(),
                    );
                }
            }
        }
        for t in [&self.initial, &self.example] {
            if t.run(&t.starter)?.iter().all(CaseResult::passed) {
                return Err("Retain meaningful unfinished work in each starter".into());
            }
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked profile bundle does not reach every practice outcome".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id
            || !s.assignment.matches(&self.initial)
            || !s.practice.matches(&self.example)
        {
            return Err(
                "Profile progress belongs to another source task; original work retained".into(),
            );
        }
        Ok(())
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok() && s.assignment.passed(&self.initial)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if s.assignment.tested.as_ref() != Some(&s.assignment.draft) {
            return vec![(
                false,
                "Not checked · test the current saved-profile draft.".into(),
            )];
        }
        match self.initial.run(&s.assignment.draft) {
            Err(e) => vec![(false, e)],
            Ok(rows) => rows
                .into_iter()
                .flat_map(|r| {
                    r.checks
                        .into_iter()
                        .map(move |(ok, text)| (ok, format!("{} · {text}", r.label)))
                })
                .collect(),
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_DEFINITION {
            return Err("Profile lesson exceeds its byte limit".into());
        }
        let d: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid profile lesson: {e}"))?;
        d.validate()?;
        Ok(d)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "files")]
    pub draft: Files,
    #[serde(deserialize_with = "optional_files")]
    pub tested: Option<Files>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub selected: String,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            tested: None,
            selected: t.cases[0]
                .environment
                .bond
                .as_ref()
                .map(|b| b.profiles.controller.file.clone())
                .or_else(|| {
                    t.cases[0]
                        .environment
                        .bridge
                        .as_ref()
                        .map(|b| b.profiles.controller.file.clone())
                })
                .unwrap_or_else(|| {
                    t.starter
                        .keys()
                        .find(|name| {
                            t.cases[0].environment.vlan.is_none()
                                || !t.protected_profiles.contains(name)
                        })
                        .or_else(|| t.starter.keys().next())
                        .unwrap()
                        .clone()
                }),
        }
    }
    fn matches(&self, t: &Task) -> bool {
        self.draft.keys().eq(t.starter.keys())
    }
    pub fn select(&mut self, name: &str) -> Result<(), String> {
        if !self.draft.contains_key(name) {
            return Err("Choose a supplied virtual profile file".into());
        }
        self.selected = name.into();
        Ok(())
    }
    pub fn edit(&mut self, name: &str, text: &str) -> Result<(), String> {
        if !text_bound(text) {
            return Err(
                "Use at most16KiB without control characters; previous draft retained".into(),
            );
        }
        let old = self
            .draft
            .get_mut(name)
            .ok_or("Only supplied virtual filenames can be edited")?;
        if old != text {
            *old = text.into();
            self.tested = None;
        }
        Ok(())
    }
    pub fn replace(&mut self, files: &Files) -> Result<(), String> {
        if !shape(files) || !files.keys().eq(self.draft.keys()) {
            return Err("Keep the supplied virtual filenames and bounded profile text".into());
        }
        if files != &self.draft {
            self.draft = files.clone();
            self.tested = None;
        }
        Ok(())
    }
    pub fn begin(&mut self, t: &Task) -> Result<Run, String> {
        self.tested = None;
        Run::new(t, &self.draft)
    }
    pub fn finish(&mut self, t: &Task, run: &Run) -> Result<(), String> {
        if !run.done() || run.files != self.draft || run.task != *t {
            return Err(
                "Stopped, unfinished or changed profile checks cannot award current credit".into(),
            );
        }
        self.tested = Some(self.draft.clone());
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<Vec<CaseResult>, String> {
        let mut run = self.begin(t)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                self.finish(t, &run)?;
                return Ok(run.results);
            }
        }
        Err("Profile checks reached their bounded phase limit".into())
    }
    pub fn passed(&self, t: &Task) -> bool {
        self.tested.as_ref() == Some(&self.draft)
            && t.run(&self.draft)
                .is_ok_and(|rows| !rows.is_empty() && rows.iter().all(CaseResult::passed))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.origin)
            || !fits(self, MAX_SESSION)
            || [&self.assignment, &self.practice].iter().any(|s| {
                !shape(&s.draft)
                    || !s.draft.contains_key(&s.selected)
                    || s.tested
                        .as_ref()
                        .is_some_and(|t| !shape(t) || !t.keys().eq(s.draft.keys()))
            })
        {
            return Err("Invalid version, selection or bounds in profile progress".into());
        }
        Ok(())
    }
    pub fn from_json(text: &str, d: &Definition) -> Result<Self, String> {
        if text.len() > MAX_SESSION {
            return Err("Profile progress exceeds its byte limit".into());
        }
        let s: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid profile progress: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
    /// Internal data-only reference adapter; not a shell command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native profile editor; this is internal reference data")?;
        if text.len() > MAX_SESSION {
            return Err("Reference bundle exceeds its byte limit".into());
        }
        #[derive(Deserialize)]
        struct Bundle(#[serde(deserialize_with = "files")] Files);
        let Bundle(files) =
            serde_json::from_str(text).map_err(|e| format!("Invalid reference profiles: {e}"))?;
        self.assignment.replace(&files)?;
        let rows = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Simulated startup profiles checked: {}/{} cases passed",
            rows.iter().filter(|r| r.passed()).count(),
            rows.len()
        ))
    }
}
pub const HELP: &str = "Edit supplied virtual NetworkManager keyfiles, then Test startup profiles. Re-reading saved files keeps existing selections; a separate fresh-startup projection selects eligible profiles by unambiguous per-device priority and tests routes. Only bounded manual or supplied-lease automatic IPv4 Ethernet settings are supported. No real files, daemon, commands, packets or restart. Timestamp ties, duplicate UUID precedence, Wi-Fi, DNS merging, DHCP packets/renewal/retries/fallback and asynchronous activation are outside this subset. Automatic route metrics are supplied test inputs. An unsupported diagnostic can describe valid real functionality that this simulator does not implement.";
pub const VLAN_HELP: &str = "Edit the supplied virtual VLAN keyfiles, then Test startup profiles. Each test keeps its supplied current selections and separately checks the next startup against the switch plan. A valid path needs the selected profile's IP settings, physical parent and matching tags outward and back. The editor checks its draft directly; it does not run a real reload or activation. This bounded subset supports manual IPv4, literal supplied parents, 802.1Q, explicit flags=1 and data IDs1–4094. Priority tag0, reserved4095, parent UUIDs, nested tags, alternate flags, priority maps, discovery, DHCP/IPv6 VLANs, bonds and bridges are outside this activity. No host files, daemon, packets, commands or restart. Unsupported settings fail explicitly. Current settings can remain faulty even when the saved future configuration passes.";
pub const BRIDGE_HELP: &str = "All four files are virtual. A bridge forwards between attached ports; it does not choose one active NIC like an active-backup bond. Put the host's manual IPv4 settings on the supplied bridge controller. Bind participating Ethernet ports using both controller and port-type=bridge. autoconnect-ports=1 starts matching bound ports even if their own autoconnect is false. Current IP/plan/membership stay separate from edited future files. Every physical case has its own expected paths: a down port isolates its segment rather than causing a backup path. Tests check address/route selection and actual untagged delivery separately, including peer-to-peer broadcasts. This bounded tree has no tagged traffic, VLAN filtering, STP election/timing, FDB learning, independent port-driven startup, general nmcli/ip commands, kernel/daemon/boot or real network. NM's omitted STP default is enabled; disabling it here does not demonstrate general loop safety. Unsupported settings fail explicitly. No learner input runs on the host.";
pub const BOND_HELP: &str = "Test the five virtual keyfile drafts against each supplied physical case. Current bond settings/membership and current VLAN selections stay separate from next startup. The controller's autoconnect-ports=1 starts matching bound ports even if their own autoconnect is false. A candidate without both controller and port-type binding keys is not a bond member. Independent port-driven startup/discovery is outside this subset; retain the supplied false port autoconnect while learning controller activation. Only one provided active-backup controller, two candidate NICs, explicit miimon=100/primary_reselect=always or0 and two manual IPv4 VLANs are supported. MII observes supplied physical carrier, never IP reachability or tags; both-direction tagged delivery is still checked. These are settled projections, not real timers, activation order, gratuitous ARP/MAC changes, bandwidth, a daemon, packets, boot or host operations. Other modes, ARP monitoring, nested controllers, bridges and DHCP/IPv6 bonds remain unsupported. Unknown syntax/settings fail honestly; no learner text runs a real command.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::NmProfiles { simulation },
        crate::engine::Goal::NmProfiles { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Saved-profile source/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Profile source identity, task and supplied drafts must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::NmProfiles { simulation } = &a.state else {
        return Err("Saved profile work belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible profile attempt metadata; previous work retained".into());
    }
    d.bound(simulation)
}
