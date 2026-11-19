//! Source-owned, deterministic offline filter tasks; never host log inspection.
use crate::log_filter::{self as core, Disposition, Filter, Kind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Matched,
    Ignored,
    Missed,
}
impl Expected {
    pub fn label(self) -> &'static str {
        match self {
            Self::Matched => "matched as a failure",
            Self::Ignored => "ignored by the exception",
            Self::Missed => "left unmatched",
        }
    }
    pub fn matches(self, actual: Disposition) -> bool {
        matches!(
            (self, actual),
            (Self::Matched, Disposition::Matched)
                | (Self::Ignored, Disposition::Ignored)
                | (Self::Missed, Disposition::Missed)
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,64>")]
    pub lines: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Expected,64>")]
    pub expected: Vec<Expected>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub log_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub filter_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub initial_filter: String,
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
pub fn fault(error: core::Fault) -> String {
    format!(
        "{}: {}",
        match error.kind {
            Kind::Syntax => "Syntax error",
            Kind::Unsupported => "Unsupported pattern or input",
            Kind::Limit => "Simulation limit",
        },
        error.detail
    )
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        if !identity(&self.id)
            || !canonical(&self.log_path)
            || !canonical(&self.filter_path)
            || self.log_path == self.filter_path
            || !(2..=12).contains(&self.cases.len())
            || self.initial_filter.len() > core::MAX_FILTER
            || !self.initial_filter.is_ascii()
            || self.initial_filter.contains('\0')
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 64 * 1024
        {
            return Err("Use a stable filter task, distinct canonical virtual paths, bounded starter and 2–12 source-owned test cases.".into());
        }
        let mut ids = BTreeSet::new();
        let mut useful = false;
        let mut negative = false;
        let source_validator =
            Filter::parse("[Definition]\nfailregex = ^<HOST>$\n").map_err(fault)?;
        for case in &self.cases {
            if !identity(&case.id)
                || !ids.insert(&case.id)
                || case.label.is_empty()
                || case.label.len() > 120
                || case.label.chars().any(char::is_control)
                || case.lines.is_empty()
                || case.lines.len() != case.expected.len()
            {
                return Err("Each filter case needs a unique identifier, readable label and an outcome for every supplied log line.".into());
            }
            core::inspect(&source_validator, &case.lines).map_err(fault)?;
            useful |= case.expected.contains(&Expected::Matched);
            negative |= case.expected.contains(&Expected::Missed);
        }
        if !useful || !negative {
            return Err("Include useful detection and contrasting non-failure messages; quiet output alone is not a working filter.".into());
        }
        Ok(())
    }
    /// Fresh behavior trials; never a command-string or cached success comparison.
    pub fn checks(&self, text: &str) -> Vec<Check> {
        if let Err(error) = self.validate() {
            return vec![check(false, "Source task", error)];
        }
        let filter = match Filter::parse(text) {
            Ok(f) => f,
            Err(e) => return vec![check(false, "Filter definition", fault(e))],
        };
        self.cases.iter().map(|case| {
            let rows=match core::inspect(&filter,&case.lines) {Ok(rows)=>rows,Err(e)=>return check(false,&case.label,fault(e))};
            let wrong:Vec<_>=rows.iter().zip(&case.expected).filter(|(r,e)|!e.matches(r.disposition)).collect();
            let detail=if let Some((row,expected))=wrong.first() {
                let actual=match row.disposition {Disposition::Matched=>"matched as a failure",Disposition::Ignored=>"ignored",Disposition::Missed=>"left unmatched"};
                let advice=match (expected,row.disposition) {
                    (Expected::Matched,Disposition::Ignored)=>"The exception also hides a genuine failure; make it more specific.",
                    (Expected::Matched,_)=>"Check the event words, changing fields and address tag against this supplied failure line.",
                    (Expected::Ignored,_)=>"The stated exception needs its own narrow ignore rule; keep genuine failures detectable.",
                    (Expected::Missed,Disposition::Matched)=>"The failure pattern also selects a non-failure; identify the event more precisely.",
                    _=>"The exception is broader than the stated policy; this message should simply remain unmatched.",
                };
                format!("Line {} should be {}, but it was {}. {} {} of {} lines need attention.",row.line,expected.label(),actual,advice,wrong.len(),rows.len())
            }else{format!("All {} supplied lines have the required outcomes.",rows.len())};
            check(wrong.is_empty(),&case.label,detail)
        }).collect()
    }
    pub fn solved(&self, text: &str) -> bool {
        let rows = self.checks(text);
        !rows.is_empty() && rows.iter().all(|r| r.passed)
    }
    pub fn observe(&self, text: &str, case: usize) -> Result<Vec<core::Row>> {
        self.validate()?;
        let source = self
            .cases
            .get(case)
            .ok_or("Choose a supplied log sample.")?;
        let filter = Filter::parse(text).map_err(fault)?;
        core::inspect(&filter, &source.lines).map_err(fault)
    }
}
