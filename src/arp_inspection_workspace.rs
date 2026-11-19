//! Separate bounded practice/drafts and deterministic ARP policy trials.
//! Source definitions stay outside learner sessions; no command interpreter.
use crate::{
    arp_inspection as core, dhcp_snooping as dhcp,
    dhcp_snooping_workspace::{CaseReport, Name},
    mac_learning::Mac,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
pub const MAX_SESSION: usize = 3 * 1024 * 1024;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn hosts(names: &[Name]) -> BTreeSet<String> {
    names.iter().map(|n| n.0.clone()).collect()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Inspected {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,16>")]
        recipients: Vec<Name>,
    },
    Delivered {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,16>")]
        recipients: Vec<Name>,
    },
    Rejected {
        cause: Option<core::Rejection>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Dhcp {
        host: Name,
        packet: dhcp::Packet,
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,16>")]
        recipients: Vec<Name>,
        learned: bool,
        removed: bool,
    },
    Arp {
        host: Name,
        packet: core::Packet,
        expected: Expected,
    },
    Advance {
        seconds: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Step,8>")]
    pub steps: Vec<Step>,
    pub leases_after: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: Name,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    pub initial: core::World,
    pub reference: core::Config,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub text: String,
    pub error: bool,
}
fn recipients_text(names: &BTreeSet<String>) -> String {
    if names.is_empty() {
        "no endpoint".into()
    } else {
        names.iter().cloned().collect::<Vec<_>>().join(", ")
    }
}
pub fn rejection(c: core::Rejection) -> &'static str {
    match c {
        core::Rejection::MissingBinding => {
            "no current DHCP address/MAC record matches this sender in this VLAN"
        }
        core::Rejection::ExplicitAclDeny => "the address list explicitly denies this sender",
        core::Rejection::StaticAclMiss => {
            "the address list requires a matching rule and skips DHCP records"
        }
        core::Rejection::SourceMacMismatch => "Ethernet source and ARP sender MACs disagree",
        core::Rejection::DestinationMacMismatch => {
            "Ethernet destination and ARP target MACs disagree in this reply"
        }
        core::Rejection::InvalidIp => {
            "the enabled IP check found a zero, broadcast or multicast address"
        }
    }
}
pub fn describe(o: &core::Outcome) -> String {
    match o {
        core::Outcome::Unavailable { reason } => format!("Unavailable: {reason}"),
        core::Outcome::Rejected { cause } => format!("Rejected: {}", rejection(*cause)),
        core::Outcome::Forwarded {
            admission,
            forwarding,
        } => format!(
            "{}; received by {}",
            match admission {
                core::Admission::VlanNotInspected => "Not inspected in this VLAN",
                core::Admission::TrustedIngress => "Trusted incoming port bypassed ARP inspection",
                core::Admission::DhcpBinding => "Checked against a current DHCP address record",
                core::Admission::ArpAcl => "Permitted by the ARP address list",
            },
            recipients_text(&forwarding.accepted_by)
        ),
    }
}
fn expectation(e: &Expected) -> String {
    match e {
        Expected::Inspected { recipients } => format!(
            "inspected delivery to {}",
            recipients_text(&hosts(recipients))
        ),
        Expected::Delivered { recipients } => {
            format!("delivery to {}", recipients_text(&hosts(recipients)))
        }
        Expected::Rejected { cause: Some(c) } => format!("rejection because {}", rejection(*c)),
        Expected::Rejected { cause: None } => "an ARP-policy rejection".into(),
    }
}
impl Task {
    pub fn compatible(&self, w: &core::World) -> Result<()> {
        w.validate().map_err(|e| e.to_string())?;
        if w.dhcp().baseline() != self.initial.dhcp().baseline()
            || w.dhcp().config() != self.initial.dhcp().config()
        {
            return Err("Saved ARP work belongs to different equipment or supplied DHCP policy; original retained.".into());
        }
        Ok(())
    }
    pub fn trial(&self, c: &core::Config) -> Result<core::World> {
        let mut w = self.initial.clone();
        w.configure(c.clone()).map_err(|e| e.to_string())?;
        Ok(w)
    }
    pub fn reference_world(&self) -> Result<core::World> {
        self.trial(&self.reference)
    }
    pub fn validate(&self) -> Result<()> {
        self.initial.validate().map_err(|e| e.to_string())?;
        self.reference_world()?;
        if !key(&self.id.0)
            || !text(&self.instructions, 1800)
            || !(3..=8).contains(&self.cases.len())
        {
            return Err(
                "Use a stable ARP task ID, instructions and3–8 distinct traffic cases.".into(),
            );
        }
        let f = self.initial.dhcp().baseline().fabric();
        let mut labels = BTreeSet::new();
        let mut positive = false;
        let mut negative = false;
        for case in &self.cases {
            if !text(&case.label, 120)
                || !labels.insert(&case.label)
                || !(1..=8).contains(&case.steps.len())
                || case.leases_after > 64
            {
                return Err(
                    "Use unique case labels,1–8 bounded steps and a bounded final record count."
                        .into(),
                );
            }
            let mut arp = false;
            for step in &case.steps {
                let (host, targets) = match step {
                    Step::Dhcp {
                        host,
                        packet,
                        recipients,
                        ..
                    } => {
                        dhcp::Packet::from_json(
                            &serde_json::to_string(packet).map_err(|_| "Invalid DHCP metadata")?,
                        )
                        .map_err(|e| e.to_string())?;
                        (Some(host), Some(recipients))
                    }
                    Step::Arp {
                        host,
                        packet,
                        expected,
                    } => {
                        core::Packet::from_json(
                            &serde_json::to_string(packet).map_err(|_| "Invalid ARP metadata")?,
                        )
                        .map_err(|e| e.to_string())?;
                        arp = true;
                        match expected {
                            Expected::Inspected { recipients } => {
                                positive |= !recipients.is_empty();
                                (Some(host), Some(recipients))
                            }
                            Expected::Delivered { recipients } => (Some(host), Some(recipients)),
                            Expected::Rejected { .. } => {
                                negative = true;
                                (Some(host), None)
                            }
                        }
                    }
                    Step::Advance { seconds } if *seconds <= 86_400 => (None, None),
                    _ => return Err("Advance at most one simulated day per step.".into()),
                };
                if host.is_some_and(|n| !key(&n.0) || !f.hosts.contains_key(&n.0))
                    || targets.is_some_and(|ns| {
                        ns.len() > 16
                            || hosts(ns).len() != ns.len()
                            || ns.iter().any(|n| !key(&n.0) || !f.hosts.contains_key(&n.0))
                    })
                {
                    return Err("Traffic must name unique supplied virtual endpoints.".into());
                }
            }
            if !arp {
                return Err("Each ARP case needs an actual ARP request or reply.".into());
            }
        }
        if !positive || !negative || self.checks(&self.initial).iter().all(|r| r.0) {
            return Err("Keep unfinished work, real inspected delivery, rejected traffic and a reachable policy.".into());
        }
        if let Some(report) = self
            .reports(&self.reference_world()?)
            .iter()
            .find(|r| !r.passed())
        {
            return Err(format!(
                "Reference case {}: {}",
                report.label.as_deref().unwrap_or("supplied setup"),
                report.failures[0]
            ));
        }
        Ok(())
    }
    fn play(
        &self,
        index: usize,
        config: &core::Config,
    ) -> Result<(core::World, Vec<Line>, CaseReport)> {
        let case = self
            .cases
            .get(index)
            .ok_or("Choose a supplied traffic sample.")?;
        let mut w = self.trial(config)?;
        let mut lines = vec![];
        let mut failures = vec![];
        for (i, step) in case.steps.iter().enumerate() {
            let (matches, actual, expected, error) = match step {
                Step::Advance { seconds } => match w.advance(*seconds) {
                    Ok(()) => (
                        true,
                        format!("Advanced {seconds} simulated seconds."),
                        String::new(),
                        false,
                    ),
                    Err(e) => (false, e.to_string(), "a bounded clock advance".into(), true),
                },
                Step::Dhcp {
                    host,
                    packet,
                    recipients,
                    learned,
                    removed,
                } => {
                    let result = w.transmit_dhcp(&host.0, packet);
                    let matches = matches!(&result,Ok(dhcp::Outcome::Forwarded{forwarding,learned:record,removed:was_removed}) if forwarding.accepted_by==hosts(recipients) && record.is_some()==*learned && was_removed==removed);
                    let (actual, error) = match result {
                        Ok(o) => (crate::dhcp_snooping_workspace::describe(&o), false),
                        Err(e) => (
                            format!(
                                "{}: {e}",
                                crate::dhcp_snooping_workspace::error_label(e.kind)
                            ),
                            true,
                        ),
                    };
                    (
                        matches,
                        actual,
                        format!(
                            "supplied DHCP delivery to {}{}",
                            recipients_text(&hosts(recipients)),
                            if *learned {
                                " and a new address record"
                            } else {
                                ""
                            }
                        ),
                        error,
                    )
                }
                Step::Arp {
                    host,
                    packet,
                    expected,
                } => {
                    let result = w.transmit(&host.0, packet);
                    let matches = match (expected, &result) {
                        (
                            Expected::Inspected { recipients },
                            Ok(core::Outcome::Forwarded {
                                admission,
                                forwarding,
                            }),
                        ) => admission.inspected() && forwarding.accepted_by == hosts(recipients),
                        (
                            Expected::Delivered { recipients },
                            Ok(core::Outcome::Forwarded { forwarding, .. }),
                        ) => forwarding.accepted_by == hosts(recipients),
                        (
                            Expected::Rejected { cause },
                            Ok(core::Outcome::Rejected { cause: actual }),
                        ) => cause.is_none_or(|c| c == *actual),
                        _ => false,
                    };
                    let (actual, error) = match result {
                        Ok(o) => (describe(&o), false),
                        Err(e) => (
                            format!(
                                "{}: {e}",
                                crate::dhcp_snooping_workspace::error_label(e.kind)
                            ),
                            true,
                        ),
                    };
                    (matches, actual, expectation(expected), error)
                }
            };
            if !matches {
                failures.push(format!(
                    "Message {}: expected {expected}; actual {actual}",
                    i + 1
                ));
            }
            let protocol = match step {
                Step::Dhcp { packet, .. } => format!(
                    "DHCP {}",
                    crate::dhcp_snooping_workspace::message_label(packet.message)
                ),
                Step::Arp { packet, .. } => {
                    if packet.operation == core::Operation::Request {
                        "ARP request".into()
                    } else {
                        "ARP reply".into()
                    }
                }
                Step::Advance { .. } => "Sample clock".into(),
            };
            lines.push(Line {
                text: format!("{}. {protocol} · {actual}", i + 1),
                error,
            });
        }
        if w.dhcp().bindings().len() != usize::from(case.leases_after) {
            failures.push(format!("Address records: expected {}, actual {}. Review the supplied exchange, Release or expiry.",case.leases_after,w.dhcp().bindings().len()));
        }
        Ok((
            w,
            lines,
            CaseReport {
                label: Some(case.label.clone()),
                failures,
            },
        ))
    }
    pub fn reports(&self, w: &core::World) -> Vec<CaseReport> {
        if let Err(e) = self.compatible(w) {
            return vec![CaseReport {
                label: None,
                failures: vec![e],
            }];
        }
        (0..self.cases.len())
            .map(|i| match self.play(i, w.config()) {
                Ok((_, _, r)) => r,
                Err(e) => CaseReport {
                    label: Some(self.cases[i].label.clone()),
                    failures: vec![e],
                },
            })
            .collect()
    }
    pub fn checks(&self, w: &core::World) -> Vec<(bool, String)> {
        self.reports(w).iter().map(CaseReport::check).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleDraft {
    pub action: core::Action,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub ip: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub mac: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterDraft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub vlan: String,
    pub static_only: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,RuleDraft,16>")]
    pub rules: Vec<RuleDraft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub vlans: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,dhcp::PortTrust,16>")]
    pub ports: Vec<dhcp::PortTrust>,
    pub checks: core::Checks,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,FilterDraft,16>")]
    pub filters: Vec<FilterDraft>,
}
impl Draft {
    pub fn new(c: &core::Config) -> Self {
        Self {
            vlans: c
                .vlans
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            ports: c.ports.clone(),
            checks: c.checks,
            filters: c
                .filters
                .iter()
                .map(|f| FilterDraft {
                    vlan: f.vlan.to_string(),
                    static_only: f.static_only,
                    rules: f
                        .rules
                        .iter()
                        .map(|r| RuleDraft {
                            action: r.action,
                            ip: r.sender_ip.to_string(),
                            mac: r.sender_mac.text(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.vlans.len() > 128
            || self.ports.len() > 16
            || self.filters.len() > 16
            || self.ports.iter().any(|p| !key(&p.port))
            || self.filters.iter().any(|f| {
                f.vlan.len() > 64
                    || f.rules.len() > 16
                    || f.rules.iter().any(|r| r.ip.len() > 64 || r.mac.len() > 64)
            })
        {
            return Err("ARP draft exceeds its bounded fields or rule counts.".into());
        }
        Ok(())
    }
    pub fn parsed(&self) -> Result<core::Config> {
        self.validate()?;
        let mut vlans = if self.vlans.trim().is_empty() {
            vec![]
        } else {
            self.vlans
                .split(',')
                .map(|v| {
                    v.trim()
                        .parse::<u16>()
                        .map_err(|_| "Inspected VLANs need comma-separated numbers, such as10,20.")
                })
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        vlans.sort_unstable();
        let mut ports = self.ports.clone();
        ports.sort_by(|a, b| a.port.cmp(&b.port));
        let mut filters=self.filters.iter().enumerate().map(|(i,f)|{
            let vlan=f.vlan.trim().parse::<u16>().map_err(|_|format!("Address list {} needs a numeric VLAN.",i+1))?;
            let rules=f.rules.iter().enumerate().map(|(j,r)|Ok(core::Rule{action:r.action,
                sender_ip:r.ip.trim().parse().map_err(|_|format!("Address list {}, row {} needs a dotted-decimal sender IP.",i+1,j+1))?,
                sender_mac:Mac::parse(r.mac.trim()).map_err(|_|format!("Address list {}, row {} needs six hexadecimal MAC pairs separated by colons.",i+1,j+1))?,
            })).collect::<Result<Vec<_>>>()?;
            Ok(core::Filter{vlan,static_only:f.static_only,rules})
        }).collect::<Result<Vec<_>>>()?;
        filters.sort_by_key(|f| f.vlan);
        Ok(core::Config {
            vlans,
            ports,
            checks: self.checks,
            filters,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::World,
    pub draft: Draft,
    pub sample: usize,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Line,16>")]
    pub output: Vec<Line>,
    pub checked: bool,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            world: t.initial.clone(),
            draft: Draft::new(t.initial.config()),
            sample: 0,
            output: vec![],
            checked: false,
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.world.validate().map_err(|e| e.to_string())?;
        self.draft.validate()?;
        if self.sample >= 8
            || self.output.len() > 16
            || self.output.iter().any(|l| !text(&l.text, 2048))
        {
            return Err("Invalid bounded ARP sample/output.".into());
        }
        Ok(())
    }
    pub fn applied(&self) -> bool {
        self.draft.parsed().is_ok_and(|c| &c == self.world.config())
    }
    pub fn apply(&mut self) -> Result<()> {
        self.validate()?;
        let c = self.draft.parsed()?;
        self.world.configure(c).map_err(|e| e.to_string())?;
        self.checked = false;
        self.output.clear();
        Ok(())
    }
    pub fn run_sample(&mut self, t: &Task) -> Result<()> {
        self.validate()?;
        t.compatible(&self.world)?;
        if !self.applied() {
            return Err("Apply the current ARP policy before replaying.".into());
        }
        let (w, lines, _) = t.play(self.sample, self.world.config())?;
        self.world = w;
        self.output = lines;
        self.checked = false;
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<(bool, String)>> {
        self.validate()?;
        t.compatible(&self.world)?;
        if !self.applied() {
            self.checked = false;
            return Err("Apply the current ARP policy before checking.".into());
        }
        self.checked = true;
        Ok(t.checks(&self.world))
    }
    pub fn fill(&mut self, t: &Task) -> Result<()> {
        t.compatible(&self.world)?;
        self.world
            .configure(t.reference.clone())
            .map_err(|e| e.to_string())?;
        self.draft = Draft::new(self.world.config());
        self.checked = false;
        self.output.clear();
        Ok(())
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
        let a = self.initial.initial.dhcp().baseline().fabric();
        let b = self.example.initial.dhcp().baseline().fabric();
        if self.initial.id == self.example.id
            || a.hosts.keys().any(|k| b.hosts.contains_key(k))
            || a.switches.keys().any(|k| b.switches.contains_key(k))
        {
            return Err("Practice needs separate task, equipment and endpoint identities.".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err("Saved ARP work belongs to another supplied task.".into());
        }
        self.initial.compatible(&s.assignment.world)?;
        self.example.compatible(&s.practice.world)?;
        if s.assignment.sample >= self.initial.cases.len()
            || s.practice.sample >= self.example.cases.len()
        {
            return Err("Choose a supplied ARP traffic sample.".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked || !s.assignment.applied() {
            return vec![(
                false,
                "Not checked · Apply current ARP policy, then Check work.".into(),
            )];
        }
        self.initial.checks(&s.assignment.world)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("ARP task data exceeds3MiB.".into());
        }
        let d: Self =
            serde_json::from_str(raw).map_err(|_| "Invalid or unsupported ARP task data.")?;
        d.validate()?;
        Ok(d)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema_version: u8,
    pub origin: Name,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            schema_version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || !key(&self.origin.0) {
            return Err("Invalid or unsupported saved ARP session.".into());
        }
        self.assignment.validate()?;
        self.practice.validate()
    }
    pub fn from_json(raw: &str, d: &Definition) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("ARP saved work exceeds3MiB.".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|_| "Invalid, unsupported or oversized ARP saved work.")?;
        d.bound(&s)?;
        Ok(s)
    }
}

pub const PREFIX: &str = "ARP OBSERVER CONFIG\n";
pub const HELP: &str = "Use the native ARP policy and supplied traffic controls. This is a bounded in-process simulator, not IOS, a host ARP command or an endpoint IP stack.";
impl Session {
    /// Fixed internal configuration adapter for engine/reference tests, not an IOS command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        d.bound(self)?;
        let raw = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if raw.len() > 65_536 {
            return Err("ARP policy data exceeds64KiB.".into());
        }
        let config: core::Config = serde_json::from_str(raw)
            .map_err(|_| "Invalid ARP policy data; use the native controls.")?;
        let mut next = self.assignment.clone();
        next.draft = Draft::new(&config);
        next.apply()?;
        let checks = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "ARP policy checked: {}/{} cases match",
            checks.iter().filter(|r| r.0).count(),
            checks.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::ArpInspection { simulation },
        crate::engine::Goal::ArpInspection { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("ARP inspection source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id.0 != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err(
            "ARP inspection source identity, task and supplied equipment must match.".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::ArpInspection { simulation } = &a.state else {
        return Err("ARP inspection work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible ARP inspection attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}
