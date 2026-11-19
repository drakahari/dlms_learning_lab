//! Compare supplied claims and forecasts with independent in-process observations.
use super::intent_path::{Current, Exchange, Spec, Trace};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    CarrierDown,
    InstalledRuleStale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assessment {
    Supported,
    Contradicted,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forecast {
    pub threshold_percent: u8,
    pub predicted_percent: u8,
    pub prior_samples: [u8; 4],
    pub next_sample: u8,
}

impl Forecast {
    pub fn validate(self) -> Result<(), String> {
        if !(1..=100).contains(&self.threshold_percent)
            || self.predicted_percent > 100
            || self.next_sample > 100
            || self.prior_samples.iter().any(|value| *value > 100)
        {
            return Err("Use calibrated percentages and an alert threshold from 1 to 100.".into());
        }
        Ok(())
    }

    pub fn assess(self, collected: bool) -> Assessment {
        if !collected {
            return Assessment::Pending;
        }
        if (self.predicted_percent >= self.threshold_percent)
            == (self.next_sample >= self.threshold_percent)
        {
            Assessment::Supported
        } else {
            Assessment::Contradicted
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlCase {
    pub exchange: Exchange,
    pub should_forward: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub topology: Spec,
    pub starter: Current,
    pub incident: Exchange,
    pub controls: [ControlCase; 2],
    pub generated_claim: Cause,
    pub forecast: Forecast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub inspected_incident: bool,
    pub collected_next_sample: bool,
    pub diagnosis: Option<Assessment>,
    pub prediction: Option<Assessment>,
    pub active: Current,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub passed: bool,
    pub title: &'static str,
    pub detail: String,
}

impl Finding {
    fn report(passed: bool, title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            passed,
            title,
            detail: detail.into(),
        }
    }
}

impl Cause {
    pub const fn describe(self) -> &'static str {
        match self {
            Self::CarrierDown => "The supplied carrier path has failed.",
            Self::InstalledRuleStale => {
                "The controller wants this exchange, but the device still has an older rule."
            }
        }
    }

    pub fn assess(self, observed: &Trace) -> Assessment {
        let supported = match self {
            Self::CarrierDown => observed.carrier.is_err(),
            Self::InstalledRuleStale => {
                observed.desired && !observed.installed && observed.carrier.is_ok()
            }
        };
        if supported {
            Assessment::Supported
        } else {
            Assessment::Contradicted
        }
    }
}

impl Task {
    pub fn new_review(&self) -> Review {
        Review {
            inspected_incident: false,
            collected_next_sample: false,
            diagnosis: None,
            prediction: None,
            active: self.starter,
        }
    }

    pub fn baseline(&self) -> Result<Trace, String> {
        self.topology.trace(self.starter, self.incident)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.topology.validate()?;
        self.forecast.validate()?;
        let trace = self.baseline()?;
        if !trace.desired
            || trace.installed
            || trace.carrier.is_err()
            || self.controls[0].exchange == self.controls[1].exchange
            || self
                .controls
                .iter()
                .any(|case| case.exchange == self.incident)
            || self.controls[0].should_forward == self.controls[1].should_forward
        {
            return Err("Supply a real stale-rule incident and distinct passing and excluded control cases.".into());
        }
        let mut repaired = self.starter;
        repaired.publish();
        for case in self.controls {
            for world in [self.starter, repaired] {
                if self.topology.trace(world, case.exchange)?.forwarded() != case.should_forward {
                    return Err("The repair must preserve the supplied control outcomes.".into());
                }
            }
        }
        if !self.topology.trace(repaired, self.incident)?.forwarded() {
            return Err("The supplied approved rule must repair the observed path.".into());
        }
        Ok(())
    }

    pub fn findings(&self, review: Review) -> Result<Vec<Finding>, String> {
        let actual = self.topology.trace(review.active, self.incident)?;
        let sample_check = self.forecast.assess(review.collected_next_sample);
        let hypothesis = self.generated_claim.assess(&self.baseline()?);
        let mut checks = vec![
            Finding::report(
                review.inspected_incident,
                "Run the independent incident probe",
                if review.inspected_incident {
                    "The initial carrier and installed-rule results are recorded."
                } else {
                    "Run the supplied incident probe before publishing network changes."
                },
            ),
            Finding::report(
                review.inspected_incident && review.diagnosis == Some(hypothesis),
                "Review the generated diagnosis",
                if review.inspected_incident {
                    "Use the actual initial carrier result and compare the desired policy with the installed rule."
                } else {
                    "Inspect the independent evidence before judging the description."
                },
            ),
            Finding::report(
                review.collected_next_sample && review.prediction == Some(sample_check),
                "Review the forecast against the later observation",
                if review.collected_next_sample {
                    "Compare both the forecast and observed reading with the same alert threshold."
                } else {
                    "Collect the next supplied reading before judging the forecast."
                },
            ),
            Finding::report(
                actual.forwarded(),
                "Repair the incident",
                if actual.forwarded() {
                    "The approved exchange now forwards over the existing carrier."
                } else {
                    "Publish the verified desired rule, then recheck the actual path."
                },
            ),
        ];
        for (index, case) in self.controls.iter().enumerate() {
            let observed = self.topology.trace(review.active, case.exchange)?;
            checks.push(Finding::report(
                observed.forwarded() == case.should_forward,
                if index == 0 {
                    "Preserve control exchange 1"
                } else {
                    "Preserve control exchange 2"
                },
                format!(
                    "The observed exchange {}. This control is {} by the work order.",
                    if observed.forwarded() {
                        "forwarded"
                    } else {
                        "did not forward"
                    },
                    if case.should_forward {
                        "allowed"
                    } else {
                        "excluded"
                    }
                ),
            ));
        }
        Ok(checks)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    NoChange,
    RestoreCarrier,
    PublishRules,
}

impl Response {
    pub const fn describe(self) -> &'static str {
        match self {
            Self::NoChange => "Keep current settings",
            Self::RestoreCarrier => "Enable supplied carrier",
            Self::PublishRules => "Publish approved rules",
        }
    }

    pub fn apply(self, active: &mut Current) {
        match self {
            Self::NoChange => {}
            Self::RestoreCarrier => active.uplink_up = true,
            Self::PublishRules => active.publish(),
        }
    }
}

impl Review {
    pub fn validate(self, task: &Task) -> Result<(), String> {
        if self.active.controller != task.starter.controller
            || (self.active.installed != task.starter.installed
                && self.active.installed != task.starter.controller)
            || self.active.uplink_up != task.starter.uplink_up
        {
            return Err(
                "The saved review must use the supplied carrier and a supported publish operation."
                    .into(),
            );
        }
        Ok(())
    }
}
