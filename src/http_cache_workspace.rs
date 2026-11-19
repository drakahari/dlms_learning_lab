//! Source-bound cache traces and isolated header drafts.
use crate::{http_cache as model, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, String>;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn raw(s: &str) -> Result<()> {
    if s.len() > 4096 || s.chars().count() > 1024 || s.chars().any(|c| c.is_control() && c != '\t')
    {
        Err("Keep each response header to one line, at most1024 characters, without nonprinting controls.".into())
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delivery {
    Cached,
    Validated,
    Downloaded,
    Unavailable,
}
impl Delivery {
    fn matches(self, actual: model::Delivery) -> bool {
        matches!(
            (self, actual),
            (Self::Cached, model::Delivery::Cached)
                | (Self::Validated, model::Delivery::Validated)
                | (Self::Downloaded, model::Delivery::Downloaded)
                | (Self::Unavailable, model::Delivery::Unavailable)
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Cached => "Reuse stored response",
            Self::Validated => "Validate stored response",
            Self::Downloaded => "Download current response",
            Self::Unavailable => "No usable response",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Storage {
    Any,
    Required,
    Forbidden,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Body {
    Unavailable,
    Version {
        #[serde(deserialize_with = "crate::cron::text::<_,80>")]
        etag: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        body: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Delivery,4>")]
    pub delivery: Vec<Delivery>,
    pub body: Body,
    pub storage: Storage,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,model::Step,64>")]
    pub steps: Vec<model::Step>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Expected,64>")]
    pub expected: Vec<Expected>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub reference: String,
    /// An explicit header-preservation requirement, not simulated browser behavior.
    pub keep_max_age: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Resource,8>")]
    pub resources: Vec<Resource>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Trace,8>")]
    pub traces: Vec<Trace>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub value: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Header,8>")]
    pub headers: Vec<Header>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task_id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Header,8>")]
    pub headers: Vec<Header>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub selected_trace: String,
    pub checked_stamp: Option<Stamp>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StepResult {
    pub index: usize,
    pub actual: model::Observation,
    pub passed: bool,
    pub feedback: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TraceResult {
    pub id: String,
    pub label: String,
    pub steps: Vec<StepResult>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub configuration_errors: Vec<String>,
    pub traces: Vec<TraceResult>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.configuration_errors.is_empty()
            && !self.traces.is_empty()
            && self
                .traces
                .iter()
                .all(|t| !t.steps.is_empty() && t.steps.iter().all(|s| s.passed))
    }
}
impl Task {
    pub fn reference_headers(&self) -> Vec<Header> {
        self.resources
            .iter()
            .map(|r| Header {
                path: r.path.clone(),
                value: r.reference.clone(),
            })
            .collect()
    }
    fn initial_headers(&self) -> Vec<Header> {
        self.resources
            .iter()
            .map(|r| Header {
                path: r.path.clone(),
                value: r.initial.clone(),
            })
            .collect()
    }
    fn bound_headers(&self, headers: &[Header]) -> Result<()> {
        if headers.len() != self.resources.len()
            || headers
                .iter()
                .zip(&self.resources)
                .any(|(h, r)| h.path != r.path)
        {
            return Err("Saved cache headers do not match the supplied resource paths/order; original work retained.".into());
        }
        for header in headers {
            raw(&header.value)?;
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !id(&self.id)
            || self.resources.is_empty()
            || self.resources.len() > 8
            || !(2..=8).contains(&self.traces.len())
        {
            return Err("Cache tasks require version1, a stable ID,1–8 resources and2–8 independent request traces.".into());
        }
        let mut paths = BTreeSet::new();
        for r in &self.resources {
            if r.label.is_empty() || r.label.len() > 120 || !paths.insert(r.path.clone()) {
                return Err("Supply distinct paths and readable resource labels.".into());
            }
            raw(&r.initial)?;
            model::Control::parse(&r.reference, model::Side::Response)
                .map_err(|e| e.to_string())?;
        }
        let mut ids = BTreeSet::new();
        let mut used = BTreeSet::new();
        for t in &self.traces {
            if !id(&t.id)
                || t.label.is_empty()
                || t.label.len() > 120
                || !ids.insert(&t.id)
                || t.steps.is_empty()
                || t.steps.len() > 64
                || t.steps.len() != t.expected.len()
            {
                return Err("Each trace needs a distinct identifier, label and1–64requests with matching requirements.".into());
            }
            for (s, e) in t.steps.iter().zip(&t.expected) {
                used.insert(s.path.clone());
                if e.delivery.is_empty()
                    || e.delivery.len() > 4
                    || e.delivery
                        .iter()
                        .enumerate()
                        .any(|(i, d)| e.delivery[..i].contains(d))
                {
                    return Err("Each request needs distinct permitted delivery outcomes.".into());
                }
            }
        }
        if used != paths {
            return Err("Every resource must have a request case, and every case must use a supplied resource.".into());
        }
        let reference = self.report_headers(&self.reference_headers())?;
        if !reference.passed() {
            return Err("The supplied worked policy cannot satisfy every cache trace and declared header requirement.".into());
        }
        if self.report_headers(&self.initial_headers())?.passed() {
            return Err("The independent starter already satisfies the cache task.".into());
        }
        Ok(())
    }
    pub fn report_headers(&self, headers: &[Header]) -> Result<Report> {
        if self.version != 1
            || !id(&self.id)
            || self.resources.is_empty()
            || self.resources.len() > 8
            || !(2..=8).contains(&self.traces.len())
            || self
                .resources
                .iter()
                .any(|r| r.label.is_empty() || r.label.len() > 120)
            || self.traces.iter().any(|t| {
                !id(&t.id)
                    || t.label.is_empty()
                    || t.label.len() > 120
                    || t.steps.is_empty()
                    || t.steps.len() > 64
                    || t.expected.len() != t.steps.len()
                    || t.expected.iter().any(|e| {
                        e.delivery.is_empty()
                            || e.delivery.len() > 4
                            || e.delivery
                                .iter()
                                .enumerate()
                                .any(|(i, d)| e.delivery[..i].contains(d))
                    })
            })
        {
            return Err(
                "Invalid cache trace shape or resource limits; no request was graded.".into(),
            );
        }
        self.bound_headers(headers)?;
        let mut errors = Vec::new();
        let mut values = BTreeMap::new();
        for (h, r) in headers.iter().zip(&self.resources) {
            match model::Control::parse(&h.value, model::Side::Response) {
                Err(e) => errors.push(format!("{}: {:?}: {}", h.path, e.kind, e.message)),
                Ok(c) => {
                    if r.keep_max_age
                        .is_some_and(|required| c.max_age != Some(required))
                    {
                        errors.push(format!("{} must retain the stated max-age={} value while you repair shared freshness.",h.path,r.keep_max_age.unwrap()));
                    }
                }
            }
            values.insert(h.path.clone(), h.value.clone());
        }
        if !errors.is_empty() {
            return Ok(Report {
                configuration_errors: errors,
                traces: Vec::new(),
            });
        }
        let mut traces = Vec::new();
        for t in &self.traces {
            let actual =
                model::run(&t.steps, &values).map_err(|e| format!("{}: {}", t.label, e))?;
            let mut results = Vec::new();
            for (index, (a, e)) in actual.into_iter().zip(&t.expected).enumerate() {
                let body_ok = match &e.body {
                    Body::Unavailable => a.body.is_none() && a.etag.is_none(),
                    Body::Version { etag, body } => {
                        a.etag.as_ref() == Some(etag) && a.body.as_ref() == Some(body)
                    }
                };
                let delivery_ok = e.delivery.iter().any(|d| d.matches(a.delivery));
                let stored_ok = match e.storage {
                    Storage::Any => true,
                    Storage::Required => a.stored,
                    Storage::Forbidden => !a.stored,
                };
                let passed = body_ok && delivery_ok && stored_ok;
                let feedback = if !stored_ok {
                    if a.stored {
                        "A cached copy exists for this path, but this content must stay out of shared storage. Requiring validation is different from preventing storage.".into()
                    } else {
                        "This response must be available for shared reuse, but nothing was stored. Check its storage directives.".into()
                    }
                } else if !body_ok {
                    format!(
                        "The returned version does not meet this request's requirement (actual tag {}). Inspect the origin change and the stored response's freshness or validation decision.",
                        a.etag.as_deref().unwrap_or("none")
                    )
                } else if !delivery_ok {
                    format!(
                        "Actual delivery: {:?}. Required: {}. Inspect whether this response was stored, fresh enough, or required an origin check.",
                        a.delivery,
                        e.delivery
                            .iter()
                            .map(|v| v.label())
                            .collect::<Vec<_>>()
                            .join(" or ")
                    )
                } else {
                    format!("Meets requirement. {}", a.explanation)
                };
                results.push(StepResult {
                    index,
                    actual: a,
                    passed,
                    feedback,
                });
            }
            traces.push(TraceResult {
                id: t.id.clone(),
                label: t.label.clone(),
                steps: results,
            });
        }
        Ok(Report {
            configuration_errors: Vec::new(),
            traces,
        })
    }
    pub fn report(&self, d: &Draft) -> Result<Report> {
        d.bound(self)?;
        self.report_headers(&d.headers)
    }
    pub fn checks(&self, d: &Draft) -> Vec<(bool, String)> {
        if let Err(e) = d.bound(self) {
            return vec![(false, e)];
        }
        if !d.checked() {
            return vec![(false,"Not checked since editing. Choose Check caching policy to run the independent sample inputs.".into())];
        }
        match self.report(d) {
            Err(e) => vec![(false, e)],
            Ok(r) => {
                let mut rows = r
                    .configuration_errors
                    .into_iter()
                    .map(|e| (false, e))
                    .collect::<Vec<_>>();
                rows.extend(r.traces.into_iter().flat_map(|t| {
                    t.steps.into_iter().map(move |s| {
                        (
                            s.passed,
                            format!("{} · request {}: {}", t.label, s.index + 1, s.feedback),
                        )
                    })
                }));
                rows
            }
        }
    }
}
impl Draft {
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            task_id: t.id.clone(),
            headers: t.initial_headers(),
            selected_trace: t.traces[0].id.clone(),
            checked_stamp: None,
        })
    }
    pub fn checked(&self) -> bool {
        self.checked_stamp
            .as_ref()
            .is_some_and(|v| v.headers == self.headers)
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        if self.task_id != t.id || !t.traces.iter().any(|r| r.id == self.selected_trace) {
            return Err(
                "Saved cache work names a different task or trace; original retained.".into(),
            );
        }
        t.bound_headers(&self.headers)?;
        if let Some(s) = &self.checked_stamp {
            t.bound_headers(&s.headers)?;
        }
        Ok(())
    }
    pub fn edit(&mut self, t: &Task, headers: Vec<Header>) -> Result<()> {
        self.bound(t)?;
        t.bound_headers(&headers)?;
        if self.headers != headers {
            self.headers = headers;
            self.checked_stamp = None;
        }
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Report> {
        let report = t.report(self)?;
        self.checked_stamp = Some(Stamp {
            headers: self.headers.clone(),
        });
        Ok(report)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.traces.iter().map(|t| &t.steps).eq(self
                .example
                .traces
                .iter()
                .map(|t| &t.steps))
        {
            return Err(
                "Provide a different workshop and independent task, with separate request facts."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        if s.version != 1 || *s.source != *self {
            return Err(
                "Saved cache source facts differ from this authored task; original retained."
                    .into(),
            );
        }
        s.assignment.bound(&self.initial)?;
        s.practice.bound(&self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            vec![(false, e)]
        } else {
            self.initial.checks(&s.assignment)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    pub source: Box<Definition>,
    pub view: View,
    pub practice: Draft,
    pub assignment: Draft,
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported cache progress version; original work retained.".into());
        }
        self.source.validate()?;
        self.source.bound(self)
    }
    pub fn validate(&self, definition: &Definition) -> Result<()> {
        self.shape()?;
        definition.validate()?;
        definition.bound(self)
    }
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source: Box::new(d.clone()),
            view: View::Introduction,
            practice: Draft::new(&d.example)?,
            assignment: Draft::new(&d.initial)?,
        })
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.practice = Draft::new(&d.example)?;
        } else {
            self.assignment = Draft::new(&d.initial)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

mod bindings;
pub use bindings::{Action, HELP, PREFIX, context, relevant, source};
