//! Bounded VLAN broadcast data plane. No I/O, hardware, protocol daemon or host command.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    marker::PhantomData,
};
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 80 && !s.chars().any(char::is_control)
}
fn vlan(n: u16) -> bool {
    (1..=4094).contains(&n)
}
fn bounded_map<'de, D, V, const N: usize>(d: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Map<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Map<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique named entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(key) = a.next_key::<String>()? {
                if result.len() >= N || !identifier(&key) || result.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate switching identifier or mapping limit",
                    ));
                }
                result.insert(key, a.next_value()?);
            }
            Ok(result)
        }
    }
    d.deserialize_map(Map::<V, N>(PhantomData))
}
fn switches<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Switch>, D::Error> {
    bounded_map::<D, Switch, 8>(d)
}
fn hosts<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Host>, D::Error> {
    bounded_map::<D, Host, 32>(d)
}
fn ports<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Port>, D::Error> {
    bounded_map::<D, Port, 16>(d)
}
fn vlans<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u16>, D::Error> {
    struct Vlans;
    impl<'de> Visitor<'de> for Vlans {
        type Value = BTreeSet<u16>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most64 unique VLANs in1–4094")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut result = BTreeSet::new();
            while let Some(n) = a.next_element::<u16>()? {
                if result.len() >= 64 || !vlan(n) || !result.insert(n) {
                    return Err(A::Error::custom("Invalid/duplicate VLAN or VLAN-set limit"));
                }
            }
            Ok(result)
        }
    }
    d.deserialize_seq(Vlans)
}
fn cables<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Cable>, D::Error> {
    struct Cables;
    impl<'de> Visitor<'de> for Cables {
        type Value = Vec<Cable>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most64 point-to-point cables")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut result = Vec::new();
            while let Some(c) = a.next_element::<Cable>()? {
                if result.len() >= 64 {
                    return Err(A::Error::custom("Cable limit exceeded"));
                }
                result.push(c);
            }
            Ok(result)
        }
    }
    d.deserialize_seq(Cables)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fabric {
    #[serde(deserialize_with = "switches")]
    pub switches: BTreeMap<String, Switch>,
    #[serde(deserialize_with = "hosts")]
    pub hosts: BTreeMap<String, Host>,
    #[serde(deserialize_with = "cables")]
    pub cables: Vec<Cable>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Switch {
    pub label: String,
    #[serde(deserialize_with = "vlans")]
    pub vlans: BTreeSet<u16>,
    #[serde(deserialize_with = "ports")]
    pub ports: BTreeMap<String, Port>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub label: String,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub up: bool,
    pub mode: Mode,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Mode {
    Access {
        vlan: u16,
    },
    Trunk {
        native: u16,
        #[serde(deserialize_with = "vlans")]
        allowed: BTreeSet<u16>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Jack {
    Switch { switch: String, port: String },
    Host { host: String },
}
impl Jack {
    fn device(&self) -> &str {
        match self {
            Self::Switch { switch, .. } => switch,
            Self::Host { host } => host,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cable {
    pub id: String,
    pub a: Jack,
    pub b: Jack,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hop {
    pub switch: String,
    pub ingress: String,
    pub vlan: u16,
    pub egress: String,
    pub tag: Option<u16>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Broadcast {
    pub recipients: BTreeMap<String, Vec<Hop>>,
    pub drops: Vec<String>,
    pub omitted_drops: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Visibility {
    pub forward: bool,
    pub reverse: bool,
}
impl Visibility {
    pub fn mutual(self) -> bool {
        self.forward && self.reverse
    }
    pub fn isolated(self) -> bool {
        !self.forward && !self.reverse
    }
}
impl Broadcast {
    fn drop_reason(&mut self, message: String) {
        if self.drops.contains(&message) {
            return;
        }
        if self.drops.len() < 64 {
            self.drops.push(message);
        } else {
            self.omitted_drops += 1;
        }
    }
}
impl Mode {
    fn valid(&self) -> bool {
        match self {
            Self::Access { vlan: n } => vlan(*n),
            Self::Trunk { native, allowed } => {
                vlan(*native) && allowed.len() <= 64 && allowed.iter().all(|n| vlan(*n))
            }
        }
    }
    fn egress(&self, local: u16) -> Option<Option<u16>> {
        match self {
            Self::Access { vlan } if *vlan == local => Some(None),
            Self::Trunk { native, allowed } if allowed.contains(&local) => {
                Some(if *native == local { None } else { Some(local) })
            }
            _ => None,
        }
    }
}
impl Fabric {
    pub fn validate(&self) -> Result<(), String> {
        if self.switches.is_empty()
            || self.switches.len() > 8
            || self.hosts.is_empty()
            || self.hosts.len() > 32
            || self.cables.len() > 64
        {
            return Err(
                "Switching model requires1–8 switches,1–32 workstations and at most64 cables"
                    .into(),
            );
        }
        let mut devices = BTreeSet::new();
        for (id, s) in &self.switches {
            if !identifier(id)
                || !devices.insert(id.as_str())
                || !label(&s.label)
                || s.vlans.len() > 64
                || !s.vlans.iter().all(|n| vlan(*n))
                || s.ports.is_empty()
                || s.ports.len() > 16
                || s.ports
                    .iter()
                    .any(|(id, p)| !identifier(id) || !p.mode.valid())
            {
                return Err("Invalid switch, VLAN database or port configuration".into());
            }
        }
        for (id, h) in &self.hosts {
            if !identifier(id) || !devices.insert(id.as_str()) || !label(&h.label) {
                return Err("Invalid/duplicate workstation identifier or label".into());
            }
        }
        let mut cable_ids = BTreeSet::new();
        let mut jacks = BTreeSet::new();
        let mut roots: BTreeMap<String, String> = devices
            .iter()
            .map(|d| (d.to_string(), d.to_string()))
            .collect();
        fn root(roots: &BTreeMap<String, String>, name: &str) -> String {
            let mut here = name;
            while roots[here] != here {
                here = &roots[here];
            }
            here.to_string()
        }
        for cable in &self.cables {
            if !identifier(&cable.id)
                || !cable_ids.insert(&cable.id)
                || matches!((&cable.a, &cable.b), (Jack::Host { .. }, Jack::Host { .. }))
            {
                return Err(
                    "Invalid/duplicate cable or unsupported direct workstation cable".into(),
                );
            }
            for jack in [&cable.a, &cable.b] {
                let exists = match jack {
                    Jack::Switch { switch, port } => self
                        .switches
                        .get(switch)
                        .is_some_and(|s| s.ports.contains_key(port)),
                    Jack::Host { host } => self.hosts.contains_key(host),
                };
                if !exists || !jacks.insert(jack) {
                    return Err("Unknown or reused physical switching jack".into());
                }
            }
            let a = root(&roots, cable.a.device());
            let b = root(&roots, cable.b.device());
            if a == b {
                return Err("Redundant/cyclic physical switching paths are unsupported until STP is modeled".into());
            }
            roots.insert(a, b);
        }
        Ok(())
    }
    /// A live physical route is required independently of VLAN filtering.
    /// This is a grading guard, not evidence of broadcast delivery.
    pub fn live_path(&self, a: &str, b: &str) -> Result<bool, String> {
        self.validate()?;
        if a == b || !self.hosts.contains_key(a) || !self.hosts.contains_key(b) {
            return Err("Choose two distinct known workstations".into());
        }
        if !self.hosts[a].up || !self.hosts[b].up {
            return Ok(false);
        }
        let operational = |jack: &Jack| match jack {
            Jack::Host { host } => self.hosts[host].up,
            Jack::Switch { switch, port } => self.switches[switch].ports[port].up,
        };
        let mut pending = VecDeque::from([a]);
        let mut seen = BTreeSet::new();
        while let Some(device) = pending.pop_front() {
            if !seen.insert(device) {
                continue;
            }
            if device == b {
                return Ok(true);
            }
            for cable in &self.cables {
                if !cable.up || !operational(&cable.a) || !operational(&cable.b) {
                    continue;
                }
                if cable.a.device() == device {
                    pending.push_back(cable.b.device());
                } else if cable.b.device() == device {
                    pending.push_back(cable.a.device());
                }
            }
        }
        Ok(false)
    }
    /// Broadcast visibility only: no ICMP, ARP, MAC learning or protocol-control simulation.
    pub fn broadcast(&self, source: &str) -> Result<Broadcast, String> {
        self.validate()?;
        let h = self.hosts.get(source).ok_or("Unknown source workstation")?;
        let mut result = Broadcast::default();
        if !h.up {
            result.drop_reason(format!("{source} is stopped"));
            return Ok(result);
        }
        let mut peers = BTreeMap::new();
        for cable in &self.cables {
            peers.insert(cable.a.clone(), (cable.b.clone(), cable.up));
            peers.insert(cable.b.clone(), (cable.a.clone(), cable.up));
        }
        let Some((peer, up)) = peers.get(&Jack::Host {
            host: source.into(),
        }) else {
            result.drop_reason(format!("{source} has no cable"));
            return Ok(result);
        };
        if !up {
            result.drop_reason(format!("{source}'s cable is down"));
            return Ok(result);
        }
        let Jack::Switch { switch, port } = peer else {
            return Err("Unsupported workstation-to-workstation attachment".into());
        };
        let mut pending = VecDeque::from([(switch.clone(), port.clone(), None, Vec::<Hop>::new())]);
        let mut visited = BTreeSet::new();
        let mut work = 0;
        while let Some((switch, ingress, tag, path)) = pending.pop_front() {
            work += 1;
            if work > 512 {
                return Err("Switching propagation limit exceeded".into());
            }
            if !visited.insert((switch.clone(), ingress.clone(), tag)) {
                continue;
            }
            let s = &self.switches[&switch];
            let input = &s.ports[&ingress];
            if !input.up {
                result.drop_reason(format!("{switch}/{ingress} is down"));
                continue;
            }
            let local = match (&input.mode, tag) {
                (Mode::Access { vlan }, None) => *vlan,
                (Mode::Access { .. }, Some(_)) => {
                    return Err(format!(
                        "Unsupported tagged ingress on access port {switch}/{ingress}; platform/voice-VLAN behavior is outside this subset"
                    ));
                }
                (Mode::Trunk { native, .. }, None) => *native,
                (Mode::Trunk { .. }, Some(vlan)) => vlan,
            };
            if !s.vlans.contains(&local) {
                result.drop_reason(format!("VLAN {local} is not active on {switch}"));
                continue;
            }
            if let Mode::Trunk { allowed, .. } = &input.mode
                && !allowed.contains(&local)
            {
                result.drop_reason(format!(
                    "VLAN {local} is not allowed into {switch}/{ingress}"
                ));
                continue;
            }
            for (egress, output) in &s.ports {
                work += 1;
                if work > 512 {
                    return Err("Switching propagation limit exceeded".into());
                }
                if egress == &ingress {
                    continue;
                }
                let Some((peer, up)) = peers.get(&Jack::Switch {
                    switch: switch.clone(),
                    port: egress.clone(),
                }) else {
                    continue;
                };
                if !output.up || !*up {
                    result.drop_reason(format!("{switch}/{egress} or its cable is down"));
                    continue;
                }
                let Some(out_tag) = output.mode.egress(local) else {
                    result.drop_reason(format!("{switch}/{egress} does not carry VLAN {local}"));
                    continue;
                };
                let mut next_path = path.clone();
                if next_path.len() >= 32 {
                    return Err("Switching trace path limit exceeded".into());
                }
                next_path.push(Hop {
                    switch: switch.clone(),
                    ingress: ingress.clone(),
                    vlan: local,
                    egress: egress.clone(),
                    tag: out_tag,
                });
                match peer {
                    Jack::Switch { switch, port } => {
                        pending.push_back((switch.clone(), port.clone(), out_tag, next_path))
                    }
                    Jack::Host { host } => {
                        if !self.hosts[host].up {
                            result.drop_reason(format!("{host} is stopped"));
                        } else if out_tag.is_some() {
                            result.drop_reason(format!("{host} accepts only untagged frames"));
                        } else {
                            result.recipients.insert(host.clone(), next_path);
                        }
                    }
                }
            }
        }
        Ok(result)
    }
    /// Requires real known endpoints; malformed/unsupported behavior cannot count as isolation.
    pub fn visibility(&self, a: &str, b: &str) -> Result<Visibility, String> {
        if a == b || !self.hosts.contains_key(a) || !self.hosts.contains_key(b) {
            return Err("Choose two distinct known workstations".into());
        }
        let forward = self.broadcast(a)?;
        let reverse = self.broadcast(b)?;
        Ok(Visibility {
            forward: forward.recipients.contains_key(b),
            reverse: reverse.recipients.contains_key(a),
        })
    }
}

/// State-based outcomes; negative policy checks always require operational attachments.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Requirement {
    Visibility {
        label: String,
        from: String,
        to: String,
        visible: bool,
    },
    AccessVlan {
        switch: String,
        port: String,
        vlan: u16,
    },
    NativeVlan {
        switch: String,
        port: String,
        vlan: u16,
    },
    ActiveVlan {
        switch: String,
        vlan: u16,
    },
}
impl Fabric {
    /// Port configuration may change; the authored equipment/cabling cannot be replaced.
    pub fn same_shape(&self, initial: &Self) -> bool {
        self.validate().is_ok()
            && initial.validate().is_ok()
            && self.hosts == initial.hosts
            && self.cables == initial.cables
            && self.switches.keys().eq(initial.switches.keys())
            && self.switches.iter().all(|(id, s)| {
                s.label == initial.switches[id].label
                    && s.ports.keys().eq(initial.switches[id].ports.keys())
            })
    }
    fn active_attachment(&self, host: &str) -> bool {
        let jack = Jack::Host { host: host.into() };
        self.cables
            .iter()
            .find_map(|c| {
                if c.a == jack {
                    Some(&c.b)
                } else if c.b == jack {
                    Some(&c.a)
                } else {
                    None
                }
            })
            .is_some_and(|j| match j {
                Jack::Switch { switch, port } => match self.switches[switch].ports[port].mode {
                    Mode::Access { vlan } => self.switches[switch].vlans.contains(&vlan),
                    _ => false,
                },
                _ => false,
            })
    }
}
impl Requirement {
    pub fn valid_for(&self, f: &Fabric) -> bool {
        if f.validate().is_err() {
            return false;
        }
        match self {
            Self::Visibility {
                label: description,
                from,
                to,
                ..
            } => {
                label(description)
                    && from != to
                    && f.hosts.contains_key(from)
                    && f.hosts.contains_key(to)
            }
            Self::AccessVlan {
                switch,
                port,
                vlan: n,
            }
            | Self::NativeVlan {
                switch,
                port,
                vlan: n,
            } => {
                vlan(*n)
                    && f.switches
                        .get(switch)
                        .is_some_and(|s| s.ports.contains_key(port))
            }
            Self::ActiveVlan { switch, vlan: n } => vlan(*n) && f.switches.contains_key(switch),
        }
    }
    pub fn check(&self, f: &Fabric) -> (bool, String) {
        if !self.valid_for(f) {
            return (false, "Invalid switching objective or topology".into());
        }
        match self {
            Self::Visibility {
                label,
                from,
                to,
                visible,
            } => {
                if !f.live_path(from, to).unwrap_or(false)
                    || !f.active_attachment(from)
                    || !f.active_attachment(to)
                {
                    return (
                        false,
                        format!(
                            "{label}: keep both workstations attached to up access ports, with their VLANs active and every link in the path up. Disconnection does not prove isolation."
                        ),
                    );
                }
                match f.visibility(from, to) {
                    Err(e) => (false, format!("{label}: {e}")),
                    Ok(v) => {
                        let okay = if *visible { v.mutual() } else { v.isolated() };
                        (
                            okay,
                            format!(
                                "{label}: {from} to {to} {}; {to} to {from} {}. {}",
                                if v.forward { "delivered" } else { "blocked" },
                                if v.reverse { "delivered" } else { "blocked" },
                                if *visible {
                                    "Both directions must deliver."
                                } else {
                                    "Neither direction may deliver."
                                }
                            ),
                        )
                    }
                }
            }
            Self::AccessVlan { switch, port, vlan } => {
                let p = &f.switches[switch].ports[port];
                (
                    p.up && matches!(p.mode,Mode::Access{vlan:n}if n==*vlan),
                    format!("{switch}/{port}: use an up access port in VLAN {vlan}"),
                )
            }
            Self::NativeVlan { switch, port, vlan } => {
                let p = &f.switches[switch].ports[port];
                (
                    p.up && matches!(p.mode,Mode::Trunk{native,..}if native==*vlan),
                    format!("{switch}/{port}: use an up trunk with native VLAN {vlan}"),
                )
            }
            Self::ActiveVlan { switch, vlan } => (
                f.switches[switch].vlans.contains(vlan),
                format!("{switch}: VLAN {vlan} is active"),
            ),
        }
    }
}

/// Every negative endpoint needs an authored positive service case, rather than shutdown credit.
pub fn valid_requirements(requirements: &[Requirement], f: &Fabric) -> bool {
    if requirements.is_empty()
        || requirements.len() > 20
        || requirements.iter().any(|r| !r.valid_for(f))
    {
        return false;
    }
    let positive: BTreeSet<_> = requirements
        .iter()
        .filter_map(|r| {
            if let Requirement::Visibility {
                from,
                to,
                visible: true,
                ..
            } = r
            {
                Some([from, to])
            } else {
                None
            }
        })
        .flatten()
        .collect();
    !positive.is_empty()
        && requirements.iter().all(|r| match r {
            Requirement::Visibility {
                from,
                to,
                visible: false,
                ..
            } => positive.contains(from) && positive.contains(to),
            _ => true,
        })
}
