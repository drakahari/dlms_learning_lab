//! Supplied-survey channel planning. No RF propagation, throughput or host radios.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const SITE_CHANNELS: [u8; 3] = [1, 6, 11];
pub const MAX_RADIOS: usize = 8;
pub const MAX_SURVEYS: usize = 3;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radio {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub managed: bool,
    pub initial_channel: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub a: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub b: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Survey {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    /// Explicit supplied mutual-hearing relationships, not generated distances.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Pair,28>")]
    pub neighbours: Vec<Pair>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Radio,8>")]
    pub radios: Vec<Radio>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Survey,3>")]
    pub surveys: Vec<Survey>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Relationship {
    SharedChannel,
    AdjacentOverlap,
    Separated,
}
/// A conservative conventional five-channel spacing rule for this supplied
/// 2.4GHz site plan, not a measured spectrum/power/airtime calculation.
pub fn relationship(a: u8, b: u8) -> Result<Relationship, String> {
    if !(1..=11).contains(&a) || !(1..=11).contains(&b) {
        return Err("This activity supplies channels 1–11 in the 2.4 GHz band.".into());
    }
    Ok(if a == b {
        Relationship::SharedChannel
    } else if a.abs_diff(b) < 5 {
        Relationship::AdjacentOverlap
    } else {
        Relationship::Separated
    })
}
fn id(s: &str) -> bool {
    s.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 120 && !s.chars().any(char::is_control)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub id: String,
    pub label: String,
    pub correct: bool,
    pub feedback: String,
}
impl Network {
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=MAX_RADIOS).contains(&self.radios.len())
            || !(1..=MAX_SURVEYS).contains(&self.surveys.len())
        {
            return Err("Supply 2–8 radios and 1–3 bounded surveys.".into());
        }
        let mut radios = BTreeSet::new();
        if !self.radios.iter().any(|r| r.managed) {
            return Err("At least one radio must be managed in this activity.".into());
        }
        for r in &self.radios {
            if !id(&r.id)
                || !label(&r.label)
                || !radios.insert(r.id.as_str())
                || !(1..=11).contains(&r.initial_channel)
            {
                return Err("Invalid or duplicate supplied radio.".into());
            }
        }
        let mut surveys = BTreeSet::new();
        let mut involved = BTreeSet::new();
        for s in &self.surveys {
            if !id(&s.id)
                || !label(&s.label)
                || !surveys.insert(s.id.as_str())
                || s.neighbours.is_empty()
                || s.neighbours.len() > MAX_RADIOS * (MAX_RADIOS - 1) / 2
            {
                return Err("Invalid, duplicate or unbounded supplied survey.".into());
            }
            let mut pairs = BTreeSet::new();
            for p in &s.neighbours {
                if p.a == p.b || !radios.contains(p.a.as_str()) || !radios.contains(p.b.as_str()) {
                    return Err("Survey pairs must name two distinct supplied radios.".into());
                }
                if !self
                    .radios
                    .iter()
                    .any(|r| (r.id == p.a || r.id == p.b) && r.managed)
                {
                    return Err(
                        "A graded survey pair must involve at least one managed radio.".into(),
                    );
                }
                let key = if p.a < p.b {
                    (&p.a, &p.b)
                } else {
                    (&p.b, &p.a)
                };
                if !pairs.insert(key) {
                    return Err("A survey repeats a mutual-hearing relationship.".into());
                }
                involved.insert(p.a.as_str());
                involved.insert(p.b.as_str());
            }
        }
        if self
            .radios
            .iter()
            .filter(|r| r.managed)
            .any(|r| !involved.contains(r.id.as_str()))
        {
            return Err(
                "Every managed radio needs a meaningful supplied survey relationship.".into(),
            );
        }
        Ok(())
    }
    pub fn initial_plan(&self) -> BTreeMap<String, u8> {
        self.radios
            .iter()
            .map(|r| (r.id.clone(), r.initial_channel))
            .collect()
    }
    pub fn validate_plan(&self, plan: &BTreeMap<String, u8>) -> Result<(), String> {
        self.validate()?;
        if plan.len() != self.radios.len()
            || self.radios.iter().any(|r| !plan.contains_key(&r.id))
            || plan.values().any(|c| !(1..=11).contains(c))
        {
            return Err(
                "A plan must contain each supplied radio exactly once, on a supported channel."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn checks(&self, plan: &BTreeMap<String, u8>) -> Result<Vec<Check>, String> {
        self.validate_plan(plan)?;
        let mut out = Vec::new();
        for r in &self.radios {
            let channel = plan[&r.id];
            let correct = if r.managed {
                SITE_CHANNELS.contains(&channel)
            } else {
                channel == r.initial_channel
            };
            out.push(Check {
                id: format!("radio-{}", r.id),
                label: r.label.clone(),
                correct,
                feedback: if r.managed {
                    if correct { format!("Channel {channel} follows this site's 1 / 6 / 11 plan; survey relationships are checked separately.") }
                    else { format!("Channel {channel} is a possible 2.4 GHz channel, but this task requires the site's 1 / 6 / 11 plan. A different number alone does not ensure separation.") }
                } else if correct { format!("Protected neighbour remains on channel {channel}.") }
                else { format!("Keep this supplied neighbour on channel {}; it is outside your change authority.", r.initial_channel) },
            });
        }
        let radios: BTreeMap<_, _> = self.radios.iter().map(|r| (r.id.as_str(), r)).collect();
        for s in &self.surveys {
            for (n, p) in s.neighbours.iter().enumerate() {
                let a = plan[&p.a];
                let b = plan[&p.b];
                let rel = relationship(a, b)?;
                out.push(Check {
                    id: format!("survey-{}-{n}", s.id),
                    label: format!("{} · {} / {}", s.label, radios[p.a.as_str()].label, radios[p.b.as_str()].label),
                    correct: rel == Relationship::Separated,
                    feedback: match rel {
                        Relationship::SharedChannel => format!("Both radios use channel {a} and can hear one another in this survey. They share airtime; this does not mean clients cannot associate. Try separated channels for this plan."),
                        Relationship::AdjacentOverlap => format!("Channels {a} and {b} are too close in this supplied plan. Different channel numbers can still overlap; choose separated site-plan channels."),
                        Relationship::Separated => format!("Channels {a} and {b} are separated under this site's supplied planning rule. This is not a throughput or real survey measurement."),
                    },
                });
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests;
