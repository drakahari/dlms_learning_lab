//! Fresh outcome trials over authored decoded records, never live traffic.
use crate::ids_rule::{self as core, Decision, Event, Fault, Rule};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,64>")]
    pub records: Vec<Event>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,bool,64>")]
    pub expected_alerts: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub rule_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub initial_rule: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,12>")]
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}

fn check(passed: bool, label: impl Into<String>, detail: impl Into<String>) -> Check {
    Check {
        passed,
        label: label.into(),
        detail: detail.into(),
    }
}
fn identity(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub fn canonical(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= 256
        && path.split('/').skip(1).all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
pub fn fault(error: Fault) -> String {
    match error {
        Fault::Syntax(message) => format!("Syntax error: {message}"),
        Fault::Unsupported(message) => format!("Unsupported rule or input: {message}"),
        Fault::Limit(message) => format!("Simulation limit: {message}"),
    }
}
pub fn decision_label(value: &Decision) -> String {
    match value {
        Decision::Alert => "Alert: all supported rule conditions match".into(),
        Decision::ProtocolMismatch => "No alert: protocol differs".into(),
        Decision::EndpointMismatch => "No alert: address, port or direction differs".into(),
        Decision::MissingContent(index) => {
            format!("No alert: content condition {} is missing", index + 1)
        }
    }
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        if !identity(&self.id)
            || !canonical(&self.rule_path)
            || !(2..=12).contains(&self.cases.len())
            || self.initial_rule.len() > 2048
            || !self.initial_rule.is_ascii()
            || self
                .initial_rule
                .bytes()
                .any(|b| b.is_ascii_control() && !b"\r\n\t".contains(&b))
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 64 * 1024
        {
            return Err("Use a stable rule task, canonical virtual rule path, bounded ASCII starter and 2–12 supplied cases within 64 KiB.".into());
        }
        let mut ids = BTreeSet::new();
        let mut positive = false;
        let mut negative = false;
        let validator =
            Rule::parse("alert tcp any any -> any any (sid:1000000;)").map_err(fault)?;
        for case in &self.cases {
            if !identity(&case.id)
                || !ids.insert(&case.id)
                || case.label.is_empty()
                || case.label.len() > 120
                || case.label.chars().any(char::is_control)
                || case.records.is_empty()
                || case.records.len() != case.expected_alerts.len()
            {
                return Err("Each record set needs a unique identifier, readable label and an expected alert decision for every record.".into());
            }
            core::inspect(&validator, &case.records).map_err(fault)?;
            positive |= case.expected_alerts.contains(&true);
            negative |= case.expected_alerts.contains(&false);
        }
        if !positive || !negative {
            return Err("Include both required detections and contrasting traffic that must not alert; silence or alerting on everything is not success.".into());
        }
        Ok(())
    }
    pub fn observe(&self, text: &str, case: usize) -> Result<Vec<Decision>> {
        self.validate()?;
        let case = self
            .cases
            .get(case)
            .ok_or("Choose a supplied record set.")?;
        core::inspect(&Rule::parse(text).map_err(fault)?, &case.records).map_err(fault)
    }
    pub fn checks(&self, text: &str) -> Vec<Check> {
        if let Err(error) = self.validate() {
            return vec![check(false, "Source task", error)];
        }
        let rule = match Rule::parse(text) {
            Ok(rule) => rule,
            Err(error) => return vec![check(false, "Rule definition", fault(error))],
        };
        self.cases.iter().map(|case| {
            let rows = match core::inspect(&rule, &case.records) {
                Ok(rows) => rows,
                Err(error) => return check(false, &case.label, fault(error)),
            };
            let wrong: Vec<_> = rows.iter().zip(&case.expected_alerts).enumerate()
                .filter(|(_, (actual, expected))| (**actual == Decision::Alert) != **expected).collect();
            let detail = if let Some((index, (actual, expected))) = wrong.first() {
                let advice = match actual {
                    Decision::Alert => "The rule also selects traffic the task excludes; check its endpoint scope and required content.",
                    Decision::ProtocolMismatch => "Check the supplied protocol against the rule header.",
                    Decision::EndpointMismatch => "Check both addresses and ports together, including the arrow direction.",
                    Decision::MissingContent(_) => "Check the required text and letter case in the supplied payload; all content conditions must match.",
                };
                format!("Record {} should {}, but the actual result is '{}'. {} {} of {} records need attention.",
                    index + 1, if **expected { "alert" } else { "not alert" }, decision_label(actual), advice, wrong.len(), rows.len())
            } else {
                format!("All {} supplied records have the required alert decisions.", rows.len())
            };
            check(wrong.is_empty(), &case.label, detail)
        }).collect()
    }
    pub fn solved(&self, text: &str) -> bool {
        let checks = self.checks(text);
        !checks.is_empty() && checks.iter().all(|c| c.passed)
    }
}
