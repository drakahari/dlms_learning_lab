//! Source-bound connection/NAT assignments and independent traffic trials.
use crate::{
    linux_connection_flow::{self as flow, Gate, Trace},
    linux_connections::{self as ct, Class, Endpoint, Transport, Tuple},
    linux_filter::{self as filter, Chain, Protocol, Rule, Verdict},
    linux_filter_policy,
    linux_nat_policy::{self as nat_policy, Requirement},
    linux_router_path::Facts,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_CONSOLE: usize = 128 * 1024;
fn within<T: Serialize>(value: &T, max: usize) -> bool {
    serde_json::to_vec(value).is_ok_and(|s| s.len() <= max)
}
fn id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub class: Class,
    pub dnat: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,16>")]
    pub domains: Vec<Rule>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,32>")]
    pub allowed: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NatPolicy {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub transport: Transport,
    pub requirement: Requirement,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,16>")]
    pub domains: Vec<Rule>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,32>")]
    pub expected: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PacketInput {
    Literal { tuple: Tuple },
    Reply { event: usize },
    Original { event: usize },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointMatch {
    pub address: std::net::Ipv4Addr,
    pub port: Option<u16>,
}
impl EndpointMatch {
    fn matches(&self, e: Endpoint) -> bool {
        self.address == e.address && self.port.is_none_or(|p| p == e.port)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Filtered,
    NotForwarded,
    Outgoing {
        source: EndpointMatch,
        destination: EndpointMatch,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub ingress: String,
    pub packet: PacketInput,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,12>")]
    pub events: Vec<Event>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub passed: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub trace: Option<Trace>,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub error: String,
    pub check: Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub case_id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Step,12>")]
    pub steps: Vec<Step>,
    pub mappings: ct::Table,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    pub forwarding: bool,
    pub initial: flow::State,
    pub preserve_prerouting: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Policy,4>")]
    pub policies: Vec<Policy>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,NatPolicy,8>")]
    pub nat_policies: Vec<NatPolicy>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
fn endpoint(e: &EndpointMatch) -> String {
    format!(
        "{}:{}",
        e.address,
        e.port
            .map_or("any available port".into(), |p| p.to_string())
    )
}
fn expected_text(e: &Expected) -> String {
    match e {
        Expected::Filtered => "The FORWARD filter must refuse this event.".into(),
        Expected::NotForwarded => "The router or filter must leave this endpoint unpublished; an unavailable route or port pool alone is not the required policy.".into(),
        Expected::Outgoing {
            source,
            destination,
        } => format!(
            "Outgoing source {}; destination {}.",
            endpoint(source),
            endpoint(destination)
        ),
    }
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.facts.validate().map_err(|e| e.message)?;
        if !id(&self.id)
            || !within(self, 384 * 1024)
            || self.facts.links.len() != 2
            || self.policies.len() != 4
            || self.nat_policies.is_empty()
            || self.nat_policies.len() > 8
            || self.cases.is_empty()
            || self.cases.len() > 8
            || !self.initial.connections.entries.is_empty()
        {
            return Err("Flow tasks need a supplied two-interface topology, bounded distinct policy contexts, translation requirements and fresh sample flows.".into());
        }
        let mut contexts = BTreeSet::new();
        for p in &self.policies {
            let key = (p.class == Class::Established, p.dnat);
            if !contexts.insert(key)
                || p.domains.is_empty()
                || p.domains
                    .iter()
                    .chain(&p.allowed)
                    .any(|r| !matches!(r.protocol, Some(Protocol::Tcp | Protocol::Udp)))
            {
                return Err("Declare all four distinct normal-flow/translation policy contexts with TCP/UDP domains.".into());
            }
            linux_filter_policy::compare_filter(
                &self
                    .initial
                    .filter
                    .specialize_context(p.class, p.dnat)
                    .map_err(|e| e.message)?,
                Chain::Forward,
                &self.facts,
                &p.domains,
                &p.allowed,
            )
            .map_err(|e| e.message)?;
        }
        for p in &self.nat_policies {
            if p.label.trim().is_empty() {
                return Err("Name each translation requirement.".into());
            }
            nat_policy::compare(
                &self.initial.nat,
                &self.facts,
                p.transport,
                p.requirement,
                &p.domains,
                &p.expected,
            )
            .map_err(|e| e.message)?;
        }
        let mut ids = BTreeSet::new();
        let mut has_request = false;
        for c in &self.cases {
            if !id(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.events.is_empty()
                || c.events.len() > 12
            {
                return Err(
                    "Traffic trials need stable unique IDs, labels and at least one event.".into(),
                );
            }
            for (i, e) in c.events.iter().enumerate() {
                if e.label.trim().is_empty()
                    || !self.facts.links.iter().any(|l| l.name == e.ingress)
                {
                    return Err(
                        "Each traffic event needs a label and supplied incoming interface.".into(),
                    );
                }
                match e.packet{PacketInput::Literal{tuple}if tuple.valid()=>{},PacketInput::Reply{event}|PacketInput::Original{event}if event<i=>{},_=>return Err("Traffic packets must be supported tuples or references to earlier trial events.".into())}
                if matches!(e.expected, Expected::Outgoing { .. }) {
                    has_request = true;
                }
            }
            self.trial(&self.initial, c)?;
        }
        if !has_request {
            return Err(
                "A flow assignment must require actual forwarding, not only refusals.".into(),
            );
        }
        Ok(())
    }
    pub fn trial(&self, state: &flow::State, case: &Case) -> Result<Trial, String> {
        state.validate()?;
        let mut run = state.clone();
        run.connections = self.initial.connections.clone();
        let mut steps: Vec<Step> = vec![];
        for e in &case.events {
            let packet = match e.packet {
                PacketInput::Literal { tuple } => Some(tuple),
                PacketInput::Reply { event } => steps
                    .get(event)
                    .and_then(|s| s.trace.as_ref())
                    .and_then(|t| t.outgoing)
                    .map(Tuple::reverse),
                PacketInput::Original { event } => steps
                    .get(event)
                    .and_then(|s| s.trace.as_ref())
                    .map(|t| t.incoming),
            };
            let result=packet.ok_or_else(||"The earlier request did not leave the router, so there is no reply tuple to test. Fix that request first.".to_string()).and_then(|p|run.event(&self.facts,self.forwarding,&e.ingress,p));
            let (trace, error, passed, actual) = match result {
                Ok(t) => {
                    let passed = match &e.expected {
                        Expected::Filtered => t.gate == Gate::FilterDrop,
                        Expected::NotForwarded => {
                            matches!(t.gate, Gate::RouterLocal | Gate::FilterDrop)
                        }
                        Expected::Outgoing {
                            source,
                            destination,
                        } => {
                            t.gate == Gate::Forwarded
                                && t.outgoing.is_some_and(|p| {
                                    source.matches(p.source) && destination.matches(p.destination)
                                })
                        }
                    };
                    let actual = if t.gate == Gate::RouterLocal {
                        "Actual: the destination remains local to the router; no forwarded server path was created.".into()
                    } else if let Some(out) = t.outgoing {
                        format!(
                            "Actual outgoing {}:{} -> {}:{}. {}",
                            out.source.address,
                            out.source.port,
                            out.destination.address,
                            out.destination.port,
                            t.detail
                        )
                    } else {
                        format!("Actual: {}", t.detail)
                    };
                    (Some(t), String::new(), passed, actual)
                }
                Err(error) => (
                    None,
                    error.clone(),
                    false,
                    format!("This event could not run: {error}"),
                ),
            };
            steps.push(Step {
                trace,
                error,
                check: Check {
                    passed,
                    label: format!("{} · {}", case.label, e.label),
                    detail: format!("Expected: {} {actual}", expected_text(&e.expected)),
                },
            });
        }
        Ok(Trial {
            case_id: case.id.clone(),
            steps,
            mappings: run.connections,
        })
    }
    fn environment(&self, state: &flow::State) -> bool {
        state.filter.base.input == self.initial.filter.base.input
            && state.filter.input == self.initial.filter.input
            && state.filter.base.output == self.initial.filter.base.output
            && state.filter.output == self.initial.filter.output
            && state.connections == self.initial.connections
    }
    pub fn command(
        &self,
        state: &mut flow::State,
        line: &str,
    ) -> Result<filter::Output, filter::Fault> {
        let fault = |class, message: String| filter::Fault { class, message };
        state
            .validate()
            .map_err(|s| fault(filter::Class::Context, s))?;
        let words = filter::words(line)?;
        let mut next = state.clone();
        let output = if words.first().is_some_and(|s| s == "iptables") {
            let nat = words
                .windows(2)
                .any(|w| matches!(w[0].as_str(), "-t" | "--table") && w[1] == "nat");
            let out = if nat {
                next.nat.command(line)?
            } else {
                next.filter.command(line)?
            };
            if !self.environment(&next) {
                return Err(fault(filter::Class::Unsupported,"This activity edits FORWARD/NAT only; router-local INPUT/OUTPUT settings stay as supplied. No change was applied.".into()));
            }
            out
        } else if words
            .iter()
            .map(String::as_str)
            .eq(["sysctl", "net.ipv4.ip_forward"])
        {
            filter::Output {
                text: format!("net.ipv4.ip_forward = {}\n", u8::from(self.forwarding)),
                changed: false,
            }
        } else if words
            .iter()
            .map(String::as_str)
            .eq(["ip", "-4", "route", "show"])
            || words.iter().map(String::as_str).eq(["ip", "route", "show"])
        {
            let mut text = String::new();
            for l in &self.facts.links {
                let net = filter::Cidr::parse(&format!("{}/{}", l.address, l.prefix))?;
                text.push_str(&format!(
                    "{}/{} dev {} proto kernel scope link src {}\n",
                    net.network, net.prefix, l.name, l.address
                ));
            }
            for r in &self.facts.routes {
                let network = if r.destination.prefix == 0 {
                    "default".into()
                } else {
                    format!("{}/{}", r.destination.network, r.destination.prefix)
                };
                text.push_str(&format!(
                    "{network}{} dev {} metric {}\n",
                    r.via.map_or(String::new(), |v| format!(" via {v}")),
                    r.device,
                    r.metric
                ));
            }
            filter::Output {
                text,
                changed: false,
            }
        } else {
            return Err(fault(filter::Class::Unsupported,"Use the documented simulated iptables filter/NAT subset, ip -4 route show, or the named forwarding read. No shell, host utility, file or network command ran.".into()));
        };
        next.validate()
            .map_err(|s| fault(filter::Class::Context, s))?;
        *state = next;
        Ok(output)
    }
}
impl Task {
    pub fn checks(&self, scope: &Scope) -> Vec<Check> {
        if let Err(e) = scope.bound(self) {
            return vec![Check {
                passed: false,
                label: "Source context".into(),
                detail: e,
            }];
        }
        let mut checks = vec![];
        if self.preserve_prerouting {
            checks.push(Check{passed:scope.state.nat.prerouting==self.initial.nat.prerouting,label:"Keep destination rules unchanged".into(),detail:"This outbound task changes source translation; leave the supplied PREROUTING configuration unchanged.".into()});
        }
        for p in &self.policies {
            let label = format!(
                "FORWARD · {} · {}",
                if p.class == Class::New {
                    "NEW"
                } else {
                    "ESTABLISHED"
                },
                if p.dnat {
                    "destination translated"
                } else {
                    "destination unchanged"
                }
            );
            let result = scope
                .state
                .filter
                .specialize_context(p.class, p.dnat)
                .and_then(|t| {
                    linux_filter_policy::compare_filter(
                        &t,
                        Chain::Forward,
                        &self.facts,
                        &p.domains,
                        &p.allowed,
                    )
                })
                .and_then(|mut result| {
                    if let Some(mismatch) = &mut result {
                        // Comparison omits inactive state rows. Recheck the same
                        // witness against the live table to name its real row.
                        let live = scope.state.filter.evaluate_context(
                            Chain::Forward,
                            &mismatch.packet,
                            p.class,
                            p.dnat,
                        )?;
                        if live.verdict != mismatch.actual {
                            return Err(filter::Fault {
                                class: filter::Class::Context,
                                message: "State-policy witness disagrees with the live rule table; no comparison accepted.".into(),
                            });
                        }
                        mismatch.rule = live.rule;
                    }
                    Ok(result)
                });
            let(passed,detail)=match result{
    Ok(None)=>(true,"Current rules match the required policy across the stated address and port ranges.".into()),
    Ok(Some(m))=>{let reason=m.rule.map_or("No rule matched; review the FORWARD policy and missing permits.".into(),|n|format!("Inspect FORWARD rule {n}, the first matching rule: compare its state list, interfaces, addresses and ports."));(false,format!("This policy would {} traffic that must be {}. {reason}",if m.actual==Verdict::Accept{"allow"}else{"block"},if m.expected==Verdict::Accept{"allowed"}else{"blocked"}))},
    Err(e)=>(false,format!("[{:?}] {} No successful comparison was accepted.",e.class,e.message)),
   };
            checks.push(Check {
                passed,
                label,
                detail,
            });
        }
        for p in &self.nat_policies {
            let result = nat_policy::compare(
                &scope.state.nat,
                &self.facts,
                p.transport,
                p.requirement,
                &p.domains,
                &p.expected,
            );
            let (passed, detail) = match result {
                Ok(None) => (
                    true,
                    "The translation matches this requirement across its whole stated range."
                        .into(),
                ),
                Ok(Some(m)) => {
                    let rule =
                        m.rule
                            .map_or("No NAT rule matched this part of the range.".into(), |n| {
                                format!(
                                    "Inspect NAT rule {n} in {}.",
                                    if matches!(p.requirement, Requirement::SourceAddress(_)) {
                                        "POSTROUTING"
                                    } else {
                                        "PREROUTING"
                                    }
                                )
                            });
                    let reason = match p.requirement {
                        Requirement::SourceAddress(_) => {
                            "The required public source address is missing for part of the stated range; check the source prefix, outgoing interface and target."
                        }
                        Requirement::ChangedToDestination(_) => {
                            if m.actual == Verdict::Accept {
                                "This would publish traffic beyond the stated public address/port."
                            } else {
                                "Part of the required public endpoint does not translate to the required server address and port."
                            }
                        }
                        Requirement::DestinationOrIdentity(_) => {
                            "This rule changes another destination instead of leaving it unchanged or sending it to the stated server."
                        }
                    };
                    (false, format!("{reason} {rule}"))
                }
                Err(e) => (
                    false,
                    format!(
                        "[{:?}] {} No successful translation proof was accepted.",
                        e.class, e.message
                    ),
                ),
            };
            checks.push(Check {
                passed,
                label: p.label.clone(),
                detail,
            });
        }
        for c in &self.cases {
            match self.trial(&scope.state, c) {
                Ok(trial) => checks.extend(trial.steps.into_iter().map(|s| s.check)),
                Err(e) => checks.push(Check {
                    passed: false,
                    label: c.label.clone(),
                    detail: e,
                }),
            }
        }
        checks
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub state: flow::State,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,131072>")]
    pub output: String,
    pub selected_case: usize,
    pub last_trial: Option<Trial>,
    pub checked: bool,
}
impl Scope {
    pub fn new(task: &Task) -> Result<Self, String> {
        task.validate()?;
        Ok(Self {
            state: task.initial.clone(),
            input: String::new(),
            output: String::new(),
            selected_case: 0,
            last_trial: None,
            checked: false,
        })
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.state.validate()?;
        if !within(self, 512 * 1024)
            || self.input.len() > 2048
            || self.output.len() > MAX_CONSOLE
            || self.selected_case >= task.cases.len()
            || !task.environment(&self.state)
        {
            return Err("Saved flow work differs from its bounded source task; original retained for recovery.".into());
        }
        if let Some(report) = &self.last_trial
            && *report != task.trial(&self.state, &task.cases[self.selected_case])?
        {
            return Err("Saved traffic results do not match the actual current sample flow; original retained.".into());
        }
        Ok(())
    }
    fn log(&mut self, text: &str) {
        self.output.push_str(text);
        if self.output.len() > MAX_CONSOLE {
            let mut n = self.output.len() - MAX_CONSOLE;
            while !self.output.is_char_boundary(n) {
                n += 1;
            }
            self.output.drain(..n);
        }
    }
    pub fn reference(&mut self, task: &Task, action: &Action) -> Result<(), String> {
        if let Action::Command { line } = action {
            let mut probe = self.state.clone();
            task.command(&mut probe, line).map_err(|e| e.message)?;
        }
        self.apply(task, action)
    }
    pub fn apply(&mut self, task: &Task, action: &Action) -> Result<(), String> {
        self.bound(task)?;
        match action {
            Action::Input { text } => {
                if text.len() > 2048 {
                    return Err("Keep one command within2KiB; previous draft retained.".into());
                }
                self.input = text.clone();
                self.checked = false;
                self.last_trial = None;
            }
            Action::Command { line } => {
                if line.len() > 2048 {
                    return Err("Keep one command within2KiB; previous work retained.".into());
                }
                self.checked = false;
                self.last_trial = None;
                self.log(&format!("$ {line}\n"));
                match task.command(&mut self.state, line) {
                    Ok(out) => {
                        self.log(&out.text);
                        self.input.clear();
                    }
                    Err(e) => {
                        self.log(&format!("[{:?}] {}\n", e.class, e.message));
                        self.input = line.clone();
                    }
                }
            }
            Action::SelectCase { index } => {
                if *index >= task.cases.len() {
                    return Err("Choose one of this lesson's supplied traffic tests.".into());
                }
                self.selected_case = *index;
                self.last_trial = None;
            }
            Action::RunTraffic => {
                // A separate read-only trial cannot break an already checked configuration.
                self.last_trial = Some(task.trial(&self.state, &task.cases[self.selected_case])?);
            }
            Action::Check => self.checked = true,
        }
        self.bound(task)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Input {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        line: String,
    },
    SelectCase {
        index: usize,
    },
    RunTraffic,
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,32>")]
    pub worked: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if !id(&self.source_id)
            || !within(self, 768 * 1024)
            || self.worked.is_empty()
            || self.worked.len() > 32
        {
            return Err(
                "Flow lessons need a stable source ID, bounded tasks and reachable worked example."
                    .into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.id == self.example.id
            || self.assignment.facts == self.example.facts
            || self.assignment.cases == self.example.cases
        {
            return Err(
                "Worked example values and traffic must be separate from the assignment.".into(),
            );
        }
        let mut example = Scope::new(&self.example)?;
        for action in &self.worked {
            example.reference(&self.example, action)?;
        }
        if !example.checked || !self.example.checks(&example).iter().all(|c| c.passed) {
            return Err("Worked flow actions do not reach all required outcomes.".into());
        }
        if self
            .assignment
            .checks(&Scope::new(&self.assignment)?)
            .iter()
            .all(|c| c.passed)
        {
            return Err(
                "The assignment needs a genuine repair, not only an unchecked result.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        if s.source_id != self.source_id || !within(s, 1024 * 1024) {
            return Err(
                "Flow progress belongs to another source or exceeds its bound; original retained."
                    .into(),
            );
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.assignment.checked
            && self
                .assignment
                .checks(&s.assignment)
                .iter()
                .all(|c| c.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn shape(&self) -> Result<(), String> {
        if !id(&self.source_id) || !within(self, 1024 * 1024) {
            return Err("Flow session source or size is invalid.".into());
        }
        for s in [&self.assignment, &self.practice] {
            s.state.validate()?;
            if !within(s, 512 * 1024)
                || s.input.len() > 2048
                || s.output.len() > MAX_CONSOLE
                || s.selected_case >= 8
            {
                return Err("Flow workspace exceeds its draft/output/traffic bounds.".into());
            }
            if let Some(t) = &s.last_trial {
                if t.steps.len() > 12 || !id(&t.case_id) {
                    return Err("Invalid bounded traffic observation.".into());
                }
                t.mappings.validate()?;
            }
        }
        Ok(())
    }
    pub fn action(
        &mut self,
        d: &Definition,
        action: &Action,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        let (task, scope) = if practice {
            (&d.example, &mut next.practice)
        } else {
            (&d.assignment, &mut next.assignment)
        };
        scope.apply(task, action)?;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        if practice {
            self.practice = Scope::new(&d.example)?;
        } else {
            self.assignment = Scope::new(&d.assignment)?;
        }
        Ok(())
    }
}
