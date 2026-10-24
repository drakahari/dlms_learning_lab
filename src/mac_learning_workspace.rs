//! Bounded native MAC drafts and pack-only typed actions, never a switch CLI.
use crate::{mac_learning as mac, switching, switching_workspace};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const HELP: &str = "Use the native port settings, Apply settings, Send frame, Advance time and Clear dynamic entries controls. Time advances only when requested. Frames contain Ethernet metadata only: no ping, ARP, real packets or IOS terminal. Check work recomputes outcomes from the current state.";
pub fn same_problem(current: &mac::State, initial: &mac::State) -> bool {
    current.validate().is_ok()
        && initial.validate().is_ok()
        && current.fabric().same_shape(initial.fabric())
        && current
            .fabric()
            .hosts
            .keys()
            .all(|h| current.address(h) == initial.address(h))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Outcome {
        outcome: mac::Requirement,
    },
    Port {
        switch: String,
        port: String,
        settings: switching::Port,
    },
    ActiveVlans {
        switch: String,
        vlans: BTreeSet<u16>,
    },
}
impl Requirement {
    pub fn valid_for(&self, initial: &mac::State) -> bool {
        match self {
            Self::Outcome { outcome } => outcome.valid_for(initial),
            Self::Port {
                switch,
                port,
                settings,
            } => {
                initial
                    .fabric()
                    .switches
                    .get(switch)
                    .is_some_and(|s| s.ports.contains_key(port))
                    && match &settings.mode {
                        switching::Mode::Access { vlan } => (1..=4094).contains(vlan),
                        switching::Mode::Trunk { native, allowed } => {
                            (1..=4094).contains(native)
                                && allowed.len() <= 64
                                && allowed.iter().all(|v| (1..=4094).contains(v))
                        }
                    }
            }
            Self::ActiveVlans { switch, vlans } => {
                initial.fabric().switches.contains_key(switch)
                    && vlans.len() <= 64
                    && vlans.iter().all(|v| (1..=4094).contains(v))
            }
        }
    }
    pub fn check(&self, state: &mac::State) -> (bool, String) {
        if !self.valid_for(state) {
            return (
                false,
                "This MAC outcome has invalid or unsupported network data".into(),
            );
        }
        match self {
            Self::Outcome { outcome } => outcome.check(state),
            Self::Port {
                switch,
                port,
                settings,
            } => {
                let ok = state
                    .fabric()
                    .switches
                    .get(switch)
                    .and_then(|s| s.ports.get(port))
                    == Some(settings);
                (
                    ok,
                    format!(
                        "{switch}/{port}: {}",
                        if ok {
                            "port settings match the task"
                        } else {
                            "inspect the port's state, mode and VLAN settings"
                        }
                    ),
                )
            }
            Self::ActiveVlans { switch, vlans } => {
                let ok = state
                    .fabric()
                    .switches
                    .get(switch)
                    .is_some_and(|s| s.vlans == *vlans);
                (
                    ok,
                    format!(
                        "{switch}: {}",
                        if ok {
                            "active VLANs match the task"
                        } else {
                            "the active VLAN list differs from the task"
                        }
                    ),
                )
            }
        }
    }
}
/// Reader bound applies before an excess requirement body is decoded.
pub fn requirements<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    struct Excess;
    impl<'de> Deserialize<'de> for Excess {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Err(serde::de::Error::custom("At most 64 MAC outcomes"))
        }
    }
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = Vec<Requirement>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most64MAC outcomes")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            loop {
                if v.len() == 64 {
                    let _ = seq.next_element::<Excess>()?;
                    return Ok(v);
                }
                match seq.next_element()? {
                    Some(x) => v.push(x),
                    None => return Ok(v),
                }
            }
        }
    }
    d.deserialize_seq(V)
}
pub fn valid_requirements(reqs: &[Requirement], initial: &mac::State) -> bool {
    initial.validate().is_ok() && !reqs.is_empty() && reqs.len()<=64 && reqs.iter().all(|r|r.valid_for(initial))
        && reqs.iter().any(|r|matches!(r,Requirement::Outcome{outcome:mac::Requirement::Frame{accepted_by,..}} if !accepted_by.is_empty()))
}
pub fn checks(
    initial: &mac::State,
    current: &mac::State,
    reqs: &[Requirement],
) -> Vec<(bool, String)> {
    if !same_problem(current, initial) || !valid_requirements(reqs, initial) {
        return vec![(
            false,
            "The supplied MAC network or objectives differ from this activity".into(),
        )];
    }
    reqs.iter().map(|r| r.check(current)).collect()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u8,
    pub configuration: switching_workspace::Draft,
    pub aging: String,
    pub source: String,
    pub destination: String,
    pub advance_seconds: String,
    pub clear_switch: String,
    /// Empty means every VLAN on the selected switch.
    pub clear_vlan: String,
}
impl Draft {
    pub fn new(s: &mac::State) -> Result<Self, String> {
        s.validate()?;
        let source = s
            .fabric()
            .hosts
            .keys()
            .next()
            .cloned()
            .ok_or("MAC practice needs at least one workstation")?;
        Ok(Self {
            version: 1,
            configuration: switching_workspace::Draft::new(s.fabric()),
            aging: s.aging_seconds().to_string(),
            source,
            destination: mac::Mac::BROADCAST.text(),
            advance_seconds: "30".into(),
            clear_switch: s
                .fabric()
                .switches
                .keys()
                .next()
                .cloned()
                .unwrap_or_default(),
            clear_vlan: String::new(),
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.configuration.valid()
            && self.aging.len() <= 4
            && self.source.len() <= 64
            && self.destination.len() <= 17
            && self.advance_seconds.len() <= 20
            && self.clear_switch.len() <= 64
            && self.clear_vlan.len() <= 4
    }
    pub fn same_shape(&self, s: &mac::State) -> bool {
        self.valid()
            && self.configuration.same_shape(s.fabric())
            && s.fabric().hosts.contains_key(&self.source)
            && s.fabric().switches.contains_key(&self.clear_switch)
    }
    pub fn apply(&self, s: &mut mac::State) -> Result<(), String> {
        s.validate()?;
        if !self.same_shape(s) {
            return Err("Keep the supplied switch and workstation names".into());
        }
        let aging = decimal(&self.aging, 3600)? as u16;
        let mut f = s.fabric().clone();
        self.configuration.apply(&mut f)?;
        s.configure(f, aging)
    }
    pub fn matches(&self, s: &mac::State) -> bool {
        self.same_shape(s)
            && decimal(&self.aging, 3600).is_ok_and(|n| n == u64::from(s.aging_seconds()))
            && self.configuration.matches(s.fabric())
    }
}
pub fn decimal(s: &str, max: u64) -> Result<u64, String> {
    if s.is_empty() || s.len() > 20 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("Enter a whole number from 0 to {max}"));
    }
    s.parse::<u64>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| format!("Enter a whole number from 0 to {max}"))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    /// Internal authoring syntax only. UI uses the native port table.
    Settings {
        operation: String,
    },
    Aging {
        seconds: u16,
    },
    Send {
        source: String,
        destination: mac::Mac,
    },
    Advance {
        seconds: u64,
    },
    Clear {
        switch: String,
        vlan: Option<u16>,
    },
}
impl Action {
    pub fn valid(&self) -> bool {
        match self {
            Self::Settings { operation } => !operation.is_empty() && operation.len() <= 4096,
            Self::Aging { seconds } => *seconds <= 3600,
            Self::Send { source, .. } => !source.is_empty() && source.len() <= 64,
            Self::Advance { seconds } => *seconds <= 86400,
            Self::Clear { switch, vlan } => {
                !switch.is_empty()
                    && switch.len() <= 64
                    && vlan.is_none_or(|v| (1..=4094).contains(&v))
            }
        }
    }
    pub fn label(&self) -> String {
        if !self.valid() {
            return "Invalid MAC example action".into();
        }
        match self {
            Self::Settings { .. } => "Apply the example port settings".into(),
            Self::Aging { seconds } => format!("Apply an aging time of {seconds} seconds"),
            Self::Send {
                source,
                destination,
            } => format!("Send from {source} to {}", destination.text()),
            Self::Advance { seconds } => format!("Advance virtual time by {seconds} seconds"),
            Self::Clear { switch, vlan } => format!(
                "Clear dynamic entries on {switch} ({})",
                vlan.map_or("all VLANs".into(), |v| format!("VLAN{v}"))
            ),
        }
    }
    pub fn apply(&self, s: &mut mac::State) -> Result<(), String> {
        s.validate()?;
        if !self.valid() {
            return Err("Invalid or excessive internal MAC action".into());
        }
        match self {
            Self::Settings { operation } => {
                let mut f = s.fabric().clone();
                switching_workspace::adapter(&mut f, operation)?;
                s.configure(f, s.aging_seconds())
            }
            Self::Aging { seconds } => s.configure(s.fabric().clone(), *seconds),
            Self::Send {
                source,
                destination,
            } => s.send(source, *destination).map(|_| ()),
            Self::Advance { seconds } => s.advance(*seconds),
            Self::Clear { switch, vlan } => s.clear_dynamic(switch, *vlan).map(|_| ()),
        }
    }
    /// Populate native inputs only; never send, clear, advance or publish state.
    pub fn insert(&self, s: &mac::State, draft: &mut Draft) -> Result<(), String> {
        if !self.valid() || !draft.same_shape(s) {
            return Err("Invalid MAC example inputs".into());
        }
        match self {
            Self::Settings { operation } => {
                let mut f = s.fabric().clone();
                draft.configuration.apply(&mut f)?;
                switching_workspace::adapter(&mut f, operation)?;
                let selected = draft.configuration.selected_switch.clone();
                draft.configuration = switching_workspace::Draft::new(&f);
                draft.configuration.selected_switch = selected;
            }
            Self::Aging { seconds } => draft.aging = seconds.to_string(),
            Self::Send {
                source,
                destination,
            } => {
                if !s.fabric().hosts.contains_key(source) {
                    return Err("Unknown example workstation".into());
                }
                draft.source = source.clone();
                draft.destination = destination.text()
            }
            Self::Advance { seconds } => draft.advance_seconds = seconds.to_string(),
            Self::Clear { switch, vlan } => {
                if !s.fabric().switches.contains_key(switch) {
                    return Err("Unknown example switch".into());
                }
                draft.clear_switch = switch.clone();
                draft.clear_vlan = vlan.map(|n| n.to_string()).unwrap_or_default()
            }
        }
        Ok(())
    }
}
/// Declarative-pack/reachability path only. Not a learner command language.
pub fn adapter(s: &mut mac::State, input: &str) -> Result<String, String> {
    if input.len() > 4096 {
        return Err("MAC authoring action exceeds 4096 bytes".into());
    }
    let json = input
        .strip_prefix("@mac ")
        .ok_or("Use native MAC controls; no IOS command interpreter is included")?;
    let action: Action =
        serde_json::from_str(json).map_err(|_| "Malformed internal MAC action".to_string())?;
    action.apply(s)?;
    Ok("Applied inside the MAC simulator only".into())
}
