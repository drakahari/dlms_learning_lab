//! Source-bound service-name tasks. All text and transport state are virtual.
use crate::{
    script_teaching::View, service_clients as client, service_names as names, transport_core as net,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(s: &str, cap: usize) -> Result<()> {
    if s.len() > cap
        || s.chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        Err("Text exceeds the virtual workspace limit or contains nonprinting controls.".into())
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Clients,
    Database,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub protocol: net::Protocol,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub service: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub client: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_role: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub payload: String,
    #[serde(deserialize_with = "roles")]
    pub disabled_roles: Vec<String>,
}
fn roles<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Role(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    Ok(crate::cron_workspace::sequence::<D, Role, 8>(d)?
        .into_iter()
        .map(|r| r.0)
        .collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub mode: Mode,
    pub profile: client::Profile,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub initial_database: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub required_database: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Setting,4>")]
    pub initial_clients: Vec<Setting>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Setting,4>")]
    pub required_clients: Vec<Setting>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Request,8>")]
    pub requests: Vec<Request>,
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
    pub passed: bool,
    pub required: String,
    pub actual: Option<client::Report>,
    pub explanation: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub cases: Vec<CaseResult>,
    pub database_errors: Vec<String>,
    pub unsaved: bool,
}
impl Report {
    pub fn passed(&self) -> bool {
        !self.unsaved
            && self.database_errors.is_empty()
            && !self.cases.is_empty()
            && self.cases.iter().all(|c| c.passed)
    }
}
fn meanings(required: &str, actual: &str) -> Vec<String> {
    let actual = match names::Database::parse(actual) {
        Ok(v) => v,
        Err(e) => return vec![format!("{:?}: {e}", e.kind)],
    };
    let required = match names::Database::parse(required) {
        Ok(v) => v,
        Err(e) => return vec![format!("Invalid supplied reference: {e}")],
    };
    let mut errors = Vec::new();
    for r in required.records() {
        for name in std::iter::once(&r.name).chain(&r.aliases) {
            let q = names::Query::parse(&format!("{name}/{}", names::protocol_name(r.protocol)))
                .unwrap();
            if actual.lookup(&q).is_none_or(|a| a.port != r.port) {
                errors.push(format!("Preserve {name}/{} at port {}. Editing the name database must not change unrelated service meanings.",names::protocol_name(r.protocol),r.port));
            }
        }
    }
    errors
}
fn settings_shape(settings: &[Setting]) -> Result<()> {
    if !(1..=4).contains(&settings.len()) {
        return Err("Supply 1–4 client settings.".into());
    }
    let mut ids = BTreeSet::new();
    for s in settings {
        if !id(&s.id) || !ids.insert(&s.id) {
            return Err("Client identifiers must be distinct and bounded.".into());
        }
        text(&s.service, 64)?;
    }
    Ok(())
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.profile.validate().map_err(|e| e.to_string())?;
        if self.version != 1 || !id(&self.id) || !(2..=8).contains(&self.requests.len()) {
            return Err(
                "Service tasks need version 1, a stable identifier and 2–8 requests.".into(),
            );
        }
        text(&self.initial_database, 32768)?;
        names::Database::parse(&self.initial_database).map_err(|e| e.to_string())?;
        names::Database::parse(&self.required_database).map_err(|e| e.to_string())?;
        settings_shape(&self.initial_clients)?;
        settings_shape(&self.required_clients)?;
        if self
            .initial_clients
            .iter()
            .map(|c| &c.id)
            .collect::<Vec<_>>()
            != self
                .required_clients
                .iter()
                .map(|c| &c.id)
                .collect::<Vec<_>>()
        {
            return Err("Keep the same client identities and order in the solution.".into());
        }
        match self.mode {
            Mode::Clients if self.initial_database != self.required_database => {
                return Err("Client tasks keep the supplied name database fixed.".into());
            }
            Mode::Database if self.initial_clients != self.required_clients => {
                return Err("Database tasks keep client selections fixed.".into());
            }
            _ => {}
        }
        let mut ids = BTreeSet::new();
        for r in &self.requests {
            if !id(&r.id)
                || !ids.insert(&r.id)
                || !id(&r.client)
                || r.label.trim().is_empty()
                || r.label.len() > 120
                || r.label.chars().any(char::is_control)
                || !self.initial_clients.iter().any(|s| s.id == r.client)
                || self.profile.listener(&r.required_role).is_none()
            {
                return Err("Requests need distinct identifiers, a supplied client and a supplied destination role.".into());
            }
            text(&r.payload, 1024)?;
            // This validates source-held disabled roles with the existing reducer.
            let target = self.profile.listener(&r.required_role).unwrap();
            client::run(
                &self.profile,
                &self.required_database,
                target.protocol,
                &target.port.to_string(),
                &r.payload,
                &r.disabled_roles,
            )
            .map_err(|e| e.to_string())?;
        }
        for c in &self.initial_clients {
            let requests = self
                .requests
                .iter()
                .filter(|r| r.client == c.id)
                .collect::<Vec<_>>();
            if requests.len() < 2
                || !requests
                    .iter()
                    .any(|r| !r.disabled_roles.contains(&r.required_role))
                || !requests
                    .iter()
                    .any(|r| r.disabled_roles.contains(&r.required_role))
            {
                return Err("Each client needs both an available-listener case and a source-held unavailable-listener case.".into());
            }
            if requests
                .iter()
                .any(|r| r.required_role != requests[0].required_role)
            {
                return Err("Each supplied client has one consistent destination role.".into());
            }
        }
        if !self
            .evaluate(&self.required_database, &self.required_clients, false)
            .passed()
        {
            return Err(
                "The supplied solution must satisfy all behavior and reference checks.".into(),
            );
        }
        if self
            .evaluate(&self.initial_database, &self.initial_clients, false)
            .passed()
        {
            return Err("Keep a genuine problem in the assignment starter.".into());
        }
        Ok(())
    }
    fn evaluate(&self, database: &str, settings: &[Setting], unsaved: bool) -> Report {
        let mut report = Report {
            cases: Vec::new(),
            database_errors: if self.mode == Mode::Database {
                meanings(&self.required_database, database)
            } else {
                Vec::new()
            },
            unsaved,
        };
        for r in &self.requests {
            let s = settings.iter().find(|c| c.id == r.client).unwrap();
            let target = self.profile.listener(&r.required_role).unwrap();
            let available = !r.disabled_roles.contains(&r.required_role);
            let expected = if available {
                net::Outcome::Delivered
            } else {
                net::Outcome::NoListener
            };
            let required = format!(
                "{} · {}/{} · {}",
                r.required_role,
                target.port,
                names::protocol_name(target.protocol),
                if available {
                    "payload delivered"
                } else {
                    "no listener (supplied outage)"
                }
            );
            let (passed, actual, explanation) = match client::run(
                &self.profile,
                database,
                s.protocol,
                &s.service,
                &r.payload,
                &r.disabled_roles,
            ) {
                Err(e) => (false, None, format!("{:?}: {e}", e.kind)),
                Ok(a) => {
                    let correct_target =
                        a.resolved.protocol == target.protocol && a.resolved.port == target.port;
                    let payload_ok = if available {
                        a.transport.received.as_deref() == Some(r.payload.as_str())
                    } else {
                        a.transport.received.is_none()
                    };
                    let passed = correct_target && a.transport.outcome == expected && payload_ok;
                    let why = if !correct_target {
                        format!(
                            "The client selects {}/{}; {} needs {}/{}. A name lookup describes a port, and delivery to a different open service does not meet this task.",
                            a.resolved.port,
                            names::protocol_name(a.resolved.protocol),
                            r.required_role,
                            target.port,
                            names::protocol_name(target.protocol)
                        )
                    } else if passed && available {
                        "The client selects the required listener and the simulated payload arrives. This does not test the real application's protocol.".into()
                    } else if passed {
                        "The name still resolves correctly while the supplied listener is unavailable. A services-file entry is not evidence of a running server.".into()
                    } else {
                        "The destination is correct, but the modeled transport result does not meet this case. Inspect the supplied listener availability and actual payload.".into()
                    };
                    (passed, Some(a), why)
                }
            };
            report.cases.push(CaseResult {
                id: r.id.clone(),
                label: r.label.clone(),
                passed,
                required,
                actual,
                explanation,
            });
        }
        report
    }
    pub fn report(&self, d: &Draft) -> Result<Report> {
        self.validate()?;
        d.validate(self)?;
        Ok(self.evaluate(&d.saved_database, &d.clients, d.editor != d.saved_database))
    }
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.requests == self.example.requests
            || self.initial.profile == self.example.profile
            || self.initial.mode != self.example.mode
        {
            return Err(
                "Use separate example values and requests with the same workspace type.".into(),
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
            return vec![(
                false,
                "Not checked since editing. Save file changes, then choose Check clients.".into(),
            )];
        }
        match t.report(d) {
            Err(e) => vec![(false, e)],
            Ok(r) => {
                let mut rows = Vec::new();
                if r.unsaved {
                    rows.push((
                        false,
                        "The editor has unsaved changes. Save the virtual file before checking."
                            .into(),
                    ))
                }
                rows.extend(r.database_errors.into_iter().map(|e| (false, e)));
                rows.extend(
                    r.cases
                        .into_iter()
                        .map(|c| (c.passed, format!("{}: {}", c.label, c.explanation))),
                );
                rows
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub saved_database: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Setting,4>")]
    pub clients: Vec<Setting>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task_id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub saved_database: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Setting,4>")]
    pub clients: Vec<Setting>,
    pub checked_stamp: Option<Stamp>,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub terminal_input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,20000>")]
    pub transcript: String,
}
impl Draft {
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            task_id: t.id.clone(),
            editor: t.initial_database.clone(),
            saved_database: t.initial_database.clone(),
            clients: t.initial_clients.clone(),
            checked_stamp: None,
            terminal_input: String::new(),
            transcript: String::new(),
        })
    }
    fn stamp(&self) -> Stamp {
        Stamp {
            editor: self.editor.clone(),
            saved_database: self.saved_database.clone(),
            clients: self.clients.clone(),
        }
    }
    pub fn checked(&self) -> bool {
        self.checked_stamp
            .as_ref()
            .is_some_and(|s| *s == self.stamp())
    }
    pub fn shape(&self) -> Result<()> {
        if !id(&self.task_id) {
            return Err("Invalid saved service-task identity; original retained.".into());
        }
        text(&self.editor, 32768)?;
        text(&self.saved_database, 32768)?;
        settings_shape(&self.clients)?;
        text(&self.terminal_input, 256)?;
        text(&self.transcript, 20000)?;
        if let Some(s) = &self.checked_stamp {
            text(&s.editor, 32768)?;
            text(&s.saved_database, 32768)?;
            settings_shape(&s.clients)?
        }
        Ok(())
    }
    pub fn validate(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.task_id != t.id
            || self.clients.iter().map(|c| &c.id).collect::<Vec<_>>()
                != t.initial_clients.iter().map(|c| &c.id).collect::<Vec<_>>()
        {
            return Err(
                "Saved service work belongs to another supplied client set; original retained."
                    .into(),
            );
        }
        match t.mode {
            Mode::Clients
                if self.editor != t.initial_database
                    || self.saved_database != t.initial_database =>
            {
                Err("This task keeps the supplied services file read-only.".into())
            }
            Mode::Database if self.clients != t.initial_clients => {
                Err("This task keeps supplied client selections fixed.".into())
            }
            _ => Ok(()),
        }
    }
    pub fn edit(&mut self, t: &Task, value: String) -> Result<()> {
        self.validate(t)?;
        if t.mode != Mode::Database {
            return Err(
                "Use the client settings for this task; the reference file is read-only.".into(),
            );
        }
        text(&value, 32768)?;
        if self.editor != value {
            self.editor = value;
            self.checked_stamp = None;
        }
        Ok(())
    }
    pub fn save(&mut self, t: &Task) -> Result<()> {
        self.validate(t)?;
        if t.mode != Mode::Database {
            return Err("This task's reference file is read-only.".into());
        }
        if self.saved_database != self.editor {
            self.saved_database = self.editor.clone();
            self.checked_stamp = None;
        }
        Ok(())
    }
    pub fn configure(
        &mut self,
        t: &Task,
        which: &str,
        protocol: net::Protocol,
        service: String,
    ) -> Result<()> {
        self.validate(t)?;
        if t.mode != Mode::Clients {
            return Err(
                "This task keeps client selections fixed; edit the virtual services file.".into(),
            );
        }
        text(&service, 64)?;
        let s = self
            .clients
            .iter_mut()
            .find(|s| s.id == which)
            .ok_or("Choose a supplied client.")?;
        if s.protocol != protocol || s.service != service {
            s.protocol = protocol;
            s.service = service;
            self.checked_stamp = None;
        }
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Report> {
        let r = t.report(self)?;
        self.checked_stamp = Some(self.stamp());
        Ok(r)
    }
    pub fn command(&mut self, t: &Task, input: &str) -> Result<()> {
        self.validate(t)?;
        text(input, 256)?;
        let result = match names::execute(&self.saved_database, input) {
            Ok(r) => format!("{}[exit {}]\n", r.stdout, r.status),
            Err(e) => format!("{:?}: {e}\n", e.kind),
        };
        let entry = format!("$ {input}\n{result}");
        // A bounded visible tail, not an unbounded history or grading authority.
        self.transcript.push_str(&entry);
        if self.transcript.len() > 20000 {
            let start = self.transcript.len() - 20000;
            let start = (start..=self.transcript.len())
                .find(|i| self.transcript.is_char_boundary(*i))
                .unwrap();
            self.transcript = self.transcript[start..].to_string();
        }
        self.terminal_input.clear();
        Ok(())
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
            return Err("Unsupported service progress version; original retained.".into());
        }
        self.source.validate()?;
        self.practice.shape()?;
        self.assignment.shape()
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        d.validate()?;
        if *self.source != *d {
            return Err("Supplied service requirements changed; original drafts retained.".into());
        }
        self.practice.validate(&d.example)?;
        self.assignment.validate(&d.initial)
    }
}

mod bindings;
pub use bindings::{Action, HELP, PREFIX, context, source};

#[cfg(test)]
mod tests;
