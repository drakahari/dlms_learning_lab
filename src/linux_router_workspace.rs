//! Source-bound router lessons. Commands and file contents never leave typed simulation.
use crate::{
    linux_filter::{Chain, Rule, Verdict},
    linux_filter_policy, linux_router_admin as admin, linux_router_path as path,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
pub const MAX_CONSOLE: usize = 128 * 1024;
fn within<T: Serialize>(v: &T, n: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|v| v.len() <= n)
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum File {
    Filter,
    Forwarding,
}
impl File {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Filter => admin::FILTER_FILE,
            Self::Forwarding => admin::FORWARD_FILE,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub chain: Chain,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,16>")]
    pub domains: Vec<Rule>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,32>")]
    pub allowed: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub origin: path::Origin,
    pub header: path::Header,
    pub allow: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub initial: admin::State,
    pub files: bool,
    pub persistence: bool,
    pub forwarding: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Policy,3>")]
    pub policies: Vec<Policy>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,16>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.message)?;
        if !within(self, 384 * 1024)
            || self.policies.is_empty()
            || self.policies.len() > 3
            || self.cases.is_empty()
            || self.cases.len() > 16
            || !self.cases.iter().any(|c| c.allow)
            || self.persistence && !self.files
        {
            return Err("Router tasks need bounded policies and meaningful supplied traffic, with editors for persistence.".into());
        }
        let facts = self.initial.facts().map_err(|e| e.message)?;
        let mut chains = vec![];
        for p in &self.policies {
            if chains.contains(&p.chain) {
                return Err("Repeat policy chain in router task".into());
            }
            chains.push(p.chain);
            linux_filter_policy::compare_filter(
                &self.initial.filter,
                p.chain,
                &facts,
                &p.domains,
                &p.allowed,
            )
            .map_err(|e| e.message)?;
        }
        let mut ids = std::collections::BTreeSet::new();
        for c in &self.cases {
            if !id(&c.id) || !ids.insert(&c.id) || c.label.trim().is_empty() {
                return Err(
                    "Traffic cases need unique stable identifiers and readable labels.".into(),
                );
            }
            self.initial
                .evaluate(&c.origin, &c.header)
                .map_err(|e| e.message)?;
        }
        Ok(())
    }
    fn environment(&self, state: &admin::State) -> bool {
        state.host.same_environment(&self.initial.host)
            && state.host.configuration.nameservers == self.initial.host.configuration.nameservers
            && state.host.configuration.interfaces.len()
                == self.initial.host.configuration.interfaces.len()
            && state.host.configuration.interfaces.iter().all(|i| {
                self.initial
                    .host
                    .configuration
                    .interfaces
                    .iter()
                    .any(|old| old.name == i.name && old.address == i.address)
            })
    }
    fn state_checks(&self, state: &admin::State, prefix: &str) -> Vec<Check> {
        let mut out = vec![];
        if let Err(e) = state.validate() {
            return vec![Check {
                passed: false,
                label: format!("{prefix}Router state"),
                detail: e.message,
            }];
        }
        out.push(Check {
            passed: state.forwarding == self.forwarding,
            label: format!("{prefix}IPv4 forwarding"),
            detail: format!(
                "Forwarding is {}; this task requires it {}.",
                if state.forwarding { "on" } else { "off" },
                if self.forwarding { "on" } else { "off" }
            ),
        });
        let facts = match state.facts() {
            Ok(f) => f,
            Err(e) => {
                return vec![Check {
                    passed: false,
                    label: format!("{prefix}Router interfaces"),
                    detail: e.message,
                }];
            }
        };
        for p in &self.policies {
            let result = linux_filter_policy::compare_filter(
                &state.filter,
                p.chain,
                &facts,
                &p.domains,
                &p.allowed,
            );
            let (passed, detail) = match result {
                Ok(None) => (true, "The current rules match the required policy across this task's modeled header domains.".into()),
                Ok(Some(m)) => {
                    let reason=if let Some(n)=m.rule {
                        format!("Inspect {} rule {n}: it is the first matching rule. Check its interfaces, addresses, protocol and ports against the requirements.",p.chain.name())
                    }else{
                        format!("No rule matched in {}; its {} policy made this decision. Review the chain policy and the required allow rules.",p.chain.name(),if m.actual==Verdict::Accept{"ACCEPT"}else{"DROP"})
                    };
                    (false,format!("The filter would {} traffic here, but the task requires it {}. {reason}",verdict_text(m.actual),if m.expected==Verdict::Accept{"allowed"}else{"blocked"}))
                },
                Err(e) => (false,format!("[{:?}] {} No successful comparison was accepted.",e.class,e.message)),
            };
            out.push(Check {
                passed,
                label: format!("{prefix}{} policy", p.chain.name()),
                detail,
            });
        }
        for c in &self.cases {
            let result = state.evaluate(&c.origin, &c.header);
            let (passed, detail) = match result {
                Ok(trace) => {
                    let actual = trace.gate == path::Gate::Allowed;
                    let reason = trace_detail(state, &trace, &c.header);
                    (
                        actual == c.allow,
                        format!(
                            "Expected {}; actual {}. {reason}",
                            if c.allow { "allowed" } else { "blocked" },
                            if actual { "allowed" } else { "blocked" }
                        ),
                    )
                }
                Err(e) => (false, format!("[{:?}] {}", e.class, e.message)),
            };
            out.push(Check {
                passed,
                label: format!("{prefix}{}", c.label),
                detail,
            });
        }
        out
    }
    pub fn checks(&self, scope: &Scope) -> Vec<Check> {
        if let Err(e) = scope.bound(self) {
            return vec![Check {
                passed: false,
                label: "Source context".into(),
                detail: e,
            }];
        }
        let mut checks = self.state_checks(&scope.state, "");
        if self.files {
            checks.push(Check{passed:scope.filter_draft==scope.state.filter_file&&scope.forward_draft==scope.state.forward_file,label:"Save current editor changes".into(),detail:"Each editor draft must be saved to its selected simulated file before checking.".into()});
        }
        if self.persistence {
            let mut replay = scope.state.clone();
            replay.filter = crate::linux_filter::Table::new(Verdict::Accept);
            replay.forwarding = false;
            let applied = replay
                .command(&format!("sysctl -p {}", admin::FORWARD_FILE))
                .and_then(|_| replay.command(&format!("iptables-restore {}", admin::FILTER_FILE)));
            match applied{Ok(_)=>checks.extend(self.state_checks(&replay,"Reloaded setup: ")),Err(e)=>checks.push(Check{passed:false,label:"Reload saved setup".into(),detail:format!("[{:?}] {} The saved setup could not be reapplied; current live state was retained.",e.class,e.message)})}
        }
        checks
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub state: admin::State,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,131072>")]
    pub output: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub filter_draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub forward_draft: String,
    pub file: File,
    pub selected_case: usize,
    pub checked: bool,
}
impl Scope {
    pub fn new(task: &Task) -> Result<Self, String> {
        task.validate()?;
        Ok(Self {
            state: task.initial.clone(),
            input: String::new(),
            output: String::new(),
            filter_draft: task.initial.filter_file.clone(),
            forward_draft: task.initial.forward_file.clone(),
            file: File::Filter,
            selected_case: 0,
            checked: false,
        })
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.state.validate().map_err(|e| e.message)?;
        if !within(self, 512 * 1024)
            || self.input.len() > 2048
            || self.output.len() > MAX_CONSOLE
            || self.filter_draft.len() > 32768
            || self.forward_draft.len() > 4096
            || self.selected_case >= task.cases.len()
            || !task.environment(&self.state)
        {
            return Err("Saved router work does not match this bounded source task; it was not silently replaced.".into());
        }
        Ok(())
    }
    pub fn draft(&self) -> &str {
        match self.file {
            File::Filter => &self.filter_draft,
            File::Forwarding => &self.forward_draft,
        }
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
            probe.command(line).map_err(|e| e.message)?;
        }
        self.apply(task, action)
    }
    pub fn apply(&mut self, task: &Task, action: &Action) -> Result<(), String> {
        self.bound(task)?;
        match action {
            Action::Input { text } => {
                if text.len() > 2048 {
                    return Err("Keep one command within2KiB; previous input retained.".into());
                }
                self.input = text.clone();
                self.checked = false;
            }
            Action::Command { line } => {
                if line.len() > 2048 {
                    return Err("Keep one command within 2 KiB; previous work retained.".into());
                }
                self.checked = false;
                self.log(&format!("$ {line}\n"));
                let old_filter = self.state.filter_file.clone();
                let old_forward = self.state.forward_file.clone();
                let dirty_filter = self.filter_draft != old_filter;
                let dirty_forward = self.forward_draft != old_forward;
                match self.state.command(line) {
                    Ok(out) => {
                        self.log(&out.text);
                        self.input.clear();
                        if self.state.filter_file != old_filter && !dirty_filter {
                            self.filter_draft = self.state.filter_file.clone();
                        }
                        if self.state.forward_file != old_forward && !dirty_forward {
                            self.forward_draft = self.state.forward_file.clone();
                        }
                        if self.state.filter_file != old_filter && dirty_filter
                            || self.state.forward_file != old_forward && dirty_forward
                        {
                            self.log("[Simulator] Saved file changed; your unsaved editor draft was retained. Reload editor only if you want to replace it.\n");
                        }
                    }
                    Err(e) => {
                        self.log(&format!("[{:?}] {}\n", e.class, e.message));
                        self.input = line.clone();
                    }
                }
            }
            Action::File { file } => {
                if !task.files {
                    return Err("This task uses the terminal workspace; saved-file editors belong to the persistence activity.".into());
                }
                self.file = file.clone();
            }
            Action::Edit { text } => {
                if !task.files {
                    return Err("This task does not expose configuration editors.".into());
                }
                let max = if self.file == File::Filter {
                    32768
                } else {
                    4096
                };
                if text.len() > max {
                    return Err(
                        "Draft exceeds its selected file limit; previous text retained.".into(),
                    );
                }
                if self.file == File::Filter {
                    self.filter_draft = text.clone();
                } else {
                    self.forward_draft = text.clone();
                }
                self.checked = false;
            }
            Action::Save => {
                if !task.files {
                    return Err("No file editor is active in this task.".into());
                }
                self.checked = false;
                let text = self.draft().to_string();
                match self.state.write_file(self.file.path(),&text){Ok(())=>self.log("[Simulator] Editor draft saved to the selected virtual file; live settings were not applied.\n"),Err(e)=>self.log(&format!("[{:?}] {}\n",e.class,e.message))}
            }
            Action::ReloadDraft => {
                if !task.files {
                    return Err("No file editor is active in this task.".into());
                }
                if self.file == File::Filter {
                    self.filter_draft = self.state.filter_file.clone();
                } else {
                    self.forward_draft = self.state.forward_file.clone();
                }
                self.checked = false;
            }
            Action::SelectCase { index } => {
                if *index >= task.cases.len() {
                    return Err("Choose one of the supplied traffic cases.".into());
                }
                self.selected_case = *index;
            }
            Action::Trace => {
                let c = &task.cases[self.selected_case];
                let line = match self.state.evaluate(&c.origin, &c.header) {
                    Ok(t) => format!(
                        "[Read-only policy trace] {}: {}\n",
                        c.label,
                        trace_detail(&self.state, &t, &c.header)
                    ),
                    Err(e) => format!("[{:?}] {}\n", e.class, e.message),
                };
                self.log(&line);
            }
            Action::Check => self.checked = true,
        }
        self.bound(task)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    File {
        file: File,
    },
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
        text: String,
    },
    Save,
    ReloadDraft,
    SelectCase {
        index: usize,
    },
    Trace,
    Check,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
                "Router lesson needs a stable source ID, bounded tasks and a worked example."
                    .into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.initial.host.profile.id == self.example.initial.host.profile.id
            || self.assignment.initial.host.profile == self.example.initial.host.profile
            || self.assignment.initial.host.configuration.interfaces
                == self.example.initial.host.configuration.interfaces
            || self.assignment.cases == self.example.cases
        {
            return Err("Worked router facts must be separate from assignment facts.".into());
        }
        let mut example = Scope::new(&self.example)?;
        for a in &self.worked {
            example.reference(&self.example, a)?;
        }
        if !example.checked || !self.example.checks(&example).iter().all(|c| c.passed) {
            return Err("Worked router actions do not reach all required outcomes.".into());
        }
        let initial = Scope::new(&self.assignment)?;
        if self.assignment.checks(&initial).iter().all(|c| c.passed) {
            return Err("The independent assignment must start with a genuine repair, not only an unchecked result.".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        if s.source_id != self.source_id || !within(s, 1024 * 1024) {
            return Err(
                "Router progress source/version bounds differ; it was retained for recovery."
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    /// Structural gate only; authored context is additionally required for grading and saving.
    pub fn shape(&self) -> Result<(), String> {
        if !id(&self.source_id) || !within(self, 1024 * 1024) {
            return Err("Router session exceeds its source or size bounds.".into());
        }
        for scope in [&self.assignment, &self.practice] {
            scope.state.validate().map_err(|e| e.message)?;
            if !within(scope, 512 * 1024)
                || scope.input.len() > 2048
                || scope.output.len() > MAX_CONSOLE
                || scope.filter_draft.len() > 32768
                || scope.forward_draft.len() > 4096
                || scope.selected_case >= 16
            {
                return Err(
                    "Router workspace exceeds its input, draft or traffic-case bounds.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        let (t, s) = if practice {
            (&d.example, &mut next.practice)
        } else {
            (&d.assignment, &mut next.assignment)
        };
        s.apply(t, a)?;
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

fn verdict_text(v: Verdict) -> &'static str {
    if v == Verdict::Accept {
        "allow"
    } else {
        "block"
    }
}
pub fn packet_text(p: &crate::linux_filter::Packet) -> String {
    let endpoint = |ip: std::net::Ipv4Addr, port: Option<u16>| {
        port.map_or_else(|| ip.to_string(), |port| format!("{ip}:{port}"))
    };
    let protocol = match p.protocol {
        crate::linux_filter::Protocol::Tcp => "TCP",
        crate::linux_filter::Protocol::Udp => "UDP",
        crate::linux_filter::Protocol::Icmp => "ICMP",
    };
    format!(
        "{protocol} {} -> {} via {} -> {}",
        endpoint(p.source, p.source_port),
        endpoint(p.destination, p.destination_port),
        p.ingress.as_deref().unwrap_or("router"),
        p.egress.as_deref().unwrap_or("router")
    )
}
pub fn trace_detail(state: &admin::State, trace: &path::Trace, header: &path::Header) -> String {
    match trace.gate {
        path::Gate::Allowed => {
            "The supplied path and filter permit this read-only test header.".into()
        }
        path::Gate::IngressDown => {
            "The incoming interface is down or has no supplied carrier.".into()
        }
        path::Gate::NoRoute => format!("No configured route matches {}.", header.destination),
        path::Gate::EgressDown => format!(
            "The selected output interface {} is down or has no supplied carrier.",
            trace
                .route
                .as_ref()
                .map_or("unknown", |r| r.device.as_str())
        ),
        path::Gate::ForwardingDisabled => {
            "IPv4 forwarding is off; this packet needs to travel through the router.".into()
        }
        path::Gate::FilterDrop => format!(
            "{} {} blocks this header.",
            trace.chain.map_or("Filter", |c| c.name()),
            trace.rule.map_or("policy".into(), |n| format!("rule {n}"))
        ),
        path::Gate::PeerUnavailable => state.host.probe(header.destination).reason,
    }
}
