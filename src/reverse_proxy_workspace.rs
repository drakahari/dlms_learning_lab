//! Separate source-bound server-fragment practice and independent assignments.
use crate::{reverse_proxy as model, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn raw(s: &str) -> Result<()> {
    if s.len() > 32768
        || s.chars().count() > 8192
        || s.chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        Err("Keep the editor within 8192 displayed characters, without nonprinting controls.".into())
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Expected {
    Return {
        status: u16,
        #[serde(deserialize_with = "optional_body")]
        body: Option<String>,
    },
    Redirect {
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        path: String,
    },
    Forward {
        endpoint: model::Endpoint,
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        path: String,
    },
}
fn optional_body<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,256>")] String);
    Ok(Option::<Text>::deserialize(d)?.map(|x| x.0))
}
impl Expected {
    pub fn matches(&self, a: &model::Route) -> bool {
        match (self, a) {
            (
                Self::Return { status, body },
                model::Route::Return {
                    status: s, body: b, ..
                },
            ) => {
                status == s
                    && body
                        .as_ref()
                        .is_none_or(|required| b.as_ref() == Some(required))
            }
            (Self::Redirect { path }, model::Route::Redirect { path: p, .. }) => path == p,
            (
                Self::Forward { endpoint, path },
                model::Route::Proxy {
                    endpoint: e,
                    path: p,
                    ..
                },
            ) => endpoint == e && path == p,
            _ => false,
        }
    }
    pub fn label(&self) -> String {
        match self {
            Self::Return { status, body } => body.as_ref().map_or_else(
                || format!("Return {status}"),
                |b| format!("Return {status}: {b}"),
            ),
            Self::Redirect { path } => format!("Redirect 301 to {path}"),
            Self::Forward { endpoint, path } => format!("Forward to {endpoint} · {path}"),
        }
    }
}
pub fn actual_label(a: &model::Route) -> String {
    match a {
        model::Route::NotListening => "No listener on the requested port".into(),
        model::Route::Return { status, body, .. } => body.as_ref().map_or_else(
            || format!("Return {status}"),
            |b| format!("Return {status}: {b}"),
        ),
        model::Route::Redirect { path, .. } => format!("Redirect 301 to {path}"),
        model::Route::Proxy { endpoint, path, .. } => format!("Forward to {endpoint} · {path}"),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub port: u16,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub initial_config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub required_config: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub path: String,
    pub expected: Expected,
    pub actual: model::Route,
    pub passed: bool,
    pub explanation: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolicyResult {
    pub label: String,
    pub difference: Option<model::Difference>,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub configuration_error: Option<String>,
    pub cases: Vec<CaseResult>,
    pub policies: Vec<PolicyResult>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.configuration_error.is_none()
            && !self.cases.is_empty()
            && self.cases.iter().all(|c| c.passed)
            && self.policies.len() == 2
            && self.policies.iter().all(|p| p.passed)
    }
}
fn feedback(c: &Case, a: &model::Route, port: u16) -> String {
    if c.expected.matches(a) {
        return "This request meets the requirement. The full routing check also covers paths absent from this table.".into();
    }
    match a {
        model::Route::NotListening=>format!("The applied fragment has no listener on port {port}. Check the listen value, then save and apply the correction."),
        model::Route::Proxy{endpoint,path,..}=>match &c.expected{
            Expected::Forward{endpoint:e,..}if endpoint!=e=>format!("This request selects backend {endpoint}; the task requires {e}. Check the selected location's proxy endpoint."),
            Expected::Forward{path:p,..}=>format!("The backend receives {path}; this request needs {p}. A URI in proxy_pass replaces the matched location prefix; omitting the URI keeps the client path."),
            _=>"This path is forwarded when a local response is required. Check the longest matching prefix and any = exact exception; file order does not override a more specific location.".into(),
        },
        model::Route::Return{status,..}=>format!("The selected location returns {status}. The task requires {}. Check exact versus prefix matches and the selected action.",c.expected.label()),
        model::Route::Redirect{path,..}=>format!("The proxied slash location redirects this bare path to {path}. Check whether the task requires that redirect or an explicit = exact exception."),
    }
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !id(&self.id)
            || self.port == 0
            || !(2..=8).contains(&self.cases.len())
        {
            return Err(
                "Routing tasks need version1, a stable identifier, a public port and 2–8 requests."
                    .into(),
            );
        }
        let initial = model::parse(&self.initial_config).map_err(|e| e.to_string())?;
        let required = model::parse(&self.required_config).map_err(|e| e.to_string())?;
        if required.listen() != self.port {
            return Err("The required fragment must listen on the declared public port.".into());
        }
        if model::difference(&initial, &required, self.port)
            .map_err(|e| e.to_string())?
            .is_none()
        {
            return Err("Retain a genuine routing problem in the starter fragment.".into());
        }
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut forward = false;
        let mut local = false;
        for c in &self.cases {
            if !id(&c.id)
                || !ids.insert(&c.id)
                || !paths.insert(&c.path)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
            {
                return Err("Requests need distinct identifiers/paths and readable labels.".into());
            }
            model::path(&c.path).map_err(|e| e.to_string())?;
            let actual = required
                .route(self.port, &c.path)
                .map_err(|e| e.to_string())?;
            if !c.expected.matches(&actual) {
                return Err(
                    "A supplied request expectation disagrees with the required routing policy."
                        .into(),
                );
            }
            forward |= matches!(c.expected, Expected::Forward { .. });
            local |= !matches!(c.expected, Expected::Forward { .. });
        }
        if !forward || !local {
            return Err(
                "Include both a forwarded request and a deliberate local response or redirect."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn report(&self, d: &Draft) -> Result<Report> {
        self.validate()?;
        d.validate(self)?;
        let mut report = Report {
            configuration_error: None,
            cases: Vec::new(),
            policies: Vec::new(),
        };
        if d.editor != d.world.saved() {
            report.configuration_error=Some("The editor has unsaved changes. Save and apply the corrected fragment before checking the handover.".into());
            return Ok(report);
        }
        let saved = match model::parse(d.world.saved()) {
            Ok(c) => c,
            Err(e) => {
                report.configuration_error =
                    Some(format!("Saved configuration · {:?}: {e}", e.kind));
                return Ok(report);
            }
        };
        let applied = model::parse(d.world.applied()).map_err(|e| e.to_string())?;
        let required = model::parse(&self.required_config).map_err(|e| e.to_string())?;
        for c in &self.cases {
            let actual = applied
                .route(self.port, &c.path)
                .map_err(|e| e.to_string())?;
            report.cases.push(CaseResult {
                id: c.id.clone(),
                label: c.label.clone(),
                path: c.path.clone(),
                expected: c.expected.clone(),
                passed: c.expected.matches(&actual),
                explanation: feedback(c, &actual, self.port),
                actual,
            });
        }
        for (label, config) in [("Applied routing", applied), ("Saved routing", saved)] {
            let difference =
                model::difference(&config, &required, self.port).map_err(|e| e.to_string())?;
            report.policies.push(PolicyResult {
                label: label.into(),
                passed: difference.is_none(),
                difference,
            });
        }
        Ok(report)
    }
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.cases == self.example.cases
            || self.initial.required_config == self.example.required_config
        {
            return Err(
                "Use separate requests and configuration values for the example and assignment."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = s.validate(self) {
            return vec![(false, e)];
        }
        let (t, d) = if practice {
            (&self.example, &s.practice)
        } else {
            (&self.initial, &s.assignment)
        };
        if !d.checked() {
            return vec![(false,"Not checked since editing. Save and apply the fragment, then choose Check routing.".into())];
        }
        let report = match t.report(d) {
            Ok(r) => r,
            Err(e) => return vec![(false, e)],
        };
        if let Some(e) = report.configuration_error {
            return vec![(false, e)];
        }
        let mut rows = report
            .cases
            .into_iter()
            .map(|c| {
                (
                    c.passed,
                    format!(
                        "{}: required {}; actual {}. {}",
                        c.label,
                        c.expected.label(),
                        actual_label(&c.actual),
                        c.explanation
                    ),
                )
            })
            .collect::<Vec<_>>();
        rows.extend(report.policies.into_iter().map(|p|{
            let message=p.difference.map_or_else(||"Every supported normalized path meets the routing requirements.".into(),|d|format!("Path {} needs {}, but gets {}. Cover the whole stated path policy, not only the table's named requests.",d.path,actual_label(&d.expected),actual_label(&d.actual)));
            (p.passed,format!("{}: {message}",p.label))
        }));
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    pub world: model::World,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub task_id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    pub world: model::World,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub selected_case: String,
    pub checked_stamp: Option<Stamp>,
}
impl Draft {
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            task_id: t.id.clone(),
            editor: t.initial_config.clone(),
            world: model::World::new(&t.initial_config).map_err(|e| e.to_string())?,
            selected_case: t.cases[0].id.clone(),
            checked_stamp: None,
        })
    }
    fn stamp(&self) -> Stamp {
        Stamp {
            editor: self.editor.clone(),
            world: self.world.clone(),
        }
    }
    pub fn checked(&self) -> bool {
        self.checked_stamp
            .as_ref()
            .is_some_and(|s| *s == self.stamp())
    }
    pub fn shape(&self) -> Result<()> {
        if !id(&self.task_id) || !id(&self.selected_case) {
            return Err("Invalid saved routing identifiers; original retained.".into());
        }
        raw(&self.editor)?;
        self.world.validate().map_err(|e| e.to_string())?;
        if let Some(s) = &self.checked_stamp {
            raw(&s.editor)?;
            s.world.validate().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.task_id != t.id || !t.cases.iter().any(|c| c.id == self.selected_case) {
            return Err(
                "Saved routing work belongs to another task or request set; original retained."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn edit(&mut self, text: String) -> Result<()> {
        raw(&text)?;
        if self.editor != text {
            self.editor = text;
            self.checked_stamp = None;
        }
        Ok(())
    }
    pub fn save(&mut self, t: &Task) -> Result<String> {
        self.validate(t)?;
        if self.world.save(&self.editor).map_err(|e| e.to_string())? {
            self.checked_stamp = None;
        }
        Ok("Saved the virtual fragment. Applied routing has not changed.".into())
    }
    pub fn validate_saved(&self, t: &Task) -> Result<String> {
        self.validate(t)?;
        self.world.validate_saved().map_err(|e| e.to_string())?;
        Ok("Saved syntax is supported. This neither applies the fragment nor verifies its routing.".into())
    }
    pub fn apply(&mut self, t: &Task) -> Result<String> {
        self.validate(t)?;
        if self.world.apply_saved().map_err(|e| e.to_string())? {
            self.checked_stamp = None;
        }
        Ok("Applied the saved fragment to the simulated proxy. Inspect requests or check the task.".into())
    }
    pub fn check(&mut self, t: &Task) -> Result<Report> {
        let report = t.report(self)?;
        self.checked_stamp = Some(self.stamp());
        Ok(report)
    }
    pub fn inspect(&self, t: &Task) -> Result<model::Route> {
        self.validate(t)?;
        let c = t.cases.iter().find(|c| c.id == self.selected_case).unwrap();
        self.world
            .effective_route(t.port, &c.path)
            .map_err(|e| e.to_string())
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
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported routing progress version; original retained.".into());
        }
        self.source.validate()?;
        self.practice.shape()?;
        self.assignment.shape()
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        d.validate()?;
        if *self.source != *d {
            return Err("The supplied routing task changed; original drafts retained rather than silently graded against new requirements.".into());
        }
        self.practice.validate(&d.example)?;
        self.assignment.validate(&d.initial)
    }
}

mod bindings;
pub use bindings::{Action, HELP, PREFIX, context, relevant, source};

#[cfg(test)]
mod tests;
