//! Bounded static-Ethernet policy projection. No daemon or host operations.
use crate::{
    ipv4_routing::Prefix,
    linux_host as host,
    nm_profiles::{self as keyfile, Addressing, Error, ErrorKind, Profile, Settings},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        line: 0,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge: Option<crate::bridge_host::Facts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bond: Option<crate::bond_host::Facts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vlan: Option<crate::vlan_host::Facts>,
    pub physical: host::Profile,
    #[serde(deserialize_with = "metrics")]
    pub default_metrics: BTreeMap<String, u32>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::nm_dhcp::deserialize"
    )]
    pub dhcp: crate::nm_dhcp::Outcomes,
}
fn metrics<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, u32>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Metrics;
    impl<'de> Visitor<'de> for Metrics {
        type Value = BTreeMap<String, u32>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most two distinct interface metrics")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Name(#[serde(deserialize_with = "crate::cron::text::<_,15>")] String);
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: serde::Deserializer<'a>>(
                    _: D,
                ) -> std::result::Result<Self, D::Error> {
                    Err(serde::de::Error::custom("Interface metric count limit"))
                }
            }
            let mut out = BTreeMap::new();
            while out.len() < 2 {
                let Some(Name(name)) = a.next_key()? else {
                    return Ok(out);
                };
                if out.contains_key(&name) {
                    return Err(A::Error::custom("Duplicate interface metric"));
                }
                out.insert(name, a.next_value()?);
            }
            let _ = a.next_key::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_map(Metrics)
}
impl Environment {
    pub fn validate(&self) -> Result<()> {
        self.physical
            .validate()
            .map_err(|e| error(ErrorKind::Invalid, e))?;
        if self.physical.ipv6.is_some() {
            return Err(error(
                ErrorKind::Unsupported,
                "This NetworkManager lesson supports IPv4 only; IPv6 facts require the separate source-enabled host workspace",
            ));
        }
        if self.default_metrics.len() != self.physical.links.len()
            || self
                .physical
                .links
                .iter()
                .any(|l| !self.default_metrics.contains_key(&l.name))
        {
            return Err(error(
                ErrorKind::Unsupported,
                "Every supplied interface needs an explicit modeled automatic metric; no universal device default is guessed",
            ));
        }
        if self.bridge.is_some()
            && (self.vlan.is_some() || self.bond.is_some() || !self.dhcp.is_empty())
        {
            return Err(error(
                ErrorKind::Unsupported,
                "The untagged manual-IPv4 bridge context is separate from VLAN/bond/DHCP capabilities",
            ));
        }
        if let Some(facts) = &self.vlan {
            facts
                .validate(&self.physical)
                .map_err(|e| error(ErrorKind::Invalid, e))?;
            if !self.dhcp.is_empty() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "DHCP-on-VLAN is outside this source-enabled activity",
                ));
            }
        }
        if let Some(bond) = &self.bond {
            let vlan = self.vlan.as_ref().ok_or_else(|| {
                error(
                    ErrorKind::Unsupported,
                    "Bond control profiles require this activity's explicit VLAN context",
                )
            })?;
            bond.validate(vlan)?;
        }
        if let Some(bridge) = &self.bridge {
            bridge.validate(&self.physical)?;
        }
        crate::nm_dhcp::validate(&self.dhcp, &self.physical)
            .map_err(|e| error(ErrorKind::Invalid, e))?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    filename: String,
    profile: Settings,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    bridge_control: Option<crate::bridge_control::Plan>,
    control: Option<crate::bond_control::Plan>,
    entries: BTreeMap<String, Entry>,
}
fn filename(s: &str) -> bool {
    !s.starts_with('.')
        && s.len() <= 64
        && s.ends_with(".nmconnection")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
impl Policy {
    pub fn from_texts(files: &BTreeMap<String, String>) -> Result<Self> {
        Self::parse(files, None, None, None)
    }
    pub fn from_texts_for(env: &Environment, files: &BTreeMap<String, String>) -> Result<Self> {
        env.validate()?;
        Self::parse(
            files,
            env.vlan.as_ref(),
            env.bond.as_ref(),
            env.bridge.as_ref(),
        )
    }
    fn parse(
        files: &BTreeMap<String, String>,
        facts: Option<&crate::vlan_host::Facts>,
        bond: Option<&crate::bond_host::Facts>,
        bridge: Option<&crate::bridge_host::Facts>,
    ) -> Result<Self> {
        if files.is_empty()
            || files.len() > keyfile::MAX_PROFILES
            || files.values().any(|s| s.len() > keyfile::MAX_TEXT)
        {
            return Err(error(
                ErrorKind::Limit,
                "Use one through eight supplied keyfiles, each at most16KiB",
            ));
        }
        let control = bond
            .map(|b| crate::bond_control::Plan::from_files(&b.profiles, files))
            .transpose()?;
        let bridge_control = bridge
            .map(|b| crate::bridge_control::Plan::from_files(&b.profiles, files))
            .transpose()?;
        let mut entries = BTreeMap::new();
        for (name, text) in files {
            if !filename(name) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This virtual bundle uses literal .nmconnection leaf filenames; host paths, directory traversal and links are not read",
                ));
            }
            if let Some(b) = bond
                && std::iter::once(&b.profiles.controller)
                    .chain(&b.profiles.members)
                    .any(|p| p.file == *name)
            {
                continue;
            }
            if bridge.is_some_and(|b| b.profiles.members.iter().any(|p| p.file == *name)) {
                continue;
            }
            let parsed = if bridge.is_some_and(|b| b.profiles.controller.file == *name) {
                Ok(bridge_control.as_ref().unwrap().controller().ipv4.clone())
            } else if facts.is_some() {
                keyfile::parse_vlan_settings(text)
            } else {
                keyfile::parse_settings(text)
            };
            let profile = parsed.map_err(|mut e| {
                e.message = format!("{name}: {}", e.message);
                e
            })?;
            if let Some(facts) = facts {
                if let Some(bond) = bond {
                    bond.validate_settings(facts, &profile).map_err(|mut e| {
                        e.message = format!("{name}: {}", e.message);
                        e
                    })?;
                } else {
                    facts
                        .validate_profile(&profile)
                        .map_err(|e| error(ErrorKind::Unsupported, format!("{name}: {e}")))?;
                }
            }
            if let Some(b) = bridge
                && (b
                    .profiles
                    .members
                    .iter()
                    .any(|p| p.interface == profile.interface)
                    || (profile.interface == b.profiles.controller.interface
                        && name != &b.profiles.controller.file))
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Use the one supplied bridge controller file for its IP; physical-member and replacement-controller IP profiles are not modeled",
                ));
            }
            if entries.contains_key(&profile.uuid) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Duplicate UUID file-precedence behavior is outside this subset; keep supplied profile identities distinct",
                ));
            }
            entries.insert(
                profile.uuid.clone(),
                Entry {
                    filename: name.clone(),
                    profile,
                },
            );
        }
        Ok(Self {
            entries,
            control,
            bridge_control,
        })
    }
    pub fn control(&self) -> Option<&crate::bond_control::Plan> {
        self.control.as_ref()
    }
    pub fn bridge_control(&self) -> Option<&crate::bridge_control::Plan> {
        self.bridge_control.as_ref()
    }
    pub fn protected_unchanged(&self, other: &Self, name: &str) -> bool {
        match (
            self.bridge_control.as_ref().and_then(|p| p.profile(name)),
            other.bridge_control.as_ref().and_then(|p| p.profile(name)),
        ) {
            (Some(a), Some(b)) => {
                return self.bridge_control.as_ref().unwrap().source()
                    == other.bridge_control.as_ref().unwrap().source()
                    && a == b;
            }
            (Some(_), None) | (None, Some(_)) => return false,
            (None, None) => {}
        }
        match (
            self.profiles().find(|(f, _)| *f == name),
            other.profiles().find(|(f, _)| *f == name),
        ) {
            (Some(a), Some(b)) => a == b,
            (None, None) => match (&self.control, &other.control) {
                (Some(a), Some(b)) if a.source == b.source => {
                    match (a.profile(name), b.profile(name)) {
                        (Some(a), Some(b)) => a == b,
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }
    pub fn profile(&self, uuid: &str) -> Option<&Settings> {
        self.entries.get(uuid).map(|e| &e.profile)
    }
    pub fn profiles(&self) -> impl Iterator<Item = (&str, &Settings)> {
        self.entries
            .values()
            .map(|e| (e.filename.as_str(), &e.profile))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    Connected,
    Default,
    Static,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub destination: Prefix,
    pub via: Option<Ipv4Addr>,
    pub device: String,
    pub metric: u32,
    pub origin: Origin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub route: Route,
    pub source: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub pass: bool,
    pub reason: String,
    pub selection: Option<Selection>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub device: String,
    pub uuid: Option<String>,
    pub reason: String,
}
/// Selected settings are owned snapshots. Editing/reloading files cannot preempt them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choices {
    bridge_control: Option<crate::bridge_control::Snapshot>,
    control: Option<crate::bond_control::Snapshot>,
    selected: BTreeMap<String, Profile>,
    automatic_devices: BTreeSet<String>,
}
impl Choices {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn for_device(&self, name: &str) -> Option<&Profile> {
        self.selected.get(name)
    }
    pub fn uses_dhcp(&self, name: &str) -> bool {
        self.automatic_devices.contains(name)
    }
    pub fn controls(&self) -> Option<&crate::bond_control::Snapshot> {
        self.control.as_ref()
    }
    pub fn bridge_controls(&self) -> Option<&crate::bridge_control::Snapshot> {
        self.bridge_control.as_ref()
    }
    /// Atomic source-held manual-IP controller selection; current membership is
    /// explicit and never inferred from edited automatic eligibility.
    pub fn seed_current_bridge(
        &self,
        env: &Environment,
        policy: &Policy,
        current: &crate::bridge_control::Current,
    ) -> Result<Self> {
        env.validate()?;
        if self.control.is_some() || self.bridge_control.is_some() || !self.selected.is_empty() {
            return Err(error(
                ErrorKind::Invalid,
                "Seed the explicit initial bridge snapshot before any other IP selection",
            ));
        }
        let facts = env.bridge.as_ref().ok_or_else(|| {
            error(
                ErrorKind::Unsupported,
                "Current bridge membership needs its original source bridge context",
            )
        })?;
        let plan = policy
            .bridge_control
            .as_ref()
            .filter(|p| p.source() == &facts.profiles)
            .ok_or_else(|| {
                error(
                    ErrorKind::Invalid,
                    "Current bridge files belong to different supplied identities",
                )
            })?;
        let mut out = self.clone();
        out.bridge_control = Some(crate::bridge_control::Snapshot::supplied(
            plan.clone(),
            current.clone(),
        )?);
        if current.controller {
            let settings = &plan.controller().ipv4;
            let Addressing::Manual { address, gateway } = &settings.addressing else {
                return Err(error(
                    ErrorKind::Unsupported,
                    "The selected current bridge needs its supplied manual IP settings",
                ));
            };
            out.selected.insert(
                settings.interface.clone(),
                settings.resolved(address.clone(), *gateway),
            );
        }
        out.physical_projection(env, vec![])?;
        Ok(out)
    }
    pub fn seed_current_controls(
        &self,
        env: &Environment,
        policy: &Policy,
        current: &crate::bond_control::Current,
    ) -> Result<Self> {
        env.validate()?;
        if self.control.is_some() || self.bridge_control.is_some() || !self.selected.is_empty() {
            return Err(error(
                ErrorKind::Invalid,
                "Seed only the supplied initial control snapshot, before selecting IP profiles",
            ));
        }
        let b = env.bond.as_ref().ok_or_else(|| {
            error(
                ErrorKind::Unsupported,
                "Current control membership requires its original source-enabled bond context",
            )
        })?;
        let v = env.vlan.as_ref().unwrap();
        let plan = policy
            .control
            .as_ref()
            .filter(|p| p.source == b.profiles)
            .ok_or_else(|| {
                error(
                    ErrorKind::Invalid,
                    "Current control files belong to different supplied identities",
                )
            })?;
        let mut out = self.clone();
        out.control = Some(crate::bond_control::Snapshot::supplied(
            plan.clone(),
            current.clone(),
            &b.carrier(v)?,
        )?);
        Ok(out)
    }
    fn activate(&mut self, env: &Environment, settings: &Settings) -> Result<()> {
        if let Some(facts) = &env.bridge
            && settings.interface == facts.profiles.controller.interface
        {
            let snapshot = self.bridge_control.as_ref().ok_or_else(|| {
                error(
                    ErrorKind::Invalid,
                    "Select the source bridge control snapshot before its IP configuration",
                )
            })?;
            if !snapshot.current.controller || &snapshot.plan.controller().ipv4 != settings {
                return Err(error(
                    ErrorKind::Invalid,
                    "The IP profile must belong to the selected owned bridge controller; re-reading future files does not reconfigure current settings",
                ));
            }
        }
        match (&env.vlan, &settings.vlan) {
            (Some(facts), _) => {
                if let Some(bond) = &env.bond {
                    bond.validate_settings(facts, settings)?;
                    if !self.control.as_ref().is_some_and(|c| c.current.controller) {
                        return Err(error(
                            ErrorKind::Invalid,
                            "The bond controller is not selected. Inspect its automatic eligibility and explicit current/future dependencies before selecting a VLAN above it",
                        ));
                    }
                } else {
                    facts
                        .validate_profile(settings)
                        .map_err(|e| error(ErrorKind::Unsupported, e))?;
                }
            }
            (None, Some(_)) => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "VLAN settings require this task's explicit supplied L2 context",
                ));
            }
            _ => {}
        }
        if let Some(vlan) = &settings.vlan
            && self
                .selected
                .values()
                .any(|p| p.interface != settings.interface && p.vlan.as_ref() == Some(vlan))
        {
            return Err(error(
                ErrorKind::Invalid,
                "The parent/VLAN ID is already used by another selected software interface; keep their supplied VLANs distinct",
            ));
        }
        let (address, router) = match &settings.addressing {
            Addressing::Manual { address, gateway } => (address.clone(), *gateway),
            Addressing::Automatic => match env.dhcp.get(&settings.interface) {
                Some(crate::nm_dhcp::Outcome::Bound { address, router }) => {
                    (address.clone(), *router)
                }
                Some(crate::nm_dhcp::Outcome::Unavailable {}) => {
                    return Err(error(
                        ErrorKind::Invalid,
                        format!(
                            "{}: supplied DHCP is unavailable; no address or default route was invented. This settled exercise does not simulate timeouts, retries or fallback.",
                            settings.interface
                        ),
                    ));
                }
                None => {
                    return Err(error(
                        ErrorKind::Unsupported,
                        format!(
                            "{}: no DHCP outcome was supplied for this exercise; automatic addressing cannot be resolved.",
                            settings.interface
                        ),
                    ));
                }
            },
        };
        self.selected.insert(
            settings.interface.clone(),
            settings.resolved(address, router),
        );
        if matches!(settings.addressing, Addressing::Automatic) {
            self.automatic_devices.insert(settings.interface.clone());
        } else {
            self.automatic_devices.remove(&settings.interface);
        }
        Ok(())
    }
    pub fn selected(&self) -> &BTreeMap<String, Profile> {
        &self.selected
    }
    fn interfaces(&self, env: &Environment) -> Result<Vec<host::Interface>> {
        env.validate()?;
        if let Some(facts) = &env.vlan {
            if let Some(bond) = &env.bond {
                bond.validate_selection(facts, &self.selected)?;
                if let Some(snapshot) = &self.control {
                    bond.selection(facts, snapshot)?;
                } else if !self.selected.is_empty() {
                    return Err(error(
                        ErrorKind::Invalid,
                        "A selected VLAN above a bond needs its explicit control snapshot",
                    ));
                }
            } else {
                if self.control.is_some() {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "A bond control snapshot cannot be reinterpreted in an unbonded context",
                    ));
                }
                facts
                    .validate_selection(&self.selected)
                    .map_err(|e| error(ErrorKind::Invalid, e))?;
            }
        } else if self.selected.values().any(|p| p.vlan.is_some()) {
            return Err(error(
                ErrorKind::Unsupported,
                "A VLAN profile snapshot requires its original source-enabled L2 context",
            ));
        }
        if self.control.is_some() && env.bond.is_none() {
            return Err(error(
                ErrorKind::Unsupported,
                "A bond control snapshot requires its original source context",
            ));
        }
        match (&env.bridge, &self.bridge_control) {
            (Some(facts), Some(snapshot)) => facts.validate_selection(snapshot, &self.selected)?,
            (Some(_), None) if self.selected.is_empty() => {}
            (None, None) => {}
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "A selected bridge/IP snapshot requires its original explicit bridge source context",
                ));
            }
        }
        if self.selected.len() > env.physical.links.len()
            || self.selected.iter().any(|(name, p)| {
                name != &p.interface || !env.physical.links.iter().any(|l| &l.name == name)
            })
        {
            return Err(error(
                ErrorKind::Invalid,
                "Selected profile snapshot has a foreign interface",
            ));
        }
        Ok(env
            .physical
            .links
            .iter()
            .map(|l| host::Interface {
                name: l.name.clone(),
                up: self.selected.contains_key(&l.name),
                address: self.selected.get(&l.name).map(|p| p.address.clone()),
            })
            .collect())
    }
    fn physical_projection(
        &self,
        env: &Environment,
        routes: Vec<host::Route>,
    ) -> Result<host::State> {
        let configuration = host::Configuration {
            ipv6: None,
            interfaces: self.interfaces(env)?,
            routes,
            nameservers: vec![],
        };
        host::State::new(env.physical.clone(), configuration).map_err(|e| {
            error(
                ErrorKind::Unsupported,
                format!("This static IPv4 projection cannot represent the supplied context: {e}"),
            )
        })
    }
    pub fn manual(&self, env: &Environment, policy: &Policy, uuid: &str) -> Result<Self> {
        self.physical_projection(env, vec![])?;
        let profile = policy.profile(uuid).ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "No loaded virtual profile has that UUID",
            )
        })?;
        let link = env
            .physical
            .links
            .iter()
            .find(|l| l.name == profile.interface)
            .ok_or_else(|| {
                error(
                    ErrorKind::Invalid,
                    format!("No supplied interface matches {}", profile.interface),
                )
            })?;
        if !link.carrier {
            return Err(error(
                ErrorKind::Unsupported,
                "This settled-state exercise cannot activate a profile while carrier is unavailable; asynchronous waiting/retries are not modeled",
            ));
        }
        let mut next = self.clone();
        next.activate(env, profile)?;
        next.physical_projection(env, vec![])?;
        Ok(next)
    }
    pub fn automatic(&self, env: &Environment, policy: &Policy) -> Result<(Self, Vec<Decision>)> {
        self.physical_projection(env, vec![])?;
        let mut next = self.clone();
        match (&env.bond, &policy.control) {
            (Some(b), Some(plan)) if plan.source == b.profiles => {
                // Keep an existing controller/membership snapshot across reload.
                // Only an unselected controller can start from the new policy.
                if next.control.as_ref().is_none_or(|s| !s.current.controller) {
                    next.control = Some(crate::bond_control::Snapshot::automatic(
                        plan.clone(),
                        &b.carrier(env.vlan.as_ref().unwrap())?,
                    )?);
                }
            }
            (None, None) => {}
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Saved control profiles and source bond capability must match",
                ));
            }
        }
        match (&env.bridge, &policy.bridge_control) {
            (Some(facts), Some(plan)) if plan.source() == &facts.profiles => {
                if next
                    .bridge_control
                    .as_ref()
                    .is_none_or(|s| !s.current.controller)
                {
                    next.bridge_control =
                        Some(crate::bridge_control::Snapshot::automatic(plan.clone())?);
                }
            }
            (None, None) => {}
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Saved bridge control files and source bridge capability must match",
                ));
            }
        }
        let mut decisions = vec![];
        for link in &env.physical.links {
            if let Some(p) = next.selected.get(&link.name) {
                if !link.carrier {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "Carrier-loss lifecycle for a current profile is outside this settled-state subset",
                    ));
                }
                decisions.push(Decision{device:link.name.clone(),uuid:Some(p.uuid.clone()),reason:"Keep the current selected settings; automatic preference changes do not preempt an existing selection.".into()});
                continue;
            }
            if !link.carrier {
                decisions.push(Decision {
                    device: link.name.clone(),
                    uuid: None,
                    reason: "No supplied carrier; this model does not schedule automatic retry."
                        .into(),
                });
                continue;
            }
            let mut eligible = policy
                .entries
                .values()
                .filter(|e| e.profile.interface == link.name && e.profile.autoconnect)
                .collect::<Vec<_>>();
            eligible.sort_by_key(|e| std::cmp::Reverse(e.profile.priority));
            let Some(first) = eligible.first() else {
                decisions.push(Decision {
                    device: link.name.clone(),
                    uuid: None,
                    reason: "No eligible autoconnect profile matches this interface.".into(),
                });
                continue;
            };
            if eligible
                .get(1)
                .is_some_and(|e| e.profile.priority == first.profile.priority)
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    format!(
                        "{} has equal highest autoconnect priorities; actual timestamp/history tie-breaking is outside this subset. Provide an unambiguous preference.",
                        link.name
                    ),
                ));
            }
            match next.activate(env, &first.profile) {
                Ok(()) => {}
                // Keep the existing settled DHCP-unavailable decision, but do
                // not hide a VLAN co-activation conflict by publishing an
                // arbitrary first-wins partial selection as a successful run.
                Err(e)
                    if e.kind == ErrorKind::Invalid
                        && first.profile.vlan.is_none()
                        && env.bridge.is_none() =>
                {
                    decisions.push(Decision {
                        device: link.name.clone(),
                        uuid: None,
                        reason: e.to_string(),
                    });
                    continue;
                }
                Err(e) => return Err(e),
            }
            decisions.push(Decision {
                device: link.name.clone(),
                uuid: Some(first.profile.uuid.clone()),
                reason: format!(
                    "{} has the highest eligible autoconnect priority {}.",
                    first.profile.id, first.profile.priority
                ),
            });
        }
        next.physical_projection(env, vec![])?;
        Ok((next, decisions))
    }
    pub fn route_table(&self, env: &Environment) -> Result<Vec<Route>> {
        self.physical_projection(env, vec![])?;
        let mut rows = vec![];
        for (device, p) in &self.selected {
            let metric = p.route_metric.unwrap_or(env.default_metrics[device]);
            rows.push(Route {
                destination: Prefix::new(p.address.address, p.address.prefix).unwrap(),
                via: None,
                device: device.clone(),
                metric,
                origin: Origin::Connected,
            });
            if let Some(via) = p.gateway {
                rows.push(Route {
                    destination: Prefix::new(Ipv4Addr::UNSPECIFIED, 0).unwrap(),
                    via: Some(via),
                    device: device.clone(),
                    metric,
                    origin: Origin::Default,
                });
            }
            for r in &p.routes {
                if p.never_default && r.destination.length == 0 {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "never-default with an explicit /0 manual route needs wider daemon conformance; this interaction is not approximated",
                    ));
                }
                rows.push(Route {
                    destination: r.destination,
                    via: Some(r.via),
                    device: device.clone(),
                    metric: r.metric.unwrap_or(metric),
                    origin: Origin::Static,
                });
            }
        }
        if rows.len() > 2 * (keyfile::MAX_ROUTES + 2) {
            return Err(error(
                ErrorKind::Limit,
                "Derived route table exceeds its bound",
            ));
        }
        let mut seen = BTreeSet::new();
        rows.retain(|r| {
            seen.insert((
                r.destination.address,
                r.destination.length,
                r.via,
                r.device.clone(),
                r.metric,
            ))
        });
        Ok(rows)
    }
    pub fn select(&self, env: &Environment, target: Ipv4Addr) -> Result<Selection> {
        let mut rows = self
            .route_table(env)?
            .into_iter()
            .filter(|r| r.destination.contains(target))
            .collect::<Vec<_>>();
        rows.sort_by_key(|r| (std::cmp::Reverse(r.destination.length), r.metric));
        let first = rows.first().ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "No configured route matches the supplied destination",
            )
        })?;
        if rows.get(1).is_some_and(|r| {
            r.destination.length == first.destination.length && r.metric == first.metric
        }) {
            return Err(error(
                ErrorKind::Unsupported,
                "Equal-prefix/equal-metric multipath selection is outside this subset; use an unambiguous route preference",
            ));
        }
        Ok(Selection {
            source: self.selected[&first.device].address.address,
            route: first.clone(),
        })
    }
    pub fn bridge_peer_delivery(
        &self,
        env: &Environment,
        from: Ipv4Addr,
        to: Ipv4Addr,
    ) -> Result<crate::bridge_host::Delivery> {
        env.validate()?;
        let facts = env.bridge.as_ref().ok_or_else(|| {
            error(
                ErrorKind::Unsupported,
                "Peer bridge paths need their explicit source context",
            )
        })?;
        let snapshot = self.bridge_control.as_ref().ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "No selected current bridge snapshot for this peer path",
            )
        })?;
        facts.peer_delivery(&env.physical, snapshot, from, to)
    }
    pub fn probe(&self, env: &Environment, target: Ipv4Addr) -> Probe {
        let selected = match self.select(env, target) {
            Ok(s) => s,
            Err(e) => {
                return Probe {
                    pass: false,
                    reason: e.to_string(),
                    selection: None,
                };
            }
        };
        // Choose using this adapter's real connected/profile metrics FIRST. The old
        // transient core is reused only for physical-path checks along that choice.
        let forced = selected
            .route
            .via
            .map(|via| host::Route {
                destination: Prefix::new(target, 32).unwrap(),
                via,
                device: selected.route.device.clone(),
                metric: selected.route.metric,
            })
            .into_iter()
            .collect();
        let projection = match self.physical_projection(env, forced) {
            Ok(s) => s,
            Err(e) => {
                return Probe {
                    pass: false,
                    reason: e.to_string(),
                    selection: Some(selected),
                };
            }
        };
        let p = projection.probe(target);
        if p.pass
            && let Some(facts) = &env.bridge
            && selected.route.device == facts.profiles.controller.interface
        {
            let delivered = self
                .bridge_control
                .as_ref()
                .ok_or_else(|| {
                    error(
                        ErrorKind::Invalid,
                        "No owned bridge control snapshot for this IP path",
                    )
                })
                .and_then(|snapshot| {
                    facts.delivery(
                        &env.physical,
                        snapshot,
                        &self.selected,
                        &selected.route.device,
                        selected.route.via.unwrap_or(target),
                    )
                });
            return match delivered {
                Ok(d) => Probe {
                    pass: d.round_trip(),
                    reason: d.detail,
                    selection: Some(selected),
                },
                Err(e) => Probe {
                    pass: false,
                    reason: e.to_string(),
                    selection: Some(selected),
                },
            };
        }
        if p.pass
            && let (Some(facts), Some(bond)) = (&env.vlan, &env.bond)
        {
            let delivered = self
                .control
                .as_ref()
                .ok_or_else(|| {
                    error(
                        ErrorKind::Invalid,
                        "No current control snapshot for this VLAN path",
                    )
                })
                .and_then(|control| {
                    bond.delivery(
                        facts,
                        control,
                        &self.selected,
                        &selected.route.device,
                        selected.route.via.unwrap_or(target),
                    )
                });
            return match delivered {
                Ok(d) => Probe {
                    pass: d.round_trip(),
                    reason: d.detail,
                    selection: Some(selected),
                },
                Err(e) => Probe {
                    pass: false,
                    reason: e.to_string(),
                    selection: Some(selected),
                },
            };
        }
        if p.pass
            && let Some(facts) = &env.vlan
        {
            return match facts.delivery(
                &self.selected,
                &selected.route.device,
                selected.route.via.unwrap_or(target),
            ) {
                Ok(delivery) => Probe {
                    pass: delivery.round_trip(),
                    reason: delivery.detail,
                    selection: Some(selected),
                },
                Err(e) => Probe {
                    pass: false,
                    reason: format!("Unsupported or invalid VLAN path: {e}"),
                    selection: Some(selected),
                },
            };
        }
        Probe {
            pass: p.pass,
            reason: p.reason,
            selection: Some(selected),
        }
    }
}
