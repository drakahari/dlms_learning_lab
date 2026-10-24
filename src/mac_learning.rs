//! Bounded Ethernet learning snapshots over the existing VLAN forest.
//! No sockets, timers, host I/O, ARP, STP, packet bytes or command execution.
use crate::switching::{Fabric, Jack, Mode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MAX_ENTRIES: usize = 256;
pub const MAX_EVENTS: usize = 512;
/// Compact serialized payload; pretty input has its own finite ceiling.
pub const MAX_STATE_BYTES: usize = 262_144;
pub const MAX_INPUT_BYTES: usize = 1_048_576;
pub const MAX_ERROR_BYTES: usize = 512;
const MAX_ADVANCE: u64 = 86_400;
fn diagnostic(text: String) -> String {
    let clipped = text.len() > MAX_ERROR_BYTES;
    let mut end = text.len().min(if clipped {
        MAX_ERROR_BYTES - 3
    } else {
        MAX_ERROR_BYTES
    });
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    // Always allocate fresh: short length alone does not bound a retained
    // String whose previous allocation held a much larger serde diagnostic.
    let mut bounded = String::with_capacity(end + if clipped { 3 } else { 0 });
    bounded.push_str(&text[..end]);
    if clipped {
        bounded.push_str("...");
    }
    bounded
}

/// Display only: comparisons still use the complete validated host sets.
/// Separate list bounds leave space for both actual and required in feedback.
fn host_list(hosts: &BTreeSet<String>) -> String {
    const LIMIT: usize = 160;
    if hosts.is_empty() {
        return "none".into();
    }
    let mut out = String::with_capacity(LIMIT);
    for host in hosts {
        let separator = if out.is_empty() { "" } else { ", " };
        if out.len() + separator.len() + host.len() > LIMIT - 3 {
            let room = (LIMIT - 3).saturating_sub(out.len() + separator.len());
            if room > 0 {
                out.push_str(separator);
                let mut end = room.min(host.len());
                while !host.is_char_boundary(end) {
                    end -= 1;
                }
                out.push_str(&host[..end]);
            }
            out.push_str("...");
            break;
        }
        out.push_str(separator);
        out.push_str(host);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mac(pub [u8; 6]);
impl Mac {
    pub const BROADCAST: Self = Self([255; 6]);
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() != 17 {
            return Err("Use six hexadecimal pairs separated by colons".into());
        }
        let mut value = [0; 6];
        for (i, part) in text.split(':').enumerate() {
            if i >= 6 || part.len() != 2 || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("Use six hexadecimal pairs separated by colons".into());
            }
            value[i] = u8::from_str_radix(part, 16).map_err(|_| "Invalid MAC address")?;
        }
        Ok(Self(value))
    }
    pub fn unicast(self) -> bool {
        self.0[0] & 1 == 0 && self.0 != [0; 6]
    }
    pub fn text(self) -> String {
        self.0
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":")
    }
}
impl Serialize for Mac {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.text())
    }
}
impl<'de> Deserialize<'de> for Mac {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Address;
        impl serde::de::Visitor<'_> for Address {
            type Value = Mac;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("six hexadecimal MAC pairs")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Mac, E> {
                Mac::parse(s).map_err(E::custom)
            }
        }
        d.deserialize_str(Address)
    }
}
fn name64<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    struct Name;
    impl serde::de::Visitor<'_> for Name {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("one bounded switching identifier")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if !s.is_empty()
                && s.len() <= 64
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                Ok(s.into())
            } else {
                Err(E::custom("Invalid switching identifier"))
            }
        }
    }
    d.deserialize_str(Name)
}
fn addresses<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Mac>, D::Error> {
    struct Key(String);
    impl<'de> Deserialize<'de> for Key {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            name64(d).map(Key)
        }
    }
    struct Map;
    impl<'de> serde::de::Visitor<'de> for Map {
        type Value = BTreeMap<String, Mac>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most 32 unique host MAC assignments")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> Result<Self::Value, A::Error> {
            use serde::de::Error as _;
            let mut out = BTreeMap::new();
            while let Some(Key(key)) = a.next_key::<Key>()? {
                if out.len() >= 32 || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Duplicate host MAC assignment or collection limit",
                    ));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Map)
}
fn entries<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Entry>, D::Error> {
    struct Excess;
    impl<'de> Deserialize<'de> for Excess {
        fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Err(serde::de::Error::custom("Dynamic MAC table limit exceeded"))
        }
    }
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = Vec<Entry>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most 256 MAC entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            loop {
                if out.len() == MAX_ENTRIES {
                    let _ = a.next_element::<Excess>()?;
                    return Ok(out);
                }
                match a.next_element()? {
                    Some(e) => out.push(e),
                    None => return Ok(out),
                }
            }
        }
    }
    d.deserialize_seq(Entries)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "name64")]
    pub switch: String,
    pub vlan: u16,
    pub address: Mac,
    #[serde(deserialize_with = "name64")]
    pub port: String,
    pub learned_at: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    schema_version: u8,
    fabric: Fabric,
    host_addresses: BTreeMap<String, Mac>,
    entries: Vec<Entry>,
    now: u64,
    aging_seconds: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u8,
    fabric: Fabric,
    #[serde(deserialize_with = "addresses")]
    host_addresses: BTreeMap<String, Mac>,
    #[serde(deserialize_with = "entries")]
    entries: Vec<Entry>,
    now: u64,
    aging_seconds: u16,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = Wire::deserialize(d)
            .map_err(|e| <D::Error as serde::de::Error>::custom(diagnostic(e.to_string())))?;
        let s = Self {
            schema_version: w.schema_version,
            fabric: w.fabric,
            host_addresses: w.host_addresses,
            entries: w.entries,
            now: w.now,
            aging_seconds: w.aging_seconds,
        };
        s.validate().map_err(serde::de::Error::custom)?;
        Ok(s)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    UnknownUnicast,
    Broadcast,
    KnownUnicast,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Receive {
        switch: String,
        ingress: String,
        vlan: u16,
    },
    Learn {
        switch: String,
        vlan: u16,
        source: Mac,
        port: String,
        previous_port: Option<String>,
    },
    Forward {
        switch: String,
        ingress: String,
        egress: String,
        vlan: u16,
        tag: Option<u16>,
        reason: Reason,
    },
    Filter {
        switch: String,
        ingress: String,
        vlan: u16,
    },
    Drop {
        location: String,
        explanation: String,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Physical NIC arrivals, including flooded unicast meant for somebody else.
    pub arrivals: BTreeSet<String>,
    /// Individual/broadcast frames accepted by the modeled untagged endpoint.
    /// This is not IP/application success or a hardware NIC conformance claim.
    pub accepted_by: BTreeSet<String>,
    pub events: Vec<Event>,
}
impl Observation {
    fn push(&mut self, e: Event) -> Result<(), String> {
        if self.events.len() >= MAX_EVENTS {
            return Err("Frame trace limit reached; no state changed".into());
        }
        self.events.push(e);
        Ok(())
    }
    fn drop_at(&mut self, location: String, explanation: &str) -> Result<(), String> {
        self.push(Event::Drop {
            location,
            explanation: explanation.into(),
        })
    }
}
fn egress(mode: &Mode, vlan: u16) -> Option<Option<u16>> {
    match mode {
        Mode::Access { vlan: v } if *v == vlan => Some(None),
        Mode::Trunk { native, allowed } if allowed.contains(&vlan) => {
            Some(if *native == vlan { None } else { Some(vlan) })
        }
        _ => None,
    }
}
impl State {
    pub fn new(
        fabric: Fabric,
        host_addresses: BTreeMap<String, Mac>,
        aging_seconds: u16,
    ) -> Result<Self, String> {
        let s = Self {
            schema_version: 1,
            fabric,
            host_addresses,
            entries: vec![],
            now: 0,
            aging_seconds,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn decode(text: &str) -> Result<Self, String> {
        if text.len() > MAX_INPUT_BYTES {
            return Err("MAC-learning JSON input exceeds 1 MiB".into());
        }
        serde_json::from_str(text).map_err(|e| diagnostic(e.to_string()))
    }
    pub fn validate(&self) -> Result<(), String> {
        self.fabric.validate()?;
        if self.schema_version != 1
            || self.aging_seconds > 3600
            || self.entries.len() > MAX_ENTRIES
            || !self.fabric.hosts.keys().eq(self.host_addresses.keys())
            || self.host_addresses.values().any(|m| !m.unicast())
            || self.host_addresses.values().collect::<BTreeSet<_>>().len()
                != self.host_addresses.len()
        {
            return Err("Invalid or unsupported MAC-learning state".into());
        }
        let mut keys = BTreeSet::new();
        for e in &self.entries {
            if !e.address.unicast()
                || e.learned_at > self.now
                || (self.aging_seconds > 0
                    && self.now - e.learned_at >= u64::from(self.aging_seconds))
                || !keys.insert((&e.switch, e.vlan, e.address))
                || !self
                    .fabric
                    .switches
                    .get(&e.switch)
                    .is_some_and(|s| s.vlans.contains(&e.vlan) && s.ports.contains_key(&e.port))
            {
                return Err("Invalid, duplicate, future or expired MAC-table entry".into());
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_STATE_BYTES {
            return Err("MAC-learning state exceeds 256 KiB".into());
        }
        Ok(())
    }
    /// An empty delivery result must not count as VLAN separation when the
    /// endpoint/physical route has simply been disabled. VLAN filtering itself
    /// is intentionally not part of this independent physical-path guard.
    fn attachment_health(&self, host: &str) -> Result<(), String> {
        let h = self.fabric.hosts.get(host).ok_or("Unknown workstation")?;
        if !h.up {
            return Err(format!("{host}: the workstation is stopped"));
        }
        let endpoint = Jack::Host { host: host.into() };
        let (cable, peer) = self
            .fabric
            .cables
            .iter()
            .find_map(|c| {
                if c.a == endpoint {
                    Some((c, &c.b))
                } else if c.b == endpoint {
                    Some((c, &c.a))
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("{host}: no physical attachment"))?;
        if !cable.up {
            return Err(format!("{host}: the cable is down"));
        }
        let Jack::Switch { switch, port } = peer else {
            return Err("Unsupported workstation attachment".into());
        };
        let sw = &self.fabric.switches[switch];
        let p = &sw.ports[port];
        if !p.up {
            return Err(format!("{host}: {switch}/{port} port is down"));
        }
        let local = match &p.mode {
            Mode::Access { vlan } => *vlan,
            Mode::Trunk { native, allowed } if allowed.contains(native) => *native,
            Mode::Trunk { .. } => {
                return Err(format!("{host}: the untagged input VLAN is not allowed"));
            }
        };
        if !sw.vlans.contains(&local) {
            return Err(format!("{host}: its untagged input VLAN is inactive"));
        }
        Ok(())
    }
    pub fn fabric(&self) -> &Fabric {
        &self.fabric
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn now(&self) -> u64 {
        self.now
    }
    pub fn aging_seconds(&self) -> u16 {
        self.aging_seconds
    }
    pub fn address(&self, host: &str) -> Option<Mac> {
        self.host_addresses.get(host).copied()
    }
    /// Explicit virtual time only. Destination lookups never refresh an entry.
    pub fn advance(&mut self, seconds: u64) -> Result<(), String> {
        self.validate()?;
        if seconds > MAX_ADVANCE {
            return Err("Advance at most one simulated day at a time".into());
        }
        let next = self
            .now
            .checked_add(seconds)
            .ok_or("Simulated clock limit reached")?;
        self.now = next;
        if self.aging_seconds > 0 {
            self.entries
                .retain(|e| next - e.learned_at < u64::from(self.aging_seconds));
        }
        Ok(())
    }
    pub fn clear_dynamic(&mut self, switch: &str, vlan: Option<u16>) -> Result<usize, String> {
        self.validate()?;
        let s = self.fabric.switches.get(switch).ok_or("Unknown switch")?;
        if vlan.is_some_and(|v| !s.vlans.contains(&v)) {
            return Err("Choose an active VLAN on this switch".into());
        }
        let before = self.entries.len();
        self.entries
            .retain(|e| e.switch != switch || vlan.is_some_and(|v| e.vlan != v));
        Ok(before - self.entries.len())
    }
    /// Changes to the physical topology/MAC identity require a separate activity.
    /// Reconfiguration flushes entries on changed ports or removed VLANs.
    pub fn configure(&mut self, fabric: Fabric, aging_seconds: u16) -> Result<(), String> {
        self.validate()?;
        if !fabric.same_shape(&self.fabric) || aging_seconds > 3600 {
            return Err(
                "Keep the authored equipment, cables and host identities; aging is 0–3600 seconds"
                    .into(),
            );
        }
        let old = &self.fabric;
        let mut entries = self.entries.clone();
        entries.retain(|e| {
            let s = &fabric.switches[&e.switch];
            s.vlans.contains(&e.vlan)
                && s.ports[&e.port] == old.switches[&e.switch].ports[&e.port]
                && (aging_seconds == 0 || self.now - e.learned_at < u64::from(aging_seconds))
        });
        let trial = Self {
            schema_version: self.schema_version,
            fabric,
            entries,
            host_addresses: self.host_addresses.clone(),
            now: self.now,
            aging_seconds,
        };
        trial.validate()?;
        *self = trial;
        Ok(())
    }
    /// One bounded metadata frame, not a real Ethernet transmission. Unsupported
    /// inputs/resource faults are atomic; ordinary modeled drops retain learning.
    pub fn send(&mut self, source: &str, destination: Mac) -> Result<Observation, String> {
        self.validate()?;
        if !self.host_addresses.contains_key(source) {
            return Err("Choose a known source workstation".into());
        }
        if !destination.unicast() && destination != Mac::BROADCAST {
            return Err(
                "Unsupported multicast MAC: use an individual address or ff:ff:ff:ff:ff:ff".into(),
            );
        }
        let mut trial = self.clone();
        let result = trial.propagate(source, destination)?;
        trial.validate()?;
        *self = trial;
        Ok(result)
    }
    fn learn(
        &mut self,
        switch: &str,
        vlan: u16,
        mac: Mac,
        port: &str,
    ) -> Result<Option<String>, String> {
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| e.switch == switch && e.vlan == vlan && e.address == mac)
        {
            let previous = (e.port != port).then(|| e.port.clone());
            e.port = port.into();
            e.learned_at = self.now;
            return Ok(previous);
        }
        if self.entries.len() >= MAX_ENTRIES {
            return Err("Dynamic MAC table limit reached; no state changed".into());
        }
        self.entries.push(Entry {
            switch: switch.into(),
            vlan,
            address: mac,
            port: port.into(),
            learned_at: self.now,
        });
        Ok(None)
    }
    fn propagate(&mut self, source: &str, destination: Mac) -> Result<Observation, String> {
        let mut out = Observation::default();
        if !self.fabric.hosts[source].up {
            out.drop_at(source.into(), "The source workstation is stopped")?;
            return Ok(out);
        }
        let source_mac = self.host_addresses[source];
        let mut peers = BTreeMap::new();
        for c in &self.fabric.cables {
            peers.insert(c.a.clone(), (c.b.clone(), c.up));
            peers.insert(c.b.clone(), (c.a.clone(), c.up));
        }
        let Some((start, up)) = peers.get(&Jack::Host {
            host: source.into(),
        }) else {
            out.drop_at(source.into(), "No physical attachment")?;
            return Ok(out);
        };
        if !*up {
            out.drop_at(source.into(), "The source cable is down")?;
            return Ok(out);
        }
        let Jack::Switch { switch, port } = start else {
            return Err("Direct workstation cable is unsupported".into());
        };
        let mut pending = VecDeque::from([(switch.clone(), port.clone(), None)]);
        let mut seen = BTreeSet::new();
        while let Some((switch, ingress, tag)) = pending.pop_front() {
            if !seen.insert(switch.clone()) {
                return Err(
                    "A frame reached one switch twice; cyclic propagation is unsupported".into(),
                );
            }
            let s = &self.fabric.switches[&switch];
            let input = &s.ports[&ingress];
            let location = format!("{switch}/{ingress}");
            if !input.up {
                out.drop_at(location, "The incoming port is down")?;
                continue;
            }
            let vlan = match (&input.mode, tag) {
                (Mode::Access { vlan }, None) => *vlan,
                (Mode::Access { .. }, Some(_)) => return Err(
                    "Unsupported tagged input on an access port; inspect the inter-switch modes"
                        .into(),
                ),
                (Mode::Trunk { native, .. }, None) => *native,
                (Mode::Trunk { .. }, Some(v)) => v,
            };
            if !s.vlans.contains(&vlan)
                || matches!(&input.mode,Mode::Trunk{allowed,..} if !allowed.contains(&vlan))
            {
                out.drop_at(location, "The incoming VLAN is inactive or not allowed")?;
                continue;
            }
            out.push(Event::Receive {
                switch: switch.clone(),
                ingress: ingress.clone(),
                vlan,
            })?;
            let previous_port = self.learn(&switch, vlan, source_mac, &ingress)?;
            out.push(Event::Learn {
                switch: switch.clone(),
                vlan,
                source: source_mac,
                port: ingress.clone(),
                previous_port,
            })?;
            let known = self
                .entries
                .iter()
                .find(|e| e.switch == switch && e.vlan == vlan && e.address == destination)
                .map(|e| e.port.clone());
            if known.as_deref() == Some(ingress.as_str()) {
                out.push(Event::Filter {
                    switch,
                    ingress,
                    vlan,
                })?;
                continue;
            }
            let reason = if destination == Mac::BROADCAST {
                Reason::Broadcast
            } else if known.is_some() {
                Reason::KnownUnicast
            } else {
                Reason::UnknownUnicast
            };
            for (egress_port, port) in &self.fabric.switches[&switch].ports {
                if egress_port == &ingress || known.as_ref().is_some_and(|k| k != egress_port) {
                    continue;
                }
                let Some((peer, cable_up)) = peers.get(&Jack::Switch {
                    switch: switch.clone(),
                    port: egress_port.clone(),
                }) else {
                    continue;
                };
                if !port.up || !*cable_up {
                    out.drop_at(
                        format!("{switch}/{egress_port}"),
                        "The outgoing port or cable is down",
                    )?;
                    continue;
                }
                let Some(tag) = egress(&port.mode, vlan) else {
                    if known.is_some() {
                        out.drop_at(
                            format!("{switch}/{egress_port}"),
                            "The known destination port does not carry this VLAN",
                        )?;
                    }
                    continue;
                };
                out.push(Event::Forward {
                    switch: switch.clone(),
                    ingress: ingress.clone(),
                    egress: egress_port.clone(),
                    vlan,
                    tag,
                    reason,
                })?;
                match peer {
                    Jack::Switch { switch, port } => {
                        pending.push_back((switch.clone(), port.clone(), tag))
                    }
                    Jack::Host { host } => {
                        if !self.fabric.hosts[host].up {
                            out.drop_at(host.clone(), "The destination workstation is stopped")?;
                        } else {
                            out.arrivals.insert(host.clone());
                            if tag.is_some() {
                                out.drop_at(
                                    host.clone(),
                                    "This modeled endpoint accepts untagged frames only",
                                )?;
                            } else if destination == Mac::BROADCAST
                                || self.host_addresses[host] == destination
                            {
                                out.accepted_by.insert(host.clone());
                            }
                        }
                    }
                }
            }
        }
        Ok(out)
    }
}

/// Proposed state/outcome grading, not command recognition. Each frame uses an
/// independent clone of the current state so earlier checks cannot teach later ones.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Learned {
        switch: String,
        vlan: u16,
        address: Mac,
        port: String,
    },
    Frame {
        label: String,
        from: String,
        destination: Mac,
        after_seconds: u64,
        arrivals: BTreeSet<String>,
        accepted_by: BTreeSet<String>,
        forwarding: Option<Reason>,
    },
    Aging {
        minimum: u16,
        maximum: u16,
    },
}
impl Requirement {
    pub fn valid_for(&self, s: &State) -> bool {
        match self {
            Self::Learned {
                switch,
                vlan,
                address,
                port,
            } => {
                address.unicast()
                    && s.fabric
                        .switches
                        .get(switch)
                        .is_some_and(|sw| sw.vlans.contains(vlan) && sw.ports.contains_key(port))
            }
            Self::Frame {
                label,
                from,
                destination,
                after_seconds,
                arrivals,
                accepted_by,
                forwarding,
            } => {
                !label.trim().is_empty()
                    && label.len() <= 160
                    && !label.chars().any(char::is_control)
                    && s.fabric.hosts.contains_key(from)
                    && (destination.unicast() || *destination == Mac::BROADCAST)
                    && *after_seconds <= MAX_ADVANCE
                    && arrivals.len() <= 32
                    && accepted_by.len() <= 32
                    && accepted_by.is_subset(arrivals)
                    && arrivals
                        .iter()
                        .all(|h| s.fabric.hosts.contains_key(h) && h != from)
                    && (forwarding.is_none() || !accepted_by.is_empty())
            }
            Self::Aging { minimum, maximum } => minimum <= maximum && *maximum <= 3600,
        }
    }
    pub fn check(&self, s: &State) -> (bool, String) {
        let (passed, text) = self.check_details(s);
        (passed, diagnostic(text))
    }
    fn check_details(&self, s: &State) -> (bool, String) {
        if s.validate().is_err() || !self.valid_for(s) {
            return (
                false,
                "This requirement has invalid or unsupported network data".into(),
            );
        }
        match self {
            Self::Learned {
                switch,
                vlan,
                address,
                port,
            } => {
                let actual = s
                    .entries
                    .iter()
                    .find(|e| e.switch == *switch && e.vlan == *vlan && e.address == *address);
                let ok = actual.is_some_and(|e| e.port == *port);
                (
                    ok,
                    if ok {
                        format!(
                            "{switch}: {} is learned on {port} in VLAN {vlan}",
                            address.text()
                        )
                    } else if let Some(e) = actual {
                        format!(
                            "{switch}: {} currently points to {} in VLAN {vlan}; inspect where that source last arrived",
                            address.text(),
                            e.port
                        )
                    } else {
                        format!(
                            "{switch}: {} has not been learned in VLAN {vlan}. A switch learns a source when it receives a frame from it",
                            address.text()
                        )
                    },
                )
            }
            Self::Aging { minimum, maximum } => {
                let ok = (*minimum..=*maximum).contains(&s.aging_seconds);
                (
                    ok,
                    if ok {
                        format!(
                            "The dynamic-entry lifetime is {} simulated seconds",
                            s.aging_seconds
                        )
                    } else {
                        format!(
                            "Use an aging time between {minimum} and {maximum} simulated seconds for this task; zero disables expiry"
                        )
                    },
                )
            }
            Self::Frame {
                label,
                from,
                destination,
                after_seconds,
                arrivals,
                accepted_by,
                forwarding,
            } => {
                if accepted_by.is_empty() {
                    if let Err(e) = s.attachment_health(from) {
                        return (
                            false,
                            format!(
                                "{label}: {e}. Keep the physical attachment working before checking separation"
                            ),
                        );
                    }
                    if let Some((target, _)) =
                        s.host_addresses.iter().find(|(_, mac)| *mac == destination)
                    {
                        if let Err(e) = s.attachment_health(target) {
                            return (
                                false,
                                format!(
                                    "{label}: {e}. A stopped destination does not prove separation"
                                ),
                            );
                        }
                        if target != from && !matches!(s.fabric.live_path(from, target), Ok(true)) {
                            return (
                                false,
                                format!(
                                    "{label}: no working physical path between the endpoints. A disabled link does not prove VLAN separation"
                                ),
                            );
                        }
                    }
                }
                let mut trial = s.clone();
                if let Err(e) = trial.advance(*after_seconds) {
                    return (false, format!("{label}: {e}"));
                }
                let out = match trial.send(from, *destination) {
                    Ok(o) => o,
                    Err(e) => return (false, format!("{label}: {e}")),
                };
                if out.accepted_by != *accepted_by {
                    return (
                        false,
                        format!(
                            "{label}: accepted by {}; required {}. Inspect the current VLAN path and destination address",
                            host_list(&out.accepted_by),
                            host_list(accepted_by)
                        ),
                    );
                }
                if out.arrivals != *arrivals {
                    return (
                        false,
                        format!(
                            "{label}: copies reached {}; required {}. Unknown unicast can flood even when only its destination accepts it",
                            host_list(&out.arrivals),
                            host_list(arrivals)
                        ),
                    );
                }
                if let Some(reason) = forwarding {
                    let reasons = out
                        .events
                        .iter()
                        .filter_map(|e| {
                            if let Event::Forward { reason, .. } = e {
                                Some(reason)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if reasons.is_empty() || reasons.iter().any(|r| *r != reason) {
                        return (
                            false,
                            format!(
                                "{label}: forwarding is {:?}, not consistently {reason:?}; inspect each switch's VLAN-specific destination entry",
                                reasons
                            ),
                        );
                    }
                }
                (
                    true,
                    format!(
                        "{label}: current forwarding meets the task after {after_seconds} simulated seconds"
                    ),
                )
            }
        }
    }
}
pub fn valid_requirements(requirements: &[Requirement], s: &State) -> bool {
    !requirements.is_empty()
        && requirements.len() <= 64
        && s.validate().is_ok()
        && requirements.iter().all(|r| r.valid_for(s))
        && requirements
            .iter()
            .any(|r| matches!(r,Requirement::Frame{accepted_by,..}if !accepted_by.is_empty()))
}

#[cfg(test)]
mod retained_diagnostic_tests {
    #[test]
    fn diagnostics_release_large_capacity_and_keep_utf8_useful_prefix() {
        let mut source = String::with_capacity(200_000);
        source.push_str("Small diagnostic");
        let result = super::diagnostic(source);
        assert_eq!(result, "Small diagnostic");
        assert!(result.capacity() <= super::MAX_ERROR_BYTES);
        let result = super::diagnostic(format!("Unknown field: {}", "é".repeat(100_000)));
        assert!(result.starts_with("Unknown field:"));
        assert!(result.ends_with("..."));
        assert!(
            result.len() <= super::MAX_ERROR_BYTES && result.capacity() <= super::MAX_ERROR_BYTES
        );
    }
}

#[cfg(test)]
mod host_list_display_tests {
    use super::*;
    #[test]
    fn host_lists_are_plain_bounded_utf8_without_losing_comparison_data() {
        assert_eq!(host_list(&BTreeSet::new()), "none");
        assert_eq!(
            host_list(&BTreeSet::from(["archive".into(), "desk".into()])),
            "archive, desk"
        );
        let names = BTreeSet::from(["é".repeat(100_000)]);
        let text = host_list(&names);
        assert!(text.len() <= 160 && text.capacity() <= 160);
        assert!(text.ends_with("..."));
        assert_eq!(names.iter().next().unwrap().len(), 200_000);
    }
}
