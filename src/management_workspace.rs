//! Source-held behavioral management-policy tasks. No saved report grants credit.
//! Native sessions/lesson integration follow only after this contract is tested.
use crate::{
    acl, acl_editor as editor, management_document as document, management_path as path,
    management_policy as policy,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt};

pub const MAX_SESSION: usize = 2 * 1024 * 1024;

fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn doc_text(s: &str) -> bool {
    s.len() <= editor::MAX_TEXT
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\r' | '\n' | '\t'))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    ReadyForAuthentication,
    ProtocolDenied,
    LegacyVersionDenied,
    SourceDenied,
}
impl Expected {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::ReadyForAuthentication => {
                "an IPv4 path and policy that permit SSH version 2 authentication"
            }
            Self::ProtocolDenied => "rejection by the line's incoming-protocol setting",
            Self::LegacyVersionDenied => {
                "rejection by the SSH version policy, with the source and SSH line still allowed"
            }
            Self::SourceDenied => {
                "rejection by the incoming source list, with SSH version 2 still available"
            }
        }
    }
    pub(crate) fn matches(self, c: &policy::Assessment) -> bool {
        let source_allowed = c
            .source_decision
            .as_ref()
            .is_none_or(|d| d.action == acl::Action::Permit);
        match self {
            Self::ReadyForAuthentication => c.allowed_to_authenticate(),
            Self::ProtocolDenied => {
                !c.protocol_allowed
                    && source_allowed
                    && c.identity_available != Some(false)
                    && c.version_allowed != Some(false)
            }
            Self::LegacyVersionDenied => {
                c.protocol_allowed
                    && source_allowed
                    && c.identity_available == Some(true)
                    && c.version_allowed == Some(false)
            }
            Self::SourceDenied => {
                c.protocol_allowed
                    && c.identity_available == Some(true)
                    && c.version_allowed == Some(true)
                    && !source_allowed
            }
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
    pub protocol: policy::Protocol,
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
    pub supplied: policy::Policy,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub reference: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug)]
