//! Source-held work orders for structural site connectivity.
use super::site_connectivity::{Outage, Topology};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub label: String,
    pub from: usize,
    pub to: usize,
    pub outage: Outage,
    pub reachable: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,220>")]
    pub retry: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub topology: Topology,
    pub initial: u32,
    pub fixed: u32,
    pub choices: u32,
    pub max_choices: u8,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Trial,12>")]
    pub trials: Vec<Trial>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    pub active: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

impl Task {
    pub fn initial_edit(&self) -> Edit {
        Edit {
            active: self.initial,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.topology.validate()?;
        if self.fixed & self.choices != 0
            || self.choices == 0
            || (self.fixed | self.choices) >> self.topology.links.len() != 0
            || self.max_choices == 0
            || u32::from(self.max_choices) > self.choices.count_ones()
            || !(3..=12).contains(&self.trials.len())
            || !self.trials.iter().any(|trial| !trial.reachable)
            || !self
                .trials
                .iter()
                .any(|trial| trial.reachable && trial.outage == Outage::None)
            || !self
                .trials
                .iter()
                .any(|trial| trial.reachable && trial.outage != Outage::None)
        {
            return Err("Supply distinct circuit options, a budget, and normal, outage and separation trials.".into());
        }
        self.initial_edit().validate(self)?;
        let mut labels = BTreeSet::new();
        for trial in &self.trials {
            if trial.label.trim().is_empty()
                || trial.retry.trim().is_empty()
                || trial.label.chars().any(char::is_control)
                || trial.retry.chars().any(char::is_control)
                || !labels.insert(&trial.label)
            {
                return Err("Use distinct plain-language trial names and recovery hints.".into());
            }
            self.topology
                .path(self.initial, trial.from, trial.to, trial.outage)?;
        }
        Ok(())
    }

    pub fn observe(
        &self,
        edit: &Edit,
        index: usize,
    ) -> Result<Option<super::site_connectivity::Path>, String> {
        edit.validate(self)?;
        let trial = self
            .trials
            .get(index)
            .ok_or("Choose a supplied work-order trial.")?;
        self.topology
            .path(edit.active, trial.from, trial.to, trial.outage)
    }

    pub fn outcomes(&self, edit: &Edit) -> Result<Vec<Outcome>, String> {
        edit.validate(self)?;
        let chosen = (edit.active & self.choices).count_ones();
        let mut outcomes = vec![Outcome {
            name: "Circuit budget".into(),
            passed: chosen <= u32::from(self.max_choices),
            detail: format!(
                "Selected {chosen} optional links. The work order allows {}.",
                self.max_choices
            ),
        }];
        for (index, trial) in self.trials.iter().enumerate() {
            let reached = self.observe(edit, index)?.is_some();
            let passed = reached == trial.reachable;
            outcomes.push(Outcome {
                name: trial.label.clone(),
                passed,
                detail: if passed {
                    if reached {
                        "A path remains available with this condition."
                    } else {
                        "No available path connects these endpoints."
                    }
                    .into()
                } else {
                    trial.retry.clone()
                },
            });
        }
        Ok(outcomes)
    }
}

impl Edit {
    pub fn validate(&self, task: &Task) -> Result<(), String> {
        if self.active & task.fixed != task.fixed || self.active & !(task.fixed | task.choices) != 0
        {
            return Err(
                "Keep fixed links attached. Only listed circuit options can change.".into(),
            );
        }
        Ok(())
    }

    pub fn set_link(&mut self, task: &Task, index: usize, active: bool) -> Result<(), String> {
        self.validate(task)?;
        if index >= task.topology.links.len() || task.choices & (1 << index) == 0 {
            return Err("Choose one of the optional links in this work order.".into());
        }
        if active {
            self.active |= 1 << index;
        } else {
            self.active &= !(1 << index);
        }
        Ok(())
    }
}
