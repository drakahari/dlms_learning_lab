//! Outcome-based power-plan requirements over declared, bounded equipment.
use crate::poe::{Facts, Observation, Plan, State};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Powered,
    DisconnectedOrDisabled,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    pub facts: Facts,
    pub initial: Plan,
    pub reference: Plan,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Target,8>")]
    pub targets: Vec<Target>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
pub fn watts(value: u32) -> String {
    let whole = value / 1000;
    let fraction = value % 1000;
    if fraction == 0 {
        format!("{whole} W")
    } else {
        format!(
            "{whole}.{} W",
            format!("{fraction:03}").trim_end_matches('0')
        )
    }
}
pub fn identity(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub fn describe(f: &Facts, row: &Observation) -> String {
    let device = &f.devices[row.device];
    match row.state {
        State::Disconnected => "Disconnected: no switch port supplies this device.".into(),
        State::Disabled => {
            "Power disabled on the selected port; no source power is reserved.".into()
        }
        State::DataOnly => {
            "This is a data-only port. Enabling a power control cannot give it PoE hardware.".into()
        }
        State::Incompatible => format!(
            "This device needs a {} source reservation, but {} supports only {}. Choose a compatible supplied port; extra shared budget cannot fix a per-port limit.",
            watts(device.class.reservation_mw()),
            f.ports[row.port.expect("observed attachment")].id,
            watts(f.ports[row.port.unwrap()].source.capacity_mw())
        ),
        State::BudgetDenied => format!(
            "Needs {}, but only {} of the shared source budget remains at this port. Check earlier reservations and which devices must stay powered; a capable port alone is not enough.",
            watts(device.class.reservation_mw()),
            watts(row.remaining_before_mw)
        ),
        State::Powered => format!(
            "Powered: {} of source capacity is reserved. This is not measured consumption or a test of IP connectivity.",
            watts(row.reservation_mw)
        ),
    }
}
impl Task {
    pub fn outcomes(&self, plan: &Plan) -> Vec<Check> {
        let report = match self.facts.observe(plan) {
            Ok(r) => r,
            Err(e) => {
                return vec![Check {
                    passed: false,
                    label: "Power plan".into(),
                    detail: e,
                }];
            }
        };
        if self.targets.len() != self.facts.devices.len() {
            return vec![Check {
                passed: false,
                label: "Task requirements".into(),
                detail: "Use the supplied equipment and requirements.".into(),
            }];
        }
        report.devices.iter().zip(&self.targets).map(|(row,target)| {
            let passed=match target {Target::Powered=>row.state==State::Powered,Target::DisconnectedOrDisabled=>matches!(row.state,State::Disconnected|State::Disabled)};
            let want=match target {Target::Powered=>"must have a compatible powered connection",Target::DisconnectedOrDisabled=>"must be deliberately disconnected or have power disabled; a fault or budget denial is not retirement"};
            Check{passed,label:format!("{} power",self.facts.devices[row.device].id),detail:format!("{} {}. {}",self.facts.devices[row.device].id,want,describe(&self.facts,row))}
        }).collect()
    }
    pub fn solved(&self, plan: &Plan) -> bool {
        let checks = self.outcomes(plan);
        !checks.is_empty() && checks.iter().all(|c| c.passed)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.facts.validate_plan(&self.initial)?;
        self.facts.validate_plan(&self.reference)?;
        if !identity(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.instructions.chars().any(char::is_control)
            || self.targets.len() != self.facts.devices.len()
            || self.solved(&self.initial)
            || !self.solved(&self.reference)
        {
            return Err("Supply a named power task, an incomplete starter, every device outcome and a reachable reference plan.".into());
        }
        Ok(())
    }
}
