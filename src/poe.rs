//! Bounded class-based PSE reservation; no electrical, host or network activity.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    DataOnly,
    Ieee8023af,
    Ieee8023at,
}
impl Source {
    pub fn capacity_mw(self) -> u32 {
        match self {
            Self::DataOnly => 0,
            Self::Ieee8023af => 15_400,
            Self::Ieee8023at => 30_000,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Unknown0,
    One,
    Two,
    Three,
    Four,
}
impl Class {
    pub fn reservation_mw(self) -> u32 {
        match self {
            Self::Unknown0 | Self::Three => 15_400,
            Self::One => 4_000,
            Self::Two => 7_000,
            Self::Four => 30_000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    pub source: Source,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    pub class: Class,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub budget_mw: u32,
    /// Listed order is the disclosed deterministic reservation order, not IOS scheduling.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Port,8>")]
    pub ports: Vec<Port>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Device,8>")]
    pub devices: Vec<Device>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// One optional supplied port index per powered device; never two plugs in one port.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Option<usize>,8>")]
    pub attachments: Vec<Option<usize>>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,bool,8>")]
    pub enabled: Vec<bool>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Disconnected,
    Disabled,
    DataOnly,
    Incompatible,
    BudgetDenied,
    Powered,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub device: usize,
    pub port: Option<usize>,
    pub state: State,
    pub reservation_mw: u32,
    pub remaining_before_mw: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub devices: Vec<Observation>,
    pub reserved_mw: u32,
    pub remaining_mw: u32,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_/.".contains(&b))
}
impl Facts {
    pub fn validate(&self) -> Result<(), String> {
        if self.budget_mw == 0
            || self.budget_mw > 240_000
            || !(1..=8).contains(&self.ports.len())
            || !(1..=8).contains(&self.devices.len())
        {
            return Err(
                "Supply up to eight ports/devices and a bounded source-power budget.".into(),
            );
        }
        for ids in [
            self.ports.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
            self.devices.iter().map(|d| d.id.as_str()).collect(),
        ] {
            if ids.iter().any(|s| !identifier(s))
                || ids.iter().collect::<BTreeSet<_>>().len() != ids.len()
            {
                return Err(
                    "Keep equipment identifiers bounded and distinct within their collection."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn validate_plan(&self, plan: &Plan) -> Result<(), String> {
        self.validate()?;
        let used: Vec<_> = plan.attachments.iter().flatten().copied().collect();
        if plan.attachments.len() != self.devices.len()
            || plan.enabled.len() != self.ports.len()
            || used.iter().any(|p| *p >= self.ports.len())
            || used.iter().collect::<BTreeSet<_>>().len() != used.len()
        {
            return Err("Connect each device to one distinct supplied port and keep the supplied port controls.".into());
        }
        Ok(())
    }
    pub fn observe(&self, plan: &Plan) -> Result<Report, String> {
        self.validate_plan(plan)?;
        let mut remaining = self.budget_mw;
        let mut devices: Vec<_> = plan
            .attachments
            .iter()
            .enumerate()
            .map(|(device, port)| Observation {
                device,
                port: *port,
                state: State::Disconnected,
                reservation_mw: 0,
                remaining_before_mw: self.budget_mw,
            })
            .collect();
        for (port, source) in self.ports.iter().enumerate() {
            let Some(device) = plan.attachments.iter().position(|p| *p == Some(port)) else {
                continue;
            };
            let need = self.devices[device].class.reservation_mw();
            let state = if source.source == Source::DataOnly {
                State::DataOnly
            } else if !plan.enabled[port] {
                State::Disabled
            } else if need > source.source.capacity_mw() {
                State::Incompatible
            } else if need > remaining {
                State::BudgetDenied
            } else {
                State::Powered
            };
            let reservation = if state == State::Powered { need } else { 0 };
            devices[device] = Observation {
                device,
                port: Some(port),
                state,
                reservation_mw: reservation,
                remaining_before_mw: remaining,
            };
            remaining -= reservation;
        }
        Ok(Report {
            devices,
            reserved_mw: self.budget_mw - remaining,
            remaining_mw: remaining,
        })
    }
}