pub enum Error {
    Source(String),
    Document(editor::Diagnostic),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(s) => f.write_str(s),
            Self::Document(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub label: String,
    pub matched_lines: usize,
    pub total_lines: usize,
    pub first_failure: Option<String>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.matched_lines == self.total_lines && self.first_failure.is_none()
    }
    pub fn check(&self) -> (bool, String) {
        (
            self.passed(),
            format!(
                "{}: {}/{} supplied lines match. {}",
                self.label,
                self.matched_lines,
                self.total_lines,
                self.first_failure
                    .as_deref()
                    .unwrap_or("Every required line policy matches.")
            ),
        )
    }
}
pub(crate) fn actual(c: &policy::Assessment) -> String {
    let mut facts = vec![];
    if !c.protocol_allowed {
        facts.push("the incoming protocol is disabled on this line".into());
    }
    if c.identity_available == Some(false) {
        facts.push("the supplied SSH host identity is unavailable".into());
    }
    if c.version_allowed == Some(false) {
        facts.push("the SSH version policy excludes this version".into());
    }
    if let Some(d) = &c.source_decision
        && d.action == acl::Action::Deny
    {
        facts.push(match &d.cause {
            acl::Cause::ImplicitDeny => {
                "the source list denies this client because no rule permits it".into()
            }
            acl::Cause::Matched { row, .. } => {
                format!("source-list rule {} denies this client", row + 1)
            }
        });
    }
    if facts.is_empty() {
        "the IP path and policy permit an authentication attempt".into()
    } else {
        facts.join("; ")
    }
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.id)
            || !prose(&self.instructions, 4000)
            || !doc_text(&self.starter)
            || !doc_text(&self.reference)
        {
            return Err(
                "Invalid management task version, identifier, instructions or document bounds."
                    .into(),
            );
        }
        self.network.validate().map_err(|e| e.to_string())?;
        self.supplied.validate().map_err(|e| e.to_string())?;
        if !(3..=8).contains(&self.cases.len()) {
            return Err("A management task needs 3–8 behavioral cases.".into());
        }
        let mut labels = BTreeSet::new();
        let mut inputs = BTreeSet::new();
        let mut used = BTreeSet::new();
        let mut approved = BTreeSet::new();
        let mut denied = false;
        for case in &self.cases {
            if !prose(&case.label, 120)
                || !labels.insert(&case.label)
                || !self.network.clients.iter().any(|c| c.id == case.client)
                || !inputs.insert((
                    &case.client,
                    match case.protocol {
                        policy::Protocol::Ssh1 => 1,
                        policy::Protocol::Ssh2 => 2,
                        policy::Protocol::Telnet => 3,
                    },
                ))
            {
                return Err(
                    "Management cases need distinct labels/inputs and supplied clients.".into(),
                );
            }
            used.insert(case.client.as_str());
            match case.expected {
                Expected::ReadyForAuthentication => {
                    if case.protocol != policy::Protocol::Ssh2 {
                        return Err("Approved cases in this unit must use SSH version 2.".into());
                    }
                    approved.insert(case.client.as_str());
                }
                Expected::LegacyVersionDenied => {
                    if case.protocol != policy::Protocol::Ssh1 {
                        return Err(
                            "Legacy-version cases must actually request SSH version 1.".into()
                        );
                    }
                    denied = true;
                }
                Expected::SourceDenied => {
                    if case.protocol != policy::Protocol::Ssh2 {
                        return Err(
                            "Source-list cases must retain SSH version 2 availability.".into()
                        );
                    }
                    denied = true;
                }
                Expected::ProtocolDenied => {
                    denied = true;
                }
            }
        }
        if approved.len() < 2 || !denied || used.len() != self.network.clients.len() {
            return Err("Check at least two approved clients, a prohibited behavior and every supplied client.".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let reference = self
            .reports(&self.reference)
            .map_err(|e| format!("Reference policy: {e}"))?;
        if let Some(r) = reference.iter().find(|r| !r.passed()) {
            return Err(format!(
                "Reference policy does not reach {}: {}",
                self.id,
                r.check().1
            ));
        }
        if self
            .reports(&self.starter)
            .is_ok_and(|r| r.iter().all(Report::passed))
        {
            return Err("The management task starter already completes its assignment.".into());
        }
        Ok(())
    }
    pub fn reports(&self, text: &str) -> Result<Vec<Report>, Error> {
        self.shape().map_err(Error::Source)?;
        let settings =
            document::parse(&self.network, &self.supplied, text).map_err(Error::Document)?;
        let mut reports = vec![];
        for case in &self.cases {
            let mut report = Report {
                label: case.label.clone(),
                matched_lines: 0,
                total_lines: self.supplied.lines.len(),
                first_failure: None,
            };
            for line in &self.supplied.lines {
                let mismatch = match self.network.inspect(
                    &settings,
                    &case.client,
                    line.number,
                    case.protocol,
                ) {
                    Ok(path::Observation::Policy { conditions, .. }) => {
                        if case.expected.matches(&conditions) {
                            None
                        } else {
                            Some(actual(&conditions))
                        }
                    }
                    Ok(path::Observation::PathUnavailable { reason }) => Some(format!(
                        "the IP path is unavailable: {reason}. That is not a policy rejection"
                    )),
                    Err(e) => Some(format!(
                        "this supplied trial could not be modeled: {e}. An unsupported trial is not a successful rejection"
                    )),
                };
                if let Some(mismatch) = mismatch {
                    if report.first_failure.is_none() {
                        report.first_failure = Some(format!(
                            "VTY {}: expected {}; actual {mismatch}.",
                            line.number,
                            case.expected.explanation()
                        ));
                    }
                } else {
                    report.matched_lines += 1;
                }
            }
            reports.push(report);
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
    pub fn bound(&self, session: &Session) -> Result<(), String> {
        session.bound(self)
    }
    pub fn checks(&self, session: &Session) -> Vec<(bool, String)> {
        session.checks(self)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported management lesson version.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.network == self.example.network
            || self.initial.reference == self.example.reference
        {
            return Err(
                "Management practice needs its own identity, network and worked document.".into(),
            );
        }
        Ok(())
    }
}

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
    /// Actual trial output is transient, never imported as observed execution.
    #[serde(skip)]
    pub observation: Option<String>,
}
impl Scope {
    pub fn new(task: &Task) -> Result<Self, String> {
        task.validate()?;
        Ok(Self {
            draft: task.starter.clone(),
            applied: task.starter.clone(),
            checked: false,
            selected_case: 0,
            selected_line: task.supplied.lines[0].number,
            observation: None,
        })
    }
    fn valid_shape(&self) -> bool {
        self.draft.len() <= editor::MAX_TEXT
            && self.applied.len() <= editor::MAX_TEXT
            && self.selected_case < 8
            && self.selected_line <= 98
            && self.observation.as_ref().is_none_or(|s| s.len() <= 8192)
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        task.shape()?;
        if !self.valid_shape()
            || usize::from(self.selected_case) >= task.cases.len()
            || !task
                .supplied
                .lines
                .iter()
                .any(|l| l.number == self.selected_line)
        {
            return Err(
                "Saved management draft or sample selector is outside this task's bounds.".into(),
            );
        }
        if self.applied != task.starter {
            document::parse(&task.network, &task.supplied, &self.applied)
                .map_err(|e| format!("Saved applied policy is invalid: {e}"))?;
        }
        Ok(())
    }
    pub fn invalidate(&mut self) {
        self.checked = false;
        self.observation = None;
    }
    pub fn edit(&mut self, text: String) -> Result<(), String> {
        if text.len() > editor::MAX_TEXT {
            return Err("The policy draft supports at most 64 KiB.".into());
        }
        if self.draft != text {
            self.draft = text;
            self.invalidate();
        }
        Ok(())
    }
    pub fn select_sample(&mut self, task: &Task, case: usize, line: u8) -> Result<(), String> {
        self.bound(task)?;
        if case >= task.cases.len() || !task.supplied.lines.iter().any(|l| l.number == line) {
            return Err("Select a supplied client trial and policy line.".into());
        }
        if usize::from(self.selected_case) != case || self.selected_line != line {
            self.selected_case = case as u8;
            self.selected_line = line;
            self.observation = None;
        }
        Ok(())
    }
    pub fn apply(&mut self, task: &Task) -> Result<(), Error> {
        self.invalidate();
        self.bound(task).map_err(Error::Source)?;
        document::parse(&task.network, &task.supplied, &self.draft).map_err(Error::Document)?;
        self.applied = self.draft.clone();
        Ok(())
    }
    pub fn check(&mut self, task: &Task) -> Result<Vec<Report>, Error> {
        self.apply(task)?;
        let reports = task.reports(&self.applied)?;
        self.checked = true;
        Ok(reports)
    }
    pub fn checks(&self, task: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(task) {
            return vec![(false, e)];
        }
        if !self.checked || self.draft != self.applied {
            return vec![(
                false,
                "Check the current policy document to evaluate every required client and line."
                    .into(),
            )];
        }
        match task.reports(&self.applied) {
            Ok(reports) => reports.iter().map(Report::check).collect(),
            Err(e) => vec![(false, e.to_string())],
        }
    }
    pub fn inspect(&mut self, task: &Task) -> Result<&str, String> {
        self.bound(task)?;
        self.observation = None;
        if self.draft != self.applied {
            return Err("Apply the edited policy before inspecting its effect.".into());
        }
        let settings = document::parse(&task.network, &task.supplied, &self.applied)
            .map_err(|e| e.to_string())?;
        let case = &task.cases[usize::from(self.selected_case)];
        let observed = task
            .network
            .inspect(&settings, &case.client, self.selected_line, case.protocol)
            .map_err(|e| e.to_string())?;
        let output = match observed {
            path::Observation::PathUnavailable { reason } => {
                format!("IP path unavailable: {reason}\nNo line-policy rejection is inferred.")
            }
            path::Observation::Policy { path, conditions } => format!(
                "Client source: {}\nManagement endpoint: {}/{}\nIPv4 forward path: {} hops to {}\nIPv4 reply path: {} hops to {}\nVTY {}: {}.",
                path.source,
                task.network.management.device,
                task.network.management.interface,
                path.forward.hops.len(),
                path.forward.destination_device,
                path.reply.hops.len(),
                path.reply.destination_device,
                self.selected_line,
                actual(&conditions)
            ),
        };
        if output.len() > 8192 {
            return Err("The trial observation exceeds its output bound.".into());
        }
        self.observation = Some(output);
        Ok(self.observation.as_deref().expect("output was stored"))
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
    pub fn validate(&self) -> Result<(), String> {
        if self.valid_shape() {
            Ok(())
        } else {
            Err("Invalid or unsupported saved management session.".into())
        }
    }
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
                "Saved management work belongs to another lesson or unsupported version.".into(),
            );
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn from_json(d: &Definition, raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved management work exceeds 2 MiB.".into());
        }
        let value: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported management work.".to_string())?;
        value.bound(d)?;
        Ok(value)
    }
    /// Populate practice only. The learner still chooses Apply/Check/Inspect.
    pub fn fill_example(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.practice.edit(d.example.reference.clone())
    }
    pub fn checks(&self, d: &Definition) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(d) {
            return vec![(false, e)];
        }
        self.assignment.checks(&d.initial)
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

pub const PREFIX: &str = "MANAGEMENT POLICY\n";
pub const HELP: &str = "Use the native management policy document and supplied client trials. This bounded simulator checks IPv4 paths and VTY policy before authentication; it does not execute IOS, SSH, Telnet, host commands or cryptographic exchanges.";
impl Session {
    /// Internal source/reference adapter, not a learner terminal command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        let raw = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if raw.len() > editor::MAX_TEXT {
            return Err("Management policy data exceeds 64 KiB.".into());
        }
        let mut next = self.assignment.clone();
        next.edit(raw.into())?;
        let reports = next.check(&d.initial).map_err(|e| e.to_string())?;
        self.assignment = next;
        Ok(format!(
            "Management policy checked: {}/{} behavioral cases match",
            reports.iter().filter(|r| r.passed()).count(),
            reports.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::ManagementPolicy { simulation },
        crate::engine::Goal::ManagementPolicy { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Management policy source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Management source identity, task and supplied context must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::ManagementPolicy { simulation } = &a.state else {
        return Err("Management policy work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible management attempt metadata; original retained.".into());
    }
    simulation.bound(d)
}
