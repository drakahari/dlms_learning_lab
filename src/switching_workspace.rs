//! Native drafts and internal declarative-pack actions. Never an IOS command interpreter.
use crate::switching::{Fabric, Mode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
fn number(text: &str) -> Result<u16, String> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Enter a VLAN number from 1 to 4094".into());
    }
    text.parse::<u16>()
        .ok()
        .filter(|n| (1..=4094).contains(n))
        .ok_or_else(|| "Enter a VLAN number from 1 to 4094".into())
}
fn list(text: &str) -> Result<BTreeSet<u16>, String> {
    if text.trim().is_empty() {
        return Ok(BTreeSet::new());
    }
    let mut result = BTreeSet::new();
    for part in text.split(',') {
        if result.len() >= 64 || !result.insert(number(part.trim())?) {
            return Err("Use at most 64 different VLAN numbers, separated by commas".into());
        }
    }
    Ok(result)
}
fn csv(values: &BTreeSet<u16>) -> String {
    values
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortDraft {
    pub up: bool,
    pub trunk: bool,
    pub access: String,
    pub native: String,
    pub allowed: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwitchDraft {
    pub vlans: String,
    pub ports: BTreeMap<String, PortDraft>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub selected_switch: String,
    pub switches: BTreeMap<String, SwitchDraft>,
}
impl Draft {
    pub fn new(f: &Fabric) -> Self {
        Self {
            version: 1,
            selected_switch: f.switches.keys().next().cloned().unwrap_or_default(),
            switches: f
                .switches
                .iter()
                .map(|(id, s)| {
                    (
                        id.clone(),
                        SwitchDraft {
                            vlans: csv(&s.vlans),
                            ports: s
                                .ports
                                .iter()
                                .map(|(id, p)| {
                                    let (trunk, access, native, allowed) = match &p.mode {
                                        Mode::Access { vlan } => {
                                            (false, vlan.to_string(), "1".into(), "1".into())
                                        }
                                        Mode::Trunk { native, allowed } => {
                                            (true, "1".into(), native.to_string(), csv(allowed))
                                        }
                                    };
                                    (
                                        id.clone(),
                                        PortDraft {
                                            up: p.up,
                                            trunk,
                                            access,
                                            native,
                                            allowed,
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
    pub fn valid(&self) -> bool {
        let id = |s: &str| {
            !s.is_empty()
                && s.len() <= 64
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        self.version == 1
            && self.switches.contains_key(&self.selected_switch)
            && !self.switches.is_empty()
            && self.switches.len() <= 8
            && self.switches.iter().all(|(name, s)| {
                id(name)
                    && s.vlans.len() <= 384
                    && !s.ports.is_empty()
                    && s.ports.len() <= 16
                    && s.ports.iter().all(|(name, p)| {
                        id(name)
                            && p.access.len() <= 4
                            && p.native.len() <= 4
                            && p.allowed.len() <= 384
                    })
            })
    }
    pub fn same_shape(&self, f: &Fabric) -> bool {
        self.valid()
            && self.switches.keys().eq(f.switches.keys())
            && self
                .switches
                .iter()
                .all(|(id, s)| s.ports.keys().eq(f.switches[id].ports.keys()))
    }
    pub fn apply(&self, f: &mut Fabric) -> Result<(), String> {
        f.validate()?;
        if !self.same_shape(f) {
            return Err("Switch/port names must match this activity".into());
        }
        let mut candidate = f.clone();
        for (id, draft) in &self.switches {
            let s = candidate.switches.get_mut(id).unwrap();
            s.vlans = list(&draft.vlans).map_err(|e| format!("{id} active VLANs: {e}"))?;
            for (port, draft) in &draft.ports {
                let p = s.ports.get_mut(port).unwrap();
                p.up = draft.up;
                p.mode = if draft.trunk {
                    Mode::Trunk {
                        native: number(&draft.native)
                            .map_err(|e| format!("{id}/{port} native: {e}"))?,
                        allowed: list(&draft.allowed)
                            .map_err(|e| format!("{id}/{port} allowed: {e}"))?,
                    }
                } else {
                    Mode::Access {
                        vlan: number(&draft.access)
                            .map_err(|e| format!("{id}/{port} access: {e}"))?,
                    }
                };
            }
        }
        candidate.validate()?;
        *f = candidate;
        Ok(())
    }
    pub fn matches(&self, f: &Fabric) -> bool {
        let mut candidate = f.clone();
        self.apply(&mut candidate).is_ok() && candidate == *f
    }
}
/// Pack reachability/test actions only. Native controls are the learner interface.
pub fn adapter(f: &mut Fabric, input: &str) -> Result<String, String> {
    if input.len() > 4096 {
        return Err("Switching reference action exceeds 4096 bytes".into());
    }
    let mut draft = Draft::new(f);
    let args: Vec<_> = input.split_whitespace().collect();
    match args.as_slice() {
        ["vlan", switch, value, status] => {
            let d = draft.switches.get_mut(*switch).ok_or("Unknown switch")?;
            let mut values = list(&d.vlans)?;
            let n = number(value)?;
            match *status {
                "present" => {
                    values.insert(n);
                }
                "absent" => {
                    values.remove(&n);
                }
                _ => return Err("Use present or absent".into()),
            };
            d.vlans = csv(&values);
        }
        [kind, switch, port, rest @ ..] => {
            let p = draft
                .switches
                .get_mut(*switch)
                .ok_or("Unknown switch")?
                .ports
                .get_mut(*port)
                .ok_or("Unknown port")?;
            match (*kind,rest) {
                ("access",[value])=>{p.trunk=false;p.access=value.to_string();},
                ("trunk",[native,allowed])=>{p.trunk=true;p.native=native.to_string();p.allowed=if *allowed=="-"{String::new()}else{allowed.to_string()};},
                ("port",[status])=>p.up=match *status {"up"=>true,"down"=>false,_=>return Err("Use up or down".into())},
                _=>return Err("Unsupported internal switching action; use native port tables, not invented IOS commands".into()),
            }
        }
        _ => return Err("Unsupported internal switching action; use native port tables".into()),
    }
    draft.apply(f)?;
    Ok("Applied to the simulated switching fabric only".into())
}
