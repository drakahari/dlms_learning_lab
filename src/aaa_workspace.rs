//! Source-held ordered authentication tasks; no server, shell or network execution.
pub use crate::local_access_workspace::Report;
use crate::{
    aaa_document as document, aaa_methods as aaa, local_access as local, management_path as path,
    management_policy as incoming, management_workspace as entry,
};
pub use entry::Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn document_text(s: &str) -> bool {
    s.len() <= crate::acl_editor::MAX_TEXT
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn method_name(m: &aaa::Method) -> String {
    match m {
        aaa::Method::LocalCase => "local-case".into(),
        aaa::Method::TrainingService { name } => format!("training service {name}"),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Authenticated {
        user: String,
        privilege: local::Privilege,
        via: aaa::Method,
    },
    AuthenticationRejected {
        via: aaa::Method,
    },
    AuthorizationDenied {
        user: String,
        via: aaa::Method,
    },
    Unavailable,
}
impl Expected {
    pub fn explanation(&self) -> String {
        match self {
            Self::Authenticated {
                user,
                privilege,
                via,
            } => format!(
                "{user} authenticated by {} and locally authorized at level {}",
                method_name(via),
                privilege.level()
            ),
            Self::AuthenticationRejected { via } => format!(
                "credential rejection returned by {}; no later method attempted",
                method_name(via)
            ),
            Self::AuthorizationDenied { user, via } => format!(
                "{user} authenticated by {}, then denied by local EXEC authorization because no local profile exists",
                method_name(via)
            ),
            Self::Unavailable => "all selected authentication methods unavailable".into(),
        }
    }
    fn matches(&self, auth: &aaa::Authentication, exec: &aaa::ExecAuthorization) -> bool {
        let last = auth.steps.last().map(|s| &s.method);
        match self {
            Self::Authenticated {
                user,
                privilege,
                via,
            } => {
                matches!(&auth.outcome,aaa::Reply::Authenticated{user:actual}if actual==user)
                    && matches!(exec,aaa::ExecAuthorization::Authorized{user:actual,privilege:p}if actual==user&&p==privilege)
                    && last == Some(via)
            }
            Self::AuthenticationRejected { via } => {
                matches!(auth.outcome, aaa::Reply::Rejected(_))
                    && matches!(exec, aaa::ExecAuthorization::NotAuthenticated)
                    && last == Some(via)
            }
            Self::AuthorizationDenied { user, via } => {
                matches!(&auth.outcome,aaa::Reply::Authenticated{user:actual}if actual==user)
                    && matches!(exec,aaa::ExecAuthorization::MissingLocalRecord{user:actual}if actual==user)
                    && last == Some(via)
            }
            Self::Unavailable => {
                auth.outcome == aaa::Reply::Unavailable
                    && matches!(exec, aaa::ExecAuthorization::NotAuthenticated)
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Availability {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub service: String,
    pub available: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub client: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub username: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub training_token: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Availability,4>")]
    pub service_states: Vec<Availability>,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4000>")]
    pub instructions: String,
    pub network: path::Network,
    pub supplied: incoming::Policy,
    pub services: aaa::Services,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,8>")]
    pub permitted_users: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,entry::Case,8>")]
    pub entry_cases: Vec<entry::Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,12>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub reference: String,
}
#[derive(Clone, Debug)]
pub enum Observation {
    PathUnavailable {
        reason: String,
    },
    PolicyRejected {
        path: crate::ipv4_routing::Probe,
        conditions: incoming::Assessment,
    },
    Identity {
        path: crate::ipv4_routing::Probe,
        authentication: aaa::Authentication,
        authorization: aaa::ExecAuthorization,
    },
}
fn identity_text(auth: &aaa::Authentication, exec: &aaa::ExecAuthorization) -> String {
    let mut rows = auth
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            format!(
                "{}. {}: {}",
                i + 1,
                method_name(&s.method),
                match &s.reply {
                    aaa::Reply::Unavailable => "unavailable; try the next method".into(),
                    aaa::Reply::Rejected(local::Rejection::UnknownUser) =>
                        "unknown account; stop authentication".into(),
                    aaa::Reply::Rejected(local::Rejection::WrongToken) =>
                        "training word does not match; stop authentication".into(),
                    aaa::Reply::Authenticated { user } =>
                        format!("authenticated {user}; stop authentication"),
                }
            )
        })
        .collect::<Vec<_>>();
    rows.push(match exec {
        aaa::ExecAuthorization::NotAuthenticated => {
            "Local EXEC authorization not reached: no authenticated identity.".into()
        }
        aaa::ExecAuthorization::MissingLocalRecord { user } => {
            format!("Local EXEC authorization denied {user}: no matching local profile.")
        }
        aaa::ExecAuthorization::Authorized { user, privilege } => format!(
            "Local EXEC authorization: {user} receives level {}.",
            privilege.level()
        ),
    });
    rows.join("\n")
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.id)
            || !prose(&self.instructions, 4000)
            || !document_text(&self.starter)
            || !document_text(&self.reference)
        {
            return Err(
                "Invalid authentication task identity, version, instructions or document bounds."
                    .into(),
            );
        }
        self.network.validate().map_err(|e| e.to_string())?;
        self.supplied.validate().map_err(|e| e.to_string())?;
        self.services.validate()?;
        if !(2..=8).contains(&self.permitted_users.len())
            || !(4..=8).contains(&self.entry_cases.len())
            || !(4..=12).contains(&self.cases.len())
        {
            return Err(
                "Supply2–8local profiles,4–8incoming cases and4–12authentication cases.".into(),
            );
        }
        let names = self.permitted_users.iter().collect::<BTreeSet<_>>();
        if names.len() != self.permitted_users.len() || names.iter().any(|s| !local::valid_name(s))
        {
            return Err("Supply unique supported local profile names.".into());
        }
        let mut labels = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut clients = BTreeSet::new();
        let mut ready = BTreeSet::new();
        let (mut telnet, mut legacy) = (false, false);
        for c in &self.entry_cases {
            if !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || !self.network.clients.iter().any(|v| v.id == c.client)
                || !seen.insert((
                    &c.client,
                    match c.protocol {
                        incoming::Protocol::Ssh1 => 1,
                        incoming::Protocol::Ssh2 => 2,
                        incoming::Protocol::Telnet => 3,
                    },
                ))
            {
                return Err(
                    "Incoming cases need unique labels/client-protocol pairs and supplied clients."
                        .into(),
                );
            }
            clients.insert(&c.client);
            match (c.expected, c.protocol) {
                (entry::Expected::ReadyForAuthentication, incoming::Protocol::Ssh2) => {
                    ready.insert(&c.client);
                }
                (entry::Expected::ProtocolDenied, incoming::Protocol::Telnet) => telnet = true,
                (entry::Expected::LegacyVersionDenied, incoming::Protocol::Ssh1) => legacy = true,
                (entry::Expected::SourceDenied, incoming::Protocol::Ssh2) => {}
                _ => return Err("Incoming cases must name their actual supported layer.".into()),
            }
        }
        if ready.len() < 2 || !telnet || !legacy || clients.len() != self.network.clients.len() {
            return Err("Exercise all incoming clients, two approved SSHv2 sources and Telnet/legacy denials.".into());
        }
        let mut inputs = BTreeSet::new();
        let mut positive = BTreeSet::new();
        let (mut service_auth, mut local_auth, mut rejection, mut outage) =
            (false, false, false, false);
        for c in &self.cases {
            if !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || !ready.contains(&c.client)
                || !local::valid_name(&c.username)
                || c.training_token.len() > 128
                || !c.training_token.bytes().all(|v| (0x20..=0x7e).contains(&v))
                || c.service_states.len() > 4
            {
                return Err("Authentication cases need distinct labels, approved clients and bounded training inputs.".into());
            }
            let mut states = BTreeSet::new();
            for v in &c.service_states {
                if !self.services.named.contains_key(&v.service) || !states.insert(&v.service) {
                    return Err("Use each supplied service state at most once in a trial.".into());
                }
                outage |= !v.available;
            }
            let mut canonical = c
                .service_states
                .iter()
                .map(|s| (&s.service, s.available))
                .collect::<Vec<_>>();
            canonical.sort();
            if !inputs.insert((&c.client, &c.username, &c.training_token, canonical)) {
                return Err(
                    "Authentication trials must use distinct input/availability conditions.".into(),
                );
            }
            let via = match &c.expected {
                Expected::Authenticated { user, via, .. } => {
                    if user != &c.username || !names.contains(user) {
                        return Err("Required authenticated local profiles must match the attempted name and permitted inventory.".into());
                    }
                    positive.insert(user);
                    match via {
                        aaa::Method::LocalCase => local_auth = true,
                        aaa::Method::TrainingService { .. } => service_auth = true,
                    };
                    Some(via)
                }
                Expected::AuthenticationRejected { via } => {
                    rejection = true;
                    Some(via)
                }
                Expected::AuthorizationDenied { user, via } => {
                    if user != &c.username || names.contains(user) {
                        return Err("A missing-local-profile trial must name the actual identity outside the permitted local inventory.".into());
                    }
                    Some(via)
                }
                Expected::Unavailable => None,
            };
            if let Some(aaa::Method::TrainingService { name }) = via
                && !self.services.named.contains_key(name)
            {
                return Err("Expected methods must refer to supplied services.".into());
            }
        }
        if positive != names || !service_auth || !local_auth || !rejection || !outage {
            return Err("Exercise every local profile, actual service and local authentication, a refusal and an explicit outage condition.".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let rows = self
            .reports(&self.reference)
            .map_err(|e| format!("Authentication reference: {e}"))?;
        if let Some(r) = rows.iter().find(|r| !r.passed()) {
            return Err(format!(
                "Unreachable authentication reference: {}",
                r.check().1
            ));
        }
        if self
            .reports(&self.starter)
            .is_ok_and(|rows| rows.iter().all(Report::passed))
        {
            return Err("The authentication starter already completes the task.".into());
        }
        Ok(())
    }
    pub fn observe(
        &self,
        settings: &document::Settings,
        case: usize,
        line: u8,
    ) -> Result<Observation, String> {
        self.shape()?;
        self.observe_validated(settings, case, line)
    }
    // reports has already validated this source once before its bounded loops.
    fn observe_validated(
        &self,
        settings: &document::Settings,
        case: usize,
        line: u8,
    ) -> Result<Observation, String> {
        let c = self
            .cases
            .get(case)
            .ok_or("Select a supplied authentication trial.")?;
        match self
            .network
            .inspect(
                &settings.incoming,
                &c.client,
                line,
                incoming::Protocol::Ssh2,
            )
            .map_err(|e| e.to_string())?
        {
            path::Observation::PathUnavailable { reason } => {
                Ok(Observation::PathUnavailable { reason })
            }
            path::Observation::Policy { path, conditions }
                if !conditions.allowed_to_authenticate() =>
            {
                Ok(Observation::PolicyRejected { path, conditions })
            }
            path::Observation::Policy { path, .. } => {
                let mut services = self.services.clone();
                for v in &c.service_states {
                    services
                        .named
                        .get_mut(&v.service)
                        .expect("validated service")
                        .available = v.available;
                }
                let authentication = aaa::authenticate(
                    &settings.authentication,
                    &services,
                    &settings.local,
                    &c.username,
                    &c.training_token,
                )?;
                let authorization = aaa::authorize_local_exec(&settings.local, &authentication)?;
                Ok(Observation::Identity {
                    path,
                    authentication,
                    authorization,
                })
            }
        }
    }
    pub fn reports(&self, text: &str) -> Result<Vec<Report>, Error> {
        self.shape().map_err(Error::Source)?;
        let settings = document::parse(&self.network, &self.supplied, &self.services, text)
            .map_err(Error::Document)?;
        let wanted = self.permitted_users.iter().collect::<BTreeSet<_>>();
        let actual = settings
            .local
            .users
            .iter()
            .map(|u| &u.name)
            .collect::<BTreeSet<_>>();
        let missing = wanted
            .difference(&actual)
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        let extra = actual
            .difference(&wanted)
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        let good = missing.is_empty() && extra.is_empty();
        let mut rows = vec![Report {
            label: "Permitted local profile inventory".into(),
            matched: usize::from(good),
            total: 1,
            scope: "inventory checks",
            first_failure: (!good).then(|| {
                format!(
                    "Missing profiles: {}; unexpected profiles: {}.",
                    if missing.is_empty() {
                        "none".into()
                    } else {
                        missing.join(", ")
                    },
                    if extra.is_empty() {
                        "none".into()
                    } else {
                        extra.join(", ")
                    }
                )
            }),
        }];
        for c in &self.entry_cases {
            let mut r = Report {
                label: c.label.clone(),
                matched: 0,
                total: self.supplied.lines.len(),
                scope: "terminal lines",
                first_failure: None,
            };
            for line in &self.supplied.lines {
                let actual = match self
                    .network
                    .inspect(&settings.incoming, &c.client, line.number, c.protocol)
                    .map_err(|e| Error::Source(e.to_string()))?
                {
                    path::Observation::Policy { conditions, .. }
                        if c.expected.matches(&conditions) =>
                    {
                        None
                    }
                    path::Observation::Policy { conditions, .. } => {
                        Some(entry::actual(&conditions))
                    }
                    path::Observation::PathUnavailable { reason } => {
                        Some(format!("IP path unavailable: {reason}"))
                    }
                };
                if let Some(actual) = actual {
                    if r.first_failure.is_none() {
                        r.first_failure = Some(format!(
                            "VTY{}: expected {}; actual {actual}.",
                            line.number,
                            c.expected.explanation()
                        ));
                    }
                } else {
                    r.matched += 1;
                }
            }
            rows.push(r);
        }
        for (index, c) in self.cases.iter().enumerate() {
            let mut r = Report {
                label: c.label.clone(),
                matched: 0,
                total: self.supplied.lines.len(),
                scope: "terminal lines",
                first_failure: None,
            };
            for line in &self.supplied.lines {
                let actual = match self
                    .observe_validated(&settings, index, line.number)
                    .map_err(Error::Source)?
                {
                    Observation::Identity {
                        authentication,
                        authorization,
                        ..
                    } if c.expected.matches(&authentication, &authorization) => None,
                    Observation::Identity {
                        authentication,
                        authorization,
                        ..
                    } => Some(identity_text(&authentication, &authorization)),
                    Observation::PolicyRejected { conditions, .. } => Some(format!(
                        "Incoming policy stops authentication: {}",
                        entry::actual(&conditions)
                    )),
                    Observation::PathUnavailable { reason } => Some(format!(
                        "IP path unavailable before authentication: {reason}"
                    )),
                };
                if let Some(actual) = actual {
                    if r.first_failure.is_none() {
                        r.first_failure = Some(format!(
                            "VTY{}: expected {}; actual {actual}.",
                            line.number,
                            c.expected.explanation()
                        ));
                    }
                } else {
                    r.matched += 1;
                }
            }
            rows.push(r);
        }
        Ok(rows)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported authentication lesson version.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.network == self.example.network
            || self.initial.reference == self.example.reference
        {
            return Err(
                "Local-account practice needs a separate identity, network and worked document."
                    .into(),
            );
        }
        Ok(())
    }
}

pub const MAX_SESSION: usize = 2 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub applied: String,
    pub checked: bool,
    pub selected_case: u8,
    pub selected_line: u8,
    #[serde(skip)]
    pub observation: Option<String>,
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self, String> {
        t.validate()?;
        Ok(Self {
            draft: t.starter.clone(),
            applied: t.starter.clone(),
            checked: false,
            selected_case: 0,
            selected_line: t.supplied.lines[0].number,
            observation: None,
        })
    }
    fn valid_shape(&self) -> bool {
        self.draft.len() <= crate::acl_editor::MAX_TEXT
            && self.applied.len() <= crate::acl_editor::MAX_TEXT
            && self.selected_case < 12
            && self.selected_line <= 98
            && self.observation.as_ref().is_none_or(|s| s.len() <= 8192)
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        t.shape()?;
        if !self.valid_shape()
            || usize::from(self.selected_case) >= t.cases.len()
            || !t
                .supplied
                .lines
                .iter()
                .any(|l| l.number == self.selected_line)
        {
            return Err(
                "Saved authentication work or its trial selectors exceed this task's bounds."
                    .into(),
            );
        }
        if self.applied != t.starter {
            document::parse(&t.network, &t.supplied, &t.services, &self.applied)
                .map_err(|e| format!("Saved applied authentication document is invalid: {e}"))?;
        }
        Ok(())
    }
    pub fn invalidate(&mut self) {
        self.checked = false;
        self.observation = None;
    }
    pub fn edit(&mut self, text: String) -> Result<(), String> {
        if self.draft != text {
            self.invalidate();
            if text.len() > crate::acl_editor::MAX_TEXT {
                return Err("The draft exceeds 64 KiB; the previous bounded text was retained and must be checked again.".into());
            }
            self.draft = text;
        }
        Ok(())
    }
    pub fn select_sample(&mut self, t: &Task, case: usize, line: u8) -> Result<(), String> {
        self.bound(t)?;
        if case >= t.cases.len() || !t.supplied.lines.iter().any(|l| l.number == line) {
            return Err("Select one supplied login trial and terminal line.".into());
        }
        if usize::from(self.selected_case) != case || self.selected_line != line {
            self.selected_case = case as u8;
            self.selected_line = line;
            self.observation = None;
        }
        Ok(())
    }
    pub fn apply(&mut self, t: &Task) -> Result<(), Error> {
        self.invalidate();
        self.bound(t).map_err(Error::Source)?;
        document::parse(&t.network, &t.supplied, &t.services, &self.draft)
            .map_err(Error::Document)?;
        self.applied = self.draft.clone();
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<Report>, Error> {
        self.apply(t)?;
        let reports = t.reports(&self.applied)?;
        self.checked = true;
        Ok(reports)
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        if !self.checked || self.draft != self.applied {
            return vec![(false,"Check the current document against the local profile inventory, incoming policy and every supplied authentication/authorization case.".into())];
        }
        match t.reports(&self.applied) {
            Ok(r) => r.iter().map(Report::check).collect(),
            Err(e) => vec![(false, e.to_string())],
        }
    }
    pub fn inspect(&mut self, t: &Task) -> Result<&str, String> {
        self.bound(t)?;
        self.observation = None;
        if self.draft != self.applied {
            return Err("Apply the edited document before inspecting its actual effect.".into());
        }
        let settings = document::parse(&t.network, &t.supplied, &t.services, &self.applied)
            .map_err(|e| e.to_string())?;
        let observed = t.observe(
            &settings,
            usize::from(self.selected_case),
            self.selected_line,
        )?;
        let output = match observed {
            Observation::PathUnavailable { reason } => {
                format!("IPv4 path unavailable: {reason}\nNo authentication method was evaluated.")
            }
            Observation::PolicyRejected { path, conditions } => format!(
                "Client source: {}\nIPv4 forward/reply path: {}/{}hops\nVTY{}: incoming policy stops authentication: {}.",
                path.source,
                path.forward.hops.len(),
                path.reply.hops.len(),
                self.selected_line,
                entry::actual(&conditions)
            ),
            Observation::Identity {
                path,
                authentication,
                authorization,
            } => format!(
                "Client source: {}\nIPv4 forward/reply path: {}/{}hops\nVTY{}\n{}\nSupplied training-service metadata and local EXEC profiles; no real server, encrypted session or host login.",
                path.source,
                path.forward.hops.len(),
                path.reply.hops.len(),
                self.selected_line,
                identity_text(&authentication, &authorization)
            ),
        };
        if output.len() > 8192 {
            return Err("The authentication trial exceeds its output bound.".into());
        }
        self.observation = Some(output);
        Ok(self.observation.as_deref().expect("stored observation"))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: crate::script_teaching::View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Scope::new(&d.initial)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn valid_shape(&self) -> bool {
        self.version == 1
            && key(&self.origin)
            && self.assignment.valid_shape()
            && self.practice.valid_shape()
            && crate::cron_workspace::within(self, MAX_SESSION)
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        d.validate()?;
        if !self.valid_shape() || self.origin != d.initial.id {
            return Err(
                "Saved authentication work belongs to another lesson or unsupported version."
                    .into(),
            );
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn from_json(d: &Definition, raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved authentication work exceeds 2 MiB.".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported saved authentication work.".to_string())?;
        s.bound(d)?;
        Ok(s)
    }
    pub fn checks(&self, d: &Definition) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(d) {
            return vec![(false, e)];
        }
        self.assignment.checks(&d.initial)
    }
    pub fn fill_example(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.practice.edit(d.example.reference.clone())
    }
    pub fn reset_assignment(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.assignment = Scope::new(&d.initial)?;
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.practice = Scope::new(&d.example)?;
        Ok(())
    }
}

pub const PREFIX: &str = "AAA ACCESS DOCUMENT\n";
pub const HELP: &str = "Edit the complete authentication document and inspect supplied trials. Ordered methods compare training identities; only unavailable methods advance, while an explicit rejection stops. Local EXEC authorization separately looks up the authenticated identity's profile. Supplied service availability is metadata, not a real AAA protocol/server, SSH session, cryptography or host login. Use only supplied training words; drafts are saved locally.";
impl Session {
    /// Internal source/reference adapter, not a learner terminal command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        let raw = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if raw.len() > crate::acl_editor::MAX_TEXT {
            return Err("AAA access data exceeds 64 KiB.".into());
        }
        let mut next = self.assignment.clone();
        next.edit(raw.into())?;
        let reports = next.check(&d.initial).map_err(|e| e.to_string())?;
        self.assignment = next;
        Ok(format!(
            "AAA access checked: {}/{} requirements match",
            reports.iter().filter(|r| r.passed()).count(),
            reports.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::AaaAccess { simulation },
        crate::engine::Goal::AaaAccess { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("AAA access source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Local-account source identity, task and supplied context must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::AaaAccess { simulation } = &a.state else {
        return Err("AAA access work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible authentication attempt metadata; original retained.".into());
    }
    simulation.bound(d)
}

impl Definition {
    pub fn bound(&self, session: &Session) -> Result<(), String> {
        session.bound(self)
    }
    pub fn checks(&self, session: &Session) -> Vec<(bool, String)> {
        session.checks(self)
    }
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.valid_shape() {
            Ok(())
        } else {
            Err("Invalid or unsupported saved authentication session.".into())
        }
    }
}
