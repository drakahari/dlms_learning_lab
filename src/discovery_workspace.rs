//! Saved native LLDP configuration drafts and pack-only typed reference actions.
//! Draft identity is pinned separately from editable text. Applying configuration never
//! advertises or advances time, though receive lifecycle/withdrawal rules still apply.
//! No host, socket, packet, shell, NIC or wall-clock access occurs in this module.
use crate::discovery::{
    Check, Endpoint, GlobalSettings, MAX_DEVICES, MAX_PORTS, Outcome, Requirement, State, Topology,
    Transmission,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, marker::PhantomData};

pub const MAX_DRAFT_BYTES: usize = 1_048_576;
pub const MAX_ACTION_BYTES: usize = 8192;
pub const MAX_DRAFT_TEXT: usize = 256;
pub const MAX_REQUIREMENTS: usize = 32;

fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn applied_label(s: &Option<String>) -> bool {
    s.as_ref().is_none_or(|s| {
        s.len() <= 80 && !s.trim().is_empty() && s.bytes().all(|b| (32..=126).contains(&b))
    })
}
fn bounded_map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct Entries<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique discovery draft keys")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key::<String>()? {
                if out.len() >= N || !key(&k) || out.contains_key(&k) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate discovery draft key or mapping limit",
                    ));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries::<V, N>(PhantomData))
}
pub(crate) fn bounded_list<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<V>, D::Error> {
    struct Entries<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<V, N> {
        type Value = Vec<V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} discovery teaching entries")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element()? {
                if out.len() >= N {
                    return Err(A::Error::custom(
                        "Discovery teaching collection limit exceeded",
                    ));
                }
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Entries::<V, N>(PhantomData))
}
fn device_drafts<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, DeviceDraft>, D::Error> {
    bounded_map::<D, DeviceDraft, MAX_DEVICES>(d)
}
fn port_drafts<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, PortDraft>, D::Error> {
    bounded_map::<D, PortDraft, MAX_PORTS>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortDraft {
    pub up: bool,
    pub transmit: bool,
    pub receive: bool,
    pub description: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceDraft {
    pub enabled: bool,
    pub ttl: String,
    pub system_name: Option<String>,
    #[serde(deserialize_with = "port_drafts")]
    pub ports: BTreeMap<String, PortDraft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub selected_device: String,
    #[serde(default = "default_advance")]
    pub advance_seconds: String,
    // The native editor never mutates this snapshot; settings inside it are ignored.
    // Only its validated identities/attachments anchor the draft to the simulation.
    identity: Topology,
    #[serde(deserialize_with = "device_drafts")]
    pub devices: BTreeMap<String, DeviceDraft>,
}
impl Draft {
    pub fn new(simulation: &State) -> Self {
        let topology = simulation.topology();
        Self {
            version: 1,
            advance_seconds: default_advance(),
            selected_device: topology.devices.keys().next().cloned().unwrap_or_default(),
            identity: topology.clone(),
            devices: topology
                .devices
                .iter()
                .map(|(id, d)| {
                    (
                        id.clone(),
                        DeviceDraft {
                            enabled: d.enabled,
                            ttl: d.ttl.to_string(),
                            system_name: d.system_name.clone(),
                            ports: d
                                .ports
                                .iter()
                                .map(|(id, p)| {
                                    (
                                        id.clone(),
                                        PortDraft {
                                            up: p.up,
                                            transmit: p.transmit,
                                            receive: p.receive,
                                            description: p.description.clone(),
                                        },
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        }
    }
    /// Storage/shape validity deliberately permits bounded, unapplied malformed text.
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.advance_seconds.len() <= 40
            && self.identity.validate().is_ok()
            && self.devices.contains_key(&self.selected_device)
            && self.devices.keys().eq(self.identity.devices.keys())
            && self.devices.iter().all(|(id, d)| {
                d.ttl.len() <= 40
                    && d.system_name
                        .as_ref()
                        .is_none_or(|s| s.len() <= MAX_DRAFT_TEXT)
                    && d.ports.keys().eq(self.identity.devices[id].ports.keys())
                    && d.ports.values().all(|p| {
                        p.description
                            .as_ref()
                            .is_none_or(|s| s.len() <= MAX_DRAFT_TEXT)
                    })
            })
    }
    pub fn same_shape(&self, simulation: &State) -> bool {
        self.valid()
            && simulation.validate().is_ok()
            && self.identity.same_identity(simulation.topology())
    }
    pub fn from_json(text: &str, simulation: &State) -> Result<Self, String> {
        if text.len() > MAX_DRAFT_BYTES {
            return Err("Discovery draft exceeds 1 MiB".into());
        }
        let draft: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid discovery draft JSON: {e}"))?;
        if !draft.same_shape(simulation) {
            return Err(
                "Discovery draft must preserve this simulation's identities and port shape".into(),
            );
        }
        Ok(draft)
    }
    /// Parses all drafts first and returns an identity-preserving configuration snapshot.
    /// Link up/down state comes from the applied simulation and is not editable here.
    pub fn snapshot(&self, simulation: &State) -> Result<Topology, String> {
        if !self.same_shape(simulation) {
            return Err("Discovery draft does not match this simulation's fixed topology".into());
        }
        let mut candidate = simulation.topology().clone();
        for (id, draft) in &self.devices {
            if !applied_label(&draft.system_name) {
                return Err(format!(
                    "{id}: advertised name needs 1–80 printable ASCII characters, or leave it empty to omit the name."
                ));
            }
            let ttl = if !draft.ttl.is_empty() && draft.ttl.bytes().all(|b| b.is_ascii_digit()) {
                draft.ttl.parse::<u16>().ok().filter(|n| *n > 0)
            } else {
                None
            }
            .ok_or_else(|| {
                format!("{id}: enter TTL as a decimal whole number from 1 through 65535")
            })?;
            let d = candidate
                .devices
                .get_mut(id)
                .ok_or("Unknown discovery draft device")?;
            d.enabled = draft.enabled;
            d.ttl = ttl;
            d.system_name = draft.system_name.clone();
            for (port, draft) in &draft.ports {
                if !applied_label(&draft.description) {
                    return Err(format!(
                        "{id}/{port}: port description needs 1–80 printable ASCII characters, or leave it empty to omit the description."
                    ));
                }
                let p = d
                    .ports
                    .get_mut(port)
                    .ok_or("Unknown discovery draft port")?;
                p.up = draft.up;
                p.transmit = draft.transmit;
                p.receive = draft.receive;
                p.description = draft.description.clone();
            }
        }
        candidate.validate().map_err(|e| e.message)?;
        Ok(candidate)
    }
    /// Atomic configuration application; no advertisements, refresh or time advancement.
    pub fn apply(&self, simulation: &mut State) -> Result<(), String> {
        let candidate = self.snapshot(simulation)?;
        simulation.reconfigure(candidate).map_err(|e| e.message)
    }
    pub fn matches(&self, simulation: &State) -> bool {
        self.snapshot(simulation)
            .is_ok_and(|candidate| candidate.same_configuration(simulation.topology()))
    }
}
fn default_advance() -> String {
    "30".into()
}

/// Pack reachability actions; not IOS commands and never dispatched to a shell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    GlobalConfiguration {
        device: String,
        settings: GlobalSettings,
    },
    PortConfiguration {
        local: Endpoint,
        up: bool,
        transmit: bool,
        receive: bool,
        description: Option<String>,
    },
    Advertise {
        from: Endpoint,
    },
    Withdraw {
        from: Endpoint,
    },
    Advance {
        seconds: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionResult {
    ConfigurationApplied,
    Transmission(Transmission),
    TimeAdvanced { now: u64 },
}
impl Action {
    pub fn is_configuration(&self) -> bool {
        matches!(
            self,
            Self::GlobalConfiguration { .. } | Self::PortConfiguration { .. }
        )
    }
    pub fn apply(&self, simulation: &mut State) -> Result<ActionResult, String> {
        simulation.validate().map_err(|e| e.message)?;
        match self {
            Self::GlobalConfiguration { device, settings } => {
                if settings.ttl == 0 || !applied_label(&settings.system_name) {
                    return Err("Global LLDP actions need TTL 1–65535 and a nonblank optional ASCII name of at most 80 bytes".into());
                }
                let mut next = simulation.topology().clone();
                let d = next
                    .devices
                    .get_mut(device)
                    .ok_or("Unknown discovery action device")?;
                d.enabled = settings.enabled;
                d.ttl = settings.ttl;
                d.system_name = settings.system_name.clone();
                simulation.reconfigure(next).map_err(|e| e.message)?;
                Ok(ActionResult::ConfigurationApplied)
            }
            Self::PortConfiguration {
                local,
                up,
                transmit,
                receive,
                description,
            } => {
                if !applied_label(description) {
                    return Err("Port LLDP actions need a nonblank optional ASCII description of at most 80 bytes".into());
                }
                let mut next = simulation.topology().clone();
                let p = next
                    .devices
                    .get_mut(&local.device)
                    .and_then(|d| d.ports.get_mut(&local.port))
                    .ok_or("Unknown discovery action port")?;
                p.up = *up;
                p.transmit = *transmit;
                p.receive = *receive;
                p.description = description.clone();
                simulation.reconfigure(next).map_err(|e| e.message)?;
                Ok(ActionResult::ConfigurationApplied)
            }
            Self::Advertise { from } => simulation
                .advertise(from)
                .map(ActionResult::Transmission)
                .map_err(|e| e.message),
            Self::Withdraw { from } => simulation
                .shutdown(from)
                .map(ActionResult::Transmission)
                .map_err(|e| e.message),
            Self::Advance { seconds } => {
                simulation.advance(*seconds).map_err(|e| e.message)?;
                Ok(ActionResult::TimeAdvanced {
                    now: simulation.now(),
                })
            }
        }
    }
}
pub fn parse_action(input: &str) -> Result<Action, String> {
    if input.len() > MAX_ACTION_BYTES {
        return Err("Discovery reference action exceeds 8192 bytes".into());
    }
    let json = input.strip_prefix("discovery ").ok_or(
        "Use native LLDP controls; pack actions require discovery followed by one JSON action",
    )?;
    serde_json::from_str(json).map_err(|e| format!("Invalid discovery reference action: {e}"))
}
pub fn adapter(simulation: &mut State, input: &str) -> Result<String, String> {
    let result = parse_action(input)?.apply(simulation)?;
    Ok(match result {
        ActionResult::ConfigurationApplied => {
            "Applied simulated LLDP configuration; advertising and time remain explicit".into()
        }
        ActionResult::Transmission(t) => {
            let outcome = match t.outcome {
                Outcome::Recorded => "the directly attached receiver recorded the advertisement",
                Outcome::Withdrawn => "the directly attached receiver withdrew the neighbor record",
                Outcome::TransmitDisabled => "transmission is disabled",
                Outcome::ReceiveDisabled => "the directly attached receiver is disabled",
                Outcome::LinkDown => "the physical port or link is down",
                Outcome::Unattached => "this port has no attached peer",
            };
            format!(
                "Simulated LLDP from {}/{}: {outcome}",
                t.sender.device, t.sender.port
            )
        }
        ActionResult::TimeAdvanced { now } => {
            format!("Simulated time is {now} seconds; no advertisements were scheduled")
        }
    })
}

pub fn valid_requirements(requirements: &[Requirement], initial: &Topology) -> bool {
    initial.validate().is_ok()
        && !requirements.is_empty()
        && requirements.len() <= MAX_REQUIREMENTS
        && requirements
            .iter()
            .enumerate()
            .all(|(i, r)| r.validate(initial).is_ok() && !requirements[..i].contains(r))
}
/// Current outcomes with concise native feedback. No prior event or check grants success.
pub fn checks(
    initial: &Topology,
    state: &State,
    requirements: &[Requirement],
) -> Vec<(bool, String)> {
    if !valid_requirements(requirements, initial) || state.validate().is_err() {
        return vec![(
            false,
            "The LLDP state or requested outcomes are invalid or exceed supported bounds.".into(),
        )];
    }
    requirements.iter().map(|requirement| {
        let context = match requirement {
            Requirement::CurrentNeighbor { local, .. } | Requirement::AbsentNeighbor { local } | Requirement::PortConfiguration { local, .. } => format!("{}/{}", local.device, local.port),
            Requirement::GlobalConfiguration { device, .. } => device.clone(),
            Requirement::Configuration { .. } => "Topology".into(),
        };
        let result = requirement.check(initial, state);
        let message = match &result {
            Ok(Check::Satisfied) => match requirement {
                Requirement::CurrentNeighbor { .. } => "current direct neighbor information matches",
                Requirement::AbsentNeighbor { .. } => "the neighbor cache is empty",
                _ => "the required configuration is preserved",
            },
            Ok(Check::IdentityChanged) => "fixed device/port identities or attachments changed; restore the authored graph",
            Ok(Check::ConfigurationMismatch) => "configuration differs; check the specified settings and preserve unrelated configuration",
            Ok(Check::MissingCurrentNeighbor) => "no unexpired neighbor record is present; missing and aged-out records need an explicit advertisement",
            Ok(Check::WrongPeer) => "the recorded peer differs from the requested direct attachment",
            Ok(Check::HistoricalNeighbor) => "cached information is historical; current delivery or advertised fields differ, so inspect settings before refreshing",
            Ok(Check::UnexpectedNeighbor) => "a neighbor record remains cached, including historical information; withdrawal or TTL aging is still needed",
            Err(_) => "the current outcome could not be evaluated against the fixed authored graph",
        };
        (result.is_ok_and(Check::passed), format!("{context}: {message}."))
    }).collect()
}
