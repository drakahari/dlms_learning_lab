//! Source-bound IPv6 policy lessons and separate bounded traffic experiments.
use crate::{
    linux_v6_connections as ct, linux_v6_filter as filter, linux_v6_flow as flow,
    linux_v6_path as path, script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_CONSOLE: usize = 128 * 1024;
fn within<T: Serialize>(v: &T, n: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|s| s.len() <= n)
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub class: Option<ct::Class>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,filter::Rule,16>")]
    pub domains: Vec<filter::Rule>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,filter::Rule,32>")]
    pub allowed: Vec<filter::Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PacketInput {
    Literal { header: path::Header },
    Reply { event: usize },
    Original { event: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Allowed,
    Filtered,
    ScopeBound,
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
    pub header: Option<path::Header>,
    pub trace: Option<flow::Trace>,
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
    pub connections: ct::Table,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: path::Facts,
    pub forwarding: bool,
    pub initial: flow::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Policy,3>")]
    pub policies: Vec<Policy>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
fn expected_text(e: Expected) -> &'static str {
    match e {
        Expected::Allowed => "Permit this header through the supplied route and filter.",
        Expected::Filtered => {
            "Drop this header in FORWARD; a missing route or disabled interface is not the required filter policy."
        }
        Expected::ScopeBound => "Keep this link-local endpoint on its own link.",
    }
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.facts.validate().map_err(|e| e.message)?;
        if !id(&self.id)
            || !within(self, 384 * 1024)
            || self.facts.links.len() != 2
            || self.policies.len() != 3
            || self.cases.is_empty()
            || self.cases.len() > 8
            || !self.initial.connections.entries.is_empty()
        {
            return Err("IPv6 tasks need a supplied two-link topology, three distinct protocol/state contexts and bounded fresh sample traffic.".into());
        }
        let names = self
            .facts
            .links
            .iter()
            .map(|l| l.name.clone())
            .collect::<Vec<_>>();
        let mut contexts = BTreeSet::new();
        for p in &self.policies {
            let key = match p.class {
                None => 0,
                Some(ct::Class::New) => 1,
                Some(ct::Class::Established) => 2,
            };
            if !contexts.insert(key) {
                return Err("Repeat IPv6 policy context.".into());
            }
            self.initial
                .filter
                .compare(
                    filter::Chain::Forward,
                    p.class,
                    &names,
                    &p.domains,
                    &p.allowed,
                )
                .map_err(|e| e.message)?;
        }
        let mut ids = BTreeSet::new();
        let mut has_allow = false;
        for c in &self.cases {
            if !id(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.events.is_empty()
                || c.events.len() > 12
            {
                return Err(
                    "IPv6 traffic tests need unique IDs, labels and bounded events.".into(),
                );
            }
            for (i, e) in c.events.iter().enumerate() {
                if e.label.trim().is_empty()
                    || !self.facts.links.iter().any(|l| l.name == e.ingress)
                {
                    return Err("Name each event and its supplied ingress interface.".into());
                }
                match &e.packet {
                    PacketInput::Literal { header } => {
                        let mut probe = self.initial.clone();
                        probe.event(&self.facts, self.forwarding, &e.ingress, header)?;
                    }
                    PacketInput::Reply { event } | PacketInput::Original { event }
                        if *event < i => {}
                    _ => {
                        return Err(
                            "Traffic references must point to an earlier event in the same test."
                                .into(),
                        );
                    }
                }
                has_allow |= e.expected == Expected::Allowed;
            }
            self.trial(&self.initial, c)?;
        }
        if !has_allow {
            return Err(
                "An IPv6 task must require permitted traffic as well as restrictions.".into(),
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
            let header = match &e.packet {
                PacketInput::Literal { header } => Some(header.clone()),
                PacketInput::Original { event } => steps.get(*event).and_then(|s| s.header.clone()),
                PacketInput::Reply { event } => steps
                    .get(*event)
                    .filter(|s| {
                        s.trace.as_ref().is_some_and(|t| {
                            t.path.gate == path::Gate::Allowed && t.tracking_refused.is_none()
                        })
                    })
                    .and_then(|s| s.header.as_ref())
                    .filter(|h| matches!(h.protocol, filter::Protocol::Tcp | filter::Protocol::Udp))
                    .map(|h| path::Header {
                        source: h.destination,
                        destination: h.source,
                        source_port: h.destination_port,
                        destination_port: h.source_port,
                        ..h.clone()
                    }),
            };
            let result=header.as_ref().ok_or_else(||"The earlier TCP/UDP request did not pass the modeled path; repair it before testing its reply.".to_string()).and_then(|h|run.event(&self.facts,self.forwarding,&e.ingress,h));
            let (trace, error, passed, actual) = match result {
                Ok(t) => {
                    let required = match e.expected {
                        Expected::Allowed => path::Gate::Allowed,
                        Expected::Filtered => path::Gate::FilterDrop,
                        Expected::ScopeBound => path::Gate::ScopeBound,
                    };
                    let passed = t.path.gate == required
                        && t.tracking_refused.is_none()
                        && (e.expected != Expected::Filtered
                            || t.path.chain == Some(filter::Chain::Forward));
                    let actual = describe(&t);
                    (Some(t), String::new(), passed, actual)
                }
                Err(error) => (
                    None,
                    error.clone(),
                    false,
                    format!("This test event could not run: {error}"),
                ),
            };
            steps.push(Step {
                header,
                trace,
                error,
                check: Check {
                    passed,
                    label: format!("{} · {}", case.label, e.label),
                    detail: format!("Expected: {} Actual: {actual}", expected_text(e.expected)),
                },
            });
        }
        Ok(Trial {
            case_id: case.id.clone(),
            steps,
            connections: run.connections,
        })
    }
    fn environment(&self, s: &flow::State) -> bool {
        s.filter.base.input == self.initial.filter.base.input
            && s.filter.input == self.initial.filter.input
            && s.filter.base.output == self.initial.filter.base.output
            && s.filter.output == self.initial.filter.output
            && s.connections == self.initial.connections
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
        let words = crate::linux_filter::words(line)?;
        let args = words.iter().map(String::as_str).collect::<Vec<_>>();
        let mut next = state.clone();
        let out = if args.first() == Some(&"ip6tables") {
            let out = next.filter.command(line)?;
            if !self.environment(&next) {
                return Err(fault(filter::Class::Unsupported,"This activity edits FORWARD only; supplied router-local INPUT/OUTPUT policies remain unchanged. No change applied.".into()));
            }
            out
        } else if args == ["sysctl", "net.ipv6.conf.all.forwarding"] {
            filter::Output {
                text: format!(
                    "net.ipv6.conf.all.forwarding = {}\n",
                    u8::from(self.forwarding)
                ),
                changed: false,
            }
        } else if args == ["ip", "-6", "route", "show"] {
            let mut text = String::new();
            for l in &self.facts.links {
                let n = filter::Cidr::parse(&format!("{}/{}", l.address, l.prefix))?;
                text.push_str(&format!(
                    "{}/{} dev {} proto kernel metric 0\n",
                    n.network, n.prefix, l.name
                ));
            }
            for r in &self.facts.routes {
                let n = if r.destination.prefix == 0 {
                    "default".into()
                } else {
                    format!("{}/{}", r.destination.network, r.destination.prefix)
                };
                text.push_str(&format!(
                    "{n}{} dev {} metric {}\n",
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
            return Err(fault(filter::Class::Unsupported,"Use the documented simulated ip6tables filter subset, ip -6 route show or the named forwarding read. No real shell, files, services or packets are used.".into()));
        };
        next.validate()
            .map_err(|s| fault(filter::Class::Context, s))?;
        *state = next;
        Ok(out)
    }
    pub fn checks(&self, scope: &Scope) -> Vec<Check> {
        if let Err(e) = scope.bound(self) {
            return vec![Check {
                passed: false,
                label: "Source context".into(),
                detail: e,
            }];
        }
        let names = self
            .facts
            .links
            .iter()
            .map(|l| l.name.clone())
            .collect::<Vec<_>>();
        let mut checks = vec![];
        for p in &self.policies {
            let label = match p.class {
                None => "FORWARD · ICMPv6 headers",
                Some(ct::Class::New) => "FORWARD · NEW TCP/UDP",
                Some(ct::Class::Established) => "FORWARD · ESTABLISHED TCP/UDP",
            }
            .into();
            let result = scope.state.filter.compare(
                filter::Chain::Forward,
                p.class,
                &names,
                &p.domains,
                &p.allowed,
            );
            let(passed,detail)=match result {
                Ok(None)=>(true,"Current rules match the required policy across the stated full IPv6 address/port/type/code ranges.".into()),
                Ok(Some(m))=>{let rule=m.rule.map_or("No rule matched; inspect the FORWARD policy or missing permits.".into(),|n|format!("Inspect FORWARD rule {n}, the first matching running rule."));(false,format!("The current policy would {} a header that must be {}. {rule} Check its interfaces, prefixes, protocol and numeric values.",if m.actual==filter::Verdict::Accept{"allow"}else{"block"},if m.expected==filter::Verdict::Accept{"allowed"}else{"blocked"}))},
                Err(e)=>(false,format!("[{:?}] {} No successful comparison accepted.",e.class,e.message)),
            };
            checks.push(Check {
                passed,
                label,
                detail,
            });
        }
        for c in &self.cases {
            match self.trial(&scope.state, c) {
                Ok(t) => checks.extend(t.steps.into_iter().map(|s| s.check)),
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
pub fn describe(t: &flow::Trace) -> String {
    let context = t
        .path
        .chain
        .map_or(String::new(), |c| format!("{}: ", c.name()));
    let state = t
        .observation
        .as_ref()
        .map_or("Stateless header".into(), |o| {
            format!(
                "{} · {}",
                if o.class == ct::Class::New {
                    "NEW"
                } else {
                    "ESTABLISHED"
                },
                if o.direction == ct::Direction::Original {
                    "original direction"
                } else {
                    "reply direction"
                }
            )
        });
    let route = t.path.route.as_ref().map_or(String::new(), |r| {
        format!(
            " Route {}/{} uses {}.",
            r.destination.network, r.destination.prefix, r.device
        )
    });
    let rule = t
        .path
        .rule
        .map_or(String::new(), |r| format!(" Running rule {r}."));
    let reason = match t.path.gate {
        path::Gate::Allowed => "The route and filter permit this header.",
        path::Gate::IngressDown => "The incoming interface is down.",
        path::Gate::NoRoute => "No supplied route matches this destination.",
        path::Gate::EgressDown => "The selected outgoing interface is down.",
        path::Gate::ForwardingDisabled => "IPv6 forwarding is off.",
        path::Gate::FilterDrop => {
            "The filter drops this header; inspect its first matching rule or chain policy."
        }
        path::Gate::ScopeBound => {
            "A link-local endpoint cannot be routed to another link; permitting it in a filter cannot change that scope."
        }
        path::Gate::TrackingRefused => {
            "The bounded connection table refused this event; that is not successful forwarding."
        }
    };
    format!(
        "{state}. {context}{reason}{route}{rule}{}",
        t.tracking_refused
            .as_deref()
            .map_or(String::new(), |e| format!(" {e}"))
    )
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
            return Err("Saved IPv6 work differs from its bounded source task; original retained for recovery.".into());
        }
        if let Some(report) = &self.last_trial
            && *report != task.trial(&self.state, &task.cases[self.selected_case])?
        {
            return Err("Saved traffic results do not match the actual current sample IPv6 test; original retained.".into());
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
                "IPv6 lessons need a stable source ID, bounded tasks and reachable worked example."
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
            return Err("Worked IPv6 actions do not reach all required outcomes.".into());
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
                "IPv6 progress belongs to another source or exceeds its bound; original retained."
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
            return Err("IPv6 session source or size is invalid.".into());
        }
        for s in [&self.assignment, &self.practice] {
            s.state.validate()?;
            if !within(s, 512 * 1024)
                || s.input.len() > 2048
                || s.output.len() > MAX_CONSOLE
                || s.selected_case >= 8
            {
                return Err("IPv6 workspace exceeds its draft/output/traffic bounds.".into());
            }
            if let Some(t) = &s.last_trial {
                if t.steps.len() > 12 || !id(&t.case_id) {
                    return Err("Invalid bounded traffic observation.".into());
                }
                t.connections.validate()?;
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
