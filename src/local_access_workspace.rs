//! Source-held local-account outcomes, not command-string or saved-report grading.
use crate::{
    local_access as local, local_access_document as document, management_path as path,
    management_policy as incoming, management_workspace as entry,
};
pub use entry::Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn document_text(s: &str) -> bool {
    s.len() <= crate::acl_editor::MAX_TEXT
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Authenticated {
        user: String,
        privilege: local::Privilege,
    },
    RejectedCredential,
}
impl Expected {
    fn explanation(&self) -> String {
        match self{Self::Authenticated{user,privilege}=>format!("local record {user} authenticated at level {}",privilege.level()),Self::RejectedCredential=>"rejection by the local account check after the IP path and incoming policy permit the trial".into()}
    }
    fn matches(&self, decision: &local::Decision) -> bool {
        match (self, decision) {
            (
                Self::Authenticated { user, privilege },
                local::Decision::Authenticated {
                    user: actual,
                    privilege: level,
                },
            ) => user == actual && privilege == level,
            (Self::RejectedCredential, local::Decision::Rejected(_)) => true,
            _ => false,
        }
    }
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
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,8>")]
    pub permitted_users: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,entry::Case,8>")]
    pub entry_cases: Vec<entry::Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub reference: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub label: String,
    pub matched: usize,
    pub total: usize,
    pub scope: &'static str,
    pub first_failure: Option<String>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.matched == self.total && self.first_failure.is_none()
    }
    pub fn check(&self) -> (bool, String) {
        (
            self.passed(),
            format!(
                "{}: {}/{} {}. {}",
                self.label,
                self.matched,
                self.total,
                self.scope,
                self.first_failure
                    .as_deref()
                    .unwrap_or("All requirements match.")
            ),
        )
    }
}
fn decision_text(decision: &local::Decision) -> String {
    match decision {
        local::Decision::Authenticated { user, privilege } => {
            format!("{user} authenticated at level {}", privilege.level())
        }
        local::Decision::Rejected(local::Rejection::UnknownUser) => {
            "the submitted account name has no matching local record".into()
        }
        local::Decision::Rejected(local::Rejection::WrongToken) => {
            "the local record exists, but its training word does not match this trial".into()
        }
    }
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
                "Invalid local-account task version, identifier, instructions or document bounds."
                    .into(),
            );
        }
        self.network.validate().map_err(|e| e.to_string())?;
        self.supplied.validate().map_err(|e| e.to_string())?;
        if !(2..=8).contains(&self.permitted_users.len())
            || !(4..=8).contains(&self.entry_cases.len())
            || !(4..=8).contains(&self.cases.len())
        {
            return Err("Supply 2–8 permitted account names and 4–8 cases for each of incoming policy and local accounts.".into());
        }
        let names = self.permitted_users.iter().collect::<BTreeSet<_>>();
        if names.len() != self.permitted_users.len() || names.iter().any(|n| !local::valid_name(n))
        {
            return Err(
                "Permitted account names must be unique supported local-case names.".into(),
            );
        }
        let mut ready = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut labels = BTreeSet::new();
        let mut clients = BTreeSet::new();
        let (mut telnet, mut legacy) = (false, false);
        for c in &self.entry_cases {
            if !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || !self.network.clients.iter().any(|v| v.id == c.client)
            {
                return Err(
                    "Incoming-policy cases need unique labels and supplied clients.".into(),
                );
            }
            let protocol = match c.protocol {
                incoming::Protocol::Ssh1 => 1,
                incoming::Protocol::Ssh2 => 2,
                incoming::Protocol::Telnet => 3,
            };
            if !seen.insert((&c.client, protocol)) {
                return Err("Incoming-policy client/protocol cases must be distinct.".into());
            }
            clients.insert(&c.client);
            match(c.expected,c.protocol){(entry::Expected::ReadyForAuthentication,incoming::Protocol::Ssh2)=>{ready.insert(&c.client);},(entry::Expected::ProtocolDenied,incoming::Protocol::Telnet)=>telnet=true,(entry::Expected::LegacyVersionDenied,incoming::Protocol::Ssh1)=>legacy=true,(entry::Expected::SourceDenied,incoming::Protocol::Ssh2)=>{},_=>return Err("Incoming cases must name the supported SSH version, Telnet or source rejection layer.".into())}
        }
        if ready.len() < 2 || !telnet || !legacy || clients.len() != self.network.clients.len() {
            return Err("Preserve two approved SSH version 2 clients, reject Telnet/legacy SSH and exercise every supplied client.".into());
        }
        let mut seen = BTreeSet::new();
        let mut authenticated = BTreeSet::new();
        let mut approved_clients = BTreeSet::new();
        let mut rejected = false;
        for c in &self.cases {
            if !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || !ready.contains(&c.client)
                || !local::valid_name(&c.username)
                || c.training_token.len() > 128
                || !c.training_token.bytes().all(|b| (0x20..=0x7e).contains(&b))
                || !seen.insert((&c.client, &c.username, &c.training_token))
            {
                return Err("Local cases need distinct labels/inputs, supported training values and an approved incoming client.".into());
            }
            match &c.expected {
                Expected::Authenticated { user, .. } => {
                    if user != &c.username || !names.contains(user) {
                        return Err("An expected local identity must match the submitted local-case name and permitted inventory.".into());
                    }
                    authenticated.insert(user);
                    approved_clients.insert(&c.client);
                }
                Expected::RejectedCredential => rejected = true,
            }
        }
        if authenticated != names || approved_clients.len() < 2 || !rejected {
            return Err("Exercise every permitted account, two approved clients and a rejected local credential trial.".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let reference = self
            .reports(&self.reference)
            .map_err(|e| format!("Local-account reference: {e}"))?;
        if let Some(r) = reference.iter().find(|r| !r.passed()) {
            return Err(format!(
                "Local-account reference is unreachable: {}",
                r.check().1
            ));
        }
        if self
            .reports(&self.starter)
            .is_ok_and(|r| r.iter().all(Report::passed))
        {
            return Err("The local-account starter already completes this task.".into());
        }
        Ok(())
    }
    pub fn reports(&self, text: &str) -> Result<Vec<Report>, Error> {
        self.shape().map_err(Error::Source)?;
        let settings =
            document::parse(&self.network, &self.supplied, text).map_err(Error::Document)?;
        let mut reports = vec![];
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
        reports.push(Report{label:"Permitted local account inventory".into(),matched:usize::from(good),total:1,scope:"inventory checks",first_failure:(!good).then(||format!("Missing accounts: {}; unexpected accounts: {}. Keep only the required named records.",if missing.is_empty(){"none".into()}else{missing.join(", ")},if extra.is_empty(){"none".into()}else{extra.join(", ")}))});
        for c in &self.entry_cases {
            let mut r = Report {
                label: c.label.clone(),
                matched: 0,
                total: self.supplied.lines.len(),
                scope: "terminal lines",
                first_failure: None,
            };
            for line in &self.supplied.lines {
                let observation = self
                    .network
                    .inspect(&settings.incoming, &c.client, line.number, c.protocol)
                    .map_err(|e| Error::Source(e.to_string()))?;
                let mismatch = match observation {
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
                if let Some(actual) = mismatch {
                    if r.first_failure.is_none() {
                        r.first_failure = Some(format!(
                            "VTY {}: expected {}; actual {actual}.",
                            line.number,
                            c.expected.explanation()
                        ));
                    }
                } else {
                    r.matched += 1;
                }
            }
            reports.push(r);
        }
        for c in &self.cases {
            let binding = &self
                .network
                .clients
                .iter()
                .find(|v| v.id == c.client)
                .expect("validated client")
                .binding;
            let source = self.network.topology.devices[&binding.device].interfaces
                [&binding.interface]
                .address
                .as_ref()
                .expect("validated address")
                .address;
            let mut r = Report {
                label: c.label.clone(),
                matched: 0,
                total: self.supplied.lines.len(),
                scope: "terminal lines",
                first_failure: None,
            };
            for line in &self.supplied.lines {
                let observation = local::inspect(
                    &self.network,
                    &settings.incoming,
                    &settings.local,
                    incoming::Attempt {
                        line: line.number,
                        source,
                        protocol: incoming::Protocol::Ssh2,
                    },
                    &c.client,
                    &c.username,
                    &c.training_token,
                )
                .map_err(Error::Source)?;
                let mismatch = match observation {
                    local::Observation::Local { decision, .. } if c.expected.matches(&decision) => {
                        None
                    }
                    local::Observation::Local { decision, .. } => Some(decision_text(&decision)),
                    local::Observation::PolicyRejected { conditions, .. } => Some(format!(
                        "incoming policy stops the trial before the local record check: {}",
                        entry::actual(&conditions)
                    )),
                    local::Observation::PathUnavailable { reason } => Some(format!(
                        "IP path unavailable before the local record check: {reason}"
                    )),
                };
                if let Some(actual) = mismatch {
                    if r.first_failure.is_none() {
                        r.first_failure = Some(format!(
                            "VTY {}: expected {}; actual {actual}.",
                            line.number,
                            c.expected.explanation()
                        ));
                    }
                } else {
                    r.matched += 1;
                }
            }
            reports.push(r);
        }
        Ok(reports)
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
            return Err("Unsupported local-account lesson version.".into());
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
            && self.selected_case < 8
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
                "Saved local-account work or its trial selectors exceed this task's bounds.".into(),
            );
        }
        if self.applied != t.starter {
            document::parse(&t.network, &t.supplied, &self.applied)
                .map_err(|e| format!("Saved applied local-account document is invalid: {e}"))?;
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
        document::parse(&t.network, &t.supplied, &self.draft).map_err(Error::Document)?;
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
            return vec![(false,"Check the current document against the account inventory, incoming policy and every supplied login case.".into())];
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
        let settings =
            document::parse(&t.network, &t.supplied, &self.applied).map_err(|e| e.to_string())?;
        let c = &t.cases[usize::from(self.selected_case)];
        let binding = &t
            .network
            .clients
            .iter()
            .find(|v| v.id == c.client)
            .expect("validated client")
            .binding;
        let source = t.network.topology.devices[&binding.device].interfaces[&binding.interface]
            .address
            .as_ref()
            .expect("validated source")
            .address;
        let observed = local::inspect(
            &t.network,
            &settings.incoming,
            &settings.local,
            incoming::Attempt {
                source,
                line: self.selected_line,
                protocol: incoming::Protocol::Ssh2,
            },
            &c.client,
            &c.username,
            &c.training_token,
        )?;
        let output = match observed {
            local::Observation::PathUnavailable { reason } => format!(
                "IPv4 path unavailable: {reason}\nNo local credential comparison was performed."
            ),
            local::Observation::PolicyRejected { path, conditions } => format!(
                "Client source: {}\nIPv4 forward/reply path: {}/{} hops\nVTY {}: incoming policy stops the trial: {}.\nNo local credential comparison was performed.",
                path.source,
                path.forward.hops.len(),
                path.reply.hops.len(),
                self.selected_line,
                entry::actual(&conditions)
            ),
            local::Observation::Local { path, decision } => format!(
                "Client source: {}\nIPv4 forward/reply path: {}/{} hops\nVTY {}: {}.\nSimulated local-case record check and local EXEC authorization; no encrypted session or host login.",
                path.source,
                path.forward.hops.len(),
                path.reply.hops.len(),
                self.selected_line,
                decision_text(&decision)
            ),
        };
        if output.len() > 8192 {
            return Err("The local-access trial exceeds its output bound.".into());
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
                "Saved local-account work belongs to another lesson or unsupported version.".into(),
            );
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn from_json(d: &Definition, raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved local-account work exceeds 2 MiB.".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported saved local-account work.".to_string())?;
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

pub const PREFIX: &str = "LOCAL ACCOUNT DOCUMENT\n";
pub const HELP: &str = "Use the native local-account document and supplied training login trials. This bounded simulator checks virtual paths, incoming policy and explicit local-case records/local EXEC levels; it does not execute IOS, host logins, SSH or cryptographic exchanges.";
impl Session {
    /// Internal source/reference adapter, not a learner terminal command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        let raw = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if raw.len() > crate::acl_editor::MAX_TEXT {
            return Err("Local account access data exceeds 64 KiB.".into());
        }
        let mut next = self.assignment.clone();
        next.edit(raw.into())?;
        let reports = next.check(&d.initial).map_err(|e| e.to_string())?;
        self.assignment = next;
        Ok(format!(
            "Local account access checked: {}/{} requirements match",
            reports.iter().filter(|r| r.passed()).count(),
            reports.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::LocalAccess { simulation },
        crate::engine::Goal::LocalAccess { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Local account access source/state mismatch.".into());
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
    let crate::engine::State::LocalAccess { simulation } = &a.state else {
        return Err("Local account access work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible local-account attempt metadata; original retained.".into());
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
            Err("Invalid or unsupported saved local-account session.".into())
        }
    }
}
